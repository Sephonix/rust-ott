//! Preallocated primitives for time-domain effects. No host, UI or effect tuning.
use std::f32::consts::{PI, TAU};

pub fn sample_rate(rate: f32) -> f32 {
    if rate.is_finite() {
        rate.clamp(8000.0, 768000.0)
    } else {
        48000.0
    }
}
pub fn finite(x: f32) -> f32 {
    if x.is_finite() {
        flush(x.clamp(-64.0, 64.0))
    } else {
        0.0
    }
}
pub fn flush(x: f32) -> f32 {
    if x.abs() < 1e-25 {
        0.0
    } else {
        x
    }
}
/// Smooth safety ceiling: normal floating point overs remain untouched.
pub fn ceiling(x: f32) -> f32 {
    let x = finite(x);
    if x.abs() <= 4.0 {
        x
    } else {
        x.signum() * (4.0 + (x.abs() - 4.0).tanh())
    }
}

pub struct DelayLine {
    data: Vec<f32>,
    cursor: usize,
}
impl DelayLine {
    pub fn new(max_samples: usize) -> Self {
        Self {
            data: vec![0.0; max_samples.max(4) + 4],
            cursor: 0,
        }
    }
    pub fn reset(&mut self) {
        self.data.fill(0.0);
        self.cursor = 0;
    }
    /// Read before push. One sample is the most recently pushed value.
    pub fn read(&self, samples: f32) -> f32 {
        let d = samples.clamp(1.0, (self.data.len() - 3) as f32);
        let n = d as usize;
        let t = d - n as f32;
        let a = if self.cursor >= n {
            self.cursor - n
        } else {
            self.cursor + self.data.len() - n
        };
        let b = if a == 0 { self.data.len() - 1 } else { a - 1 };
        self.data[a] + t * (self.data[b] - self.data[a])
    }
    pub fn push(&mut self, x: f32) {
        self.data[self.cursor] = flush(finite(x));
        self.cursor += 1;
        if self.cursor == self.data.len() {
            self.cursor = 0;
        }
    }
}

pub struct Allpass {
    line: DelayLine,
}
impl Allpass {
    pub fn new(max_samples: usize) -> Self {
        Self {
            line: DelayLine::new(max_samples),
        }
    }
    pub fn reset(&mut self) {
        self.line.reset();
    }
    pub fn process(&mut self, x: f32, length: f32, gain: f32) -> f32 {
        let g = gain.clamp(-0.85, 0.85);
        let delayed = self.line.read(length);
        let y = delayed - g * x;
        self.line.push(x + g * y);
        y
    }
}
#[derive(Default, Clone, Copy)]
pub struct OnePole {
    state: f64,
    coefficient: f64,
}
impl OnePole {
    pub fn configure(&mut self, hz: f32, rate: f32) {
        self.coefficient =
            1.0 - (-std::f64::consts::TAU * hz.clamp(0.0, rate * 0.45) as f64 / rate as f64).exp();
    }
    pub fn reset(&mut self) {
        self.state = 0.0;
    }
    pub fn low(&mut self, x: f32) -> f32 {
        self.state += self.coefficient * (x as f64 - self.state);
        if self.state.abs() < 1e-25 {
            self.state = 0.0;
        }
        self.state as f32
    }
    pub fn high(&mut self, x: f32) -> f32 {
        x - self.low(x)
    }
}

