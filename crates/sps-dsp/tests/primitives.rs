use sps_dsp::{db_to_gain, filter::Crossover, gain_to_db, params::Spec};

#[test]
fn parameter_mapping_round_trips_and_sanitizes_host_values() {
    let frequency = Spec {
        id: "frequency",
        name: "Frequency",
        min: 20.0,
        max: 20000.0,
        default: 1000.0,
        unit: "Hz",
        logarithmic: true,
        discrete: false,
    };
    for value in [20.0, 100.0, 1000.0, 20000.0] {
        let restored = frequency.denormalize(frequency.normalize(value));
        assert!((restored - value).abs() <= value * 1e-6);
    }
    assert_eq!(frequency.sanitize(f32::NAN), 1000.0);
    assert_eq!(frequency.sanitize(f32::INFINITY), 1000.0);
    assert_eq!(frequency.sanitize(-1.0), 20.0);
    assert_eq!(frequency.denormalize(2.0), 20000.0);
    let toggle = Spec {
        min: 0.0,
        max: 1.0,
        default: 0.0,
        logarithmic: false,
        discrete: true,
        ..frequency
    };
    assert_eq!(toggle.denormalize(0.49), 0.0);
    assert_eq!(toggle.denormalize(0.51), 1.0);
}

#[test]
fn gain_conversions_preserve_amplitude_and_meter_floor() {
    for db in [-100.0, -36.0, -6.0, 0.0, 18.0, 36.0] {
        assert!((gain_to_db(db_to_gain(db)) - db).abs() < 1e-4);
    }
    assert_eq!(db_to_gain(0.0), 1.0);
    assert_eq!(gain_to_db(0.0), -120.0);
}

#[test]
fn crossover_has_flat_sum_without_plugin_parameters() {
    for rate in [44100.0, 48000.0, 96000.0] {
        for hz in [30.0, 120.0, 1000.0, 3000.0, 16000.0] {
            let mut crossover = Crossover::new(rate, 120.0, 3000.0);
            let mut input_energy = 0.0f64;
            let mut output_energy = 0.0f64;
            for i in 0..rate as usize * 2 {
                let x = (std::f32::consts::TAU * hz * i as f32 / rate).sin();
                let y: f32 = crossover.split(x, 0).iter().sum();
                if i >= rate as usize {
                    input_energy += f64::from(x).powi(2);
                    output_energy += f64::from(y).powi(2);
                }
            }
            let db = 10.0 * (output_energy / input_energy).log10();
            assert!(db.abs() < 0.025, "rate={rate}, hz={hz}, sum={db} dB");
        }
    }
}

#[test]
fn crossover_reset_clears_history_and_channels_are_independent() {
    let mut crossover = Crossover::new(48000.0, 100.0, 2500.0);
    for _ in 0..1000 {
        crossover.split(0.25, 0);
        assert_eq!(crossover.split(0.0, 1), [0.0; 3]);
    }
    crossover.configure(48000.0, 200.0, 4000.0);
    crossover.reset();
    for channel in 0..2 {
        assert_eq!(crossover.split(0.0, channel), [0.0; 3]);
    }
    let mut fresh = Crossover::new(48000.0, 200.0, 4000.0);
    for i in 0..1000 {
        let input = (i as f32 * 0.1).sin();
        assert_eq!(crossover.split(input, 0), fresh.split(input, 0));
    }
}
