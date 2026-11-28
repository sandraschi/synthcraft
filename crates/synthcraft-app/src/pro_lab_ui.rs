//! Workbench 12 -- Pro Lab UI:
//! - Subpage 0: 6-Operator Thru-Zero FM Routing Matrix + 16-Voice Hyper-Unison Stack
//! - Subpage 1: Reorderable Studio FX Rack (OTT 3-Band, Convolution IR, Shimmer Pitch Reverb, Analog Tape & Bitcrusher)
//! - Subpage 2: Goliath-Class 2D FDTD Physical Modeling Plate/Membrane Mesh + 8x Oversampled WDF Circuits (Moog CP3 / Buchla 292 LPG / TB-303)

use crate::app::SynthCraftApp;
use crate::controls::{rotary_knob, vertical_fader};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Vec2};
use synthcraft_core::{
    Fm6Params, FxSlotKind, IrPreset, PhysicalMaterial, UnisonStackMode, WdfCircuitMode, FM_OPS,
    MESH_SIDE,
};

pub fn render_pro_lab(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("12. Pro Lab: 6-Op FM / Unison / Reorderable FX / 2D Physics & 8x WDF");
        for (idx, lbl) in [
            (0_usize, "6-Op FM & 16-Voice Unison"),
            (1, "Reorderable Studio FX Rack"),
            (2, "2D FDTD Mesh & 8x WDF Circuits"),
        ] {
            if ui.selectable_label(app.subpage == idx, lbl).clicked() {
                app.subpage = idx;
            }
        }
    });
    ui.separator();

    match app.subpage {
        0 => render_fm6_and_unison(app, ui),
        1 => render_studio_fx_rack(app, ui),
        _ => render_physical_and_wdf(app, ui),
    }
}

