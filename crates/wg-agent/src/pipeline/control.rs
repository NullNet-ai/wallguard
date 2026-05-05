use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Runtime control signals that the server can flip while the agent is running.
///
/// Passed as a single `Arc<PipelineControl>` to all pipeline tasks and to the
/// control-channel handler, replacing the previous pattern of threading separate
/// `Arc<AtomicU32>` and `Arc<AtomicBool>` through every function signature.
pub struct PipelineControl {
    /// Packet sampling rate stored as f32 bits. 1.0 = full rate, 0.0 = drop all.
    sampling_rate:     AtomicU32,
    /// When false the metrics pipeline skips collection and sends nothing.
    telemetry_enabled: AtomicBool,
}

impl PipelineControl {
    pub fn new() -> Self {
        Self {
            sampling_rate:     AtomicU32::new(1.0f32.to_bits()),
            telemetry_enabled: AtomicBool::new(true),
        }
    }

    pub fn sampling_rate(&self) -> f32 {
        f32::from_bits(self.sampling_rate.load(Ordering::Relaxed))
    }

    pub fn set_sampling_rate(&self, rate: f32) {
        self.sampling_rate
            .store(rate.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn telemetry_enabled(&self) -> bool {
        self.telemetry_enabled.load(Ordering::Relaxed)
    }

    pub fn set_telemetry_enabled(&self, enabled: bool) {
        self.telemetry_enabled.store(enabled, Ordering::Relaxed);
    }
}
