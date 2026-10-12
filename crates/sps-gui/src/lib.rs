//! Shared effect controls and disk-preset browser; all work runs on the UI thread.
use nih_plug_egui::egui::*;
use sps_dsp::params::Spec;
pub const WIDTH: u32 = 980;
pub const HEIGHT: u32 = 760;
#[derive(Default)]
pub struct UiState {
    name: String,
    names: Vec<String>,
    selected: String,
    status: String,
    initialized: bool,
    mix_lock: bool,
    factory: usize,
}
pub fn configure(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = Visuals::dark();
    style.visuals.panel_fill = Color32::from_rgb(18, 23, 30);
    style.visuals.selection.bg_fill = Color32::from_rgb(45, 99, 110);
    style.spacing.item_spacing = vec2(12.0, 8.0);
    style.spacing.slider_width = 175.0;
    ctx.set_style(style);
}
#[allow(clippy::too_many_arguments)]
pub fn draw<const N: usize>(
    ctx: &Context,
    state: &mut UiState,
    title: &str,
    identity: &str,
    specs: &[Spec; N],
    values: &[f32; N],
    meters: &[f32; 2],
    choices: impl Fn(usize) -> &'static [&'static str],
    factory_names: &[&str],
    factory: impl Fn(usize) -> [f32; N],
    mut change: impl FnMut(usize, f32),
) {
    let directory = sps_presets::directory(title);
    if !state.initialized {
        state.initialized = true;
        match &directory {
            Ok(d) => match sps_presets::list(d) {
                Ok(n) => state.names = n,
                Err(e) => state.status = e.to_string(),
            },
            Err(e) => state.status = e.to_string(),
        }
    }
    let mut v = *values;
    CentralPanel::default()
        .frame(
            Frame::new()
                .fill(Color32::from_rgb(18, 23, 30))
                .inner_margin(22.0),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("SPS")
                        .color(Color32::from_rgb(113, 223, 205))
                        .strong()
                        .size(22.0),
                );
                ui.label(RichText::new(title).strong().size(30.0));
                ui.add_space(20.0);
                ui.label(
                    RichText::new("SEPHONIX PLUGIN SUITE")
                        .small()
                        .color(Color32::GRAY),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut on = v[2] >= 0.5;
                    if ui.toggle_value(&mut on, "Bypass").changed() {
                        v[2] = f32::from(on);
                    }
                    for (label, db) in [("OUT", meters[1]), ("IN", meters[0])] {
                        ui.label(format!("{label} {db:.1} dB"));
                    }
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                let old = state.factory;
                ComboBox::from_id_salt("factory")
                    .selected_text(factory_names[state.factory])
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for (i, name) in factory_names.iter().enumerate() {
                            ui.selectable_value(&mut state.factory, i, *name);
                        }
                    });
                if ui.button("Load factory").clicked() || old != state.factory {
                    v = factory(state.factory);
                    if state.mix_lock {
                        v[0] = values[0];
                    }
                }
                ui.checkbox(&mut state.mix_lock, "Lock Mix");
                ComboBox::from_id_salt("user")
                    .selected_text(if state.selected.is_empty() {
                        "User presets"
                    } else {
                        &state.selected
                    })
                    .width(160.0)
                    .show_ui(ui, |ui| {
                        for name in &state.names {
                            ui.selectable_value(&mut state.selected, name.clone(), name);
                        }
                    });
                if ui.button("Load").clicked() {
                    if let Ok(d) = &directory {
                        match sps_presets::load(d, &state.selected, identity, specs) {
                            Ok(p) => {
                                v.copy_from_slice(&p);
                                if state.mix_lock {
                                    v[0] = values[0];
                                }
                                state.status = format!("Loaded {}", state.selected);
                            }
                            Err(e) => state.status = e.to_string(),
                        }
                    }
                }
                if ui.button("Refresh").clicked() {
                    state.initialized = false;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Save as");
                ui.add(
                    TextEdit::singleline(&mut state.name)
                        .desired_width(230.0)
                        .hint_text("New preset name"),
                );
                if ui.button("Save new preset").clicked() {
                    if let Ok(d) = &directory {
                        match sps_presets::save(d, &state.name, identity, specs, &v) {
                            Ok(()) => {
                                state.selected = state.name.clone();
                                state.status = format!("Saved {}", state.name);
                                state.initialized = false;
                            }
                            Err(e) => state.status = e.to_string(),
                        }
                    }
                }
                ui.label(RichText::new(&state.status).small());
            });
            ui.separator();
            ScrollArea::vertical().max_height(550.0).show(ui, |ui| {
                let per_column = (N - 1).div_ceil(3);
                ui.columns(3, |columns| {
                    for (column, ui) in columns.iter_mut().enumerate() {
                        for slot in column * per_column..((column + 1) * per_column).min(N - 1) {
                            let i = if slot < 2 { slot } else { slot + 1 };
                            let s = specs[i];
                            ui.label(
                                RichText::new(s.name)
                                    .strong()
                                    .color(Color32::from_rgb(151, 209, 215)),
                            );
                            let labels = choices(i);
                            if !labels.is_empty() {
                                ComboBox::from_id_salt(i)
                                    .selected_text(
                                        labels[(v[i].round() as usize).min(labels.len() - 1)],
                                    )
                                    .width(220.0)
                                    .show_ui(ui, |ui| {
                                        for (n, label) in labels.iter().enumerate() {
                                            ui.selectable_value(&mut v[i], n as f32, *label);
                                        }
                                    });
                            } else if s.discrete {
                                let mut on = v[i] >= 0.5;
                                if ui.checkbox(&mut on, "Enabled").changed() {
                                    v[i] = f32::from(on);
                                }
                            } else {
                                ui.add(
                                    Slider::new(&mut v[i], s.min..=s.max)
                                        .logarithmic(s.logarithmic)
                                        .suffix(format!(" {}", s.unit))
                                        .clamping(SliderClamping::Always),
                                );
                            }
                            ui.add_space(2.0);
                        }
                    }
                });
            });
            ui.separator();
            if let Ok(d) = &directory {
                ui.label(
                    RichText::new(d.display().to_string())
                        .small()
                        .color(Color32::GRAY),
                );
            }
            ui.label(
                RichText::new(
                    "Double-click slider values to type · presets are shared by CLAP, VST3 and LV2",
                )
                .small(),
            );
        });
    for i in 0..N {
        v[i] = specs[i].sanitize(v[i]);
        if v[i] != values[i] {
            change(i, v[i]);
        }
    }
    // Immediate repaint avoids the pinned baseview backend postponing deadlines.
    ctx.request_repaint();
}
