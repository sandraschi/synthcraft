//! Comprehensive In-App Documentation & Contextual 37-Subpage Encyclopedia for SynthCraft v0.4.0.
//!
//! Single source of truth: embeds `HELP.md` and all 14 workbench sub-READMEs from
//! `docs/workbenches/*.md` via `include_str!`, rendering human-readable guides,
//! hardware lineage/provenance, famous tracks, and CLI/MCP references inside egui.

use crate::app::{HelpTab, Page, SynthCraftApp};
use eframe::egui;
use synthcraft_core::APP_VERSION;

const HELP_MD: &str = include_str!("../../../HELP.md");

const WB_DOCS: &[(&str, &str)] = &[
    (
        "1. VCV 2 Eurorack & VCO/VCF",
        include_str!("../../../docs/workbenches/01_modular_rack.md"),
    ),
    (
        "2. 1990-Style PCM & Extreme DSP",
        include_str!("../../../docs/workbenches/02_m1_extreme_dsp.md"),
    ),
    (
        "3. MIDI Keyboard & MIDI Depot",
        include_str!("../../../docs/workbenches/03_midi_depot.md"),
    ),
    (
        "4. Demo Showcase & Audition",
        include_str!("../../../docs/workbenches/04_demo_showcase.md"),
    ),
    (
        "5. Sequencer, Drums & ADSR/LFO",
        include_str!("../../../docs/workbenches/05_sequencer_env.md"),
    ),
    (
        "6. Oscilloscope & Spectrum",
        include_str!("../../../docs/workbenches/06_scope_spectrum.md"),
    ),
    (
        "7. CLAP & SoundCraft Bridge",
        include_str!("../../../docs/workbenches/07_clap_soundcraft.md"),
    ),
    (
        "8. DJCraft Stem & WAV Forge",
        include_str!("../../../docs/workbenches/08_djcraft_forge.md"),
    ),
    (
        "9. Assistant & Help",
        include_str!("../../../docs/workbenches/09_assistant_help.md"),
    ),
    (
        "10. Studio Settings & Prefs",
        include_str!("../../../docs/workbenches/10_studio_settings.md"),
    ),
    (
        "11. Next-Gen Lab (MSEG/Granular/AI)",
        include_str!("../../../docs/workbenches/11_nextgen_lab.md"),
    ),
    (
        "12. Pro Lab (6-Op FM/FX/2D Physics)",
        include_str!("../../../docs/workbenches/12_pro_lab.md"),
    ),
    (
        "13. Visuals, MPE & Neural DDSP",
        include_str!("../../../docs/workbenches/13_visuals_mpe_ddsp.md"),
    ),
    (
        "14. Performance & Mastering Suite",
        include_str!("../../../docs/workbenches/14_perf_master.md"),
    ),
];

/// Map any active `(Page, subpage)` pair to `(workbench_index_0_based, subpage_ordinal_0_based)`
/// matching the `## Subpage X.Y:` headings in `docs/workbenches/*.md`.
pub fn map_page_to_doc(page: Page, sub: usize) -> (usize, usize) {
    match page {
        Page::ModularRack => (0, sub.min(1)),
        Page::M1ExtremeDsp => (1, sub.min(1)),
        Page::MidiDepot => (2, sub.min(1)),
        Page::DemoShowcase => (3, 0),
        Page::SequencerEnv => match sub {
            0 => (4, 0),
            2 => (4, 1), // Subpage 5.2: TR-808/909 8-Voice Drum Machine
            _ => (4, 2), // Subpage 5.3: ADSR Envelope & Tempo LFO
        },
        Page::ScopeSpectrum => (5, sub.min(1)),
        Page::ClapSoundcraft => (6, sub.min(1)),
        Page::DjcraftForge => (7, sub.min(1)),
        Page::Chat => (8, 0),
        Page::Help => (8, 1),
        Page::Settings => (9, sub.min(5)),
        Page::NextGenLab => (10, sub.min(2)),
        Page::ProLab => (11, sub.min(2)),
        Page::VisualsMpeDdsp => (12, sub.min(2)),
        Page::PerfMaster => (13, sub.min(3)),
    }
}

