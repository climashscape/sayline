use std::time::Duration;
use tauri::{AppHandle, Manager, PhysicalPosition};
use tokio_util::sync::CancellationToken;

use crate::asr::{self, AsrConfig};
use crate::audio;
use crate::credentials;
use crate::events::{emit, SaylineEvent};
use crate::state::{AppState, RecordingHandle};

pub fn emit_state(app: &AppHandle, state: &str, source: &str) {
    emit(
        app,
        SaylineEvent::State {
            state: state.to_string(),
            source: source.to_string(),
        },
    );
    if source == "mic" {
        update_overlay(app, state);
    }
}

fn update_overlay(app: &AppHandle, state: &str) {
    let Some(window) = app.get_webview_window("overlay") else {
        return;
    };
    if state == "idle" {
        let window = window.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(900)).await;
            let _ = window.hide();
        });
        return;
    }
    position_overlay(&window);
    let _ = window.show();
}

fn position_overlay(window: &tauri::WebviewWindow) {
    let Ok(Some(monitor)) = window.current_monitor() else {
        return;
    };
    let Ok(size) = window.outer_size() else {
        return;
    };
    let screen = monitor.size();
    let x = screen.width.saturating_sub(size.width) / 2;
    let y = screen.height.saturating_sub(size.height + 110);
    let _ = window.set_position(PhysicalPosition::new(x as i32, y as i32));
}

pub fn start(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let settings = state
        .settings
        .lock()
        .map(|guard| guard.clone())
        .map_err(|_| "配置锁失败".to_string())?;
    {
        let guard = state.recording.lock().map_err(|_| "录音状态锁失败".to_string())?;
        if guard.is_some() {
            return Err("正在录音，请先停止当前录音。".to_string());
        }
    }

    let token = credentials::load_token()?;
    let attempt_id = uuid::Uuid::new_v4().to_string();
    let (audio_tx, audio_rx) = tokio::sync::mpsc::unbounded_channel();
    let (stop_tx, stop_rx) = std::sync::mpsc::channel();
    let cancel = CancellationToken::new();

    {
        let mut guard = state.recording.lock().map_err(|_| "录音状态锁失败".to_string())?;
        *guard = Some(RecordingHandle {
            attempt_id: attempt_id.clone(),
            stop_tx,
            cancel: cancel.clone(),
        });
    }

    let _capture = audio::spawn_capture(app.clone(), audio_tx, stop_rx);
    let config = AsrConfig {
        endpoint: settings.endpoint_origin.clone(),
        token,
        language: settings.language.clone(),
        source: "mic".to_string(),
        title: None,
        client: state.http.clone(),
    };
    emit_state(app, "requesting", "mic");
    tauri::async_runtime::spawn(asr::run_session(
        app.clone(),
        attempt_id,
        config,
        audio_rx,
        cancel,
    ));
    Ok(())
}

pub fn stop(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let guard = state.recording.lock().map_err(|_| "录音状态锁失败".to_string())?;
    let Some(handle) = guard.as_ref() else {
        return Ok(());
    };
    let _ = handle.stop_tx.send(());
    emit_state(app, "finishing", "mic");
    Ok(())
}

pub fn cancel(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let handle = state
        .recording
        .lock()
        .map_err(|_| "录音状态锁失败".to_string())?
        .take();
    if let Some(handle) = handle {
        handle.cancel.cancel();
        let _ = handle.stop_tx.send(());
    }
    emit_state(app, "idle", "mic");
    Ok(())
}

pub fn toggle(app: &AppHandle) -> Result<(), String> {
    let recording = app
        .state::<AppState>()
        .recording
        .lock()
        .map(|guard| guard.is_some())
        .unwrap_or(false);
    if recording {
        stop(app)
    } else {
        start(app)
    }
}

pub fn session_finished(app: &AppHandle, attempt_id: &str) {
    let state = app.state::<AppState>();
    if let Ok(mut guard) = state.recording.lock() {
        if guard
            .as_ref()
            .map(|handle| handle.attempt_id == attempt_id)
            .unwrap_or(false)
        {
            *guard = None;
        }
    };
}

pub fn is_recording(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .recording
        .lock()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}
