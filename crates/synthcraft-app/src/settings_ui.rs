//! Massive 6-Tab Studio Settings & Hardware Preferences Workbench for SynthCraft:
//! 1. Audio Engine & Low-Latency Driver (WASAPI / ASIO / CoreAudio / JACK, Sample Rate, Buffer Size, Oversampling, Polyphony, A4=440Hz Reference, Soft Clipper)
//! 2. MIDI Hardware, Clock Sync & MPE (Ports, Channel, MPE Pitch Bend, MIDI Clock Sync, Velocity Curve, Custom CC Matrix)
//! 3. Appearance, Hardware Skins, Layouts & Rotary Physics (Skin, Layout, UI Zoom, Cable Sag & Opacity, Knob Drag Sensitivity, VU Ballistics)
//! 4. Korg M1 (1990) AI2 DAC & Extreme DSP Tuning (32kHz MB87404 DAC Aliasing Emulation, Linear/Sinc Interpolation, Granular Density, Wavefolder Stages, Unison Voices)
//! 5. CLAP 1.2.2, SoundCraft & DJCraft Paths (CLAP Bundle Directory, SoundCraft IPC Port, DJCraft Stem Output Folder, Auto-Cue, Bit Depth, Loudness Normalization)
//! 6. AI Assistant, MCP 2026-07-28 & SQLite Storage (Ollama Endpoint, Model Selector, Temperature, DB Path, Auto-Save, Factory Reset)

use crate::app::{AppLocale, SynthCraftApp};
use crate::controls::{rotary_knob, vertical_fader, SynthLayout, SynthSkin};
use eframe::egui;
use synthcraft_core::{SynthEngine, APP_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    AudioDsp,
    MidiMpe,
    AppearanceRack,
    M1ExtremeTuning,
    ClapDjcraftPaths,
    AiMcpStorage,
}

