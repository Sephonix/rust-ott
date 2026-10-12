use sps_echo_dsp::{params::*, Engine};
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

#[test]
fn zero_shift_echo_timing_and_independent_channels() {
    let mut v = defaults();
    v[MIX] = 100.0;
    v[TIME] = 100.0;
    v[SHIFT] = 0.0;
    v[LOW_CUT] = 0.0;
    v[HIGH_CUT] = 20000.0;
    let mut e = engine(&v, 48000.0);
    let mut peak = (0, 0.0f32);
    for i in 0..18000 {
        let y = e.process_frame(if i == 0 { [1.0, 0.0] } else { [0.0; 2] }, 2);
        assert_eq!(y[1], 0.0);
        if y[0].abs() > peak.1 {
            peak = (i, y[0].abs());
        }
    }
    assert_eq!(peak.0, 4800);
    assert!(peak.1 > 0.7);
}
#[test]
fn opposing_stereo_shift_has_distinct_frequency_peaks() {
    let mut v = defaults();
    v[MIX] = 100.0;
    v[SHIFT] = 100.0;
    v[FEEDBACK] = 0.0;
    v[STEREO] = 1.0;
    let mut e = engine(&v, 48000.0);
    let (mut wanted, mut image) = ([0.0f64; 2], [0.0f64; 2]);
    for i in 0..96000 {
        let x = (std::f32::consts::TAU * 1000.0 * i as f32 / 48000.0).cos();
        let y = e.process_frame([x; 2], 2);
        if i >= 48000 {
            for c in 0..2 {
                let f = if c == 0 { 1100.0 } else { 900.0 };
                wanted[c] += y[c] as f64 * (std::f64::consts::TAU * f * i as f64 / 48000.0).cos();
                image[c] +=
                    y[c] as f64 * (std::f64::consts::TAU * (2000.0 - f) * i as f64 / 48000.0).cos();
            }
        }
    }
    for c in 0..2 {
        assert!(wanted[c].abs() > 1000.0);
        assert!(image[c].abs() < wanted[c].abs() * 0.01);
    }
}
