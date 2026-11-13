use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::AppHandle;
use tokio::sync::mpsc::UnboundedSender;

use crate::audio::{AudioMessage, CaptureStats, CHUNK_SAMPLES};

/// Starts a system recorder and streams 16 kHz mono PCM16 to the ASR session.
pub fn spawn_capture(
    _app: AppHandle,
    tx: UnboundedSender<AudioMessage>,
    stop_rx: Receiver<()>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(error) = run_capture(&tx, stop_rx) {
            let _ = tx.send(AudioMessage::Error(error));
        }
    })
}

fn run_capture(
    tx: &UnboundedSender<AudioMessage>,
    stop_rx: Receiver<()>,
) -> Result<(), String> {
    let mut child = start_recorder()?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "无法读取录音进程输出".to_string())?;
    let stats = Arc::new(Mutex::new(CaptureStats::default()));
    let reader_tx = tx.clone();
    let reader_stats = stats.clone();

    let reader = std::thread::spawn(move || {
        let mut pending: Vec<u8> = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            match stdout.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    pending.extend_from_slice(&buffer[..count]);
                    drain_pcm(&mut pending, &reader_tx, &reader_stats, false);
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
        drain_pcm(&mut pending, &reader_tx, &reader_stats, true);
    });

    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = reader.join();
                    return Err(
                        "录音进程已退出。请检查 PulseAudio / PipeWire 输入设备。".to_string()
                    );
                }
                break;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return Err(format!("等待录音进程失败: {error}"));
            }
        }
    }

    let _ = child.kill();
    let _ = child.wait();
    let _ = reader.join();
    let final_stats = *stats.lock().map_err(|_| "录音统计锁失败".to_string())?;
    let _ = tx.send(AudioMessage::End(final_stats));
    Ok(())
}

fn start_recorder() -> Result<Child, String> {
    if command_exists("parec") {
        return spawn_parec();
    }
    if command_exists("pw-record") {
        return spawn_pw_record();
    }
    if command_exists("arecord") {
        return spawn_arecord();
    }
    Err("未找到可用的录音程序。请安装 PulseAudio、PipeWire 或 ALSA 工具。".to_string())
}

fn spawn_parec() -> Result<Child, String> {
    Command::new("parec")
        .args([
            "--format=s16le",
            "--rate=16000",
            "--channels=1",
            "--latency-msec=100",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("无法启动 parec: {error}"))
}

fn spawn_pw_record() -> Result<Child, String> {
    Command::new("pw-record")
        .args(["--format=s16", "--rate=16000", "--channels=1", "-"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("无法启动 pw-record: {error}"))
}

fn spawn_arecord() -> Result<Child, String> {
    Command::new("arecord")
        .args(["-q", "-t", "raw", "-f", "S16_LE", "-r", "16000", "-c", "1"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("无法启动 arecord: {error}"))
}

fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths)
                .any(|directory| Path::new(&directory).join(name).is_file())
        })
        .unwrap_or(false)
}

/// Converts raw PCM16 bytes into 100 ms chunks and updates peak/duration.
fn drain_pcm(
    pending: &mut Vec<u8>,
    tx: &UnboundedSender<AudioMessage>,
    stats: &Arc<Mutex<CaptureStats>>,
    flush: bool,
) {
    loop {
        let chunk_bytes = CHUNK_SAMPLES * 2;
        if pending.is_empty() || (!flush && pending.len() < chunk_bytes) {
            break;
        }
        let take = pending.len().min(chunk_bytes);
        let mut bytes = pending.drain(..take).collect::<Vec<u8>>();
        if bytes.len() % 2 == 1 {
            bytes.pop();
        }
        let mut peak = 0.0f32;
        for pair in bytes.chunks_exact(2) {
            let sample = i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32768.0;
            let absolute = sample.abs();
            if absolute > peak {
                peak = absolute;
            }
        }
        if let Ok(mut guard) = stats.lock() {
            guard.peak = guard.peak.max(peak);
            guard.samples = guard.samples.saturating_add((bytes.len() / 2) as u64);
            guard.bytes = guard.bytes.saturating_add(bytes.len() as u64);
        }
        let _ = tx.send(AudioMessage::Chunk { bytes, peak });
    }
    if flush {
        pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::SAMPLE_RATE;

    #[test]
    fn duration_uses_16khz() {
        let stats = CaptureStats {
            peak: 0.0,
            samples: SAMPLE_RATE as u64,
            bytes: SAMPLE_RATE as u64 * 2,
        };
        assert_eq!(stats.duration_ms(), 1000);
    }
}
