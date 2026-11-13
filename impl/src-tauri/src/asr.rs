use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::collections::VecDeque;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

use crate::audio::{AudioMessage, CaptureStats};
use crate::events::{emit, SaylineEvent};
use crate::history::NewTranscript;
use crate::state::AppState;
use crate::{recording, settings};

const TICKET_TIMEOUT: Duration = Duration::from_secs(10);
const START_TIMEOUT: Duration = Duration::from_secs(8);
const FINISH_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_PENDING_CHUNKS: usize = 200;

pub struct AsrConfig {
    pub endpoint: String,
    pub token: String,
    pub language: String,
    pub source: String,
    pub title: Option<String>,
    pub client: reqwest::Client,
}

#[derive(Debug, Clone)]
struct AsrFailure {
    code: String,
    message: String,
}

impl AsrFailure {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

struct Ticket {
    ticket: String,
    ws_path: String,
}

pub async fn run_session(
    app: AppHandle,
    attempt_id: String,
    config: AsrConfig,
    mut audio_rx: UnboundedReceiver<AudioMessage>,
    cancel: CancellationToken,
) {
    let source = config.source.clone();
    let is_mic = source == "mic";
    recording::emit_state(&app, "requesting", &source);

    let ticket = match request_ticket(&config).await {
        Ok(ticket) => ticket,
        Err(failure) => {
            fail(&app, &source, failure);
            finish(&app, &attempt_id, &source);
            return;
        }
    };

    let url = ws_url(&config.endpoint, &ticket.ws_path, &ticket.ticket);
    let connection = tokio::select! {
        _ = cancel.cancelled() => {
            finish(&app, &attempt_id, &source);
            return;
        }
        result = tokio_tungstenite::connect_async(&url) => result,
    };
    let (socket, _) = match connection {
        Ok(value) => value,
        Err(error) => {
            fail(&app, &source, AsrFailure::new("network", format!("无法连接语音识别服务: {error}")));
            finish(&app, &attempt_id, &source);
            return;
        }
    };
    let (mut write, mut read) = socket.split();

    let start_message = serde_json::json!({
        "type": "start",
        "audio": { "encoding": "pcm_s16le", "sample_rate": 16_000, "channels": 1 },
        "language": config.language,
        "options": { "punctuation": true, "itn": true }
    });
    if let Err(error) = write
        .send(Message::Text(start_message.to_string().into()))
        .await
    {
        fail(&app, &source, AsrFailure::new("network", format!("发送识别请求失败: {error}")));
        finish(&app, &attempt_id, &source);
        return;
    }

    let mut started = false;
    let mut stopped: Option<CaptureStats> = None;
    let mut pending: VecDeque<Vec<u8>> = VecDeque::new();
    let mut terminal_text: Option<String> = None;
    let mut start_timer = Box::pin(tokio::time::sleep(START_TIMEOUT));

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                let _ = write.send(Message::Text("{\"type\":\"cancel\"}".into())).await;
                finish(&app, &attempt_id, &source);
                return;
            }
            _ = &mut start_timer, if !started => {
                fail(&app, &source, AsrFailure::new("unavailable", "语音识别连接超时，请重试。"));
                let _ = write.send(Message::Text("{\"type\":\"cancel\"}".into())).await;
                finish(&app, &attempt_id, &source);
                return;
            }
            incoming = audio_rx.recv() => {
                match incoming {
                    Some(AudioMessage::Chunk { bytes, peak }) => {
                        emit(&app, SaylineEvent::Level { level: level_of(peak), source: source.clone() });
                        if started {
                            if write.send(Message::Binary(bytes.into())).await.is_err() {
                                fail(&app, &source, AsrFailure::new("network", "语音识别连接已断开。"));
                                finish(&app, &attempt_id, &source);
                                return;
                            }
                        } else if pending.len() < MAX_PENDING_CHUNKS {
                            pending.push_back(bytes);
                        }
                    }
                    Some(AudioMessage::End(stats)) => {
                        stopped = Some(stats);
                        if started {
                            break;
                        }
                    }
                    Some(AudioMessage::Error(message)) => {
                        fail(&app, &source, AsrFailure::new("device", message));
                        let _ = write.send(Message::Text("{\"type\":\"cancel\"}".into())).await;
                        finish(&app, &attempt_id, &source);
                        return;
                    }
                    None => {
                        if started {
                            stopped = Some(CaptureStats::default());
                            break;
                        }
                    }
                }
            }
            incoming = read.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match server_type(&text).as_str() {
                            "started" => {
                                started = true;
                                tracing::info!(
                                    "asr started source={source} provider={}",
                                    server_string(&text, "provider").unwrap_or_default()
                                );
                                recording::emit_state(&app, "recording", &source);
                                for chunk in pending.drain(..) {
                                    if write.send(Message::Binary(chunk.into())).await.is_err() {
                                        fail(&app, &source, AsrFailure::new("network", "语音识别连接已断开。"));
                                        finish(&app, &attempt_id, &source);
                                        return;
                                    }
                                }
                                if stopped.is_some() {
                                    break;
                                }
                            }
                            "result" => {
                                let text = server_text(&text);
                                if !text.is_empty() {
                                    emit(&app, SaylineEvent::Partial { text, source: source.clone() });
                                }
                            }
                            "completed" => {
                                terminal_text = Some(server_text(&text));
                                break;
                            }
                            "error" => {
                                fail(&app, &source, map_server_error(&text));
                                finish(&app, &attempt_id, &source);
                                return;
                            }
                            _ => {}
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        fail(&app, &source, AsrFailure::new("network", "语音识别连接已断开。"));
                        finish(&app, &attempt_id, &source);
                        return;
                    }
                    Some(Err(error)) => {
                        fail(&app, &source, AsrFailure::new("network", format!("语音识别连接异常: {error}")));
                        finish(&app, &attempt_id, &source);
                        return;
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(text) = terminal_text {
        finalize(&app, &attempt_id, &config, text, stopped.unwrap_or_default());
        return;
    }

    let stats = stopped.unwrap_or_default();
    if is_mic {
        if let Some((code, message)) = classify_captured(&stats) {
            fail(&app, &source, AsrFailure::new(code, message));
            let _ = write.send(Message::Text("{\"type\":\"cancel\"}".into())).await;
            finish(&app, &attempt_id, &source);
            return;
        }
    }

    if write
        .send(Message::Text("{\"type\":\"finish\"}".into()))
        .await
        .is_err()
    {
        fail(&app, &source, AsrFailure::new("network", "语音识别连接已断开。"));
        finish(&app, &attempt_id, &source);
        return;
    }
    recording::emit_state(&app, "finishing", &source);

    let mut finish_timer = Box::pin(tokio::time::sleep(FINISH_TIMEOUT));
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                let _ = write.send(Message::Text("{\"type\":\"cancel\"}".into())).await;
                finish(&app, &attempt_id, &source);
                return;
            }
            _ = &mut finish_timer => {
                fail(&app, &source, AsrFailure::new("upstream_timeout", "语音识别超时，请重试。"));
                let _ = write.send(Message::Text("{\"type\":\"cancel\"}".into())).await;
                finish(&app, &attempt_id, &source);
                return;
            }
            incoming = read.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match server_type(&text).as_str() {
                            "result" => {
                                let text = server_text(&text);
                                if !text.is_empty() {
                                    emit(&app, SaylineEvent::Partial { text, source: source.clone() });
                                }
                            }
                            "completed" => {
                                finalize(&app, &attempt_id, &config, server_text(&text), stats);
                                return;
                            }
                            "error" => {
                                fail(&app, &source, map_server_error(&text));
                                finish(&app, &attempt_id, &source);
                                return;
                            }
                            _ => {}
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        fail(&app, &source, AsrFailure::new("network", "语音识别连接已断开。"));
                        finish(&app, &attempt_id, &source);
                        return;
                    }
                    Some(Err(error)) => {
                        fail(&app, &source, AsrFailure::new("network", format!("语音识别连接异常: {error}")));
                        finish(&app, &attempt_id, &source);
                        return;
                    }
                    _ => {}
                }
            }
        }
    }
}

