//! Workbench 13 -- Studio Visuals, MPE / CLAP Poly-Mod & Neural DDSP Timbre Transfer:
//! - Subpage 0: 3D Perspective Scrolling STFT Waterfall Spectrogram + Lissajous Stereo Vectorscope + Phase Correlation Meter
//! - Subpage 1: Interactive MPE & CLAP 1.2.2 Poly-Mod 2D Touch Pad (Pitch Slide X / Timbre CC74 Y / Pressure)
//! - Subpage 2: Neural DDSP Timbre Transfer (Harmonic + Filtered-Noise Decoder with Latent Morphing)

use crate::app::SynthCraftApp;
use crate::audio_io::AudioMsg;
use crate::controls::{rotary_knob, vertical_fader};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use synthcraft_core::{analyze_patch_spectrum, DdspTimbre, SynthRt};

pub fn render_visuals_mpe_ddsp(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("13. Studio Visuals, MPE Poly-Mod & Neural DDSP Timbre Transfer");
        for (idx, lbl) in [
            (0_usize, "3D Waterfall & Lissajous Scope"),
            (1, "MPE & CLAP Poly-Mod Touch Pad"),
            (2, "Neural DDSP Timbre Transfer"),
        ] {
            if ui.selectable_label(app.subpage == idx, lbl).clicked() {
                app.subpage = idx;
            }
        }
    });
    ui.separator();

    match app.subpage {
        0 => render_waterfall_and_vectorscope(app, ui),
        1 => render_mpe_touch_pad(app, ui),
        _ => render_neural_ddsp(app, ui),
    }
}

