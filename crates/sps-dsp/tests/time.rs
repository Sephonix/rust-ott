use sps_dsp::time::*;
use std::f32::consts::TAU;
fn amplitude(data: &[f32], hz: f32, rate: f32) -> f32 {
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for (i, x) in data.iter().enumerate() {
        let phase = std::f64::consts::TAU * hz as f64 * i as f64 / rate as f64;
        re += *x as f64 * phase.cos();
        im += *x as f64 * phase.sin();
    }
    (2.0 * re.hypot(im) / data.len() as f64) as f32
}
#[test]
fn fractional_delay_and_clear() {
    let mut d = DelayLine::new(10);
    d.push(1.0);
    assert_eq!(d.read(1.0), 1.0);
    d.push(0.0);
    assert_eq!(d.read(1.5), 0.5);
    d.reset();
    assert_eq!(d.read(2.0), 0.0);
}
#[test]
fn hilbert_shifts_signed_hz_and_rejects_image() {
    for rate in [44100.0, 48000.0, 96000.0] {
        for shift in [-100.0, 100.0] {
            let mut f = FrequencyShifter::default();
            f.configure(shift, rate);
            let n = rate as usize;
            let mut data = Vec::with_capacity(n);
            for i in 0..n * 2 {
                let y = f.process((TAU * 1000.0 * i as f32 / rate).cos());
                if i >= n {
                    data.push(y);
                }
            }
            let wanted = amplitude(&data, 1000.0 + shift, rate);
            let image = amplitude(&data, 1000.0 - shift, rate);
            assert!(wanted > 0.95, "{rate}: wanted {wanted}");
            assert!(
                image / wanted < 0.0032,
                "{rate}: image {image}, wanted {wanted}"
            );
            f.reset();
            assert_eq!(f.process(0.0), 0.0);
        }
    }
}
#[test]
fn pitch_is_multiplicative_and_unity_is_exact() {
    let rate = 48000.0;
    let mut p = PitchShifter::new(rate);
    let mut data = Vec::new();
    for i in 0..96000 {
        let x = (TAU * 1000.0 * i as f32 / rate).sin();
        let y = p.process(x, 1.5);
        if i >= 48000 {
            data.push(y);
        }
    }
    assert!(amplitude(&data, 1500.0, rate) > 0.9);
    assert!(amplitude(&data, 1000.0, rate) < 0.02);
    p.reset();
    assert_eq!(p.process(0.125, 1.0), 0.125);
}
#[test]
fn tempo_subdivisions_are_exact() {
    assert_eq!(synced_ms(17.0, 0, 4, 120.0), 17.0);
    assert_eq!(synced_ms(0.0, 1, 4, 120.0), 500.0);
    assert_eq!(synced_ms(0.0, 2, 4, 120.0), 750.0);
    assert!((synced_ms(0.0, 3, 4, 120.0) - 1000.0 / 3.0).abs() < 0.001);
    assert_eq!(synced_ms(0.0, 1, 3, 60.0), 500.0);
}
#[test]
fn low_latency_quadrature_has_signed_shift_and_rejects_image() {
    for rate in [44100.0, 48000.0, 96000.0] {
        for frequency in [100.0, 1000.0, 10000.0] {
            for shift in [-17.0, 17.0] {
                let mut f = QuadratureShifter::default();
                f.configure(shift, rate);
                f.reset();
                let n = rate as usize;
                let mut data = Vec::with_capacity(n);
                for i in 0..n * 2 {
                    let y = f.process((TAU * frequency * i as f32 / rate).cos());
                    if i >= n {
                        data.push(y);
                    }
                }
                let wanted = amplitude(&data, frequency + shift, rate);
                let image = amplitude(&data, frequency - shift, rate);
                assert!(wanted > 0.98, "{rate} {frequency} {shift}: {wanted}");
                assert!(
                    image / wanted < 0.001,
                    "{rate} {frequency}: {image}/{wanted}"
                );
            }
        }
    }
    let mut f = QuadratureShifter::default();
    f.configure(0.0, 48000.0);
    f.reset();
    assert_eq!(f.process(0.25), 0.25);
}
