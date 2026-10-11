//! Allocation-free, format independent, stereo-linked OTT dynamics.
pub mod params;
use params::*;
use sps_dsp::filter::Crossover;
pub use sps_dsp::{db_to_gain, gain_to_db, AtomicValue};

pub const METER_COUNT: usize = 11;
pub const METER_NAMES: [&str; METER_COUNT] = [
    "Input",
    "Output",
    "Low level",
    "Mid level",
    "High level",
    "Low dynamics",
    "Mid dynamics",
    "High dynamics",
    "Low output",
    "Mid output",
    "High output",
];

pub struct MeterBus(pub [AtomicValue; METER_COUNT]);
impl Default for MeterBus {
    fn default() -> Self {
        Self(std::array::from_fn(|i| {
            AtomicValue::new(if (5..8).contains(&i) { 0.0 } else { -120.0 })
        }))
    }
}
impl MeterBus {
    pub fn publish(&self, values: &[f32; METER_COUNT]) {
        for (a, v) in self.0.iter().zip(values) {
            a.store(*v);
        }
    }
    pub fn snapshot(&self) -> [f32; METER_COUNT] {
        std::array::from_fn(|i| self.0[i].load())
    }
}

/// Compression gain in dB. Ratio >= 999 is an infinite downward ratio.
pub fn compression_gain(
    level: f32,
    down: f32,
    down_ratio: f32,
    up: f32,
    up_ratio: f32,
    knee: f32,
) -> f32 {
    let down_slope = if down_ratio >= 999.0 {
        1.0
    } else {
        1.0 - down_ratio.recip()
    };
    let up_slope = 1.0 - up_ratio.recip();
    let downward = -down_slope * soft_positive(level - down, knee);
    // Crossed thresholds are clamped by the engine. Soft transitions can overlap.
    let upward = (up_slope * soft_positive(up - level, knee)).min(36.0);
    // Upward compression fades out below -72 dBFS to avoid lifting the noise floor.
    let t = ((level + 90.0) / 18.0).clamp(0.0, 1.0);
    downward + upward * t * t * (3.0 - 2.0 * t)
}
fn soft_positive(x: f32, width: f32) -> f32 {
    if width <= 0.0 {
        x.max(0.0)
    } else if x <= -width * 0.5 {
        0.0
    } else if x >= width * 0.5 {
        x
    } else {
        (x + width * 0.5).powi(2) / (2.0 * width)
    }
}

#[derive(Clone, Copy)]
struct Band {
    envelope: f32,
    peak_envelope: f32,
    gain_db: f32,
    input_gain: f32,
    output_gain: f32,
    attack: f32,
    release: f32,
    down: f32,
    down_ratio: f32,
    up: f32,
    up_ratio: f32,
    audible: f32,
    audible_target: f32,
}
impl Default for Band {
    fn default() -> Self {
        Self {
            envelope: 0.0,
            peak_envelope: 0.0,
            gain_db: 0.0,
            input_gain: 1.0,
            output_gain: 1.0,
            attack: 0.0,
            release: 0.0,
            down: -30.0,
            down_ratio: 1.0,
            up: -40.0,
            up_ratio: 1.0,
            audible: 1.0,
            audible_target: 1.0,
        }
    }
}