impl SettingsTab {
    pub fn all() -> &'static [Self] {
        &[
            Self::AudioDsp,
            Self::MidiMpe,
            Self::AppearanceRack,
            Self::M1ExtremeTuning,
            Self::ClapDjcraftPaths,
            Self::AiMcpStorage,
        ]
    }

    pub fn from_subpage(sub: usize) -> Self {
        match sub {
            0 => Self::AudioDsp,
            1 => Self::MidiMpe,
            2 => Self::AppearanceRack,
            3 => Self::M1ExtremeTuning,
            4 => Self::ClapDjcraftPaths,
            _ => Self::AiMcpStorage,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::AudioDsp => 0,
            Self::MidiMpe => 1,
            Self::AppearanceRack => 2,
            Self::M1ExtremeTuning => 3,
            Self::ClapDjcraftPaths => 4,
            Self::AiMcpStorage => 5,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::AudioDsp => "1. Audio & DSP Engine",
            Self::MidiMpe => "2. MIDI, Clock & MPE",
            Self::AppearanceRack => "3. Skins, Rack & Knobs",
            Self::M1ExtremeTuning => "4. Korg M1 & Extreme DSP",
            Self::ClapDjcraftPaths => "5. CLAP, SoundCraft & DJCraft",
            Self::AiMcpStorage => "6. AI Assistant, MCP & SQLite",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StudioSettings {
    // 1. Audio & DSP Engine
    pub audio_backend: String,
    pub output_device: String,
    pub buffer_frames: u32,
    pub oversampling_factor: u32,
    pub max_polyphony_voices: u32,
    pub master_tuning_a4_hz: f32,
    pub master_headroom_db: f32,
    pub soft_clip_limiter: bool,
    pub dc_blocker_enabled: bool,
    pub denormal_flush_simd: bool,

    // 2. MIDI, Clock & MPE
    pub mpe_enabled: bool,
    pub mpe_pitch_bend_range: u8,
    pub midi_clock_mode: String,
    pub velocity_curve: String,
    pub aftertouch_target: String,
    pub mod_wheel_target: String,
    pub midi_thru_echo: bool,

    // 3. Appearance, Rack Cables & Knob Physics
    pub cable_sag_gravity: f32,
    pub cable_opacity: f32,
    pub knob_drag_sensitivity: f32,
    pub vu_meter_decay_db_per_sec: f32,
    pub scope_fps_target: u32,
    pub show_knob_tooltips: bool,
    pub high_contrast_borders: bool,

    // 4. Korg M1 (1990) & Extreme DSP Calibration
    pub m1_dac_32khz_vintage_aliasing: bool,
    pub m1_pcm_interpolation: String,
    pub m1_exciter_presence: f32,
    pub extreme_granular_grain_count: u32,
    pub extreme_wavefolder_stages: u32,
    pub extreme_unison_voices: u32,
    pub extreme_unison_stereo_spread: f32,

    // 5. CLAP 1.2.2, SoundCraft & DJCraft Integration
    pub clap_install_dir: String,
    pub soundcraft_bridge_host: String,
    pub soundcraft_auto_sync_params: bool,
    pub djcraft_stems_dir: String,
    pub wav_bit_depth: String,
    pub djcraft_normalize_lufs: f32,
    pub djcraft_embed_camelot_id3: bool,
    pub djcraft_embed_cue_json: bool,

    // 6. AI Assistant, MCP & SQLite Storage
    pub llm_provider: String,
    pub ollama_endpoint: String,
    pub llm_model_name: String,
    pub llm_temperature: f32,
    pub auto_save_sqlite_on_tweak: bool,
    pub backup_patch_bank_on_exit: bool,
}

impl Default for StudioSettings {
    fn default() -> Self {
        Self {
            audio_backend: "WASAPI Exclusive (Windows Low-Latency)".into(),
            output_device: "Default Studio Monitor Interface (24-bit / 48 kHz)".into(),
            buffer_frames: 128,
            oversampling_factor: 2,
            max_polyphony_voices: 16,
            master_tuning_a4_hz: 440.0,
            master_headroom_db: -1.5,
            soft_clip_limiter: true,
            dc_blocker_enabled: true,
            denormal_flush_simd: true,

            mpe_enabled: true,
            mpe_pitch_bend_range: 48,
            midi_clock_mode: "Internal Master Clock + CLAP Host Sync".into(),
            velocity_curve: "Exponential Studio (Dynamic)".into(),
            aftertouch_target: "Ladder VCF Cutoff + Extreme DSP Morph".into(),
            mod_wheel_target: "LFO Filter Depth + VCO-2 FM Index".into(),
            midi_thru_echo: true,

            cable_sag_gravity: 1.0,
            cable_opacity: 0.92,
            knob_drag_sensitivity: 1.0,
            vu_meter_decay_db_per_sec: 24.0,
            scope_fps_target: 60,
            show_knob_tooltips: true,
            high_contrast_borders: true,

            m1_dac_32khz_vintage_aliasing: true,
            m1_pcm_interpolation: "4-Point Hermite Cubic + MB87404 DAC Warmth".into(),
            m1_exciter_presence: 0.45,
            extreme_granular_grain_count: 24,
            extreme_wavefolder_stages: 5,
            extreme_unison_voices: 7,
            extreme_unison_stereo_spread: 0.80,

            clap_install_dir:
                "C:\\Program Files\\Common Files\\CLAP\\dev.sandraschi.synthcraft.clap".into(),
            soundcraft_bridge_host: "127.0.0.1:10845".into(),
            soundcraft_auto_sync_params: true,
            djcraft_stems_dir: "D:\\DJCraft\\Stems\\SynthCraft".into(),
            wav_bit_depth: "24-Bit Studio PCM (.wav)".into(),
            djcraft_normalize_lufs: -14.0,
            djcraft_embed_camelot_id3: true,
            djcraft_embed_cue_json: true,

            llm_provider: "Local Ollama (Goliath RTX 4090)".into(),
            ollama_endpoint: "http://127.0.0.1:11434".into(),
            llm_model_name: "no model selected".into(),
            llm_temperature: 0.35,
            auto_save_sqlite_on_tweak: true,
            backup_patch_bank_on_exit: true,
        }
    }
}

pub fn render_studio_settings(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let active_tab = SettingsTab::from_subpage(app.subpage);
    ui.heading("Studio Settings, DSP Calibration, Hardware MIDI & Ecosystem Preferences");
    ui.label(
        "Configure real-time audio buffers, oversampling, 1990 Korg M1 32 kHz DAC emulation, Extreme DSP unison, hardware skins, rotary physics, and SoundCraft / DJCraft bridges.",
    );
    ui.separator();

    // Top 6-tab bar
    ui.horizontal_wrapped(|ui| {
        for tab in SettingsTab::all() {
            if ui
                .selectable_label(active_tab == *tab, tab.label())
                .clicked()
            {
                app.subpage = tab.index();
            }
        }
        ui.separator();
        if ui.button("Restore Studio Defaults").clicked() {
            app.settings = StudioSettings::default();
            app.engine.sample_rate_hz = 48_000;
            app.status_line = "Restored all Studio Settings to factory calibration".into();
        }
    });
    ui.separator();

    match SettingsTab::from_subpage(app.subpage) {
        SettingsTab::AudioDsp => render_tab_audio_dsp(app, ui),
        SettingsTab::MidiMpe => render_tab_midi_mpe(app, ui),
        SettingsTab::AppearanceRack => render_tab_appearance_rack(app, ui),
        SettingsTab::M1ExtremeTuning => render_tab_m1_extreme(app, ui),
        SettingsTab::ClapDjcraftPaths => render_tab_clap_djcraft(app, ui),
        SettingsTab::AiMcpStorage => render_tab_ai_mcp_storage(app, ui),
    }
}

fn render_tab_audio_dsp(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.strong("Audio Driver Backend, Sample Rate & Buffer Latency");
            egui::ComboBox::from_label("Audio Driver Backend")
                .selected_text(&app.settings.audio_backend)
                .show_ui(ui, |ui| {
                    for b in [
                        "WASAPI Exclusive (Windows Low-Latency)",
                        "WASAPI Shared (Windows Desktop)",
                        "Steinberg ASIO 2.3 Direct Hardware",
                        "CLAP 1.2.2 Host Audio Thread (SoundCraft / Bitwig / REAPER)",
                        "JACK / PipeWire Pro Audio",
                    ] {
                        ui.selectable_value(&mut app.settings.audio_backend, b.into(), b);
                    }
                });

            egui::ComboBox::from_label("Hardware Output Device")
                .selected_text(&app.settings.output_device)
                .show_ui(ui, |ui| {
                    for d in [
                        "Default Studio Monitor Interface (24-bit / 48 kHz)",
                        "NVIDIA High Definition Audio (Goliath HDMI)",
                        "Focusrite USB ASIO / Scarlett 4i4 Studio",
                        "RME Fireface USB / Babyface Pro FS",
                        "SoundCraft Internal CLAP Audio Bus",
                    ] {
                        ui.selectable_value(&mut app.settings.output_device, d.into(), d);
                    }
                });

            ui.horizontal_wrapped(|ui| {
                ui.label("Engine Sample Rate:");
                for sr in [44_100_u32, 48_000, 88_200, 96_000, 192_000] {
                    if ui
                        .selectable_label(app.engine.sample_rate_hz == sr, format!("{sr} Hz"))
                        .clicked()
                    {
                        app.engine.sample_rate_hz = sr;
                    }
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("Audio Buffer Size:");
                for buf in [32_u32, 64, 128, 256, 512, 1024] {
                    if ui
                        .selectable_label(app.settings.buffer_frames == buf, format!("{buf} smp"))
                        .clicked()
                    {
                        app.settings.buffer_frames = buf;
                    }
                }
            });

            let latency_ms =
                (app.settings.buffer_frames as f32 / app.engine.sample_rate_hz as f32) * 1000.0;
            ui.monospace(format!(
                "Computed Round-Trip Block Latency: {:.2} ms ({} frames @ {} Hz)",
                latency_ms, app.settings.buffer_frames, app.engine.sample_rate_hz
            ));
        });

        cols[1].group(|ui| {
            ui.strong("DSP Oversampling, Voice Polyphony & Master Reference Tuning");
            ui.horizontal_wrapped(|ui| {
                ui.label("Non-Linear VCF / Wavefolder Oversampling:");
                for os in [1_u32, 2, 4, 8] {
                    if ui
                        .selectable_label(
                            app.settings.oversampling_factor == os,
                            format!("{os}x Oversampling"),
                        )
                        .clicked()
                    {
                        app.settings.oversampling_factor = os;
                    }
                }
            });

            ui.add(
                egui::Slider::new(&mut app.settings.max_polyphony_voices, 1..=64)
                    .text("Max Polyphony Voices"),
            );

            ui.horizontal_wrapped(|ui| {
                rotary_knob(
                    ui,
                    &pal,
                    "A4 TUNING",
                    &mut app.settings.master_tuning_a4_hz,
                    430.0,
                    450.0,
                    "Hz",
                );
                vertical_fader(
                    ui,
                    &pal,
                    "HEADROOM",
                    &mut app.settings.master_headroom_db,
                    -12.0,
                    0.0,
                    "dB",
                );
            });

            ui.checkbox(
                &mut app.settings.soft_clip_limiter,
                "Enable Master Analog Tanh Brickwall Soft-Clipper (0.0 dBFS Guard)",
            );
            ui.checkbox(
                &mut app.settings.dc_blocker_enabled,
                "Enable 5 Hz High-Pass Subsonic DC-Offset Blocker",
            );
            ui.checkbox(
                &mut app.settings.denormal_flush_simd,
                "Enable IEEE-754 Subnormal / Denormal Flush-to-Zero (FTZ/DAZ)",
            );
        });
    });
}

