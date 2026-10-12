# SPS Delay

Stereo colored delay for Linux in CLAP, VST3 and LV2. Delay implements the public
control families of ValhallaDelay, including seventeen delay characters and five
routing styles, using original DSP, tuning, presets and UI. It is independent
software with no Valhalla affiliation, preset compatibility, or exact-sound claim.

## Delay time and routing

Delay Left/Right cover sub-sample requests through 20 seconds (the minimum
physical feedback delay is one sample). Sync can use free milliseconds, straight,
dotted or triplet notes. Note spans 1/64 through 32 bars; bars assume 4/4.
CLAP/VST3 read host transport; LV2 receives optional time:Position atoms. If a
host provides no tempo, 120 BPM is used. Synced times are capped at 20 seconds.
Free times stay stored when switching sync or styles.

| Style | Routing |
| --- | --- |
| Single | Independent channels with the same time and modulation; Spread adds to left and subtracts from right. |
| Dual | Independent left/right time, modulation and feedback. |
| Ratio | Right time = left time × Right Ratio; an orthogonal feedback matrix builds echo density. |
| Ping Pong | Inputs are averaged into the left delay and repeats alternate channels. Anti-phase inputs cancel in this intentional mono injection. |
| Quad | Four switchable stereo heads. Spacing compresses toward the early or late heads and offsets the stereo taps. Repeat feeds back head D regardless of selection; Swell uses the audible head sum. |

Width scales the wet side signal: 100% preserves sides, 0% collapses to mono,
−100% exchanges channels. Feedback is a percentage of loop amplitude. Above
100% intentionally creates self-oscillation, contained by a soft safety ceiling;
multiple heads and diffusion can oscillate earlier. Output controls final gain.
Use low input levels while exploring high-feedback settings.

## Characters and shaping

| Character | Original SPS behavior |
| --- | --- |
| Tape | Band-limited saturation, time-dependent wow/flutter, signal-dependent age noise. |
| HiFi | Wider-band saturation with independent rate/depth modulation. |
| Bucket Brigade | Dark companded echo, saturation and age noise. |
| Digital | Cleaner delay with age-dependent quantization. |
| Spectral | Frequency shifting in Hz, stereo frequency detune, saturation and diffusion. |
| Pitch | Moving-head semitone pitch shifting, independent stereo cents detune. |
| Reverse Pitch | Overlapping backwards read heads; Pitch changes backward playback rate. |
| Ducked BBD | Bucket brigade with an input-driven wet attenuator. |
| Clarity | Clean filtered and limited delay with ducking. |
| Ducked Tape | Tape character with less noise and ducking. |
| Ducked Pitch | Pitch shifting with ducking. |
| LoFi | Dark saturation, strong age noise and quantization. |
| Crystal | Clean limiting; Future bypasses the color lowpass and puts user EQ after the feedback loop. |
| Phaser | 4/6/12 allpass stages selected by Era, modulation and Age-controlled resonance. |
| Rich Pitch | Blend of normal delayed and pitch-shifted signals using Pitch Blend. |
| Chrome Tape | Tape character with greatly reduced age noise. |
| Analog | A brighter companded echo with bucket-brigade saturation. |

Era changes brightness for color modes and phaser stage count. Drive boosts
nonlinear coloration and reduces output gain by half that boost in dB; clean
modes use it to set a limiter knee. Age controls noise/quantization/brightness,
or phaser resonance. Wow/Flutter apply to Tape, Ducked Tape, Chrome Tape and LoFi;
other modulation modes use Mod Rate/Depth. Low Cut/High Cut shape the feedback
(except Crystal, where they shape the output). Diffusion cascades four allpass
delays, scaled by Diffusion Size and delay time; below 1% it is bypassed.
Ducking uses a stereo peak envelope with 5 ms attack and 150 ms release.

Controls stay visible and retain values when inactive. SPS deliberately uses
its own compact common editor and does not copy Valhalla's artwork or dynamic
control layout. The color modes approximate sound families, not measured models
of the vendor's hardware emulations. The phaser uses one cascade per stereo
channel, including Quad; it does not duplicate a distinct phaser per tape head.
Ghost-style Spectral uses a 128-sample analytic FIR group delay; pitch shifts use
80 ms overlapping read windows. These are wet effect delays, with zero dry-path
latency. Reverse Pitch uses two overlapping backwards grains within the selected
delay span. Delay-time automation produces Doppler changes; routing/character/era
changes fade wet audio over 5 ms rather than deleting stored repeats.

## Presets and project recall

