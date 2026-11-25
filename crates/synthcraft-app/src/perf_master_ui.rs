//! Workbench 14 -- Performance & Mastering Suite:
//! - Subpage 0: Polyphonic 1-Finger Chord Memory, Scale Lock Quantizer & Riff Generator
//! - Subpage 1: 16-Band Analog Vocoder & Robotic Talkbox Engine
//! - Subpage 2: 8-Scene Live Performance Crossfader & 4-Corner XY Morph Pad
//! - Subpage 3: Studio Mastering Chain (4-Band EQ, Sub-Bass Mono Maker, M/S Stereo Imager & Brickwall Limiter)

use crate::app::SynthCraftApp;
use crate::controls::{rotary_knob, vertical_fader};
use eframe::egui::{self, Color32, Pos2, RichText, Sense, Stroke, Vec2};
use synthcraft_core::{
    ChordMode, PerformanceScene, ScaleLock, SceneMorphBank, VocoderPhrase, SCENE_COUNT,
};

pub fn render_perf_master(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("14. Performance & Mastering: Chord Memory / 16-Band Vocoder / 8-Scene Morph / Mastering");
        for (idx, lbl) in [
            (0_usize, "Chord Memory & Scale Lock"),
            (1, "16-Band Analog Vocoder"),
            (2, "8-Scene Morph & XY Pad"),
            (3, "Studio Mastering Bus"),
        ] {
            if ui.selectable_label(app.subpage == idx, lbl).clicked() {
                app.subpage = idx;
            }
        }
    });
    ui.separator();

    match app.subpage {
        0 => render_chord_memory(app, ui),
        1 => render_vocoder(app, ui),
        2 => render_scene_morph(app, ui),
        _ => render_mastering_bus(app, ui),
    }
}

fn render_chord_memory(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let before = app.engine.active_patch().clone();
    let mut edited = before.clone();
    let note_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.strong(
                "1-Finger Polyphonic Chord Memory (Plays Full Chords from Any Key / CLAP / MIDI)",
            );
            ui.horizontal_wrapped(|ui| {
                for cm in ChordMode::ALL {
                    if ui
                        .selectable_label(edited.chord_mem.chord_mode == cm, cm.label())
                        .clicked()
                    {
                        edited.chord_mem.chord_mode = cm;
                    }
                }
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::Slider::new(&mut edited.chord_mem.inversion, 0..=3)
                        .text("Chord Inversion"),
                );
                ui.checkbox(
                    &mut edited.chord_mem.drop2_spread,
                    "Drop-2 Open Voicing Spread (-12 st on 2nd note)",
                );
            });
        });

        cols[1].group(|ui| {
            ui.strong("Scale Lock Quantizer & Intelligent Riff Generator");
            ui.horizontal_wrapped(|ui| {
                ui.label("Root:");
                for (i, n) in note_names.iter().enumerate() {
                    if ui
                        .selectable_label(edited.chord_mem.root_semitone as usize == i, *n)
                        .clicked()
                    {
                        edited.chord_mem.root_semitone = i as u8;
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                for sc in ScaleLock::ALL {
                    if ui
                        .selectable_label(edited.chord_mem.scale_lock == sc, sc.label())
                        .clicked()
                    {
                        edited.chord_mem.scale_lock = sc;
                    }
                }
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.strong("Generate Scale-Locked Riff -> Sequencer:");
                for (style_id, lbl) in [
                    ("acid_303", "Acid 303 Riff"),
                    ("berlin_hypnotic", "Berlin Hypnotic"),
                    ("house_chord_groove", "90s House Groove"),
                ] {
                    if ui.button(lbl).clicked() {
                        let cur_len = edited.steps.len().max(16);
                        edited.steps = edited.chord_mem.generate_riff(cur_len, 3, style_id);
                        app.status_line = format!(
                            "Generated {cur_len}-step '{lbl}' riff in {} {}",
                            note_names[(edited.chord_mem.root_semitone as usize) % 12],
                            edited.chord_mem.scale_lock.label()
                        );
                    }
                }
            });
        });
    });

    app.apply_patch_edit(&before, &edited);
}