fn render_tab_midi_mpe(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.strong("Hardware MIDI Input Routing, Clock Sync & MPE Configuration");
            let ports = app.engine.midi_hw.available_ports.clone();
            egui::ComboBox::from_label("Active Hardware MIDI Port")
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

            ui.add(
                egui::Slider::new(&mut app.engine.midi_hw.midi_channel, 0..=16)
                    .text("MIDI Receive Channel (0 = Omni)"),
            );
            ui.add(
                egui::Slider::new(&mut app.engine.midi_hw.pitch_bend_semitones, 1..=24)
                    .text("Standard Pitch Bend Range (+/- st)"),
            );

            ui.separator();
            ui.checkbox(
                &mut app.settings.mpe_enabled,
                "Enable MPE (MIDI Polyphonic Expression -- Per-Note Pitch/Slide/Pressure)",
            );
            ui.add(
                egui::Slider::new(&mut app.settings.mpe_pitch_bend_range, 12..=96)
                    .text("MPE Per-Note Bend Range (+/- st)"),
            );

            egui::ComboBox::from_label("MIDI Clock & Transport Sync")
                .selected_text(&app.settings.midi_clock_mode)
                .show_ui(ui, |ui| {
                    for m in [
                        "Internal Master Clock + CLAP Host Sync",
                        "External USB/DIN MIDI Clock Slave (24 PPQN)",
                        "Ableton Link / DJCraft Beat-Grid Network Sync",
                        "Free-Running Internal Sequencer Only",
                    ] {
                        ui.selectable_value(&mut app.settings.midi_clock_mode, m.into(), m);
                    }
                });

            egui::ComboBox::from_label("Keyboard Velocity Response Curve")
                .selected_text(&app.settings.velocity_curve)
                .show_ui(ui, |ui| {
                    for v in [
                        "Linear 1:1 Studio Reference",
                        "Exponential Studio (Dynamic)",
                        "Logarithmic Heavy-Keys (Korg M1 Synth Action)",
                        "Fixed Full Velocity (127 Acid / Organ Mode)",
                    ] {
                        ui.selectable_value(&mut app.settings.velocity_curve, v.into(), v);
                    }
                });
            ui.checkbox(
                &mut app.settings.midi_thru_echo,
                "Echo Incoming MIDI Notes to CLAP Event Output / SoundCraft Bus",
            );
        });

        cols[1].group(|ui| {
            ui.strong("MIDI Continuous Controller (CC) & Expression Matrix");
            egui::ComboBox::from_label("Channel Aftertouch Target")
                .selected_text(&app.settings.aftertouch_target)
                .show_ui(ui, |ui| {
                    for t in [
                        "Ladder VCF Cutoff + Extreme DSP Morph",
                        "LFO Vibrato Depth (Korg M1 Joystick +Y)",
                        "Buchla Wavefolder Drive & Harmonics",
                        "VCO-2 FM Modulation Index",
                    ] {
                        ui.selectable_value(&mut app.settings.aftertouch_target, t.into(), t);
                    }
                });

            egui::ComboBox::from_label("Modulation Wheel (CC#01) Target")
                .selected_text(&app.settings.mod_wheel_target)
                .show_ui(ui, |ui| {
                    for t in [
                        "LFO Filter Depth + VCO-2 FM Index",
                        "Korg M1 PCM Layer Crossfade Mix",
                        "Extreme DSP Morph Macro (0..100%)",
                        "4-Pole Ladder Filter Resonance Q",
                    ] {
                        ui.selectable_value(&mut app.settings.mod_wheel_target, t.into(), t);
                    }
                });

            ui.separator();
            ui.strong("Active Hardware MIDI CC Bindings:");
            egui::Grid::new("settings_midi_cc_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("MIDI CC#");
                    ui.strong("SynthCraft Parameter Target");
                    ui.end_row();
                    for (cc, target) in &app.engine.midi_hw.cc_mappings {
                        ui.monospace(format!("CC#{cc:02}"));
                        ui.label(target);
                        ui.end_row();
                    }
                });
        });
    });
}