/// Complex oscillator with periodic normalization, avoiding per-sample trig.
#[derive(Clone, Copy)]
pub struct Oscillator {
    sin: f64,
    cos: f64,
    step_sin: f64,
    step_cos: f64,
    count: u32,
}
impl Default for Oscillator {
    fn default() -> Self {
        Self {
            sin: 0.0,
            cos: 1.0,
            step_sin: 0.0,
            step_cos: 1.0,
            count: 0,
        }
    }
}
impl Oscillator {
    pub fn configure(&mut self, hz: f32, rate: f32) {
        (self.step_sin, self.step_cos) =
            (std::f64::consts::TAU * hz as f64 / rate as f64).sin_cos();
    }
    pub fn reset(&mut self) {
        self.sin = 0.0;
        self.cos = 1.0;
        self.count = 0;
    }
    pub fn tick(&mut self) -> (f32, f32) {
        let result = (self.sin as f32, self.cos as f32);
        (self.sin, self.cos) = (
            self.sin * self.step_cos + self.cos * self.step_sin,
            self.cos * self.step_cos - self.sin * self.step_sin,
        );
        self.count += 1;
        if self.count == 4096 {
            let norm = self.sin.hypot(self.cos);
            self.sin /= norm;
            self.cos /= norm;
            self.count = 0;
        }
        result
    }
}
/// Windowed ideal Hilbert FIR, designed from its mathematical impulse response.
/// 256-sample history, 128-sample group delay. The useful band excludes DC/Nyquist.
/// The real path uses the same delay; positive shifts suppress the lower sideband.
pub struct FrequencyShifter {
    history: [f32; 512],
    coefficients: [f32; 64],
    cursor: usize,
    oscillator: Oscillator,
}
impl Default for FrequencyShifter {
    fn default() -> Self {
        Self {
            history: [0.0; 512],
            coefficients: std::array::from_fn(|i| {
                let k = (2 * i + 1) as f32;
                let window = 0.42 + 0.5 * (PI * k / 128.0).cos() + 0.08 * (TAU * k / 128.0).cos();
                2.0 * window / (PI * k)
            }),
            cursor: 0,
            oscillator: Oscillator::default(),
        }
    }
}
impl FrequencyShifter {
    pub fn configure(&mut self, hz: f32, rate: f32) {
        self.oscillator
            .configure(hz.clamp(-rate * 0.4, rate * 0.4), rate);
    }
    pub fn reset(&mut self) {
        self.history.fill(0.0);
        self.cursor = 0;
        self.oscillator.reset();
    }
    pub fn process(&mut self, x: f32) -> f32 {
        self.history[self.cursor] = x;
        let center = (self.cursor + 512 - 128) & 511;
        let real = self.history[center];
        let mut imag = 0.0;
        // Only the first 64 positive odd taps lie inside the 257-tap window.
        for i in 0..64 {
            let k = 2 * i + 1;
            imag += self.coefficients[i]
                * (self.history[(center + 512 - k) & 511] - self.history[(center + k) & 511]);
        }
        self.cursor = (self.cursor + 1) & 511;
        let (sin, cos) = self.oscillator.tick();
        real * cos - imag * sin
    }
}
/// Two overlapping moving read heads. Ratio 1 is an exact transparent path.
pub struct PitchShifter {
    line: DelayLine,
    phase: f32,
    rate: f32,
}
impl PitchShifter {
    pub fn new(rate: f32) -> Self {
        Self {
            line: DelayLine::new((rate * 0.25) as usize),
            phase: 0.0,
            rate,
        }
    }
    pub fn reset(&mut self) {
        self.line.reset();
        self.phase = 0.0;
    }
    pub fn process(&mut self, x: f32, ratio: f32) -> f32 {
        let window = self.rate * 0.08;
        self.line.push(x);
        if (ratio - 1.0).abs() < 1e-6 {
            return x;
        }
        self.phase = (self.phase + (1.0 - ratio) / window).rem_euclid(1.0);
        let p = self.phase;
        let q = (p + 0.5).fract();
        // Complementary triangular windows put zero weight on head wraparound.
        let a = 1.0 - (2.0 * p - 1.0).abs();
        self.line.read(2.0 + window * p) * a + self.line.read(2.0 + window * q) * (1.0 - a)
    }
}
/// Smoothed controls updated on a 32-sample cadence. Configuration work is only
/// needed when tick returns true. Discrete controls are switched by the caller.
pub struct Controls<const N: usize> {
    pub current: [f32; N],
    target: [f32; N],
    discrete: [bool; N],
    coefficient: f32,
    count: usize,
}
impl<const N: usize> Controls<N> {
    pub fn new(values: [f32; N], discrete: [bool; N], rate: f32) -> Self {
        Self {
            current: values,
            target: values,
            discrete,
            coefficient: 1.0 - (-32.0 / (rate * 0.02)).exp(),
            count: 0,
        }
    }
    pub fn set(&mut self, values: [f32; N]) {
        self.target = values;
    }
    pub fn reset(&mut self) {
        self.current = self.target;
        self.count = 0;
    }
    pub fn tick(&mut self) -> bool {
        self.count = (self.count + 1) & 31;
        if self.count != 1 {
            return false;
        }
        let mut changed = false;
        for i in 0..N {
            let d = self.target[i] - self.current[i];
            if d != 0.0 {
                changed = true;
                self.current[i] = if self.discrete[i] || d.abs() < 1e-5 {
                    self.target[i]
                } else {
                    self.current[i] + d * self.coefficient
                };
            }
        }
        changed
    }
}
/// Tempo subdivisions in quarter-note beats, with optional dotted/triplet scale.
pub fn synced_ms(free_ms: f32, sync: usize, division: usize, tempo: f32) -> f32 {
    const BEATS: [f32; 12] = [
        0.0625, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0,
    ];
    if sync == 0 {
        free_ms
    } else {
        let tempo = if tempo.is_finite() {
            tempo.clamp(1.0, 999.0)
        } else {
            120.0
        };
        60000.0 / tempo
            * BEATS[division.min(11)]
            * match sync {
                2 => 1.5,
                3 => 2.0 / 3.0,
                _ => 1.0,
            }
    }
}
/// Final linear dry/wet blend with exact endpoint bypass. Dry is unfiltered.
pub fn mix_frame(
    dry: [f32; 2],
    wet: [f32; 2],
    mix: f32,
    gain: f32,
    bypass: f32,
    channels: usize,
) -> [f32; 2] {
    let m = mix * 0.01;
    let b = bypass.clamp(0.0, 1.0);
    let wet = if channels == 1 {
        [(wet[0] + wet[1]) * 0.5; 2]
    } else {
        wet
    };
    std::array::from_fn(|i| {
        if b == 1.0 {
            dry[i]
        } else {
            ((dry[i] * (1.0 - m) + wet[i] * m) * gain) * (1.0 - b) + dry[i] * b
        }
    })
}

