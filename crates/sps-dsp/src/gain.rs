//! Amplitude gain conversions. Meter readings have a -120 dB floor.

#[inline]
pub fn db_to_gain(db: f32) -> f32 {
    (db * std::f32::consts::LN_10 / 20.0).exp()
}
#[inline]
pub fn gain_to_db(gain: f32) -> f32 {
    20.0 * gain.max(1e-6).log10()
}
