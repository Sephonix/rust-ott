//! Independent atomic meter mailboxes. Values do not form synchronized snapshots.
use crate::AtomicValue;
pub struct MeterBus<const N: usize>(pub [AtomicValue; N]);
impl<const N: usize> Default for MeterBus<N> {
    fn default() -> Self {
        Self(std::array::from_fn(|_| AtomicValue::new(-120.0)))
    }
}
impl<const N: usize> MeterBus<N> {
    pub fn publish(&self, values: &[f32; N]) {
        for (a, v) in self.0.iter().zip(values) {
            a.store(*v);
        }
    }
    pub fn snapshot(&self) -> [f32; N] {
        std::array::from_fn(|i| self.0[i].load())
    }
}
/// Peak envelope with sample-rate independent falloff, calculated outside the UI.
pub struct Levels {
    peaks: [f32; 2],
    decay: f32,
    values: [f32; 2],
}
impl Levels {
    pub fn new(rate: f32) -> Self {
        Self {
            peaks: [0.0; 2],
            decay: (-1.0 / (rate * 0.3)).exp(),
            values: [-120.0; 2],
        }
    }
    pub fn reset(&mut self) {
        self.peaks = [0.0; 2];
        self.values = [-120.0; 2];
    }
    pub fn process(&mut self, input: [f32; 2], output: [f32; 2]) {
        for (i, frame) in [input, output].iter().enumerate() {
            self.peaks[i] = frame[0]
                .abs()
                .max(frame[1].abs())
                .max(self.peaks[i] * self.decay);
        }
    }
    pub fn read(&mut self) -> &[f32; 2] {
        for i in 0..2 {
            self.values[i] = crate::gain_to_db(self.peaks[i]);
        }
        &self.values
    }
}
