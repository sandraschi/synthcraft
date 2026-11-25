//! Hardware-styled egui interface for the `SynthCraft.clap` instrument plug-in.
//!
//! Pure safe Rust: reads parameter values and meters lock-free from [`GuiBridge`] and pushes
//! parameter/note gestures back into the bridge so the audio thread emits CLAP automation events.

use crate::bridge::{GuiBridge, UiEvent};
use egui::{Color32, CornerRadius, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2};
use std::f32::consts::PI;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use synthcraft_core::params::{ParamDef, PARAMS};
use synthcraft_core::{FilterMode, SynthEngine, SynthPatch, WaveformKind};

/// Persistent UI interaction state for one open CLAP editor instance.
#[derive(Default)]
pub(crate) struct ClapUiState {
    active_drag_param: Option<u32>,
    held_note: Option<u8>,
    available_banks: Vec<PathBuf>,
    banks_scanned: bool,
    factory_patches: Vec<SynthPatch>,
}

const BG_PANEL: Color32 = Color32::from_rgb(9, 14, 26);
const BG_MODULE: Color32 = Color32::from_rgb(19, 27, 46);
const BORDER_MODULE: Color32 = Color32::from_rgb(56, 78, 118);
const KNOB_OUTER: Color32 = Color32::from_rgb(28, 36, 52);
const KNOB_CAP: Color32 = Color32::from_rgb(48, 60, 82);
const KNOB_PTR: Color32 = Color32::from_rgb(248, 250, 252);
const ARC_TRACK: Color32 = Color32::from_rgb(30, 41, 59);
const EMERALD: Color32 = Color32::from_rgb(34, 197, 94);
const CYAN: Color32 = Color32::from_rgb(56, 189, 248);
const AMBER: Color32 = Color32::from_rgb(249, 115, 22);
const TEXT_PRI: Color32 = Color32::from_rgb(241, 245, 249);
const TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184);

impl ClapUiState {
    fn scan_banks_if_needed(&mut self) {
        if self.factory_patches.is_empty() {
            self.factory_patches = SynthEngine::factory_seed().patches;
        }
        if self.banks_scanned {
            return;
        }
        self.banks_scanned = true;
        self.refresh_banks();
    }

    fn apply_factory_patch(&self, bridge: &GuiBridge, patch: &SynthPatch) {
        if let Ok(mut eng) = bridge.engine.lock() {
            eng.replace_patch(patch.clone());
            bridge.sync_values(&eng);
        }
    }

