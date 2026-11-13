use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::str::FromStr;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;
use tauri::AppHandle;

use crate::events::{emit, SaylineEvent};
use crate::recording;

pub enum HotkeyCommand {
    Set(String),
}

pub fn spawn(app: AppHandle) -> Sender<HotkeyCommand> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let manager = match GlobalHotKeyManager::new() {
            Ok(manager) => manager,
            Err(error) => {
                emit(
                    &app,
                    SaylineEvent::Error {
                        message: format!("无法注册全局热键: {error}"),
                        source: "mic".to_string(),
                        code: "hotkey".to_string(),
                    },
                );
                return;
            }
        };
        let mut current: Option<HotKey> = None;
        let mut active = false;

        loop {
            while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                let Some(hotkey) = current else {
                    continue;
                };
                if event.id() != hotkey.id() {
                    continue;
                }
                match event.state() {
                    HotKeyState::Pressed => {
                        if !active && !recording::is_recording(&app) {
                            match recording::start(&app) {
                                Ok(()) => active = true,
                                Err(message) => {
                                    emit(
                                        &app,
                                        SaylineEvent::Error {
                                            message,
                                            source: "mic".to_string(),
                                            code: "auth".to_string(),
                                        },
                                    );
                                    recording::emit_state(&app, "idle", "mic");
                                }
                            }
                        }
                    }
                    HotKeyState::Released => {
                        if active {
                            let _ = recording::stop(&app);
                            active = false;
                        }
                    }
                }
            }

            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(HotkeyCommand::Set(value)) => {
                    let hotkey = match HotKey::from_str(&value) {
                        Ok(hotkey) => hotkey,
                        Err(_) => {
                            emit(
                                &app,
                                SaylineEvent::Error {
                                    message: format!("热键格式无效: {value}"),
                                    source: "mic".to_string(),
                                    code: "hotkey".to_string(),
                                },
                            );
                            continue;
                        }
                    };
                    if current.map(|old| old.id()) == Some(hotkey.id()) {
                        continue;
                    }
                    if manager.register(hotkey).is_ok() {
                        if let Some(old) = current.take() {
                            let _ = manager.unregister(old);
                        }
                        current = Some(hotkey);
                    } else {
                        emit(
                            &app,
                            SaylineEvent::Error {
                                message: format!("热键被其他程序占用: {value}"),
                                source: "mic".to_string(),
                                code: "hotkey".to_string(),
                            },
                        );
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        if let Some(hotkey) = current {
            let _ = manager.unregister(hotkey);
        }
    });
    tx
}
