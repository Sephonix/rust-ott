//! Echo metadata from its stable DSP schema.
use sps_echo_dsp::params::{choices, SPECS};
use std::path::Path;
pub fn metadata(directory: &Path) -> std::io::Result<()> {
    crate::time_metadata::metadata(directory, "sps-echo", "Echo", "Delay", &SPECS, choices)
}