/// Split a workbench markdown document into its individual `## Subpage ...` sections.
fn extract_subpages(md: &str) -> Vec<(&str, &str)> {
    let mut sections = Vec::new();
    let mut starts = Vec::new();
    let mut offset = 0usize;
    for line in md.split_inclusive('\n') {
        if line.starts_with("## Subpage ") {
            starts.push(offset);
        }
        offset += line.len();
    }
    for (idx, &start) in starts.iter().enumerate() {
        let end = starts.get(idx + 1).copied().unwrap_or(md.len());
        let slice = md[start..end].trim();
        let title = slice
            .lines()
            .next()
            .unwrap_or("Subpage")
            .trim_start_matches("## ")
            .trim();
        sections.push((title, slice));
    }
    sections
}

/// Render human-readable Markdown cleanly inside `egui` with high-contrast colors in both themes.
fn render_markdown(ui: &mut egui::Ui, md: &str, dark_mode: bool) {
    let heading_col = if dark_mode {
        egui::Color32::from_rgb(255, 198, 92)
    } else {
        egui::Color32::from_rgb(138, 66, 0)
    };
    let history_col = if dark_mode {
        egui::Color32::from_rgb(104, 216, 255)
    } else {
        egui::Color32::from_rgb(0, 92, 138)
    };
    let tracks_col = if dark_mode {
        egui::Color32::from_rgb(245, 146, 224)
    } else {
        egui::Color32::from_rgb(132, 24, 108)
    };

    let mut in_code = false;
    let mut code_buf = String::new();

    for raw_line in md.lines() {
        let line = raw_line.trim_end();
        if line.starts_with("```") {
            if in_code {
                ui.group(|ui| {
                    ui.monospace(code_buf.trim_end());
                });
                code_buf.clear();
                in_code = false;
            } else {
                in_code = true;
            }
            continue;
        }
        if in_code {
            code_buf.push_str(line);
            code_buf.push('\n');
            continue;
        }
        if line.is_empty() {
            ui.add_space(4.0);
        } else if line == "---" {
            ui.separator();
        } else if let Some(h1) = line.strip_prefix("# ") {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(h1)
                    .size(18.0)
                    .strong()
                    .color(heading_col),
            );
        } else if let Some(h2) = line.strip_prefix("## ") {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(h2)
                    .size(15.5)
                    .strong()
                    .color(heading_col),
            );
        } else if let Some(h3) = line.strip_prefix("### ") {
            ui.add_space(4.0);
            let col = if h3.contains("Provenance") || h3.contains("Lineage") {
                history_col
            } else if h3.contains("Famous Tracks") {
                tracks_col
            } else {
                heading_col
            };
            ui.label(egui::RichText::new(h3).size(13.5).strong().color(col));
        } else {
            let clean = line.replace("**", "");
            ui.label(clean);
        }
    }
    if in_code && !code_buf.is_empty() {
        ui.group(|ui| {
            ui.monospace(code_buf.trim_end());
        });
    }
}

/// Contextual right-side `? Subpage Guide` drawer shown alongside any of the 37 subpages.
pub fn render_subpage_help_drawer(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let (wb_idx, sub_idx) = map_page_to_doc(app.page, app.subpage);
    let (wb_title, wb_md) = WB_DOCS.get(wb_idx).copied().unwrap_or(WB_DOCS[0]);
    let subs = extract_subpages(wb_md);
    let (sub_title, sub_md) = subs.get(sub_idx).copied().unwrap_or((wb_title, wb_md));

    ui.horizontal(|ui| {
        ui.strong("Subpage Guide");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Close").clicked() {
                app.show_subpage_help = false;
            }
            if ui.button("Full Encyclopedia (F1)").clicked() {
                app.help_selected_wb = wb_idx;
                app.help_selected_sub = sub_idx;
                app.help_tab = HelpTab::SubpageEncyclopedia;
                app.show_subpage_help = false;
                app.goto(Page::Help, 0);
            }
        });
    });
    ui.label(egui::RichText::new(wb_title).small().weak());
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("subpage_help_drawer_scroll")
        .show(ui, |ui| {
            let _ = sub_title;
            render_markdown(ui, sub_md, app.dark_mode);
        });
}

