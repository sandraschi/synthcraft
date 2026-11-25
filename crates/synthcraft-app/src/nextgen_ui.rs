//! Next-Gen Lab UI (`Workbench 11`):
//! - Subpage 0: Drawable MSEG Spline Envelope + Dual Morphing Filter B (Formant / Comb / MS-20) + Animated Mod Rings
//! - Subpage 1: Sample-Buffer Granular Cloud Visualizer + 64-Partial Spectral Harmonic Sculptor
//! - Subpage 2: Multi-Core "Genopatch" Audio-to-Patch Evolutionary Matcher (Synplant 2 style)

use crate::app::SynthCraftApp;
use crate::controls::{rotary_knob, rotary_knob_mod, vertical_fader};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::path::PathBuf;
use synthcraft_core::{
    evolve_matching_patches, load_wav_mono_samples, synthesize_audio_buffer, Command, Filter2Mode,
    Filter2Routing, MsegMode, MsegParams, MsegPoint, MSEG_MAX_POINTS, SPECTRAL_PARTIALS,
};

pub fn render_nextgen_lab(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("11. Next-Gen Lab: MSEG / Dual Morph Filter / Granular / Spectral / Genopatch");
        for (idx, lbl) in [
            (0_usize, "MSEG & Dual Filter B"),
            (1, "Granular Cloud & 64-Partial Spectral"),
            (2, "Genopatch AI Audio-to-Patch"),
        ] {
            if ui.selectable_label(app.subpage == idx, lbl).clicked() {
                app.subpage = idx;
            }
        }
    });
    ui.separator();

    match app.subpage {
        0 => render_mseg_and_filter2(app, ui),
        1 => render_granular_and_spectral(app, ui),
        _ => render_genopatch_lab(app, ui),
    }
}

