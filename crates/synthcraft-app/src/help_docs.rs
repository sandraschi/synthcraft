//! Comprehensive 6-Tab In-App Documentation for SynthCraft.

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
            ui.heading("Workbench 1: Dual Anti-Aliased VCOs, FM & 4-Pole Resonant Ladder VCF");
            ui.label(
                "SynthCraft's signal chain models a dual-oscillator subtractive/FM modular voice feeding a non-linear 4-pole resonant filter with tanh soft-clipping.",
            );
            ui.add_space(6.0);

            ui.group(|ui| {
                ui.strong("1. Oscillator Waveform Library (VCO 1 & VCO 2)");
                egui::Grid::new("help_synth_waves_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Waveform");
                        ui.strong("Harmonic Spectrum");
                        ui.strong("Typical Sound Design Use");
                        ui.end_row();

                        for (w, harm, use_case) in [
                            (
                                "sine",
                                "Fundamental f_0 only (pure tone)",
                                "FM carrier/modulator, sub-drops, bell plucks",
                            ),
                            (
                                "saw",
                                "All integer harmonics 1/n",
                                "TB-303 acid basslines, brass leads, pads",
                            ),
                            (
                                "square",
                                "Odd harmonics 1/(2k+1), hollow timbre",
                                "Chiptune leads, woody clarinets, sub-basses",
                            ),
                            (
                                "triangle",
                                "Odd harmonics 1/(2k+1)^2, rapid roll-off",
                                "Flute leads, mellow sub-layering",
                            ),
                            (
                                "pulse_pwm",
                                "Variable duty-cycle rectangle (0.05..0.95)",
                                "Dub techno chord stabs, PWM string pads",
                            ),
                            (
                                "supersaw",
                                "4-voice detuned sawtooth unison stack",
                                "Reese DnB basses, anthem trance/techno pads",
                            ),
                            (
                                "white_noise",
                                "Flat broadband energy (deterministic PRNG)",
                                "DJCraft transition risers, snare/hat synthesis",
                            ),
                            (
                                "sub_octave",
                                "Sine wave locked 1 octave below VCO1 (f_0 / 2)",
                                "Club system sub-weight reinforcement",
                            ),
                        ] {
                            ui.monospace(w);
                            ui.label(harm);
                            ui.label(use_case);
                            ui.end_row();
                        }
                    });
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. 4-Pole Resonant Filter Modes (VCF)");
                ui.label("- lowpass_24db: Classic 24 dB/octave cascaded ladder filter with warm bass retention.");
                ui.label("- bandpass_12db: Isolates resonant midrange formants; ideal for dub techno chords and risers.");
                ui.label("- highpass_12db: Removes low-end mud below cutoff_hz for DJ transitions.");
                ui.label("- acid_resonant: Coupled lowpass + resonant bandpass feedback with tanh diode saturation for authentic TB-303 squelch.");
            });
        }
        HelpTab::SequencerEnv => {
            ui.heading("Workbench 2: 16-Step TB-303 Acid Sequencer, ADSR Envelope & Tempo LFO");
            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("1. 16-Step BPM-Locked Acid Sequencer");
                ui.label("Each step represents a 1/16th note at the patch's BPM (step_duration = 15 / BPM seconds):");
                ui.label("- MIDI Note (24..96): Converts to exact concert pitch via f = 440 * 2^((note - 69) / 12) Hz (e.g. 45 = A2 110 Hz, 57 = A3 220 Hz, 69 = A4 440 Hz).");
                ui.label("- Accent Flag: Multiplies VCF envelope modulation depth and drive saturation by 1.35x on accented steps.");
                ui.label("- Slide (Portamento) Flag: Slews the oscillator pitch exponentially from the previous step's frequency instead of re-triggering instantaneously.");
                ui.label("- Active Gate Flag: Mutes or unmutes individual 1/16th steps for syncopated rhythms.");
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. ADSR Contour & LFO Modulation");
                ui.monospace(
                    "Attack (ms)  : Linear ramp from 0.0 to 1.0 peak at note onset\nDecay (ms)   : Fall from 1.0 peak down to Sustain Level\nSustain (0..1): Held amplitude/filter contour while step gate is open\nRelease (ms) : Decay to 0.0 after step gate closes\nLFO Rate (Hz): Continuous sinusoidal modulator sweeping VCF cutoff (+/- 900 Hz * depth)",
                );
            });
        }
        HelpTab::ClapDjcraft => {
            ui.heading("Workbenches 4 & 5: CLAP 1.2.2 Plugin Bridge for SoundCraft & Stem Forge for DJCraft");
            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("1. Why CLAP 1.2.2 Instead of VST3 for SoundCraft");
                ui.label("SoundCraft rejects proprietary Steinberg VST3 licensing and natively hosts CLAP (Clever Audio Plug-in ABI by u-he & Bitwig, MIT license):");
                ui.label("- Plugin Identifier: dev.sandraschi.synthcraft (ABI entry symbol: clap_entry).");
                ui.label("- Polyphonic Per-Note Modulation: Parameters 0x1001..0x1006 and 0x100A expose CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID so SoundCraft can apply non-destructive per-voice modulation.");
                ui.label("- Zero-Copy Preset Sync: SoundCraft and SynthCraft share the exact same SynthPatch JSON and SQLite schema, allowing patches tweaked in SynthCraft to load directly inside SoundCraft.");
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. DJCraft Stem, Loop & Riser Integration");
                ui.label("Every SynthCraft patch carries explicit DJ metadata so exported .wav stems drop onto DJCraft decks without beatgrid drift or harmonic clash:");
                egui::Grid::new("help_djcraft_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Factory Patch ID");
                        ui.strong("BPM");
                        ui.strong("Camelot Key");
                        ui.strong("DJCraft Deck Role");
                        ui.end_row();

                        for (id, bpm, key, role) in [
                            ("vienna_acid_303", "128.0 BPM", "8A Am", "Peak-hour acid techno bassline loop"),
                            ("goliath_reese_sub", "174.0 BPM", "4A Fm", "Drum & Bass rolling neuro Reese sub"),
                            ("danube_dub_chord", "124.0 BPM", "2A D#m", "Deep dub techno chord stab bed"),
                            ("stephansdom_bell_fm", "120.0 BPM", "9A Em", "Melodic progressive FM arpeggio pluck"),
                            ("djcraft_riser_sweep", "128.0 BPM", "8A Am", "4-bar / 8-bar tension build before drop"),
                            ("soundcraft_warm_pad", "110.0 BPM", "11B A", "Ambient intro / outro harmonic bed"),
                        ] {
                            ui.monospace(id);
                            ui.monospace(bpm);
                            ui.monospace(key);
                            ui.label(role);
                            ui.end_row();
                        }
                    });
            });
        }
        HelpTab::CliMcp => {
            ui.heading("L2 CLI Subcommands & 14 MCP 2026-07-28 Tools Reference");
            ui.label(
                "Both the CLI and the MCP 2026-07-28 stdio server invoke L1 'Command::dispatch'.",
            );
            ui.add_space(6.0);

            ui.group(|ui| {
                ui.strong("Complete 14 MCP Tools Table");
                egui::Grid::new("help_synth_mcp_table")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("MCP Tool Name");
                        ui.strong("CLI Subcommand");
                        ui.strong("Description");
                        ui.end_row();

                        for (tool, cli, desc) in [
                            ("synth_status", "status", "Engine version, active patch, BPM, Camelot key, and CLAP plugin ID"),
                            ("synth_summary", "summary", "Full active patch parameters and 16-step sequence dump"),
                            ("list_patches", "list-patches", "List all 6 factory and custom patches"),
                            ("load_patch", "load-patch", "Activate a patch by ID (e.g. vienna_acid_303)"),
                            ("update_oscillators", "update-oscillators", "Set VCO1/VCO2 waveforms, detune cents, mix, and FM index"),
                            ("update_filter", "update-filter", "Set 4-pole VCF mode, cutoff Hz, resonance Q, and env mod depth"),
                            ("update_envelope_lfo", "update-envelope-lfo", "Set ADSR (attack, decay, sustain, release) and LFO rate/depth"),
                            ("set_sequencer_step", "set-step", "Edit step 0..15 (midi_note, accent, slide, active)"),
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
                    "L1: crates/synthcraft-core  (Pure-Rust DSP engine, 32-band DFT, RIFF WAV writer, CLAP 1.2.2 descriptor)\nL2: crates/synthcraft-mcp   (clap 4.5 CLI + MCP 2026-07-28 JSON-RPC 2.0 stdio server)\nL3: crates/synthcraft-app   (eframe/egui 0.37 native desktop studio + egui_kittest harness)",
                );
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. SQLite Persistence Schema (synth_patches & synth_state)");
                ui.monospace(
                    "CREATE TABLE synth_patches (id TEXT PRIMARY KEY, patch_json TEXT NOT NULL);\nCREATE TABLE synth_state (id INTEGER PRIMARY KEY CHECK (id = 1), active_patch_id TEXT NOT NULL, sample_rate_hz INTEGER NOT NULL);",
                );
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("3. Safety & Quality Containment Invariants");
                ui.label("- #![forbid(unsafe_code)] enforced across all three workspace crates.");
                ui.label("- clippy::deny(unwrap_used, expect_used, panic, todo, unimplemented) enforced at workspace level.");
                ui.label("- Deterministic PRNG for WhiteNoise so unit tests and stem renders are 100% bit-reproducible.");
            });
        }
    }
}
