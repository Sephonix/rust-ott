---
name: sps-plugin-validation
description: Build and validate Sephonix Plugin Suite plugin bundles, including runtime audio, generated metadata, native UI embedding, host validators, and distributable archives. Use for release checks or changes affecting DSP exports, workspace layout, or bundling.
---

# Validate suite plugins

Read `AGENTS.md` and `docs/validation.md`. Use the selected plugin's manual and
scripts; the concrete commands below are for SPS OTT. Run from the workspace
root. Bundling writes `target/bundled` (or `$CARGO_TARGET_DIR/bundled`); it does
not install into the user's DAW scan paths.

Run relevant Rust checks and build the actual exports:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask bundle sps-ott --release --locked
```

Regenerate `cargo xtask metadata sps-ott`; compare
`plugins/sps-ott/metadata` with the expected checked-in files. If changes are
intentional, ensure a second generation is unchanged. Validate Turtle with
`lv2_validate target/bundled/sps-ott.lv2/*.ttl` when available.

Exercise the built library through its host ABI:

```sh
python3 plugins/sps-ott/scripts/smoke_lv2.py
python3 plugins/sps-ott/scripts/smoke_lv2.py --ui --exercise-ui --seconds 3
```

The audio smoke test needs liblilv-0; it checks discovery, 53 ports, exact bypass,
processing, anti-phase stereo, meters and zero-size blocks. The UI test needs
X11/XWayland and OpenGL and checks embedding, idle callbacks, meter updates,
Amount edits, factory reset and cleanup. Use an existing display, or `xvfb-run
-a` when Xvfb/software OpenGL is available. With a custom target directory, pass
the LV2 bundle path explicitly. A headless audio pass cannot establish UI health.

Run available CLAP and VST3 validators against the newly built bundles; commands
and pinned CI download/checksum details are in `docs/validation.md` and
`.github/workflows/linux.yml`. Report unavailable validators honestly.
For another plugin, use its identities, paths, metadata and dedicated smoke test.

For packaging changes or distribution verification, run
`python3 scripts/package.py sps-ott` and inspect the archive for all declared
bundles, reachable docs, original license, dependency notices, and `BUILD.json`
source/revision metadata. For performance changes, use
`cargo bench -p sps-ott-dsp --bench throughput` and record measurement conditions.

Summarize fresh results separately from historical validation. Install into
`~/.clap`, `~/.vst3`, or `~/.lv2` only when installation is in the user's scope.
