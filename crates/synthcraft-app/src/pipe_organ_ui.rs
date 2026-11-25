//! Workbench 15 UI: Cathedral Pipe Organ Console & GrandOrgue Bach MIDI Depot.
//!
//! - Subpage 15.1: 3-Division Stop Jambs (Hauptwerk / Schwellwerk / Pedalwerk, 16 tactile drawknobs),
//!   8 Combination Thumb Pistons, Couplers (SW->HW, HW->PED, SW->PED, Sub 16', Super 4'),
//!   Swell Box Venetian Shutter Pedal, Crescendo Walze, Wind-Chest Chiff, Bellows Wind Sag,
//!   Tremulant, Historical Organ Temperaments (Werckmeister III, Silbermann Meantone, Kirnberger III),
//!   and 1-click Stephansdom Cathedral IR.
//! - Subpage 15.2: GrandOrgue Bach MIDI Depot Scanner & 1-Click 64-Step Organ Repertoire Importer.

use crate::app::SynthCraftApp;
use eframe::egui;
use std::path::PathBuf;
use synthcraft_core::{
    import_midi_smf_file, scan_organ_midi_depot, IrPreset, OrganDivision, OrganTemperament,
    PipeOrganParams, ORGAN_STOPS,
};

pub fn render_pipe_organ_workbench(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    if app.subpage == 0 {
        render_organ_console(app, ui);
    } else {
        render_organ_midi_depot(app, ui);
    }
}

