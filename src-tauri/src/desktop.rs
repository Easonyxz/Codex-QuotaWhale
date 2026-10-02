use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};
use tauri::{Manager, PhysicalPosition};
use windows_sys::Win32::Graphics::Gdi::{CreateRectRgn, CombineRgn, DeleteObject, SetWindowRgn, RGN_OR};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position { x: i32, y: i32 }

#[derive(Clone, Copy)]
struct Area { x: i32, y: i32, width: i32, height: i32 }

fn fit(position: Position, width: i32, height: i32, areas: &[Area]) -> Position {
    // Choose the nearest usable screen, including screens with negative coordinates.
    areas.iter().map(|area| {
        let x = position.x.clamp(area.x, area.x + (area.width - width).max(0));
        let y = position.y.clamp(area.y, area.y + (area.height - height).max(0));
        let distance = (i64::from(x) - i64::from(position.x)).pow(2)
            + (i64::from(y) - i64::from(position.y)).pow(2);
        (distance, Position { x, y })
    }).min_by_key(|item| item.0).map(|item| item.1).unwrap_or(position)
}

pub fn keep_visible(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let pos = window.outer_position()?;
    let size = window.outer_size()?;
    let areas: Vec<_> = window.available_monitors()?.iter().map(|monitor| {
        let area = monitor.work_area();
        Area { x: area.position.x, y: area.position.y, width: area.size.width as i32, height: area.size.height as i32 }
    }).collect();
    let current = Position { x: pos.x, y: pos.y };
    let visible = fit(current, size.width as i32, size.height as i32, &areas);
    if current != visible { window.set_position(PhysicalPosition::new(visible.x, visible.y))?; }
    Ok(())
}

fn position_path(app: &tauri::AppHandle) -> tauri::Result<PathBuf> {
    Ok(app.path().app_config_dir()?.join("position.json"))
}

pub fn save_position(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else { return };
    if let (Ok(pos), Ok(path)) = (window.outer_position(), position_path(app)) {
        let position = Position { x: pos.x, y: pos.y };
        if std::fs::create_dir_all(path.parent().unwrap()).is_ok() {
            let _ = std::fs::write(path, serde_json::to_vec(&position).unwrap());
        }
    }
}

pub fn setup(app: &tauri::App) -> tauri::Result<()> {
    let window = app.get_webview_window("main").unwrap();
    if let Some(pos) = std::fs::read(position_path(app.handle())?).ok()
        .and_then(|bytes| serde_json::from_slice::<Position>(&bytes).ok()) {
        window.set_position(PhysicalPosition::new(pos.x, pos.y))?;
    }
    keep_visible(&window)?;
    let handle = app.handle().clone();
    // Check only once a second, save only settled changes. Also recovers from screen removal.
    std::thread::spawn(move || {
        let mut previous = None;
        let mut saved = None;
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let Some(window) = handle.get_webview_window("main") else { break };
            let Ok(pos) = window.outer_position() else { continue };
            let current = Position { x: pos.x, y: pos.y };
            if previous == Some(current) {
                let _ = keep_visible(&window);
                if saved != Some(current) { save_position(&handle); saved = Some(current); }
            }
            previous = Some(current);
        }
    });
    Ok(())
}

#[derive(Deserialize)]
pub struct HitRect { x: f64, y: f64, width: f64, height: f64 }

#[tauri::command]
pub fn set_hit_regions(window: tauri::WebviewWindow, rects: Vec<HitRect>) -> Result<(), String> {
    let dpi = window.scale_factor().map_err(|e| e.to_string())?;
    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    // The OS excludes all other parts of this window from drawing and mouse hit testing.
    unsafe {
        let region = CreateRectRgn(0, 0, 0, 0);
        if region.is_null() { return Err("无法创建窗口区域".into()); }
        for rect in rects {
            let part = CreateRectRgn((rect.x * dpi).floor() as i32, (rect.y * dpi).floor() as i32,
                ((rect.x + rect.width) * dpi).ceil() as i32, ((rect.y + rect.height) * dpi).ceil() as i32);
            if part.is_null() { DeleteObject(region); return Err("无法创建窗口区域".into()); }
            let result = CombineRgn(region, region, part, RGN_OR);
            DeleteObject(part);
            if result == 0 { DeleteObject(region); return Err("无法合并窗口区域".into()); }
        }
        if SetWindowRgn(hwnd.0 as _, region, 1) == 0 {
            DeleteObject(region);
            return Err("无法更新窗口区域".into());
        }
        // Windows owns the region after a successful SetWindowRgn call.
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_position_on_negative_coordinate_monitor() {
        let areas = [Area { x: -1920, y: 0, width: 1920, height: 1040 }];
        let pos = Position { x: -1000, y: 500 };
        assert_eq!(fit(pos, 250, 370, &areas), pos);
    }
    #[test]
    fn recovers_removed_monitor_and_respects_work_area() {
        let areas = [Area { x: 0, y: 0, width: 1920, height: 1040 }];
        assert_eq!(fit(Position { x: 2500, y: 900 }, 250, 370, &areas), Position { x: 1670, y: 670 });
        assert_eq!(fit(Position { x: -1800, y: 10 }, 250, 370, &areas), Position { x: 0, y: 10 });
    }
}
