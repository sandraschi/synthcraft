//! 1990-style PCM Rompler, Extreme Modern DSP Workbench, Interactive 25-Key MIDI Keyboard,
//! MIDI Depot, and 1-Click Demo Showcase. (3U Eurorack skiff lives in `rack_canvas.rs`.)

use crate::app::SynthCraftApp;
use crate::controls::{rotary_knob, vertical_fader};
pub use crate::rack_canvas::{render_vcv_eurorack_canvas, CableDrag};
use eframe::egui::{self, Color32, Pos2, RichText, Sense, Stroke, Vec2};
use std::f32::consts::PI;
use std::path::PathBuf;
use synthcraft_core::{
    analyze_patch_spectrum, export_midi_smf_file, midi_to_freq_hz, write_wav_file, Command,
    ExtremeDspMode, PcmProgram,
};

/// Render Workbench 4: 1990-style PCM workstation programs & Extreme Modern DSP Generators.
pub fn render_m1_and_extreme_dsp(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    ui.heading("4. 1990-style PCM Programs & Extreme Modern DSP Forge");
    ui.label(
        "Layer 1990-style PCM workstation 16-bit 32 kHz AI2 PCM multi-samples with modern physical modeling, granular clouds, and West-Coast wavefolding.",
    );
    ui.separator();

    let mut changed = false;
    {
        let before = app.engine.active_patch().clone();
        let mut edited = before.clone();
        let p = &mut edited;
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.strong("1990-style PCM program bank (synthesized, 32 kHz character)");
                let is_off = p.m1_program.is_none();
                if ui
                    .selectable_label(is_off, "PCM Layer Off (Pure Synth Only)")
                    .clicked()
                {
                    p.m1_program = None;
                    changed = true;
                }
                for prog in PcmProgram::all() {
                    let active = p.m1_program == Some(*prog);
                    if ui.selectable_label(active, prog.display_name()).clicked() {
                        p.m1_program = Some(*prog);
                        changed = true;
                    }
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    changed |=
                        vertical_fader(ui, &pal, "PCM MIX", &mut p.m1_layer_mix, 0.0, 1.0, "%");
                    changed |=
                        rotary_knob(ui, &pal, "CUTOFF", &mut p.cutoff_hz, 40.0, 12000.0, "Hz");
                    changed |= rotary_knob(ui, &pal, "MASTER", &mut p.master_gain, 0.05, 1.0, "%");
                });
            });

            cols[1].group(|ui| {
                ui.strong("Extreme Modern DSP Generators (Physical / Granular / West-Coast / Phase-Distortion)");
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
        app.apply_patch_edit(&before, &edited);
    }
    if changed {
        app.persist_state();
    }
    render_sampler_and_fx(app, ui);

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
            app.render_midi_port_picker(ui);
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
            m1.sample_pcm(freq * t0, t0)
        } else if p.extreme_dsp != ExtremeDspMode::Off {
            p.extreme_dsp.sample_dsp(freq * t0, t0, p.extreme_morph)
        } else {
            (2.0 * PI * freq * t0).sin()
        };
        let y1 = if let Some(m1) = p.m1_program {
            m1.sample_pcm(freq * t1, t1)
        } else if p.extreme_dsp != ExtremeDspMode::Off {
            p.extreme_dsp.sample_dsp(freq * t1, t1, p.extreme_morph)
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
            "25-key on-screen keyboard (C3..C5): hold a key to play. Last note: MIDI {} ({:.1} Hz)",
            last_note,
            midi_to_freq_hz(last_note)
        ));

        let note_names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let mut pressed: Option<u8> = None;
        ui.horizontal_wrapped(|ui| {
            for midi in 48_u8..=72_u8 {
                let semitone = (midi % 12) as usize;
                let octave = (midi / 12) - 1;
                let is_black = matches!(semitone, 1 | 3 | 6 | 8 | 10);
                let label = format!("{}{}", note_names[semitone], octave);
                let is_active = app.held_ui_note == Some(midi);
                let fill = if is_active {
                    pal.arc_primary
                } else if is_black {
                    Color32::from_rgb(30, 41, 59)
                } else {
                    Color32::from_rgb(100, 116, 139)
                };
                let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).small())
                    .fill(fill);
                if ui.add(btn).is_pointer_button_down_on() {
                    pressed = Some(midi);
                }
            }
        });

        if pressed != app.held_ui_note {
            if let Some(old) = app.held_ui_note.take() {
                app.note_off(old);
            }
            if let Some(note) = pressed {
                app.held_ui_note = Some(note);
                app.note_on(note, 0.85);
                app.status_line = format!(
                    "Playing MIDI note {} ({:.1} Hz) on {}",
                    note,
                    midi_to_freq_hz(note),
                    app.engine.active_patch().name
                );
            }
        }
    });
}

