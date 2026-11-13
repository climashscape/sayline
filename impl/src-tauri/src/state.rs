use std::sync::mpsc::Sender;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::history::History;
use crate::hotkey::HotkeyCommand;
use crate::settings::Settings;

pub struct RecordingHandle {
    pub attempt_id: String,
    pub stop_tx: Sender<()>,
    pub cancel: CancellationToken,
}

pub struct FileJob {
    pub cancel: CancellationToken,
}

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub history: History,
    pub recording: Mutex<Option<RecordingHandle>>,
    pub file_job: Mutex<Option<FileJob>>,
    pub hotkey_tx: Mutex<Option<Sender<HotkeyCommand>>>,
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new(settings: Settings, history: History) -> Self {
        Self {
            settings: Mutex::new(settings),
            history,
            recording: Mutex::new(None),
            file_job: Mutex::new(None),
            hotkey_tx: Mutex::new(None),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }
}