fn render_organ_console(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let before = app.engine.active_patch().clone();
    let mut after = before.clone();
    let mut changed = false;

    let wood_bg = if app.dark_mode {
        egui::Color32::from_rgb(38, 26, 18)
    } else {
        egui::Color32::from_rgb(245, 238, 224)
    };
    let ivory_on = egui::Color32::from_rgb(248, 236, 198);
    let stop_off = if app.dark_mode {
        egui::Color32::from_rgb(28, 22, 18)
    } else {
        egui::Color32::from_rgb(220, 210, 194)
    };

    ui.heading("Cathedral Pipe Organ Console (Werkprinzip -- 16 Stop Ranks)");
    ui.horizontal_wrapped(|ui| {
        if ui
            .checkbox(&mut after.pipe_organ.enabled, "Enable Cathedral Pipe Organ")
            .changed()
        {
            changed = true;
        }
        ui.separator();
        ui.label("Organ Mix:");
        if ui
            .add(egui::Slider::new(&mut after.pipe_organ.mix, 0.0..=1.0).text("Mix"))
            .changed()
        {
            changed = true;
        }
        if ui.button("Engage Stephansdom Cathedral IR").clicked() {
            after.pipe_organ.enabled = true;
            after.studio_fx.enabled = true;
            after.studio_fx.conv_enabled = true;
            after.studio_fx.conv_preset = IrPreset::StephansdomCathedral;
            after.studio_fx.conv_mix = 0.58;
            changed = true;
            app.status_line = "Engaged Stephansdom Cathedral Convolution IR Reverb".into();
        }
        ui.separator();
        ui.strong(format!(
            "Active Stops: {}/16  |  Piston: {}",
            after.pipe_organ.active_stop_count(),
            after.pipe_organ.piston_name
        ));
    });

    ui.add_space(6.0);

    // 8 Combination Thumb Pistons
    ui.group(|ui| {
        ui.strong("Combination Thumb Pistons (Instant Registrations)");
        ui.horizontal_wrapped(|ui| {
            for &(pid, label) in PipeOrganParams::piston_list() {
                if ui.button(label).clicked() {
                    after.pipe_organ.enabled = pid != "general_cancel";
                    after.pipe_organ.load_piston(pid);
                    changed = true;
                    app.status_line = format!("Organ Piston: {label}");
                }
            }
        });
    });

    ui.add_space(6.0);

    // 3-Division Stop Jambs (Hauptwerk / Schwellwerk / Pedalwerk)
    egui::Frame::group(ui.style())
        .fill(wood_bg)
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.columns(3, |cols| {
                let divisions = [
                    (OrganDivision::Hauptwerk, "I. HAUPTWERK (Great)"),
                    (OrganDivision::Schwellwerk, "II. SCHWELLWERK (Swell)"),
                    (OrganDivision::Pedalwerk, "PEDALWERK (Pedal)"),
                ];
                for (col_idx, (div, title)) in divisions.iter().enumerate() {
                    cols[col_idx].strong(*title);
                    cols[col_idx].separator();
                    for stop in ORGAN_STOPS.iter().filter(|s| s.division == *div) {
                        let on = after.pipe_organ.stops[stop.index];
                        let txt = if on {
                            egui::RichText::new(format!("[ON]  {}", stop.name))
                                .strong()
                                .color(egui::Color32::from_rgb(30, 18, 8))
                        } else {
                            egui::RichText::new(format!("[  ]  {}", stop.name))
                        };
                        let btn = egui::Button::new(txt)
                            .fill(if on { ivory_on } else { stop_off })
                            .min_size(egui::vec2(cols[col_idx].available_width().min(220.0), 26.0));
                        if cols[col_idx].add(btn).clicked() {
                            after.pipe_organ.stops[stop.index] = !on;
                            after.pipe_organ.enabled = true;
                            changed = true;
                        }
                    }
                }
            });
        });

    ui.add_space(6.0);

    // Couplers, Expression Pedals, Wind Physics & Historical Temperaments
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.strong("Couplers & Expression Pedals");
            ui.horizontal_wrapped(|ui| {
                changed |= ui
                    .checkbox(&mut after.pipe_organ.coupler_sw_to_hw, "SW -> HW")
                    .changed();
                changed |= ui
                    .checkbox(&mut after.pipe_organ.coupler_hw_to_ped, "HW -> PED")
                    .changed();
                changed |= ui
                    .checkbox(&mut after.pipe_organ.coupler_sw_to_ped, "SW -> PED")
                    .changed();
                changed |= ui
                    .checkbox(&mut after.pipe_organ.coupler_sub_16, "Sub 16'")
                    .changed();
                changed |= ui
                    .checkbox(&mut after.pipe_organ.coupler_super_4, "Super 4'")
                    .changed();
            });
            ui.add_space(4.0);
            changed |= ui
                .add(
                    egui::Slider::new(&mut after.pipe_organ.swell_pedal, 0.0..=1.0)
                        .text("Swell Box Shutters"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut after.pipe_organ.crescendo_pedal, 0.0..=1.0)
                        .text("Crescendo Walze (pp -> Tutti)"),
                )
                .changed();
        });

        cols[1].group(|ui| {
            ui.strong("Wind-Chest Physics, Tremulant & Historical Temperament");
            changed |= ui
                .add(
                    egui::Slider::new(&mut after.pipe_organ.chiff_amount, 0.0..=1.0)
                        .text("Wind-Chest Chiff (Pipe Speech)"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut after.pipe_organ.wind_sag, 0.0..=1.0)
                        .text("Bellows Wind Pressure Sag"),
                )
                .changed();
            ui.horizontal(|ui| {
                changed |= ui
                    .checkbox(&mut after.pipe_organ.tremulant_enabled, "Tremulant")
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut after.pipe_organ.tremulant_rate_hz, 2.5..=8.5)
                            .text("Hz"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut after.pipe_organ.tremulant_depth, 0.0..=1.0)
                            .text("Depth"),
                    )
                    .changed();
            });
            egui::ComboBox::from_label("Temperament:")
                .selected_text(after.pipe_organ.temperament.label())
                .show_ui(ui, |ui| {
                    for t in OrganTemperament::ALL {
                        if ui
                            .selectable_value(&mut after.pipe_organ.temperament, t, t.label())
                            .changed()
                        {
                            changed = true;
                        }
                    }
                });
        });
    });

    if changed {
        app.apply_patch_edit(&before, &after);
    }
}

