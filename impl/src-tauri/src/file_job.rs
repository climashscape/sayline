use std::path::Path;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

use crate::asr::{self, AsrConfig};
use crate::audio::{AudioMessage, CaptureStats, CHUNK_SAMPLES};
use crate::credentials;
use crate::events::{emit, SaylineEvent};
use crate::file_decode;
use crate::recording;
use crate::state::{AppState, FileJob};

pub fn start(app: &AppHandle, path: String) -> Result<(), String> {
    let file = Path::new(&path);
    if !file.is_file() {
        return Err("音频文件不存在。".to_string());
    }
    let state = app.state::<AppState>();
    if recording::is_recording(app) {
        return Err("正在录音，请先停止录音。".to_string());
    }
    {
        let guard = state.file_job.lock().map_err(|_| "文件任务锁失败".to_string())?;
        if guard.is_some() {
            return Err("已有文件正在转写。".to_string());
        }
    }

    let settings = state
        .settings
        .lock()
        .map(|guard| guard.clone())
        .map_err(|_| "配置锁失败".to_string())?;
    let token = credentials::load_token()?;
    let title = file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("audio")
        .to_string();
    let cancel = CancellationToken::new();
    {
        let mut guard = state.file_job.lock().map_err(|_| "文件任务锁失败".to_string())?;
        *guard = Some(FileJob {
            cancel: cancel.clone(),
        });
    }

    let app = app.clone();
    let source = "file".to_string();
    tauri::async_runtime::spawn(async move {
        recording::emit_state(&app, "requesting", &source);
        let decoded = tokio::task::spawn_blocking(move || file_decode::decode_audio(&path)).await;
        let samples = match decoded {
            Ok(Ok(value)) => value,
            Ok(Err(error)) => {
                emit(
                    &app,
                    SaylineEvent::Error {
                        message: format!("无法解码音频: {error}"),
                        source: source.clone(),
                        code: "file_decode".to_string(),
                    },
                );
                recording::emit_state(&app, "idle", &source);
                session_finished(&app);
                return;
            }
            Err(error) => {
                emit(
                    &app,
                    SaylineEvent::Error {
                        message: format!("解码任务失败: {error}"),
                        source: source.clone(),
                        code: "file_decode".to_string(),
                    },
                );
                recording::emit_state(&app, "idle", &source);
                session_finished(&app);
                return;
            }
        };

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let config = AsrConfig {
            endpoint: settings.endpoint_origin.clone(),
            token,
            language: settings.language.clone(),
            source: source.clone(),
            title: Some(title),
            client: app.state::<AppState>().http.clone(),
        };
        let attempt_id = uuid::Uuid::new_v4().to_string();
        let asr_task = tauri::async_runtime::spawn(asr::run_session(
            app.clone(),
            attempt_id,
            config,
            rx,
            cancel.clone(),
        ));

        let total_samples = samples.len();
        let mut sent_samples = 0usize;
        let speed = settings.file_speed.max(0.5);
        for chunk in samples.chunks(CHUNK_SAMPLES) {
            if cancel.is_cancelled() {
                break;
            }
            let mut bytes = Vec::with_capacity(chunk.len() * 2);
            let mut peak = 0.0f32;
            for sample in chunk {
                let absolute = sample.abs();
                if absolute > peak {
                    peak = absolute;
                }
                let value = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            let _ = tx.send(AudioMessage::Chunk { bytes, peak });
            sent_samples += chunk.len();
            let percent = if total_samples == 0 {
                0.0
            } else {
                (sent_samples as f32 / total_samples as f32) * 100.0
            };
            emit(
                &app,
                SaylineEvent::Progress {
                    percent,
                    source: source.clone(),
                },
            );
            let delay = (100.0f32 / speed).max(10.0) as u64;
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }

        let stats = CaptureStats {
            peak: 1.0,
            samples: sent_samples as u64,
            bytes: sent_samples as u64 * 2,
        };
        let _ = tx.send(AudioMessage::End(stats));
        let _ = asr_task.await;
    });
    Ok(())
}

pub fn cancel(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let job = state
        .file_job
        .lock()
        .map_err(|_| "文件任务锁失败".to_string())?
        .take();
    if let Some(job) = job {
        job.cancel.cancel();
    }
    recording::emit_state(app, "idle", "file");
    Ok(())
}

pub fn session_finished(app: &AppHandle) {
    let state = app.state::<AppState>();
    if let Ok(mut guard) = state.file_job.lock() {
        *guard = None;
    };
}
