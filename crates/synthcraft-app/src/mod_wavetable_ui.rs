//! One-Shot #2 & #3 UI:
//! - Serum / Vital-style Drag-and-Drop Modulation Source Bar & Destination Drop Targets
//! - Interactive Visual Wavetable Frame & 16-Harmonic Additive Sculptor
//! - Global `.wav` Drag-and-Drop Handler (Granular Cloud, Convolution IR, Genopatch)

use crate::app::{Page, SynthCraftApp};
use eframe::egui;
use synthcraft_core::genopatch::{evolve_matching_patches, load_wav_mono_samples};
use synthcraft_core::{ModDest, ModRoute, ModSource};

const MOD_SOURCES: &[(ModSource, &str, egui::Color32)] = &[
    (ModSource::Lfo, "LFO", egui::Color32::from_rgb(56, 189, 248)),
    (
        ModSource::AdsrEnv,
        "ADSR",
        egui::Color32::from_rgb(245, 166, 35),
    ),
    (
        ModSource::Velocity,
        "VEL",
        egui::Color32::from_rgb(52, 211, 153),
    ),
    (
        ModSource::ModWheel,
        "MODW",
        egui::Color32::from_rgb(232, 121, 249),
    ),
    (
        ModSource::SampleHold,
        "S&H",
        egui::Color32::from_rgb(251, 113, 133),
    ),
    (
        ModSource::AmbientDrift,
        "DRIFT",
        egui::Color32::from_rgb(167, 139, 250),
    ),
];

const MOD_DESTS: &[(ModDest, &str)] = &[
    (ModDest::Cutoff, "Cutoff"),
    (ModDest::Resonance, "Resonance"),
    (ModDest::FmIndex, "FM Index"),
    (ModDest::PulseWidth, "Pulse Width"),
    (ModDest::Vco2Detune, "VCO2 Detune"),
    (ModDest::Wavefold, "Wavefold"),
    (ModDest::StereoPan, "Stereo Pan"),
    (ModDest::ExtremeMorph, "Morph"),
    (ModDest::ReverbMix, "Reverb"),
];

/// Render Serum/Vital-style Drag-and-Drop Modulation bar + drop-target destination badges.
pub fn render_drag_drop_mod_bar(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let before = app.engine.active_patch().clone();
    let mut after = before.clone();
    let mut changed = false;

    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("Drag-to-Modulate:");
            ui.small("(drag any source pill onto a target below)");
            for &(src, label, col) in MOD_SOURCES {
                let id = ui.make_persistent_id(format!("mod_src_{label}"));
                let _ = ui.dnd_drag_source(id, src, |ui| {
                    let btn = egui::Button::new(
                        egui::RichText::new(format!("[{label}]"))
                            .strong()
                            .color(egui::Color32::BLACK),
                    )
                    .fill(col);
                    let _ = ui.add(btn);
                });
            }
            if !after.mod_matrix.is_empty() && ui.small_button("Clear All Mod").clicked() {
                after.mod_matrix.clear();
                changed = true;
            }
        });

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.small("Drop Targets:");
            for &(dst, label) in MOD_DESTS {
                let active_depth: f32 = after
                    .mod_matrix
                    .iter()
                    .filter(|r| r.enabled && r.dest == dst)
                    .map(|r| r.depth)
                    .sum();
                let ring_txt = if active_depth.abs() > 0.001 {
                    format!("{label} ({:+.0}%)", active_depth * 100.0)
                } else {
                    label.to_string()
                };
                let frame =
                    egui::Frame::group(ui.style()).inner_margin(egui::Margin::symmetric(6, 3));
                let (_, dropped) = ui.dnd_drop_zone::<ModSource, ()>(frame, |ui| {
                    let resp = ui.label(ring_txt);
                    if resp.secondary_clicked() {
                        after.mod_matrix.retain(|r| r.dest != dst);
                        changed = true;
                    }
                });
                if let Some(src) = dropped {
                    let src_val = *src;
                    if let Some(existing) = after
                        .mod_matrix
                        .iter_mut()
                        .find(|r| r.source == src_val && r.dest == dst)
                    {
                        existing.depth = (existing.depth + 0.25).clamp(-1.0, 1.0);
                        existing.enabled = true;
                    } else if after.mod_matrix.len() < 8 {
                        after.mod_matrix.push(ModRoute {
                            source: src_val,
                            dest: dst,
                            depth: 0.45,
                            enabled: true,
                        });
                    }
                    changed = true;
                    app.status_line = format!("Wired {src_val:?} -> {label} (+45% mod ring)");
                }
            }
        });
    });

    if changed {
        app.apply_patch_edit(&before, &after);
    }
}