fn render_tab_appearance_rack(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.strong("Hardware Skin Theme, Studio Layout & UI Scaling");
            for sk in SynthSkin::all() {
                if ui
                    .selectable_label(
                        app.skin == *sk,
                        format!("{} -- {}", sk.label(), sk.description()),
                    )
                    .clicked()
                {
                    app.set_skin(*sk);
                }
            }
            ui.separator();
            ui.strong("Default Active Studio Layout:");
            for lay in SynthLayout::all() {
                if ui
                    .selectable_label(app.layout_mode == *lay, lay.label())
                    .clicked()
                {
                    app.layout_mode = *lay;
                }
            }
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Interface Language:");
                if ui
                    .selectable_label(app.locale == AppLocale::En, "English")
                    .clicked()
                {
                    app.locale = AppLocale::En;
                }
                if ui
                    .selectable_label(app.locale == AppLocale::De, "Deutsch")
                    .clicked()
                {
                    app.locale = AppLocale::De;
                }
            });
            ui.add(
                egui::Slider::new(&mut app.zoom_factor, 0.75..=1.75)
                    .text("Global UI Zoom Factor (Ctrl+Wheel)"),
            );
        });

        cols[1].group(|ui| {
            ui.strong("VCV 3U Patch-Cable Physics, Rotary Knobs & Scope Ballistics");
            ui.horizontal_wrapped(|ui| {
                rotary_knob(
                    ui,
                    &pal,
                    "CABLE SAG",
                    &mut app.settings.cable_sag_gravity,
                    0.2,
                    2.5,
                    "g",
                );
                rotary_knob(
                    ui,
                    &pal,
                    "CABLE ALPHA",
                    &mut app.settings.cable_opacity,
                    0.2,
                    1.0,
                    "%",
                );
                rotary_knob(
                    ui,
                    &pal,
                    "KNOB SPEED",
                    &mut app.settings.knob_drag_sensitivity,
                    0.25,
                    2.5,
                    "x",
                );
                rotary_knob(
                    ui,
                    &pal,
                    "VU DECAY",
                    &mut app.settings.vu_meter_decay_db_per_sec,
                    6.0,
                    48.0,
                    "dB/s",
                );
            });
            ui.add(
                egui::Slider::new(&mut app.settings.scope_fps_target, 24..=144)
                    .text("Oscilloscope & Spectrum Target Refresh (FPS)"),
            );
            ui.checkbox(
                &mut app.settings.show_knob_tooltips,
                "Show Parameter Unit & CLAP Address Tooltips on Rotary Hover",
            );
            ui.checkbox(
                &mut app.settings.high_contrast_borders,
                "Enforce WCAG AAA High-Contrast Module Borders & LED Readouts",
            );
        });
    });
}