fn render_organ_midi_depot(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    ui.heading("GrandOrgue Bach MIDI Depot & Organ Repertoire Importer");
    ui.label(
        "Scan your local GrandOrgue / J.S. Bach `.mid` repertoire folder and import any chorale, prelude, fugue, or toccata directly into SynthCraft's 64-step sequencer with Cathedral Pipe Organ registration.",
    );
    ui.add_space(6.0);

    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label("MIDI Depot Folder:");
            ui.add(
                egui::TextEdit::singleline(&mut app.organ_midi_dir)
                    .desired_width(360.0)
                    .hint_text(r"D:\Dev\repos\grandorgue-mcp\midi_depot"),
            );
            if ui.button("Scan Folder").clicked() {
                let dir = PathBuf::from(app.organ_midi_dir.trim());
                app.organ_midi_entries = scan_organ_midi_depot(&dir);
                app.status_line = format!(
                    "Found {} organ .mid files in {}",
                    app.organ_midi_entries.len(),
                    dir.display()
                );
            }
            ui.separator();
            ui.add(
                egui::TextEdit::singleline(&mut app.organ_midi_filter)
                    .desired_width(180.0)
                    .hint_text("Filter (e.g. jesu, air, prelud)..."),
            );
        });
    });

    ui.add_space(6.0);

    let q = app.organ_midi_filter.trim().to_lowercase();
    let entries: Vec<_> = app
        .organ_midi_entries
        .iter()
        .filter(|e| q.is_empty() || e.title.to_lowercase().contains(&q))
        .cloned()
        .collect();

    if entries.is_empty() {
        ui.label("No `.mid` files loaded yet -- click 'Scan Folder' above to scan your GrandOrgue MIDI Depot.");
        return;
    }

    ui.label(format!(
        "Showing {} organ MIDI pieces (click Import & Play to load into 64-step organ sequencer):",
        entries.len()
    ));

    let mut import_target: Option<(String, PathBuf)> = None;
    egui::ScrollArea::vertical()
        .id_salt("organ_midi_depot_scroll")
        .max_height(380.0)
        .show(ui, |ui| {
            egui::Grid::new("organ_midi_depot_grid")
                .striped(true)
                .num_columns(3)
                .show(ui, |ui| {
                    ui.strong("Piece / File");
                    ui.strong("Size");
                    ui.strong("Action");
                    ui.end_row();

                    for entry in &entries {
                        ui.label(&entry.title);
                        ui.monospace(format!("{} B", entry.bytes));
                        if ui.button("Import to Organ").clicked() {
                            import_target = Some((entry.title.clone(), entry.path.clone()));
                        }
                        ui.end_row();
                    }
                });
        });

    if let Some((title, path)) = import_target {
        match import_midi_smf_file(&path) {
            Ok(imported) => {
                let before = app.engine.active_patch().clone();
                let mut after = before.clone();
                after.pipe_organ.enabled = true;
                after.pipe_organ.mix = 0.92;
                after.studio_fx.enabled = true;
                after.studio_fx.conv_enabled = true;
                after.studio_fx.conv_preset = IrPreset::StephansdomCathedral;
                after.studio_fx.conv_mix = 0.55;
                after.bpm = imported.bpm.unwrap_or(92.0).clamp(55.0, 180.0);
                after.steps = imported.steps;
                app.apply_patch_edit(&before, &after);
                app.status_line = format!(
                    "Imported Bach/Organ piece '{title}' ({} steps @ {:.0} BPM) into Cathedral Pipe Organ!",
                    after.steps.len(),
                    after.bpm
                );
            }
            Err(e) => {
                app.status_line = format!("MIDI import failed for '{title}': {e}");
            }
        }
    }
}
