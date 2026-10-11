use sps_ott_dsp::{compression_gain, db_to_gain, gain_to_db, params::*, Engine};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct TrackingAllocator;
thread_local! { static TRACK: Cell<bool> = const { Cell::new(false) }; static ALLOCS: Cell<usize> = const { Cell::new(0) }; }
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        TRACK.with(|track| {
            if track.get() {
                ALLOCS.with(|n| n.set(n.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        unsafe { System.dealloc(p, layout) }
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        TRACK.with(|track| {
            if track.get() {
                ALLOCS.with(|n| n.set(n.get() + 1));
            }
        });
        unsafe { System.realloc(p, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

fn engine(values: &[f32; PARAM_COUNT], rate: f32) -> Engine {
    let mut e = Engine::new(rate);
    e.set_parameters(values);
    e.reset();
    e
}
fn close(a: f32, b: f32, tolerance: f32) {
    assert!((a - b).abs() < tolerance, "{a} != {b} ±{tolerance}");
}

#[test]
fn factory_preset_is_exact_and_ids_are_unique() {
    let v = defaults();
    close(v[LOW_CROSS], 88.3, 1e-4);
    close(v[HIGH_CROSS], 2500.0, 1e-4);
    for (b, (attack, release, down, ratio, up, out)) in [
        (47.8, 282.0, -33.8, 66.7, -40.8, 10.3),
        (22.4, 282.0, -30.3, 66.7, -41.8, 5.7),
        (13.5, 132.0, -35.5, 1000.0, -40.8, 10.3),
    ]
    .into_iter()
    .enumerate()
    {
        for (f, x) in [
            (ATTACK, attack),
            (RELEASE, release),
            (DOWN_THRESHOLD, down),
            (DOWN_RATIO, ratio),
            (UP_THRESHOLD, up),
            (UP_RATIO, 4.17),
            (INPUT_GAIN, 5.2),
            (BAND_OUTPUT, out),
        ] {
            close(v[band_index(b, f)], x, 1e-4);
        }
    }
    for (i, s) in SPECS.iter().enumerate() {
        assert!(SPECS[..i].iter().all(|p| p.id != s.id));
        assert_eq!(s.sanitize(s.default), s.default);
    }
}
#[test]
fn downward_upward_and_infinite_transfer_curves() {
    close(
        compression_gain(-10.0, -30.0, 4.0, -40.0, 4.0, 0.0),
        -15.0,
        1e-5,
    );
    close(
        compression_gain(-50.0, -30.0, 4.0, -40.0, 4.0, 0.0),
        7.5,
        1e-5,
    );
    close(
        compression_gain(-10.0, -30.0, 1000.0, -40.0, 4.0, 0.0),
        -20.0,
        1e-5,
    );
    close(
        compression_gain(-35.0, -30.0, 4.0, -40.0, 4.0, 0.0),
        0.0,
        1e-5,
    );
    close(
        compression_gain(-120.0, -30.0, 4.0, -40.0, 4.0, 6.0),
        0.0,
        1e-5,
    );
    assert!(compression_gain(-80.0, -30.0, 4.0, -40.0, 4.0, 6.0) < 24.0);
}
#[test]
fn soft_knee_is_continuous_at_both_boundaries() {
    for threshold in [-30.0, -40.0] {
        for edge in [threshold - 3.0, threshold + 3.0] {
            let a = compression_gain(edge - 0.0001, -30.0, 66.7, -40.0, 4.17, 6.0);
            let b = compression_gain(edge + 0.0001, -30.0, 66.7, -40.0, 4.17, 6.0);
            close(a, b, 0.0003);
        }
    }
}
#[test]
fn crossover_recombination_is_flat_at_and_around_both_splits() {
    let mut v = defaults();
    v[AMOUNT] = 0.0;
    for rate in [44100.0, 48000.0, 96000.0] {
        for hz in [30.0, 88.3, 180.0, 1000.0, 2500.0, 7000.0, 16000.0] {
            let mut e = engine(&v, rate);
            let mut input_power = 0.0f64;
            let mut output_power = 0.0f64;
            for i in 0..rate as usize {
                let x = (i as f32 * hz * std::f32::consts::TAU / rate).sin() * 0.2;
                let y = e.process_frame([x, x], 2)[0];
                if i >= rate as usize / 2 {
                    input_power += (x * x) as f64;
                    output_power += (y * y) as f64;
                }
            }
            let db = 10.0 * (output_power / input_power).log10();
            assert!(db.abs() < 0.025, "rate {rate} hz {hz}: {db} dB");
        }
    }
}
#[test]
fn settled_low_band_matches_the_preset_transfer_function() {
    let v = defaults();
    let mut e = engine(&v, 48000.0);
    let x = db_to_gain(-20.0);
    let mut y = 0.0;
    for _ in 0..96000 {
        y = e.process_frame([x, x], 2)[0];
    }
    let level = -20.0 + v[band_index(0, INPUT_GAIN)];
    let gain = compression_gain(level, -33.8, 66.7, -40.8, 4.17, 6.0);
    close(gain_to_db(y.abs()), level + gain + 10.3, 0.03);
}
#[test]
fn stereo_link_preserves_antiphase_and_one_sided_audio() {
    let mut e = Engine::new(48000.0);
    for i in 0..48000 {
        let x = (i as f32 * 0.031).sin() * 0.3;
        let y = e.process_frame([x, -x], 2);
        close(y[0], -y[1], 1e-6);
    }
    e.reset();
    for i in 0..48000 {
        let x = (i as f32 * 0.031).sin() * 0.3;
        let y = e.process_frame([x, 0.0], 2);
        assert_eq!(y[1], 0.0);
    }
}
#[test]
fn output_gain_and_phase_aligned_mix_are_correct() {
    let mut v = defaults();
    v[AMOUNT] = 0.0;
    v[OUTPUT] = 6.0;
    let mut e = engine(&v, 48000.0);
    let mut y = 0.0;
    for _ in 0..48000 {
        y = e.process_frame([0.1, 0.1], 2)[0];
    }
    close(y, 0.1 * db_to_gain(6.0), 1e-5);
    let mut dry = defaults();
    dry[MIX] = 0.0;
    let mut amount_zero = defaults();
    amount_zero[AMOUNT] = 0.0;
    let mut a = engine(&dry, 48000.0);
    let mut b = engine(&amount_zero, 48000.0);
    for i in 0..24000 {
        let x = (i as f32 * 0.31).sin();
        close(
            a.process_frame([x, x], 2)[0],
            b.process_frame([x, x], 2)[0],
            1e-5,
        );
    }
}
#[test]
fn bypass_is_sample_exact_and_reset_removes_history() {
    let mut v = defaults();
    v[BYPASS] = 1.0;
    let mut e = engine(&v, 48000.0);
    for i in 0..1000 {
        let x = (i as f32 * 0.83).sin();
        assert_eq!(e.process_frame([x, -x], 2), [x, -x]);
    }
    e.reset();
    for _ in 0..1000 {
        assert_eq!(e.process_frame([0.0, 0.0], 2), [0.0, 0.0]);
    }
}
#[test]
fn silence_nonfinite_controls_and_audio_are_safe() {
    for rate in [8000.0, 44100.0, 192000.0, 768000.0] {
        let mut e = Engine::new(rate);
        for _ in 0..10000 {
            assert_eq!(e.process_frame([0.0, 0.0], 2), [0.0, 0.0]);
        }
        e.set_parameters(&[f32::NAN; PARAM_COUNT]);
        for _ in 0..10000 {
            assert!(e
                .process_frame([f32::NAN, f32::INFINITY], 2)
                .iter()
                .all(|v| v.is_finite()));
        }
    }
}
#[test]
fn automation_extremes_stay_finite_and_mute_solo_work() {
    let mut e = Engine::new(48000.0);
    for block in 0..250 {
        let v = SPECS.map(|s| if block % 2 == 0 { s.min } else { s.max });
        e.set_parameters(&v);
        for i in 0..128 {
            let y = e.process_frame([(i as f32 * 0.2).sin(), (i as f32 * 0.3).sin()], 2);
            assert!(y.iter().all(|s| s.is_finite() && s.abs() < 1000.0));
        }
    }
    let mut v = defaults();
    v[band_index(0, MUTE)] = 1.0;
    let mut e = engine(&v, 48000.0);
    let mut y = 0.0;
    for _ in 0..48000 {
        y = e.process_frame([0.1, 0.1], 2)[0];
    }
    assert!(y.abs() < 1e-5);
    v[band_index(0, MUTE)] = 0.0;
    v[band_index(2, SOLO)] = 1.0;
    e.set_parameters(&v);
    e.reset();
    for _ in 0..48000 {
        y = e.process_frame([0.1, 0.1], 2)[0];
    }
    assert!(y.abs() < 1e-5);
}
#[test]
fn mono_and_stereo_processing_match_for_dual_mono() {
    let mut a = Engine::new(48000.0);
    let mut b = Engine::new(48000.0);
    for i in 0..24000 {
        let x = (i as f32 * 0.03).sin() * 0.2;
        close(
            a.process_frame([x, 0.0], 1)[0],
            b.process_frame([x, x], 2)[0],
            1e-6,
        );
    }
}
#[test]
fn rms_and_peak_modes_are_distinct_and_mode_changes_are_finite() {
    let mut v = defaults();
    let mut rms = engine(&v, 48000.0);
    v[DETECTION] = 1.0;
    let mut peak = engine(&v, 48000.0);
    for i in 0..48000 {
        let x = (i as f32 * 0.131).sin() * 0.1;
        rms.process_frame([x, x], 2);
        peak.process_frame([x, x], 2);
    }
    assert!((rms.meters()[3] - peak.meters()[3]).abs() > 0.1);
    for mode in [0.0, 1.0, 0.0] {
        v[DETECTION] = mode;
        peak.set_parameters(&v);
        for _ in 0..4800 {
            assert!(peak.process_frame([0.1, 0.1], 2)[0].is_finite());
        }
    }
}
#[test]
fn real_time_path_does_not_allocate_even_during_automation() {
    let mut e = Engine::new(48000.0);
    let mut v = defaults();
    ALLOCS.with(|n| n.set(0));
    TRACK.with(|t| t.set(true));
    for block in 0..100 {
        v[AMOUNT] = block as f32;
        e.set_parameters(&v);
        for i in 0..128 {
            e.process_frame([i as f32 / 128.0, -0.1], 2);
        }
    }
    TRACK.with(|t| t.set(false));
    assert_eq!(ALLOCS.with(|n| n.get()), 0);
}
