use sps_dsp::{params::*, Engine};
use std::{hint::black_box, time::Instant};
fn main() {
    let frames = 48000 * 20;
    let input: Vec<_> = (0..4096)
        .map(|i| {
            let x = (i as f32 * 0.037).sin() * 0.13 + (i as f32 * 0.713).sin() * 0.021;
            [x, -x * 0.7]
        })
        .collect();
    for rate in [48000.0, 96000.0] {
        for automate in [false, true] {
            let mut e = Engine::new(rate);
            let mut v = defaults();
            let start = Instant::now();
            for i in 0..frames {
                if automate && i % 128 == 0 {
                    v[AMOUNT] = (i / 128 % 100) as f32;
                    v[LOW_CROSS] = 40.0 + (i / 128 % 400) as f32;
                    e.set_parameters(&v);
                }
                black_box(e.process_frame(black_box(input[i % input.len()]), 2));
            }
            let elapsed = start.elapsed().as_secs_f64();
            let audio_seconds = frames as f64 / rate as f64;
            println!("{rate:.0} Hz stereo, automation={automate}: {:.0} ns/frame, {:.2}% of one core ({audio_seconds:.1}s audio in {elapsed:.3}s)",elapsed*1e9/frames as f64,elapsed/audio_seconds*100.0);
        }
    }
}
