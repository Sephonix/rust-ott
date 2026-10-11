# Architecture

The Cargo workspace separates audio processing from plugin APIs and graphics.

| Crate | Responsibility |
| --- | --- |
| `sps-dsp` (`crates/sps-dsp`) | Dependency-free crossovers, gain conversions, parameter mapping and atomic floats. |
| `sps-ott-dsp` (`plugins/sps-ott/dsp`) | OTT engine, parameter table, factory tuning, dynamics curve and meter layout. |
| `sps-ott-gui` (`plugins/sps-ott/gui`) | OTT native egui editor and generated-activity preview, shared by OTT formats. |
| `sps-ott` (`plugins/sps-ott/plugin`) | NIH-plug CLAP/VST3 export, automation and project state. |
| `sps-ott-lv2` (`plugins/sps-ott/lv2`) | LV2 core and Linux X11 UI ABI, using the same DSP/editor. |
| `xtask` | Build, bundle, install and LV2 metadata/preset generation. |

## Reuse and adding plugins

All OTT implementation source lives under `plugins/sps-ott`; names of
implementation crates use the `sps-ott-` prefix. The exported CLAP/VST3 package
is simply `sps-ott`. Shared `sps-dsp` never depends on a plugin, GUI, or host API.
Each plugin's format adapters use its own format-independent DSP engine and
editor; the GUI does not run on the audio thread.

The shared crate exposes:

- `filter::Crossover`: phase-matched stereo three-band LR4 splitting, with
  independent channel history, f64 recursive state, reset and reconfiguration.
  Callers enforce finite `0 < low < high < sample_rate / 2` and channel 0 or 1.
  Recombination rotates phase; dry/wet effects should use the summed bands as
  their dry reference when they need phase alignment.
- `db_to_gain` / `gain_to_db`: amplitude conversions with a −120 dB meter floor.
- `params::Spec`: finite-value sanitization and linear/logarithmic mapping.
  Plugins provide IDs, ranges and defaults; logarithmic ranges must be positive.
- `AtomicValue`: relaxed atomic f32 mailboxes. Multiple values do not form a
  synchronized snapshot and cannot publish ownership or signal readiness.

These support other multiband effects, gain stages, parameter-driven processors,
and audio/UI telemetry. The OTT-specific upward noise fade, +36 dB cap,
38-control schema, 11-meter layout, and three-band editor are kept local.
Host ABI/lifecycle code is a reference for a new adapter; keep raw pointers and
unsafe host contracts inside format crates. Extract shared GUI widgets or host
helpers when another plugin has a concrete matching need.

For a new `sps-<name>` plugin, create DSP/editor/format packages under
`plugins/sps-<name>`, add them to the workspace and workspace dependencies,
and give the exported package a `bundler.toml` entry. Allocate new CLAP/VST3/LV2
identities. Register the plugin in `xtask/src/main.rs::PLUGINS`, with its LV2
metadata generator in a separate xtask module and checked-in metadata under
the plugin directory. The current registry handles CLAP, VST3, and Linux LV2;
adjust it explicitly for plugins with different format requirements.
Set `[package.metadata.sps].bundles` on the exported package for
`python3 scripts/package.py sps-<name>` and add the plugin manual, behavior
tests, runtime smoke test, and CI validation/artifact entry.

## SPS OTT signal path

```text
input ── LR4 Low/Mid split ─┬─ low ─ high-split allpass ─ low dynamics ─┐
                           └─ upper ─ LR4 Mid/High split ─┬─ mid dynamics ─┼─ sum
                                                         └─ high dynamics ┘

sum ─ phase-aligned dry/wet mix ─ final Output ─ original-input bypass blend ─ output
```

Each LR4 low/high branch cascades two second-order Butterworth filters. The low
band receives the second-order allpass equivalent to the high crossover's LP+HP
sum. That matches all three phases and gives flat summed magnitude. Filter
coefficients/state use f64 for low-frequency numerical stability; the public
audio samples and dynamics use f32. Recursive state below 1e−30 is flushed.

Each band applies its input trim, runs stereo-linked envelope detectors, computes
a downward and upward gain curve, and applies dynamics, output trim and mute/solo.
The max of the two channel powers/magnitudes drives a common gain. Anti-phase
material therefore does not cancel in the detector. RMS smooths squared samples;
Peak smooths absolute samples. Attack/release describe exponential time constants
of the corresponding detector state, not a fixed-window RMS or lookahead detector.

For level L, downward threshold D, upward threshold U and ratios Rd/Ru, the
hard-knee gains in dB are `−max(L−D, 0) × (1−1/Rd)` and
`max(U−L, 0) × (1−1/Ru)`. A 6 dB quadratic knee replaces each positive-part
function. Upward gain is limited to 36 dB with a smoothstep fade between −90 and
−72 dBFS. The gains add, then Amount scales them. Amount also scales both band
trims in dB, making Amount zero unity magnitude. The high band's original
−40.8 dB upward threshold is preserved even where the two soft-knee edges overlap.

## Realtime behavior

The engine is allocated during initialization and owns fixed arrays. The process
path does not allocate, take locks, call the host, log, or draw. Controls are
read once per host block; NIH-plug splits blocks at sample-accurate automation
events. Continuous values use a 20 ms exponential smoother updated every 32
samples; coefficients are recomputed only while values change. The detector
runs at the audio sample rate and dynamics gain uses a 1 ms smoother. Bypass,
detector mode, and mute/solo use 5 ms transitions. Bypass takes the exact original
input once the fade settles. This is deliberately a smoothed response to
sample-accurate automation, rather than instantaneous stepped audio changes.

Meter values are calculated in the DSP and published once per host block.
CLAP/VST3 publish to relaxed atomic floats only while the editor is open. LV2
writes optional control output ports. Meter decays are sample-rate independent.
The GUI keeps its own six-second history; there is no FFT or waveform buffer on
the audio thread. OpenGL draws the lightweight native editor. DSP unit tests
use an allocation-counting allocator to check automation processing.

## Parameter and state compatibility

`plugins/sps-ott/dsp/src/params.rs` is the source of truth for the 38 controls and
supplied defaults. NIH-plug uses stable string IDs; LV2 uses the same IDs as
symbols, and fixed port indices. Do not reorder/change published IDs or LV2
indices. Audio ports 0–3 are stereo input/output; controls 4–41 follow; optional
meter ports 42–52 finish the layout. CLAP/VST3 support mono and stereo layouts;
LV2 currently exposes stereo. All wrappers reset signal history on activation.

NIH-plug persists control values for CLAP/VST3. LV2 requires no opaque DSP state:
DAWs persist its control ports and can load `factory.ttl`. The LV2 editor is a
separate instance that receives host `port_event` updates. Its baseview event
thread writes a fixed atomic mailbox. The LV2 idle callback drains that mailbox
and invokes host writes on the host's UI thread. It does not retain host port
buffer pointers. Meter notifications are declared in the Turtle metadata.

Only ABI boundaries contain raw host/window pointers. Their owners are the host,
and cleanup stops the editor before the host destroys its parent window. LV2
control values are sanitized before processing, disconnected optional controls
use defaults, and nonfinite input samples are replaced with zero. Excessive input
samples are bounded at ±64; normal audio and floating-point overs remain unclipped.

## Extending platforms

Keep DSP free of GUI and host dependencies. Add Windows/macOS CI and exercise
NIH-plug's platform-specific embedding and packaging before claiming support.
The LV2 core is portable; the custom editor needs new platform descriptors and
parent-window handling. No Linux-only calls belong in `sps-dsp` or `sps-ott`.
