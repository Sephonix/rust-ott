//! Stable plugin schema. Order also defines LV2 control port indices.
use sps_dsp::params::Spec;
pub const MIX: usize = 0;
pub const OUTPUT: usize = 1;
pub const BYPASS: usize = 2;
pub const MODE: usize = 3;
pub const STYLE: usize = 4;
pub const ERA: usize = 5;
pub const TIME_L: usize = 6;
pub const TIME_R: usize = 7;
pub const SYNC_L: usize = 8;
pub const SYNC_R: usize = 9;
pub const DIVISION_L: usize = 10;
pub const DIVISION_R: usize = 11;
pub const SPREAD: usize = 12;
pub const RATIO: usize = 13;
pub const FEEDBACK: usize = 14;
pub const WIDTH: usize = 15;
pub const DRIVE: usize = 16;
pub const AGE: usize = 17;
pub const DIFFUSION: usize = 18;
pub const DIFF_SIZE: usize = 19;
pub const MOD_RATE: usize = 20;
pub const MOD_DEPTH: usize = 21;
pub const WOW: usize = 22;
pub const FLUTTER: usize = 23;
pub const SHIFT: usize = 24;
pub const FREQ_DETUNE: usize = 25;
pub const PITCH: usize = 26;
pub const DETUNE: usize = 27;
pub const PITCH_MIX: usize = 28;
pub const LOW_CUT: usize = 29;
pub const HIGH_CUT: usize = 30;
pub const DUCK: usize = 31;
pub const SPACING: usize = 32;
pub const TAP_A: usize = 33;
pub const TAP_B: usize = 34;
pub const TAP_C: usize = 35;
pub const TAP_D: usize = 36;
pub const SWELL: usize = 37;
pub const PARAM_COUNT: usize = 38;
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
        max: 16.0,
        default: 0.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "style",
        name: "Style",
        min: 0.0,
        max: 4.0,
        default: 0.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "era",
        name: "Era",
        min: 0.0,
        max: 2.0,
        default: 1.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "time_l",
        name: "Delay Left",
        min: 0.01,
        max: 20000.0,
        default: 350.0,
        unit: "ms",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "time_r",
        name: "Delay Right",
        min: 0.01,
        max: 20000.0,
        default: 500.0,
        unit: "ms",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "sync_l",
        name: "Left Sync",
        min: 0.0,
        max: 3.0,
        default: 0.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "sync_r",
        name: "Right Sync",
        min: 0.0,
        max: 3.0,
        default: 0.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "division_l",
        name: "Left Note",
        min: 0.0,
        max: 11.0,
        default: 4.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "division_r",
        name: "Right Note",
        min: 0.0,
        max: 11.0,
        default: 4.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "spread",
        name: "Spread",
        min: -100.0,
        max: 100.0,
        default: 0.0,
        unit: "ms",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "ratio",
        name: "Right Ratio",
        min: 1.0,
        max: 200.0,
        default: 61.8,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "feedback",
        name: "Feedback",
        min: 0.0,
        max: 200.0,
        default: 40.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "width",
        name: "Width",
        min: -100.0,
        max: 100.0,
        default: 100.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "drive",
        name: "Drive",
        min: 0.0,
        max: 36.0,
        default: 0.0,
        unit: "dB",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "age",
        name: "Age / Resonance",
        min: 0.0,
        max: 100.0,
        default: 10.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "diffusion",
        name: "Diffusion",
        min: 0.0,
        max: 100.0,
        default: 0.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "diff_size",
        name: "Diffusion Size",
        min: 1.0,
        max: 100.0,
        default: 30.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "mod_rate",
        name: "Mod Rate",
        min: 0.01,
        max: 20.0,
        default: 0.5,
        unit: "Hz",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "mod_depth",
        name: "Mod Depth",
        min: 0.0,
        max: 100.0,
        default: 0.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "wow",
        name: "Wow",
        min: 0.0,
        max: 100.0,
        default: 10.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "flutter",
        name: "Flutter",
        min: 0.0,
        max: 100.0,
        default: 5.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
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
        id: "freq_detune",
        name: "Frequency Detune",
        min: 0.0,
        max: 100.0,
        default: 0.0,
        unit: "Hz",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "pitch",
        name: "Pitch Shift",
        min: -24.0,
        max: 24.0,
        default: 0.0,
        unit: "st",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "detune",
        name: "Pitch Detune",
        min: 0.0,
        max: 100.0,
        default: 0.0,
        unit: "ct",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "pitch_mix",
        name: "Pitch Blend",
        min: 0.0,
        max: 100.0,
        default: 100.0,
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
        default: 16000.0,
        unit: "Hz",
        logarithmic: true,
        discrete: false,
    },
    Spec {
        id: "duck",
        name: "Ducking",
        min: 0.0,
        max: 100.0,
        default: 50.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "spacing",
        name: "Tap Spacing",
        min: -100.0,
        max: 100.0,
        default: 0.0,
        unit: "%",
        logarithmic: false,
        discrete: false,
    },
    Spec {
        id: "tap_a",
        name: "Tap A",
        min: 0.0,
        max: 1.0,
        default: 1.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "tap_b",
        name: "Tap B",
        min: 0.0,
        max: 1.0,
        default: 1.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "tap_c",
        name: "Tap C",
        min: 0.0,
        max: 1.0,
        default: 1.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "tap_d",
        name: "Tap D",
        min: 0.0,
        max: 1.0,
        default: 1.0,
        unit: "",
        logarithmic: false,
        discrete: true,
    },
    Spec {
        id: "swell",
        name: "Swell Feedback",
        min: 0.0,
        max: 1.0,
        default: 0.0,
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
        MODE => &[
            "Tape",
            "HiFi",
            "Bucket Brigade",
            "Digital",
            "Spectral",
            "Pitch",
            "Reverse Pitch",
            "Ducked BBD",
            "Clarity",
            "Ducked Tape",
            "Ducked Pitch",
            "LoFi",
            "Crystal",
            "Phaser",
            "Rich Pitch",
            "Chrome Tape",
            "Analog",
        ],
        STYLE => &["Single", "Dual", "Ratio", "Ping Pong", "Quad"],
        ERA => &["Past", "Present", "Future"],
        SYNC_L => &["Free", "Straight", "Dotted", "Triplet"],
        SYNC_R => &["Free", "Straight", "Dotted", "Triplet"],
        DIVISION_L => &[
            "1/64", "1/32", "1/16", "1/8", "1/4", "1/2", "1 bar", "2 bars", "4 bars", "8 bars",
            "16 bars", "32 bars",
        ],
        DIVISION_R => &[
            "1/64", "1/32", "1/16", "1/8", "1/4", "1/2", "1 bar", "2 bars", "4 bars", "8 bars",
            "16 bars", "32 bars",
        ],
        _ => &[],
    }
}
pub const FACTORY_NAMES: &[&str] = &[
    "Tape Echo",
    "Stereo Quarter",
    "Ping Pong",
    "Golden Wash",
    "Four Heads",
    "Shimmer",
    "Reverse Cloud",
    "Ducked Lead",
    "Phased Echo",
];
pub fn factory(index: usize) -> [f32; PARAM_COUNT] {
    let mut v = defaults();
    match index {
        0 => {}
        1 => {
            v[MODE] = 12.0;
            v[STYLE] = 1.0;
            v[SYNC_L] = 1.0;
            v[SYNC_R] = 2.0;
            v[FEEDBACK] = 45.0;
            v[MIX] = 30.0;
        }
        2 => {
            v[STYLE] = 3.0;
            v[FEEDBACK] = 55.0;
        }
        3 => {
            v[STYLE] = 2.0;
            v[RATIO] = 61.8;
            v[DIFFUSION] = 75.0;
            v[FEEDBACK] = 75.0;
        }
        4 => {
            v[STYLE] = 4.0;
            v[SPACING] = 25.0;
            v[TIME_L] = 800.0;
            v[MODE] = 15.0;
        }
        5 => {
            v[MODE] = 14.0;
            v[PITCH] = 12.0;
            v[PITCH_MIX] = 60.0;
            v[DIFFUSION] = 80.0;
            v[FEEDBACK] = 65.0;
        }
        6 => {
            v[MODE] = 6.0;
            v[PITCH] = -5.0;
            v[DIFFUSION] = 45.0;
        }
        7 => {
            v[MODE] = 8.0;
            v[DUCK] = 80.0;
            v[FEEDBACK] = 60.0;
        }
        8 => {
            v[MODE] = 13.0;
            v[MOD_DEPTH] = 70.0;
            v[AGE] = 40.0;
        }
        _ => {}
    }
    v
}
