//! Comprehensive 6-Tab In-App Documentation for SynthCraft v0.3.0.

use crate::app::{HelpTab, SynthCraftApp};
use eframe::egui;
use synthcraft_core::APP_VERSION;

const HELP_MD: &str = include_str!("../../../HELP.md");

pub fn render_help_tab_content(app: &SynthCraftApp, ui: &mut egui::Ui) {
    match app.help_tab {
        HelpTab::GetStarted => {
            ui.label(HELP_MD);
        }
        HelpTab::ModularRack => {
            ui.heading(
                "Workbenches 1 & 2: 3U Eurorack, 1990 PCM workstation Rompler & Extreme DSP",
            );
            ui.label(
                "SynthCraft combines a visual 3U Eurorack bay (with drooping catenary patch cables) alongside a 1990-style PCM sample player and 6 Extreme Modern DSP algorithms.",
            );
            ui.add_space(6.0);

            ui.group(|ui| {
                ui.strong("1. 1990-style PCM program bank (32 kHz 16-Bit DAC)");
                egui::Grid::new("help_m1_rom_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Program ID");
                        ui.strong("1990-style PCM Program");
                        ui.strong("AI2 PCM Transient & Harmonic Architecture");
                        ui.end_row();

                        for (id, name, desc) in [
                            ("m1_00_universe", "Prog 00: Universe", "glass bell attack transient + detuned choir formant pad"),
                            ("m1_01_piano16", "Prog 01: Piano 16'", "Iconic 1990 Ibiza house/rave piano: bright hammer strike + stretched piano loop"),
                            ("m1_17_organ2", "Prog 17: Organ 2", "Percussive 3rd-harmonic (8'+4'+2 2/3') drawbar organ bass + 8ms key-click"),
                            ("m1_11_fretless", "Prog 11: Fretless Bass", "Pitch-envelope attack scoop + warm mwah harmonics"),
                            ("m1_12_slapbass", "Prog 12: Slap Bass", "Metallic thumb-pop transient + slap funk decay"),
                            ("m1_24_panflute", "Prog 24: Pan Flute", "Breathy chiff noise burst + odd-harmonic bamboo pipe with 5.2 Hz vibrato"),
                            ("m1_36_choir", "Prog 36: Ooh/Aah Choir", "3-formant vocal filter bank (F1=720 Hz, F2=1240 Hz, F3=2600 Hz)"),
                            ("m1_48_orchhit", "Prog 48: Orchestra Hit", "Stacked minor-triad staccato orchestra stab"),
                        ] {
                            ui.monospace(id);
                            ui.label(name);
                            ui.label(desc);
                            ui.end_row();
                        }
                    });
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. Extreme Modern DSP Generators");
                ui.label("- karplus_strong_pluck: Waveguide physical string modeling with frequency-dependent harmonic dispersion damping.");
                ui.label("- granular_cloud: 4 overlapping Hann-windowed microsound grains with +1 octave and +5th shimmer.");
                ui.label("- wavetable_morph: Continuous 3D wavetable interpolation (Sine -> Vocal Formant -> Digital Fold -> HyperSaw).");
                ui.label("- west_coast_wavefolder: 5-stage trigonometric wavefolder paired with an organic vactrol Low-Pass Gate (LPG).");
                ui.label("- phase_distortion_cz: non-linear phase-angle knee warping with hard-synced resonant window.");
                ui.label("- modal_struck_bar: Euler-Bernoulli uniform bar eigenmodes (f_0 * [1.0, 2.756, 5.404, 8.933]) for metallic/glass percussion.");
            });
        }
        HelpTab::SequencerEnv => {
            ui.heading("Workbenches 3, 4 & 5: How to Connect Your MIDI Keyboard, MIDI Depot & Acid Sequencer");
            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("1. How to Connect Your Physical USB or 5-Pin DIN MIDI Keyboard");
                ui.label("Step 1 (Direct USB / WinMM Studio Mode): Plug your USB MIDI controller (Arturia KeyStep/MiniLab, Akai MPK Mini, PCM workstation/nanoKEY, NI Komplete Kontrol, Novation Launchkey) into Goliath. Open Workbench 3 ('MIDI Keyboard & MIDI Depot') and select your device in the 'MIDI Input Device' dropdown (default: 'USB MIDI Keyboard (WinMM Auto-Detect / Omni)').");
                ui.label("Step 2 (MIDI Channel & CC Mapping): Leave MIDI Channel on '0 (Omni)' to receive on all 16 channels, or lock to '1..16'. Your hardware knobs map automatically:");
                ui.monospace(
                    "Mod Wheel (CC#01) -> LFO Filter Modulation Depth (CLAP 0x100A)\nBrightness (CC#74) -> 4-Pole Ladder VCF Cutoff Frequency Hz (CLAP 0x1004)\nTimbre     (CC#71) -> VCF Resonance Q (CLAP 0x1005)\nAttack     (CC#73) -> ADSR Attack Time ms (CLAP 0x1007)\nFX Depth   (CC#91) -> Extreme DSP Morph / Harmonics Macro",
                );
                ui.label("Step 3 (DAW / SoundCraft Low-Latency ASIO/WASAPI Mode): Once the loadable CLAP build exists (not built yet; today SynthCraft exports the descriptor JSON and WAV stems), load 'dev.sandraschi.synthcraft' as a CLAP 1.2.2 instrument plugin inside SoundCraft, Bitwig Studio, or REAPER and arm the instrument track for sub-5ms live keyboard performance.");
                ui.label("Step 4 (On-Screen Audition): You can also click any of the 25 piano keys (C3..C5) directly in Workbench 3 to audition notes without hardware attached.");
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. Built-In MIDI Pattern Depot & Standard MIDI File (.MID) Exporter");
                ui.label("Switch to 'MIDI Pattern Depot & .MID Export' under Workbench 3 to load 8 iconic patterns (1990 Ibiza Rave Piano, Club Organ 2 Bass, Vienna 303 Acid, Universe Pad, Wavefolder Polyrhythm, Karplus Harp, Modal Marimba, Neuro Reese) or export 480-PPQN Standard MIDI Files (.mid) to disk.");
            });
        }
        HelpTab::ClapDjcraft => {
            ui.heading("Workbenches 7 & 8: CLAP 1.2.2 Plugin Bridge for SoundCraft & Stem Forge for DJCraft");
            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("1. Why CLAP 1.2.2 Instead of VST3 for SoundCraft");
                ui.label("SoundCraft natively hosts open-source CLAP (Clever Audio Plug-in ABI by u-he & Bitwig, MIT license) rather than proprietary Steinberg VST3:");
                ui.label("- Plugin Identifier: dev.sandraschi.synthcraft (ABI entry symbol: clap_entry).");
                ui.label("- Polyphonic Per-Note Modulation: Parameters 0x1001..0x1006 and 0x100A expose CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID so SoundCraft can apply non-destructive per-voice modulation.");
                ui.label("- Zero-Copy Preset Sync: SoundCraft and SynthCraft share the exact same SynthPatch JSON and SQLite schema.");
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. DJCraft Stem, Loop & Riser Integration");
                ui.label("Every SynthCraft patch and MIDI Depot clip carries explicit BPM and Camelot Key metadata (8A Am, 4A Fm, 2A D#m, 9A Em, 11B A) plus cue points (LOOP_IN, FILTER_PEAK, DROP_OUT) so rendered 48 kHz 16-bit .wav stems drop onto DJCraft decks with zero beatgrid drift.");
            });
        }
        HelpTab::CliMcp => {
            ui.heading("L2 CLI Subcommands & 19 MCP 2026-07-28 Tools Reference");
            ui.label(
                "Both the CLI and the MCP 2026-07-28 stdio server invoke L1 'Command::dispatch'.",
            );
            ui.add_space(6.0);

            ui.group(|ui| {
                ui.strong("Complete 19 MCP Tools Table");
                egui::Grid::new("help_synth_mcp_table")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("MCP Tool Name");
                        ui.strong("CLI Subcommand");
                        ui.strong("Description");
                        ui.end_row();

                        for (tool, cli, desc) in [
                            ("synth_status", "status", "Engine version, active patch, BPM, Camelot key, and CLAP plugin ID"),
                            ("synth_summary", "summary", "Full active patch parameters, PCM, Extreme DSP, and 16-step sequence"),
                            ("list_patches", "list-patches", "List all 6 factory and custom patches"),
                            ("load_patch", "load-patch", "Activate a patch by ID (e.g. vienna_acid_303)"),
                            ("update_oscillators", "update-oscillators", "Set VCO1/VCO2 waveforms, detune cents, mix, and FM index"),
                            ("update_filter", "update-filter", "Set 4-pole VCF mode, cutoff Hz, resonance Q, and env mod depth"),
                            ("update_envelope_lfo", "update-envelope-lfo", "Set ADSR (attack, decay, sustain, release) and LFO rate/depth"),
                            ("set_sequencer_step", "set-step", "Edit step 0..15 (midi_note, accent, slide, active)"),
                            ("load_pcm_program", "load-m1", "Select a 1990 PCM workstation PCM program (m1_00_universe, m1_01_piano16, m1_17_organ2, ...) and mix"),
                            ("set_extreme_dsp", "set-extreme-dsp", "Select an Extreme DSP mode (karplus_strong_pluck, granular_cloud, west_coast_wavefolder, ...)"),
                            ("load_midi_depot_clip", "load-midi-clip", "Load one of the 8 built-in MIDI Depot patterns into the sequencer"),
                            ("export_midi_file", "export-midi", "Export a MIDI Depot clip as a Standard MIDI File (.mid, 480 PPQN)"),
                            ("run_demo_showcase", "run-demo", "Run one of the 6 curated 1-click Demo Showcases and return spectral metrics"),
                            ("synthesize_preview", "preview", "Synthesize audio buffer and return first 64 PCM float samples"),
                            ("analyze_spectrum", "analyze-spectrum", "Compute 32-band DFT spectrum, 128-sample scope, RMS/Peak dBFS, and pitch Hz"),
                            ("render_wav_file", "render-wav", "Write a 16-bit 48 kHz RIFF PCM .wav file to disk"),
                            ("export_clap_manifest", "export-clap", "Export CLAP 1.2.2 plugin descriptor and automation table for SoundCraft"),
                            ("export_djcraft_stem", "export-djcraft-stem", "Export BPM/Camelot loop metadata and cue points for DJCraft"),
                            ("reset_factory_patches", "reset-factory", "Restore all 6 factory patches to default settings"),
                        ] {
                            ui.monospace(tool);
                            ui.monospace(cli);
                            ui.label(desc);
                            ui.end_row();
                        }
                    });
            });
        }
        HelpTab::TechnicalInfo => {
            ui.heading("Technical Architecture, DSP Signal Equations & SQLite Schema");
            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("1. 3-Tier Pure-Rust ArtCraft Architecture");
                ui.label(format!("Application Version: v{APP_VERSION}"));
                ui.label(format!("Active SQLite Database Path: {}", app.db_path.display()));
                ui.monospace(
                    "L1: crates/synthcraft-core  (VCO/VCF, 1990-style PCM, Extreme DSP, SMF .mid & RIFF .wav writers)\nL2: crates/synthcraft-mcp   (clap 4.5 CLI + MCP 2026-07-28 JSON-RPC 2.0 stdio server, 19 tools)\nL3: crates/synthcraft-app   (eframe/egui 0.37 VCV 2 Eurorack studio + egui_kittest harness)",
                );
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. SQLite Persistence Schema (synth_patches & synth_state)");
                ui.monospace(
                    "CREATE TABLE synth_patches (id TEXT PRIMARY KEY, patch_json TEXT NOT NULL);\nCREATE TABLE synth_state (id INTEGER PRIMARY KEY CHECK (id = 1), active_patch_id TEXT NOT NULL, sample_rate_hz INTEGER NOT NULL);",
                );
            });
        }
    }
}
