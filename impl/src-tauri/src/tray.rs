use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

static TRAY_AVAILABLE: AtomicBool = AtomicBool::new(false);

pub fn available() -> bool {
    TRAY_AVAILABLE.load(Ordering::Relaxed)
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    if std::env::var_os("SAYLINE_DISABLE_TRAY").is_some() {
        return Ok(());
    }
    let show = MenuItem::with_id(app, "show", "显示 Sayline", true, None::<&str>)?;
    let record = MenuItem::with_id(app, "record", "开始 / 停止录音", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &record, &quit])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("default window icon".to_string()))?;

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main(app),
            "record" => {
                if let Err(error) = crate::recording::toggle(app) {
                    tracing::warn!("tray toggle failed: {error}");
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;
    app.manage(tray);
    TRAY_AVAILABLE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
