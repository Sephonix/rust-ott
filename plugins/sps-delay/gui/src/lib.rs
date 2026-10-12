//! Delay editor: local schema and factory sound design, shared controls/presets.
use nih_plug_egui::egui::Context;
use sps_delay_dsp::{params::*, METER_COUNT};
pub use sps_gui::{configure, UiState, HEIGHT, WIDTH};
pub fn draw(
    ctx: &Context,
    state: &mut UiState,
    values: &[f32; PARAM_COUNT],
    meters: &[f32; METER_COUNT],
    change: impl FnMut(usize, f32),
) {
    sps_gui::draw(
        ctx,
        state,
        "Delay",
        "com.sephonix.sps-delay",
        &SPECS,
        values,
        meters,
        choices,
        FACTORY_NAMES,
        factory,
        change,
    );
}
