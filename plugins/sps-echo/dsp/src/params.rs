//! Stable plugin schema. Order also defines LV2 control port indices.
use sps_dsp::params::Spec;
pub const MIX: usize = 0;
pub const OUTPUT: usize = 1;
pub const BYPASS: usize = 2;
pub const TIME: usize = 3;
pub const SYNC: usize = 4;
pub const DIVISION: usize = 5;
pub const SHIFT: usize = 6;
pub const FEEDBACK: usize = 7;
pub const LOW_CUT: usize = 8;
pub const HIGH_CUT: usize = 9;
pub const STEREO: usize = 10;
pub const PARAM_COUNT: usize = 11;
pub const SPECS: [Spec; PARAM_COUNT] = [
    Spec {
        id: "mix",
        name: "Mix",
        min: 0.0,
        max: 100.0,
        default: 35.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "output",
        name: "Output",
        min: -24.0,
        max: 24.0,
        default: 0.0,
        unit: "dB",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "bypass",
        name: "Bypass",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "time",
        name: "Delay",
        min: 0.01,
        max: 1000.0,
        default: 250.0,
        unit: "ms",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "sync",
        name: "Sync",
        min: 0.0,
        max: 3.0,
        default: 0.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "division",
        name: "Note",
        min: 0.0,
        max: 11.0,
        default: 4.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "shift",
        name: "Frequency Shift",
        min: -1000.0,
        max: 1000.0,
        default: 0.0,
        unit: "Hz",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "feedback",
        name: "Feedback",
        min: 0.0,
        max: 100.0,
        default: 40.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "low_cut",
        name: "Low Cut",
        min: 0.0,
        max: 2000.0,
        default: 20.0,
        unit: "Hz",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "high_cut",
        name: "High Cut",
        min: 200.0,
        max: 20000.0,
        default: 12000.0,
        unit: "Hz",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "stereo",
        name: "Stereo Shift",
        min: 0.0,
        max: 1.0,
        default: 1.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
];
pub fn defaults() -> [f32; PARAM_COUNT] {
    std::array::from_fn(|i| SPECS[i].default)
}
pub fn choices(index: usize) -> &'static [&'static str] {
    match index {
        SYNC => &["Free", "Straight", "Dotted", "Triplet"],
        STEREO => &["Mono shift", "Opposing stereo"],
        DIVISION => &[
            "1/64", "1/32", "1/16", "1/8", "1/4", "1/2", "1 bar", "2 bars", "4 bars", "8 bars",
            "16 bars", "32 bars",
        ],
        _ => &[],
    }
}
pub const FACTORY_NAMES: &[&str] = &[
    "Dub",
    "Ascending",
    "Descending",
    "Stereo Orbit",
    "Barberpole",
];
pub fn factory(index: usize) -> [f32; PARAM_COUNT] {
    let mut v = defaults();
    match index {
        0 => {}
        1 => {
            v[SHIFT] = 17.0;
            v[FEEDBACK] = 70.0;
        }
        2 => {
            v[SHIFT] = -27.0;
            v[FEEDBACK] = 75.0;
        }
        3 => {
            v[SHIFT] = 0.8;
            v[TIME] = 12.0;
            v[FEEDBACK] = 70.0;
        }
        4 => {
            v[SHIFT] = 3.0;
            v[TIME] = 0.1;
            v[FEEDBACK] = 75.0;
        }
        _ => {}
    }
    v
}
