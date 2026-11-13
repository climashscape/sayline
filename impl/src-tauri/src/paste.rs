use std::process::{Command, Stdio};
use std::time::Duration;

/// Puts text on the X11 clipboard and keeps a serving owner alive long enough
/// for the user (or the next paste) to consume it. The Clipboard object owns
/// the selection on X11; dropping it immediately would clear the content.
pub fn copy_text(text: &str) -> Result<(), String> {
    let owned = text.to_string();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || match arboard::Clipboard::new() {
        Ok(mut clipboard) => match clipboard.set_text(owned) {
            Ok(()) => {
                let _ = ready_tx.send(Ok(()));
                std::thread::sleep(Duration::from_secs(120));
                drop(clipboard);
            }
            Err(error) => {
                let _ = ready_tx.send(Err(format!("写入剪贴板失败: {error}")));
            }
        },
        Err(error) => {
            let _ = ready_tx.send(Err(format!("剪贴板不可用: {error}")));
        }
    });
    ready_rx
        .recv()
        .map_err(|_| "剪贴板线程异常".to_string())?
}

/// Copies the final transcript and sends Ctrl+V to the focused application.
/// The text goes through the clipboard, never through a shell or argv.
pub fn paste_text(text: &str, delay_ms: u64) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|error| format!("剪贴板不可用: {error}"))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|error| format!("写入剪贴板失败: {error}"))?;
    std::thread::sleep(Duration::from_millis(delay_ms.clamp(0, 2000)));
    let status = Command::new("xdotool")
        .args(["key", "--clearmodifiers", "ctrl+v"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("无法调用 xdotool: {error}"))?;
    std::thread::sleep(Duration::from_millis(600));
    drop(clipboard);
    if !status.success() {
        return Err("xdotool 粘贴失败；请确认当前会话是 X11。".to_string());
    }
    Ok(())
}
