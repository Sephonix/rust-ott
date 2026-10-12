//! Original multi-style delay with independent coloration, modulation and shifts.
pub mod params;
use params::*;
pub use sps_dsp::AtomicValue;
use sps_dsp::{meter::Levels, time::*};
pub type MeterBus = sps_dsp::meter::MeterBus<2>;
pub const METER_COUNT: usize = 2;
pub const METER_NAMES: [&str; 2] = ["Input", "Output"];
#[derive(Clone, Copy, Default)]
struct PhaseStage {
    x: f32,
    y: f32,
}
impl PhaseStage {
    fn process(&mut self, x: f32, a: f32) -> f32 {
        let y = -a * x + self.x + a * self.y;
        self.x = x;
        self.y = flush(y);
        y
    }
}
pub struct Engine {
    rate: f32,
    controls: Controls<PARAM_COUNT>,
    lines: [DelayLine; 2],
    diffusers: [[Allpass; 4]; 4],
    pitch: [PitchShifter; 4],
    shift: [FrequencyShifter; 4],
    cuts: [[OnePole; 3]; 4],
    modulator: [Oscillator; 2],
    wow: Oscillator,
    flutter: Oscillator,
    phaser: [[PhaseStage; 12]; 4],
    phaser_last: [f32; 4],
    phase_a: [f32; 2],
    reverse: [[ReverseHead; 5]; 2],
    lengths: [f32; 2],
    length_ramps: [DelayRamp; 2],
    pitch_ratio: [f32; 2],
    gain: f32,
    drive: f32,
    compensation: f32,
    tempo: f32,
    envelope: f32,
    attack: f32,
    release: f32,
    noise: u32,
    mode: usize,
    style: usize,
    era: usize,
    switch_gain: f32,
    switch_step: f32,
    clock: usize,
    levels: Levels,
}
impl Engine {
    pub fn new(rate: f32) -> Self {
        let rate = sample_rate(rate);
        let mut discrete = std::array::from_fn(|i| SPECS[i].discrete);
        discrete[BYPASS] = false;
        let mut e = Self {
            rate,
            controls: Controls::new(defaults(), discrete, rate),
            lines: std::array::from_fn(|_| DelayLine::new((rate * 20.01) as usize)),
            diffusers: std::array::from_fn(|_| {
                std::array::from_fn(|_| Allpass::new((rate * 0.55) as usize))
            }),
            pitch: std::array::from_fn(|_| PitchShifter::new(rate)),
            shift: std::array::from_fn(|_| FrequencyShifter::default()),
            cuts: [[OnePole::default(); 3]; 4],
            modulator: [Oscillator::default(); 2],
            wow: Oscillator::default(),
            flutter: Oscillator::default(),
            phaser: [[PhaseStage::default(); 12]; 4],
            phaser_last: [0.0; 4],
            phase_a: [0.0; 2],
            reverse: [[ReverseHead::default(); 5]; 2],
            lengths: [0.0; 2],
            length_ramps: [DelayRamp::default(); 2],
            pitch_ratio: [1.0; 2],
            gain: 1.0,
            drive: 1.0,
            compensation: 1.0,
            tempo: 120.0,
            envelope: 0.0,
            attack: 1.0 - (-1.0 / (rate * 0.005)).exp(),
            release: 1.0 - (-1.0 / (rate * 0.15)).exp(),
            noise: 0x6ad5_49b1,
            mode: 0,
            style: 0,
            era: 1,
            switch_gain: 1.0,
            switch_step: 1.0 / (rate * 0.005),
            clock: 0,
            levels: Levels::new(rate),
        };
        e.reset();
        e
    }
    pub fn set_parameters(&mut self, values: &[f32; PARAM_COUNT]) {
        self.controls
            .set(std::array::from_fn(|i| SPECS[i].sanitize(values[i])));
    }
    pub fn set_tempo(&mut self, tempo: f32) {
        let t = if tempo.is_finite() {
            tempo.clamp(1.0, 999.0)
        } else {
            120.0
        };
        if self.tempo != t {
            self.tempo = t;
            self.configure();
        }
    }
    fn configure(&mut self) {
        let v = self.controls.current;
        let left = synced_ms(
            v[TIME_L],
            v[SYNC_L] as usize,
            v[DIVISION_L] as usize,
            self.tempo,
        );
        let right = synced_ms(
            v[TIME_R],
            v[SYNC_R] as usize,
            v[DIVISION_R] as usize,
            self.tempo,
        );
        self.lengths = match self.style {
            0 => [left + v[SPREAD], left - v[SPREAD]],
            2 => [left, left * v[RATIO] * 0.01],
            4 => [left, left],
            _ => [left, right],
        }
        .map(|t| t.clamp(0.01, 20000.0) * self.rate * 0.001);
        self.gain = sps_dsp::db_to_gain(v[OUTPUT]);
        self.drive = sps_dsp::db_to_gain(v[DRIVE]);
        self.compensation = self.drive.sqrt().recip();
        for c in 0..2 {
            self.length_ramps[c].set(self.lengths[c]);
            let detune = if c == 0 { v[DETUNE] } else { -v[DETUNE] };
            self.pitch_ratio[c] = 2.0_f32.powf((v[PITCH] + detune * 0.01) / 12.0);
            self.shift[c].configure(
                v[SHIFT]
                    + if c == 0 {
                        v[FREQ_DETUNE]
                    } else {
                        -v[FREQ_DETUNE]
                    },
                self.rate,
            );
            self.modulator[c].configure(
                v[MOD_RATE]
                    * (if self.style == 0 {
                        1.0
                    } else {
                        1.0 + c as f32 * 0.113
                    }),
                self.rate,
            );
            self.cuts[c][0].configure(v[LOW_CUT], self.rate);
            self.cuts[c][1].configure(v[HIGH_CUT], self.rate);
            let era = [0.65, 1.0, 1.45][self.era];
            let time_color = (350.0 / (self.lengths[c] * 1000.0 / self.rate).max(1.0))
                .sqrt()
                .clamp(0.35, 2.0);
            let color = match self.mode {
                0 | 9 | 15 => 7000.0,
                1 | 4 => 14000.0,
                2 | 7 => 2500.0 * time_color,
                11 => 1500.0,
                16 => 5000.0 * time_color,
                8 => 18000.0 * (1.0 - v[AGE] * 0.008),
                12 => {
                    if self.era == 2 {
                        self.rate * 0.45
                    } else {
                        18000.0 * (1.0 - v[AGE] * 0.008)
                    }
                }
                _ => 18000.0,
            };
            self.cuts[c][2].configure((color * era).min(self.rate * 0.45), self.rate);
            self.cuts[c + 2][0].configure(v[LOW_CUT], self.rate);
            self.cuts[c + 2][1].configure(v[HIGH_CUT], self.rate);
            self.cuts[c + 2][2].configure((color * era).min(self.rate * 0.45), self.rate);
            self.shift[c + 2].configure(
                v[SHIFT]
                    + if c == 0 {
                        v[FREQ_DETUNE]
                    } else {
                        -v[FREQ_DETUNE]
                    },
                self.rate,
            );
        }
        let time = (left * 0.001).max(0.05);
        self.wow
            .configure((0.13 / time).clamp(0.05, 4.0), self.rate);
        self.flutter
            .configure((2.3 / time).clamp(1.0, 30.0), self.rate);
    }
    pub fn reset(&mut self) {
        self.controls.reset();
        let v = self.controls.current;
        self.mode = v[MODE] as usize;
        self.style = v[STYLE] as usize;
        self.era = v[ERA] as usize;
        self.switch_gain = 1.0;
        for c in 0..2 {
            self.lines[c].reset();
            self.modulator[c].reset();
        }
        for c in 0..4 {
            self.pitch[c].reset();
            self.shift[c].reset();
            for d in &mut self.diffusers[c] {
                d.reset();
            }
            for cut in &mut self.cuts[c] {
                cut.reset();
            }
        }
        self.wow.reset();
        self.flutter.reset();
        self.phaser = [[PhaseStage::default(); 12]; 4];
        self.phaser_last = [0.0; 4];
        self.reverse = [[ReverseHead::default(); 5]; 2];
        self.envelope = 0.0;
        self.noise = 0x6ad5_49b1;
        self.clock = 0;
        self.levels.reset();
        self.configure();
        for ramp in &mut self.length_ramps {
            ramp.reset();
        }
    }
    fn random(&mut self) -> f32 {
        self.noise ^= self.noise << 13;
        self.noise ^= self.noise >> 17;
        self.noise ^= self.noise << 5;
        (self.noise as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
    fn color(&mut self, c: usize, x: f32) -> f32 {
        let v = self.controls.current;
        let m = self.mode;
        let mut y = x;
        // Tape and BBD use nonlinear coloration; digital modes retain small signals.
        if matches!(m, 0 | 1 | 2 | 4 | 7 | 9 | 11 | 15 | 16) {
            let knee = if matches!(m, 2 | 7 | 16) { 1.5 } else { 1.0 };
            y = (y * self.drive * knee).tanh() / knee * self.compensation;
            let noise_scale = match m {
                15 => 0.000002,
                9 => 0.000004,
                11 => 0.00025,
                2 | 7 => 0.00005,
                _ => 0.000025,
            };
            y += self.random() * v[AGE] * 0.01 * noise_scale * self.envelope.min(1.0);
        } else {
            let threshold = self.compensation.max(0.02);
            if y.abs() > threshold {
                y = y.signum() * (threshold + (y.abs() - threshold).tanh() * threshold * 0.25);
            }
        }
        if matches!(m, 3 | 5 | 6 | 10 | 11 | 14) && v[AGE] > 0.0 {
            let bits = 24.0 - v[AGE] * 0.2;
            let steps = 2.0_f32.powf(bits);
            y = (y * steps).round() / steps;
        }
        if m != 12 || self.era != 2 {
            y = self.cuts[c][2].low(y);
        }
        if m != 12 {
            let filtered = self.cuts[c][0].high(y);
            y = self.cuts[c][1].low(filtered);
        }
        ceiling(y)
    }
    // Output and head-D feedback need independent recursive history. The
    // same transform applies to both, so every character works in every style.
    fn transform(&mut self, c: usize, mut wet: f32) -> f32 {
        let channel = c % 2;
        let v = self.controls.current;
        if matches!(self.mode, 2 | 7 | 16) {
            wet = wet.signum() * wet * wet;
        }
        if matches!(self.mode, 5 | 10 | 14) {
            let shifted = self.pitch[c].process(wet, self.pitch_ratio[channel]);
            wet = if self.mode == 14 {
                wet * (1.0 - v[PITCH_MIX] * 0.01) + shifted * v[PITCH_MIX] * 0.01
            } else {
                shifted
            };
        }
        if self.mode == 4 {
            wet = self.shift[c].process(wet);
        }
        if self.mode == 13 {
            let mut y = wet + self.phaser_last[c] * v[AGE] * 0.007;
            for stage in self.phaser[c].iter_mut().take([4, 6, 12][self.era]) {
                y = stage.process(y, self.phase_a[channel]);
            }
            self.phaser_last[c] = ceiling(y);
            wet = (wet + y) * 0.5;
        }
        if v[DIFFUSION] >= 1.0 {
            for i in 0..4 {
                let seconds = (0.0031 + i as f32 * 0.0067)
                    * (v[DIFF_SIZE] * 0.01)
                    * (self.lengths[channel] / self.rate).clamp(0.1, 5.0);
                wet = self.diffusers[c][i].process(
                    wet,
                    seconds * self.rate * (1.0 + channel as f32 * 0.037),
                    v[DIFFUSION] * 0.0082,
                );
            }
        }
        wet = self.color(c, wet);

        wet
    }
    pub fn process_frame(&mut self, input: [f32; 2], channels: usize) -> [f32; 2] {
        if self.controls.tick() {
            self.configure();
        }
        let v = self.controls.current;
        let desired = (v[MODE] as usize, v[STYLE] as usize, v[ERA] as usize);
        if desired != (self.mode, self.style, self.era) {
            self.switch_gain = (self.switch_gain - self.switch_step).max(0.0);
            if self.switch_gain == 0.0 {
                (self.mode, self.style, self.era) = desired;
                self.configure();
            }
        } else {
            self.switch_gain = (self.switch_gain + self.switch_step).min(1.0);
        }
        let dry = input.map(finite);
        let mut source = if channels == 1 { [dry[0]; 2] } else { dry };
        let level = source[0].abs().max(source[1].abs());
        self.envelope += (level - self.envelope)
            * if level > self.envelope {
                self.attack
            } else {
                self.release
            };
        let wow = self.wow.tick().0;
        let flutter = self.flutter.tick().0;
        let mut wet = [0.0; 2];
        let mut last = [0.0; 2];
        for c in 0..2 {
            let (sin, cos) = self.modulator[c].tick();
            self.clock = (self.clock + 1) & 63;
            if self.clock < 2 {
                let frequency = (350.0
                    * (2.0_f32.powf((sin * (1.0 + v[MOD_DEPTH] * 0.04)) + c as f32 * 0.15)))
                .min(self.rate * 0.4);
                let t = (std::f32::consts::PI * frequency / self.rate).tan();
                self.phase_a[c] = (1.0 - t) / (1.0 + t);
            }
            let modulation = if matches!(self.mode, 0 | 9 | 11 | 15) {
                (wow * v[WOW] * 0.000003 + flutter * v[FLUTTER] * 0.0000008)
                    * self.rate
                    * self.lengths[c].min(self.rate)
                    / self.rate
            } else {
                (if self.style == 0 {
                    sin
                } else if c == 1 {
                    cos
                } else {
                    sin
                }) * v[MOD_DEPTH]
                    * 0.00004
                    * self.rate
            };
            let length = (self.length_ramps[c].tick() + modulation).clamp(1.0, self.rate * 20.0);
            last[c] = if self.mode == 6 {
                self.reverse[c][4].read(&self.lines[c], length, self.pitch_ratio[c])
            } else {
                self.lines[c].read(length)
            };
            wet[c] = if self.style == 4 {
                let mut y = 0.0;
                for tap in 0..4 {
                    if v[TAP_A + tap] >= 0.5 {
                        let p = (tap + 1) as f32 * 0.25;
                        let offset =
                            v[SPACING] * 0.004 * p * (1.0 - p) * if c == 0 { 1.0 } else { -1.0 };
                        let fraction =
                            (p - v[SPACING] * 0.008 * p * (1.0 - p) + offset).clamp(0.01, 1.0);
                        let read = if self.mode == 6 {
                            self.reverse[c][tap].read(
                                &self.lines[c],
                                length * fraction,
                                self.pitch_ratio[c],
                            )
                        } else {
                            self.lines[c].read(length * fraction)
                        };
                        y += read * 0.5;
                    }
                }
                y
            } else {
                last[c]
            };
            wet[c] = self.transform(c, wet[c]);
        }
        let feedback = if self.style == 4 && v[SWELL] < 0.5 {
            // Repeat takes its independent, fully processed feedback from D,
            // even when head D is disabled in the audible tap selection.
            std::array::from_fn(|c| self.transform(c + 2, last[c]))
        } else {
            wet
        };
        let routed = match self.style {
            2 => [
                (feedback[0] + feedback[1]) * std::f32::consts::FRAC_1_SQRT_2,
                (feedback[1] - feedback[0]) * std::f32::consts::FRAC_1_SQRT_2,
            ],
            3 => [feedback[1], feedback[0]],
            _ => feedback,
        };
        if self.style == 3 {
            source = [(source[0] + source[1]) * 0.5, 0.0];
        }
        for c in 0..2 {
            let x = ceiling(source[c] + routed[c] * v[FEEDBACK] * 0.01);
            // BBD companding encodes the stored signal and decodes read heads.
            // A power-law pair preserves unity gain before coloration/noise.
            self.lines[c].push(if matches!(self.mode, 2 | 7 | 16) {
                x.signum() * x.abs().sqrt()
            } else {
                x
            });
        }
        let duck = if matches!(self.mode, 7..=10) {
            1.0 / (1.0 + self.envelope * v[DUCK] * 0.3)
        } else {
            1.0
        };
        let mid = (wet[0] + wet[1]) * 0.5;
        let side = (wet[0] - wet[1]) * 0.5 * v[WIDTH] * 0.01;
        wet = [
            (mid + side) * duck * self.switch_gain,
            (mid - side) * duck * self.switch_gain,
        ];
        if self.mode == 12 {
            for (c, value) in wet.iter_mut().enumerate() {
                let filtered = self.cuts[c][0].high(*value);
                *value = self.cuts[c][1].low(filtered);
            }
        }
        let output = mix_frame(dry, wet, v[MIX], self.gain, v[BYPASS], channels);
        self.levels.process(dry, output);
        output
    }
    pub fn meters(&mut self) -> &[f32; 2] {
        self.levels.read()
    }
}