fn finalize(
    app: &AppHandle,
    attempt_id: &str,
    config: &AsrConfig,
    text: String,
    stats: CaptureStats,
) {
    let state = app.state::<AppState>();
    let settings = state
        .settings
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    let cleaned = settings::apply_replacements(text.trim(), &settings.replacements);
    tracing::info!(
        "asr completed source={} bytes={} duration_ms={} length={}",
        config.source,
        stats.bytes,
        stats.duration_ms(),
        cleaned.chars().count()
    );
    if cleaned.trim().is_empty() {
        fail(
            app,
            &config.source,
            AsrFailure::new("no_speech", "没有识别到内容，请靠近麦克风再说一次。"),
        );
        finish(app, attempt_id, &config.source);
        return;
    }

    let item = state
        .history
        .insert(&NewTranscript {
            text: cleaned.clone(),
            language: config.language.clone(),
            duration_ms: stats.duration_ms() as i64,
            source: config.source.clone(),
            title: config.title.clone(),
        })
        .ok();

    emit(
        app,
        SaylineEvent::Final {
            text: cleaned.clone(),
            source: config.source.clone(),
            item,
        },
    );
    recording::emit_state(app, "idle", &config.source);

    if config.source == "mic" && settings.auto_paste {
        let delay = settings.paste_delay_ms;
        std::thread::spawn(move || {
            if let Err(error) = crate::paste::paste_text(&cleaned, delay) {
                tracing::warn!("auto paste failed: {error}");
            }
        });
    }

    finish(app, attempt_id, &config.source);
}

