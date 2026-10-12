//! Original true-stereo early energy and frequency-shaped feedback delay network.
pub mod params;
use params::*;
pub use sps_dsp::AtomicValue;
use sps_dsp::{meter::Levels, time::*};
pub type MeterBus = sps_dsp::meter::MeterBus<2>;
pub const METER_COUNT: usize = 2;
pub const METER_NAMES: [&str; 2] = ["Input", "Output"];
// Original profiles: time scale, brightness ceiling, modulation, density.
const PROFILES: [[f32; 4]; 12] = [
    [1.0, 18000.0, 1.0, 0.55],
    [0.72, 16000.0, 1.3, 0.35],
    [1.2, 20000.0, 2.0, 0.7],
    [0.85, 15000.0, 0.7, 1.0],
    [0.9, 8500.0, 1.4, 0.4],
    [1.35, 7500.0, 1.7, 1.0],
    [1.8, 6000.0, 2.0, 0.3],
    [2.4, 6500.0, 2.3, 0.15],
    [1.35, 6500.0, 2.0, 0.5],
    [1.1, 5500.0, 1.6, 0.65],
    [2.0, 7000.0, 2.4, 1.0],
    [0.65, 11000.0, 0.8, 1.0],
];
const TIMES: [f32; 8] = [
    0.0233, 0.0299, 0.0371, 0.0437, 0.0533, 0.0617, 0.0719, 0.0839,
];
struct Network {
    lines: [DelayLine; 8],
    bass: [OnePole; 8],
    high: [OnePole; 8],
    lfo: [Oscillator; 8],
}
impl Network {
    fn new(rate: f32) -> Self {
        Self {
            lines: std::array::from_fn(|_| DelayLine::new((rate * 0.7) as usize)),
            bass: [OnePole::default(); 8],
            high: [OnePole::default(); 8],
            lfo: [Oscillator::default(); 8],
        }
    }
    fn reset(&mut self) {
        for i in 0..8 {
            self.lines[i].reset();
            self.bass[i].reset();
            self.high[i].reset();
            self.lfo[i].reset();
        }
    }
}
pub struct Engine {
    rate: f32,
    controls: Controls<PARAM_COUNT>,
    predelay: [DelayLine; 2],
    early: [DelayLine; 2],
    diffuser: [[Allpass; 4]; 2],
    early_lfo: [Oscillator; 2],
    network: [Network; 2],
    cuts: [[OnePole; 2]; 2],
    space_cuts: [[OnePole; 2]; 2],
    early_feedback: [f32; 2],
    lengths: [[f32; 8]; 2],
    length_ramps: [[DelayRamp; 8]; 2],
    early_ramp: DelayRamp,
    pre_ramp: DelayRamp,
    mode: usize,
    switch_gain: f32,
    gains: [[[f32; 3]; 8]; 2],
    modulation: f32,
    density: f32,
    low_attenuation: f32,
    gain: f32,
    cross: (f32, f32),
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
            predelay: std::array::from_fn(|_| DelayLine::new((rate * 0.51) as usize)),
            early: std::array::from_fn(|_| DelayLine::new((rate * 1.02) as usize)),
            diffuser: std::array::from_fn(|_| {
                std::array::from_fn(|_| Allpass::new((rate * 0.08) as usize))
            }),
            early_lfo: [Oscillator::default(); 2],
            network: std::array::from_fn(|_| Network::new(rate)),
            cuts: [[OnePole::default(); 2]; 2],
            space_cuts: [[OnePole::default(); 2]; 2],
            early_feedback: [0.0; 2],
            lengths: [[0.0; 8]; 2],
            length_ramps: [[DelayRamp::default(); 8]; 2],
            early_ramp: DelayRamp::default(),
            pre_ramp: DelayRamp::default(),
            mode: 0,
            switch_gain: 1.0,
            gains: [[[0.0; 3]; 8]; 2],
            modulation: 0.0,
            density: 0.0,
            low_attenuation: 0.0,
            gain: 1.0,
            cross: (1.0, 0.0),
            levels: Levels::new(rate),
        };
        e.reset();
        e
    }
    pub fn set_parameters(&mut self, values: &[f32; PARAM_COUNT]) {
        self.controls
            .set(std::array::from_fn(|i| SPECS[i].sanitize(values[i])));
    }
    fn configure(&mut self) {
        let v = self.controls.current;
        let profile = PROFILES[self.mode];
        self.early_ramp.set(v[EARLY_SIZE] * self.rate * 0.001);
        self.pre_ramp.set(v[PREDELAY] * self.rate * 0.001);
        let scale = (0.18 + v[LATE_SIZE] * 0.022) * profile[0];
        let angle = v[LATE_CROSS] * 0.01 * std::f32::consts::FRAC_PI_4;
        self.cross = (angle.cos(), angle.sin());
        self.density = profile[3];
        self.low_attenuation = v[LOW_CUT] / (v[LOW_CUT] + 200.0);
        self.modulation = v[LATE_MOD] * 0.000025 * self.rate * profile[2];
        self.gain = sps_dsp::db_to_gain(v[OUTPUT]);
        let bass = v[BASS_XOVER].min(self.rate * 0.2);
        let high = v[HIGH_XOVER].max(bass * 1.2).min(self.rate * 0.45);
        for c in 0..2 {
            self.early_lfo[c].configure(v[EARLY_RATE] * (1.0 + c as f32 * 0.17), self.rate);
            self.space_cuts[c][0].configure(v[LOW_CUT], self.rate);
            self.space_cuts[c][1].configure(v[HIGH_CUT].min(profile[1]), self.rate);
            self.cuts[c][0].configure(v[LOW_CUT], self.rate);
            self.cuts[c][1].configure(v[HIGH_CUT].min(profile[1]), self.rate);
            for (i, time) in TIMES.iter().enumerate() {
                let seconds = time * scale * (1.0 + c as f32 * 0.027);
                self.lengths[c][i] = seconds * self.rate;
                self.length_ramps[c][i].set(self.lengths[c][i]);
                // Use each channel's own physical length for an equal RT60.
                self.network[c].bass[i].configure(bass, self.rate);
                self.network[c].high[i].configure(high, self.rate);
                self.network[c].lfo[i].configure(
                    v[LATE_RATE] * (0.67 + i as f32 * 0.117 + c as f32 * 0.053),
                    self.rate,
                );
            }
        }
        for c in 0..2 {
            for (i, time) in TIMES.iter().enumerate() {
                let seconds = time * scale * (1.0 + c as f32 * 0.027);
                self.gains[c][i] = [
                    (-std::f32::consts::LN_10 * 3.0 * seconds / (v[DECAY] * v[BASS_MULT])).exp(),
                    (-std::f32::consts::LN_10 * 3.0 * seconds / v[DECAY]).exp(),
                    (-std::f32::consts::LN_10 * 3.0 * seconds / (v[DECAY] * v[HIGH_MULT])).exp(),
                ];
            }
        }
    }

    pub fn reset(&mut self) {
        self.controls.reset();
        self.mode = self.controls.current[MODE] as usize;
        self.switch_gain = 1.0;
        for c in 0..2 {
            self.predelay[c].reset();
            self.early[c].reset();
            self.early_lfo[c].reset();
            self.network[c].reset();
            for d in &mut self.diffuser[c] {
                d.reset();
            }
            for cut in &mut self.cuts[c] {
                cut.reset();
            }
            for cut in &mut self.space_cuts[c] {
                cut.reset();
            }
        }
        self.early_feedback = [0.0; 2];
        self.levels.reset();
        self.configure();
        self.pre_ramp.reset();
        self.early_ramp.reset();
        for channel in &mut self.length_ramps {
            for ramp in channel {
                ramp.reset();
            }
        }
    }
    pub fn process_frame(&mut self, input: [f32; 2], channels: usize) -> [f32; 2] {
        if self.controls.tick() {
            self.configure();
        }
        let v = self.controls.current;
        if v[MODE] as usize != self.mode {
            self.switch_gain = (self.switch_gain - 1.0 / (self.rate * 0.005)).max(0.0);
            if self.switch_gain == 0.0 {
                self.mode = v[MODE] as usize;
                self.configure();
            }
        } else {
            self.switch_gain = (self.switch_gain + 1.0 / (self.rate * 0.005)).min(1.0);
        }
        let early_size = self.early_ramp.tick();
        let predelay = self.pre_ramp.tick();
        let dry = input.map(finite);
        let source = if channels == 1 { [dry[0]; 2] } else { dry };
        let mut pre = [0.0; 2];
        let mut early = [0.0; 2];
        for c in 0..2 {
            let x = ceiling(source[c] + self.early_feedback[c] * v[SPACE] * 0.0095);
            self.predelay[c].push(x);
            pre[c] = if predelay < 1.0 {
                x
            } else {
                self.predelay[c].read(predelay + 1.0)
            };
            self.early[c].push(pre[c]);
            let lfo = self.early_lfo[c].tick().0 * v[EARLY_MOD] * self.rate * 0.00001;
            let size = early_size;
            for tap in 0..24 {
                let fraction = (tap as f32 + 1.0) / 24.0;
                let weight = if tap % 3 == 0 {
                    1.0
                } else {
                    v[DIFFUSION] * 0.01
                };
                let sign = if tap % 5 == 0 { -1.0 } else { 1.0 };
                early[c] += self.early[c]
                    .read((size * fraction + lfo * (1.0 - fraction) + 1.0).max(1.0))
                    * weight
                    * sign
                    / 24.0;
            }
        }
        let cross = v[EARLY_CROSS] * 0.005;
        early = [
            early[0] * (1.0 - cross) + early[1] * cross,
            early[1] * (1.0 - cross) + early[0] * cross,
        ];
        let mut injection = [0.0; 2];
        for c in 0..2 {
            let filtered = early[c] - self.space_cuts[c][0].low(early[c]) * self.low_attenuation;
            self.early_feedback[c] = self.space_cuts[c][1].low(filtered);
            injection[c] = pre[c] * (1.0 - v[EARLY_SEND] * 0.01) + early[c] * v[EARLY_SEND] * 0.01;
            for i in 0..4 {
                injection[c] = self.diffuser[c][i].process(
                    injection[c],
                    self.rate * (0.0041 + i as f32 * 0.0037) * (0.3 + self.density),
                    0.55 * self.density,
                );
            }
        }
        let mut reads = [[0.0; 8]; 2];
        let mut late = [0.0; 2];
        for c in 0..2 {
            for (i, read) in reads[c].iter_mut().enumerate() {
                let length = self.length_ramps[c][i].tick()
                    + self.network[c].lfo[i].tick().0 * self.modulation;
                let x = self.network[c].lines[i].read(length);
                let [gb, gm, gh] = self.gains[c][i];
                let high = self.network[c].high[i].low(x);
                let shaped = gh * x + (gm - gh) * high;
                let bass = self.network[c].bass[i].low(shaped);
                *read = shaped + (gb / gm.max(1e-20) - 1.0) * bass;
                late[c] += x * if i % 2 == 0 { 0.18 } else { -0.18 };
            }
        }
        // Householder reflections within channels and orthogonal stereo rotation:
        // both matrices preserve energy, with decay controlled solely by shelves.
        let sums = [
            reads[0].iter().sum::<f32>() * 0.25,
            reads[1].iter().sum::<f32>() * 0.25,
        ];
        for (i, (left, right)) in reads[0].iter().zip(&reads[1]).enumerate() {
            let a = sums[0] - left;
            let b = sums[1] - right;
            let feedback = [
                self.cross.0 * a + self.cross.1 * b,
                self.cross.0 * b - self.cross.1 * a,
            ];
            for c in 0..2 {
                self.network[c].lines[i].push(ceiling(feedback[c] + injection[c] * 0.24));
            }
        }
        let wet = std::array::from_fn(|c| {
            let x = early[c] * (1.0 - v[DEPTH] * 0.01) + late[c] * v[DEPTH] * 0.01;
            let filtered = x - self.cuts[c][0].low(x) * self.low_attenuation;
            self.cuts[c][1].low(filtered) * self.switch_gain
        });
        let output = mix_frame(dry, wet, v[MIX], self.gain, v[BYPASS], channels);
        self.levels.process(dry, output);
        output
    }
    pub fn meters(&mut self) -> &[f32; 2] {
        self.levels.read()
    }
}