fn render_fm6_and_unison(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    let before = app.engine.active_patch().clone();
    let mut edited = before.clone();

    // 1. 6-Operator Thru-Zero FM Matrix
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("6-Operator Thru-Zero FM Routing Matrix (FM8 / Opsix / DX7 Style)");
            if ui
                .selectable_label(
                    edited.fm6.enabled,
                    if edited.fm6.enabled {
                        "6-OP FM: ON"
                    } else {
                        "6-OP FM: OFF"
                    },
                )
                .clicked()
            {
                edited.fm6.enabled = !edited.fm6.enabled;
            }
            ui.separator();
            ui.label("Algorithms:");
            for (pid, lbl) in Fm6Params::preset_names() {
                if ui.small_button(*lbl).clicked() {
                    edited.fm6 = Fm6Params::preset(pid);
                }
            }
        });

        ui.add_space(4.0);
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.strong("6x6 FM Matrix (Row = Source -> Col = Dest; Diagonal = Self-Feedback)");
                egui::Grid::new("fm6_matrix_grid")
                    .spacing(Vec2::new(4.0, 4.0))
                    .show(ui, |ui| {
                        ui.strong("Src\\Dst");
                        for d in 0..FM_OPS {
                            ui.strong(format!("Op{}", d + 1));
                        }
                        ui.end_row();
                        for (s, row) in edited.fm6.matrix.iter_mut().enumerate().take(FM_OPS) {
                            ui.strong(format!("Op{}", s + 1));
                            for (d, v) in row.iter_mut().enumerate().take(FM_OPS) {
                                let active = *v > 0.01;
                                let fill = if s == d && active {
                                    pal.led_accent
                                } else if active {
                                    pal.arc_primary
                                } else {
                                    pal.module_bg
                                };
                                let txt = if active {
                                    format!("{:.1}", *v)
                                } else {
                                    ".".into()
                                };
                                let btn = egui::Button::new(
                                    RichText::new(txt).monospace().small().color(if active {
                                        Color32::BLACK
                                    } else {
                                        pal.text_muted
                                    }),
                                )
                                .fill(fill)
                                .min_size(Vec2::new(42.0, 24.0));
                                let resp = ui.add(btn);
                                if resp.clicked() {
                                    *v = if *v < 0.05 {
                                        0.75
                                    } else if *v < 1.5 {
                                        1.75
                                    } else if *v < 2.8 {
                                        3.0
                                    } else {
                                        0.0
                                    };
                                } else if resp.secondary_clicked() {
                                    *v = 0.0;
                                }
                            }
                            ui.end_row();
                        }
                    });
            });

            cols[1].group(|ui| {
                ui.strong("Operator Harmonic Ratios, Output Carriers & Master FM Mix");
                ui.horizontal(|ui| {
                    vertical_fader(ui, &pal, "FM6 MIX", &mut edited.fm6.mix, 0.0, 1.0, "%");
                    ui.vertical(|ui| {
                        for op in 0..FM_OPS {
                            ui.horizontal(|ui| {
                                ui.monospace(format!("Op{}", op + 1));
                                ui.add(
                                    egui::Slider::new(&mut edited.fm6.ratios[op], 0.25..=16.0)
                                        .text(format!("R{}", op + 1)),
                                );
                                ui.add(
                                    egui::Slider::new(&mut edited.fm6.levels[op], 0.0..=1.0)
                                        .text(format!("Out{}", op + 1)),
                                );
                            });
                        }
                    });
                });
            });
        });
    });

    ui.add_space(6.0);

    // 2. 16-Voice Hyper-Unison Stack
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("Per-Voice 16-Voice Hyper-Unison Stack (SuperSaw / Chord / Golden Ratio)");
            if ui
                .selectable_label(
                    edited.unison.enabled,
                    if edited.unison.enabled {
                        "UNISON: ON"
                    } else {
                        "UNISON: OFF"
                    },
                )
                .clicked()
            {
                edited.unison.enabled = !edited.unison.enabled;
            }
            ui.separator();
            for m in UnisonStackMode::ALL {
                if ui
                    .selectable_label(edited.unison.stack_mode == m, m.as_str())
                    .clicked()
                {
                    edited.unison.stack_mode = m;
                }
            }
        });
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            vertical_fader(ui, &pal, "UNI MIX", &mut edited.unison.mix, 0.0, 1.0, "%");
            rotary_knob(
                ui,
                &pal,
                "DETUNE",
                &mut edited.unison.detune_cents,
                0.0,
                100.0,
                "ct",
            );
            rotary_knob(
                ui,
                &pal,
                "SPREAD",
                &mut edited.unison.stereo_spread,
                0.0,
                1.0,
                "%",
            );
            ui.add(egui::Slider::new(&mut edited.unison.voices, 2..=16).text("Unison Voices"));
        });
    });

    app.apply_patch_edit(&before, &edited);
}

