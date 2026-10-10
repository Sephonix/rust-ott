//! A shared, native egui editor: no webview, FFT, or audio-thread drawing.
use nih_plug_egui::egui::*;
use sps_dsp::{params::*, METER_COUNT};
use std::f32::consts::PI;

pub const WIDTH: u32 = 900;
pub const HEIGHT: u32 = 760;
const BG: Color32 = Color32::from_rgb(15, 18, 23);
const PANEL: Color32 = Color32::from_rgb(24, 28, 35);
const DIM: Color32 = Color32::from_rgb(142, 152, 167);
const TEXT: Color32 = Color32::from_rgb(232, 237, 241);
pub const COLORS: [Color32; 3] = [
    Color32::from_rgb(197, 241, 105),
    Color32::from_rgb(100, 212, 221),
    Color32::from_rgb(185, 151, 255),
];
const HISTORY: usize = 180;

pub struct UiState {
    history: [[f32; METER_COUNT]; HISTORY],
    cursor: usize,
    last_frame: f64,
    clip_until: f64,
}
impl Default for UiState {
    fn default() -> Self {
        Self {
            history: [[-90.0; METER_COUNT]; HISTORY],
            cursor: 0,
            last_frame: -1.0,
            clip_until: 0.0,
        }
    }
}

pub fn configure(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = Visuals::dark();
    style.visuals.panel_fill = BG;
    style.visuals.window_fill = PANEL;
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.selection.bg_fill = COLORS[0].gamma_multiply(0.25);
    style.visuals.selection.stroke = Stroke::new(1.0_f32, COLORS[0]);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(36, 42, 51);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(49, 57, 67);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(65, 76, 88);
    style.visuals.widgets.noninteractive.bg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(44, 51, 60));
    style.spacing.item_spacing = vec2(10.0, 5.0);
    style.spacing.slider_width = 126.0;
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(13.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(12.0));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(10.0));
    ctx.set_style(style);
}