/// Render the active tab inside `Page::Help` (`F1`).
pub fn render_help_tab_content(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    match app.help_tab {
        HelpTab::SubpageEncyclopedia => render_encyclopedia(app, ui),
        HelpTab::GetStarted => {
            render_markdown(ui, HELP_MD, app.dark_mode);
        }
        HelpTab::ModularRack => {
            render_markdown(ui, WB_DOCS[0].1, app.dark_mode);
            ui.separator();
            render_markdown(ui, WB_DOCS[1].1, app.dark_mode);
        }
        HelpTab::SequencerEnv => {
            render_markdown(ui, WB_DOCS[2].1, app.dark_mode);
            ui.separator();
            render_markdown(ui, WB_DOCS[4].1, app.dark_mode);
        }
        HelpTab::ClapDjcraft => {
            render_markdown(ui, WB_DOCS[6].1, app.dark_mode);
            ui.separator();
            render_markdown(ui, WB_DOCS[7].1, app.dark_mode);
        }
        HelpTab::CliMcp => render_cli_mcp_reference(ui),
        HelpTab::TechnicalInfo => {
            ui.heading("Technical Architecture, DSP Signal Chain & SQLite Schema");
            ui.add_space(4.0);
            ui.group(|ui| {
                ui.strong("1. 3-Tier Pure-Rust ArtCraft Architecture");
                ui.label(format!("Application Version: v{APP_VERSION}"));
                ui.label(format!("Active SQLite Database Path: {}", app.db_path.display()));
                ui.monospace(
                    "L1: crates/synthcraft-core  (VCO/VCF, 64-Step Seq, TR-808/909 Drums, MSEG, Granular, Spectral, Genopatch, 6-Op FM, 16-Voice Unison, FX Rack, 2D FDTD, 8x WDF, Neural DDSP, Vocoder, Mastering)\n\
                     L2: crates/synthcraft-mcp   (clap 4.5 CLI + MCP 2026-07-28 JSON-RPC 2.0 stdio server, 60 tools)\n\
                     L3: crates/synthcraft-app   (eframe/egui 0.37 14-workbench / 37-subpage studio + egui_kittest)\n\
                     Plugins: crates/synthcraft-clap (SynthCraft.clap 1.2.2) + crates/synthcraft-ffi + rack/ (VCV Rack 2.0.0) + crates/synthcraft-wasm",
                );
            });
            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. SQLite Persistence Schema (synth_patches & synth_state)");
                ui.monospace(
                    "CREATE TABLE synth_patches (id TEXT PRIMARY KEY, patch_json TEXT NOT NULL);\n\
                     CREATE TABLE synth_state (id INTEGER PRIMARY KEY CHECK (id = 1), active_patch_id TEXT NOT NULL, sample_rate_hz INTEGER NOT NULL);",
                );
            });
        }
    }
}

fn render_encyclopedia(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.strong("Search all 14 Workbenches & 37 Subpages:");
        ui.add(
            egui::TextEdit::singleline(&mut app.help_search_query)
                .hint_text("Try: 303, Daft Punk, Buchla, vocoder, OTT, MPE, DX7, 808..."),
        );
        if !app.help_search_query.is_empty() && ui.button("Clear").clicked() {
            app.help_search_query.clear();
        }
    });
    ui.add_space(6.0);

    let q = app.help_search_query.trim().to_lowercase();
    if !q.is_empty() {
        let mut hits = 0usize;
        for (wb_idx, (wb_title, wb_md)) in WB_DOCS.iter().enumerate() {
            for (sub_idx, (sub_title, sub_md)) in extract_subpages(wb_md).into_iter().enumerate() {
                if wb_title.to_lowercase().contains(&q) || sub_md.to_lowercase().contains(&q) {
                    hits += 1;
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(format!("{wb_title}  --  {sub_title}"));
                            if ui.button("Open Guide").clicked() {
                                app.help_selected_wb = wb_idx;
                                app.help_selected_sub = sub_idx;
                                app.help_search_query.clear();
                            }
                        });
                    });
                }
            }
        }
        if hits == 0 {
            ui.label(format!("No subpage guides matched '{q}'."));
        }
        return;
    }

    ui.horizontal_wrapped(|ui| {
        for (idx, (title, _)) in WB_DOCS.iter().enumerate() {
            if ui
                .selectable_label(app.help_selected_wb == idx, *title)
                .clicked()
            {
                app.help_selected_wb = idx;
                app.help_selected_sub = 0;
            }
        }
    });
    ui.separator();

    let wb_idx = app.help_selected_wb.min(WB_DOCS.len().saturating_sub(1));
    let (_, wb_md) = WB_DOCS[wb_idx];
    let subs = extract_subpages(wb_md);
    if subs.len() > 1 {
        ui.horizontal_wrapped(|ui| {
            for (s_idx, (s_title, _)) in subs.iter().enumerate() {
                if ui
                    .selectable_label(app.help_selected_sub == s_idx, *s_title)
                    .clicked()
                {
                    app.help_selected_sub = s_idx;
                }
            }
        });
        ui.separator();
    }

    let s_idx = app.help_selected_sub.min(subs.len().saturating_sub(1));
    if let Some((_, sub_md)) = subs.get(s_idx) {
        render_markdown(ui, sub_md, app.dark_mode);
    } else {
        render_markdown(ui, wb_md, app.dark_mode);
    }
}

