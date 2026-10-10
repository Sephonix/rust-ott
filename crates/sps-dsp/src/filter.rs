//! Phase matched, fourth order Linkwitz–Riley crossover.
use std::f64::consts::{PI, SQRT_2};

#[derive(Clone, Copy)]
pub struct Coefficients {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
}

impl Coefficients {
    fn butterworth(hz: f32, rate: f32, high: bool) -> Self {
        let k = (PI * hz as f64 / rate as f64).tan();
        let norm = 1.0 / (1.0 + SQRT_2 * k + k * k);
        let b0 = if high { norm } else { k * k * norm };
        Self {
            b0,
            b1: if high { -2.0 * b0 } else { 2.0 * b0 },
            b2: b0,
            a1: 2.0 * (k * k - 1.0) * norm,
            a2: (1.0 - SQRT_2 * k + k * k) * norm,
        }
    }
    fn allpass(hz: f32, rate: f32) -> Self {
        let c = Self::butterworth(hz, rate, false);
        Self {
            b0: c.a2,
            b1: c.a1,
            b2: 1.0,
            a1: c.a1,
            a2: c.a2,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Biquad {
    z1: f64,
    z2: f64,
}
impl Biquad {
    #[inline]
    fn process(&mut self, x: f64, c: Coefficients) -> f64 {
        let y = c.b0 * x + self.z1;
        self.z1 = c.b1 * x - c.a1 * y + self.z2;
        self.z2 = c.b2 * x - c.a2 * y;
        // Flush tiny recursive state without relying on a platform-specific FTZ mode.
        if self.z1.abs() < 1e-30 {
            self.z1 = 0.0;
        }
        if self.z2.abs() < 1e-30 {
            self.z2 = 0.0;
        }
        y
    }
}

#[derive(Default)]
struct Channel {
    low: [Biquad; 2],
    upper: [Biquad; 2],
    mid: [Biquad; 2],
    high: [Biquad; 2],
    align: Biquad,
}

pub struct Crossover {
    channels: [Channel; 2],
    low_lp: Coefficients,
    low_hp: Coefficients,
    high_lp: Coefficients,
    high_hp: Coefficients,
    align: Coefficients,
}

impl Crossover {
    pub fn new(rate: f32, low: f32, high: f32) -> Self {
        Self {
            channels: Default::default(),
            low_lp: Coefficients::butterworth(low, rate, false),
            low_hp: Coefficients::butterworth(low, rate, true),
            high_lp: Coefficients::butterworth(high, rate, false),
            high_hp: Coefficients::butterworth(high, rate, true),
            align: Coefficients::allpass(high, rate),
        }
    }
    pub fn configure(&mut self, rate: f32, low: f32, high: f32) {
        self.low_lp = Coefficients::butterworth(low, rate, false);
        self.low_hp = Coefficients::butterworth(low, rate, true);
        self.high_lp = Coefficients::butterworth(high, rate, false);
        self.high_hp = Coefficients::butterworth(high, rate, true);
        self.align = Coefficients::allpass(high, rate);
    }
    pub fn reset(&mut self) {
        self.channels = Default::default();
    }
    #[inline]
    pub fn split(&mut self, x: f32, channel: usize) -> [f32; 3] {
        let c = &mut self.channels[channel];
        let x = x as f64;
        let low = c.low[0].process(x, self.low_lp);
        let low = c.low[1].process(low, self.low_lp);
        let low = c.align.process(low, self.align);
        let upper = c.upper[0].process(x, self.low_hp);
        let upper = c.upper[1].process(upper, self.low_hp);
        let mid = c.mid[0].process(upper, self.high_lp);
        let mid = c.mid[1].process(mid, self.high_lp);
        let high = c.high[0].process(upper, self.high_hp);
        let high = c.high[1].process(high, self.high_hp);
        [low as f32, mid as f32, high as f32]
    }
}
