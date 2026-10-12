# SPS Echo

A frequency-shifting feedback echo for Linux in CLAP, VST3 and LV2. Echo
implements the public control family of ValhallaFreqEcho using original DSP,
factory presets and UI. It is independent software with no Valhalla affiliation,
preset compatibility or exact-sound claim.

## Using Echo

Mix blends the original dry signal and filtered repeats. Delay sets the feedback
loop length up to one second. Sync selects free time, straight, dotted or triplet
notes; Note spans 1/64 through 32 bars (4/4). CLAP/VST3 use host tempo and LV2 uses
optional time:Position atoms. Without transport, tempo defaults to 120 BPM;
synced delay lengths are capped at one second.

Frequency Shift adds/subtracts a fixed number of Hertz to spectral components;
it does not multiply frequencies as a semitone pitch shifter would. Every trip
through the feedback loop shifts the next repeat again. Stereo Shift applies
opposite shifts to left/right, making subtle slow movement or dramatic opposing
glissandos. Turn it off to shift both channels in the same direction. Independent
channel histories preserve stereo audio; mono hosts fold the wet stereo pair.
Feedback runs from no repeats to sustained loops with a soft safety ceiling.
Low Cut and High Cut filter repeats inside feedback. Output sets final gain.

Try Dub for conventional echo, Ascending/Descending for cascading shifts,
Stereo Orbit for slowly moving flanging, or Barberpole for short-loop motion.
The analytic signal uses sixteen second-order allpasses with independently
numerically designed quadrature coefficients. There is no fixed block/FIR
latency; frequency-dependent phase delay is part of the wet signal. A zero shift
fades to a transparent shifter path. Minimum physical delay is one sample. The
useful quadrature band excludes DC/Nyquist; measured image rejection is tested
at 100, 1000 and 10000 Hz at 44.1/48/96 kHz. Extreme frequency shifts can wrap
spectral components around DC or Nyquist, as a discrete-time frequency shifter
does; High Cut helps contain those textures.

Continuous controls smooth over 20 ms and moving delay heads interpolate
between control ticks. Fast delay automation intentionally makes pitch slews.
The dry path has zero latency and settled bypass is sample exact for finite
normal input. Nonfinite, excessive and denormal input is sanitized.

## Presets and project recall

Choose a factory preset and press **Load factory**, or select a user preset and
press **Load**. **Lock Mix** keeps the current wet/dry blend while loading either.
To save, type a name and press **Save new preset**. Names accept letters, numbers,
spaces, underscores and hyphens. Existing names are protected: save under a new
name to make a variation. **Refresh** rescans files added from another instance.

User presets are versioned JSON under `Documents/SPS/Echo/`. Linux XDG
user-dirs configuration is respected, including localized Documents folders;
the fallback is `~/Documents/SPS/Echo/`. Files work across the three formats.
Only editor threads access disk. A failed/corrupt/wrong-plugin preset reports an
error and does not apply any controls. Saves publish complete files atomically.
DAWs also persist all parameters in their normal project state; delay/reverb
history is cleared on activation/reset and is not serialized.

## Build, validate and install

Requires stable Rust, a C/C++ toolchain, pkg-config, X11/XCB/XCursor development
libraries and OpenGL. On Debian/Ubuntu:

```sh
sudo apt-get install build-essential pkg-config libx11-dev libx11-xcb-dev libxcb1-dev libxcursor-dev libgl1-mesa-dev liblilv-0-0 lv2-dev sordi
cargo xtask bundle sps-echo --release --locked
python3 plugins/sps-echo/scripts/smoke_lv2.py
python3 plugins/sps-echo/scripts/smoke_lv2.py --ui --exercise-ui --seconds 3
python3 scripts/package.py sps-echo
```

Outputs: `target/bundled/SPS Echo.clap`, `SPS Echo.vst3`, and
`sps-echo.lv2`; the distribution archive is under `dist/`. Run
`cargo xtask install sps-echo --release --locked` to install into your DAW's
Linux scan directories. Each plugin is self-contained; no other SPS plugin or
separate shared Rust library needs to be installed. UI checks need X11/XWayland
and OpenGL (CI uses Xvfb). Portable source is not a claim of validated macOS or
Windows support. [Validation](../../docs/time-effects-validation.md) records fresh
measurements, limits, and host checks. Original code is GPL-3.0-or-later; see
[license](../../LICENSE) and [dependency notices](../../THIRD_PARTY.md).

## Stable controls

The DSP schema is authoritative. These IDs and LV2 port ordering are stable.
Audio ports are 0–3; controls begin at 4; two output meters follow.
An optional tempo atom input follows the meters.

| ID | Control | Range | Default |
| --- | --- | --- | --- |
| `mix` | Mix | 0.0 – 100.0 % | 35.0 |
| `output` | Output | -24.0 – 24.0 dB | 0.0 |
| `bypass` | Bypass | 0.0 – 1.0  | 0.0 |
| `time` | Delay | 0.01 – 1000.0 ms | 250.0 |
| `sync` | Sync | 0.0 – 3.0  | 0.0 |
| `division` | Note | 0.0 – 11.0  | 4.0 |
| `shift` | Frequency Shift | -1000.0 – 1000.0 Hz | 0.0 |
| `feedback` | Feedback | 0.0 – 100.0 % | 40.0 |
| `low_cut` | Low Cut | 0.0 – 2000.0 Hz | 20.0 |
| `high_cut` | High Cut | 200.0 – 20000.0 Hz | 12000.0 |
| `stereo` | Stereo Shift | 0.0 – 1.0  | 1.0 |
