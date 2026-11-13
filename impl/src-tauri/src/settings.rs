use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replacement {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Global hotkey string, e.g. `Ctrl+Alt+Space`.
    pub hotkey: String,
    /// `zh-CN` or `en-US`. These are the two languages the ZCode ASR accepts.
    pub language: String,
    /// `system`, `light`, or `dark`.
    pub theme: String,
    /// Paste the final text into the focused application.
    pub auto_paste: bool,
    /// ZCode API origin. Kept configurable for tests and private deployments.
    pub endpoint_origin: String,
    /// Post-processing rules, applied after the final transcript arrives.
    pub replacements: Vec<Replacement>,
    /// Delay before sending Ctrl+V, so the target window has time to regain focus.
    pub paste_delay_ms: u64,
    /// File transcription pacing relative to real time (1.0 = real time).
    pub file_speed: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "Ctrl+Alt+Space".to_string(),
            language: "zh-CN".to_string(),
            theme: "system".to_string(),
            auto_paste: true,
            endpoint_origin: "https://zcode.z.ai".to_string(),
            replacements: Vec::new(),
            paste_delay_ms: 140,
            file_speed: 2.0,
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sayline")
        .join("settings.json")
}

pub fn load() -> Settings {
    let path = config_path();
    let Ok(raw) = fs::read_to_string(&path) else {
        return Settings::default();
    };
    match serde_json::from_str::<Settings>(&raw) {
        Ok(mut settings) => {
            if settings.language != "zh-CN" && settings.language != "en-US" {
                settings.language = "zh-CN".to_string();
            }
            if settings.theme != "system" && settings.theme != "light" && settings.theme != "dark" {
                settings.theme = "system".to_string();
            }
            if settings.endpoint_origin.trim().is_empty() {
                settings.endpoint_origin = Settings::default().endpoint_origin;
            }
            settings.endpoint_origin = settings.endpoint_origin.trim_end_matches('/').to_string();
            settings.file_speed = settings.file_speed.clamp(0.5, 4.0);
            settings
        }
        Err(error) => {
            tracing::warn!("settings parse failed, using defaults: {error}");
            Settings::default()
        }
    }
}

pub fn save(settings: &Settings) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("无法创建配置目录: {error}"))?;
    }
    let body = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(&path, body).map_err(|error| format!("无法写入配置: {error}"))
}

/// Rules run in order. Longer sources run first so a short rule cannot eat
/// the prefix of a longer one.
pub fn apply_replacements(text: &str, rules: &[Replacement]) -> String {
    let mut ordered: Vec<&Replacement> = rules.iter().filter(|rule| !rule.from.is_empty()).collect();
    ordered.sort_by_key(|rule| std::cmp::Reverse(rule.from.chars().count()));
    let mut output = text.to_string();
    for rule in ordered {
        output = output.replace(&rule.from, &rule.to);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacements_prefer_longer_sources() {
        let rules = vec![
            Replacement {
                from: "嗯".into(),
                to: "".into(),
            },
            Replacement {
                from: "嗯嗯".into(),
                to: "是的".into(),
            },
        ];
        assert_eq!(apply_replacements("嗯嗯，嗯", &rules), "是的，");
    }
}
