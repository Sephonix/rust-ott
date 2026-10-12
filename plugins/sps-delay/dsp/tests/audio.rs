use sps_delay_dsp::{params::*, Engine};
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

fn engine(v: &[f32; PARAM_COUNT], rate: f32) -> Engine {
    let mut e = Engine::new(rate);
    e.set_parameters(v);
    e.reset();
    e
}
#[test]
fn schema_and_factory_presets_are_valid() {
    for (i, s) in SPECS.iter().enumerate() {
        assert!(SPECS[..i].iter().all(|p| p.id != s.id));
        assert_eq!(s.sanitize(s.default), s.default);
    }
    for i in 0..FACTORY_NAMES.len() {
        let p = factory(i);
        for (s, v) in SPECS.iter().zip(p) {
            assert_eq!(s.sanitize(v), v);
        }
    }
}
#[test]
fn silence_reset_invalid_input_and_true_bypass() {
    for rate in [8000.0, 44100.0, 48000.0, 96000.0, 192000.0] {
        let mut v = defaults();
        let mut e = engine(&v, rate);
        for _ in 0..1000 {
            assert_eq!(e.process_frame([0.0; 2], 2), [0.0; 2]);
        }
        e.process_frame([0.5, -0.4], 2);
        for _ in 0..10000 {
            e.process_frame([0.0; 2], 2);
        }
        e.reset();
        for _ in 0..1000 {
            assert_eq!(e.process_frame([0.0; 2], 2), [0.0; 2]);
        }
        v[BYPASS] = 1.0;
        e.set_parameters(&v);
        e.reset();
        assert_eq!(e.process_frame([0.1, -0.2], 2), [0.1, -0.2]);
        for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, f32::MAX] {
            assert!(e.process_frame([x, x], 2).iter().all(|x| x.is_finite()));
        }
        v.fill(f32::NAN);
        e.set_parameters(&v);
        e.reset();
        assert!(e.process_frame([0.1, -0.1], 1)[0].is_finite());
    }
    let mut e = Engine::new(768000.0);
    assert!(e.process_frame([0.0; 2], 2).iter().all(|x| x.is_finite()));
}
#[test]
fn process_and_automation_allocate_nothing() {
    let mut v = defaults();
    let mut e = engine(&v, 48000.0);
    ALLOCS.with(|n| n.set(0));
    TRACK.with(|t| t.set(true));
    for block in 0..1000 {
        for (i, s) in SPECS.iter().enumerate() {
            v[i] = if block % 2 == 0 { s.min } else { s.max };
        }
        e.set_parameters(&v);
        for _ in 0..32 {
            let y = e.process_frame([0.1, -0.2], 2);
            assert!(y.iter().all(|x| x.is_finite()));
        }
        e.meters();
    }
    TRACK.with(|t| t.set(false));
    assert_eq!(ALLOCS.with(|n| n.get()), 0);
}
#[test]
fn settled_bypass_automation_reaches_exact_input() {
    let mut v = defaults();
    let mut e = engine(&v, 48000.0);
    v[BYPASS] = 1.0;
    e.set_parameters(&v);
    for _ in 0..24000 {
        e.process_frame([0.1, -0.2], 2);
    }
    assert_eq!(e.process_frame([0.1, -0.2], 2), [0.1, -0.2]);
}