fn render_studio_fx_rack(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    let before = app.engine.active_patch().clone();
    let mut edited = before.clone();

    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("Reorderable Studio FX Rack (OTT Multiband / Convolution IR / Shimmer / Tape & Bitcrusher)");
            if ui
                .selectable_label(
                    edited.studio_fx.enabled,
                    if edited.studio_fx.enabled {
                        "STUDIO FX RACK: ON"
                    } else {
                        "STUDIO FX RACK: BYPASS"
                    },
                )
                .clicked()
            {
                edited.studio_fx.enabled = !edited.studio_fx.enabled;
            }
        });
    });

    ui.add_space(4.0);

    let mut swap_idx: Option<(usize, usize)> = None;
    for slot_pos in 0..4 {
        let kind = edited.studio_fx.order[slot_pos];
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(format!("Slot #{}: {}", slot_pos + 1, kind.label()));
                if slot_pos > 0 && ui.small_button("Move Up").clicked() {
                    swap_idx = Some((slot_pos, slot_pos - 1));
                }
                if slot_pos < 3 && ui.small_button("Move Down").clicked() {
                    swap_idx = Some((slot_pos, slot_pos + 1));
                }
                ui.separator();
                match kind {
                    FxSlotKind::OttMultiband => {
                        ui.checkbox(&mut edited.studio_fx.ott_enabled, "Enable OTT");
                    }
                    FxSlotKind::ConvolutionIr => {
                        ui.checkbox(&mut edited.studio_fx.conv_enabled, "Enable IR");
                    }
                    FxSlotKind::ShimmerReverb => {
                        ui.checkbox(&mut edited.studio_fx.shimmer_enabled, "Enable Shimmer");
                    }
                    FxSlotKind::TapeBitcrush => {
                        ui.checkbox(&mut edited.studio_fx.tape_enabled, "Enable Tape/Crush");
                    }
                }
            });

            match kind {
                FxSlotKind::OttMultiband => {
                    ui.horizontal_wrapped(|ui| {
                        rotary_knob(
                            ui,
                            &pal,
                            "OTT DEPTH",
                            &mut edited.studio_fx.ott_depth,
                            0.0,
                            1.0,
                            "%",
                        );
                        rotary_knob(
                            ui,
                            &pal,
                            "OTT TIME",
                            &mut edited.studio_fx.ott_time,
                            0.1,
                            2.0,
                            "x",
                        );
                    });
                }
                FxSlotKind::ConvolutionIr => {
                    ui.horizontal_wrapped(|ui| {
                        for p in IrPreset::ALL {
                            if ui
                                .selectable_label(edited.studio_fx.conv_preset == p, p.label())
                                .clicked()
                            {
                                edited.studio_fx.conv_preset = p;
                            }
                        }
                        rotary_knob(
                            ui,
                            &pal,
                            "IR MIX",
                            &mut edited.studio_fx.conv_mix,
                            0.0,
                            1.0,
                            "%",
                        );
                    });
                }
                FxSlotKind::ShimmerReverb => {
                    ui.horizontal_wrapped(|ui| {
                        rotary_knob(
                            ui,
                            &pal,
                            "SHIM MIX",
                            &mut edited.studio_fx.shimmer_mix,
                            0.0,
                            1.0,
                            "%",
                        );
                        rotary_knob(
                            ui,
                            &pal,
                            "PITCH",
                            &mut edited.studio_fx.shimmer_pitch_st,
                            0.0,
                            12.0,
                            "st",
                        );
                        rotary_knob(
                            ui,
                            &pal,
                            "FEEDBACK",
                            &mut edited.studio_fx.shimmer_feedback,
                            0.0,
                            0.85,
                            "%",
                        );
                    });
                }
                FxSlotKind::TapeBitcrush => {
                    ui.horizontal_wrapped(|ui| {
                        rotary_knob(
                            ui,
                            &pal,
                            "TAPE WOW",
                            &mut edited.studio_fx.tape_wow,
                            0.0,
                            1.0,
                            "%",
                        );
                        rotary_knob(
                            ui,
                            &pal,
                            "TAPE DRV",
                            &mut edited.studio_fx.tape_drive,
                            0.0,
                            2.0,
                            "x",
                        );
                        rotary_knob(
                            ui,
                            &pal,
                            "BIT DEPTH",
                            &mut edited.studio_fx.crush_bits,
                            4.0,
                            16.0,
                            "bit",
                        );
                        ui.add(
                            egui::Slider::new(&mut edited.studio_fx.crush_downsample, 1..=16)
                                .text("Downsample"),
                        );
                    });
                }
            }
        });
    }

    if let Some((a, b)) = swap_idx {
        edited.studio_fx.order.swap(a, b);
    }

    app.apply_patch_edit(&before, &edited);
}