/// Render Demo Showcase & 1-Click Audition Cards.
pub fn render_demo_showcase(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let green = Color32::from_rgb(34, 197, 94);
    let cyan = Color32::from_rgb(56, 189, 248);

    ui.heading("Demo Showcase: 1-Click 1990-style PCM, Extreme DSP & Acid Audition");
    ui.label(
        "Click any showcase card below to configure the Eurorack VCO/VCF, 1990 PCM workstation PCM ROM, Extreme DSP engine, and 16-step MIDI sequence in one click.",
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
                    d.m1_program.map(PcmProgram::as_str).unwrap_or("off"),
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
            "Active Demo Telemetry: Patch '{}' | PCM: {} | Extreme DSP: {} | RMS {:.2} dBFS | Peak {:.2} dBFS | Pitch {:.1} Hz",
            app.engine.active_patch().name,
            app.engine
                .active_patch()
                .m1_program
                .map(PcmProgram::display_name)
                .unwrap_or("None"),
            app.engine.active_patch().extreme_dsp.display_name(),
            rep.rms_dbfs,
            rep.peak_dbfs,
            rep.estimated_pitch_hz
        ),
    );
}

fn banks_dir() -> std::path::PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_default();
    home.join("Documents").join("SynthCraft").join("Banks")
}

