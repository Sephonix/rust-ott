# Time effects design and provenance

Room, Delay and Echo are independent GPL Rust implementations informed by
public behavioral descriptions. Research was conducted outside the repository
in `../sps-research`. No vendor manual, proprietary source, binary, artwork,
coefficient table, sample library or factory preset is included in this source
or used as implementation input. The original SPS UI uses the suite's existing
egui integration. Tests exercise mathematical and host contracts rather than
comparing with proprietary executables. No exact sonic equivalence is asserted.

The vendor's [support documentation](https://valhalladsp.com/support/) describes
its plugins as self-documenting and links official control guides as manuals.
Direct downloads returned HTTP 403 during this research; indexed official-page
contents were consulted. Sources include:

- [Room product/mode descriptions](https://valhalladsp.com/shop/reverb/valhalla-room/),
  [early controls](https://valhalladsp.com/2011/05/18/valhallaroom-the-early-controls/),
  [late controls](https://valhalladsp.com/2011/05/18/valhallaroom-the-late-controls/),
  and [Space/Lo Cut update](https://valhalladsp.com/2023/11/20/valhallaroom-updated-to-2-0-0-new-space-lo-cut-controls/).
- [Delay controls](https://valhalladsp.com/2019/04/16/valhalladelay-the-controls/),
  [styles](https://valhalladsp.com/2019/04/16/valhalladelay-the-style-control/),
  [current mode inventory](https://valhalladsp.com/shop/delay/valhalladelay/),
  and [Chrome Tape/Analog update](https://valhalladsp.com/2024/11/23/valhalladelay-updated-to-version-3-0-0-two-new-delay-modes-chrome-tape-and-analog/).
- [FreqEcho description](https://valhalladsp.com/shop/delay/valhalla-freq-echo/).
- [LV2 specification](https://github.com/lv2/lv2) and its installed C headers
  define the host ABI; [atom documentation](https://lv2plug.in/ns/ext/atom)
  describes transport sequence framing.

## Original algorithms

Room uses two eight-line feedback networks with Householder reflections and an
orthogonal stereo rotation. Delay times, reflection signs/spacing, excitation
allpasses and twelve profile tunings were chosen for SPS. Per-line feedback
amplitudes derive from `10^(-3 × line_seconds / RT60_seconds)`; cascaded shelves
shape bass/high decay. Modulation uses independent complex oscillators. Early
energy is a finite 24-tap pattern, with interpolation, optional crossfeed and
filtered Space feedback. The low cut shelf has an original gradual attenuation
curve. This implements the documented control family with a distinct reverb.

Delay uses circular delay buffers, orthogonal Ratio mixing, alternating Ping
Pong feedback and four-head reads. All characters apply to every routing style;
Quad Repeat has an independent fully processed head-D feedback path even when
head D is disabled in the audible output. Tape/BBD coloration, power-law companding,
noise amplitude, era bandwidth, nonlinear limiting, diffusion sizes, ducking
and modulation scales are original approximations. They do not replicate
private physical modeling or compander designs. Spectral frequency shifting
uses a windowed ideal Hilbert transform, whose coefficients derive directly
from `2/(pi*k)` on odd taps with a Blackman window. Pitch shifting uses two
complementary moving read windows; Reverse Pitch uses backwards grains. Phaser
uses a local 4/6/12-stage allpass cascade per channel. Rich Pitch blends shifted
and unshifted reads. See the Delay manual for wet latency and Quad differences.

Echo uses low-latency quadrature allpasses. Their coefficients were generated
by numerical fitting of the public mathematical transfer function
`(z^-2-a)/(1-a*z^-2)`: the two eight-stage paths differ by a quarter-cycle after
a one-sample delay on the second path. `scripts/design_quadrature.py` reproduces
the design procedure (optional offline NumPy/SciPy). Plugin builds have no
Python/scientific-library requirement. Double precision recursive state and
periodic oscillator normalization avoid long-run drift. At zero shift it fades
to transparent audio; nonzero shifts have frequency-dependent wet phase delay.

## Compatibility and limits

All three exports have new SPS identities and independent factory presets.
They cannot import Valhalla's preset format or replace its instances in DAW
projects. Public control behavior does not identify a proprietary algorithm's
implementation or sound; these plugins provide equivalent effect families,
not validated emulations. Sample-rate support is 8–768 kHz with filters bounded
below Nyquist, and fixed memory allocated for the supported delay span. Higher
sample rates increase memory in proportion to rate. Host/GUI/runtime validation
is currently Linux x86-64. Unit tests and validators cannot establish subjective
mix quality or compatibility with every DAW; listening and real-session testing
remain appropriate before critical use.