pub struct Engine {
    rate: f32,
    crossover: Crossover,
    bands: [Band; 3],
    targets: [f32; PARAM_COUNT],
    current: [f32; PARAM_COUNT],
    control_tick: u32,
    control_smooth: f32,
    gain_smooth: f32,
    fade_smooth: f32,
    meter_decay: f32,
    peaks: [f32; 5],
    meters: [f32; METER_COUNT],
    output_gain: f32,
    amount: f32,
    mix: f32,
    bypass: f32,
    detector: f32,
    last_crossovers: [f32; 2],
}
impl Engine {
    pub fn new(sample_rate: f32) -> Self {
        let rate = if sample_rate.is_finite() {
            sample_rate.clamp(8000.0, 768000.0)
        } else {
            48000.0
        };
        let values = defaults();
        let mut engine = Self {
            rate,
            crossover: Crossover::new(rate, values[LOW_CROSS], values[HIGH_CROSS].min(rate * 0.45)),
            bands: [Band::default(); 3],
            targets: values,
            current: values,
            control_tick: 0,
            control_smooth: (-32.0 / (0.020 * rate)).exp(),
            gain_smooth: (-1.0 / (0.001 * rate)).exp(),
            fade_smooth: (-1.0 / (0.005 * rate)).exp(),
            meter_decay: (-1.0 / (0.180 * rate)).exp(),
            peaks: [0.0; 5],
            meters: [-120.0; METER_COUNT],
            output_gain: 1.0,
            amount: 1.0,
            mix: 1.0,
            bypass: 0.0,
            detector: 0.0,
            last_crossovers: [0.0; 2],
        };
        engine.refresh();
        engine
    }
    /// Called once per host block (including automation sub-blocks).
    pub fn set_parameters(&mut self, values: &[f32; PARAM_COUNT]) {
        self.targets = std::array::from_fn(|i| SPECS[i].sanitize(values[i]));
    }
    /// Snap to the loaded preset before audio starts; reset all signal history.
    pub fn reset(&mut self) {
        self.current = self.targets;
        self.bands = [Band::default(); 3];
        self.crossover.reset();
        self.peaks = [0.0; 5];
        self.control_tick = 0;
        self.bypass = self.targets[BYPASS];
        self.detector = self.targets[DETECTION];
        self.meters = [-120.0; METER_COUNT];
        self.meters[5..8].fill(0.0);
        self.refresh();
        for b in &mut self.bands {
            b.audible = b.audible_target;
        }
    }
    fn refresh(&mut self) {
        let low = self.current[LOW_CROSS].min(self.rate * 0.40);
        let high = self.current[HIGH_CROSS]
            .min(self.rate * 0.45)
            .max(low * 1.01);
        if self.last_crossovers != [low, high] {
            self.crossover.configure(self.rate, low, high);
            self.last_crossovers = [low, high];
        }
        self.amount = self.current[AMOUNT] * 0.01;
        self.mix = self.current[MIX] * 0.01;
        self.output_gain = db_to_gain(self.current[OUTPUT]);
        let any_solo = (0..3).any(|b| self.targets[band_index(b, SOLO)] >= 0.5);
        for (i, band) in self.bands.iter_mut().enumerate() {
            let n = band_index(i, 0);
            band.attack = (-1.0 / (self.current[n + ATTACK] * 0.001 * self.rate)).exp();
            band.release = (-1.0 / (self.current[n + RELEASE] * 0.001 * self.rate)).exp();
            band.input_gain = db_to_gain(self.current[n + INPUT_GAIN] * self.amount);
            band.output_gain = db_to_gain(self.current[n + BAND_OUTPUT] * self.amount);
            band.down = self.current[n + DOWN_THRESHOLD];
            band.down_ratio = self.current[n + DOWN_RATIO];
            band.up = self.current[n + UP_THRESHOLD].min(band.down);
            band.up_ratio = self.current[n + UP_RATIO];
            band.audible_target =
                if self.targets[n + MUTE] >= 0.5 || (any_solo && self.targets[n + SOLO] < 0.5) {
                    0.0
                } else {
                    1.0
                };
        }
    }
    #[inline]
    pub fn process_frame(&mut self, input: [f32; 2], channels: usize) -> [f32; 2] {
        let channels = channels.clamp(1, 2);
        if self.control_tick == 0 {
            let mut changed = false;
            for i in 0..PARAM_COUNT {
                changed |= self.current[i] != self.targets[i];
                let delta = self.targets[i] - self.current[i];
                self.current[i] = if delta.abs() < 1e-5 {
                    self.targets[i]
                } else {
                    self.targets[i] - delta * self.control_smooth
                };
            }
            if changed {
                self.refresh();
            }
        }
        self.control_tick = (self.control_tick + 1) & 31;
        self.bypass =
            self.targets[BYPASS] + (self.bypass - self.targets[BYPASS]) * self.fade_smooth;
        self.detector =
            self.targets[DETECTION] + (self.detector - self.targets[DETECTION]) * self.fade_smooth;
        let input = input.map(|x| {
            if x.is_finite() {
                x.clamp(-64.0, 64.0)
            } else {
                0.0
            }
        });
        let mut split = [[0.0; 3]; 2];
        for c in 0..channels {
            split[c] = self.crossover.split(input[c], c);
        }
        let mut wet = [0.0; 2];
        let mut dry = [0.0; 2];
        for c in 0..channels {
            dry[c] = split[c].iter().sum();
        }
        let knee = if self.targets[KNEE] >= 0.5 { 6.0 } else { 0.0 };
        for (i, band) in self.bands.iter_mut().enumerate() {
            let l = split[0][i] * band.input_gain;
            let r = if channels == 2 {
                split[1][i] * band.input_gain
            } else {
                l
            };
            // Max channel power preserves stereo image, including anti-phase material.
            let rms_power = (l * l).max(r * r);
            let peak = l.abs().max(r.abs());
            let coeff = if rms_power > band.envelope {
                band.attack
            } else {
                band.release
            };
            band.envelope = rms_power + (band.envelope - rms_power) * coeff;
            let peak_coeff = if peak > band.peak_envelope {
                band.attack
            } else {
                band.release
            };
            band.peak_envelope = peak + (band.peak_envelope - peak) * peak_coeff;
            if band.envelope < 1e-20 {
                band.envelope = 0.0;
            }
            if band.peak_envelope < 1e-20 {
                band.peak_envelope = 0.0;
            }
            let level = 10.0 * band.envelope.max(1e-12).log10() * (1.0 - self.detector)
                + 20.0 * band.peak_envelope.max(1e-6).log10() * self.detector;
            let target_gain = compression_gain(
                level,
                band.down,
                band.down_ratio,
                band.up,
                band.up_ratio,
                knee,
            ) * self.amount;
            band.gain_db = target_gain + (band.gain_db - target_gain) * self.gain_smooth;
            band.audible =
                band.audible_target + (band.audible - band.audible_target) * self.fade_smooth;
            let gain = db_to_gain(band.gain_db) * band.output_gain * band.audible;
            let bl = l * gain;
            let br = r * gain;
            wet[0] += bl;
            wet[1] += br;
            self.meters[2 + i] = level;
            self.meters[5 + i] = band.gain_db;
            self.peaks[2 + i] = bl
                .abs()
                .max(br.abs())
                .max(self.peaks[2 + i] * self.meter_decay);
            self.meters[8 + i] = gain_to_db(self.peaks[2 + i]);
        }
        let mut output = [0.0; 2];
        for c in 0..channels {
            let processed = (dry[c] + (wet[c] - dry[c]) * self.mix) * self.output_gain;
            output[c] = processed + (input[c] - processed) * self.bypass;
            if self.bypass >= 0.99999 {
                output[c] = input[c];
            }
        }
        self.peaks[0] = input[0]
            .abs()
            .max(if channels == 2 { input[1].abs() } else { 0.0 })
            .max(self.peaks[0] * self.meter_decay);
        self.peaks[1] = output[0]
            .abs()
            .max(output[1].abs())
            .max(self.peaks[1] * self.meter_decay);
        self.meters[0] = gain_to_db(self.peaks[0]);
        self.meters[1] = gain_to_db(self.peaks[1]);
        output
    }
    pub fn meters(&self) -> &[f32; METER_COUNT] {
        &self.meters
    }
}