fn render_vocoder(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    let before = app.engine.active_patch().clone();
    let mut edited = before.clone();

    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("16-Band Sennheiser/Roland-Style Analog Vocoder (110 Hz .. 7.6 kHz Filterbank)");
            if ui
                .selectable_label(
                    edited.vocoder.enabled,
                    if edited.vocoder.enabled {
                        "VOCODER: ON"
                    } else {
                        "VOCODER: OFF"
                    },
                )
                .clicked()
            {
                edited.vocoder.enabled = !edited.vocoder.enabled;
            }
        });
        ui.label(
            "Modulates your active synth/chord carrier through 16 log-spaced bandpass envelopes driven by the tempo-synced robotic phrase engine (or live DJFX vocal input from DJCraft!).",
        );
        ui.horizontal_wrapped(|ui| {
            ui.label("Robotic Phrase Modulator:");
            for ph in VocoderPhrase::ALL {
                if ui
                    .selectable_label(edited.vocoder.phrase == ph, ph.label())
                    .clicked()
                {
                    edited.vocoder.phrase = ph;
                }
            }
        });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            vertical_fader(ui, &pal, "VOC MIX", &mut edited.vocoder.mix, 0.0, 1.0, "%");
            rotary_knob(
                ui,
                &pal,
                "FORMANT",
                &mut edited.vocoder.formant_shift_st,
                -12.0,
                12.0,
                "st",
            );
            rotary_knob(
                ui,
                &pal,
                "ATTACK",
                &mut edited.vocoder.attack_ms,
                2.0,
                60.0,
                "ms",
            );
            rotary_knob(
                ui,
                &pal,
                "RELEASE",
                &mut edited.vocoder.release_ms,
                15.0,
                300.0,
                "ms",
            );
            rotary_knob(
                ui,
                &pal,
                "SIBILANCE",
                &mut edited.vocoder.sibilance_hf,
                0.0,
                1.0,
                "%",
            );
        });
    });

    app.apply_patch_edit(&before, &edited);
}

fn render_scene_morph(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    let before = app.engine.active_patch().clone();
    let mut edited = before.clone();
    edited.scenes.ensure_len();

    ui.columns(2, |cols| {
        // Left: 8 Scene Slots + DJ Crossfader
        cols[0].group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("8-Scene Snapshot Bank & DJ Crossfader (Octatrack Style)");
                if ui.small_button("Reset 8 Club Scenes").clicked() {
                    edited.scenes = SceneMorphBank::factory_club_scenes();
                }
            });
            ui.add_space(4.0);

            for idx in 0..SCENE_COUNT {
                let sname = edited.scenes.scenes[idx].name.clone();
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(edited.scenes.scene_a == idx, "A")
                        .clicked()
                    {
                        edited.scenes.scene_a = idx;
                        let snap = edited.scenes.blended_crossfade();
                        snap.apply_to(&mut edited);
                    }
                    if ui
                        .selectable_label(edited.scenes.scene_b == idx, "B")
                        .clicked()
                    {
                        edited.scenes.scene_b = idx;
                        let snap = edited.scenes.blended_crossfade();
                        snap.apply_to(&mut edited);
                    }
                    ui.label(&sname);
                    if ui.small_button("Capture").clicked() {
                        let captured = PerformanceScene::capture_from(&edited, &sname);
                        edited.scenes.scenes[idx] = captured;
                    }
                    if ui.small_button("Load").clicked() {
                        let snap = edited.scenes.scenes[idx].clone();
                        snap.apply_to(&mut edited);
                    }
                });
            }

            ui.separator();
            ui.strong(
                "Live Scene A <-> Scene B Crossfader (Morphs 11 Macro Parameters Simultaneously):",
            );
            if ui
                .add(
                    egui::Slider::new(&mut edited.scenes.crossfade, 0.0..=1.0)
                        .text("Crossfader A <-> B"),
                )
                .changed()
            {
                let snap = edited.scenes.blended_crossfade();
                snap.apply_to(&mut edited);
            }
        });

        // Right: 4-Corner Bilinear Scene XY Morph Pad (Scenes 1..4)
        cols[1].group(|ui| {
            ui.strong("4-Corner Bilinear Scene Morph XY Pad (Drag to Morph Scenes 1 / 2 / 3 / 4)");
            let avail_w = ui.available_width().max(260.0);
            let (rect, resp) =
                ui.allocate_exact_size(Vec2::new(avail_w, 230.0), Sense::click_and_drag());
            if (resp.clicked() || resp.dragged()) && resp.interact_pointer_pos().is_some() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    edited.scenes.xy_x =
                        ((pos.x - rect.min.x) / rect.width().max(1.0)).clamp(0.0, 1.0);
                    edited.scenes.xy_y =
                        ((pos.y - rect.min.y) / rect.height().max(1.0)).clamp(0.0, 1.0);
                    let snap = edited.scenes.blended_xy();
                    snap.apply_to(&mut edited);
                }
            }
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 6.0, pal.lcd_bg);
            painter.rect_stroke(
                rect,
                6.0,
                Stroke::new(1.4, pal.lcd_border),
                egui::StrokeKind::Inside,
            );
            // Corner labels
            let corners = [
                (
                    Pos2::new(rect.min.x + 8.0, rect.min.y + 12.0),
                    egui::Align2::LEFT_CENTER,
                    0,
                ),
                (
                    Pos2::new(rect.max.x - 8.0, rect.min.y + 12.0),
                    egui::Align2::RIGHT_CENTER,
                    1,
                ),
                (
                    Pos2::new(rect.min.x + 8.0, rect.max.y - 12.0),
                    egui::Align2::LEFT_CENTER,
                    2,
                ),
                (
                    Pos2::new(rect.max.x - 8.0, rect.max.y - 12.0),
                    egui::Align2::RIGHT_CENTER,
                    3,
                ),
            ];
            for (pos, align, idx) in corners {
                painter.text(
                    pos,
                    align,
                    &edited.scenes.scenes[idx].name,
                    egui::FontId::monospace(10.0),
                    pal.lcd_subtext,
                );
            }
            let px = rect.min.x + edited.scenes.xy_x * rect.width();
            let py = rect.min.y + edited.scenes.xy_y * rect.height();
            painter.circle_filled(Pos2::new(px, py), 10.0, pal.arc_primary);
            painter.circle_stroke(Pos2::new(px, py), 16.0, Stroke::new(1.5, pal.arc_secondary));
        });
    });

    app.apply_patch_edit(&before, &edited);
}