fn fail(app: &AppHandle, source: &str, failure: AsrFailure) {
    tracing::warn!("asr failed source={source} code={}", failure.code);
    emit(
        app,
        SaylineEvent::Error {
            message: failure.message,
            source: source.to_string(),
            code: failure.code,
        },
    );
    recording::emit_state(app, "idle", source);
}

fn finish(app: &AppHandle, attempt_id: &str, source: &str) {
    if source == "mic" {
        recording::session_finished(app, attempt_id);
    } else {
        crate::file_job::session_finished(app);
    }
}

async fn request_ticket(config: &AsrConfig) -> Result<Ticket, AsrFailure> {
    let url = format!(
        "{}/api/v1/asr/tickets",
        config.endpoint.trim_end_matches('/')
    );
    let response = config
        .client
        .post(url)
        .bearer_auth(&config.token)
        .header("User-Agent", "ZCode/3.18.0")
        .header("HTTP-Referer", &config.endpoint)
        .header("X-Title", "Z Code@electron")
        .header("X-ZCode-App-Version", "3.18.0")
        .header("X-Platform", "linux-x64")
        .header("X-Release-Channel", "production")
        .header("X-Client-Language", &config.language)
        .header("X-Client-Timezone", "Asia/Shanghai")
        .header("X-Os-Category", "linux")
        .json(&serde_json::json!({ "mode": "realtime" }))
        .timeout(TICKET_TIMEOUT)
        .send()
        .await
        .map_err(|error| {
            AsrFailure::new("network", format!("无法连接语音识别服务: {error}"))
        })?;
    let status = response.status();
    let body: Value = response.json().await.unwrap_or(Value::Null);
    let code = body.get("code").and_then(Value::as_i64).unwrap_or(-1);
    if !status.is_success() || code != 0 {
        let error_code = body
            .get("data")
            .and_then(|data| data.get("error_code"))
            .and_then(Value::as_str);
        tracing::warn!(
            "asr ticket failed status={} code={} error_code={:?} msg={:?}",
            status.as_u16(),
            code,
            error_code,
            body.get("msg")
                .and_then(|value| value.as_str())
                .unwrap_or("")
        );
        return Err(map_http_error(status.as_u16(), error_code));
    }
    tracing::debug!("asr ticket acquired");
    let data = body
        .get("data")
        .ok_or_else(|| AsrFailure::new("failed", "语音识别服务响应异常。"))?;
    let ticket = data
        .get("ticket")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let ws_path = data
        .get("ws_path")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    if ticket.is_empty() || ws_path.is_empty() {
        return Err(AsrFailure::new("failed", "语音识别服务响应异常。"));
    }
    Ok(Ticket {
        ticket: ticket.to_string(),
        ws_path: ws_path.to_string(),
    })
}

