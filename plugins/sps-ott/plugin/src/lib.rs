// The pinned upstream VST3 bindings emit a deprecated expression semicolon.
#![allow(semicolon_in_expressions_from_non_local_macros)]
//! CLAP and VST3 wrappers. All audio work lives in sps-ott-dsp.
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, EguiState};
use sps_ott_dsp::{params::*, Engine, MeterBus};
use std::sync::Arc;

pub struct OttParams {
    // Never resized after construction: parameter pointers stay valid for the Arc's lifetime.
    values: [FloatParam; PARAM_COUNT],
    editor_state: Arc<EguiState>,
    bypass: BoolParam,
}
impl Default for OttParams {
    fn default() -> Self {
        Self {
            values: std::array::from_fn(|i| {
                let spec = SPECS[i];
                let name = if i < GLOBAL_COUNT {
                    spec.name.to_owned()
                } else {
                    format!(
                        "{} {}",
                        BAND_NAMES[(i - GLOBAL_COUNT) / BAND_STRIDE],
                        spec.name
                    )
                };
                let range = if spec.logarithmic {
                    FloatRange::Skewed {
                        min: spec.min,
                        max: spec.max,
                        factor: FloatRange::skew_factor(-2.0),
                    }
                } else {
                    FloatRange::Linear {
                        min: spec.min,
                        max: spec.max,
                    }
                };
                let param = FloatParam::new(name, spec.default, range)
                    .with_unit(if spec.unit.is_empty() {
                        ""
                    } else {
                        match spec.unit {
                            "%" => " %",
                            "dB" => " dB",
                            "Hz" => " Hz",
                            "ms" => " ms",
                            _ => ":1",
                        }
                    })
                    .with_value_to_string(Arc::new(move |v| {
                        if i == DETECTION {
                            if v < 0.5 { "RMS" } else { "Peak" }.to_owned()
                        } else if spec.discrete {
                            if v < 0.5 { "Off" } else { "On" }.to_owned()
                        } else if i >= GLOBAL_COUNT
                            && (i - GLOBAL_COUNT) % BAND_STRIDE == DOWN_RATIO
                            && v >= 999.0
                        {
                            "∞".to_owned()
                        } else {
                            format!("{v:.2}")
                        }
                    }));
                let param = param.with_string_to_value(Arc::new(move |text| {
                    let text = text.trim().to_lowercase();
                    if i == DETECTION {
                        return match text.as_str() {
                            "rms" => Some(0.0),
                            "peak" => Some(1.0),
                            _ => text.parse().ok(),
                        };
                    }
                    if spec.discrete {
                        return match text.as_str() {
                            "on" => Some(1.0),
                            "off" => Some(0.0),
                            _ => text.parse().ok(),
                        };
                    }
                    if text.contains('∞') || text == "inf" || text == "inf:1" {
                        return Some(INFINITE_RATIO);
                    }
                    let text = text
                        .trim_end_matches(":1")
                        .trim_end_matches(spec.unit.to_lowercase().as_str())
                        .trim();
                    text.parse().ok()
                }));
                if spec.discrete {
                    param.with_step_size(1.0)
                } else {
                    param
                }
            }),
            editor_state: EguiState::from_size(sps_ott_gui::WIDTH, sps_ott_gui::HEIGHT),
            bypass: BoolParam::new("Bypass", false).make_bypass(),
        }
    }
}
// SAFETY: pointers refer to a fixed array inside the Params Arc held by NIH-plug.
unsafe impl Params for OttParams {
    fn param_map(&self) -> Vec<(String, ParamPtr, String)> {
        self.values
            .iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    SPECS[i].id.to_owned(),
                    if i == BYPASS {
                        self.bypass.as_ptr()
                    } else {
                        p.as_ptr()
                    },
                    if i < GLOBAL_COUNT {
                        String::new()
                    } else {
                        BAND_NAMES[(i - GLOBAL_COUNT) / BAND_STRIDE].to_owned()
                    },
                )
            })
            .collect()
    }
}
impl OttParams {
    fn snapshot(&self) -> [f32; PARAM_COUNT] {
        std::array::from_fn(|i| {
            if i == BYPASS {
                if self.bypass.value() {
                    1.0
                } else {
                    0.0
                }
            } else {
                self.values[i].value()
            }
        })
    }
}

pub struct Ott {
    params: Arc<OttParams>,
    meters: Arc<MeterBus>,
    engine: Engine,
}
impl Default for Ott {
    fn default() -> Self {
        Self {
            params: Arc::new(OttParams::default()),
            meters: Arc::new(MeterBus::default()),
            engine: Engine::new(48000.0),
        }
    }
}
impl Plugin for Ott {
    const NAME: &'static str = "SPS OTT";
    const VENDOR: &'static str = "Sephonix";
    const URL: &'static str = "https://github.com/Sephonix/sps-dsp";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();
    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }
    fn editor(&mut self, _: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let meters = self.meters.clone();
        create_egui_editor(
            params.editor_state.clone(),
            sps_ott_gui::UiState::default(),
            |ctx, _| sps_ott_gui::configure(ctx),
            move |ctx, setter, state| {
                let values = params.snapshot();
                sps_ott_gui::draw(ctx, state, &values, &meters.snapshot(), |i, value| {
                    if i == BYPASS {
                        setter.begin_set_parameter(&params.bypass);
                        setter.set_parameter(&params.bypass, value >= 0.5);
                        setter.end_set_parameter(&params.bypass);
                        return;
                    }
                    let param = &params.values[i];
                    setter.begin_set_parameter(param);
                    setter.set_parameter(param, value);
                    setter.end_set_parameter(param);
                });
            },
        )
    }
    fn initialize(
        &mut self,
        _: &AudioIOLayout,
        config: &BufferConfig,
        _: &mut impl InitContext<Self>,
    ) -> bool {
        self.engine = Engine::new(config.sample_rate);
        self.engine.set_parameters(&self.params.snapshot());
        self.engine.reset();
        true
    }
    fn reset(&mut self) {
        self.engine.set_parameters(&self.params.snapshot());
        self.engine.reset();
        self.meters.publish(self.engine.meters());
    }
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _: &mut AuxiliaryBuffers,
        _: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.engine.set_parameters(&self.params.snapshot());
        let channels = buffer.channels();
        for mut frame in buffer.iter_samples() {
            let input = [
                *frame.get_mut(0).unwrap(),
                if channels == 2 {
                    *frame.get_mut(1).unwrap()
                } else {
                    0.0
                },
            ];
            let output = self.engine.process_frame(input, channels);
            for (c, sample) in output.iter().enumerate().take(channels) {
                *frame.get_mut(c).unwrap() = *sample;
            }
        }
        if self.params.editor_state.is_open() {
            self.meters.publish(self.engine.meters());
        }
        ProcessStatus::Normal
    }
}
impl ClapPlugin for Ott {
    const CLAP_ID: &'static str = "com.sephonix.sps-ott";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Aggressive three-band upward / downward compression");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Compressor,
        ClapFeature::Stereo,
        ClapFeature::Mono,
    ];
}
impl Vst3Plugin for Ott {
    const VST3_CLASS_ID: [u8; 16] = *b"SephonixSPSOTT01";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}
nih_export_clap!(Ott);
nih_export_vst3!(Ott);