fn render_mastering_bus(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    let before = app.engine.active_patch().clone();
    let mut edited = before.clone();

    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("Studio Mastering Chain (4-Band EQ + Sub-Bass Mono Maker + M/S Imager + Brickwall Limiter)");
            let lbl = if edited.mastering.enabled {
                "MASTERING BUS: ACTIVE"
            } else {
                "MASTERING BUS: BYPASS"
            };
            let fill = if edited.mastering.enabled {
                pal.arc_primary
            } else {
                pal.module_border
            };
            if ui
                .add(egui::Button::new(RichText::new(lbl).strong().color(Color32::WHITE)).fill(fill))
                .clicked()
            {
                edited.mastering.enabled = !edited.mastering.enabled;
            }
        });
    });

    ui.add_space(6.0);
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.strong("4-Band Mastering Parametric EQ (-12 .. +12 dB)");
            ui.horizontal_wrapped(|ui| {
                vertical_fader(
                    ui,
                    &pal,
                    "LOW 80Hz",
                    &mut edited.mastering.eq_low_db,
                    -12.0,
                    12.0,
                    "dB",
                );
                vertical_fader(
                    ui,
                    &pal,
                    "LMID 350",
                    &mut edited.mastering.eq_low_mid_db,
                    -12.0,
                    12.0,
                    "dB",
                );
                vertical_fader(
                    ui,
                    &pal,
                    "HMID 2.8k",
                    &mut edited.mastering.eq_high_mid_db,
                    -12.0,
                    12.0,
                    "dB",
                );
                vertical_fader(
                    ui,
                    &pal,
                    "AIR 9.5k",
                    &mut edited.mastering.eq_high_db,
                    -12.0,
                    12.0,
                    "dB",
                );
            });
        });

        cols[1].group(|ui| {
            ui.strong("Mid/Side Stereo Imager, Club Sub-Bass Mono & True-Peak Brickwall Limiter");
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(
                    &mut edited.mastering.mono_bass_below_120hz,
                    "Sub-Bass Mono Maker (< 120 Hz -> Mono for Club PA / Vinyl)",
                );
                ui.checkbox(
                    &mut edited.mastering.limiter_enabled,
                    "Enable True-Peak Brickwall Limiter",
                );
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                rotary_knob(
                    ui,
                    &pal,
                    "M/S WIDTH",
                    &mut edited.mastering.stereo_width,
                    0.0,
                    2.0,
                    "x",
                );
                rotary_knob(
                    ui,
                    &pal,
                    "LIM DRIVE",
                    &mut edited.mastering.limiter_drive_db,
                    0.0,
                    12.0,
                    "dB",
                );
                rotary_knob(
                    ui,
                    &pal,
                    "CEILING",
                    &mut edited.mastering.limiter_ceiling_db,
                    -6.0,
                    -0.1,
                    "dB",
                );
            });
        });
    });

    app.apply_patch_edit(&before, &edited);
}