fn render_mseg_and_filter2(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    let before = app.engine.active_patch().clone();
    let mut edited = before.clone();

    // Animated visual MSEG preview phase (runs smoothly even when audio engine is idle)
    let time_s = ui.input(|i| i.time) as f32;
    let phase_vis = if edited.mseg.mode == MsegMode::Off {
        0.0
    } else {
        let hz = match edited.mseg.mode {
            MsegMode::FreeHz => edited.mseg.rate_hz.clamp(0.05, 32.0),
            _ => (edited.bpm / 60.0) / edited.mseg.tempo_beats.clamp(0.25, 16.0),
        };
        (time_s * hz).fract()
    };
    let live_mod = if edited.mseg.mode == MsegMode::Off {
        0.0
    } else {
        edited.mseg.eval(phase_vis)
    };

    ui.group(|ui| {
        ui.strong("Drawable Multi-Segment Envelope Generator (MSEG) -- Drag Points, Double-Click to Add, Right-Click to Delete");
        ui.horizontal_wrapped(|ui| {
            ui.label("Mode:");
            for m in MsegMode::ALL {
                if ui
                    .selectable_label(edited.mseg.mode == m, m.as_str())
                    .clicked()
                {
                    edited.mseg.mode = m;
                }
            }
            ui.separator();
            ui.label("Presets:");
            for (pid, lbl) in MsegParams::preset_names() {
                if ui.small_button(*lbl).clicked() {
                    edited.mseg.points = MsegParams::preset_points(pid);
                    if edited.mseg.mode == MsegMode::Off {
                        edited.mseg.mode = MsegMode::TempoLoop;
                    }
                }
            }
        });

        // Interactive Drawable MSEG Canvas
        let avail_w = ui.available_width().max(520.0);
        let canvas_h = 165.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(avail_w, canvas_h), Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 6.0, pal.lcd_bg);
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.4, pal.lcd_border),
            egui::StrokeKind::Inside,
        );

        // Beat grid lines
        for gx in 1..8 {
            let x = rect.min.x + rect.width() * (gx as f32 / 8.0);
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(0.6, Color32::from_rgba_unmultiplied(148, 163, 184, 35)),
            );
        }
        for gy in 1..4 {
            let y = rect.min.y + rect.height() * (gy as f32 / 4.0);
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(0.6, Color32::from_rgba_unmultiplied(148, 163, 184, 35)),
            );
        }

        let to_screen = |x: f32, y: f32| -> Pos2 {
            Pos2::new(
                rect.min.x + 8.0 + x.clamp(0.0, 1.0) * (rect.width() - 16.0),
                rect.max.y - 8.0 - y.clamp(0.0, 1.0) * (rect.height() - 16.0),
            )
        };
        let from_screen = |p: Pos2| -> (f32, f32) {
            let x = ((p.x - (rect.min.x + 8.0)) / (rect.width() - 16.0).max(1.0)).clamp(0.0, 1.0);
            let y = (((rect.max.y - 8.0) - p.y) / (rect.height() - 16.0).max(1.0)).clamp(0.0, 1.0);
            (x, y)
        };

        // Handle point drag / double-click add / right-click delete
        if response.double_clicked() && edited.mseg.points.len() < MSEG_MAX_POINTS {
            if let Some(pos) = response.interact_pointer_pos() {
                let (nx, ny) = from_screen(pos);
                edited.mseg.points.push(MsegPoint {
                    x: nx,
                    y: ny,
                    curve: 0.0,
                });
                edited.mseg.sort_and_clamp();
            }
        } else if response.secondary_clicked() && edited.mseg.points.len() > 2 {
            if let Some(pos) = response.interact_pointer_pos() {
                let mut best: Option<(usize, f32)> = None;
                for (idx, pt) in edited.mseg.points.iter().enumerate() {
                    if idx == 0 || idx + 1 == edited.mseg.points.len() {
                        continue;
                    }
                    let d = to_screen(pt.x, pt.y).distance(pos);
                    if d < 18.0 && best.is_none_or(|(_, bd)| d < bd) {
                        best = Some((idx, d));
                    }
                }
                if let Some((idx, _)) = best {
                    edited.mseg.points.remove(idx);
                    edited.mseg.sort_and_clamp();
                }
            }
        } else if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let (nx, ny) = from_screen(pos);
                let mut nearest = 0usize;
                let mut best_d = f32::MAX;
                for (idx, pt) in edited.mseg.points.iter().enumerate() {
                    let d = to_screen(pt.x, pt.y).distance(pos);
                    if d < best_d {
                        best_d = d;
                        nearest = idx;
                    }
                }
                if best_d < 36.0 {
                    let last_idx = edited.mseg.points.len().saturating_sub(1);
                    if let Some(pt) = edited.mseg.points.get_mut(nearest) {
                        if nearest != 0 && nearest != last_idx {
                            pt.x = nx;
                        }
                        pt.y = ny;
                    }
                    edited.mseg.sort_and_clamp();
                }
            }
        }

        // Draw evaluated MSEG curve
        let curve_pts: Vec<Pos2> = (0..=128)
            .map(|i| {
                let u = i as f32 / 128.0;
                to_screen(u, edited.mseg.eval(u))
            })
            .collect();
        painter.add(egui::Shape::line(curve_pts, Stroke::new(2.4, pal.arc_primary)));

        // Draw live animated playhead & glowing mod dot
        if edited.mseg.mode != MsegMode::Off {
            let ph_p = to_screen(phase_vis, live_mod);
            painter.line_segment(
                [Pos2::new(ph_p.x, rect.min.y + 4.0), Pos2::new(ph_p.x, rect.max.y - 4.0)],
                Stroke::new(1.5, pal.arc_secondary),
            );
            painter.circle_filled(ph_p, 5.0, pal.arc_secondary);
        }

        // Draw interactive control nodes
        for pt in &edited.mseg.points {
            let sp = to_screen(pt.x, pt.y);
            painter.circle_filled(sp, 5.5, pal.knob_pointer);
            painter.circle_stroke(sp, 5.5, Stroke::new(1.8, pal.arc_primary));
        }

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            rotary_knob(ui, &pal, "BEATS", &mut edited.mseg.tempo_beats, 0.25, 16.0, "bt");
            rotary_knob(ui, &pal, "FREE HZ", &mut edited.mseg.rate_hz, 0.05, 16.0, "Hz");
            rotary_knob(ui, &pal, "-> CUTOFF", &mut edited.mseg.cutoff_depth, -1.0, 1.0, "%");
            rotary_knob(ui, &pal, "-> FILTER B", &mut edited.mseg.filter2_depth, -1.0, 1.0, "%");
            rotary_knob(ui, &pal, "-> MORPH", &mut edited.mseg.morph_depth, -1.0, 1.0, "%");
        });
    });

    ui.add_space(6.0);

    // Dual Morphing Filter B Console with Live Animated Modulation Rings!
    ui.group(|ui| {
        ui.strong("Dual Morphing Filter B (Formant A-E-I-O-U / Comb +/- / MS-20 Screamer) + Live Animated Mod Rings");
        ui.horizontal_wrapped(|ui| {
            ui.label("Filter B Mode:");
            for fm in Filter2Mode::ALL {
                if ui
                    .selectable_label(edited.filter2.mode == fm, fm.as_str())
                    .clicked()
                {
                    edited.filter2.mode = fm;
                }
            }
            ui.separator();
            ui.label("Routing:");
            for rt in [Filter2Routing::Serial, Filter2Routing::Parallel] {
                if ui
                    .selectable_label(edited.filter2.routing == rt, rt.as_str())
                    .clicked()
                {
                    edited.filter2.routing = rt;
                }
            }
        });
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let f2_mod_ring = live_mod * edited.mseg.filter2_depth;
            let cut_mod_ring = live_mod * edited.mseg.cutoff_depth;
            rotary_knob_mod(
                ui,
                &pal,
                "FILT B CUT",
                &mut edited.filter2.cutoff_hz,
                40.0,
                16000.0,
                "Hz",
                f2_mod_ring,
            );
            rotary_knob_mod(
                ui,
                &pal,
                "FILT B RES",
                &mut edited.filter2.resonance,
                0.0,
                0.98,
                "%",
                f2_mod_ring * 0.4,
            );
            rotary_knob_mod(
                ui,
                &pal,
                "VOWEL A-U",
                &mut edited.filter2.vowel_morph,
                0.0,
                4.0,
                "v",
                f2_mod_ring * 0.7,
            );
            vertical_fader(ui, &pal, "FILT B MIX", &mut edited.filter2.mix, 0.0, 1.0, "%");
            ui.separator();
            rotary_knob_mod(
                ui,
                &pal,
                "FILT A CUT",
                &mut edited.cutoff_hz,
                40.0,
                12000.0,
                "Hz",
                cut_mod_ring,
            );
            rotary_knob_mod(
                ui,
                &pal,
                "X-MORPH",
                &mut edited.extreme_morph,
                0.0,
                1.0,
                "%",
                live_mod * edited.mseg.morph_depth,
            );
        });
    });

    app.apply_patch_edit(&before, &edited);
}