    fn refresh_banks(&mut self) {
        self.available_banks.clear();
        let home = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from);
        let Some(root) = home.map(|h| h.join("Documents").join("SynthCraft").join("Banks")) else {
            return;
        };
        collect_bank_files(&root, 2, &mut self.available_banks);
        self.available_banks.sort();
    }

    pub(crate) fn draw(&mut self, ui: &mut egui::Ui, bridge: &GuiBridge) {
        self.scan_banks_if_needed();

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BG_PANEL;
        visuals.window_fill = BG_MODULE;
        ui.ctx().set_visuals(visuals);

        egui::Panel::top("clap_header")
            .frame(egui::Frame::new().fill(BG_MODULE).inner_margin(8.0))
            .show(ui, |ui| {
                self.draw_header(ui, bridge);
            });

        egui::Panel::bottom("clap_keyboard")
            .frame(egui::Frame::new().fill(BG_MODULE).inner_margin(6.0))
            .show(ui, |ui| {
                self.draw_keyboard(ui, bridge);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG_PANEL).inner_margin(8.0))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.draw_modules(ui, bridge);
                });
            });
    }

    fn draw_header(&mut self, ui: &mut egui::Ui, bridge: &GuiBridge) {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("SYNTHCRAFT")
                    .strong()
                    .size(16.0)
                    .color(EMERALD)
                    .monospace(),
            );
            ui.label(
                RichText::new("CLAP 1.2.2")
                    .size(11.0)
                    .color(CYAN)
                    .monospace(),
            );
            ui.separator();

            // Factory patch selector (< [Patch] >)
            if !self.factory_patches.is_empty() {
                let (cur_id, cur_name) = bridge
                    .engine
                    .try_lock()
                    .map(|e| (e.patch().id.clone(), e.patch().name.clone()))
                    .unwrap_or_else(|_| (String::new(), String::from("Patch")));
                let cur_idx = self
                    .factory_patches
                    .iter()
                    .position(|p| p.id == cur_id)
                    .unwrap_or(0);
                let count = self.factory_patches.len();
                let mut pick: Option<usize> = None;
                if ui
                    .small_button("<")
                    .on_hover_text("Previous patch")
                    .clicked()
                {
                    pick = Some((cur_idx + count - 1) % count);
                }
                egui::ComboBox::from_label("Patch")
                    .width(170.0)
                    .selected_text(cur_name)
                    .show_ui(ui, |ui| {
                        for (i, p) in self.factory_patches.iter().enumerate() {
                            let label = format!("{} [{}]", p.name, p.category);
                            if ui.selectable_label(p.id == cur_id, label).clicked() {
                                pick = Some(i);
                            }
                        }
                    });
                if ui.small_button(">").on_hover_text("Next patch").clicked() {
                    pick = Some((cur_idx + 1) % count);
                }
                if let Some(i) = pick {
                    if let Some(patch) = self.factory_patches.get(i).cloned() {
                        self.apply_factory_patch(bridge, &patch);
                    }
                }
                ui.separator();
            }

            // Live voice count & stereo peak meter
            let voices = bridge.active_voices.load(Ordering::Relaxed);
            let pl = f32::from_bits(bridge.peak_l.load(Ordering::Relaxed)).clamp(0.0, 1.0);
            let pr = f32::from_bits(bridge.peak_r.load(Ordering::Relaxed)).clamp(0.0, 1.0);
            ui.label(
                RichText::new(format!("VOICES: {voices:02}"))
                    .monospace()
                    .size(11.0)
                    .color(if voices > 0 { EMERALD } else { TEXT_MUTED }),
            );
            draw_stereo_meter(ui, pl, pr);
            draw_mini_scope(ui, &bridge.read_scope());

            ui.separator();

            // Waveform & Filter mode quick selectors (main thread only)
            if let Ok(mut eng) = bridge.engine.try_lock() {
                let p = eng.patch_mut();
                egui::ComboBox::from_id_salt("clap_vco1")
                    .width(92.0)
                    .selected_text(format!("VCO1: {}", p.vco1_wave.as_str()))
                    .show_ui(ui, |ui| {
                        for w in all_waveforms() {
                            ui.selectable_value(&mut p.vco1_wave, w, w.as_str());
                        }
                    });
                egui::ComboBox::from_id_salt("clap_vcf")
                    .width(102.0)
                    .selected_text(format!("VCF: {}", p.filter_mode.as_str()))
                    .show_ui(ui, |ui| {
                        for f in all_filters() {
                            ui.selectable_value(&mut p.filter_mode, f, f.as_str());
                        }
                    });
            }

            // Sample bank dropdown (scans Documents/SynthCraft/Banks)
            let bank_label = bridge
                .bank_status
                .lock()
                .map(|s| s.clone())
                .unwrap_or_default();
            egui::ComboBox::from_id_salt("clap_bank")
                .width(140.0)
                .selected_text("Sample Bank")
                .show_ui(ui, |ui| {
                    if ui.button("Refresh Banks Folder").clicked() {
                        self.refresh_banks();
                    }
                    if ui.button("Clear Sample Bank").clicked() {
                        if let Ok(mut eng) = bridge.engine.lock() {
                            eng.clear_bank();
                        }
                        if let Ok(mut st) = bridge.bank_status.lock() {
                            *st = "No sample bank loaded".into();
                        }
                    }
                    ui.separator();
                    if self.available_banks.is_empty() {
                        ui.label(
                            RichText::new("No banks in Documents/SynthCraft/Banks")
                                .small()
                                .color(TEXT_MUTED),
                        );
                    } else {
                        for path in self.available_banks.clone() {
                            let label = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("bank")
                                .to_string();
                            if ui.button(&label).clicked() {
                                let spec = path.to_string_lossy().into_owned();
                                let res = bridge
                                    .engine
                                    .lock()
                                    .map_err(|_| "Engine busy".to_string())
                                    .and_then(|mut eng| {
                                        eng.load_bank(&spec).map_err(|e| e.to_string())
                                    });
                                if let Ok(mut st) = bridge.bank_status.lock() {
                                    *st = match res {
                                        Ok(()) => format!("Loaded: {label}"),
                                        Err(e) => format!("Err: {e}"),
                                    };
                                }
                            }
                        }
                    }
                });
            ui.label(RichText::new(bank_label).small().color(TEXT_MUTED));
        });
    }

    fn draw_modules(&mut self, ui: &mut egui::Ui, bridge: &GuiBridge) {
        // Group all 25 CLAP parameters by hardware module section.
        let sections: [(&str, &[usize]); 5] = [
            ("VCO / FM / MORPH", &[0, 1, 2, 14, 16]),
            ("VCF LADDER & DRIVE", &[3, 4, 5, 12]),
            ("ADSR ENVELOPE & LFO", &[6, 7, 8, 10, 9, 11]),
            ("PCM / SAMPLER / MASTER", &[15, 17, 13]),
            ("STEREO / FX / AMBIENT", &[22, 18, 19, 20, 21, 23, 24]),
        ];

        ui.horizontal_top(|ui| {
            let col_w = ((ui.available_width() - 16.0) / 2.0).max(320.0);
            ui.vertical(|ui| {
                ui.set_width(col_w);
                self.draw_section_card(ui, bridge, sections[0].0, sections[0].1);
                ui.add_space(6.0);
                self.draw_section_card(ui, bridge, sections[2].0, sections[2].1);
                ui.add_space(6.0);
                self.draw_section_card(ui, bridge, sections[3].0, sections[3].1);
            });
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.set_width(col_w);
                self.draw_section_card(ui, bridge, sections[1].0, sections[1].1);
                ui.add_space(6.0);
                self.draw_section_card(ui, bridge, sections[4].0, sections[4].1);
            });
        });
    }

    fn draw_section_card(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &GuiBridge,
        title: &str,
        indices: &[usize],
    ) {
        egui::Frame::new()
            .fill(BG_MODULE)
            .stroke(Stroke::new(1.0, BORDER_MODULE))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(title)
                        .strong()
                        .size(11.0)
                        .monospace()
                        .color(CYAN),
                );
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    for &idx in indices {
                        if let Some(def) = PARAMS.get(idx) {
                            self.draw_knob(ui, bridge, idx, def);
                        }
                    }
                });
            });
    }

    fn draw_knob(&mut self, ui: &mut egui::Ui, bridge: &GuiBridge, idx: usize, def: &ParamDef) {
        let cur = bridge.get_value_by_index(idx);
        let span = (def.max - def.min).max(1e-6);
        let is_log = def.id == 0x1004; // cutoff_hz feels much better on a log curve
        let norm = if is_log {
            let lo = def.min.max(1.0).ln();
            let hi = def.max.max(2.0).ln();
            ((cur.clamp(def.min, def.max).ln() - lo) / (hi - lo)).clamp(0.0, 1.0)
        } else {
            ((cur - def.min) / span).clamp(0.0, 1.0)
        };

        let (rect, resp) = ui.allocate_exact_size(Vec2::new(72.0, 68.0), Sense::click_and_drag());
        let center = Pos2::new(rect.center().x, rect.top() + 22.0);
        let painter = ui.painter_at(rect);

        if resp.drag_started() {
            self.active_drag_param = Some(def.id);
            bridge.push_ui_event(UiEvent::ParamGestureBegin(def.id));
        }

        let mut next_norm = norm;
        let mut changed = false;
        if resp.dragged() {
            let fine = ui.input(|i| i.modifiers.shift);
            let speed = if fine { 0.0015 } else { 0.0055 };
            next_norm = (norm - resp.drag_delta().y * speed).clamp(0.0, 1.0);
            changed = (next_norm - norm).abs() > 1e-6;
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.1 {
                next_norm = (norm + scroll * 0.0015).clamp(0.0, 1.0);
                changed = true;
            }
        }
        if resp.double_clicked() {
            bridge.push_ui_event(UiEvent::ParamGestureBegin(def.id));
            bridge.push_ui_event(UiEvent::ParamValue(def.id, def.default));
            bridge.push_ui_event(UiEvent::ParamGestureEnd(def.id));
        } else if changed {
            let next_val = if is_log {
                let lo = def.min.max(1.0).ln();
                let hi = def.max.max(2.0).ln();
                (lo + next_norm * (hi - lo)).exp().clamp(def.min, def.max)
            } else {
                (def.min + next_norm * span).clamp(def.min, def.max)
            };
            bridge.push_ui_event(UiEvent::ParamValue(def.id, next_val));
        }

        if resp.drag_stopped() && self.active_drag_param == Some(def.id) {
            self.active_drag_param = None;
            bridge.push_ui_event(UiEvent::ParamGestureEnd(def.id));
        }

        let active = resp.hovered() || resp.dragged();
        painter.circle_filled(center, 17.0, KNOB_OUTER);
        painter.circle_stroke(
            center,
            17.0,
            Stroke::new(
                if active { 2.4 } else { 1.8 },
                if active { CYAN } else { ARC_TRACK },
            ),
        );
        painter.circle_filled(center, 12.5, KNOB_CAP);

        let start_ang = PI * 0.75;
        let sweep = PI * 1.5 * next_norm;
        let arc_pts: Vec<Pos2> = (0..=16)
            .map(|s| {
                let a = start_ang + (s as f32 / 16.0) * sweep;
                Pos2::new(center.x + a.cos() * 17.5, center.y + a.sin() * 17.5)
            })
            .collect();
        painter.add(egui::Shape::line(
            arc_pts,
            Stroke::new(2.5, if active { AMBER } else { EMERALD }),
        ));

        let ang = start_ang + sweep;
        let tip = Pos2::new(center.x + ang.cos() * 12.0, center.y + ang.sin() * 12.0);
        painter.line_segment([center, tip], Stroke::new(2.2, KNOB_PTR));

        let val_text = if cur.abs() >= 100.0 {
            format!("{cur:.0}")
        } else {
            format!("{cur:.2}")
        };
        painter.text(
            Pos2::new(rect.center().x, rect.top() + 47.0),
            egui::Align2::CENTER_CENTER,
            def.name,
            FontId::monospace(9.0),
            TEXT_PRI,
        );
        painter.text(
            Pos2::new(rect.center().x, rect.top() + 59.0),
            egui::Align2::CENTER_CENTER,
            val_text,
            FontId::monospace(8.5),
            TEXT_MUTED,
        );
    }

    fn draw_keyboard(&mut self, ui: &mut egui::Ui, bridge: &GuiBridge) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("AUDITION (C3-C5)")
                    .monospace()
                    .size(10.0)
                    .color(TEXT_MUTED),
            );
            let white_notes: [u8; 15] =
                [48, 50, 52, 53, 55, 57, 59, 60, 62, 64, 65, 67, 69, 71, 72];
            let avail = (ui.available_width() - 8.0).max(240.0);
            let key_w = (avail / white_notes.len() as f32).min(38.0);
            let (rect, _) = ui.allocate_exact_size(
                Vec2::new(key_w * white_notes.len() as f32, 34.0),
                Sense::hover(),
            );
            let painter = ui.painter_at(rect);

            let mut any_down = false;
            for (i, &note) in white_notes.iter().enumerate() {
                let krect = Rect::from_min_size(
                    Pos2::new(rect.left() + i as f32 * key_w, rect.top()),
                    Vec2::new(key_w - 2.0, 34.0),
                );
                let resp = ui.interact(
                    krect,
                    ui.scope_id().with(("clap_key", note)),
                    Sense::click_and_drag(),
                );
                let pressed = resp.is_pointer_button_down_on();
                if pressed {
                    any_down = true;
                    if self.held_note != Some(note) {
                        if let Some(prev) = self.held_note.replace(note) {
                            bridge.push_ui_event(UiEvent::NoteOff(prev));
                        }
                        bridge.push_ui_event(UiEvent::NoteOn(note, 0.85));
                    }
                }
                let fill = if self.held_note == Some(note) {
                    EMERALD
                } else if resp.hovered() {
                    Color32::from_rgb(203, 213, 225)
                } else {
                    Color32::from_rgb(226, 232, 240)
                };
                painter.rect_filled(krect, CornerRadius::same(3), fill);
                painter.rect_stroke(
                    krect,
                    CornerRadius::same(3),
                    Stroke::new(1.0, BG_PANEL),
                    StrokeKind::Inside,
                );
            }
            if !any_down {
                if let Some(prev) = self.held_note.take() {
                    bridge.push_ui_event(UiEvent::NoteOff(prev));
                }
            }
        });
    }
}

