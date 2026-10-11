# Sephonix Plugin Suite DSP workspace

This repository is `Sephonix/sps-dsp`, the shared Rust workspace for the suite.
Read [docs/architecture.md](docs/architecture.md) for reuse boundaries and
[docs/validation.md](docs/validation.md) for runtime validation. The current
plugin manual is [plugins/sps-ott/README.md](plugins/sps-ott/README.md).

## Ownership and naming

- `crates/sps-dsp`: dependency-free, plugin-independent DSP primitives. Do not
  place a plugin's presets, parameter IDs, meter count, or editor here.
- `plugins/sps-<name>/`: plugin-specific engine, schema, editor, format adapters,
  checked-in LV2 metadata, manual, and runtime smoke tests. The exported package
  is `sps-<name>`; helper packages use `sps-<name>-dsp`, `-gui`, and `-lv2`.
- `xtask`: suite build/install registry and separate plugin metadata generators.
- `scripts/package.py`: packages a selected exported plugin using its manifest's
  `[package.metadata.sps].bundles` and includes docs and dependency notices.
- `vendor/nih-plug`: pinned core with documented local fixes in `PATCHES.md`.
  Preserve patches when updating upstream; record intentional vendor changes.

Reuse the existing primitives where they fit. Keep behavior and tuning local
until another plugin has a concrete shared need; extract common code without
creating a dependency from shared crates to plugin crates.

## Audio and compatibility contracts

Allocate state during construction/initialization. Audio processing and
parameter updates must not allocate, lock, log, perform I/O, or call GUI code.
Sanitize controls, bound crossover frequencies below Nyquist, retain independent
channel history and stereo linking where intended, and reset signal history on
activation. Meter/UI mailboxes use independent relaxed atomics, not synchronized
snapshots or readiness signals. Host/window raw pointers belong in format ABI
boundaries with explicit lifetime/safety reasoning.

Preserve published CLAP IDs, VST3 class IDs, LV2 URIs, parameter IDs, and LV2 port
indices when changing an existing plugin. Repository/package/path renames do
not justify changing its host identity. New plugins need distinct identities.
For OTT, `plugins/sps-ott/dsp/src/params.rs` owns controls/defaults; generate
metadata with `cargo xtask metadata sps-ott` rather than editing Turtle by hand.

## Validation

Run from the workspace root. For Rust changes, use `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`, and appropriate
`cargo test --workspace --locked` coverage. DSP behavior changes need meaningful
audio tests (including allocation checks where the processing path changes).

For format, engine, or workspace/bundling changes, build with
`cargo xtask bundle <plugin> --release --locked` and run that plugin's runtime
smoke tests/available validators. Compilation alone does not show that a host
can discover and process audio through the exports. Run UI checks when editor
or embedding changes warrant them. See the validation skill for OTT commands.
Keep generated metadata reproducible, pinned dependency revisions and lockfile
consistent, and packaging notices intact. Report actual results and unavailable
checks without treating historical results as a new run.

## Repository skills

Project skills are discoverable under `.agents/skills/`:

- [sps-add-plugin](.agents/skills/sps-add-plugin/SKILL.md): adding another suite
  plugin, registering builds and packaging, and assigning host identities.
- [sps-dsp-change](.agents/skills/sps-dsp-change/SKILL.md): implementing DSP
  changes or extracting reusable processing while preserving audio contracts.
- [sps-plugin-validation](.agents/skills/sps-plugin-validation/SKILL.md): building
  bundles and verifying metadata, runtime audio, native UI, and archives.

Use the relevant skill when its workflow applies. Instructions support the
user's requested scope; installation, publishing, commits, and pushes follow
that scope rather than being implied by a validation or development skill.