fn ws_url(endpoint: &str, ws_path: &str, ticket: &str) -> String {
    let base = if ws_path.starts_with("http://")
        || ws_path.starts_with("https://")
        || ws_path.starts_with("ws://")
        || ws_path.starts_with("wss://")
    {
        ws_path.to_string()
    } else {
        format!(
            "{}{}",
            endpoint.trim_end_matches('/'),
            if ws_path.starts_with('/') {
                ws_path.to_string()
            } else {
                format!("/{ws_path}")
            }
        )
    };
    let base = base
        .replacen("https://", "wss://", 1)
        .replacen("http://", "ws://", 1);
    let separator = if base.contains('?') { '&' } else { '?' };
    format!("{base}{separator}ticket={ticket}")
}

fn server_type(text: &str) -> String {
    serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|value| value.get("type").and_then(Value::as_str).map(str::to_string))
        .unwrap_or_default()
}

fn server_text(text: &str) -> String {
    serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|value| value.get("text").and_then(Value::as_str).map(str::to_string))
        .unwrap_or_default()
}

fn server_string(text: &str, key: &str) -> Option<String> {
    serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|value| value.get(key).and_then(Value::as_str).map(str::to_string))
}

fn map_server_error(text: &str) -> AsrFailure {
    let value: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    let code = value
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("failed");
    tracing::warn!("asr server error code={code}");
    map_error_code(code)
}

fn map_http_error(status: u16, error_code: Option<&str>) -> AsrFailure {
    match status {
        401 => AsrFailure::new("unauthorized", "登录已过期，请重新登录 ZCode。"),
        429 => AsrFailure::new("rate_limited", "语音识别繁忙，请稍后再试。"),
        503 => AsrFailure::new("unavailable", "语音识别服务暂不可用，请稍后再试。"),
        _ => match error_code {
            Some(code) => map_error_code(code),
            None => AsrFailure::new("failed", "语音识别失败，请重试。"),
        },
    }
}

fn map_error_code(code: &str) -> AsrFailure {
    let (normalized, message) = match code {
        "unauthorized" => ("unauthorized", "登录已过期，请重新登录 ZCode。"),
        "rate_limited" => ("rate_limited", "语音识别繁忙，请稍后再试。"),
        "unavailable" | "upstream_unavailable" => {
            ("unavailable", "语音识别服务暂不可用，请稍后再试。")
        }
        "upstream_timeout" => ("upstream_timeout", "语音识别超时，请重试。"),
        "audio_limit_exceeded" => ("audio_limit_exceeded", "已达到单次录音时长上限。"),
        "empty_audio" | "audio_too_short" => ("audio_too_short", "说话时间太短，请重试。"),
        _ => ("failed", "语音识别失败，请重试。"),
    };
    AsrFailure::new(normalized, message)
}

fn classify_captured(stats: &CaptureStats) -> Option<(&'static str, &'static str)> {
    if stats.peak == 0.0 {
        return Some((
            "silent",
            "没有收到麦克风声音。请检查系统输入设备和音量。",
        ));
    }
    if stats.peak < 0.003_162_277_7 {
        return Some((
            "too_quiet",
            "没有检测到声音。请检查麦克风是否静音或离得太远。",
        ));
    }
    if stats.duration_ms() < 300 {
        return Some(("audio_too_short", "说话时间太短，请重试。"));
    }
    None
}

fn level_of(peak: f32) -> f32 {
    if peak <= 0.0 {
        0.0
    } else {
        ((20.0 * peak.log10() + 50.0) / 40.0).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_audio_is_rejected() {
        let stats = CaptureStats {
            peak: 0.001,
            samples: 16_000,
            bytes: 32_000,
        };
        assert_eq!(
            classify_captured(&stats).map(|value| value.0),
            Some("too_quiet")
        );
    }

    #[test]
    fn websocket_url_keeps_https() {
        let url = ws_url("https://zcode.z.ai", "/api/v1/asr/realtime", "abc");
        assert_eq!(url, "wss://zcode.z.ai/api/v1/asr/realtime?ticket=abc");
    }

    #[test]
    fn level_maps_decibels() {
        assert_eq!(level_of(0.0), 0.0);
        assert!((level_of(0.01) - 0.25).abs() < 0.001);
    }
}