fn render_granular_and_spectral(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    let before = app.engine.active_patch().clone();
    let mut edited = before.clone();
    edited.spectral.ensure_len();

    // 1. Sample-Buffer / Live-Capture Granular Cloud
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("24-Grain Live-Capture Granular Cloud Engine (Hann Windowed)");
            if ui
                .selectable_label(
                    edited.granular.enabled,
                    if edited.granular.enabled {
                        "GRANULAR: ON"
                    } else {
                        "GRANULAR: OFF"
                    },
                )
                .clicked()
            {
                edited.granular.enabled = !edited.granular.enabled;
            }
            if ui
                .selectable_label(
                    edited.granular.freeze,
                    if edited.granular.freeze {
                        "BUFFER: FROZEN"
                    } else {
                        "BUFFER: LIVE CAPTURE"
                    },
                )
                .clicked()
            {
                edited.granular.freeze = !edited.granular.freeze;
            }
        });

        // Visual Grain Cloud Canvas
        let avail_w = ui.available_width().max(520.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, 84.0), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6.0, pal.lcd_bg);
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.2, pal.lcd_border),
            egui::StrokeKind::Inside,
        );

        let pos_x = rect.min.x + edited.granular.position.clamp(0.0, 1.0) * rect.width();
        let spray_w = edited.granular.spray.clamp(0.0, 1.0) * rect.width() * 0.35;
        let window_rect = Rect::from_min_max(
            Pos2::new((pos_x - spray_w * 0.5).max(rect.min.x), rect.min.y + 4.0),
            Pos2::new((pos_x + spray_w * 0.5).min(rect.max.x), rect.max.y - 4.0),
        );
        painter.rect_filled(
            window_rect,
            3.0,
            Color32::from_rgba_unmultiplied(56, 189, 248, 38),
        );
        painter.line_segment(
            [
                Pos2::new(pos_x, rect.min.y + 3.0),
                Pos2::new(pos_x, rect.max.y - 3.0),
            ],
            Stroke::new(2.0, pal.arc_primary),
        );
        let t = ui.input(|i| i.time) as f32;
        for g in 0..edited.granular.density.clamp(1, 24) {
            let phase = (t * 0.9 + g as f32 * 0.37).fract();
            let gx = (pos_x + (phase - 0.5) * spray_w.max(18.0))
                .clamp(rect.min.x + 6.0, rect.max.x - 6.0);
            let gy =
                rect.min.y + 14.0 + ((g as f32 * 19.3).sin() * 0.5 + 0.5) * (rect.height() - 28.0);
            let r = 3.0 + (1.0 - (phase - 0.5).abs() * 2.0) * 4.0;
            painter.circle_filled(Pos2::new(gx, gy), r, pal.arc_secondary);
        }

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            vertical_fader(
                ui,
                &pal,
                "CLOUD MIX",
                &mut edited.granular.mix,
                0.0,
                1.0,
                "%",
            );
            rotary_knob(
                ui,
                &pal,
                "POSITION",
                &mut edited.granular.position,
                0.0,
                1.0,
                "%",
            );
            rotary_knob(ui, &pal, "SPRAY", &mut edited.granular.spray, 0.0, 1.0, "%");
            rotary_knob(
                ui,
                &pal,
                "GRAIN MS",
                &mut edited.granular.grain_size_ms,
                10.0,
                400.0,
                "ms",
            );
            rotary_knob(
                ui,
                &pal,
                "PITCH",
                &mut edited.granular.pitch_semitones,
                -24.0,
                24.0,
                "st",
            );
            rotary_knob(
                ui,
                &pal,
                "SPREAD",
                &mut edited.granular.stereo_spread,
                0.0,
                1.0,
                "%",
            );
            ui.add(egui::Slider::new(&mut edited.granular.density, 1..=24).text("Grains"));
        });
    });

    ui.add_space(6.0);

    // 2. 64-Partial Spectral Additive Resynthesis Sculptor
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong(
                "64-Partial Spectral Additive Resynthesis -- Drag Across Bars to Sculpt Harmonics",
            );
            if ui
                .selectable_label(
                    edited.spectral.enabled,
                    if edited.spectral.enabled {
                        "SPECTRAL: ON"
                    } else {
                        "SPECTRAL: OFF"
                    },
                )
                .clicked()
            {
                edited.spectral.enabled = !edited.spectral.enabled;
            }
            if ui.button("Capture Active Patch -> 64 Partials").clicked() {
                let pcm = synthesize_audio_buffer(&edited, app.engine.sample_rate_hz, 120);
                edited.spectral.capture_from_buffer(&pcm);
                edited.spectral.enabled = true;
            }
            for (pid, lbl) in [
                ("vocal_choir", "Vocal Choir"),
                ("glass_bell", "Glass Bell"),
                ("cathedral_organ", "Cathedral Organ"),
                ("warm_saw", "Harmonic Saw"),
            ] {
                if ui.small_button(lbl).clicked() {
                    edited.spectral.load_preset(pid);
                    edited.spectral.enabled = true;
                }
            }
        });

        // Interactive 64-bar harmonic partial editor
        let avail_w = ui.available_width().max(520.0);
        let (rect, resp) =
            ui.allocate_exact_size(Vec2::new(avail_w, 120.0), Sense::click_and_drag());
        if (resp.clicked() || resp.dragged()) && resp.interact_pointer_pos().is_some() {
            if let Some(pos) = resp.interact_pointer_pos() {
                let bin = (((pos.x - rect.min.x) / rect.width().max(1.0))
                    * SPECTRAL_PARTIALS as f32)
                    .floor() as usize;
                let amp = (1.0 - ((pos.y - rect.min.y) / rect.height().max(1.0))).clamp(0.0, 1.0);
                if bin < edited.spectral.partials.len() {
                    edited.spectral.partials[bin] = amp;
                }
            }
        }
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6.0, pal.lcd_bg);
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.2, pal.lcd_border),
            egui::StrokeKind::Inside,
        );
        let bw = rect.width() / SPECTRAL_PARTIALS as f32;
        for (k, &amp) in edited.spectral.partials.iter().enumerate() {
            let h = amp.clamp(0.0, 1.0) * (rect.height() - 10.0);
            let bar = Rect::from_min_max(
                Pos2::new(rect.min.x + k as f32 * bw + 1.0, rect.max.y - 5.0 - h),
                Pos2::new(rect.min.x + (k + 1) as f32 * bw - 1.0, rect.max.y - 5.0),
            );
            let col = if k.is_multiple_of(2) {
                pal.arc_primary
            } else {
                pal.arc_secondary
            };
            painter.rect_filled(bar, 1.5, col);
        }

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            vertical_fader(
                ui,
                &pal,
                "SPEC MIX",
                &mut edited.spectral.mix,
                0.0,
                1.0,
                "%",
            );
            rotary_knob(
                ui,
                &pal,
                "FORMANT",
                &mut edited.spectral.formant_shift_st,
                -12.0,
                12.0,
                "st",
            );
            rotary_knob(
                ui,
                &pal,
                "ODD/EVEN",
                &mut edited.spectral.odd_even_tilt,
                -1.0,
                1.0,
                "",
            );
            rotary_knob(
                ui,
                &pal,
                "H-WARP",
                &mut edited.spectral.harmonic_warp,
                -0.5,
                0.5,
                "",
            );
        });
    });

    app.apply_patch_edit(&before, &edited);
}