fn render_tab_m1_extreme(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.strong("1990 Korg M1 AI2 PCM Rompler & MB87404 32 kHz DAC Calibration");
            ui.checkbox(
                &mut app.settings.m1_dac_32khz_vintage_aliasing,
                "Emulate Authentic 1990 16-Bit / 32 kHz MB87404 DAC Sample-Hold Grit",
            );
            egui::ComboBox::from_label("PCM Multi-Sample Pitch Interpolation")
                .selected_text(&app.settings.m1_pcm_interpolation)
                .show_ui(ui, |ui| {
                    for mode in [
                        "Nearest-Neighbor 1990 Hardware Step (Raw Crunch)",
                        "2-Point Linear Hardware DAC Interpolation",
                        "4-Point Hermite Cubic + MB87404 DAC Warmth",
                        "64-Tap Windowed Sinc (Hi-Fi Studio Remaster)",
                    ] {
                        ui.selectable_value(
                            &mut app.settings.m1_pcm_interpolation,
                            mode.into(),
                            mode,
                        );
                    }
                });
            ui.horizontal_wrapped(|ui| {
                let mut m1_mix = app.engine.active_patch().m1_layer_mix;
                if vertical_fader(ui, &pal, "M1 ROM MIX", &mut m1_mix, 0.0, 1.0, "%") {
                    app.engine.active_patch_mut().m1_layer_mix = m1_mix;
                    app.persist_state();
                }
                rotary_knob(
                    ui,
                    &pal,
                    "AI2 EXCITER",
                    &mut app.settings.m1_exciter_presence,
                    0.0,
                    1.0,
                    "%",
                );
            });
        });

        cols[1].group(|ui| {
            ui.strong("Extreme Modern DSP Generator Calibration (Granular / Buchla / Swarm)");
            ui.add(
                egui::Slider::new(&mut app.settings.extreme_granular_grain_count, 4..=64)
                    .text("Granular Cloud Concurrent Grains"),
            );
            ui.add(
                egui::Slider::new(&mut app.settings.extreme_wavefolder_stages, 1..=9)
                    .text("West-Coast Buchla 259 Wavefolder Stages"),
            );
            ui.add(
                egui::Slider::new(&mut app.settings.extreme_unison_voices, 3..=16)
                    .text("Hyper-Unison Swarm Oscillator Voices"),
            );
            ui.horizontal_wrapped(|ui| {
                rotary_knob(
                    ui,
                    &pal,
                    "STEREO SPREAD",
                    &mut app.settings.extreme_unison_stereo_spread,
                    0.0,
                    1.0,
                    "%",
                );
                let mut morph = app.engine.active_patch().extreme_morph;
                if rotary_knob(ui, &pal, "X-DSP MORPH", &mut morph, 0.0, 1.0, "%") {
                    app.engine.active_patch_mut().extreme_morph = morph;
                    app.persist_state();
                }
            });
        });
    });
}

