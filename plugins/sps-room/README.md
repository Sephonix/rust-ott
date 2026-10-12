# SPS Room

True-stereo early energy and modulated algorithmic reverb for Linux, in CLAP,
VST3 and LV2. Room implements the public control families of ValhallaRoom with
original DSP, tuning, factory presets and UI. It is independent software with no
Valhalla affiliation, preset compatibility, or claim of matching its exact sound.

## Working with Room

Start with Studio, Drum Room or Concert Hall. Mix balances unfiltered dry audio
with the reverb. Predelay separates the source attack from the room. Decay sets
the nominal middle-band RT60. Late Blend crossfades early energy and the late
tail: 0% isolates early energy, 100% isolates the late network.

Early Size spreads 24 reflection taps across up to one second. Early Diffusion
fills the sparse pattern. Early Cross spreads reflections across stereo; zero
preserves separate channel histories. Early Mod Rate/Depth moves reflection
taps for subtle movement or chorusing. Early Send crossfades the late network's
excitation from direct predelayed audio to early energy. Space feeds filtered
early energy back through predelay, creating repeated rooms and diffuse echoes.

Late Size scales the network's physical delay times. Late Cross couples two
independent eight-line networks through an energy-preserving stereo rotation.
Late Mod Rate/Depth decorrelates moving delay heads. Bass Decay and High Decay
multiply the nominal Decay below/above their crossover regions. Crossovers are
bounded at the actual sample rate. High Cut filters wet audio and Space feedback;
Low Cut gradually attenuates bass with a shelf (0 Hz is transparent). The two
paths have independent filter history.

The twelve original characters vary delay geometry, excitation density,
modulation and brightness: Large Room, Medium Room, Bright Hall, Chamber,
Dark Room, Dark Chamber, Dark Space, Vast Space, Small Space, Centered Space,
Deep Space and Dense Room. These are original profiles of a shared SPS network,
rather than reproductions of Valhalla's twelve proprietary algorithms.

Room has zero dry-path latency. Delay and size automation intentionally moves
read heads, producing Doppler pitch changes. Continuous controls smooth over
20 ms with delay interpolation between control ticks; character changes fade
wet audio down/up over 5 ms. Factory Gated Energy uses a finite early response,
not a dynamics gate. Nominal RT60 depends on frequency, modulation and network
size; very long decay values can ring for a long time.

## Presets and project recall

Choose a factory preset and press **Load factory**, or select a user preset and
press **Load**. **Lock Mix** keeps the current wet/dry blend while loading either.
To save, type a name and press **Save new preset**. Names accept letters, numbers,
spaces, underscores and hyphens. Existing names are protected: save under a new
name to make a variation. **Refresh** rescans files added from another instance.

User presets are versioned JSON under `Documents/SPS/Room/`. Linux XDG
user-dirs configuration is respected, including localized Documents folders;
the fallback is `~/Documents/SPS/Room/`. Files work across the three formats.
Only editor threads access disk. A failed/corrupt/wrong-plugin preset reports an
error and does not apply any controls. Saves publish complete files atomically.
DAWs also persist all parameters in their normal project state; delay/reverb
history is cleared on activation/reset and is not serialized.

## Build, validate and install

Requires stable Rust, a C/C++ toolchain, pkg-config, X11/XCB/XCursor development
libraries and OpenGL. On Debian/Ubuntu:

```sh
sudo apt-get install build-essential pkg-config libx11-dev libx11-xcb-dev libxcb1-dev libxcursor-dev libgl1-mesa-dev liblilv-0-0 lv2-dev sordi
cargo xtask bundle sps-room --release --locked
python3 plugins/sps-room/scripts/smoke_lv2.py
python3 plugins/sps-room/scripts/smoke_lv2.py --ui --exercise-ui --seconds 3
python3 scripts/package.py sps-room
```

Outputs: `target/bundled/SPS Room.clap`, `SPS Room.vst3`, and
`sps-room.lv2`; the distribution archive is under `dist/`. Run
`cargo xtask install sps-room --release --locked` to install into your DAW's
Linux scan directories. Each plugin is self-contained; no other SPS plugin or
separate shared Rust library needs to be installed. UI checks need X11/XWayland
and OpenGL (CI uses Xvfb). Portable source is not a claim of validated macOS or
Windows support. [Validation](../../docs/time-effects-validation.md) records fresh
measurements, limits, and host checks. Original code is GPL-3.0-or-later; see
[license](../../LICENSE) and [dependency notices](../../THIRD_PARTY.md).

## Stable controls

The DSP schema is authoritative. These IDs and LV2 port ordering are stable.
Audio ports are 0–3; controls begin at 4; two output meters follow.

| ID | Control | Range | Default |
| --- | --- | --- | --- |
| `mix` | Mix | 0.0 – 100.0 % | 35.0 |
| `output` | Output | -24.0 – 24.0 dB | 0.0 |
| `bypass` | Bypass | 0.0 – 1.0  | 0.0 |
| `mode` | Character | 0.0 – 11.0  | 0.0 |
| `predelay` | Predelay | 0.0 – 500.0 ms | 15.0 |
| `decay` | Decay | 0.1 – 100.0 s | 2.0 |
| `high_cut` | High Cut | 200.0 – 20000.0 Hz | 12000.0 |
| `low_cut` | Low Cut | 0.0 – 2000.0 Hz | 0.0 |
| `depth` | Late Blend | 0.0 – 100.0 % | 70.0 |
| `early_size` | Early Size | 1.0 – 1000.0 ms | 45.0 |
| `early_cross` | Early Cross | 0.0 – 100.0 % | 30.0 |
| `early_rate` | Early Mod Rate | 0.01 – 10.0 Hz | 0.4 |
| `early_mod` | Early Mod Depth | 0.0 – 100.0 % | 10.0 |
| `early_send` | Early Send | 0.0 – 100.0 % | 60.0 |
| `diffusion` | Early Diffusion | 0.0 – 100.0 % | 85.0 |
| `space` | Space | 0.0 – 100.0 % | 0.0 |
| `late_size` | Late Size | 0.0 – 100.0 % | 50.0 |
| `late_cross` | Late Cross | 0.0 – 100.0 % | 60.0 |
| `late_rate` | Late Mod Rate | 0.01 – 10.0 Hz | 0.5 |
| `late_mod` | Late Mod Depth | 0.0 – 100.0 % | 15.0 |
| `bass_mult` | Bass Decay | 0.25 – 4.0 x | 1.2 |
| `bass_xover` | Bass Crossover | 20.0 – 4000.0 Hz | 600.0 |
| `high_mult` | High Decay | 0.1 – 2.0 x | 0.5 |
| `high_xover` | High Crossover | 500.0 – 20000.0 Hz | 6000.0 |
