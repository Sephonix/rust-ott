//! Stable plugin schema. Order also defines LV2 control port indices.
use sps_dsp::params::Spec;
pub const MIX: usize = 0;
pub const OUTPUT: usize = 1;
pub const BYPASS: usize = 2;
pub const MODE: usize = 3;
pub const PREDELAY: usize = 4;
pub const DECAY: usize = 5;
pub const HIGH_CUT: usize = 6;
pub const LOW_CUT: usize = 7;
pub const DEPTH: usize = 8;
pub const EARLY_SIZE: usize = 9;
pub const EARLY_CROSS: usize = 10;
pub const EARLY_RATE: usize = 11;
pub const EARLY_MOD: usize = 12;
pub const EARLY_SEND: usize = 13;
pub const DIFFUSION: usize = 14;
pub const SPACE: usize = 15;
pub const LATE_SIZE: usize = 16;
pub const LATE_CROSS: usize = 17;
pub const LATE_RATE: usize = 18;
pub const LATE_MOD: usize = 19;
pub const BASS_MULT: usize = 20;
pub const BASS_XOVER: usize = 21;
pub const HIGH_MULT: usize = 22;
pub const HIGH_XOVER: usize = 23;
pub const PARAM_COUNT: usize = 24;
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
        id: "mode",
        name: "Character",
        min: 0.0,
        max: 11.0,
        default: 0.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "predelay",
        name: "Predelay",
        min: 0.0,
        max: 500.0,
        default: 15.0,
        unit: "ms",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "decay",
        name: "Decay",
        min: 0.1,
        max: 100.0,
        default: 2.0,
        unit: "s",
        logarithmic: true,
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
        id: "low_cut",
        name: "Low Cut",
        min: 0.0,
        max: 2000.0,
        default: 0.0,
        unit: "Hz",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "depth",
        name: "Late Blend",
        min: 0.0,
        max: 100.0,
        default: 70.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "early_size",
        name: "Early Size",
        min: 1.0,
        max: 1000.0,
        default: 45.0,
        unit: "ms",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "early_cross",
        name: "Early Cross",
        min: 0.0,
        max: 100.0,
        default: 30.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "early_rate",
        name: "Early Mod Rate",
        min: 0.01,
        max: 10.0,
        default: 0.4,
        unit: "Hz",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "early_mod",
        name: "Early Mod Depth",
        min: 0.0,
        max: 100.0,
        default: 10.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "early_send",
        name: "Early Send",
        min: 0.0,
        max: 100.0,
        default: 60.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "diffusion",
        name: "Early Diffusion",
        min: 0.0,
        max: 100.0,
        default: 85.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "space",
        name: "Space",
        min: 0.0,
        max: 100.0,
        default: 0.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "late_size",
        name: "Late Size",
        min: 0.0,
        max: 100.0,
        default: 50.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "late_cross",
        name: "Late Cross",
        min: 0.0,
        max: 100.0,
        default: 60.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "late_rate",
        name: "Late Mod Rate",
        min: 0.01,
        max: 10.0,
        default: 0.5,
        unit: "Hz",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "late_mod",
        name: "Late Mod Depth",
        min: 0.0,
        max: 100.0,
        default: 15.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "bass_mult",
        name: "Bass Decay",
        min: 0.25,
        max: 4.0,
        default: 1.2,
        unit: "x",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "bass_xover",
        name: "Bass Crossover",
        min: 20.0,
        max: 4000.0,
        default: 600.0,
        unit: "Hz",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "high_mult",
        name: "High Decay",
        min: 0.1,
        max: 2.0,
        default: 0.5,
        unit: "x",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "high_xover",
        name: "High Crossover",
        min: 500.0,
        max: 20000.0,
        default: 6000.0,
        unit: "Hz",
        logarithmic: true,
        discrete: false,
    },
];
pub fn defaults() -> [f32; PARAM_COUNT] {
    std::array::from_fn(|i| SPECS[i].default)
}
pub fn choices(index: usize) -> &'static [&'static str] {
    match index {
        MODE => &[
            "Large Room",
            "Medium Room",
            "Bright Hall",
            "Chamber",
            "Dark Room",
            "Dark Chamber",
            "Dark Space",
            "Vast Space",
            "Small Space",
            "Centered Space",
            "Deep Space",
            "Dense Room",
        ],
        _ => &[],
    }
}
pub const FACTORY_NAMES: &[&str] = &[
    "Studio",
    "Drum Room",
    "Concert Hall",
    "Deep Space",
    "Gated Energy",
    "Space Echo",
];
pub fn factory(index: usize) -> [f32; PARAM_COUNT] {
    let mut v = defaults();
    match index {
        0 => {}
        1 => {
            v[DECAY] = 0.6;
            v[EARLY_SIZE] = 18.0;
            v[LATE_SIZE] = 20.0;
            v[DEPTH] = 45.0;
            v[MODE] = 11.0;
        }
        2 => {
            v[DECAY] = 3.5;
            v[PREDELAY] = 28.0;
            v[MODE] = 2.0;
            v[LATE_SIZE] = 75.0;
        }
        3 => {
            v[DECAY] = 15.0;
            v[PREDELAY] = 80.0;
            v[MODE] = 10.0;
            v[LATE_MOD] = 55.0;
            v[HIGH_CUT] = 7000.0;
        }
        4 => {
            v[DEPTH] = 0.0;
            v[EARLY_SIZE] = 240.0;
            v[DIFFUSION] = 95.0;
            v[EARLY_MOD] = 0.0;
        }
        5 => {
            v[SPACE] = 75.0;
            v[DEPTH] = 20.0;
            v[PREDELAY] = 180.0;
            v[EARLY_SIZE] = 180.0;
        }
        _ => {}
    }
    v
}
