//! VCV Rack 2-Style 3U Eurorack & Patch-Cable Canvas, 1990 Korg M1 AI2 PCM Rompler,
//! Extreme Modern DSP Workbench, Interactive 25-Key MIDI Keyboard, MIDI Depot,
//! and 1-Click Demo Showcase.

use crate::app::SynthCraftApp;
use crate::controls::{rotary_knob, vertical_fader};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::path::PathBuf;
use synthcraft_core::{
    analyze_patch_spectrum, export_midi_smf_file, midi_to_freq_hz, write_wav_file, Command,
    ExtremeDspMode, KorgM1Program,
};

/// Render the VCV Rack 2-style 3U Eurorack module & draped patch-cable canvas.
pub fn render_vcv_eurorack_canvas(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    let green = pal.arc_primary;
    let cyan = pal.arc_secondary;
    let amber = pal.led_accent;
    let purple = Color32::from_rgb(168, 85, 247);
    let red = Color32::from_rgb(239, 68, 68);

    ui.strong("VCV Rack 2-Style 3U Eurorack Modular Bay (Live Catenary Patch Cables)");
    let avail_w = ui.available_width().max(720.0);
    let rack_h = 275.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, rack_h), Sense::hover());
    let painter = ui.painter_at(rect);

    // 1. Anodized 3U rails top and bottom
    painter.rect_filled(rect, 8.0, pal.panel_bg);

    let top_rail = Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.min.y + 14.0));
    let bot_rail = Rect::from_min_max(Pos2::new(rect.min.x, rect.max.y - 14.0), rect.max);
    painter.rect_filled(top_rail, 3.0, pal.rail_col);
    painter.rect_filled(bot_rail, 3.0, pal.rail_col);

    // 2. Six 3U Eurorack Modules across the bay
    let ap = app.engine.active_patch().clone();
    let m1_name = ap.m1_program.map(KorgM1Program::as_str).unwrap_or("OFF");
    let modules = [
        (
            "VCO-1",
            ap.vco1_wave.as_str().to_uppercase(),
            vec![
                ("FREQ", (ap.bpm / 200.0).clamp(0.1, 1.0)),
                ("PWM", ap.pulse_width),
            ],
            vec![("FM CV", false), ("SAW OUT", true)],
        ),
        (
            "VCO-2 / X-DSP",
            ap.extreme_dsp.as_str().to_uppercase(),
            vec![
                (
                    "DETUNE",
                    ((ap.vco2_detune_cents + 50.0) / 100.0).clamp(0.0, 1.0),
                ),
                ("MORPH", ap.extreme_morph),
            ],
            vec![("SYNC IN", false), ("DSP OUT", true)],
        ),
        (
            "KORG-M1-PCM",
            m1_name.to_uppercase(),
            vec![("ROM MIX", ap.m1_layer_mix), ("32kHz DAC", 0.85)],
            vec![("GATE IN", false), ("AI2 ROM OUT", true)],
        ),
        (
            "VCF-LADDER",
            ap.filter_mode.as_str().to_uppercase(),
            vec![
                ("CUTOFF", (ap.cutoff_hz / 12000.0).clamp(0.05, 1.0)),
                ("RES Q", (ap.resonance_q / 12.0).clamp(0.05, 1.0)),
            ],
            vec![
                ("AUDIO IN", false),
                ("AUX IN", false),
                ("CUTOFF CV", false),
                ("LP24 OUT", true),
            ],
        ),
        (
            "LFO / ADSR",
            format!("{:.1} Hz", ap.lfo_rate_hz),
            vec![
                ("LFO DEPTH", ap.lfo_depth),
                ("DECAY", (ap.decay_ms / 2000.0).clamp(0.05, 1.0)),
            ],
            vec![("SINE OUT", true), ("ENV OUT", true)],
        ),
        (
            "VCA-MASTER",
            format!("{} KEY", ap.camelot_key),
            vec![
                ("DRIVE", (ap.drive_saturation / 3.0).clamp(0.05, 1.0)),
                ("MASTER", ap.master_gain),
            ],
            vec![("SIGNAL IN", false), ("GAIN CV", false)],
        ),
    ];

    let mod_w = (rect.width() - 28.0) / (modules.len() as f32);
    let mut jack_positions: HashMap<(String, String), Pos2> = HashMap::new();

    for (idx, (m_title, subtitle, knobs, jacks)) in modules.iter().enumerate() {
        let x0 = rect.left() + 10.0 + (idx as f32) * mod_w;
        let mod_rect = Rect::from_min_max(
            Pos2::new(x0 + 2.0, rect.top() + 16.0),
            Pos2::new(x0 + mod_w - 4.0, rect.bottom() - 16.0),
        );
        painter.rect_filled(mod_rect, 4.0, pal.module_bg);
        painter.rect_stroke(
            mod_rect,
            4.0,
            Stroke::new(1.5, pal.module_border),
            egui::StrokeKind::Outside,
        );

        // Four corner Eurorack mounting screws
        for sx in [mod_rect.left() + 8.0, mod_rect.right() - 8.0] {
            for sy in [mod_rect.top() + 7.0, mod_rect.bottom() - 7.0] {
                painter.circle_filled(Pos2::new(sx, sy), 2.5, pal.text_muted);
            }
        }

        // Module header & subtitle
        painter.text(
            Pos2::new(mod_rect.center().x, mod_rect.top() + 16.0),
            egui::Align2::CENTER_CENTER,
            *m_title,
            egui::FontId::monospace(11.5),
            cyan,
        );
        painter.text(
            Pos2::new(mod_rect.center().x, mod_rect.top() + 31.0),
            egui::Align2::CENTER_CENTER,
            subtitle,
            egui::FontId::monospace(9.5),
            green,
        );

        // Rotary potentiometer dials with glowing value arcs
        for (kidx, (k_label, norm_val)) in knobs.iter().enumerate() {
            let ky = mod_rect.top() + 68.0 + (kidx as f32) * 58.0;
            let center = Pos2::new(mod_rect.center().x, ky);
            painter.circle_filled(center, 16.0, pal.knob_outer);
            painter.circle_stroke(center, 16.0, Stroke::new(2.0, pal.arc_track));
            painter.circle_filled(center, 12.0, pal.knob_cap);

            // 270-degree LED value arc
            let start_ang = PI * 0.75;
            let sweep = PI * 1.5 * norm_val.clamp(0.0, 1.0);
            let arc_pts: Vec<Pos2> = (0..=16)
                .map(|s| {
                    let a = start_ang + (s as f32 / 16.0) * sweep;
                    Pos2::new(center.x + a.cos() * 16.5, center.y + a.sin() * 16.5)
                })
                .collect();
            painter.add(egui::Shape::line(
                arc_pts,
                Stroke::new(2.4, pal.arc_primary),
            ));

            let angle = start_ang + sweep;
            let tip = Pos2::new(center.x + angle.cos() * 12.0, center.y + angle.sin() * 12.0);
            painter.line_segment([center, tip], Stroke::new(2.2, pal.knob_pointer));
            painter.text(
                Pos2::new(center.x, ky + 23.0),
                egui::Align2::CENTER_CENTER,
                *k_label,
                egui::FontId::monospace(9.0),
                pal.text_primary,
            );
        }

        // 3.5mm Eurorack Jack Sockets at bottom of faceplate
        let j_count = jacks.len().max(1);
        for (jidx, (j_name, is_out)) in jacks.iter().enumerate() {
            let col_idx = jidx % 2;
            let row_idx = jidx / 2;
            let jx = if j_count == 1 {
                mod_rect.center().x
            } else if col_idx == 0 {
                mod_rect.left() + mod_rect.width() * 0.28
            } else {
                mod_rect.left() + mod_rect.width() * 0.72
            };
            let jy = mod_rect.bottom() - 48.0 + (row_idx as f32) * 24.0;
            let jpos = Pos2::new(jx, jy);
            let ring_col = if *is_out { green } else { cyan };
            painter.circle_filled(jpos, 6.5, pal.knob_outer);
            painter.circle_stroke(jpos, 6.5, Stroke::new(2.0, ring_col));
            painter.circle_filled(jpos, 2.5, Color32::BLACK);
            painter.text(
                Pos2::new(jx, jy + 10.5),
                egui::Align2::CENTER_CENTER,
                *j_name,
                egui::FontId::monospace(7.5),
                pal.text_primary,
            );
            jack_positions.insert(((*m_title).to_string(), (*j_name).to_string()), jpos);
            // Also index LFO-MOD and ADSR-ENV aliases to the combined LFO / ADSR faceplate
            if *m_title == "LFO / ADSR" {
                jack_positions.insert(("LFO-MOD".to_string(), (*j_name).to_string()), jpos);
                jack_positions.insert(("ADSR-ENV".to_string(), (*j_name).to_string()), jpos);
            }
        }
    }

    // 3. Draw catenary / drooping Bezier patch cables between connected 3.5mm jacks
    for cab in &app.engine.rack_cables {
        if let (Some(&p1), Some(&p2)) = (
            jack_positions.get(&(cab.from_module.clone(), cab.from_jack.clone())),
            jack_positions.get(&(cab.to_module.clone(), cab.to_jack.clone())),
        ) {
            let col = match cab.color_name.as_str() {
                "green" => green,
                "amber" => amber,
                "purple" => purple,
                "red" => red,
                _ => cyan,
            };
            let sag = 34.0 + ((p2.x - p1.x).abs() * 0.12);
            let ctrl = Pos2::new((p1.x + p2.x) * 0.5, p1.y.max(p2.y) + sag);
            let steps = 18;
            let mut prev = p1;
            for s in 1..=steps {
                let t = (s as f32) / (steps as f32);
                let inv = 1.0 - t;
                let pt = Pos2::new(
                    inv * inv * p1.x + 2.0 * inv * t * ctrl.x + t * t * p2.x,
                    inv * inv * p1.y + 2.0 * inv * t * ctrl.y + t * t * p2.y,
                );
                painter.line_segment([prev, pt], Stroke::new(3.0, col));
                prev = pt;
            }
            painter.circle_filled(p1, 4.0, col);
            painter.circle_filled(p2, 4.0, col);
        }
    }
}

