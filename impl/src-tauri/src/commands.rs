use std::str::FromStr;
use tauri::{AppHandle, Manager};

use crate::credentials::{self, ZCodeStatus};
use crate::history::HistoryItem;
use crate::settings::{self, Settings};
use crate::state::AppState;
use crate::{file_job, paste, recording, tray};

#[tauri::command]
pub fn start_recording(app: AppHandle) -> Result<(), String> {
    recording::start(&app)
}

#[tauri::command]
pub fn stop_recording(app: AppHandle) -> Result<(), String> {
    recording::stop(&app)
}

#[tauri::command]
pub fn cancel_recording(app: AppHandle) -> Result<(), String> {
    recording::cancel(&app)
}

#[tauri::command]
pub fn recording_active(app: AppHandle) -> bool {
    recording::is_recording(&app)
}

#[tauri::command]
pub async fn zcode_status() -> ZCodeStatus {
    tauri::async_runtime::spawn_blocking(credentials::status)
        .await
        .unwrap_or(ZCodeStatus {
            connected: false,
            expires_at: None,
            message: "无法读取 ZCode 登录态".to_string(),
        })
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    app.state::<AppState>()
        .settings
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, mut value: Settings) -> Result<(), String> {
    if global_hotkey::hotkey::HotKey::from_str(&value.hotkey).is_err() {
        return Err("热键格式无效。示例：Ctrl+Alt+Space".to_string());
    }
    if value.language != "zh-CN" && value.language != "en-US" {
        value.language = "zh-CN".to_string();
    }
    if value.theme != "system" && value.theme != "light" && value.theme != "dark" {
        value.theme = "system".to_string();
    }
    value.endpoint_origin = value.endpoint_origin.trim().trim_end_matches('/').to_string();
    if value.endpoint_origin.is_empty() {
        value.endpoint_origin = Settings::default().endpoint_origin;
    }
    value.paste_delay_ms = value.paste_delay_ms.clamp(0, 2000);
    value.file_speed = value.file_speed.clamp(0.5, 4.0);
    value.replacements.retain(|rule| !rule.from.is_empty());
    settings::save(&value)?;
    {
        let state = app.state::<AppState>();
        {
            let mut guard = state.settings.lock().map_err(|_| "配置锁失败".to_string())?;
            *guard = value.clone();
        }
        let sender = state
            .hotkey_tx
            .lock()
            .map_err(|_| "热键锁失败".to_string())?
            .clone();
        if let Some(sender) = sender {
            let _ = sender.send(crate::hotkey::HotkeyCommand::Set(value.hotkey));
        }
    }
    Ok(())
}

#[tauri::command]
pub fn list_history(app: AppHandle, query: Option<String>) -> Result<Vec<HistoryItem>, String> {
    app.state::<AppState>().history.list(query.as_deref(), 500)
}

#[tauri::command]
pub fn delete_history(app: AppHandle, id: i64) -> Result<(), String> {
    app.state::<AppState>().history.delete(id)
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<(), String> {
    app.state::<AppState>().history.clear()
}

#[tauri::command]
pub fn export_history(app: AppHandle, format: String) -> Result<String, String> {
    let body = app.state::<AppState>().history.export(&format)?;
    let extension = if format == "json" { "json" } else { "md" };
    let directory = dirs::download_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "找不到可写目录".to_string())?;
    let name = format!(
        "sayline-history-{}.{}",
        chrono::Local::now().format("%Y%m%d-%H%M%S"),
        extension
    );
    let path = directory.join(name);
    std::fs::write(&path, body).map_err(|error| format!("写入导出文件失败: {error}"))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn copy_text(text: String) -> Result<(), String> {
    paste::copy_text(&text)
}

#[tauri::command]
pub fn transcribe_file(app: AppHandle, path: String) -> Result<(), String> {
    file_job::start(&app, path)
}

#[tauri::command]
pub fn cancel_file(app: AppHandle) -> Result<(), String> {
    file_job::cancel(&app)
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    tray::show_main(&app);
}