pub fn draw(
    ctx: &Context,
    state: &mut UiState,
    values: &[f32; PARAM_COUNT],
    meters: &[f32; METER_COUNT],
    mut change: impl FnMut(usize, f32),
) {
    let now = ctx.input(|i| i.time);
    if now - state.last_frame >= 1.0 / 30.0 {
        state.history[state.cursor] = *meters;
        state.cursor = (state.cursor + 1) % HISTORY;
        state.last_frame = now;
    }
    if meters[1] >= 0.0 {
        state.clip_until = now + 1.0;
    }
    let mut v = *values;
    CentralPanel::default().frame(Frame::new().fill(BG).inner_margin(20.0)).show(ctx, |ui| {
        ui.horizontal(|ui| {
            Frame::new().fill(COLORS[0]).corner_radius(5.0).inner_margin(vec2(9.0, 5.0)).show(ui, |ui| {
                ui.label(RichText::new("SPS").strong().size(15.0).color(BG));
            });
            ui.label(RichText::new("OTT").size(29.0).strong());
            ui.add_space(5.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("SEPHONIX PLUGIN SUITE").size(10.0).color(DIM));
                ui.label(RichText::new("MULTIBAND UPWARD / DOWNWARD COMPRESSION").size(10.0).color(DIM));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("↺  Factory preset").on_hover_text("Restore the supplied aggressive starting preset, including Amount 100% and Output 0 dB.").clicked() { v = defaults(); }
                toggle(ui, &mut v[BYPASS], "BYPASS", Color32::from_rgb(242, 177, 99));
            });
        });
        ui.add_space(10.0);
        Frame::new().fill(PANEL).corner_radius(8.0).inner_margin(14.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                knob(ui, AMOUNT, &mut v[AMOUNT], "AMOUNT", COLORS[0], 84.0,
                    "Compression depth. Scales upward/downward dynamics and band input/output gains. 0% is unity gain through the crossover; 100% is full OTT.");
                ui.add_space(24.0);
                knob(ui, OUTPUT, &mut v[OUTPUT], "OUTPUT", TEXT, 84.0, "Final output gain after the dry/wet mix. Use it to match loudness and keep the output below 0 dBFS.");
                ui.add_space(24.0);
                knob(ui, MIX, &mut v[MIX], "MIX", COLORS[1], 66.0, "Parallel blend with the phase-aligned dry signal. 0% is dry; 100% is processed.");
                ui.add_space(18.0);
                ui.separator();
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("LEVELS").color(DIM).size(10.0));
                    level_row(ui, "IN", meters[0], 285.0, TEXT);
                    level_row(ui, "OUT", meters[1], 285.0, COLORS[0]);
                    ui.label(RichText::new(if now < state.clip_until { "OVER 0 dBFS · lower Output" } else { "ZERO LATENCY  /  STEREO LINKED" })
                        .color(if now < state.clip_until { Color32::from_rgb(255, 140, 104) } else { DIM }).size(10.0));
                    ui.label(RichText::new("AGGRESSIVE BY DESIGN").color(DIM).size(10.0));
                });
            });
        });
        ui.add_space(7.0);
        activity(ui, state, meters);
        ui.add_space(7.0);
        ui.columns(3, |columns| {
            for (band, ui) in columns.iter_mut().enumerate() {
                let color = COLORS[band];
                let n = band_index(band, 0);
                Frame::new().fill(PANEL).corner_radius(8.0).inner_margin(12.0).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(BAND_NAMES[band].to_uppercase()).strong().size(17.0).color(color));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            toggle(ui, &mut v[n + SOLO], "S", color);
                            toggle(ui, &mut v[n + MUTE], "M", color);
                        });
                    });
                    ui.label(RichText::new(match band {
                        0 => format!("20 — {:.1} Hz", v[LOW_CROSS]),
                        1 => format!("{:.1} Hz — {:.0} Hz", v[LOW_CROSS], v[HIGH_CROSS]),
                        _ => format!("{:.0} Hz — 20 kHz", v[HIGH_CROSS]),
                    }).size(10.0).color(DIM));
                    gain_meter(ui, meters[5 + band], color);
                    ui.add_space(3.0);
                    control(ui, n + INPUT_GAIN, &mut v[n + INPUT_GAIN], color);
                    section(ui, "DOWNWARD", color);
                    control(ui, n + DOWN_THRESHOLD, &mut v[n + DOWN_THRESHOLD], color);
                    control(ui, n + DOWN_RATIO, &mut v[n + DOWN_RATIO], color);
                    section(ui, "UPWARD", color);
                    control(ui, n + UP_THRESHOLD, &mut v[n + UP_THRESHOLD], color);
                    control(ui, n + UP_RATIO, &mut v[n + UP_RATIO], color);
                    section(ui, "TIMING / GAIN", color);
                    control(ui, n + ATTACK, &mut v[n + ATTACK], color);
                    control(ui, n + RELEASE, &mut v[n + RELEASE], color);
                    control(ui, n + BAND_OUTPUT, &mut v[n + BAND_OUTPUT], color);
                });
            }
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("CROSSOVERS").size(10.0).color(DIM));
            ui.label("L/M");
            numeric(ui, LOW_CROSS, &mut v[LOW_CROSS]);
            ui.label("M/H");
            numeric(ui, HIGH_CROSS, &mut v[HIGH_CROSS]);
            ui.add_space(10.0);
            if ui.selectable_label(v[DETECTION] < 0.5, "RMS").clicked() { v[DETECTION] = 0.0; }
            if ui.selectable_label(v[DETECTION] >= 0.5, "Peak").clicked() { v[DETECTION] = 1.0; }
            toggle(ui, &mut v[KNEE], "SOFT KNEE", COLORS[0]);
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Drag to adjust · double-click to reset · type exact values").size(10.0).color(DIM));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(format!("SEPHONIX  /  v{}", env!("CARGO_PKG_VERSION"))).size(10.0).color(DIM));
            });
        });
    });
    for i in 0..PARAM_COUNT {
        v[i] = SPECS[i].sanitize(v[i]);
        if v[i] != values[i] {
            change(i, v[i]);
        }
    }
    // egui-baseview repeatedly postpones delayed repaint deadlines at its frame tick.
    // Request an immediate paint so live meters also update without mouse activity.
    ctx.request_repaint();
}