Choose a factory preset and press **Load factory**, or select a user preset and
press **Load**. **Lock Mix** keeps the current wet/dry blend while loading either.
To save, type a name and press **Save new preset**. Names accept letters, numbers,
spaces, underscores and hyphens. Existing names are protected: save under a new
name to make a variation. **Refresh** rescans files added from another instance.

User presets are versioned JSON under `Documents/SPS/Delay/`. Linux XDG
user-dirs configuration is respected, including localized Documents folders;
the fallback is `~/Documents/SPS/Delay/`. Files work across the three formats.
Only editor threads access disk. A failed/corrupt/wrong-plugin preset reports an
error and does not apply any controls. Saves publish complete files atomically.
DAWs also persist all parameters in their normal project state; delay/reverb
history is cleared on activation/reset and is not serialized.

## Build, validate and install

Requires stable Rust, a C/C++ toolchain, pkg-config, X11/XCB/XCursor development
libraries and OpenGL. On Debian/Ubuntu:

```sh
sudo apt-get install build-essential pkg-config libx11-dev libx11-xcb-dev libxcb1-dev libxcursor-dev libgl1-mesa-dev liblilv-0-0 lv2-dev sordi
cargo xtask bundle sps-delay --release --locked
python3 plugins/sps-delay/scripts/smoke_lv2.py
python3 plugins/sps-delay/scripts/smoke_lv2.py --ui --exercise-ui --seconds 3
python3 scripts/package.py sps-delay
```

Outputs: `target/bundled/SPS Delay.clap`, `SPS Delay.vst3`, and
`sps-delay.lv2`; the distribution archive is under `dist/`. Run
`cargo xtask install sps-delay --release --locked` to install into your DAW's
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
| `mode` | Character | 0.0 – 16.0  | 0.0 |
| `style` | Style | 0.0 – 4.0  | 0.0 |
| `era` | Era | 0.0 – 2.0  | 1.0 |
| `time_l` | Delay Left | 0.01 – 20000.0 ms | 350.0 |
| `time_r` | Delay Right | 0.01 – 20000.0 ms | 500.0 |
| `sync_l` | Left Sync | 0.0 – 3.0  | 0.0 |
| `sync_r` | Right Sync | 0.0 – 3.0  | 0.0 |
| `division_l` | Left Note | 0.0 – 11.0  | 4.0 |
| `division_r` | Right Note | 0.0 – 11.0  | 4.0 |
| `spread` | Spread | -100.0 – 100.0 ms | 0.0 |
| `ratio` | Right Ratio | 1.0 – 200.0 % | 61.8 |
| `feedback` | Feedback | 0.0 – 200.0 % | 40.0 |
| `width` | Width | -100.0 – 100.0 % | 100.0 |
| `drive` | Drive | 0.0 – 36.0 dB | 0.0 |
| `age` | Age / Resonance | 0.0 – 100.0 % | 10.0 |
| `diffusion` | Diffusion | 0.0 – 100.0 % | 0.0 |
| `diff_size` | Diffusion Size | 1.0 – 100.0 % | 30.0 |
| `mod_rate` | Mod Rate | 0.01 – 20.0 Hz | 0.5 |
| `mod_depth` | Mod Depth | 0.0 – 100.0 % | 0.0 |
| `wow` | Wow | 0.0 – 100.0 % | 10.0 |
| `flutter` | Flutter | 0.0 – 100.0 % | 5.0 |
| `shift` | Frequency Shift | -1000.0 – 1000.0 Hz | 0.0 |
| `freq_detune` | Frequency Detune | 0.0 – 100.0 Hz | 0.0 |
| `pitch` | Pitch Shift | -24.0 – 24.0 st | 0.0 |
| `detune` | Pitch Detune | 0.0 – 100.0 ct | 0.0 |
| `pitch_mix` | Pitch Blend | 0.0 – 100.0 % | 100.0 |
| `low_cut` | Low Cut | 0.0 – 2000.0 Hz | 20.0 |
| `high_cut` | High Cut | 200.0 – 20000.0 Hz | 16000.0 |
| `duck` | Ducking | 0.0 – 100.0 % | 50.0 |
| `spacing` | Tap Spacing | -100.0 – 100.0 % | 0.0 |
| `tap_a` | Tap A | 0.0 – 1.0  | 1.0 |
| `tap_b` | Tap B | 0.0 – 1.0  | 1.0 |
| `tap_c` | Tap C | 0.0 – 1.0  | 1.0 |
| `tap_d` | Tap D | 0.0 – 1.0  | 1.0 |
| `swell` | Swell Feedback | 0.0 – 1.0  | 0.0 |