/// Render Workbench 4: 1990 Korg M1 AI2 PCM Workstation Rompler & Extreme Modern DSP Generators.
pub fn render_m1_and_extreme_dsp(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    ui.heading("4. Korg M1 (1990) AI2 PCM Rompler & Extreme Modern DSP Forge");
    ui.label(
        "Layer authentic 1990 Korg M1 16-bit 32 kHz AI2 PCM multi-samples with modern physical modeling, granular clouds, and West-Coast wavefolding.",
    );
    ui.separator();

    let mut changed = false;
    {
        let p = app.engine.active_patch_mut();
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.strong("1990 Korg M1 AI2 PCM Sample ROM Bank (MB87404 32 kHz DAC)");
                let is_off = p.m1_program.is_none();
                if ui
                    .selectable_label(is_off, "M1 Layer Off (Pure Synth Only)")
                    .clicked()
                {
                    p.m1_program = None;
                    changed = true;
                }
                for prog in KorgM1Program::all() {
                    let active = p.m1_program == Some(*prog);
                    if ui.selectable_label(active, prog.display_name()).clicked() {
                        p.m1_program = Some(*prog);
                        changed = true;
                    }
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    changed |=
                        vertical_fader(ui, &pal, "M1 ROM MIX", &mut p.m1_layer_mix, 0.0, 1.0, "%");
                    changed |=
                        rotary_knob(ui, &pal, "CUTOFF", &mut p.cutoff_hz, 40.0, 12000.0, "Hz");
                    changed |= rotary_knob(ui, &pal, "MASTER", &mut p.master_gain, 0.05, 1.0, "%");
                });
            });

            cols[1].group(|ui| {
                ui.strong("Extreme Modern DSP Generators (Physical / Granular / Buchla / CZ)");
                for mode in ExtremeDspMode::all() {
                    let active = p.extreme_dsp == *mode;
                    if ui.selectable_label(active, mode.display_name()).clicked() {
                        p.extreme_dsp = *mode;
                        changed = true;
                    }
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    changed |=
                        rotary_knob(ui, &pal, "X-MORPH", &mut p.extreme_morph, 0.0, 1.0, "%");
                    changed |= rotary_knob(ui, &pal, "FM INDEX", &mut p.fm_index, 0.0, 4.0, "idx");
                    changed |=
                        vertical_fader(ui, &pal, "DRIVE", &mut p.drive_saturation, 0.0, 3.0, "x");
                });
            });
        });
    }
    if changed {
        app.persist_state();
    }

    if app.subpage == 1 {
        ui.add_space(6.0);
        let rep = analyze_patch_spectrum(app.engine.active_patch(), app.engine.sample_rate_hz, 200);
        ui.group(|ui| {
            ui.strong(format!(
                "Live M1 + Extreme DSP Acoustic Signature: RMS {:.2} dBFS | Peak {:.2} dBFS | Fundamental {:.1} Hz",
                rep.rms_dbfs, rep.peak_dbfs, rep.estimated_pitch_hz
            ));
        });
    }
}

