pub const SAMPLE_RATE: u32 = 16_000;
pub const CHUNK_SAMPLES: usize = 1_600;

#[derive(Debug, Clone, Copy, Default)]
pub struct CaptureStats {
    pub peak: f32,
    pub samples: u64,
    pub bytes: u64,
}

impl CaptureStats {
    pub fn duration_ms(&self) -> u64 {
        self.samples.saturating_mul(1000) / SAMPLE_RATE as u64
    }
}

pub enum AudioMessage {
    Chunk { bytes: Vec<u8>, peak: f32 },
    End(CaptureStats),
    Error(String),
}

pub use crate::capture::spawn_capture;
