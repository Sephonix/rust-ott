//! One stable parameter table shared by the DSP, all wrappers, and the GUI.
pub const GLOBAL_COUNT: usize = 8;
pub const BAND_STRIDE: usize = 10;
pub const PARAM_COUNT: usize = GLOBAL_COUNT + 3 * BAND_STRIDE;
pub const AMOUNT: usize = 0;
pub const OUTPUT: usize = 1;
pub const MIX: usize = 2;
pub const LOW_CROSS: usize = 3;
pub const HIGH_CROSS: usize = 4;
pub const DETECTION: usize = 5;
pub const KNEE: usize = 6;
pub const BYPASS: usize = 7;
pub const ATTACK: usize = 0;
pub const RELEASE: usize = 1;
pub const DOWN_THRESHOLD: usize = 2;
pub const DOWN_RATIO: usize = 3;
pub const UP_THRESHOLD: usize = 4;
pub const UP_RATIO: usize = 5;
pub const INPUT_GAIN: usize = 6;
pub const BAND_OUTPUT: usize = 7;
pub const MUTE: usize = 8;
pub const SOLO: usize = 9;
pub const INFINITE_RATIO: f32 = 1000.0;
pub const BAND_NAMES: [&str; 3] = ["Low", "Mid", "High"];

pub const fn band_index(band: usize, field: usize) -> usize {
    GLOBAL_COUNT + band * BAND_STRIDE + field
}

pub use sps_dsp::params::Spec;

macro_rules! spec {
    ($id:literal, $name:literal, $min:expr, $max:expr, $default:expr, $unit:literal, $log:expr, $discrete:expr) => {
        Spec {
            id: $id,
            name: $name,
            min: $min,
            max: $max,
            default: $default,
            unit: $unit,
            logarithmic: $log,
            discrete: $discrete,
        }
    };
}

pub const SPECS: [Spec; PARAM_COUNT] = [
    spec!("amount", "Amount", 0.0, 100.0, 100.0, "%", false, false),
    spec!("output", "Output", -36.0, 18.0, 0.0, "dB", false, false),
    spec!("mix", "Mix", 0.0, 100.0, 100.0, "%", false, false),
    spec!("lo_x", "Low / Mid", 20.0, 1000.0, 88.3, "Hz", true, false),
    spec!(
        "hi_x",
        "Mid / High",
        1000.0,
        18000.0,
        2500.0,
        "Hz",
        true,
        false
    ),
    spec!("detect", "Detection", 0.0, 1.0, 0.0, "", false, true),
    spec!("knee", "Soft knee", 0.0, 1.0, 1.0, "", false, true),
    spec!("bypass", "Bypass", 0.0, 1.0, 0.0, "", false, true),
    spec!("l_atk", "Attack", 0.1, 200.0, 47.8, "ms", true, false),
    spec!("l_rel", "Release", 5.0, 2000.0, 282.0, "ms", true, false),
    spec!(
        "l_dt",
        "Down threshold",
        -72.0,
        0.0,
        -33.8,
        "dB",
        false,
        false
    ),
    spec!("l_dr", "Down ratio", 1.0, 1000.0, 66.7, ":1", true, false),
    spec!(
        "l_ut",
        "Up threshold",
        -72.0,
        0.0,
        -40.8,
        "dB",
        false,
        false
    ),
    spec!("l_ur", "Up ratio", 1.0, 20.0, 4.17, ":1", true, false),
    spec!("l_in", "Input", -24.0, 24.0, 5.2, "dB", false, false),
    spec!("l_out", "Output", -24.0, 24.0, 10.3, "dB", false, false),
    spec!("l_mute", "Mute", 0.0, 1.0, 0.0, "", false, true),
    spec!("l_solo", "Solo", 0.0, 1.0, 0.0, "", false, true),
    spec!("m_atk", "Attack", 0.1, 200.0, 22.4, "ms", true, false),
    spec!("m_rel", "Release", 5.0, 2000.0, 282.0, "ms", true, false),
    spec!(
        "m_dt",
        "Down threshold",
        -72.0,
        0.0,
        -30.3,
        "dB",
        false,
        false
    ),
    spec!("m_dr", "Down ratio", 1.0, 1000.0, 66.7, ":1", true, false),
    spec!(
        "m_ut",
        "Up threshold",
        -72.0,
        0.0,
        -41.8,
        "dB",
        false,
        false
    ),
    spec!("m_ur", "Up ratio", 1.0, 20.0, 4.17, ":1", true, false),
    spec!("m_in", "Input", -24.0, 24.0, 5.2, "dB", false, false),
    spec!("m_out", "Output", -24.0, 24.0, 5.7, "dB", false, false),
    spec!("m_mute", "Mute", 0.0, 1.0, 0.0, "", false, true),
    spec!("m_solo", "Solo", 0.0, 1.0, 0.0, "", false, true),
    spec!("h_atk", "Attack", 0.1, 200.0, 13.5, "ms", true, false),
    spec!("h_rel", "Release", 5.0, 2000.0, 132.0, "ms", true, false),
    spec!(
        "h_dt",
        "Down threshold",
        -72.0,
        0.0,
        -35.5,
        "dB",
        false,
        false
    ),
    spec!("h_dr", "Down ratio", 1.0, 1000.0, 1000.0, ":1", true, false),
    spec!(
        "h_ut",
        "Up threshold",
        -72.0,
        0.0,
        -40.8,
        "dB",
        false,
        false
    ),
    spec!("h_ur", "Up ratio", 1.0, 20.0, 4.17, ":1", true, false),
    spec!("h_in", "Input", -24.0, 24.0, 5.2, "dB", false, false),
    spec!("h_out", "Output", -24.0, 24.0, 10.3, "dB", false, false),
    spec!("h_mute", "Mute", 0.0, 1.0, 0.0, "", false, true),
    spec!("h_solo", "Solo", 0.0, 1.0, 0.0, "", false, true),
];

pub fn defaults() -> [f32; PARAM_COUNT] {
    SPECS.map(|s| s.default)
}
