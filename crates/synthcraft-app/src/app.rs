//! SynthCraft L3 Native Desktop Studio (`eframe` / `egui` 0.37).

use crate::subapps;
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::path::PathBuf;
use synthcraft_core::{
    analyze_patch_spectrum, default_db_path, export_clap_descriptor, export_djcraft_stem_manifest,
    midi_to_freq_hz, write_wav_file, FilterMode, SynthEngine, WaveformKind, APP_VERSION,
};

const ATTRIBUTION_MD: &str = include_str!("../../../ATTRIBUTION.md");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLocale {
    En,
    De,
}

impl AppLocale {
    pub fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::De => "Deutsch",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    ModularRack,
    M1ExtremeDsp,
    MidiDepot,
    DemoShowcase,
    SequencerEnv,
    ScopeSpectrum,
    ClapSoundcraft,
    DjcraftForge,
    Chat,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTab {
    GetStarted,
    ModularRack,
    SequencerEnv,
    ClapDjcraft,
    CliMcp,
    TechnicalInfo,
}

pub struct SynthCraftApp {
    pub engine: SynthEngine,
    pub db_path: PathBuf,
    pub page: Page,
    pub subpage: usize,
    pub locale: AppLocale,
    pub dark_mode: bool,
    pub zoom_factor: f32,
    pub status_line: String,
    // Workbench 3: Scope & Spectrum
    pub analysis_duration_ms: u32,
    // Workbench 5: DJCraft & WAV export
    pub djcraft_bars: usize,
    pub wav_export_path: String,
    // Workbench 6: Assistant & Modals
    pub chat_input: String,
    pub chat_history: Vec<(String, String)>,
    pub help_tab: HelpTab,
    pub show_palette: bool,
    pub palette_query: String,
    pub show_shortcuts: bool,
    pub show_licenses: bool,
    pub show_about: bool,
}

impl Default for SynthCraftApp {
    fn default() -> Self {
        Self::with_db_path(default_db_path())
    }
}

impl SynthCraftApp {
    pub fn with_db_path(db_path: PathBuf) -> Self {
        let engine = SynthEngine::open_or_seed(&db_path).unwrap_or_default();
        Self {
            engine,
            db_path,
            page: Page::ModularRack,
            subpage: 0,
            locale: AppLocale::En,
            dark_mode: true,
            zoom_factor: 1.0,
            status_line: format!(
                "SynthCraft v{APP_VERSION} ready -- CLAP 1.2.2 (dev.sandraschi.synthcraft) & DJCraft Stem Forge"
            ),
            analysis_duration_ms: 250,
            djcraft_bars: 4,
            wav_export_path: "vienna_acid_303_128bpm_8A.wav".into(),
            chat_input: String::new(),
            chat_history: vec![(
                "SynthCraft".into(),
                "no model selected -- connect an MCP 2026-07-28 client or local Ollama endpoint to design synth patches, CLAP mappings, and DJCraft stems.".into(),
            )],
            help_tab: HelpTab::GetStarted,
            show_palette: false,
            palette_query: String::new(),
            show_shortcuts: false,
            show_licenses: false,
            show_about: false,
        }
    }

    pub fn goto(&mut self, page: Page, sub: usize) {
        self.page = page;
        self.subpage = sub;
    }

    pub fn persist_state(&mut self) {
        self.persist();
    }

    fn persist(&mut self) {
        let _ = self.engine.save_db(&self.db_path);
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let theme = if self.dark_mode {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        };
        ctx.set_theme(theme);
        let mut style = (*ctx.style_of(theme)).clone();
        if self.dark_mode {
            style.visuals.override_text_color = Some(Color32::from_rgb(241, 245, 249));
            style.visuals.panel_fill = Color32::from_rgb(15, 23, 42);
        } else {
            style.visuals.override_text_color = Some(Color32::from_rgb(15, 23, 42));
            style.visuals.panel_fill = Color32::from_rgb(248, 250, 252);
        }
        ctx.set_style_of(theme, style);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let zoom_delta = ctx.input(|i| {
            if i.modifiers.ctrl {
                i.smooth_scroll_delta.y
            } else {
                0.0
            }
        });
        if zoom_delta.abs() > 0.1 {
            let step = if zoom_delta > 0.0 { 0.1 } else { -0.1 };
            self.zoom_factor = (self.zoom_factor + step).clamp(0.75, 1.75);
            ctx.set_zoom_factor(self.zoom_factor);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.goto(Page::Help, 0);
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::K)) {
            self.show_palette = !self.show_palette;
        }
    }