fn render_physical_and_wdf(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    let before = app.engine.active_patch().clone();
    let mut edited = before.clone();

    ui.columns(2, |cols| {
        // 1. 2D FDTD Physical Modeling Plate / Membrane Resonator
        cols[0].group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Goliath 2D FDTD Waveguide Mesh (8x8 Coupled Spatial Nodes)");
                if ui
                    .selectable_label(
                        edited.physical_mesh.enabled,
                        if edited.physical_mesh.enabled {
                            "2D MESH: ON"
                        } else {
                            "2D MESH: OFF"
                        },
                    )
                    .clicked()
                {
                    edited.physical_mesh.enabled = !edited.physical_mesh.enabled;
                }
            });
            ui.horizontal_wrapped(|ui| {
                for m in PhysicalMaterial::ALL {
                    if ui
                        .selectable_label(edited.physical_mesh.material == m, m.label())
                        .clicked()
                    {
                        edited.physical_mesh.material = m;
                    }
                }
            });

            // Live 8x8 Vibrating Plate Heatmap Visualizer
            let (rect, _) = ui.allocate_exact_size(Vec2::new(220.0, 148.0), Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 6.0, pal.lcd_bg);
            let t = ui.input(|i| i.time) as f32;
            let cw = (rect.width() - 12.0) / MESH_SIDE as f32;
            let ch = (rect.height() - 12.0) / MESH_SIDE as f32;
            for y in 0..MESH_SIDE {
                for x in 0..MESH_SIDE {
                    let wave = if edited.physical_mesh.enabled {
                        ((x as f32 * 0.9 + t * 5.0).sin() * (y as f32 * 0.9 - t * 4.0).cos() * 0.5
                            + 0.5)
                            * edited.physical_mesh.mix
                    } else {
                        0.08
                    };
                    let col = Color32::from_rgb(
                        (34.0 + wave * 210.0) as u8,
                        (60.0 + wave * 170.0) as u8,
                        (120.0 + (1.0 - wave) * 110.0) as u8,
                    );
                    let cell = Rect::from_min_size(
                        Pos2::new(rect.min.x + 6.0 + x as f32 * cw, rect.min.y + 6.0 + y as f32 * ch),
                        Vec2::new(cw - 2.0, ch - 2.0),
                    );
                    painter.rect_filled(cell, 2.0, col);
                }
            }

            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                vertical_fader(ui, &pal, "MESH MIX", &mut edited.physical_mesh.mix, 0.0, 1.0, "%");
                rotary_knob(
                    ui,
                    &pal,
                    "TENSION",
                    &mut edited.physical_mesh.tension,
                    0.05,
                    0.44,
                    "c2",
                );
                rotary_knob(
                    ui,
                    &pal,
                    "DAMPING",
                    &mut edited.physical_mesh.damping,
                    0.001,
                    0.08,
                    "d",
                );
            });
        });

        // 2. 8x Oversampled Wave Digital Filter (WDF) Analog Circuit Emulation
        cols[1].group(|ui| {
            ui.strong("8x Oversampled Wave Digital Filter (WDF) Analog Circuits (384 kHz Internal)");
            ui.label(
                "Zero-aliasing non-linear circuit emulation: Moog CP3 asymmetric overdrive, Buchla 292 CdS Vactrol Low-Pass Gate ring-out, and TB-303 diode ladder.",
            );
            ui.horizontal_wrapped(|ui| {
                for m in WdfCircuitMode::ALL {
                    if ui
                        .selectable_label(edited.wdf_circuit.mode == m, m.label())
                        .clicked()
                    {
                        edited.wdf_circuit.mode = m;
                    }
                }
            });
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                vertical_fader(ui, &pal, "WDF MIX", &mut edited.wdf_circuit.mix, 0.0, 1.0, "%");
                rotary_knob(ui, &pal, "DRIVE", &mut edited.wdf_circuit.drive, 0.0, 3.0, "x");
                rotary_knob(
                    ui,
                    &pal,
                    "VACTROL",
                    &mut edited.wdf_circuit.vactrol_decay,
                    0.05,
                    1.0,
                    "%",
                );
            });
        });
    });

    app.apply_patch_edit(&before, &edited);
}
