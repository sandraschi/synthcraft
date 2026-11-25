//! Full 8-Voice TR-808 / TR-909 Analog & FM Drum Machine Workbench UI (up to 64 Steps / 4 Bars).

use crate::app::SynthCraftApp;
use crate::audio_io::AudioMsg;
use crate::controls::{rotary_knob, vertical_fader};
use eframe::egui::{self, Color32, RichText, Vec2};
use synthcraft_core::{DrumMachinePatch, DrumVoiceKind, DRUM_MAX_STEPS, DRUM_VOICE_COUNT};

/// Render the 8-Voice TR-808 / TR-909 Drum Machine workbench (up to 64 steps, 4 x 16-step bars).
pub fn render_drum_machine(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    ui.heading("TR-808 / TR-909 8-Voice Analog & FM Drum Machine (16 / 32 / 48 / 64 Steps)");
    ui.label(
        "Pure-Rust real-time percussion synth (Kick, Snare, Clap, Closed/Open Hat with choke, Tom, 808 Cowbell, Cymbal) locked to the master sequencer clock with swing and Euclidean fills.",
    );
    ui.separator();

    let playing_step = app.playing_step();
    let before = app.engine.active_patch().clone();
    let mut edited = before.clone();
    edited.drums.ensure_valid();

    let mut audition_voice: Option<(usize, bool)> = None;

    // 1. Top Control Strip: Power, Kit Presets, Step Length (16/32/48/64), Bar Page (A/B/C/D)
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            let on_lbl = if edited.drums.enabled {
                "DRUMS: ON"
            } else {
                "DRUMS: OFF"
            };
            let on_col = if edited.drums.enabled {
                pal.arc_primary
            } else {
                pal.module_border
            };
            if ui
                .add(
                    egui::Button::new(RichText::new(on_lbl).strong().color(Color32::WHITE))
                        .fill(on_col),
                )
                .clicked()
            {
                edited.drums.enabled = !edited.drums.enabled;
            }

            ui.separator();
            ui.strong("Length:");
            for len in [16_usize, 32, 48, 64] {
                if ui
                    .selectable_label(edited.drums.length == len, format!("{len} st"))
                    .clicked()
                {
                    edited.drums.length = len;
                    let max_bars = len / 16;
                    if app.drum_bar_page >= max_bars {
                        app.drum_bar_page = max_bars.saturating_sub(1);
                    }
                }
            }

            ui.separator();
            ui.strong("Bar Page:");
            let num_bars = (edited.drums.length / 16).clamp(1, 4);
            for b in 0..num_bars {
                let bar_letter = ["A", "B", "C", "D"][b];
                let lo = b * 16 + 1;
                let hi = lo + 15;
                let is_playing_bar = playing_step.is_some_and(|ps| {
                    let wrapped = ps % edited.drums.length.max(1);
                    wrapped >= b * 16 && wrapped < (b + 1) * 16
                });
                let lbl = if is_playing_bar {
                    format!(">Bar {bar_letter} ({lo}-{hi})")
                } else {
                    format!("Bar {bar_letter} ({lo}-{hi})")
                };
                if ui.selectable_label(app.drum_bar_page == b, lbl).clicked() {
                    app.drum_bar_page = b;
                }
            }

            ui.separator();
            if ui.button("Copy Bar A -> All").clicked() {
                for tr in &mut edited.drums.tracks {
                    tr.ensure_len();
                    for b in 1..4 {
                        for i in 0..16 {
                            tr.steps[b * 16 + i] = tr.steps[i];
                        }
                    }
                }
            }
            if ui.button("Clear Bar").clicked() {
                let base = app.drum_bar_page * 16;
                for tr in &mut edited.drums.tracks {
                    tr.ensure_len();
                    for i in 0..16 {
                        tr.steps[base + i] = 0;
                    }
                }
            }
        });

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.strong("Club & Classic Kits:");
            for (pid, label) in DrumMachinePatch::preset_ids() {
                if ui.button(*label).clicked() {
                    edited.drums = DrumMachinePatch::preset(pid);
                    let max_bars = (edited.drums.length / 16).clamp(1, 4);
                    if app.drum_bar_page >= max_bars {
                        app.drum_bar_page = 0;
                    }
                }
            }
        });
    });

    ui.add_space(6.0);

    // 2. Classic TR-808 / TR-909 Color-Coded 8x16 Step Matrix (for active Bar A/B/C/D)
    let bar_base = (app.drum_bar_page.min(3)) * 16;
    let active_play_col = playing_step.and_then(|ps| {
        let wrapped = ps % edited.drums.length.clamp(1, DRUM_MAX_STEPS);
        if wrapped >= bar_base && wrapped < bar_base + 16 {
            Some(wrapped - bar_base)
        } else {
            None
        }
    });

    // TR-808 beat-group colors: Beats 1..4 Red, 5..8 Orange, 9..12 Yellow, 13..16 Cream
    let beat_group_col = |col: usize| -> Color32 {
        match col / 4 {
            0 => Color32::from_rgb(220, 38, 38),
            1 => Color32::from_rgb(234, 88, 12),
            2 => Color32::from_rgb(202, 138, 4),
            _ => Color32::from_rgb(214, 211, 209),
        }
    };

    ui.group(|ui| {
        ui.strong(format!(
            "8-Voice Step Grid -- Kit: {} | Bar {} (Steps {}..{}) -- Click Step: Off -> Hit -> Accent",
            edited.drums.kit_name,
            ["A", "B", "C", "D"][app.drum_bar_page.min(3)],
            bar_base + 1,
            bar_base + 16
        ));

        egui::Grid::new("drum_machine_8x16_grid")
            .spacing(Vec2::new(4.0, 4.0))
            .show(ui, |ui| {
                ui.strong("Voice");
                ui.strong("Ctl");
                for col in 0..16 {
                    let step_num = bar_base + col + 1;
                    let col_txt = if active_play_col == Some(col) {
                        RichText::new(format!(">{step_num:02}"))
                            .strong()
                            .color(pal.arc_primary)
                    } else {
                        RichText::new(format!("{step_num:02}")).small()
                    };
                    ui.label(col_txt);
                }
                ui.strong("Euc");
                ui.end_row();

                for (vi, kind) in DrumVoiceKind::ALL.iter().enumerate().take(DRUM_VOICE_COUNT) {
                    let tr = &mut edited.drums.tracks[vi];
                    let is_sel = app.drum_selected_voice == vi;

                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(is_sel, format!("{} {}", kind.code(), kind.label()))
                            .clicked()
                        {
                            app.drum_selected_voice = vi;
                            audition_voice = Some((vi, false));
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.small_button("Hit").clicked() {
                            app.drum_selected_voice = vi;
                            audition_voice = Some((vi, true));
                        }
                        let m_fill = if tr.mute {
                            Color32::from_rgb(185, 28, 28)
                        } else {
                            pal.module_bg
                        };
                        if ui
                            .add(egui::Button::new(RichText::new("M").small()).fill(m_fill))
                            .clicked()
                        {
                            tr.mute = !tr.mute;
                        }
                        let s_fill = if tr.solo {
                            Color32::from_rgb(202, 138, 4)
                        } else {
                            pal.module_bg
                        };
                        if ui
                            .add(egui::Button::new(RichText::new("S").small()).fill(s_fill))
                            .clicked()
                        {
                            tr.solo = !tr.solo;
                        }
                    });

                    for col in 0..16 {
                        let step_idx = bar_base + col;
                        let st = tr.step_state(step_idx);
                        let is_head = active_play_col == Some(col);
                        let (fill, txt, txt_col) = match st {
                            2 => (beat_group_col(col), "ACC", Color32::BLACK),
                            1 => (
                                if is_head {
                                    pal.arc_primary
                                } else {
                                    beat_group_col(col).linear_multiply(0.72)
                                },
                                "ON",
                                Color32::BLACK,
                            ),
                            _ => (
                                if is_head {
                                    pal.module_header
                                } else {
                                    pal.module_bg
                                },
                                ".",
                                pal.text_muted,
                            ),
                        };
                        let btn = egui::Button::new(RichText::new(txt).monospace().small().color(txt_col))
                            .fill(fill)
                            .min_size(Vec2::new(34.0, 26.0));
                        if ui.add(btn).clicked() {
                            let next = (st + 1) % 3;
                            tr.set_step(step_idx, next);
                            if next > 0 {
                                audition_voice = Some((vi, next >= 2));
                            }
                        }
                    }

                    // Quick Euclidean pulse fill per lane
                    let default_pulses = match vi {
                        0 => 4,
                        1 | 2 => 2,
                        3 => 8,
                        4 => 4,
                        5 => 3,
                        6 => 5,
                        _ => 1,
                    };
                    if ui
                        .small_button(format!("E({default_pulses})"))
                        .on_hover_text("Fill lane with Euclidean rhythm")
                        .clicked()
                    {
                        edited.drums.fill_euclidean(vi, default_pulses * (edited.drums.length / 16).max(1));
                    }

                    ui.end_row();
                }
            });
    });

    ui.add_space(6.0);

    // 3. Selected Voice Hardware Rotary Strip + Master Drum Console
    let sel_vi = app.drum_selected_voice.min(DRUM_VOICE_COUNT - 1);
    let sel_kind = DrumVoiceKind::ALL[sel_vi];
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.strong(format!(
                "Selected Voice Sculptor: {} -- {}",
                sel_kind.code(),
                sel_kind.label()
            ));
            let tr = &mut edited.drums.tracks[sel_vi];
            ui.horizontal_wrapped(|ui| {
                vertical_fader(ui, &pal, "LEVEL", &mut tr.level, 0.0, 1.4, "%");
                rotary_knob(ui, &pal, "TUNE", &mut tr.tune, -12.0, 12.0, "st");
                rotary_knob(ui, &pal, "DECAY", &mut tr.decay, 0.05, 1.0, "%");
                rotary_knob(ui, &pal, "PAN", &mut tr.pan, -1.0, 1.0, "");
            });
        });

        cols[1].group(|ui| {
            ui.strong("Master Drum Bus, Swing & Global Tempo");
            ui.horizontal_wrapped(|ui| {
                vertical_fader(
                    ui,
                    &pal,
                    "DRUM MIX",
                    &mut edited.drums.master_level,
                    0.0,
                    1.5,
                    "%",
                );
                rotary_knob(ui, &pal, "SWING", &mut edited.drums.swing, 0.50, 0.75, "%");
                rotary_knob(ui, &pal, "BPM", &mut edited.bpm, 70.0, 190.0, "BPM");
                rotary_knob(ui, &pal, "MASTER", &mut edited.master_gain, 0.05, 1.0, "%");
            });
        });
    });

    app.apply_patch_edit(&before, &edited);

    if let Some((vi, acc)) = audition_voice {
        if let Some(audio) = &app.audio {
            audio.send(AudioMsg::TriggerDrum(vi, acc));
        }
    }
}