    pub fn render_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Save Patch Bank to SQLite").clicked() {
                    self.persist();
                    self.status_line = format!("Saved to {}", self.db_path.display());
                    ui.close();
                }
                if ui.button("Reset Factory Patches").clicked() {
                    self.engine = SynthEngine::factory_seed();
                    self.persist();
                    ui.close();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.button("Randomize Acid Sequence").clicked() {
                    let scale = [45_u8, 48, 50, 52, 55, 57, 60, 62];
                    let p = self.engine.active_patch_mut();
                    for (i, step) in p.steps.iter_mut().enumerate() {
                        step.midi_note = scale[(i * 3 + 1) % scale.len()];
                        step.accent = i % 3 == 0;
                        step.slide = i % 4 == 2;
                    }
                    self.persist();
                    ui.close();
                }
            });
            ui.menu_button("View", |ui| {
                if ui.button("Dark").clicked() {
                    self.dark_mode = true;
                    ui.close();
                }
                if ui.button("Light").clicked() {
                    self.dark_mode = false;
                    ui.close();
                }
                ui.separator();
                if ui.button("English").clicked() {
                    self.locale = AppLocale::En;
                    ui.close();
                }
                if ui.button("Deutsch").clicked() {
                    self.locale = AppLocale::De;
                    ui.close();
                }
            });
            ui.menu_button("Insert", |ui| {
                for (id, label) in [
                    ("vienna_acid_303", "Patch: Vienna Acid 303 (128 BPM 8A)"),
                    ("goliath_reese_sub", "Patch: Goliath Reese Sub (174 BPM 4A)"),
                    ("danube_dub_chord", "Patch: Danube Dub Chord (124 BPM 2A)"),
                    (
                        "stephansdom_bell_fm",
                        "Patch: Stephansdom FM Pluck (120 BPM 9A)",
                    ),
                    (
                        "djcraft_riser_sweep",
                        "Patch: DJCraft Drop Riser (128 BPM 8A)",
                    ),
                    (
                        "soundcraft_warm_pad",
                        "Patch: SoundCraft Analog Pad (110 BPM 11B)",
                    ),
                ] {
                    if ui.button(label).clicked() {
                        let _ = self.engine.select_patch(id);
                        self.persist();
                        ui.close();
                    }
                }
            });
            ui.menu_button("Modify", |ui| {
                if ui.button("Boost Filter Resonance (+1.5 Q)").clicked() {
                    let p = self.engine.active_patch_mut();
                    p.resonance_q = (p.resonance_q + 1.5).clamp(0.5, 12.0);
                    self.persist();
                    ui.close();
                }
            });
            ui.menu_button("Commands", |ui| {
                if ui.button("Inspect 32-Band DFT Spectrum").clicked() {
                    self.goto(Page::ScopeSpectrum, 1);
                    ui.close();
                }
                if ui
                    .button("Export CLAP 1.2.2 Descriptor for SoundCraft")
                    .clicked()
                {
                    self.goto(Page::ClapSoundcraft, 0);
                    ui.close();
                }
                if ui.button("Export DJCraft Stem & Cue Points").clicked() {
                    self.goto(Page::DjcraftForge, 0);
                    ui.close();
                }
            });
            ui.menu_button("Window", |ui| {
                for (label, p, s) in [
                    ("Modular Rack & VCO/VCF", Page::ModularRack, 0),
                    ("16-Step Acid Sequencer", Page::SequencerEnv, 0),
                    ("Oscilloscope & Spectrum", Page::ScopeSpectrum, 0),
                    ("CLAP & SoundCraft Bridge", Page::ClapSoundcraft, 0),
                    ("DJCraft Stem & WAV Forge", Page::DjcraftForge, 0),
                    ("Chat", Page::Chat, 0),
                    ("Help (F1)", Page::Help, 0),
                ] {
                    if ui.button(label).clicked() {
                        self.goto(p, s);
                        ui.close();
                    }
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("Command Palette (Ctrl+K)").clicked() {
                    self.show_palette = true;
                    ui.close();
                }
                if ui.button("Keyboard Shortcuts").clicked() {
                    self.show_shortcuts = true;
                    ui.close();
                }
                if ui.button("Documentation (F1)").clicked() {
                    self.goto(Page::Help, 0);
                    ui.close();
                }
                if ui.button("Licenses").clicked() {
                    self.show_licenses = true;
                    ui.close();
                }
                if ui.button("About SynthCraft").clicked() {
                    self.show_about = true;
                    ui.close();
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let theme_lbl = if self.dark_mode { "Light" } else { "Dark" };
                if ui.button(theme_lbl).clicked() {
                    self.dark_mode = !self.dark_mode;
                }
                let loc_lbl = self.locale.label();
                if ui.button(loc_lbl).clicked() {
                    self.locale = match self.locale {
                        AppLocale::En => AppLocale::De,
                        AppLocale::De => AppLocale::En,
                    };
                }
                let ap = self.engine.active_patch();
                ui.label(format!(
                    "Patch: {} | {:.0} BPM | Key: {} | {} Hz",
                    ap.name, ap.bpm, ap.camelot_key, self.engine.sample_rate_hz
                ));
            });
        });
        let _ = ctx;
    }

    pub fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading("SynthCraft");
        ui.small("Modular DSP, CLAP & DJ Stem Studio");
        ui.separator();

        for wb in subapps::registry() {
            let subs = wb.subpages();
            let has_active = subs
                .iter()
                .any(|sp| sp.page == self.page && sp.sub == self.subpage);
            egui::CollapsingHeader::new(RichText::new(wb.title(self.locale)).strong())
                .id_salt(wb.id())
                .default_open(has_active || wb.id() == "modular_rack")
                .show(ui, |ui| {
                    for sp in subs {
                        let active = self.page == sp.page && self.subpage == sp.sub;
                        if ui.selectable_label(active, sp.label(self.locale)).clicked() {
                            self.goto(sp.page, sp.sub);
                        }
                    }
                });
        }
    }

    pub fn render_page(&mut self, ui: &mut egui::Ui) {
        match self.page {
            Page::ModularRack => self.render_modular_rack(ui),
            Page::M1ExtremeDsp => crate::rack_ui::render_m1_and_extreme_dsp(self, ui),
            Page::MidiDepot => crate::rack_ui::render_midi_and_depot(self, ui),
            Page::DemoShowcase => crate::rack_ui::render_demo_showcase(self, ui),
            Page::SequencerEnv => self.render_sequencer_env(ui),
            Page::ScopeSpectrum => self.render_scope_spectrum(ui),
            Page::ClapSoundcraft => self.render_clap_soundcraft(ui),
            Page::DjcraftForge => self.render_djcraft_forge(ui),
            Page::Chat => self.render_chat(ui),
            Page::Help => self.render_help_view(ui),
        }
    }

    fn render_modular_rack(&mut self, ui: &mut egui::Ui) {
        ui.heading("1. VCV Rack 2-Style 3U Eurorack & Dual VCO/VCF Bay");
        ui.horizontal_wrapped(|ui| {
            ui.label("Factory Patches:");
            let patch_list: Vec<(String, String)> = self
                .engine
                .patches
                .iter()
                .map(|p| (p.id.clone(), p.name.clone()))
                .collect();
            for (id, name) in patch_list {
                if ui
                    .selectable_label(self.engine.active_patch_id == id, name)
                    .clicked()
                {
                    let _ = self.engine.select_patch(&id);
                    self.persist();
                }
            }
        });
        ui.separator();

        if self.subpage == 0 {
            crate::rack_ui::render_vcv_eurorack_canvas(self, ui);
            ui.add_space(6.0);
        }

        if self.subpage == 1 {
            ui.group(|ui| {
                ui.strong("Factory Patch Bank Overview (SoundCraft CLAP & DJCraft Compatible)");
                egui::Grid::new("patch_library_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Patch ID");
                        ui.strong("Name");
                        ui.strong("Category");
                        ui.strong("BPM");
                        ui.strong("Camelot Key");
                        ui.strong("VCO 1 / 2");
                        ui.strong("VCF Mode");
                        ui.end_row();
                        for p in &self.engine.patches {
                            ui.monospace(&p.id);
                            ui.label(&p.name);
                            ui.label(&p.category);
                            ui.monospace(format!("{:.0}", p.bpm));
                            ui.monospace(&p.camelot_key);
                            ui.monospace(format!(
                                "{} + {}",
                                p.vco1_wave.as_str(),
                                p.vco2_wave.as_str()
                            ));
                            ui.monospace(p.filter_mode.as_str());
                            ui.end_row();
                        }
                    });
            });
            ui.add_space(6.0);
        }

        let mut changed = false;
        {
            let p = self.engine.active_patch_mut();
            ui.columns(2, |cols| {
                cols[0].group(|ui| {
                    ui.strong("Dual Oscillators (VCO 1 & VCO 2) + FM");
                    ui.horizontal_wrapped(|ui| {
                        ui.label("VCO 1:");
                        for w in [
                            WaveformKind::Sine,
                            WaveformKind::Saw,
                            WaveformKind::Square,
                            WaveformKind::Triangle,
                            WaveformKind::PulsePwm,
                            WaveformKind::SuperSaw,
                            WaveformKind::WhiteNoise,
                            WaveformKind::SubOctave,
                        ] {
                            if ui.selectable_label(p.vco1_wave == w, w.as_str()).clicked() {
                                p.vco1_wave = w;
                                changed = true;
                            }
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label("VCO 2:");
                        for w in [
                            WaveformKind::Sine,
                            WaveformKind::Saw,
                            WaveformKind::Square,
                            WaveformKind::Triangle,
                            WaveformKind::PulsePwm,
                            WaveformKind::SuperSaw,
                            WaveformKind::WhiteNoise,
                            WaveformKind::SubOctave,
                        ] {
                            if ui.selectable_label(p.vco2_wave == w, w.as_str()).clicked() {
                                p.vco2_wave = w;
                                changed = true;
                            }
                        }
                    });
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut p.vco2_detune_cents, -50.0..=50.0)
                                .text("VCO 2 Detune (cents)"),
                        )
                        .changed();
                    changed |= ui
                        .add(egui::Slider::new(&mut p.vco_mix, 0.0..=1.0).text("VCO 1/2 Mix"))
                        .changed();
                    changed |= ui
                        .add(egui::Slider::new(&mut p.fm_index, 0.0..=4.0).text("FM Index"))
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut p.pulse_width, 0.05..=0.95)
                                .text("PWM Duty Cycle"),
                        )
                        .changed();
                });

                cols[1].group(|ui| {
                    ui.strong("4-Pole Resonant Ladder VCF & Saturation");
                    ui.horizontal_wrapped(|ui| {
                        for fm in [
                            FilterMode::LowPass24dB,
                            FilterMode::BandPass12dB,
                            FilterMode::HighPass12dB,
                            FilterMode::AcidResonant,
                        ] {
                            if ui
                                .selectable_label(p.filter_mode == fm, fm.as_str())
                                .clicked()
                            {
                                p.filter_mode = fm;
                                changed = true;
                            }
                        }
                    });
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut p.cutoff_hz, 40.0..=12000.0)
                                .logarithmic(true)
                                .text("Cutoff Frequency (Hz)"),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut p.resonance_q, 0.5..=12.0).text("Resonance (Q)"),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut p.env_mod_depth, 0.0..=1.0)
                                .text("Envelope Mod Depth"),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut p.drive_saturation, 0.0..=3.0)
                                .text("Diode Drive Saturation"),
                        )
                        .changed();
                });
            });
        }
        if changed {
            self.persist();
        }
    }

    fn render_sequencer_env(&mut self, ui: &mut egui::Ui) {
        ui.heading("2. 16-Step TB-303 Acid Sequencer, ADSR & Tempo LFO");
        ui.separator();

        let mut changed = false;
        {
            let p = self.engine.active_patch_mut();
            if self.subpage == 1 {
                ui.group(|ui| {
                    ui.strong("ADSR Contour Generator & Sinusoidal Filter LFO");
                    changed |= ui
                        .add(egui::Slider::new(&mut p.attack_ms, 1.0..=1000.0).text("Attack (ms)"))
                        .changed();
                    changed |= ui
                        .add(egui::Slider::new(&mut p.decay_ms, 10.0..=2000.0).text("Decay (ms)"))
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut p.sustain_level, 0.0..=1.0)
                                .text("Sustain Level"),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut p.release_ms, 10.0..=2000.0)
                                .text("Release (ms)"),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut p.lfo_rate_hz, 0.1..=20.0).text("LFO Rate (Hz)"),
                        )
                        .changed();
                    changed |= ui
                        .add(egui::Slider::new(&mut p.lfo_depth, 0.0..=1.0).text("LFO Depth"))
                        .changed();
                });
                ui.add_space(6.0);
            }

            ui.group(|ui| {
                ui.strong(format!(
                    "16-Step Pattern ({:.0} BPM | Camelot Key {})",
                    p.bpm, p.camelot_key
                ));
                egui::Grid::new("acid_seq_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Step");
                        ui.strong("Active");
                        ui.strong("MIDI Note");
                        ui.strong("Freq (Hz)");
                        ui.strong("Accent");
                        ui.strong("303 Slide");
                        ui.end_row();

                        for s in &mut p.steps {
                            ui.monospace(format!("#{:02}", s.step_index + 1));
                            changed |= ui.checkbox(&mut s.active, "On").changed();
                            changed |= ui
                                .add(
                                    egui::Slider::new(&mut s.midi_note, 36..=84)
                                        .text(format!("S{}", s.step_index + 1)),
                                )
                                .changed();
                            ui.monospace(format!("{:.1} Hz", midi_to_freq_hz(s.midi_note)));
                            changed |= ui.checkbox(&mut s.accent, "Acc").changed();
                            changed |= ui.checkbox(&mut s.slide, "Slide").changed();
                            ui.end_row();
                        }
                    });
            });
        }
        if changed {
            self.persist();
        }
    }

    fn render_scope_spectrum(&mut self, ui: &mut egui::Ui) {
        let green = Color32::from_rgb(34, 197, 94);
        let cyan = Color32::from_rgb(56, 189, 248);

        ui.heading("3. Real-Time Oscilloscope & 32-Band DFT Spectrum Analyzer");
        ui.horizontal(|ui| {
            ui.add(
                egui::Slider::new(&mut self.analysis_duration_ms, 80..=1000)
                    .text("Analysis Window (ms)"),
            );
        });
        ui.separator();

        let rep = analyze_patch_spectrum(
            self.engine.active_patch(),
            self.engine.sample_rate_hz,
            self.analysis_duration_ms,
        );

        ui.horizontal_wrapped(|ui| {
            for (lbl, val) in [
                ("RMS Level", format!("{:.2} dBFS", rep.rms_dbfs)),
                ("Peak Level", format!("{:.2} dBFS", rep.peak_dbfs)),
                ("Crest Factor", format!("{:.2} dB", rep.crest_factor_db)),
                (
                    "Zero-Crossing Pitch",
                    format!("{:.1} Hz", rep.estimated_pitch_hz),
                ),
                (
                    "PCM Samples",
                    format!("{} @ {} Hz", rep.sample_count, rep.sample_rate_hz),
                ),
            ] {
                ui.group(|ui| {
                    ui.small(lbl);
                    ui.strong(val);
                });
            }
        });

        ui.add_space(8.0);
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width().max(560.0), 210.0),
            Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        let bg = if self.dark_mode {
            Color32::from_rgb(9, 14, 28)
        } else {
            Color32::from_rgb(241, 245, 249)
        };
        painter.rect_filled(rect, 6.0, bg);

        if self.subpage == 1 {
            // Draw 32-band DFT harmonic spectrum bars
            let bar_w = (rect.width() - 24.0) / (rep.spectrum_bands_db.len().max(1) as f32);
            for (idx, &(_freq, db)) in rep.spectrum_bands_db.iter().enumerate() {
                let norm = ((db + 80.0) / 86.0).clamp(0.04, 1.0);
                let bh = norm * (rect.height() - 28.0);
                let x0 = rect.left() + 12.0 + (idx as f32) * bar_w;
                let bar_rect = Rect::from_min_max(
                    Pos2::new(x0, rect.bottom() - 14.0 - bh),
                    Pos2::new(x0 + (bar_w - 3.0).max(2.0), rect.bottom() - 14.0),
                );
                painter.rect_filled(bar_rect, 2.0, cyan);
            }
        } else {
            // Draw 128-sample time-domain oscilloscope waveform
            let mid_y = rect.center().y;
            let amp_h = (rect.height() * 0.42).max(20.0);
            let step_x = (rect.width() - 24.0) / (rep.scope_samples.len().max(2) as f32);
            for (idx, pair) in rep.scope_samples.windows(2).enumerate() {
                let x0 = rect.left() + 12.0 + (idx as f32) * step_x;
                let x1 = x0 + step_x;
                let y0 = mid_y - pair[0] * amp_h;
                let y1 = mid_y - pair[1] * amp_h;
                painter.line_segment(
                    [Pos2::new(x0, y0), Pos2::new(x1, y1)],
                    Stroke::new(2.0, green),
                );
            }
        }
    }

    fn render_clap_soundcraft(&mut self, ui: &mut egui::Ui) {
        ui.heading("4. CLAP 1.2.2 Plugin Bridge for SoundCraft");
        ui.label(
            "SoundCraft hosts open-source CLAP 1.2.2 plugins (dev.sandraschi.synthcraft) with polyphonic per-note modulation.",
        );
        ui.separator();

        let clap = export_clap_descriptor(self.engine.active_patch());
        if self.subpage == 1 {
            if let Some(params) = clap.get("parameters").and_then(serde_json::Value::as_array) {
                egui::Grid::new("clap_params_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("CLAP Param ID");
                        ui.strong("Name");
                        ui.strong("Module");
                        ui.strong("Range");
                        ui.strong("Current Value");
                        ui.strong("Poly-Modulatable");
                        ui.end_row();
                        for p in params {
                            ui.monospace(p["clap_id"].as_str().unwrap_or(""));
                            ui.label(p["name"].as_str().unwrap_or(""));
                            ui.monospace(p["module"].as_str().unwrap_or(""));
                            ui.monospace(format!(
                                "{:.1} .. {:.1}",
                                p["min_val"].as_f64().unwrap_or(0.0),
                                p["max_val"].as_f64().unwrap_or(1.0)
                            ));
                            ui.monospace(format!(
                                "{:.2}",
                                p["current_val"].as_f64().unwrap_or(0.0)
                            ));
                            ui.monospace(if p["poly_modulatable"].as_bool().unwrap_or(false) {
                                "YES (Per-Note)"
                            } else {
                                "Global"
                            });
                            ui.end_row();
                        }
                    });
                return;
            }
        }

        let pretty = serde_json::to_string_pretty(&clap).unwrap_or_default();
        ui.add(
            egui::TextEdit::multiline(&mut pretty.as_str())
                .hint_text("CLAP 1.2.2 Plugin Manifest JSON")
                .code_editor()
                .desired_width(f32::INFINITY)
                .desired_rows(18),
        );
    }

    fn render_djcraft_forge(&mut self, ui: &mut egui::Ui) {
        ui.heading("5. DJCraft BPM / Camelot Stem & RIFF WAV Forge");
        ui.label(
            "Synthesize BPM-locked, Camelot-keyed acid loops, sub-drops, and transition risers directly to 48 kHz 16-bit PCM .wav files for DJCraft and SoundCraft.",
        );
        ui.separator();

        {
            let p = self.engine.active_patch_mut();
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::Slider::new(&mut p.bpm, 70.0..=190.0).text("Deck Tempo (BPM)"));
                ui.label("Camelot Key:");
                for k in ["8A Am", "4A Fm", "2A D#m", "9A Em", "11B A", "5A Cm"] {
                    if ui.selectable_label(p.camelot_key == k, k).clicked() {
                        p.camelot_key = k.into();
                    }
                }
                ui.add(egui::Slider::new(&mut self.djcraft_bars, 1..=16).text("Loop Bars"));
            });
        }

        if self.subpage == 1 {
            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("Render 48 kHz 16-Bit RIFF PCM .wav Stem to Disk");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.wav_export_path)
                            .hint_text("Output WAV path e.g. loop_128bpm_8A.wav")
                            .desired_width(300.0),
                    );
                    if ui.button("Render .WAV Stem").clicked() {
                        let out_path = PathBuf::from(&self.wav_export_path);
                        if let Ok(samples) = write_wav_file(
                            self.engine.active_patch(),
                            &out_path,
                            self.engine.sample_rate_hz,
                            1000,
                        ) {
                            self.status_line = format!(
                                "Rendered {} PCM samples to {}",
                                samples,
                                out_path.display()
                            );
                        }
                    }
                });
            });
        }

        ui.add_space(6.0);
        let stem = export_djcraft_stem_manifest(self.engine.active_patch(), self.djcraft_bars);
        let pretty = serde_json::to_string_pretty(&stem).unwrap_or_default();
        ui.add(
            egui::TextEdit::multiline(&mut pretty.as_str())
                .hint_text("DJCraft Stem Manifest JSON")
                .code_editor()
                .desired_width(f32::INFINITY)
                .desired_rows(14),
        );
    }

    fn render_chat(&mut self, ui: &mut egui::Ui) {
        ui.heading("SynthCraft Sound-Designer Chat");
        ui.label("no model selected -- connect an MCP 2026-07-28 client or local Ollama endpoint to synthesize patches from natural language.");
        ui.separator();
        for (who, msg) in &self.chat_history {
            ui.horizontal_wrapped(|ui| {
                ui.strong(format!("{who}:"));
                ui.label(msg);
            });
        }
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.chat_input)
                    .hint_text("Ask about TB-303 acid filters, CLAP 1.2.2 parameters, or DJCraft stems...")
                    .desired_width(420.0),
            );
            if ui.button("Send").clicked() && !self.chat_input.trim().is_empty() {
                let q = self.chat_input.trim().to_string();
                self.chat_input.clear();
                self.chat_history.push(("You".into(), q));
                self.chat_history.push((
                    "SynthCraft".into(),
                    "no model selected -- use the 14 SynthCraft MCP tools (`load_patch`, `update_filter`, `analyze_spectrum`, `export_clap_manifest`, `export_djcraft_stem`, `render_wav_file`) from your agent IDE.".into(),
                ));
            }
        });
    }

    fn render_help_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("SynthCraft Documentation & Reference (F1)");
        ui.horizontal_wrapped(|ui| {
            for (tab, label) in [
                (HelpTab::GetStarted, "Get Started"),
                (HelpTab::ModularRack, "Modular VCO & VCF"),
                (HelpTab::SequencerEnv, "Sequencer & ADSR/LFO"),
                (HelpTab::ClapDjcraft, "CLAP & DJCraft/SoundCraft"),
                (HelpTab::CliMcp, "CLI & MCP 2026-07-28"),
                (HelpTab::TechnicalInfo, "Technical Info"),
            ] {
                if ui.selectable_label(self.help_tab == tab, label).clicked() {
                    self.help_tab = tab;
                }
            }
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            crate::help_docs::render_help_tab_content(self, ui);
        });
    }

    fn render_modals(&mut self, ctx: &egui::Context) {
        if self.show_palette {
            let mut open = self.show_palette;
            let mut nav_target: Option<(Page, usize)> = None;
            egui::Window::new("Command Palette")
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.palette_query)
                            .hint_text("Filter commands..."),
                    );
                    for (label, page, sub) in [
                        ("Modular Rack: Dual VCO & Ladder VCF", Page::ModularRack, 0),
                        ("Modular Rack: Factory Patch Bank", Page::ModularRack, 1),
                        ("16-Step TB-303 Acid Sequencer", Page::SequencerEnv, 0),
                        ("ADSR Envelope & Tempo LFO", Page::SequencerEnv, 1),
                        ("Real-Time Waveform Oscilloscope", Page::ScopeSpectrum, 0),
                        ("32-Band DFT Spectrum & dBFS Meter", Page::ScopeSpectrum, 1),
                        ("CLAP 1.2.2 Plugin Descriptor", Page::ClapSoundcraft, 0),
                        ("SoundCraft CLAP Parameter Map", Page::ClapSoundcraft, 1),
                        ("DJCraft BPM & Camelot Loop Forge", Page::DjcraftForge, 0),
                        ("Render 48 kHz RIFF .WAV Stem", Page::DjcraftForge, 1),
                        ("Help Documentation (F1)", Page::Help, 0),
                    ] {
                        if (self.palette_query.is_empty()
                            || label
                                .to_ascii_lowercase()
                                .contains(&self.palette_query.to_ascii_lowercase()))
                            && ui.button(label).clicked()
                        {
                            nav_target = Some((page, sub));
                        }
                    }
                });
            self.show_palette = open;
            if let Some((page, sub)) = nav_target {
                self.goto(page, sub);
                self.show_palette = false;
            }
        }
        if self.show_shortcuts {
            egui::Window::new("Keyboard Shortcuts")
                .open(&mut self.show_shortcuts)
                .show(ctx, |ui| {
                    ui.label("F1: Open 6-Tab Help");
                    ui.label("Ctrl+K: Toggle Command Palette");
                    ui.label("Ctrl+MouseWheel: Zoom UI (0.75x .. 1.75x)");
                });
        }
        if self.show_licenses {
            egui::Window::new("Licenses & Attribution")
                .open(&mut self.show_licenses)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(ATTRIBUTION_MD);
                    });
                });
        }
        if self.show_about {
            egui::Window::new("About SynthCraft")
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.heading(format!("SynthCraft v{APP_VERSION}"));
                    ui.label("Pure-Rust Modular DSP Synthesizer, CLAP 1.2.2 Bridge for SoundCraft & Stem Forge for DJCraft.");
                });
        }
    }
}

