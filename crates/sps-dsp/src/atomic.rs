//! Independent atomic floats for audio/UI mailboxes and telemetry.
use std::sync::atomic::{AtomicU32, Ordering};

/// A relaxed atomic float. Each value is independent; loads do not synchronize
/// other memory or guarantee a coherent snapshot of multiple values.
pub struct AtomicValue(AtomicU32);
impl AtomicValue {
    pub fn new(v: f32) -> Self {
        Self(AtomicU32::new(v.to_bits()))
    }
    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }
    pub fn store(&self, v: f32) {
        self.0.store(v.to_bits(), Ordering::Relaxed);
    }
}