fn render_tab_clap_djcraft(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.strong("CLAP 1.2.2 Plugin Bundle & SoundCraft Host Bridge");
            ui.label("CLAP Bundle Target Path (.clap):");
            ui.add(
                egui::TextEdit::singleline(&mut app.settings.clap_install_dir)
                    .hint_text("Path to dev.sandraschi.synthcraft.clap")
                    .desired_width(f32::INFINITY),
            );
            ui.label("SoundCraft IPC / OSC Parameter Bridge Endpoint:");
            ui.add(
                egui::TextEdit::singleline(&mut app.settings.soundcraft_bridge_host)
                    .hint_text("127.0.0.1:10845")
                    .desired_width(240.0),
            );
            ui.checkbox(
                &mut app.settings.soundcraft_auto_sync_params,
                "Auto-Broadcast CLAP Polyphonic Modulation Events to SoundCraft",
            );
            ui.small(
                "Plugin ID: dev.sandraschi.synthcraft | ABI: CLAP 1.2.2 (Note-Ports, Params, State, Audio-Ports-Stereo)",
            );
        });

        cols[1].group(|ui| {
            ui.strong("DJCraft Stem Export Directory, Loudness & Camelot Metadata");
            ui.label("DJCraft Stem Output Directory:");
            ui.add(
                egui::TextEdit::singleline(&mut app.settings.djcraft_stems_dir)
                    .hint_text("D:\\DJCraft\\Stems\\SynthCraft")
                    .desired_width(f32::INFINITY),
            );
            egui::ComboBox::from_label("RIFF WAV PCM Export Bit-Depth")
                .selected_text(&app.settings.wav_bit_depth)
                .show_ui(ui, |ui| {
                    for bd in [
                        "16-Bit CD / Club Standard PCM (.wav)",
                        "24-Bit Studio PCM (.wav)",
                        "32-Bit IEEE Float Master Stem (.wav)",
                    ] {
                        ui.selectable_value(&mut app.settings.wav_bit_depth, bd.into(), bd);
                    }
                });
            ui.add(
                egui::Slider::new(&mut app.settings.djcraft_normalize_lufs, -24.0..=-6.0)
                    .text("Stem Integrated Loudness Target (LUFS)"),
            );
            ui.checkbox(
                &mut app.settings.djcraft_embed_camelot_id3,
                "Embed BPM & Camelot Key Tag (e.g. 128bpm_8A) in Filename & RIFF INFO Chunk",
            );
            ui.checkbox(
                &mut app.settings.djcraft_embed_cue_json,
                "Write Sidecar DJCraft Hot-Cue & Phrase-Marker Manifest (.stem.json)",
            );
        });
    });
}