fn render_waterfall_and_vectorscope(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();

    // Update 24-slice scrolling 3D waterfall history from live spectrum analysis
    let rep = analyze_patch_spectrum(app.engine.active_patch(), app.engine.sample_rate_hz, 90);
    let mut slice = [0.0_f32; 32];
    for (i, slot) in slice.iter_mut().enumerate() {
        let db = rep
            .spectrum_bands_db
            .get(i)
            .map(|&(_, mag_db)| mag_db)
            .unwrap_or(-60.0);
        *slot = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
    }
    if app.waterfall_history.len() >= 24 {
        app.waterfall_history.remove(0);
    }
    app.waterfall_history.push(slice);
    while app.waterfall_history.len() < 24 {
        app.waterfall_history.insert(0, slice);
    }

    // Generate short stereo pair from SynthRt for true Lissajous X-Y phase correlation
    let mut rt = SynthRt::new(
        app.engine.sample_rate_hz as f32,
        app.engine.active_patch().clone(),
    );
    rt.note_on(app.engine.midi_hw.last_triggered_note, 0.9);
    let mut l_buf = vec![0.0_f32; 512];
    let mut r_buf = vec![0.0_f32; 512];
    rt.process_stereo(&mut l_buf, &mut r_buf);

    // Compute stereo phase correlation (-1.0 = out of phase, 0.0 = wide, +1.0 = mono)
    let mut num = 0.0_f32;
    let mut den_l = 1e-6_f32;
    let mut den_r = 1e-6_f32;
    for (&l, &r) in l_buf.iter().zip(&r_buf) {
        num += l * r;
        den_l += l * l;
        den_r += r * r;
    }
    let correlation = (num / (den_l * den_r).sqrt()).clamp(-1.0, 1.0);

    ui.columns(2, |cols| {
        // Left: 3D Perspective Scrolling STFT Waterfall Spectrogram
        cols[0].group(|ui| {
            ui.strong("3D Perspective Scrolling STFT Waterfall Spectrogram (24-Slice Depth)");
            let avail_w = ui.available_width().max(320.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, 260.0), Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 6.0, pal.lcd_bg);
            painter.rect_stroke(
                rect,
                6.0,
                Stroke::new(1.3, pal.lcd_border),
                egui::StrokeKind::Inside,
            );

            let slices = app.waterfall_history.len().max(1);
            for (s_idx, row) in app.waterfall_history.iter().enumerate() {
                // Depth factor: 0.0 = far back (top), 1.0 = foreground (bottom)
                let depth = s_idx as f32 / (slices - 1).max(1) as f32;
                let base_y = rect.min.y + 42.0 + depth * (rect.height() - 62.0);
                let x_margin = (1.0 - depth) * 58.0 + 12.0;
                let span_w = (rect.width() - x_margin * 2.0).max(40.0);
                let amp_scale = 26.0 + depth * 42.0;

                let pts: Vec<Pos2> = row
                    .iter()
                    .enumerate()
                    .map(|(b, &mag)| {
                        let u = b as f32 / 31.0;
                        let x = rect.min.x + x_margin + u * span_w;
                        let y =
                            (base_y - mag * amp_scale).clamp(rect.min.y + 6.0, rect.max.y - 6.0);
                        Pos2::new(x, y)
                    })
                    .collect();

                let alpha = (60.0 + depth * 195.0) as u8;
                let base_col = if s_idx + 1 == slices {
                    pal.arc_primary
                } else {
                    pal.arc_secondary
                };
                let col = Color32::from_rgba_unmultiplied(
                    base_col.r(),
                    base_col.g(),
                    base_col.b(),
                    alpha,
                );
                painter.add(egui::Shape::line(pts, Stroke::new(1.1 + depth * 1.1, col)));
            }
        });

        // Right: Lissajous Stereo Vectorscope (X-Y Phase Scope) + Phase Correlation Bar
        cols[1].group(|ui| {
            ui.strong(format!(
                "Lissajous Stereo Vectorscope (X-Y Phase Scope) | Correlation: {:+.2}",
                correlation
            ));
            let avail_w = ui.available_width().max(260.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, 260.0), Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 6.0, pal.lcd_bg);
            painter.rect_stroke(
                rect,
                6.0,
                Stroke::new(1.3, pal.lcd_border),
                egui::StrokeKind::Inside,
            );

            let c = rect.center();
            let rad = rect.width().min(rect.height()) * 0.42;
            // Graticule circle + M/S diagonal axes
            painter.circle_stroke(
                c,
                rad,
                Stroke::new(0.8, Color32::from_rgba_unmultiplied(148, 163, 184, 55)),
            );
            painter.line_segment(
                [Pos2::new(c.x - rad, c.y), Pos2::new(c.x + rad, c.y)],
                Stroke::new(0.7, Color32::from_rgba_unmultiplied(148, 163, 184, 45)),
            );
            painter.line_segment(
                [Pos2::new(c.x, c.y - rad), Pos2::new(c.x, c.y + rad)],
                Stroke::new(0.7, Color32::from_rgba_unmultiplied(148, 163, 184, 45)),
            );

            // Plot rotated L/R -> Mid (vertical) / Side (horizontal) phosphor trace
            let mut prev: Option<Pos2> = None;
            for (&l, &r) in l_buf.iter().zip(&r_buf).step_by(2) {
                let side = (l - r) * 0.707;
                let mid = (l + r) * 0.707;
                let pt = Pos2::new(c.x + side * rad * 1.4, c.y - mid * rad * 1.4);
                if let Some(p0) = prev {
                    painter.line_segment([p0, pt], Stroke::new(1.4, pal.arc_primary));
                }
                prev = Some(pt);
            }

            // Phase Correlation Bar at bottom of scope
            let bar = Rect::from_min_max(
                Pos2::new(rect.min.x + 16.0, rect.max.y - 18.0),
                Pos2::new(rect.max.x - 16.0, rect.max.y - 8.0),
            );
            painter.rect_filled(bar, 3.0, pal.arc_track);
            let norm = (correlation + 1.0) * 0.5;
            let cx = bar.min.x + norm * bar.width();
            painter.circle_filled(Pos2::new(cx, bar.center().y), 5.0, pal.arc_secondary);
        });
    });
}

