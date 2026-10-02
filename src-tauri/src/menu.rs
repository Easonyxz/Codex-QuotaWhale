use tauri::{menu::{Menu, MenuItem, CheckMenuItem, Submenu}, tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState}, Manager, Emitter};
use tauri_plugin_autostart::ManagerExt;
use std::sync::Mutex;

pub struct Scale(pub Mutex<f64>);

#[tauri::command]
pub fn get_scale(state: tauri::State<'_, Scale>) -> f64 { *state.0.lock().unwrap() }

fn apply_scale(app: &tauri::AppHandle, scale: f64) -> tauri::Result<()> {
    let window = app.get_webview_window("main").unwrap();
    let position = window.outer_position()?;
    let size = window.outer_size()?;
    let dpi = window.scale_factor()?;
    window.set_size(tauri::LogicalSize::new(250.0 * scale, 370.0 * scale))?;
    window.set_position(tauri::PhysicalPosition::new(
        position.x + size.width as i32 - (250.0 * scale * dpi).round() as i32,
        position.y + size.height as i32 - (370.0 * scale * dpi).round() as i32,
    ))?;
    *app.state::<Scale>().0.lock().unwrap() = scale;
    crate::desktop::keep_visible(&window)?;
    window.emit("scale-changed", scale)?;
    Ok(())
}

pub struct PetMenu(pub Menu<tauri::Wry>);

#[tauri::command]
pub fn show_menu(window: tauri::WebviewWindow, state: tauri::State<'_, PetMenu>) -> Result<(), String> {
    window.popup_menu(&state.0).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_pet(window: tauri::WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

pub fn setup(app: &tauri::App) -> tauri::Result<()> {
    let scales = [("scale75", "75%", 0.75), ("scale100", "100%", 1.0), ("scale125", "125%", 1.25), ("scale150", "150%", 1.5)];
    let settings_path = app.path().app_config_dir()?.join("scale.json");
    let scale = std::fs::read(&settings_path).ok().and_then(|bytes| serde_json::from_slice::<f64>(&bytes).ok())
        .filter(|value| scales.iter().any(|(_, _, s)| s == value)).unwrap_or(1.0);
    app.manage(Scale(Mutex::new(scale)));
    let scale_items: Vec<_> = scales.iter().map(|(id, text, s)| CheckMenuItem::with_id(app, *id, *text, true, *s == scale, None::<&str>)).collect::<tauri::Result<_>>()?;
    let scale_menu = Submenu::with_items(app, "缩放", true, &[&scale_items[0], &scale_items[1], &scale_items[2], &scale_items[3]])?;
    let show = MenuItem::with_id(app, "show", "显示桌宠", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "隐藏桌宠", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "刷新额度", true, None::<&str>)?;
    let startup = CheckMenuItem::with_id(app, "startup", "开机自启动", true,
        app.autolaunch().is_enabled().unwrap_or(false), None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &refresh, &scale_menu, &startup, &quit])?;
    app.manage(PetMenu(menu.clone()));
    app.on_menu_event(move |app, event| {
        let window = app.get_webview_window("main").unwrap();
        if let Some((_, _, scale)) = scales.iter().find(|(id, _, _)| *id == event.id.as_ref()) {
            if apply_scale(app, *scale).is_ok() {
                let saved = std::fs::create_dir_all(settings_path.parent().unwrap())
                    .and_then(|_| std::fs::write(&settings_path, scale.to_string()));
                if saved.is_err() { let _ = window.emit("settings-error", "缩放已应用，但未能保存设置"); }
            } else { let _ = window.emit("settings-error", "缩放失败，请重试"); }
            let current = *app.state::<Scale>().0.lock().unwrap();
            for (item, (_, _, s)) in scale_items.iter().zip(scales.iter()) { let _ = item.set_checked(*s == current); }
            return;
        }
        match event.id.as_ref() {
            "show" => { let _ = window.show(); let _ = window.set_focus(); },
            "hide" => { let _ = window.hide(); },
            "refresh" => { let _ = window.emit("refresh-requested", ()); },
            "startup" => {
                let manager = app.autolaunch();
                let result = manager.is_enabled().and_then(|enabled| if enabled { manager.disable() } else { manager.enable() });
                let _ = startup.set_checked(manager.is_enabled().unwrap_or(false));
                if result.is_err() {
                    let _ = window.show();
                    let _ = window.emit("settings-error", "开机启动设置失败，请稍后重试");
                }
            },
            "quit" => { crate::desktop::save_position(app); app.exit(0); },
            _ => {}
        }
    });
    TrayIconBuilder::with_id("pet")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Codex-QuotaWhale · 右键打开菜单")
        .menu(&menu).show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show(); let _ = window.set_focus();
                }
            }
        }).build(app)?;
    apply_scale(app.handle(), scale)?;
    Ok(())
}