fn clean() -> [f32; PARAM_COUNT] {
    let mut v = defaults();
    v[MIX] = 100.0;
    v[MODE] = 12.0;
    v[ERA] = 2.0;
    v[FEEDBACK] = 50.0;
    v[TIME_L] = 100.0;
    v[TIME_R] = 150.0;
    v[LOW_CUT] = 0.0;
    v[HIGH_CUT] = 20000.0;
    v[AGE] = 0.0;
    v[WOW] = 0.0;
    v[FLUTTER] = 0.0;
    v[MOD_DEPTH] = 0.0;
    v
}
#[test]
fn echo_timing_feedback_and_ping_pong_routing() {
    let mut v = clean();
    v[STYLE] = 3.0;
    let mut e = engine(&v, 48000.0);
    let mut first = [0usize; 2];
    let mut peak = [0.0f32; 2];
    for i in 0..24000 {
        let y = e.process_frame(if i == 0 { [1.0, 1.0] } else { [0.0; 2] }, 2);
        for c in 0..2 {
            if y[c].abs() > peak[c] {
                peak[c] = y[c].abs();
                first[c] = i;
            }
        }
    }
    assert_eq!(first, [4800, 12000]);
    assert!(peak[0] > 0.7);
    assert!((peak[1] / peak[0] - 0.5).abs() < 0.03);
}
#[test]
fn dual_has_no_crossfeed_and_quad_has_four_heads() {
    let mut v = clean();
    v[STYLE] = 1.0;
    let mut e = engine(&v, 48000.0);
    for i in 0..24000 {
        let y = e.process_frame(if i == 0 { [1.0, 0.0] } else { [0.0; 2] }, 2);
        assert_eq!(y[1], 0.0);
    }
    v[STYLE] = 4.0;
    v[FEEDBACK] = 0.0;
    let mut e = engine(&v, 48000.0);
    let mut peaks = [0.0f32; 4];
    for i in 0..6000 {
        let y = e.process_frame(if i == 0 { [1.0, 0.0] } else { [0.0; 2] }, 2);
        for (t, p) in peaks.iter_mut().enumerate() {
            if i == 1200 * (t + 1) {
                *p = y[0];
            }
        }
    }
    assert!(peaks.iter().all(|p| *p > 0.35), "{peaks:?}");
}
#[test]
fn sync_uses_host_tempo_and_ducking_reduces_wet_energy() {
    let mut v = clean();
    v[SYNC_L] = 1.0;
    v[DIVISION_L] = 3.0;
    v[FEEDBACK] = 0.0;
    let mut e = engine(&v, 48000.0);
    e.set_tempo(120.0);
    let mut peak = (0, 0.0f32);
    for i in 0..13000 {
        let y = e.process_frame(if i == 0 { [1.0, 0.0] } else { [0.0; 2] }, 2);
        if y[0].abs() > peak.1 {
            peak = (i, y[0].abs());
        }
    }
    assert_eq!(peak.0, 12000);
    v = clean();
    v[MODE] = 8.0;
    v[DUCK] = 0.0;
    let mut a = engine(&v, 48000.0);
    v[DUCK] = 100.0;
    let mut b = engine(&v, 48000.0);
    let (mut pa, mut pb) = (0.0f64, 0.0f64);
    for i in 0..48000 {
        let x = (i as f32 * 0.05).sin() * 0.5;
        let ya = a.process_frame([x; 2], 2)[0];
        let yb = b.process_frame([x; 2], 2)[0];
        if i > 24000 {
            pa += (ya * ya) as f64;
            pb += (yb * yb) as f64;
        }
    }
    assert!(pb < pa * 0.05);
}
#[test]
fn all_modes_and_styles_are_finite_under_super_unity_feedback() {
    for mode in 0..17 {
        for style in 0..5 {
            let mut v = defaults();
            v[MODE] = mode as f32;
            v[STYLE] = style as f32;
            v[FEEDBACK] = 200.0;
            v[TIME_L] = 1.0;
            v[TIME_R] = 2.0;
            v[PITCH] = 12.0;
            v[SHIFT] = -300.0;
            v[DIFFUSION] = 90.0;
            let mut e = engine(&v, 48000.0);
            for i in 0..12000 {
                let y = e.process_frame(if i < 100 { [0.5, -0.3] } else { [0.0; 2] }, 2);
                assert!(
                    y.iter().all(|v| v.is_finite() && v.abs() < 20.0),
                    "mode {mode} style {style}: {y:?}"
                );
            }
        }
    }
}

#[test]
fn reverse_pitch_reverses_in_every_routing_style() {
    for style in 0..5 {
        let mut v = clean();
        v[FEEDBACK] = 0.0;
        v[STYLE] = style as f32;
        v[MODE] = 5.0;
        v[PITCH] = 0.0;
        let mut forward = engine(&v, 48000.0);
        v[MODE] = 6.0;
        let mut reverse = engine(&v, 48000.0);
        let (mut difference, mut power) = (0.0f64, 0.0f64);
        for i in 0..24000 {
            let t = i as f32 / 48000.0;
            let x = if i < 7000 {
                ((t * 311.0).sin() * (t * 1837.0).sin() + (t * 7091.0).cos() * 0.3) * 0.1
            } else {
                0.0
            };
            let a = forward.process_frame([x, x * 0.8], 2);
            let b = reverse.process_frame([x, x * 0.8], 2);
            for c in 0..2 {
                difference += ((a[c] - b[c]).powi(2)) as f64;
                power += (a[c] * a[c]) as f64;
            }
        }
        assert!(
            difference > power * 0.1,
            "style {style} read forwards: {difference}/{power}"
        );
    }
}
#[test]
fn quad_repeat_head_d_applies_frequency_shift_in_feedback() {
    let mut v = clean();
    v[MODE] = 4.0;
    v[STYLE] = 4.0;
    v[SHIFT] = 100.0;
    v[FEEDBACK] = 60.0;
    v[TAP_B] = 0.0;
    v[TAP_C] = 0.0;
    v[TAP_D] = 0.0;
    v[SWELL] = 0.0;
    let mut e = engine(&v, 48000.0);
    let mut bins = [(0.0f64, 0.0f64); 2];
    for i in 0..8000 {
        let x = if i < 1200 {
            (std::f32::consts::TAU * 1000.0 * i as f32 / 48000.0).cos() * 0.1
        } else {
            0.0
        };
        let y = e.process_frame([x, 0.0], 2)[0];
        if (6200..7160).contains(&i) {
            for (bin, hz) in bins.iter_mut().zip([1200.0, 1100.0]) {
                let a = std::f64::consts::TAU * hz * i as f64 / 48000.0;
                bin.0 += y as f64 * a.cos();
                bin.1 += y as f64 * a.sin();
            }
        }
    }
    let wanted = bins[0].0.hypot(bins[0].1);
    let unshifted = bins[1].0.hypot(bins[1].1);
    assert!(
        wanted > 1.0 && wanted > unshifted * 4.0,
        "repeat spectrum: {wanted} versus {unshifted}"
    );
}