fn render_mpe_touch_pad(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    ui.group(|ui| {
        ui.strong("MPE & CLAP 1.2.2 Polyphonic Expression Pad (Drag X = Pitch Slide, Y = Timbre CC74, Scroll/Knob = Pressure)");
        ui.label(
            "SynthCraft.clap natively handles CLAP_EVENT_NOTE_EXPRESSION (Tuning, Brightness, Pressure), CLAP_EVENT_PARAM_MOD, MIDI Poly-Aftertouch (0xA0), and MPE CC74 independently on every held voice.",
        );
        let avail_w = ui.available_width().max(420.0);
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(avail_w, 220.0), Sense::click_and_drag());
        if (resp.clicked() || resp.dragged()) && resp.interact_pointer_pos().is_some() {
            if let Some(pos) = resp.interact_pointer_pos() {
                let nx = ((pos.x - rect.min.x) / rect.width().max(1.0)).clamp(0.0, 1.0);
                let ny = (1.0 - (pos.y - rect.min.y) / rect.height().max(1.0)).clamp(0.0, 1.0);
                app.mpe_pad_pitch_st = (nx * 2.0 - 1.0) * 12.0;
                app.mpe_pad_timbre = ny;
                if let Some(audio) = &app.audio {
                    audio.send(AudioMsg::NoteExpression(
                        None,
                        Some(app.mpe_pad_pitch_st),
                        Some(app.mpe_pad_timbre),
                        Some(app.mpe_pad_pressure),
                    ));
                }
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
        let px = rect.min.x + ((app.mpe_pad_pitch_st / 12.0) * 0.5 + 0.5).clamp(0.0, 1.0) * rect.width();
        let py = rect.max.y - app.mpe_pad_timbre.clamp(0.0, 1.0) * rect.height();
        painter.line_segment(
            [Pos2::new(px, rect.min.y), Pos2::new(px, rect.max.y)],
            Stroke::new(1.0, pal.arc_secondary),
        );
        painter.line_segment(
            [Pos2::new(rect.min.x, py), Pos2::new(rect.max.x, py)],
            Stroke::new(1.0, pal.arc_secondary),
        );
        painter.circle_filled(
            Pos2::new(px, py),
            10.0 + app.mpe_pad_pressure * 10.0,
            pal.arc_primary,
        );

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            rotary_knob(ui, &pal, "PITCH SLIDE", &mut app.mpe_pad_pitch_st, -12.0, 12.0, "st");
            rotary_knob(ui, &pal, "TIMBRE CC74", &mut app.mpe_pad_timbre, 0.0, 1.0, "%");
            rotary_knob(ui, &pal, "PRESSURE", &mut app.mpe_pad_pressure, 0.0, 1.0, "%");
            if ui.button("Center MPE").clicked() {
                app.mpe_pad_pitch_st = 0.0;
                app.mpe_pad_timbre = 0.5;
                app.mpe_pad_pressure = 0.0;
                if let Some(audio) = &app.audio {
                    audio.send(AudioMsg::NoteExpression(None, Some(0.0), Some(0.5), Some(0.0)));
                }
            }
        });
    });
}

fn render_neural_ddsp(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    let before = app.engine.active_patch().clone();
    let mut edited = before.clone();

    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("Neural DDSP Timbre Transfer (32-Harmonic + Filtered-Noise Decoder)");
            if ui
                .selectable_label(
                    edited.ddsp.enabled,
                    if edited.ddsp.enabled {
                        "DDSP TIMBRE TRANSFER: ON"
                    } else {
                        "DDSP TIMBRE TRANSFER: OFF"
                    },
                )
                .clicked()
            {
                edited.ddsp.enabled = !edited.ddsp.enabled;
            }
        });
        ui.label(
            "Tracks live synth pitch and loudness envelope in real time and resynthesizes the signal through a continuous latent morph between two acoustic/neural timbre embeddings (works natively in SynthCraft, SoundCraft CLAP, and VCV Rack 2).",
        );
        ui.separator();

        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.strong("Latent Timbre A Embedding:");
                for t in DdspTimbre::ALL {
                    if ui
                        .selectable_label(edited.ddsp.timbre_a == t, t.label())
                        .clicked()
                    {
                        edited.ddsp.timbre_a = t;
                    }
                }
            });
            cols[1].group(|ui| {
                ui.strong("Latent Timbre B Embedding:");
                for t in DdspTimbre::ALL {
                    if ui
                        .selectable_label(edited.ddsp.timbre_b == t, t.label())
                        .clicked()
                    {
                        edited.ddsp.timbre_b = t;
                    }
                }
            });
        });

        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            vertical_fader(ui, &pal, "DDSP MIX", &mut edited.ddsp.mix, 0.0, 1.0, "%");
            rotary_knob(ui, &pal, "A <-> B", &mut edited.ddsp.morph, 0.0, 1.0, "%");
            rotary_knob(ui, &pal, "BOW PRESS", &mut edited.ddsp.bow_pressure, 0.0, 1.0, "%");
            rotary_knob(ui, &pal, "BREATH/AIR", &mut edited.ddsp.breath_noise, 0.0, 1.0, "%");
        });
    });

    app.apply_patch_edit(&before, &edited);
}