fn render_tab_ai_mcp_storage(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.strong("AI Sound-Designer LLM & MCP 2026-07-28 Server Settings");
            egui::ComboBox::from_label("AI Sound-Designer Provider")
                .selected_text(&app.settings.llm_provider)
                .show_ui(ui, |ui| {
                    for p in [
                        "Local Ollama (Goliath RTX 4090)",
                        "LM Studio Local OpenAI-Compatible Server",
                        "External MCP 2026-07-28 Client Sampling",
                    ] {
                        ui.selectable_value(&mut app.settings.llm_provider, p.into(), p);
                    }
                });
            ui.label("Local LLM Endpoint URL:");
            ui.add(
                egui::TextEdit::singleline(&mut app.settings.ollama_endpoint)
                    .hint_text("http://127.0.0.1:11434")
                    .desired_width(280.0),
            );
            egui::ComboBox::from_label("Active Sound-Designer Model")
                .selected_text(&app.settings.llm_model_name)
                .show_ui(ui, |ui| {
                    for m in [
                        "no model selected",
                        "qwen3.8-coder:32b-instruct-q4_K_M",
                        "llama-3.3-70b-synthcraft-tuned",
                        "deepseek-r1:32b-dsp",
                    ] {
                        ui.selectable_value(&mut app.settings.llm_model_name, m.into(), m);
                    }
                });
            ui.add(
                egui::Slider::new(&mut app.settings.llm_temperature, 0.0..=1.2)
                    .text("Patch Synthesis Creativity (Temperature)"),
            );
            ui.monospace(format!(
                "MCP Server Binary: synthcraft-mcp v{APP_VERSION} (Protocol 2026-07-28 | 19 Tools)"
            ));
        });

        cols[1].group(|ui| {
            ui.strong("SQLite Patch Database, Persistence & Factory Reset");
            ui.monospace(format!("SQLite Database Path: {}", app.db_path.display()));
            ui.monospace(format!(
                "Loaded Factory & User Patches: {} | MIDI Depot Clips: {} | Eurorack Cables: {}",
                app.engine.patches.len(),
                app.engine.midi_depot.len(),
                app.engine.rack_cables.len()
            ));
            ui.checkbox(
                &mut app.settings.auto_save_sqlite_on_tweak,
                "Auto-Commit Patch Tweaks to SQLite Transaction Log on Rotary Release",
            );
            ui.checkbox(
                &mut app.settings.backup_patch_bank_on_exit,
                "Write Rolling .db.bak Snapshot on Studio Shutdown",
            );
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Flush & Save SQLite Now").clicked() {
                    app.persist_state();
                    app.status_line =
                        format!("Saved patch bank & settings to {}", app.db_path.display());
                }
                if ui.button("Reset Factory Patch Bank").clicked() {
                    app.engine = SynthEngine::factory_seed();
                    app.persist_state();
                    app.status_line =
                        "Reset SQLite patch bank to 6 factory patches & default Eurorack cables"
                            .into();
                }
            });
        });
    });
}
