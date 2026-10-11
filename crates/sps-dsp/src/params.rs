//! Shared parameter mapping; each plugin owns its IDs, ordering and defaults.
//! Logarithmic specs require 0 < min < max; linear specs require min < max.

#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub id: &'static str,
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub unit: &'static str,
    pub logarithmic: bool,
    pub discrete: bool,
}

impl Spec {
    pub fn sanitize(self, value: f32) -> f32 {
        let value = if value.is_finite() {
            value.clamp(self.min, self.max)
        } else {
            self.default
        };
        if self.discrete {
            value.round()
        } else {
            value
        }
    }
    pub fn normalize(self, value: f32) -> f32 {
        let v = self.sanitize(value);
        if self.logarithmic {
            (v / self.min).ln() / (self.max / self.min).ln()
        } else {
            (v - self.min) / (self.max - self.min)
        }
    }
    pub fn denormalize(self, value: f32) -> f32 {
        let t = value.clamp(0.0, 1.0);
        self.sanitize(if self.logarithmic {
            self.min * (self.max / self.min).powf(t)
        } else {
            self.min + t * (self.max - self.min)
        })
    }
}
