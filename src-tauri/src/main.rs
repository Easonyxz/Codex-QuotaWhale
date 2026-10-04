#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod quota;
mod menu;
mod desktop;
use std::time::Duration;
use tauri::Manager;

#[tauri::command]
async fn get_quota(client: tauri::State<'_, reqwest::Client>) -> Result<quota::Snapshot, String> {
    quota::fetch(&client).await
}

#[tauri::command]
async fn get_reset_credits(client: tauri::State<'_, reqwest::Client>) -> Result<quota::ResetCredits, String> {
    quota::fetch_credits(&client).await
}

fn main() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Codex-QuotaWhale/0.1.1")
        .build().expect("HTTP client initialization failed");
    // Diagnostic mode uses the same request/parser as the UI and prints only quota fields.
    if std::env::args().any(|arg| arg == "--check-quota") {
        let result = tauri::async_runtime::block_on(quota::fetch(&client));
        let output = match result {
            Ok(snapshot) => serde_json::json!({"ok": true, "quota": snapshot}),
            Err(message) => serde_json::json!({"ok": false, "error": message}),
        };
        if let Some(path) = std::env::args().skip_while(|arg| arg != "--check-quota").nth(1) {
            std::fs::write(path, serde_json::to_vec_pretty(&output).unwrap()).unwrap();
        } else { println!("{output}"); }
        std::process::exit(if output["ok"] == true { 0 } else { 1 });
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = desktop::keep_visible(&window);
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .manage(client)
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            if let Some(monitor) = window.primary_monitor()? {
                let area = monitor.work_area();
                let scale = monitor.scale_factor();
                window.set_position(tauri::PhysicalPosition::new(
                    area.position.x + area.size.width as i32 - (270.0 * scale) as i32,
                    area.position.y + area.size.height as i32 - (385.0 * scale) as i32,
                ))?;
            }
            menu::setup(app)?;
            desktop::setup(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_quota, get_reset_credits, menu::show_menu, menu::hide_pet, menu::get_scale, desktop::set_hit_regions])
        .run(tauri::generate_context!())
        .expect("Codex Pet failed to start");
}
