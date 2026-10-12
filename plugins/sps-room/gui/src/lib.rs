//! Room editor: local schema and factory sound design, shared controls/presets.
use nih_plug_egui::egui::Context;
pub use sps_gui::{configure, UiState, WIDTH};
pub const HEIGHT: u32 = 640;
use sps_room_dsp::{params::*, METER_COUNT};
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
        "Room",
        "com.sephonix.sps-room",
        &SPECS,
        values,
        meters,
        choices,
        FACTORY_NAMES,
        factory,
        change,
    );
}
