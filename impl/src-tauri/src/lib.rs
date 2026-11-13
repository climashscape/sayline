mod asr;
mod audio;
mod capture;
mod commands;
mod credentials;
mod events;
mod file_decode;
mod file_job;
mod history;
mod hotkey;
mod paste;
mod recording;
mod settings;
mod state;
mod tray;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("sayline=info")),
        )
        .try_init();

    let settings = settings::load();
    let history = history::History::open().expect("failed to open Sayline history database");
    let app_state = state::AppState::new(settings, history);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .setup(|app| {
            let handle = app.handle().clone();
            let sender = hotkey::spawn(handle.clone());
            {
                let state = handle.state::<state::AppState>();
                if let Ok(mut guard) = state.hotkey_tx.lock() {
                    *guard = Some(sender.clone());
                }
                let hotkey = state
                    .settings
                    .lock()
                    .map(|guard| guard.hotkey.clone())
                    .unwrap_or_else(|_| settings::Settings::default().hotkey);
                let _ = sender.send(hotkey::HotkeyCommand::Set(hotkey));
            }
            if let Err(error) = tray::setup(&handle) {
                tracing::warn!("tray setup failed, continuing without tray: {error}");
            }

            if let Some(window) = handle.get_webview_window("main") {
                let close_target = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // Without a tray there is no way back to the window, so let
                        // the close request terminate the process instead of hiding it.
                        if !tray::available() {
                            return;
                        }
                        api.prevent_close();
                        let _ = close_target.hide();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording,
            commands::cancel_recording,
            commands::recording_active,
            commands::zcode_status,
            commands::get_settings,
            commands::save_settings,
            commands::list_history,
            commands::delete_history,
            commands::clear_history,
            commands::export_history,
            commands::copy_text,
            commands::transcribe_file,
            commands::cancel_file,
            commands::show_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sayline");
}