fn section(ui: &mut Ui, label: &str, color: Color32) {
    ui.add_space(1.0);
    ui.label(
        RichText::new(label)
            .size(9.0)
            .strong()
            .color(color.gamma_multiply(0.8)),
    );
}
fn toggle(ui: &mut Ui, value: &mut f32, label: &str, color: Color32) {
    let on = *value >= 0.5;
    let button =
        Button::new(
            RichText::new(label)
                .strong()
                .size(10.0)
                .color(if on { color } else { DIM }),
        )
        .fill(if on {
            color.gamma_multiply(0.15)
        } else {
            Color32::from_rgb(32, 38, 46)
        })
        .stroke(Stroke::new(
            1.0_f32,
            if on {
                color.gamma_multiply(0.6)
            } else {
                Color32::from_rgb(45, 53, 63)
            },
        ));
    if ui.add(button).clicked() {
        *value = if on { 0.0 } else { 1.0 };
    }
}
fn numeric(ui: &mut Ui, index: usize, value: &mut f32) {
    let s = SPECS[index];
    let speed = if s.logarithmic {
        (*value * 0.01).max(0.01)
    } else {
        0.1
    };
    let r = ui.add(
        DragValue::new(value)
            .range(s.min..=s.max)
            .speed(speed)
            .max_decimals(2)
            .suffix(format!(" {}", s.unit)),
    );
    if r.double_clicked() {
        *value = s.default;
    }
}
fn control(ui: &mut Ui, index: usize, value: &mut f32, color: Color32) {
    let s = SPECS[index];
    let field = (index - GLOBAL_COUNT) % BAND_STRIDE;
    let label = match field {
        DOWN_THRESHOLD | UP_THRESHOLD => "Threshold",
        DOWN_RATIO | UP_RATIO => "Ratio",
        BAND_OUTPUT => "Output",
        _ => s.name,
    };
    ui.horizontal(|ui| {
        ui.add_sized(
            [61.0, 18.0],
            Label::new(RichText::new(label).size(11.0).color(DIM)),
        );
        let width = (ui.available_width() - 76.0).max(35.0);
        let mut normalized = s.normalize(*value);
        let (rect, response) = ui.allocate_exact_size(vec2(width, 18.0), Sense::click_and_drag());
        if let Some(pos) = response.interact_pointer_pos() {
            normalized = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            *value = s.denormalize(normalized);
        }
        if response.double_clicked() {
            *value = s.default;
            normalized = s.normalize(*value);
        }
        let line = Rect::from_center_size(rect.center(), vec2(rect.width(), 3.0));
        ui.painter()
            .rect_filled(line, 2.0, Color32::from_rgb(48, 56, 65));
        ui.painter().rect_filled(
            Rect::from_min_max(
                line.min,
                pos2(line.left() + normalized * line.width(), line.bottom()),
            ),
            2.0,
            color.gamma_multiply(0.8),
        );
        ui.painter().circle_filled(
            pos2(rect.left() + normalized * rect.width(), rect.center().y),
            4.0,
            color,
        );
        let speed = if s.logarithmic {
            (*value * 0.015).max(0.01)
        } else {
            0.1
        };
        ui.add(
            DragValue::new(value)
                .range(s.min..=s.max)
                .speed(speed)
                .max_decimals(2)
                .custom_formatter(move |v, _| {
                    if field == DOWN_RATIO && v >= 999.0 {
                        "∞:1".to_owned()
                    } else {
                        if field == UP_RATIO || field == DOWN_RATIO {
                            format!("{v:.2}:1")
                        } else {
                            format!("{v:.1} {}", s.unit)
                        }
                    }
                })
                .custom_parser(|s| {
                    if s.contains('∞') || s.to_lowercase().contains("inf") {
                        Some(1000.0)
                    } else {
                        s.split_whitespace()
                            .next()?
                            .trim_end_matches(":1")
                            .parse()
                            .ok()
                    }
                }),
        );
    });
}
fn knob(
    ui: &mut Ui,
    index: usize,
    value: &mut f32,
    label: &str,
    color: Color32,
    diameter: f32,
    help: &str,
) {
    ui.vertical(|ui| {
        ui.set_width(diameter + 20.0);
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.label(RichText::new(label).size(11.0).strong().color(DIM));
            let (rect, response) =
                ui.allocate_exact_size(vec2(diameter, diameter), Sense::click_and_drag());
            let spec = SPECS[index];
            let mut t = spec.normalize(*value);
            if response.dragged() {
                t = (t - response.drag_delta().y / 180.0 + response.drag_delta().x / 360.0)
                    .clamp(0.0, 1.0);
                *value = spec.denormalize(t);
            }
            if response.double_clicked() {
                *value = spec.default;
                t = spec.normalize(*value);
            }
            response.on_hover_text(help);
            let painter = ui.painter();
            let center = rect.center();
            let radius = diameter * 0.41;
            let start = PI * 0.75;
            let sweep = PI * 1.5;
            let arc = |progress: f32| {
                (0..=48)
                    .map(|i| {
                        let a = start + sweep * progress * i as f32 / 48.0;
                        center + vec2(a.cos(), a.sin()) * radius
                    })
                    .collect::<Vec<_>>()
            };
            painter.add(Shape::line(
                arc(1.0),
                Stroke::new(4.0_f32, Color32::from_rgb(50, 58, 67)),
            ));
            painter.add(Shape::line(arc(t), Stroke::new(4.0_f32, color)));
            painter.circle_filled(center, radius - 8.0, Color32::from_rgb(33, 39, 47));
            let a = start + sweep * t;
            painter.line_segment(
                [
                    center + vec2(a.cos(), a.sin()) * (radius - 18.0),
                    center + vec2(a.cos(), a.sin()) * (radius - 10.0),
                ],
                Stroke::new(3.0_f32, color),
            );
            numeric(ui, index, value);
        });
    });
}
fn level_row(ui: &mut Ui, label: &str, db: f32, width: f32, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(10.0).color(DIM));
        let (rect, _) = ui.allocate_exact_size(vec2(width, 9.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, 2.0, Color32::from_rgb(41, 48, 57));
        let t = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
        ui.painter().rect_filled(
            Rect::from_min_max(rect.min, pos2(rect.left() + t * width, rect.bottom())),
            2.0,
            if db >= 0.0 {
                Color32::from_rgb(255, 140, 104)
            } else {
                color
            },
        );
        ui.label(
            RichText::new(if db <= -119.0 {
                "−∞".to_owned()
            } else {
                format!("{db:.1} dB")
            })
            .monospace()
            .size(10.0),
        );
    });
}
fn gain_meter(ui: &mut Ui, db: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 23.0), Sense::hover());
    let center = rect.center().x;
    ui.painter().rect_filled(rect, 4.0, BG);
    let x = center + (db / 36.0).clamp(-1.0, 1.0) * rect.width() * 0.5;
    ui.painter().rect_filled(
        Rect::from_min_max(
            pos2(x.min(center), rect.top() + 5.0),
            pos2(x.max(center), rect.bottom() - 5.0),
        ),
        2.0,
        color.gamma_multiply(0.45),
    );
    ui.painter().line_segment(
        [
            pos2(center, rect.top() + 4.0),
            pos2(center, rect.bottom() - 4.0),
        ],
        Stroke::new(1.0_f32, DIM),
    );
    ui.painter().text(
        rect.left_center() + vec2(5.0, 0.0),
        Align2::LEFT_CENTER,
        "−",
        FontId::proportional(12.0),
        DIM,
    );
    ui.painter().text(
        rect.right_center() - vec2(5.0, 0.0),
        Align2::RIGHT_CENTER,
        format!("{db:+.1} dB"),
        FontId::monospace(10.0),
        color,
    );
}
fn activity(ui: &mut Ui, state: &UiState, meters: &[f32; METER_COUNT]) {
    Frame::new()
        .fill(PANEL)
        .corner_radius(8.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("BAND ENERGY").size(9.0).color(DIM));
                ui.label(RichText::new("6 SECOND HISTORY").size(9.0).color(DIM));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    for i in (0..3).rev() {
                        ui.label(
                            RichText::new(format!("{}  {:.0} dB", BAND_NAMES[i], meters[2 + i]))
                                .size(9.0)
                                .color(COLORS[i]),
                        );
                    }
                });
            });
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 83.0), Sense::hover());
            let painter = ui.painter();
            for db in [-60.0, -30.0, 0.0] {
                let y = rect.bottom() - (db + 72.0) / 78.0 * rect.height();
                painter.line_segment(
                    [pos2(rect.left(), y), pos2(rect.right(), y)],
                    Stroke::new(1.0_f32, Color32::from_rgb(38, 44, 53)),
                );
                painter.text(
                    pos2(rect.left() + 3.0, y - 2.0),
                    Align2::LEFT_BOTTOM,
                    format!("{db:.0}"),
                    FontId::monospace(8.0),
                    DIM,
                );
            }
            for (b, color) in COLORS.iter().enumerate() {
                let points: Vec<Pos2> = (0..HISTORY)
                    .map(|i| {
                        let db = state.history[(state.cursor + i) % HISTORY][2 + b];
                        pos2(
                            rect.left() + i as f32 / (HISTORY - 1) as f32 * rect.width(),
                            rect.bottom() - ((db + 72.0) / 78.0).clamp(0.0, 1.0) * rect.height(),
                        )
                    })
                    .collect();
                painter.add(Shape::line(points, Stroke::new(1.6_f32, *color)));
            }
        });
}