fn draw_stereo_meter(ui: &mut egui::Ui, pl: f32, pr: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(64.0, 16.0), Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 3.0, BG_PANEL);
    for (row, val) in [(0.0, pl), (8.5, pr)] {
        let bar = Rect::from_min_size(
            Pos2::new(rect.left() + 2.0, rect.top() + 2.0 + row),
            Vec2::new((rect.width() - 4.0) * val.clamp(0.0, 1.0), 5.5),
        );
        let col = if val > 0.92 {
            Color32::from_rgb(239, 68, 68)
        } else if val > 0.7 {
            AMBER
        } else {
            EMERALD
        };
        p.rect_filled(bar, 2.0, col);
    }
}

fn draw_mini_scope(ui: &mut egui::Ui, samples: &[f32]) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(110.0, 18.0), Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 3.0, BG_PANEL);
    p.rect_stroke(
        rect,
        3.0,
        Stroke::new(1.0, BORDER_MODULE),
        StrokeKind::Inside,
    );
    if samples.len() < 2 {
        return;
    }
    let mid = rect.center().y;
    let amp = rect.height() * 0.42;
    let pts: Vec<Pos2> = samples
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let x = rect.left() + (i as f32 / (samples.len() - 1) as f32) * rect.width();
            let y = mid - s.clamp(-1.0, 1.0) * amp;
            Pos2::new(x, y)
        })
        .collect();
    p.add(egui::Shape::line(pts, Stroke::new(1.4, EMERALD)));
}

fn all_waveforms() -> [WaveformKind; 10] {
    [
        WaveformKind::Sine,
        WaveformKind::Saw,
        WaveformKind::Square,
        WaveformKind::Triangle,
        WaveformKind::PulsePwm,
        WaveformKind::SuperSaw,
        WaveformKind::WhiteNoise,
        WaveformKind::SubOctave,
        WaveformKind::AtmoPad,
        WaveformKind::Wavetable,
    ]
}

fn all_filters() -> [FilterMode; 4] {
    [
        FilterMode::LowPass24dB,
        FilterMode::BandPass12dB,
        FilterMode::HighPass12dB,
        FilterMode::AcidResonant,
    ]
}

fn collect_bank_files(dir: &PathBuf, depth: usize, out: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_bank_files(&p, depth - 1, out);
        } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if matches!(ext.to_ascii_lowercase().as_str(), "sfz" | "sf2" | "wav") {
                out.push(p);
            }
        }
    }
}