fn render_cli_mcp_reference(ui: &mut egui::Ui) {
    ui.heading("L2 CLI Subcommands & 60 MCP 2026-07-28 Tools Reference");
    ui.label("Both the CLI (`synthcraft-mcp`) and the sandboxed MCP 2026-07-28 stdio server invoke L1 `Command::dispatch` (60 tools total).");
    ui.add_space(6.0);

    ui.group(|ui| {
        ui.strong("Core & Next-Gen MCP Tools Overview (60 Tools)");
        egui::Grid::new("help_synth_mcp_table")
            .striped(true)
            .show(ui, |ui| {
                ui.strong("MCP Tool Name");
                ui.strong("CLI Subcommand");
                ui.strong("What It Controls");
                ui.end_row();

                for (tool, cli, desc) in [
                    ("synth_status / synth_summary", "status / summary", "Engine status, active patch, BPM, Camelot key, and CLAP plugin ID"),
                    ("list_patches / load_patch / reset_factory_patches", "list-patches / load-patch / reset-factory", "Browse, recall, or restore built-in factory patches"),
                    ("update_oscillators / update_filter / update_envelope_lfo", "update-oscillators / update-filter / update-envelope-lfo", "Dual VCO waveforms/FM, 4-pole ladder filter, ADSR & tempo LFO"),
                    ("set_sequencer_step / set_sequencer_length / record_sequencer_notes", "set-step / set-sequencer-length / record-sequencer-notes", "16/32/48/64-step (4-bar) acid sequencer steps, accent, slide & live record"),
                    ("load_drum_preset", "load-drum-preset", "8-voice pure-Rust TR-808/909 drum machine kit, level & swing"),
                    ("load_pcm_program / set_extreme_dsp / set_sampler_bank / list_banks / fetch_bank", "load-m1 / set-extreme-dsp / set-sampler-bank / list-banks / fetch-bank", "1990 PCM workstation programs, Extreme DSP modes, and CC0 SFZ/SF2 banks"),
                    ("set_mod_route / clear_mod_matrix / set_arpeggiator / set_tuning", "set-mod-route / clear-mod-matrix / set-arpeggiator / set-tuning", "8-route modulation matrix, arpeggiator, and Scala .scl microtuning"),
                    ("list_rack_modules / add_rack_module / connect_rack_cable / load_rack_preset", "list-rack-modules / add-rack-module / connect-rack-cable / load-rack-preset", "13-module 3U Eurorack skiff, patch cables & 12 pre-patched Rack Rigs"),
                    ("scan_clap_plugins / inspect_clap_plugin / configure_clap_host / get_clap_host_status", "scan-clap-plugins / inspect-clap-plugin / configure-clap-host / get-clap-host-status", "3U CLAP 1.2.2 Host module for loading external .clap plugins"),
                    ("set_mseg_filter2 / set_granular_spectral / run_genopatch", "set-mseg-filter2 / set-granular-spectral / run-genopatch", "Workbench 11: 16-point MSEG, Dual Filter B, 24-grain Cloud, 64-Partial Spectral & Genopatch"),
                    ("set_fm6_unison / set_studio_fx / set_physical_wdf", "set-fm6-unison / set-studio-fx / set-physical-wdf", "Workbench 12: 6-Op Thru-Zero FM, 16-Voice Unison, OTT/IR/Shimmer/Tape FX, 2D FDTD & 8x WDF"),
                    ("set_ddsp_timbre", "set-ddsp-timbre", "Workbench 13: Neural DDSP Timbre Transfer (Violin, Flute, Sax, Cello, Trumpet, Choir)"),
                    ("set_perf_master", "set-perf-master", "Workbench 14: Chord Memory, Scale Lock, 16-Band Vocoder, 8-Scene XY Morph & Mastering Bus"),
                    ("synthesize_preview / analyze_spectrum / render_wav_file / export_clap_manifest / export_djcraft_stem / export_wasm_bundle", "preview / analyze-spectrum / render-wav / export-clap / export-djcraft-stem / export-wasm-bundle", "Offline audio preview, 32-band DFT analysis, 48 kHz WAV export, CLAP/DJCraft/WASM bridges"),
                ] {
                    ui.monospace(tool);
                    ui.monospace(cli);
                    ui.label(desc);
                    ui.end_row();
                }
            });
    });
}
