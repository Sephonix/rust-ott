use sps_room_dsp::{params::*, Engine};
use std::{hint::black_box, time::Instant};
fn main() {
    for rate in [48000.0, 96000.0] {
        for mode in 0..12 {
            for automated in [false, true] {
                let mut v = defaults();
                v[MODE] = mode as f32;
                let mut e = Engine::new(rate);
                e.set_parameters(&v);
                e.reset();
                let input: Vec<_> = (0..4096)
                    .map(|i| {
                        (
                            (i as f32 * 0.037).sin() * 0.1,
                            (i as f32 * 0.057).sin() * 0.1,
                        )
                    })
                    .collect();
                let start = Instant::now();
                let count = 480000;
                for i in 0..count {
                    if automated && i % 32 == 0 {
                        let t = (i % 48000) as f32 / 48000.0;
                        v[LATE_SIZE] = SPECS[LATE_SIZE].denormalize(t);
                        v[MIX] = 40.0 + t * 20.0;
                        e.set_parameters(&v);
                    }
                    let x = input[i % input.len()];
                    black_box(e.process_frame(black_box([x.0, x.1]), 2));
                }
                let ns = start.elapsed().as_nanos() as f64 / count as f64;
                println!("room {rate:.0} Hz mode {mode} automated {automated}: {ns:.1} ns/stereo frame, {:.2}% core",ns*rate as f64/1e7);
            }
        }
    }
}