/// Sample bank layer and master effects for the active patch.
fn render_sampler_and_fx(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    ui.separator();
    ui.heading("M1-Style AI2 Dual-Layer Rompler & Master Effects");
    ui.label(
        "Stack Layer A (e.g. Cembalo / Glockenspiel attack) with Layer B (e.g. Cathedral Pipe Organ / Strings sustain). Supports .sf2 (#N preset), .sfz, .wav, and note-named WAV folders.",
    );
    let mut changed = false;
    let mut load: Option<String> = None;
    let mut load_b: Option<String> = None;
    ui.horizontal(|ui| {
        ui.label("Layer A:");
        ui.add(
            egui::TextEdit::singleline(&mut app.bank_input)
                .desired_width(440.0)
                .hint_text("C:\\path\\to\\layer_a.sfz or bank.sf2#0"),
        );
        if ui.button("Load A").clicked() && !app.bank_input.trim().is_empty() {
            load = Some(app.bank_input.trim().to_string());
        }
        if ui.button("Clear A").clicked() {
            let before = app.engine.active_patch().clone();
            let mut after = before.clone();
            after.sampler.bank = None;
            app.apply_patch_edit(&before, &after);
            app.bank_status.clear();
        }
        if ui.button("List SF2 presets").clicked() {
            let spec = app.bank_input.trim();
            let path = spec.rsplit_once('#').map_or(spec, |(p, _)| p);
            match synthcraft_core::sampler::list_sf2_presets(std::path::Path::new(path)) {
                Ok(v) => app.sf2_presets = v,
                Err(e) => app.bank_status = format!("Preset list failed: {e}"),
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("Layer B:");
        ui.add(
            egui::TextEdit::singleline(&mut app.bank_b_input)
                .desired_width(440.0)
                .hint_text("Optional AI2 Combi Layer B (e.g. vcsl-pipe-organ-loud.sfz)"),
        );
        if ui.button("Load B").clicked() && !app.bank_b_input.trim().is_empty() {
            load_b = Some(app.bank_b_input.trim().to_string());
        }
        if ui.button("Clear B").clicked() {
            let before = app.engine.active_patch().clone();
            let mut after = before.clone();
            after.sampler.bank_b = None;
            app.apply_patch_edit(&before, &after);
            app.bank_b_input.clear();
        }
    });
    let dir = banks_dir();
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("Banks folder: {}", dir.display()));
        if ui.button("Create folder").clicked() {
            let _ = std::fs::create_dir_all(&dir);
        }
    });
    if let Ok(rd) = std::fs::read_dir(&dir) {
        ui.horizontal_wrapped(|ui| {
            for e in rd.filter_map(Result::ok) {
                let p = e.path();
                let ext = p
                    .extension()
                    .map(|x| x.to_string_lossy().to_ascii_lowercase())
                    .unwrap_or_default();
                if p.is_dir() || matches!(ext.as_str(), "sf2" | "sfz" | "wav") {
                    let label = p
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    if ui.button(format!("A: {label}")).clicked() {
                        load = Some(p.to_string_lossy().into_owned());
                    }
                    if ui
                        .small_button("B")
                        .on_hover_text(format!("Load {label} into AI2 Layer B"))
                        .clicked()
                    {
                        load_b = Some(p.to_string_lossy().into_owned());
                    }
                }
            }
        });
    }
    egui::CollapsingHeader::new("Free sample banks (download)")
        .id_salt("free_banks")
        .show(ui, |ui| {
            ui.label("CC0 packs, checksum verified, saved in the Banks folder. Strings are looped automatically.");
            match synthcraft_core::banks::manifest() {
                Ok(m) => {
                    let busy = app.bank_fetch_rx.is_some();
                    for b in &m.banks {
                        ui.horizontal(|ui| {
                            let mb = b.files.iter().map(|f| f.bytes).sum::<u64>() / 1_000_000;
                            ui.label(format!("{} ({} MB, {})", b.name, mb, b.license));
                            let running = app.bank_fetch_id.as_deref() == Some(b.id.as_str());
                            let label = if running { "Downloading..." } else { "Download and load" };
                            if ui.add_enabled(!busy, egui::Button::new(label)).clicked() {
                                app.bank_status = format!("Starting download of {}...", b.name);
                                app.bank_fetch_id = Some(b.id.clone());
                                app.bank_fetch_rx =
                                    Some(crate::netfetch::start(b.id.clone(), dir.clone()));
                            }
                        });
                    }
                }
                Err(e) => {
                    ui.label(format!("Bank list unavailable: {e}"));
                }
            }
        });
    if !app.sf2_presets.is_empty() {
        let base = app
            .bank_input
            .rsplit_once('#')
            .map_or(app.bank_input.clone(), |(p, _)| p.to_string());
        egui::ScrollArea::vertical()
            .max_height(140.0)
            .id_salt("sf2_presets")
            .show(ui, |ui| {
                for (i, name) in app.sf2_presets.iter().enumerate() {
                    if ui
                        .selectable_label(false, format!("{i:>3}  {name}"))
                        .clicked()
                    {
                        load = Some(format!("{base}#{i}"));
                    }
                }
            });
    }
    if let Some(spec) = load {
        app.bank_input = spec.clone();
        if app.bank_fetch_rx.is_none() {
            app.bank_status = format!("Loading {spec} ...");
            app.bank_fetch_id = Some(spec.clone());
            app.bank_fetch_rx = Some(crate::netfetch::prime(spec));
        }
    }
    if let Some(spec) = load_b {
        app.bank_b_input = spec.clone();
        let before = app.engine.active_patch().clone();
        let mut after = before.clone();
        after.sampler.bank_b = Some(spec);
        if after.sampler.mix < 0.05 {
            after.sampler.mix = 0.85;
        }
        app.apply_patch_edit(&before, &after);
    }
    if !app.bank_status.is_empty() {
        ui.label(&app.bank_status);
    }
    {
        let before = app.engine.active_patch().clone();
        let mut edited = before.clone();
        let p = &mut edited;
        ui.horizontal_wrapped(|ui| {
            changed |= rotary_knob(ui, &pal, "SAMPLE MIX", &mut p.sampler.mix, 0.0, 1.0, "%");
            changed |= rotary_knob(
                ui,
                &pal,
                "A/B BAL",
                &mut p.sampler.layer_b_mix,
                0.0,
                1.0,
                "%",
            );
            changed |= rotary_knob(
                ui,
                &pal,
                "B DETUNE",
                &mut p.sampler.layer_b_detune_cents,
                -50.0,
                50.0,
                "ct",
            );
            changed |= rotary_knob(ui, &pal, "CHORUS", &mut p.fx.chorus_mix, 0.0, 1.0, "%");
            changed |= rotary_knob(ui, &pal, "REVERB", &mut p.fx.reverb_mix, 0.0, 1.0, "%");
            changed |= rotary_knob(ui, &pal, "DECAY", &mut p.fx.reverb_decay_s, 0.3, 20.0, "s");
            changed |= rotary_knob(ui, &pal, "DAMP", &mut p.fx.reverb_damping, 0.0, 0.95, "%");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Layer B Octave:");
            for (oct, lbl) in [(-2, "32'"), (-1, "16'"), (0, "8'"), (1, "4'"), (2, "2'")] {
                if ui
                    .selectable_label(p.sampler.layer_b_octave == oct, lbl)
                    .clicked()
                {
                    p.sampler.layer_b_octave = oct;
                    changed = true;
                }
            }
            ui.separator();
            changed |= ui
                .checkbox(
                    &mut p.sampler.vintage_dac,
                    "1988 Vintage 32 kHz / 12-bit DAC Crunch",
                )
                .changed();
        });
        app.apply_patch_edit(&before, &edited);
    }
    if changed {
        app.persist_state();
    }
}
