//! Independent stereo frequency-shifting echo. Feedback shift is additive Hz.
pub mod params;
use params::*;
pub use sps_dsp::AtomicValue;
use sps_dsp::{meter::Levels, time::*};
pub type MeterBus = sps_dsp::meter::MeterBus<2>;
pub const METER_COUNT: usize = 2;
pub const METER_NAMES: [&str; 2] = ["Input", "Output"];
pub struct Engine {
    rate: f32,
    controls: Controls<PARAM_COUNT>,
    delay: [DelayLine; 2],
    shift: [QuadratureShifter; 2],
    low: [OnePole; 2],
    high: [OnePole; 2],
    lengths: f32,
    length_ramp: DelayRamp,
    gain: f32,
    tempo: f32,
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
            delay: std::array::from_fn(|_| DelayLine::new((rate * 1.05) as usize)),
            shift: std::array::from_fn(|_| QuadratureShifter::default()),
            low: [OnePole::default(); 2],
            high: [OnePole::default(); 2],
            lengths: 0.0,
            length_ramp: DelayRamp::default(),
            gain: 1.0,
            tempo: 120.0,
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
        if t != self.tempo {
            self.tempo = t;
            self.configure();
        }
    }
    fn configure(&mut self) {
        let v = self.controls.current;
        self.lengths = synced_ms(v[TIME], v[SYNC] as usize, v[DIVISION] as usize, self.tempo)
            .clamp(0.01, 1000.0)
            * self.rate
            * 0.001;
        self.length_ramp.set(self.lengths);
        for c in 0..2 {
            self.shift[c].configure(
                v[SHIFT]
                    * if c == 1 && v[STEREO] >= 0.5 {
                        -1.0
                    } else {
                        1.0
                    },
                self.rate,
            );
            self.low[c].configure(v[LOW_CUT], self.rate);
            self.high[c].configure(v[HIGH_CUT], self.rate);
        }
        self.gain = sps_dsp::db_to_gain(v[OUTPUT]);
    }
    pub fn reset(&mut self) {
        self.controls.reset();
        for c in 0..2 {
            self.delay[c].reset();
            self.shift[c].reset();
            self.low[c].reset();
            self.high[c].reset();
        }
        self.levels.reset();
        self.configure();
        self.length_ramp.reset();
    }
    pub fn process_frame(&mut self, input: [f32; 2], channels: usize) -> [f32; 2] {
        if self.controls.tick() {
            self.configure();
        }
        let v = self.controls.current;
        let dry = input.map(finite);
        let source = if channels == 1 { [dry[0]; 2] } else { dry };
        let length = self.length_ramp.tick();
        let mut wet = [0.0; 2];
        for c in 0..2 {
            let delayed = self.delay[c].read(length);
            wet[c] = self.high[c].low(self.low[c].high(self.shift[c].process(delayed)));
            self.delay[c].push(ceiling(source[c] + wet[c] * v[FEEDBACK] * 0.01));
        }
        let output = mix_frame(dry, wet, v[MIX], self.gain, v[BYPASS], channels);
        self.levels.process(dry, output);
        output
    }
    pub fn meters(&mut self) -> &[f32; 2] {
        self.levels.read()
    }
}
