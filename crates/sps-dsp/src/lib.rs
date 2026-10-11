//! Dependency-free DSP building blocks for the Sephonix Plugin Suite.
//! Plugin engines, parameter tables and presets belong under `plugins/`.
mod atomic;
pub mod filter;
mod gain;
pub mod params;

pub use atomic::AtomicValue;
pub use gain::{db_to_gain, gain_to_db};
