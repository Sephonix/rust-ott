# Room, Delay and Echo validation

Fresh local runs on Linux x86-64, Arch Linux/KDE Wayland with XWayland,
Intel Core Ultra 9 185H, Rust 1.99.0, 2026-10-11. Research notes, intermediate
logs and native screenshots are outside the repository in `../sps-research`.
These results use the newly built exports; they are separate from historical
OTT validation. Compilation and validators do not establish exact Valhalla
sound matching or subjective production quality.

| Check | Fresh result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | 52 tests passed: 9 shared primitives, 13 OTT, 8 Room, 10 Delay, 6 Echo, 4 LV2 lifecycle, 1 preset storage, 1 transport parser |
| `cargo xtask bundle --release --locked` | All four plugins built in CLAP, VST3 and LV2; new Room/Echo editors also rebuilt after sizing cleanup |
| CLAP validator 0.4.1 (`152b982`) | Each new plugin: 34 passed, 0 failures, 0 warnings, 10 not applicable |
| Steinberg VST3 SDK 3.8.1 validator | Each new plugin: 47 passed, 0 failed |
| `lv2_validate` | 0 errors for all four plugin bundles |
| New LV2 runtime smoke tests | Discovery, port counts (Room 30, Delay 45, Echo 18), exact bypass, finite stereo audio, meters and zero-frame runs passed |
| LV2 tempo transport | Delay and Echo received a real URID map/time:Position sequence at 60 BPM and produced the exact expected 500 ms eighth-note repeat |
| Native LV2 editor checks | Room, Delay and Echo: X11 embedding, first paint, idle, meters, Mix edits, factory load, cleanup passed; native screenshots visually inspected |
| Native preset workflow | All three editors saved JSON under isolated temporary Documents/SPS directories, changed Mix and loaded the saved values through host callbacks |
| OTT regression runtime | Audio smoke and native editor/Amount/factory-reset smoke passed after shared additions |
| Metadata reproducibility | Two generations byte-identical; new checked-in metadata matches release bundles |
| Packaging | Three independent archives include all declared formats, original manuals, docs, workspace license, dependency notices and BUILD.json |

VST3 supports f32 processing; f64 is reported as unsupported. CLAP skips tests
for note/MIDI/voice extensions these effects do not advertise. Native editor
checks used the existing XWayland display; local Xvfb was unavailable. CI runs
these checks under Xvfb. No local Windows/macOS builds or broad DAW compatibility
matrix has been run. No binaries were installed into the user's DAW scan paths.

## Behavior and stability coverage

DSP tests cover finite input/control handling, independent history, activation
reset, true bypass, silence, common sample rates through 192 kHz and 768 kHz
initialization, mono processing, stable IDs and factory values, and zero heap
allocation under extreme automation. Input denormals are flushed before they
can feed recursive state. CLAP's earlier Delay denormal performance warning
was corrected and the final validator runs have zero warnings.

Room tests check decaying tails and stereo coupling for all twelve profiles,
zero-crossfeed isolation, predelay and finite early energy timing, and the
observable tail-energy effects of bass/high decay multipliers. Delay tests
check exact repeat time and ping-pong routing, independent dual histories,
four audible heads, host sync, ducking, and all 85 mode/style combinations
under 200% feedback. Regressions specifically distinguish reverse from forward
reads in all five styles and confirm Quad Repeat head-D feedback shifts the
next repeat again even when head D is not audible.

The FIR analytic shifter rejects the image below −50 dB in the tested 1 kHz
44.1/48/96 kHz cases. The low-latency quadrature shifter rejects it below −60 dB
at 100 Hz, 1 kHz and 10 kHz over those rates, for positive and negative shifts.
Pitch tests distinguish multiplicative 1.5× shifting from additive Hz shifting
and check exact unity. The useful analytic bands exclude DC and Nyquist; no
claim of these rejection limits at every frequency/sample rate is made.

Preset tests check round trips, sorted discovery, existing-file protection,
traversal rejection, cross-plugin isolation, schema/version/range corruption,
64 KiB limits, and temporary-file cleanup. Transport tests check timestamps
and truncated/malformed atoms. Runtime tests cover the ABI and generated
metadata in addition to direct engine tests.

## Reproduction

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask bundle --release --locked
cargo xtask metadata
for plugin in room delay echo; do
  lv2_validate target/bundled/sps-$plugin.lv2/*.ttl
  python3 plugins/sps-$plugin/scripts/smoke_lv2.py
  python3 plugins/sps-$plugin/scripts/smoke_lv2.py --ui --exercise-ui --exercise-presets --seconds 6
  python3 scripts/package.py sps-$plugin
done
```

The preset UI test needs `xdotool`; its temporary XDG configuration avoids
writing test presets into the real Documents directory. Optional
`--screenshot /absolute/path.png` needs ImageMagick. The dedicated smoke scripts
use a shared ABI harness with each plugin's own identities and port counts.
For validator installation/version pins, see [validation.md](validation.md).
Replace `SPS OTT` in those commands with `SPS Room`, `SPS Delay` or `SPS Echo`.

## Release throughput

Run `cargo bench -p sps-room-dsp -p sps-delay-dsp -p sps-echo-dsp --bench throughput
--locked`. Each case processes 480,000 stereo frames of precomputed two-channel
signals. Static cases cover every character with the factory routing/defaults;
automated cases change Mix and Room Late Size or Delay/Echo time every 32
samples. The benchmark includes DSP/control updates and uses `black_box`;
it excludes host wrappers, GUI drawing and initialization. Core percentage is
`nanoseconds/frame × sample_rate / 10^7`. The values below are measured ranges
across characters, not a universal CPU guarantee or a measurement of all
possible Quad/diffusion/feedback settings.

| Plugin | Rate | Static ns/frame | Automated ns/frame | Static core | Automated core |
| --- | --- | --- | --- | --- | --- |
| Room | 48 kHz | 521.7–551.3 | 530.9–565.4 | 2.50–2.65% | 2.55–2.71% |
| Room | 96 kHz | 519.4–543.7 | 538.6–587.8 | 4.99–5.22% | 5.17–5.64% |
| Delay | 48 kHz | 69.9–171.8 | 76.3–178.1 | 0.34–0.82% | 0.37–0.85% |
| Delay | 96 kHz | 72.5–173.2 | 89.4–181.5 | 0.70–1.66% | 0.86–1.74% |
| Echo | 48 kHz | 82.2–82.2 | 82.2–82.2 | 0.39–0.39% | 0.39–0.39% |
| Echo | 96 kHz | 81.4–81.4 | 82.8–82.8 | 0.78–0.78% | 0.79–0.79% |

Delay buffers are allocated for the maximum span at initialization; memory
scales with sample rate. All dependencies are built into each exported library
except ordinary system GUI/runtime libraries. Each archive works without another
SPS plugin installed. Factory sound design and the original algorithm limits
are documented in the individual manuals and [design notes](time-effects-design.md).
