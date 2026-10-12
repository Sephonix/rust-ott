// The pinned upstream VST3 bindings emit a deprecated expression semicolon.
#![allow(semicolon_in_expressions_from_non_local_macros)]
//! CLAP and VST3 wrappers. All audio work lives in sps-delay-dsp.
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, EguiState};
use sps_delay_dsp::{params::*, Engine, MeterBus};
use std::sync::Arc;

pub struct DelayParams {
    // Never resized after construction: parameter pointers stay valid for the Arc's lifetime.
    values: [FloatParam; PARAM_COUNT],
    editor_state: Arc<EguiState>,
    bypass: BoolParam,
}
impl Default for DelayParams {
    fn default() -> Self {
        Self {
            values: std::array::from_fn(|i| {
                let spec = SPECS[i];
                let name = spec.name;
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
                let labels = choices(i);
                let param = FloatParam::new(name, spec.default, range)
                    .with_unit(spec.unit)
                    .with_value_to_string(Arc::new(move |v| {
                        if labels.is_empty() {
                            format!("{v:.2}")
                        } else {
                            labels[(v.round() as usize).min(labels.len() - 1)].to_owned()
                        }
                    }))
                    .with_string_to_value(Arc::new(move |text| {
                        labels
                            .iter()
                            .position(|s| s.eq_ignore_ascii_case(text.trim()))
                            .map(|i| i as f32)
                            .or_else(|| text.trim().trim_end_matches(spec.unit).trim().parse().ok())
                    }));
                if spec.discrete {
                    param.with_step_size(1.0)
                } else {
                    param
                }
            }),
            editor_state: EguiState::from_size(sps_delay_gui::WIDTH, sps_delay_gui::HEIGHT),
            bypass: BoolParam::new("Bypass", false).make_bypass(),
        }
    }
}
// SAFETY: pointers refer to a fixed array inside the Params Arc held by NIH-plug.
unsafe impl Params for DelayParams {
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
                    String::new(),
                )
            })
            .collect()
    }
}
impl DelayParams {
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

pub struct Delay {
    params: Arc<DelayParams>,
    meters: Arc<MeterBus>,
    engine: Engine,
}
impl Default for Delay {
    fn default() -> Self {
        Self {
            params: Arc::new(DelayParams::default()),
            meters: Arc::new(MeterBus::default()),
            engine: Engine::new(48000.0),
        }
    }
}
impl Plugin for Delay {
    const NAME: &'static str = "SPS Delay";
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
            sps_delay_gui::UiState::default(),
            |ctx, _| sps_delay_gui::configure(ctx),
            move |ctx, setter, state| {
                let values = params.snapshot();
                sps_delay_gui::draw(ctx, state, &values, &meters.snapshot(), |i, value| {
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
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.engine
            .set_tempo(context.transport().tempo.unwrap_or(120.0) as f32);
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
        ProcessStatus::Tail(u32::MAX)
    }
}
impl ClapPlugin for Delay {
    const CLAP_ID: &'static str = "com.sephonix.sps-delay";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Stereo multi-style colored, diffused and shifted delay");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Delay,
        ClapFeature::Stereo,
        ClapFeature::Mono,
    ];
}
impl Vst3Plugin for Delay {
    const VST3_CLASS_ID: [u8; 16] = *b"SephonixDelay001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Delay];
}
nih_export_clap!(Delay);
nih_export_vst3!(Delay);
