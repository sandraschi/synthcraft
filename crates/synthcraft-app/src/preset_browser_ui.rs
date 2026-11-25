//! Serum / Pigments-style Pro Preset Browser UX for SynthCraft (129+ Factory Presets).
//!
//! Provides live search, 12 genre categories (including Cathedral Pipe Organ), engine
//! filter pills, star favorites, musical patch mutator (+/- 15%), and 1-click load/audition.

use crate::app::SynthCraftApp;
use eframe::egui;
use synthcraft_core::preset_bank::{mutate_patch, preset_engine_badges, PRESET_CATEGORIES};

const ENGINE_FILTER_PILLS: &[&str] = &[
    "All", "M1", "808", "MSEG", "FLT2", "GRAN", "SPEC", "FM6", "UNI16", "OTT", "SHIM", "IR",
    "TAPE", "FDTD", "WDF", "DDSP", "CHRD", "VOC",
];

pub fn render_pro_preset_browser(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let total = app.engine.patches.len();

    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!("Pro Preset Browser ({total} Presets)"));
            ui.separator();
            ui.add(
                egui::TextEdit::singleline(&mut app.preset_search)
                    .desired_width(220.0)
                    .hint_text("Search name, organ, 303, DDSP..."),
            );
            if !app.preset_search.is_empty() && ui.button("Clear").clicked() {
                app.preset_search.clear();
            }
            ui.toggle_value(&mut app.preset_favorites_only, "Favorites Only");
            if ui
                .button("Mutate (+/- 15%)")
                .on_hover_text("Create a musical variation of the active patch")
                .clicked()
            {
                app.mutate_counter = app.mutate_counter.wrapping_add(1);
                let before = app.engine.active_patch().clone();
                let mutated = mutate_patch(
                    &before,
                    0.15,
                    app.mutate_counter.wrapping_mul(0x517C_C1B7_2722_0A95),
                );
                app.apply_patch_edit(&before, &mutated);
                app.status_line = format!(
                    "Mutated '{}' (+/- 15%, variation #{})",
                    app.engine.active_patch().name,
                    app.mutate_counter
                );
            }
        });

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.small("Engine:");
            for &pill in ENGINE_FILTER_PILLS {
                let selected = app.preset_engine_filter == pill;
                if ui.selectable_label(selected, pill).clicked() {
                    app.preset_engine_filter = pill.to_string();
                }
            }
        });
    });

    ui.add_space(6.0);

    // Category tabs row
    ui.horizontal_wrapped(|ui| {
        if ui
            .selectable_label(
                app.preset_category_filter == "All",
                format!("All ({total})"),
            )
            .clicked()
        {
            app.preset_category_filter = "All".to_string();
        }
        for &cat in PRESET_CATEGORIES {
            let count = app
                .engine
                .patches
                .iter()
                .filter(|p| p.category == cat)
                .count();
            if count > 0 {
                let active = app.preset_category_filter == cat;
                if ui
                    .selectable_label(active, format!("{cat} ({count})"))
                    .clicked()
                {
                    app.preset_category_filter = cat.to_string();
                }
            }
        }
    });

    ui.separator();

    let query = app.preset_search.trim().to_lowercase();
    let cat_filter = app.preset_category_filter.clone();
    let eng_filter = app.preset_engine_filter.clone();
    let fav_only = app.preset_favorites_only;
    let active_id = app.engine.active_patch_id.clone();

    let mut select_id: Option<String> = None;
    let mut toggle_fav_id: Option<String> = None;

    let matching: Vec<usize> = app
        .engine
        .patches
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            if cat_filter != "All" && p.category != cat_filter {
                return false;
            }
            if fav_only && !app.favorite_patch_ids.contains(&p.id) {
                return false;
            }
            if eng_filter != "All" && !preset_engine_badges(p).contains(&eng_filter.as_str()) {
                return false;
            }
            if !query.is_empty() {
                let hay =
                    format!("{} {} {} {}", p.id, p.name, p.category, p.camelot_key).to_lowercase();
                if !hay.contains(&query) {
                    return false;
                }
            }
            true
        })
        .map(|(i, _)| i)
        .collect();

    ui.label(
        egui::RichText::new(format!(
            "Showing {} of {} presets  --  click any row to load immediately",
            matching.len(),
            total
        ))
        .small()
        .weak(),
    );
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .id_salt("pro_preset_browser_scroll")
        .max_height(380.0)
        .show(ui, |ui| {
            egui::Grid::new("pro_preset_browser_grid")
                .striped(true)
                .num_columns(6)
                .show(ui, |ui| {
                    ui.strong("Fav");
                    ui.strong("Preset Name");
                    ui.strong("Category");
                    ui.strong("BPM / Key");
                    ui.strong("Active Engines");
                    ui.strong("Action");
                    ui.end_row();

                    for idx in matching {
                        let p = &app.engine.patches[idx];
                        let is_active = p.id == active_id;
                        let is_fav = app.favorite_patch_ids.contains(&p.id);
                        let fav_lbl = if is_fav { "[*]" } else { "[ ]" };
                        if ui.small_button(fav_lbl).clicked() {
                            toggle_fav_id = Some(p.id.clone());
                        }
                        if ui.selectable_label(is_active, &p.name).clicked() {
                            select_id = Some(p.id.clone());
                        }
                        ui.label(&p.category);
                        ui.monospace(format!("{:.0} {}", p.bpm, p.camelot_key));
                        let badges = preset_engine_badges(p).join(" ");
                        ui.monospace(badges);
                        if ui
                            .small_button(if is_active { "Active" } else { "Load" })
                            .clicked()
                        {
                            select_id = Some(p.id.clone());
                        }
                        ui.end_row();
                    }
                });
        });

    if let Some(fid) = toggle_fav_id {
        if let Some(pos) = app.favorite_patch_ids.iter().position(|x| x == &fid) {
            app.favorite_patch_ids.remove(pos);
        } else {
            app.favorite_patch_ids.push(fid);
        }
    }

    if let Some(pid) = select_id {
        if let Ok(loaded) = app.engine.select_patch(&pid) {
            app.status_line = format!("Loaded preset: {} ({})", loaded.name, loaded.category);
            app.mark_dirty();
        }
    }
}
