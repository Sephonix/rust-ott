use sps_room_dsp::{params::*, Engine};
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
fn all_characters_produce_a_decaying_stereo_tail() {
    for mode in 0..12 {
        let mut v = defaults();
        v[MODE] = mode as f32;
        v[MIX] = 100.0;
        v[DECAY] = 0.6;
        v[PREDELAY] = 0.0;
        v[DEPTH] = 100.0;
        v[LATE_MOD] = 0.0;
        let mut e = engine(&v, 48000.0);
        let mut early = 0.0f64;
        let mut late = 0.0f64;
        let mut right = 0.0f64;
        for i in 0..144000 {
            let y = e.process_frame(if i == 0 { [1.0, 0.0] } else { [0.0; 2] }, 2);
            let power = (y[0] * y[0] + y[1] * y[1]) as f64;
            if (2000..24000).contains(&i) {
                early += power;
            }
            if i >= 96000 {
                late += power;
            }
            right += (y[1] * y[1]) as f64;
        }
        assert!(early > 1e-5, "mode {mode} has no tail");
        assert!(right > 1e-6, "mode {mode} has no stereo coupling");
        assert!(
            late < early * 0.001,
            "mode {mode} does not decay: {early} -> {late}"
        );
    }
}
#[test]
fn zero_crossfeed_preserves_channel_history() {
    let mut v = defaults();
    v[MIX] = 100.0;
    v[EARLY_CROSS] = 0.0;
    v[LATE_CROSS] = 0.0;
    let mut e = engine(&v, 48000.0);
    for i in 0..48000 {
        let y = e.process_frame(if i == 0 { [1.0, 0.0] } else { [0.0; 2] }, 2);
        assert_eq!(y[1], 0.0);
    }
}
#[test]
fn predelay_and_early_size_control_onset_and_duration() {
    let mut v = defaults();
    v[MIX] = 100.0;
    v[DEPTH] = 0.0;
    v[PREDELAY] = 50.0;
    v[EARLY_SIZE] = 100.0;
    v[EARLY_MOD] = 0.0;
    let mut e = engine(&v, 48000.0);
    let mut first = None;
    let mut last = 0;
    for i in 0..12000 {
        let y = e.process_frame(if i == 0 { [1.0, 0.0] } else { [0.0; 2] }, 2);
        if y[0].abs() > 1e-5 {
            first.get_or_insert(i);
            last = i;
        }
    }
    assert!((2500..2700).contains(&first.unwrap()));
    assert!((7000..7500).contains(&last));
}

#[test]
fn bass_and_high_multipliers_shape_tail_energy() {
    use sps_dsp::time::OnePole;
    fn tail(bass: f32, high: f32) -> [f64; 2] {
        let mut v = defaults();
        v[MIX] = 100.0;
        v[DEPTH] = 100.0;
        v[EARLY_SEND] = 0.0;
        v[DECAY] = 0.7;
        v[BASS_MULT] = bass;
        v[HIGH_MULT] = high;
        v[LATE_MOD] = 0.0;
        v[HIGH_CUT] = 20000.0;
        let mut e = engine(&v, 48000.0);
        let mut low = OnePole::default();
        let mut treble = OnePole::default();
        low.configure(120.0, 48000.0);
        treble.configure(8000.0, 48000.0);
        let mut energy = [0.0f64; 2];
        for i in 0..96000 {
            let y = e.process_frame(if i == 0 { [1.0; 2] } else { [0.0; 2] }, 2)[0];
            let a = low.low(y);
            let b = treble.high(y);
            if i > 38400 {
                energy[0] += (a * a) as f64;
                energy[1] += (b * b) as f64;
            }
        }
        energy
    }
    let short = tail(0.5, 0.1);
    let warm = tail(2.0, 0.1);
    let bright = tail(0.5, 1.0);
    assert!(
        warm[0] > short[0] * 10.0,
        "bass mult: {short:?} -> {warm:?}"
    );
    assert!(
        bright[1] > short[1] * 10.0,
        "high mult: {short:?} -> {bright:?}"
    );
}
