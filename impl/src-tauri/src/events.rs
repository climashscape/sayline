use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::history::HistoryItem;

/// Every background task reports through one event name so the frontend only
/// needs one listener. `source` is `mic` or `file`.
#[derive(Clone, Serialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum SaylineEvent {
    State {
        state: String,
        source: String,
    },
    Level {
        level: f32,
        source: String,
    },
    Partial {
        text: String,
        source: String,
    },
    Final {
        text: String,
        source: String,
        item: Option<HistoryItem>,
    },
    Error {
        message: String,
        source: String,
        code: String,
    },
    Progress {
        percent: f32,
        source: String,
    },
}

pub fn emit(app: &AppHandle, event: SaylineEvent) {
    if let Err(error) = app.emit("sayline", event) {
        tracing::warn!("emit sayline event failed: {error}");
    }
}
