---
name: sps-dsp-change
description: Implement or refactor audio processing in the Sephonix Plugin Suite while keeping reusable primitives independent and preserving real-time, channel, reset, automation, and parameter compatibility behavior.
---

# Change suite DSP

Read `AGENTS.md` and the relevant portions of `docs/architecture.md`, then locate
the affected plugin's DSP/schema and tests. For OTT these are
`plugins/sps-ott/dsp/src/` and `plugins/sps-ott/dsp/tests/audio.rs`.

Choose ownership by behavior: a generic primitive belongs in `crates/sps-dsp`;
an effect's tuning, topology, dynamics policy, parameter table, or meter layout
belongs in its plugin DSP package. When extracting code, preserve arithmetic,
state precision, smoothing and reset order unless changing them is requested.
Do not generalize an editor or host adapter merely because formats share it.

Keep process-time parameter updates and samples free of heap allocation, locks,
I/O and GUI calls. Initialize buffers before processing. Preserve independent
channel history, intentional stereo linking, finite output for invalid controls,
Nyquist bounds, denormal handling, and reset/lifecycle behavior. For phase-rotating
filters, choose the dry reference deliberately so mixing does not comb-filter.
`AtomicValue` uses relaxed operations and conveys only its individual float.

OTT-specific contracts include phase-matched LR4 band summation, max-channel
RMS/peak linking, smoothed automation, exact settled bypass, upward gain cap and
noise fade. Its IDs/port order are published; change the schema deliberately
and regenerate metadata through `cargo xtask metadata sps-ott` when relevant.

Use audio tests that distinguish the intended behavior: transfer curves,
frequency response, impulse/reset history, channel isolation/linking, bypass,
automation extremes or measured process allocations, as applicable. Preserve
existing regression coverage when moving code. Run the Rust checks and rebuilt
plugin runtime checks appropriate to the change. Report measured tolerances and
any deliberate behavior change; benchmark when cost changes materially.
