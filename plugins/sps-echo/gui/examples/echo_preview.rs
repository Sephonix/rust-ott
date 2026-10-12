//! Native GUI preview with generated meter activity. No audio device required.
use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use egui_baseview::EguiWindow;
use sps_echo_dsp::{params::*, Engine};
struct Preview {
    ui: sps_echo_gui::UiState,
    values: [f32; PARAM_COUNT],
    engine: Engine,
    frame: usize,
}
fn main() {
    EguiWindow::open_blocking(
        WindowOpenOptions {
            title: "SPS Echo — GUI Preview".into(),
            size: Size::new(sps_echo_gui::WIDTH as f64, sps_echo_gui::HEIGHT as f64),
            scale: WindowScalePolicy::ScaleFactor(1.0),
            gl_config: Some(Default::default()),
        },
        Default::default(),
        Preview {
            ui: Default::default(),
            values: defaults(),
            engine: Engine::new(48000.0),
            frame: 0,
        },
        |ctx, _, _| sps_echo_gui::configure(ctx),
        |ctx, _, state| {
            state.engine.set_parameters(&state.values);
            for _ in 0..1600 {
                let t = state.frame as f32 / 48000.0;
                let envelope = (t * 1.6).sin() * 0.4 + 0.6;
                let x = ((t * 55.0 * std::f32::consts::TAU).sin() * 0.04
                    + (t * 740.0 * std::f32::consts::TAU).sin() * 0.05
                    + (t * 6700.0 * std::f32::consts::TAU).sin() * 0.014)
                    * envelope;
                state.engine.process_frame([x, x * 0.92], 2);
                state.frame += 1;
            }
            let values = state.values;
            sps_echo_gui::draw(
                ctx,
                &mut state.ui,
                &values,
                state.engine.meters(),
                |i, v| state.values[i] = v,
            );
        },
    );
}