/// Render Workbench 5: Hardware MIDI Keyboard Setup, 25-Key Interactive Piano, and MIDI Pattern Depot.
pub fn render_midi_and_depot(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let green = Color32::from_rgb(34, 197, 94);
    let cyan = Color32::from_rgb(56, 189, 248);

    ui.heading("5. Hardware MIDI Controller, 25-Key Studio Piano & MIDI Pattern Depot");
    ui.separator();

    if app.subpage == 1 {
        ui.group(|ui| {
            ui.strong("Built-In Classic & Club MIDI Pattern Depot (1-Click Load or .MID Export)");
            let clips = app.engine.midi_depot.clone();
            let mut load_id: Option<String> = None;
            let mut export_id: Option<String> = None;

            egui::Grid::new("midi_depot_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Clip ID");
                    ui.strong("Title");
                    ui.strong("Genre");
                    ui.strong("BPM / Key");
                    ui.strong("Recommended Engine");
                    ui.strong("Actions");
                    ui.end_row();

                    for c in &clips {
                        ui.monospace(&c.id);
                        ui.label(&c.title);
                        ui.label(&c.genre);
                        ui.monospace(format!("{:.0} BPM ({})", c.bpm, c.camelot_key));
                        ui.monospace(format!(
                            "M1: {} | DSP: {}",
                            c.recommended_m1.as_deref().unwrap_or("none"),
                            c.recommended_extreme_dsp
                        ));
                        ui.horizontal(|ui| {
                            if ui.button(format!("Load {}", c.id)).clicked() {
                                load_id = Some(c.id.clone());
                            }
                            if ui.button(format!("Export {}.mid", c.id)).clicked() {
                                export_id = Some(c.id.clone());
                            }
                        });
                        ui.end_row();
                    }
                });

            if let Some(cid) = load_id {
                let _ = Command::LoadMidiDepotClip {
                    clip_id: cid.clone(),
                }
                .dispatch(&mut app.engine);
                app.status_line = format!("Loaded MIDI Depot clip '{cid}' into 16-step sequencer");
                app.persist_state();
            }
            if let Some(eid) = export_id {
                if let Some(clip) = app.engine.midi_depot.iter().find(|c| c.id == eid) {
                    let path = PathBuf::from(format!("{eid}.mid"));
                    if let Ok(bytes) = export_midi_smf_file(clip, &path) {
                        app.status_line = format!(
                            "Exported Standard MIDI File ({bytes} bytes) to {}",
                            path.display()
                        );
                    }
                }
            }
        });
        return;
    }

    // Subpage 0: Hardware MIDI Keyboard Connection & Interactive 25-Key Piano
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.strong("Hardware USB / DIN MIDI Keyboard Connection");
            let ports = app.engine.midi_hw.available_ports.clone();
            egui::ComboBox::from_label("MIDI Input Device")
                .selected_text(&app.engine.midi_hw.selected_port)
                .show_ui(ui, |ui| {
                    for port in &ports {
                        ui.selectable_value(
                            &mut app.engine.midi_hw.selected_port,
                            port.clone(),
                            port,
                        );
                    }
                });
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut app.engine.midi_hw.midi_channel, 0..=16)
                        .text("MIDI Channel (0 = Omni)"),
                );
                ui.add(
                    egui::Slider::new(&mut app.engine.midi_hw.pitch_bend_semitones, 1..=12)
                        .text("Pitch Bend (+/- st)"),
                );
            });
            ui.small(
                "Tip: Plug any class-compliant USB MIDI keyboard in (Arturia, Akai, Korg, NI, Novation) or load dev.sandraschi.synthcraft as a CLAP 1.2.2 instrument inside SoundCraft / Bitwig / REAPER.",
            );
        });

        cols[1].group(|ui| {
            ui.strong("MIDI Continuous Controller (CC) Map");
            for (cc, desc) in &app.engine.midi_hw.cc_mappings {
                ui.monospace(format!("CC#{cc:02} : {desc}"));
            }
        });
    });

    ui.add_space(8.0);
    render_piano_keyboard_strip(app, ui);

    // Live single-note waveform preview for the triggered key
    let freq = midi_to_freq_hz(app.engine.midi_hw.last_triggered_note);
    let p = app.engine.active_patch().clone();
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width().max(520.0), 85.0),
        Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, app.skin.palette().lcd_bg);
    let mid_y = rect.center().y;
    let steps = 96;
    let step_x = (rect.width() - 16.0) / (steps as f32);
    for i in 0..steps {
        let t0 = (i as f32) / 16000.0;
        let t1 = ((i + 1) as f32) / 16000.0;
        let y0 = if let Some(m1) = p.m1_program {
            m1.sample_pcm(freq, t0)
        } else if p.extreme_dsp != ExtremeDspMode::Off {
            p.extreme_dsp.sample_dsp(freq, t0, p.extreme_morph)
        } else {
            (2.0 * PI * freq * t0).sin()
        };
        let y1 = if let Some(m1) = p.m1_program {
            m1.sample_pcm(freq, t1)
        } else if p.extreme_dsp != ExtremeDspMode::Off {
            p.extreme_dsp.sample_dsp(freq, t1, p.extreme_morph)
        } else {
            (2.0 * PI * freq * t1).sin()
        };
        painter.line_segment(
            [
                Pos2::new(rect.left() + 8.0 + (i as f32) * step_x, mid_y - y0 * 32.0),
                Pos2::new(
                    rect.left() + 8.0 + ((i + 1) as f32) * step_x,
                    mid_y - y1 * 32.0,
                ),
            ],
            Stroke::new(2.0, cyan),
        );
    }
    let _ = green;
}

