# Validation

Validated locally on Linux x86-64 (Arch Linux, KDE Wayland with XWayland),
Rust 1.99.0, 2026-10-10. These checks use the built plugin binaries; no audio
hardware or live audio server is required.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | 13 DSP tests + 1 LV2 lifecycle test passed |
| `cargo xtask bundle --release --locked` | CLAP, VST3 and LV2 Linux bundles produced |
| CLAP validator release 0.4.1 (`152b982` build) | 34 passed, 0 failed, 0 warnings; 10 not-applicable tests skipped |
| Steinberg VST3 SDK 3.8.1 validator | 47 passed, 0 failed |
| `lv2_validate target/bundled/sps-ott.lv2/*.ttl` | 0 errors |
| `python3 scripts/smoke_lv2.py` | Lilv discovery, port metadata, exact bypass, processing, anti-phase stereo, meters and zero-frame run passed |
| `python3 scripts/smoke_lv2.py --ui --exercise-ui --seconds 8` | Native embedded editor, idle loop, meter events, Amount drag, factory reset and cleanup passed; host received both edits |

The CLAP skipped tests concern note/MIDI, voice information and other extensions
this audio effect does not advertise. VST3 processes f32 audio; the validator
reports f64 processing as unsupported. No claim of compatibility with every DAW
or with Windows/macOS is made yet. The shared GUI was visually inspected in its
native preview and as an embedded LV2 X11 child. A backend repaint scheduling
issue was fixed so history and meters update without mouse activity.

The DSP tests check exact factory defaults, hard/soft transfer curves,
infinite ratios, flat crossover recombination (within 0.025 dB across tested
frequencies at 44.1/48/96 kHz), preset steady-state gain, stereo linking,
anti-phase and one-sided audio, mono/stereo agreement, dry/wet and output gain,
true bypass, reset, silence, invalid audio/control values, extreme automation,
mute/solo, detection modes, and zero process-time heap allocations.

## CPU measurement

Run `cargo bench -p sps-dsp --bench throughput`. On an Intel Core Ultra 9 185H,
release builds measured approximately **135–141 ns per stereo frame**:

| Sample rate | Static preset | Continuously automated Amount and crossover |
| --- | ---: | ---: |
| 48 kHz | 0.68% of one core | 0.66% of one core |
| 96 kHz | 1.30% of one core | 1.34% of one core |

Each measurement processes 960,000 stereo frames. The benchmark includes
filtering, detectors, compression, smoothing and metering, with a prepared
multi-tone signal and `black_box` to prevent dead-code elimination. It excludes
DAW wrapper and GUI rendering overhead. Values depend on CPU, power settings,
compiler, buffer size and workload; they are measurements, not a universal CPU
guarantee. GUI drawing runs only while an editor is open.

## Reproducing host checks

Download [CLAP validator](https://github.com/free-audio/clap-validator/releases/tag/0.4.1)
and run:

```sh
clap-validator validate 'target/bundled/SPS OTT.clap'
```

For the VST3 validator, build the
[official SDK](https://github.com/steinbergmedia/vst3sdk):

```sh
git clone --recursive https://github.com/steinbergmedia/vst3sdk.git /tmp/vst3sdk
cmake -S /tmp/vst3sdk -B /tmp/vst3-build -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DSMTG_ENABLE_VST3_PLUGIN_EXAMPLES=OFF \
  -DSMTG_ENABLE_VSTGUI_SUPPORT=OFF
cmake --build /tmp/vst3-build --target validator
/tmp/vst3-build/bin/Release/validator 'target/bundled/SPS OTT.vst3'
```

LV2 RDF validation uses the upstream `lv2_validate` script and `sord_validate`
(Debian/Ubuntu packages `lv2-dev` and `sordi`). Headless UI checks can run under
Xvfb with Mesa software OpenGL:

```sh
xvfb-run -a python3 scripts/smoke_lv2.py --ui --exercise-ui --seconds 3
```

The GitHub workflow runs Rust checks, all format builds, CLAP validation, LV2
metadata/audio/UI validation and archive packaging. VST3 SDK validation was
run locally; it is not currently rebuilt in every CI job.
