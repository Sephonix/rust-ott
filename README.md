# SPS DSP

**SPS DSP** (`sps-dsp`) is the Rust workspace for DSP plugins in the
**Sephonix Plugin Suite**. Shared building blocks live in `crates/`; each
plugin owns its engine, parameter schema, editor, and format adapters under
`plugins/sps-<name>/`.

| Plugin | Description | Linux formats |
| --- | --- | --- |
| [SPS OTT](plugins/sps-ott/README.md) | Three-band upward/downward compressor | CLAP, VST3, LV2 |
| [SPS Room](plugins/sps-room/README.md) | True-stereo early energy and modulated reverb | CLAP, VST3, LV2 |
| [SPS Delay](plugins/sps-delay/README.md) | Seventeen characters, five routing styles, diffusion and shifting | CLAP, VST3, LV2 |
| [SPS Echo](plugins/sps-echo/README.md) | Low-latency frequency-shifting feedback echo | CLAP, VST3, LV2 |

SPS OTT retains its plugin identifiers, presets, and bundle names. Its complete
[manual](plugins/sps-ott/README.md) covers controls, factory settings, installation,
and Linux build dependencies.

## Build and validate

Install stable Rust and the native X11/OpenGL dependencies listed in the plugin
manual, then run these commands from the workspace root:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask bundle --release --locked
python3 plugins/sps-ott/scripts/smoke_lv2.py
python3 plugins/sps-ott/scripts/smoke_lv2.py --ui --exercise-ui --seconds 3
python3 scripts/package.py sps-ott
```

Bundles are written to `target/bundled/`; distribution archives go to `dist/`.
`cargo xtask bundle`, `install`, and `metadata` select all registered plugins
by default. Pass a package name, for example
`cargo xtask bundle sps-ott --release --locked`, to select one plugin. `install` replaces that plugin's
bundles in the current user's `~/.clap`, `~/.vst3`, and `~/.lv2` directories.
The Linux CI workflow builds and validates all plugins and uploads their archives.
Room, Delay and Echo share a user preset browser, with JSON presets in
`Documents/SPS/Room`, `Documents/SPS/Delay` and `Documents/SPS/Echo`.
Their DSP and factory tuning are original implementations of publicly documented
Valhalla control families; they do not promise exact proprietary sound matching.
See [time effects validation](docs/time-effects-validation.md).

Linux is currently supported. Windows and macOS require platform builds and
host validation before distribution. Cross-compilation through
`cargo xtask bundle sps-ott --release --target=<triple>` builds CLAP/VST3 only.

## Developing another plugin

[`crates/sps-dsp`](crates/sps-dsp) provides dependency-free gain conversions,
a phase-matched stereo three-band LR4 crossover, parameter mapping, and atomic
float values. OTT's dynamics curve, tuning, fixed parameter table, meter layout,
and editor remain in [`plugins/sps-ott`](plugins/sps-ott).

[Architecture](docs/architecture.md) describes the boundaries, reusable pieces,
and plugin registration points. [Validation](docs/validation.md) records test
coverage and host checks. [AGENTS.md](AGENTS.md) supplies repository guidance;
project skills live in [.agents/skills](.agents/skills) for adding plugins,
changing DSP, and validating plugin bundles.

Dependency revisions and `Cargo.lock` are committed. The vendored NIH-plug core
has local CLAP state fixes documented in
[vendor/nih-plug/PATCHES.md](vendor/nih-plug/PATCHES.md).

## License

Original suite code is **GPL-3.0-or-later**; see [LICENSE](LICENSE).
Dependency licenses and notices are described in [THIRD_PARTY.md](THIRD_PARTY.md).