/// Reusable 25-Key On-Screen Studio Piano Keyboard (C3..C5) used across workbenches and layouts.
pub fn render_piano_keyboard_strip(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    ui.group(|ui| {
        let last_note = app.engine.midi_hw.last_triggered_note;
        ui.strong(format!(
            "Interactive 25-Key On-Screen Studio Keyboard (C3..C5) -- Last Triggered: MIDI {} ({:.1} Hz)",
            last_note,
            midi_to_freq_hz(last_note)
        ));

        let note_names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let mut triggered: Option<u8> = None;
        ui.horizontal_wrapped(|ui| {
            for midi in 48_u8..=72_u8 {
                let semitone = (midi % 12) as usize;
                let octave = (midi / 12) - 1;
                let is_black = matches!(semitone, 1 | 3 | 6 | 8 | 10);
                let label = format!("{}{}", note_names[semitone], octave);
                let is_active = app.engine.midi_hw.last_triggered_note == midi;
                let fill = if is_active {
                    pal.arc_primary
                } else if is_black {
                    Color32::from_rgb(30, 41, 59)
                } else {
                    Color32::from_rgb(100, 116, 139)
                };
                let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).small())
                    .fill(fill);
                if ui.add(btn).clicked() {
                    triggered = Some(midi);
                }
            }
        });

        if let Some(note) = triggered {
            app.engine.midi_hw.last_triggered_note = note;
            if let Some(first_step) = app.engine.active_patch_mut().steps.first_mut() {
                first_step.midi_note = note;
            }
            app.status_line = format!(
                "Triggered MIDI Note {} ({:.1} Hz) on {}",
                note,
                midi_to_freq_hz(note),
                app.engine.active_patch().name
            );
            app.persist_state();
        }
    });
}