#[derive(Default, Clone, Copy)]
struct QuadratureStage {
    x: [f64; 2],
    y: [f64; 2],
    cursor: usize,
}
impl QuadratureStage {
    fn process(&mut self, x: f64, a: f64) -> f64 {
        let c = self.cursor;
        let y = -a * x + self.x[c] + a * self.y[c];
        self.x[c] = x;
        self.y[c] = if y.abs() < 1e-25 { 0.0 } else { y };
        self.cursor ^= 1;
        y
    }
}
/// Low-latency analytic signal from two independent allpass cascades. Coefficients
/// are an original numerical phase fit of (z^-2-a)/(1-a*z^-2), reproducible with
/// scripts/design_quadrature.py. No fixed block latency; phase delay varies with
/// frequency. At exactly zero shift the transparent path fades in over 2 ms.
pub struct QuadratureShifter {
    stages: [[QuadratureStage; 8]; 2],
    previous: f64,
    oscillator: Oscillator,
    enabled: f32,
    target: f32,
    smoothing: f32,
}
impl Default for QuadratureShifter {
    fn default() -> Self {
        Self {
            stages: [[QuadratureStage::default(); 8]; 2],
            previous: 0.0,
            oscillator: Oscillator::default(),
            enabled: 0.0,
            target: 0.0,
            smoothing: 0.01,
        }
    }
}
impl QuadratureShifter {
    pub fn configure(&mut self, hz: f32, rate: f32) {
        self.oscillator
            .configure(hz.clamp(-rate * 0.4, rate * 0.4), rate);
        self.target = if hz == 0.0 { 0.0 } else { 1.0 };
        self.smoothing = 1.0 - (-1.0 / (rate * 0.002)).exp();
    }
    pub fn reset(&mut self) {
        self.stages = [[QuadratureStage::default(); 8]; 2];
        self.previous = 0.0;
        self.oscillator.reset();
        self.enabled = self.target;
    }
    pub fn process(&mut self, x: f32) -> f32 {
        const A: [f64; 8] = [
            0.06114783047911325,
            0.41245573699654275,
            0.7335900585963658,
            0.8970527556774885,
            0.962705248179655,
            0.9868583797571414,
            0.9955131379384673,
            0.9987318974629334,
        ];
        const B: [f64; 8] = [
            0.218072799009236,
            0.5927230061086786,
            0.8323482694758132,
            0.9377401443880077,
            0.9778028906981068,
            0.9922708207535442,
            0.9974828505128236,
            0.9996145981770883,
        ];
        let mut real = x as f64;
        let mut imaginary = x as f64;
        for i in 0..8 {
            real = self.stages[0][i].process(real, A[i]);
            imaginary = self.stages[1][i].process(imaginary, B[i]);
        }
        let quadrature = self.previous;
        self.previous = imaginary;
        let (sin, cos) = self.oscillator.tick();
        self.enabled += (self.target - self.enabled) * self.smoothing;
        if (self.target - self.enabled).abs() < 1e-5 {
            self.enabled = self.target;
        }
        // Path A leads delayed path B by pi/2; B is the sine quadrature.
        let shifted = real as f32 * cos - quadrature as f32 * sin;
        shifted * self.enabled + x * (1.0 - self.enabled)
    }
}
/// Linear interpolation between control-rate delay lengths. Reading a moving
/// head each sample avoids stair-step clicks while retaining Doppler behavior.
#[derive(Default, Clone, Copy)]
pub struct DelayRamp {
    value: f32,
    target: f32,
    step: f32,
    remaining: u32,
}
impl DelayRamp {
    pub fn set(&mut self, target: f32) {
        self.target = target;
        self.step = (target - self.value) / 32.0;
        self.remaining = 32;
    }
    pub fn reset(&mut self) {
        self.value = self.target;
        self.step = 0.0;
        self.remaining = 0;
    }
    pub fn tick(&mut self) -> f32 {
        if self.remaining != 0 {
            self.remaining -= 1;
            self.value = if self.remaining == 0 {
                self.target
            } else {
                self.value + self.step
            };
        }
        self.value
    }
}
/// Two overlapping backwards playback heads within a chosen delay span.
#[derive(Default, Clone, Copy)]
pub struct ReverseHead {
    phase: f32,
}
impl ReverseHead {
    pub fn read(&mut self, line: &DelayLine, length: f32, ratio: f32) -> f32 {
        let window = (length * 0.5).max(4.0);
        self.phase = (self.phase + (1.0 + ratio) / window).fract();
        let p = self.phase;
        let q = (p + 0.5).fract();
        let a = 1.0 - (2.0 * p - 1.0).abs();
        line.read(length * 0.5 + p * window) * a + line.read(length * 0.5 + q * window) * (1.0 - a)
    }
}