fn render_genopatch_lab(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    ui.group(|ui| {
        ui.strong("Multi-Core Genopatch Audio-to-Patch Evolutionary Matcher (Synplant 2 Style)");
        ui.label(
            "Point at any .wav file (or test against a target synth sound), then run parallel evolutionary search across all CPU cores to breed 6 editable SynthCraft DNA branches.",
        );
        ui.horizontal_wrapped(|ui| {
            ui.label("Target .WAV Path:");
            ui.add(
                egui::TextEdit::singleline(&mut app.genopatch_wav_path)
                    .hint_text("e.g. demo_1990_m1_piano.wav")
                    .desired_width(280.0),
            );
            ui.add(egui::Slider::new(&mut app.genopatch_gens, 3..=20).text("Generations"));
            ui.add(egui::Slider::new(&mut app.genopatch_pop, 16..=96).text("Population"));
        });

        ui.horizontal_wrapped(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("Evolve Matching Patch from .WAV (Multi-Core)")
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(pal.arc_primary),
                )
                .clicked()
            {
                let p = PathBuf::from(&app.genopatch_wav_path);
                match load_wav_mono_samples(&p) {
                    Ok((samples, sr)) => {
                        let seed = app.engine.active_patch().clone();
                        let report = evolve_matching_patches(
                            &app.genopatch_wav_path,
                            &samples,
                            &seed,
                            sr,
                            app.genopatch_gens,
                            app.genopatch_pop,
                        );
                        app.status_line = format!(
                            "Genopatch evolved {} evaluations on {} threads in {} ms (best {:.1}%)",
                            report.total_evaluations,
                            report.threads_used,
                            report.elapsed_ms,
                            report.candidates.first().map(|c| c.similarity_pct).unwrap_or(0.0)
                        );
                        app.genopatch_report = Some(report);
                    }
                    Err(err) => {
                        app.status_line = format!("Genopatch WAV error: {err}");
                    }
                }
            }

            if ui.button("Evolve Against Current Patch (Self-Test)").clicked() {
                let seed = app.engine.active_patch().clone();
                let target = synthesize_audio_buffer(&seed, app.engine.sample_rate_hz, 160);
                let report = evolve_matching_patches(
                    &seed.name,
                    &target,
                    &seed,
                    app.engine.sample_rate_hz,
                    app.genopatch_gens,
                    app.genopatch_pop,
                );
                app.status_line = format!(
                    "Genopatch self-evolved {} evals on {} threads in {} ms",
                    report.total_evaluations, report.threads_used, report.elapsed_ms
                );
                app.genopatch_report = Some(report);
            }
        });
    });

    ui.add_space(6.0);

    if let Some(report) = app.genopatch_report.clone() {
        ui.group(|ui| {
            ui.strong(format!(
                "Evolved DNA Branches for '{}' -- {} evaluations | {} CPU threads | {} ms",
                report.target_name,
                report.total_evaluations,
                report.threads_used,
                report.elapsed_ms
            ));
            let mut apply_branch: Option<synthcraft_core::SynthPatch> = None;
            for cand in &report.candidates {
                ui.horizontal_wrapped(|ui| {
                    ui.strong(format!("{:5.1}%", cand.similarity_pct));
                    ui.add(
                        egui::ProgressBar::new((cand.similarity_pct / 100.0).clamp(0.0, 1.0))
                            .desired_width(140.0),
                    );
                    ui.monospace(&cand.label);
                    if ui.button(format!("Load DNA #{}", cand.branch_id)).clicked() {
                        apply_branch = Some(cand.patch.clone());
                    }
                });
            }
            if let Some(patch) = apply_branch {
                let _ = Command::ApplyEvolvedPatch {
                    patch: Box::new(patch),
                }
                .dispatch(&mut app.engine);
                app.persist_state();
                app.status_line = "Loaded evolved Genopatch DNA branch into active synth".into();
            }
        });
    }
}