/// Render Demo Showcase & 1-Click Audition Cards.
pub fn render_demo_showcase(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let green = Color32::from_rgb(34, 197, 94);
    let cyan = Color32::from_rgb(56, 189, 248);

    ui.heading("Demo Showcase: 1-Click Korg M1 (1990), Extreme DSP & Acid Audition");
    ui.label(
        "Click any showcase card below to configure the Eurorack VCO/VCF, 1990 Korg M1 PCM ROM, Extreme DSP engine, and 16-step MIDI sequence in one click.",
    );
    ui.separator();

    let demos = app.engine.demos.clone();
    let mut trigger_demo: Option<String> = None;
    let mut render_demo_wav: Option<String> = None;

    for d in &demos {
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(cyan, RichText::new(&d.title).strong());
                ui.monospace(format!(
                    "| {:.0} BPM | M1: {} | Extreme DSP: {} | MIDI: {}",
                    d.bpm,
                    d.m1_program.map(KorgM1Program::as_str).unwrap_or("off"),
                    d.extreme_dsp.as_str(),
                    d.midi_clip_id
                ));
                if ui.button(format!("Load & Audition {}", d.id)).clicked() {
                    trigger_demo = Some(d.id.clone());
                }
                if ui.button(format!("Render {}.wav", d.id)).clicked() {
                    render_demo_wav = Some(d.id.clone());
                }
            });
            ui.label(&d.description);
        });
        ui.add_space(4.0);
    }

    if let Some(did) = trigger_demo {
        let cmd = Command::RunDemoShowcase {
            demo_id: did.clone(),
        };
        if let Ok(res) = cmd.dispatch(&mut app.engine) {
            app.status_line = format!(
                "Auditioning Demo '{did}' -> RMS {} dBFS, Peak {} dBFS",
                res["rms_dbfs"], res["peak_dbfs"]
            );
            app.persist_state();
        }
    }

    if let Some(wid) = render_demo_wav {
        let _ = Command::RunDemoShowcase {
            demo_id: wid.clone(),
        }
        .dispatch(&mut app.engine);
        let out_path = PathBuf::from(format!("{wid}.wav"));
        if let Ok(samples) = write_wav_file(
            app.engine.active_patch(),
            &out_path,
            app.engine.sample_rate_hz,
            1500,
        ) {
            app.status_line = format!(
                "Rendered Demo Showcase '{wid}' ({samples} samples) to {}",
                out_path.display()
            );
            app.persist_state();
        }
    }

    ui.add_space(6.0);
    let rep = analyze_patch_spectrum(app.engine.active_patch(), app.engine.sample_rate_hz, 200);
    ui.colored_label(
        green,
        format!(
            "Active Demo Telemetry: Patch '{}' | M1 ROM: {} | Extreme DSP: {} | RMS {:.2} dBFS | Peak {:.2} dBFS | Pitch {:.1} Hz",
            app.engine.active_patch().name,
            app.engine
                .active_patch()
                .m1_program
                .map(KorgM1Program::display_name)
                .unwrap_or("None"),
            app.engine.active_patch().extreme_dsp.display_name(),
            rep.rms_dbfs,
            rep.peak_dbfs,
            rep.estimated_pitch_hz
        ),
    );
}