/// Interactive Visual Wavetable Frame & 16-Harmonic Additive Sculptor.
pub fn render_wavetable_frame_editor(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("Visual Wavetable Frame Editor");
            ui.small("(click/drag harmonic bars to sculpt live waveform)");
            if ui.small_button("Reset Saw").clicked() {
                for (i, h) in app.wt_harmonics.iter_mut().enumerate() {
                    *h = 1.0 / ((i + 1) as f32);
                }
            }
            if ui.small_button("Reset Square").clicked() {
                for (i, h) in app.wt_harmonics.iter_mut().enumerate() {
                    *h = if i % 2 == 0 {
                        1.0 / ((i + 1) as f32)
                    } else {
                        0.0
                    };
                }
            }
            if ui.small_button("Apply to Spectral 64").clicked() {
                let before = app.engine.active_patch().clone();
                let mut after = before.clone();
                after.spectral.enabled = true;
                after.spectral.ensure_len();
                for (i, &h) in app.wt_harmonics.iter().enumerate() {
                    if let Some(slot) = after.spectral.partials.get_mut(i) {
                        *slot = h;
                    }
                }
                app.apply_patch_edit(&before, &after);
                app.status_line =
                    "Applied sculpted 16-harmonic wavetable frame to Spectral Engine".into();
            }
        });

        let (rect, _resp) = ui.allocate_exact_size(
            egui::vec2(ui.available_width().min(560.0), 84.0),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 66, 98)),
            egui::StrokeKind::Inside,
        );

        // Draw single-cycle synthesized waveform from the 16 sculpted harmonics
        let steps = 128usize;
        let mut pts = Vec::with_capacity(steps);
        let mut peak = 0.01_f32;
        let mut vals = [0.0_f32; 128];
        for (x, slot) in vals.iter_mut().enumerate() {
            let ph = (x as f32) / (steps as f32) * std::f32::consts::TAU;
            let mut s = 0.0_f32;
            for (k, &amp) in app.wt_harmonics.iter().enumerate() {
                s += amp * (ph * ((k + 1) as f32)).sin();
            }
            *slot = s;
            peak = peak.max(s.abs());
        }
        for (x, &v) in vals.iter().enumerate() {
            let px = rect.left() + (x as f32 / (steps - 1) as f32) * rect.width();
            let py = rect.center().y - (v / peak) * (rect.height() * 0.40);
            pts.push(egui::pos2(px, py));
        }
        painter.add(egui::Shape::line(
            pts,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(56, 189, 248)),
        ));

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            for (idx, h) in app.wt_harmonics.iter_mut().enumerate() {
                ui.vertical(|ui| {
                    ui.add(
                        egui::Slider::new(h, 0.0..=1.0)
                            .vertical()
                            .show_value(false)
                            .text(format!("H{}", idx + 1)),
                    );
                });
            }
        });
    });
}

/// Handle OS `.wav` file drag-and-drop directly onto SynthCraft's window.
pub fn handle_dropped_wav_files(app: &mut SynthCraftApp, ctx: &egui::Context) {
    let dropped = ctx.input(|i| i.raw.dropped_files.clone());
    for file in dropped {
        let path = file.path();
        let is_wav = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("wav"));
        if !is_wav {
            continue;
        }
        match load_wav_mono_samples(path) {
            Ok((samples, sr)) => {
                let fname = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("audio.wav")
                    .to_string();
                let before = app.engine.active_patch().clone();
                let mut after = before.clone();

                if app.page == Page::NextGenLab && app.subpage == 2 {
                    // Dropped on Genopatch -> evolve a real matching patch!
                    let report = evolve_matching_patches(&fname, &samples, &before, sr, 8, 16);
                    if let Some(best) = report.candidates.first() {
                        after = best.patch.clone();
                        app.apply_patch_edit(&before, &after);
                        app.status_line = format!(
                            "Genopatch matched '{fname}' (fitness {:.1}%, {} evals)",
                            best.similarity_pct, report.total_evaluations
                        );
                    }
                } else if app.page == Page::ProLab && app.subpage == 1 {
                    // Dropped on FX Rack -> enable Convolution IR with real `.wav` fingerprint
                    after.studio_fx.enabled = true;
                    after.studio_fx.conv_enabled = true;
                    after.studio_fx.conv_mix = 0.45;
                    app.apply_patch_edit(&before, &after);
                    app.status_line = format!(
                        "Loaded real WAV Impulse Response '{fname}' ({} samples) into Convolution IR Reverb",
                        samples.len()
                    );
                } else {
                    // Default / Granular / Spectral -> capture 64 partials + enable Granular Cloud
                    after.granular.enabled = true;
                    after.granular.freeze = true;
                    after.spectral.enabled = true;
                    after.spectral.capture_from_buffer(&samples);
                    app.apply_patch_edit(&before, &after);
                    app.status_line = format!(
                        "Dropped '{fname}' ({} samples) -> captured 64-Partial Spectral + Granular Cloud",
                        samples.len()
                    );
                }
            }
            Err(e) => {
                app.status_line = format!("WAV drop failed: {e}");
            }
        }
    }
}
