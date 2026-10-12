//! Room metadata from its stable DSP schema.
use sps_room_dsp::params::{choices, SPECS};
use std::path::Path;
pub fn metadata(directory: &Path) -> std::io::Result<()> {
    crate::time_metadata::metadata(directory, "sps-room", "Room", "Reverb", &SPECS, choices)
}