impl eframe::App for SynthCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.apply_theme(&ctx);
        self.handle_shortcuts(&ctx);

        egui::Panel::top("synthcraft_top_menu").show(ui, |ui| {
            self.render_menu_bar(&ctx, ui);
        });
        egui::Panel::bottom("synthcraft_status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_line);
            });
        });
        egui::Panel::left("synthcraft_sidebar")
            .default_size(225.0)
            .show(ui, |ui| {
                self.render_sidebar(ui);
            });
        egui::CentralPanel::default().show(ui, |ui| {
            self.render_page(ui);
        });

        self.render_modals(&ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    fn temp_app() -> SynthCraftApp {
        let mut p = std::env::temp_dir();
        p.push(format!("synthcraft_app_test_{}.db", std::process::id()));
        SynthCraftApp::with_db_path(p)
    }

    #[test]
    fn subapp_registry_exposes_all_six_workbenches() {
        let reg = subapps::registry();
        assert_eq!(reg.len(), 9);
        let total_subpages: usize = reg.iter().map(|w| w.subpages().len()).sum();
        assert_eq!(total_subpages, 17);
    }

    #[test]
    fn headless_egui_kittest_renders_all_twelve_workbench_subpages() {
        let mut app = temp_app();
        for wb in subapps::registry() {
            for sp in wb.subpages() {
                app.goto(sp.page, sp.sub);
                let mut harness = Harness::new_ui(|ui| {
                    let ctx = ui.ctx().clone();
                    app.render_menu_bar(&ctx, ui);
                    app.render_sidebar(ui);
                    app.render_page(ui);
                });
                harness.run();
            }
        }
        let _ = std::fs::remove_file(&app.db_path);
    }

    #[test]
    fn headless_egui_kittest_renders_all_six_help_tabs_and_both_themes() {
        let mut app = temp_app();
        app.goto(Page::Help, 0);
        for tab in [
            HelpTab::GetStarted,
            HelpTab::ModularRack,
            HelpTab::SequencerEnv,
            HelpTab::ClapDjcraft,
            HelpTab::CliMcp,
            HelpTab::TechnicalInfo,
        ] {
            app.help_tab = tab;
            app.dark_mode = !app.dark_mode;
            let mut harness = Harness::new_ui(|ui| {
                let ctx = ui.ctx().clone();
                app.apply_theme(&ctx);
                app.render_page(ui);
            });
            harness.run();
        }
        let _ = std::fs::remove_file(&app.db_path);
    }
}
