// Authentication/request approach adapted from quota-float (MIT); see licenses/.
use serde::Serialize;
use serde_json::Value;
use std::{path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Window { remaining: f64, resets_at: Option<u64> }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot { short: Option<Window>, weekly: Option<Window>, updated_at: u64 }

fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }

fn parse_window(value: &Value, timestamp: u64) -> Option<Window> {
    let used = value.get("used_percent")?.as_f64()?;
    Some(Window {
        remaining: (100.0 - used).clamp(0.0, 100.0),
        resets_at: value.get("reset_at").and_then(Value::as_u64)
            .or_else(|| value.get("reset_after_seconds").and_then(Value::as_u64).map(|s| timestamp.saturating_add(s))),
    })
}

fn parse(value: &Value, timestamp: u64) -> Result<Snapshot, String> {
    let limits = value.get("rate_limit").ok_or("额度响应缺少 rate_limit")?;
    let mut snapshot = Snapshot { short: None, weekly: None, updated_at: timestamp };
    for key in ["primary_window", "secondary_window"] {
        if let Some(window) = limits.get(key) {
            // Match by duration: some subscriptions return only one window.
            match window.get("limit_window_seconds").and_then(Value::as_u64) {
                Some(18_000) => snapshot.short = parse_window(window, timestamp),
                Some(604_800) => snapshot.weekly = parse_window(window, timestamp),
                _ => {}
            }
        }
    }
    if snapshot.short.is_none() && snapshot.weekly.is_none() {
        return Err("服务未返回可识别的 5h / Weekly 额度".into());
    }
    Ok(snapshot)
}

pub async fn fetch(client: &reqwest::Client) -> Result<Snapshot, String> {
    let home = std::env::var_os("CODEX_HOME").map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join(".codex")))
        .ok_or("找不到 Codex 登录目录")?;
    let path = home.join("auth.json");
    let metadata = std::fs::metadata(&path).map_err(|_| "请先登录本机 Codex")?;
    if metadata.len() > 262_144 { return Err("Codex 登录文件格式异常".into()); }
    let raw = std::fs::read(path).map_err(|_| "无法读取 Codex 登录信息")?;
    let auth: Value = serde_json::from_slice(&raw).map_err(|_| "Codex 登录文件格式异常")?;
    let tokens = auth.get("tokens").unwrap_or(&auth);
    let token = tokens.get("access_token").and_then(Value::as_str)
        .ok_or("请使用 ChatGPT 账户登录 Codex")?;
    let mut request = client.get("https://chatgpt.com/backend-api/wham/usage")
        .bearer_auth(token).header("Accept", "application/json")
        .header("originator", "Codex Desktop").header("OAI-Product-Sku", "CODEX");
    if let Some(id) = tokens.get("account_id").and_then(Value::as_str) {
        request = request.header("ChatGPT-Account-Id", id);
    }
    let mut response = request.send().await.map_err(|_| "网络连接失败，稍后自动重试")?;
    match response.status().as_u16() {
        200 => {},
        401 => return Err("登录已过期，请在 Codex 中重新登录".into()),
        403 => return Err("额度服务拒绝访问，请检查登录或网络".into()),
        429 => return Err("请求过于频繁，稍后自动重试".into()),
        _ => return Err("额度服务暂时不可用".into()),
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "读取额度响应失败")? {
        if bytes.len() + chunk.len() > 1_048_576 { return Err("额度响应过大".into()); }
        bytes.extend_from_slice(&chunk);
    }
    let value = serde_json::from_slice(&bytes).map_err(|_| "额度服务响应格式异常")?;
    parse(&value, now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn maps_duration_and_preserves_small_percent() {
        let data = json!({"rate_limit": {
            "primary_window": {"used_percent": 0.5, "limit_window_seconds": 604800, "reset_at": 2000},
            "secondary_window": {"used_percent": 100, "limit_window_seconds": 18000, "reset_after_seconds": 60}
        }});
        let result = parse(&data, 1000).unwrap();
        assert_eq!(result.weekly.unwrap().remaining, 99.5);
        let short = result.short.unwrap();
        assert_eq!(short.remaining, 0.0);
        assert_eq!(short.resets_at, Some(1060));
    }
    #[test]
    fn absent_window_is_not_zero_quota() {
        let result = parse(&json!({"rate_limit": {"primary_window": {
            "used_percent": 23, "limit_window_seconds": 604800
        }}}), 1000).unwrap();
        assert!(result.short.is_none());
        assert_eq!(result.weekly.unwrap().remaining, 77.0);
        assert!(parse(&json!({"rate_limit": {}}), 1000).is_err());
    }
}
