//! SynthCraft L1 Core Engine & Universal Command Bus.
//!
//! Pure-Rust Modular Audio Synthesizer, 16-Step Acid Sequencer, 32-Band DFT
//! Spectrum Analyzer, RIFF `.wav` Renderer, CLAP 1.2.2 Plugin Bridge for SoundCraft,
//! and BPM/Camelot Stem & Riser Forge for DJCraft.
#![forbid(unsafe_code)]

pub mod arp;
pub mod banks;
pub mod clap_host;
pub mod db;
pub mod dsp;
pub mod edit;
pub mod error;
pub mod midi;
pub mod mod_matrix;
pub mod model;
pub mod params;
pub mod plugin;
pub mod rack;
pub mod rack_presets;
pub mod rompler;
pub mod rt;
pub mod sampler;
pub mod sandbox;
pub mod smf_import;
pub mod tuning;
pub mod wavetable;

pub use arp::{ArpMode, ArpParams};
pub use clap_host::{ClapHostConfig, ClapParamInfo, ClapPluginInfo, ExternalClapProcessor};
pub use db::{default_db_path, load_from_db, save_to_db};
pub use error::SynthError;
pub use mod_matrix::{ModDest, ModRoute, ModSource};
pub use model::{
    analyze_patch_spectrum, export_clap_descriptor, export_djcraft_stem_manifest, midi_to_freq_hz,
    stem_duration_ms, synthesize_audio_buffer, write_wav_file, ClapParameterSpec, FilterMode,
    FxParams, SamplerParams, SequencerStep, SpectrumAnalysisReport, SynthPatch, WaveformKind,
    MAX_RENDER_MS,
};
pub use rack::{
    compute_rack_routing, default_rack_modules, euclidean_pattern, RackCategory,
    RackModuleInstance, RackModuleKind,
};
pub use rack_presets::{apply_rack_preset, list_rack_presets, RackPresetInfo, RACK_PRESETS};
pub use rompler::{
    builtin_demo_showcases, builtin_midi_depot, default_rack_cables, export_midi_smf_file,
    DemoShowcaseItem, ExtremeDspMode, MidiDepotClip, MidiHardwareConfig, PcmProgram, RackCable,
    RackRouting,
};
pub use rt::{RtOptions, SynthRt, MAX_VOICES};
pub use smf_import::{import_midi_smf_file, parse_smf_to_steps, ImportedMidiPattern};
pub use tuning::{build_pitch_table, parse_scala_scl, TuningPreset};
pub use wavetable::Wavetable;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthEngine {
    pub active_patch_id: String,
    pub sample_rate_hz: u32,
    pub patches: Vec<SynthPatch>,
    #[serde(default = "default_rack_modules")]
    pub rack_modules: Vec<RackModuleInstance>,
    #[serde(default = "default_rack_cables")]
    pub rack_cables: Vec<RackCable>,
    #[serde(default = "builtin_midi_depot")]
    pub midi_depot: Vec<MidiDepotClip>,
    #[serde(default = "builtin_demo_showcases")]
    pub demos: Vec<DemoShowcaseItem>,
    #[serde(default)]
    pub midi_hw: MidiHardwareConfig,
    #[serde(default)]
    pub clap_host: ClapHostConfig,
    /// Problems found while loading the database (unreadable rows, missing active patch).
    #[serde(skip)]
    pub load_warnings: Vec<String>,
}

impl Default for SynthEngine {
    fn default() -> Self {
        Self::factory_seed()
    }
}

impl SynthEngine {
    pub fn open_or_seed(path: &Path) -> Result<Self, SynthError> {
        if let Some(mut loaded) = load_from_db(path)? {
            if loaded.add_missing_factory_patches() {
                save_to_db(&loaded, path)?;
            }
            return Ok(loaded);
        }
        let seeded = Self::factory_seed();
        save_to_db(&seeded, path)?;
        Ok(seeded)
    }

    /// Append factory patches that an older database does not have yet. Existing patches are
    /// never touched. Returns true when something was added.
    pub fn add_missing_factory_patches(&mut self) -> bool {
        let seed = Self::factory_seed();
        let mut added = false;
        for p in seed.patches {
            if !self.patches.iter().any(|q| q.id == p.id) {
                self.patches.push(p);
                added = true;
            }
        }
        added
    }

    pub fn save_db(&self, path: &Path) -> Result<(), SynthError> {
        save_to_db(self, path)
    }

    pub fn factory_seed() -> Self {
        let mk_steps = |notes: &[u8]| -> Vec<SequencerStep> {
            notes
                .iter()
                .enumerate()
                .map(|(i, &n)| SequencerStep {
                    step_index: i,
                    midi_note: n,
                    velocity: if i % 4 == 0 { 0.95 } else { 0.78 },
                    gate_pct: if i % 2 == 0 { 0.75 } else { 0.55 },
                    accent: i % 4 == 0,
                    slide: i % 5 == 3,
                    active: true,
                })
                .collect()
        };

        let mut patches = vec![
            SynthPatch {
                id: "vienna_acid_303".into(),
                name: "Vienna Acid 303 Bassline".into(),
                category: "Acid Bass / DJCraft Loop".into(),
                bpm: 128.0,
                camelot_key: "8A Am".into(),
                vco1_wave: WaveformKind::Saw,
                vco2_wave: WaveformKind::SubOctave,
                vco2_detune_cents: 7.0,
                vco_mix: 0.25,
                fm_index: 0.15,
                pulse_width: 0.5,
                filter_mode: FilterMode::AcidResonant,
                cutoff_hz: 680.0,
                resonance_q: 7.8,
                env_mod_depth: 0.85,
                attack_ms: 4.0,
                decay_ms: 190.0,
                sustain_level: 0.25,
                release_ms: 80.0,
                lfo_rate_hz: 2.0,
                lfo_depth: 0.18,
                drive_saturation: 1.4,
                master_gain: 0.85,
                m1_program: None,
                m1_layer_mix: 0.50,
                extreme_dsp: ExtremeDspMode::Off,
                extreme_morph: 0.50,
                sampler: SamplerParams::default(),
                fx: FxParams::default(),
                mod_matrix: Vec::new(),
                arp: ArpParams::default(),
                tuning: TuningPreset::default(),
                wavetable: "analog_morph".into(),
                steps: mk_steps(&[
                    45, 45, 57, 45, 48, 45, 55, 52, 45, 45, 60, 45, 50, 52, 55, 57,
                ]),
            },
            SynthPatch {
                id: "goliath_reese_sub".into(),
                name: "Goliath Reese Sub Bass".into(),
                category: "DnB / Neuro Sub".into(),
                bpm: 174.0,
                camelot_key: "4A Fm".into(),
                vco1_wave: WaveformKind::SuperSaw,
                vco2_wave: WaveformKind::Saw,
                vco2_detune_cents: 24.0,
                vco_mix: 0.5,
                fm_index: 0.05,
                pulse_width: 0.5,
                filter_mode: FilterMode::LowPass24dB,
                cutoff_hz: 320.0,
                resonance_q: 3.2,
                env_mod_depth: 0.35,
                attack_ms: 18.0,
                decay_ms: 420.0,
                sustain_level: 0.80,
                release_ms: 210.0,
                lfo_rate_hz: 1.45,
                lfo_depth: 0.42,
                drive_saturation: 1.1,
                master_gain: 0.88,
                m1_program: Some(PcmProgram::OrchestraHit48),
                m1_layer_mix: 0.25,
                extreme_dsp: ExtremeDspMode::WavetableMorph,
                extreme_morph: 0.65,
                sampler: SamplerParams::default(),
                fx: FxParams::default(),
                mod_matrix: Vec::new(),
                arp: ArpParams::default(),
                tuning: TuningPreset::default(),
                wavetable: "analog_morph".into(),
                steps: mk_steps(&[
                    41, 41, 41, 41, 44, 44, 44, 44, 39, 39, 39, 39, 36, 36, 39, 41,
                ]),
            },
            SynthPatch {
                id: "danube_dub_chord".into(),
                name: "Danube Dub Techno Chord".into(),
                category: "Dub Stabs / SoundCraft".into(),
                bpm: 124.0,
                camelot_key: "2A D#m".into(),
                vco1_wave: WaveformKind::PulsePwm,
                vco2_wave: WaveformKind::Triangle,
                vco2_detune_cents: 11.0,
                vco_mix: 0.40,
                fm_index: 0.30,
                pulse_width: 0.32,
                filter_mode: FilterMode::BandPass12dB,
                cutoff_hz: 1150.0,
                resonance_q: 4.5,
                env_mod_depth: 0.70,
                attack_ms: 12.0,
                decay_ms: 340.0,
                sustain_level: 0.40,
                release_ms: 320.0,
                lfo_rate_hz: 0.65,
                lfo_depth: 0.55,
                drive_saturation: 0.65,
                master_gain: 0.80,
                m1_program: Some(PcmProgram::Organ2_17),
                m1_layer_mix: 0.45,
                extreme_dsp: ExtremeDspMode::Off,
                extreme_morph: 0.40,
                sampler: SamplerParams::default(),
                fx: FxParams::default(),
                mod_matrix: Vec::new(),
                arp: ArpParams::default(),
                tuning: TuningPreset::default(),
                wavetable: "analog_morph".into(),
                steps: mk_steps(&[
                    51, 51, 58, 51, 54, 51, 61, 58, 51, 51, 58, 54, 51, 54, 58, 63,
                ]),
            },
            SynthPatch {
                id: "stephansdom_bell_fm".into(),
                name: "Stephansdom FM Bell Pluck".into(),
                category: "FM Lead / Pluck".into(),
                bpm: 120.0,
                camelot_key: "9A Em".into(),
                vco1_wave: WaveformKind::Sine,
                vco2_wave: WaveformKind::Sine,
                vco2_detune_cents: 14.0,
                vco_mix: 0.35,
                fm_index: 2.1,
                pulse_width: 0.5,
                filter_mode: FilterMode::LowPass24dB,
                cutoff_hz: 3800.0,
                resonance_q: 2.1,
                env_mod_depth: 0.60,
                attack_ms: 2.0,
                decay_ms: 280.0,
                sustain_level: 0.15,
                release_ms: 240.0,
                lfo_rate_hz: 4.5,
                lfo_depth: 0.12,
                drive_saturation: 0.25,
                master_gain: 0.82,
                m1_program: Some(PcmProgram::Piano16_01),
                m1_layer_mix: 0.55,
                extreme_dsp: ExtremeDspMode::KarplusStrongPluck,
                extreme_morph: 0.50,
                sampler: SamplerParams::default(),
                fx: FxParams::default(),
                mod_matrix: Vec::new(),
                arp: ArpParams::default(),
                tuning: TuningPreset::default(),
                wavetable: "analog_morph".into(),
                steps: mk_steps(&[
                    64, 67, 71, 76, 74, 71, 67, 64, 66, 69, 71, 74, 71, 69, 66, 64,
                ]),
            },
            SynthPatch {
                id: "djcraft_riser_sweep".into(),
                name: "DJCraft 4-Bar Drop Riser".into(),
                category: "FX Transition / DJCraft".into(),
                bpm: 128.0,
                camelot_key: "8A Am".into(),
                vco1_wave: WaveformKind::WhiteNoise,
                vco2_wave: WaveformKind::SuperSaw,
                vco2_detune_cents: 35.0,
                vco_mix: 0.60,
                fm_index: 0.80,
                pulse_width: 0.5,
                filter_mode: FilterMode::BandPass12dB,
                cutoff_hz: 2400.0,
                resonance_q: 6.4,
                env_mod_depth: 0.95,
                attack_ms: 140.0,
                decay_ms: 600.0,
                sustain_level: 0.85,
                release_ms: 350.0,
                lfo_rate_hz: 6.0,
                lfo_depth: 0.75,
                drive_saturation: 0.90,
                master_gain: 0.78,
                m1_program: None,
                m1_layer_mix: 0.0,
                extreme_dsp: ExtremeDspMode::PhaseDistortionCz,
                extreme_morph: 0.70,
                sampler: SamplerParams::default(),
                fx: FxParams::default(),
                mod_matrix: Vec::new(),
                arp: ArpParams::default(),
                tuning: TuningPreset::default(),
                wavetable: "analog_morph".into(),
                steps: mk_steps(&[
                    57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72,
                ]),
            },
            SynthPatch {
                id: "soundcraft_warm_pad".into(),
                name: "SoundCraft Analog Strings Pad".into(),
                category: "Poly Pad / SoundCraft CLAP".into(),
                bpm: 110.0,
                camelot_key: "11B A".into(),
                vco1_wave: WaveformKind::SuperSaw,
                vco2_wave: WaveformKind::Triangle,
                vco2_detune_cents: 18.0,
                vco_mix: 0.45,
                fm_index: 0.10,
                pulse_width: 0.5,
                filter_mode: FilterMode::LowPass24dB,
                cutoff_hz: 1650.0,
                resonance_q: 1.8,
                env_mod_depth: 0.40,
                attack_ms: 220.0,
                decay_ms: 500.0,
                sustain_level: 0.75,
                release_ms: 480.0,
                lfo_rate_hz: 0.85,
                lfo_depth: 0.30,
                drive_saturation: 0.35,
                master_gain: 0.80,
                m1_program: Some(PcmProgram::Universe00),
                m1_layer_mix: 0.65,
                extreme_dsp: ExtremeDspMode::GranularCloud,
                extreme_morph: 0.55,
                sampler: SamplerParams::default(),
                fx: FxParams::default(),
                mod_matrix: Vec::new(),
                arp: ArpParams::default(),
                tuning: TuningPreset::default(),
                wavetable: "analog_morph".into(),
                steps: mk_steps(&[
                    57, 57, 61, 61, 64, 64, 69, 69, 68, 68, 64, 64, 61, 61, 57, 57,
                ]),
            },
        ];

        patches.extend(ambient_patches(&mk_steps));

        Self {
            active_patch_id: "vienna_acid_303".into(),
            sample_rate_hz: 48000,
            patches,
            rack_modules: default_rack_modules(),
            rack_cables: default_rack_cables(),
            midi_depot: builtin_midi_depot(),
            demos: builtin_demo_showcases(),
            midi_hw: MidiHardwareConfig::default(),
            clap_host: ClapHostConfig::default(),
            load_warnings: Vec::new(),
        }
    }

    pub fn active_patch(&self) -> &SynthPatch {
        self.patches
            .iter()
            .find(|p| p.id == self.active_patch_id)
            .unwrap_or(&self.patches[0])
    }

    pub fn active_patch_mut(&mut self) -> &mut SynthPatch {
        let id = self.active_patch_id.clone();
        if let Some(pos) = self.patches.iter().position(|p| p.id == id) {
            &mut self.patches[pos]
        } else {
            &mut self.patches[0]
        }
    }

    pub fn select_patch(&mut self, patch_id: &str) -> Result<&SynthPatch, SynthError> {
        if !self.patches.iter().any(|p| p.id == patch_id) {
            return Err(SynthError::NotFound(patch_id.to_string()));
        }
        self.active_patch_id = patch_id.to_string();
        Ok(self.active_patch())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "params", rename_all = "snake_case")]
pub enum Command {
    Status,
    Summary,
    ListPatches,
    LoadPatch {
        patch_id: String,
    },
    UpdateOscillators {
        vco1_wave: String,
        vco2_wave: String,
        detune_cents: f32,
        mix: f32,
        fm_index: f32,
    },
    UpdateFilter {
        mode: String,
        cutoff_hz: f32,
        resonance_q: f32,
        env_mod_depth: f32,
    },
    UpdateEnvelopeLfo {
        attack_ms: f32,
        decay_ms: f32,
        sustain_level: f32,
        release_ms: f32,
        lfo_rate_hz: f32,
        lfo_depth: f32,
    },
    SetSequencerStep {
        step_index: usize,
        midi_note: u8,
        accent: bool,
        slide: bool,
        active: bool,
    },
    SynthesizePreview {
        duration_ms: u32,
    },
    AnalyzeSpectrum {
        duration_ms: u32,
    },
    RenderWavFile {
        output_path: String,
        duration_ms: u32,
    },
    ExportClapManifest,
    /// Stem metadata for `bars` bars; with `output_path` the mono loop is also rendered to a WAV.
    ExportDjcraftStem {
        bars: usize,
        output_path: Option<String>,
    },
    LoadPcmProgram {
        program: String,
        mix: f32,
    },
    SetExtremeDsp {
        mode: String,
        morph: f32,
    },
    /// Load a sample bank (spec as in `sampler::load_bank`) into the active patch, or `None`
    /// to unload. The bank is decoded now so a bad path fails here.
    SetSamplerBank {
        bank: Option<String>,
        mix: f32,
    },
    /// Master chorus and reverb of the active patch (all values 0..=1 except decay seconds).
    SetFx {
        chorus_mix: f32,
        reverb_mix: f32,
        reverb_decay_s: f32,
        reverb_damping: f32,
    },
    LoadMidiDepotClip {
        clip_id: String,
    },
    ExportMidiFile {
        clip_id: String,
        output_path: String,
    },
    RunDemoShowcase {
        demo_id: String,
    },
    /// Set one scalar parameter of the active patch (see `edit::PATCH_PARAMS`, `stepN.velocity`,
    /// `stepN.gate_pct`).
    SetPatchParam {
        param: String,
        value: f32,
    },
    /// Set the Camelot key label of the active patch (for example `8A Am`).
    SetCamelotKey {
        key: String,
    },
    ListRackCables,
    ConnectRackCable {
        from_module: String,
        from_jack: String,
        to_module: String,
        to_jack: String,
        color_name: Option<String>,
        gain: Option<f32>,
    },
    DisconnectRackCable {
        cable_id: String,
    },
    ResetRackCables,
    ListRackModules,
    AddRackModule {
        kind: String,
        row: Option<usize>,
    },
    RemoveRackModule {
        module_id: String,
    },
    MoveRackModule {
        module_id: String,
        delta_row: i32,
        delta_slot: i32,
    },
    SetRackModuleParam {
        module_id: String,
        param_a: Option<f32>,
        param_b: Option<f32>,
        bypassed: Option<bool>,
    },
    ResetRackLayout,
    ListRackPresets,
    LoadRackPreset {
        preset_id: String,
    },
    SetModRoute {
        index: usize,
        source: String,
        dest: String,
        depth: f32,
        enabled: bool,
    },
    ClearModMatrix,
    SetArpeggiator {
        mode: String,
        rate_div: u8,
        octaves: u8,
        gate: f32,
        latch: bool,
    },
    SetTuning {
        preset: String,
        scala_scl: Option<String>,
    },
    LoadWavetable {
        spec: String,
    },
    ImportMidiFile {
        path: String,
    },
    ConfigureClapHost {
        plugin_path: Option<String>,
        plugin_index: u32,
        cv_param_id: Option<u32>,
        mix: f32,
        cv_depth: f32,
    },
    ResetFactoryPatches,
}

impl Command {
    pub fn dispatch(&self, engine: &mut SynthEngine) -> Result<Value, SynthError> {
        match self {
            Self::Status => {
                let p = engine.active_patch();
                Ok(json!({
                    "app": "synthcraft",
                    "version": APP_VERSION,
                    "status": "ready",
                    "active_patch_id": p.id,
                    "active_patch_name": p.name,
                    "bpm": p.bpm,
                    "camelot_key": p.camelot_key,
                    "sample_rate_hz": engine.sample_rate_hz,
                    "patch_count": engine.patches.len(),
                    "clap_plugin_id": "dev.sandraschi.synthcraft",
                }))
            }
            Self::Summary => Ok(json!({
                "active_patch_id": engine.active_patch_id,
                "sample_rate_hz": engine.sample_rate_hz,
                "active_patch": engine.active_patch(),
                "patch_count": engine.patches.len(),
            })),
            Self::ListPatches => {
                let list: Vec<Value> = engine
                    .patches
                    .iter()
                    .map(|p| {
                        json!({
                            "id": p.id,
                            "name": p.name,
                            "category": p.category,
                            "bpm": p.bpm,
                            "camelot_key": p.camelot_key,
                            "vco1": p.vco1_wave.as_str(),
                            "vco2": p.vco2_wave.as_str(),
                            "filter": p.filter_mode.as_str(),
                        })
                    })
                    .collect();
                Ok(json!({ "patches": list }))
            }
            Self::LoadPatch { patch_id } => {
                let p = engine.select_patch(patch_id)?.clone();
                Ok(json!({ "loaded": p }))
            }
            Self::UpdateOscillators {
                vco1_wave,
                vco2_wave,
                detune_cents,
                mix,
                fm_index,
            } => {
                let w1 = WaveformKind::parse(vco1_wave).ok_or_else(|| {
                    SynthError::Validation(format!("Invalid VCO1 waveform '{vco1_wave}'"))
                })?;
                let w2 = WaveformKind::parse(vco2_wave).ok_or_else(|| {
                    SynthError::Validation(format!("Invalid VCO2 waveform '{vco2_wave}'"))
                })?;
                let p = engine.active_patch_mut();
                p.vco1_wave = w1;
                p.vco2_wave = w2;
                p.vco2_detune_cents = detune_cents.clamp(-100.0, 100.0);
                p.vco_mix = mix.clamp(0.0, 1.0);
                p.fm_index = fm_index.clamp(0.0, 4.0);
                Ok(json!({ "updated_patch": p.clone() }))
            }
            Self::UpdateFilter {
                mode,
                cutoff_hz,
                resonance_q,
                env_mod_depth,
            } => {
                let fm = FilterMode::parse(mode).ok_or_else(|| {
                    SynthError::Validation(format!("Invalid filter mode '{mode}'"))
                })?;
                let p = engine.active_patch_mut();
                p.filter_mode = fm;
                p.cutoff_hz = cutoff_hz.clamp(35.0, 16000.0);
                p.resonance_q = resonance_q.clamp(0.5, 12.0);
                p.env_mod_depth = env_mod_depth.clamp(0.0, 1.0);
                Ok(json!({ "updated_patch": p.clone() }))
            }
            Self::UpdateEnvelopeLfo {
                attack_ms,
                decay_ms,
                sustain_level,
                release_ms,
                lfo_rate_hz,
                lfo_depth,
            } => {
                let p = engine.active_patch_mut();
                p.attack_ms = attack_ms.clamp(1.0, 2000.0);
                p.decay_ms = decay_ms.clamp(5.0, 4000.0);
                p.sustain_level = sustain_level.clamp(0.0, 1.0);
                p.release_ms = release_ms.clamp(5.0, 4000.0);
                p.lfo_rate_hz = lfo_rate_hz.clamp(0.05, 30.0);
                p.lfo_depth = lfo_depth.clamp(0.0, 1.0);
                Ok(json!({ "updated_patch": p.clone() }))
            }
            Self::SetSequencerStep {
                step_index,
                midi_note,
                accent,
                slide,
                active,
            } => {
                let p = engine.active_patch_mut();
                let step = p.steps.get_mut(*step_index).ok_or_else(|| {
                    SynthError::Validation(format!("Step index {step_index} out of bounds (0..15)"))
                })?;
                step.midi_note = (*midi_note).clamp(24, 96);
                step.accent = *accent;
                step.slide = *slide;
                step.active = *active;
                Ok(json!({ "step": step.clone() }))
            }
            Self::SynthesizePreview { duration_ms } => {
                let samples = synthesize_audio_buffer(
                    engine.active_patch(),
                    engine.sample_rate_hz,
                    *duration_ms,
                );
                let preview: Vec<f32> = samples.iter().take(64).copied().collect();
                Ok(json!({
                    "patch_id": engine.active_patch_id,
                    "sample_rate_hz": engine.sample_rate_hz,
                    "total_samples": samples.len(),
                    "first_64_samples": preview,
                }))
            }
            Self::AnalyzeSpectrum { duration_ms } => {
                let rep = analyze_patch_spectrum(
                    engine.active_patch(),
                    engine.sample_rate_hz,
                    *duration_ms,
                );
                serde_json::to_value(rep).map_err(|e| SynthError::Serialization(e.to_string()))
            }
            Self::RenderWavFile {
                output_path,
                duration_ms,
            } => {
                let p = PathBuf::from(output_path);
                let count = write_wav_file(
                    engine.active_patch(),
                    &p,
                    engine.sample_rate_hz,
                    *duration_ms,
                )?;
                Ok(json!({
                    "written_wav": p.display().to_string(),
                    "sample_rate_hz": engine.sample_rate_hz,
                    "sample_count": count,
                    "duration_ms": duration_ms,
                }))
            }
            Self::ExportClapManifest => Ok(export_clap_descriptor(engine.active_patch())),
            Self::ExportDjcraftStem { bars, output_path } => {
                let sr = engine.sample_rate_hz;
                let mut manifest = export_djcraft_stem_manifest(engine.active_patch(), *bars, sr);
                if let Some(path) = output_path {
                    let ms = stem_duration_ms(engine.active_patch(), *bars);
                    let frames = write_wav_file(engine.active_patch(), Path::new(path), sr, ms)?;
                    manifest["wav_path"] = json!(path);
                    manifest["wav_frames"] = json!(frames);
                }
                Ok(manifest)
            }
            Self::SetPatchParam { param, value } => {
                let v = edit::set_patch_param(engine.active_patch_mut(), param, *value)?;
                Ok(json!({ "param": param, "value": v }))
            }
            Self::SetCamelotKey { key } => {
                let k = key.trim();
                if k.is_empty() || k.len() > 16 {
                    return Err(SynthError::Validation(
                        "Camelot key must be 1 to 16 characters".into(),
                    ));
                }
                engine.active_patch_mut().camelot_key = k.to_string();
                Ok(json!({ "camelot_key": k }))
            }
            Self::LoadPcmProgram { program, mix } => {
                let p = engine.active_patch_mut();
                if program.trim().eq_ignore_ascii_case("off")
                    || program.trim().eq_ignore_ascii_case("none")
                {
                    p.m1_program = None;
                } else {
                    let m1 = PcmProgram::parse(program).ok_or_else(|| {
                        SynthError::Validation(format!(
                            "Unknown PCM workstation 1990 program '{program}'. Expected m1_00_universe, m1_01_piano16, m1_11_fretless, m1_12_slapbass, m1_17_organ2, m1_24_panflute, m1_36_choir, or m1_48_orchhit."
                        ))
                    })?;
                    p.m1_program = Some(m1);
                }
                p.m1_layer_mix = mix.clamp(0.0, 1.0);
                Ok(json!({
                    "m1_program": p.m1_program.map(PcmProgram::as_str),
                    "m1_layer_mix": p.m1_layer_mix,
                }))
            }
            Self::SetExtremeDsp { mode, morph } => {
                let x = ExtremeDspMode::parse(mode).ok_or_else(|| {
                    SynthError::Validation(format!("Unknown Extreme DSP mode '{mode}'"))
                })?;
                let p = engine.active_patch_mut();
                p.extreme_dsp = x;
                p.extreme_morph = morph.clamp(0.0, 1.0);
                Ok(json!({
                    "extreme_dsp": p.extreme_dsp.as_str(),
                    "extreme_morph": p.extreme_morph,
                }))
            }
            Self::SetSamplerBank { bank, mix } => {
                let info = match bank {
                    Some(spec) => {
                        let b = sampler::load_cached(spec)?;
                        json!({ "name": b.name, "zones": b.zones.len(), "frames": b.total_frames() })
                    }
                    None => json!(null),
                };
                let p = engine.active_patch_mut();
                p.sampler.bank.clone_from(bank);
                p.sampler.mix = mix.clamp(0.0, 1.0);
                Ok(json!({ "bank": p.sampler.bank, "mix": p.sampler.mix, "loaded": info }))
            }
            Self::SetFx {
                chorus_mix,
                reverb_mix,
                reverb_decay_s,
                reverb_damping,
            } => {
                let fx = &mut engine.active_patch_mut().fx;
                fx.chorus_mix = chorus_mix.clamp(0.0, 1.0);
                fx.reverb_mix = reverb_mix.clamp(0.0, 1.0);
                fx.reverb_decay_s = reverb_decay_s.clamp(0.2, 20.0);
                fx.reverb_damping = reverb_damping.clamp(0.0, 0.95);
                Ok(json!(*fx))
            }
            Self::LoadMidiDepotClip { clip_id } => {
                let clip = engine
                    .midi_depot
                    .iter()
                    .find(|c| c.id == *clip_id)
                    .cloned()
                    .ok_or_else(|| SynthError::NotFound(clip_id.clone()))?;
                let p = engine.active_patch_mut();
                p.bpm = clip.bpm;
                p.camelot_key = clip.camelot_key.clone();
                if let Some(ref m1_str) = clip.recommended_m1 {
                    p.m1_program = PcmProgram::parse(m1_str);
                }
                if let Some(xd) = ExtremeDspMode::parse(&clip.recommended_extreme_dsp) {
                    p.extreme_dsp = xd;
                }
                for (i, &note) in clip.notes.iter().enumerate() {
                    if let Some(step) = p.steps.get_mut(i) {
                        step.midi_note = note;
                        step.active = true;
                    }
                }
                Ok(json!({
                    "loaded_midi_clip": clip.id,
                    "title": clip.title,
                    "bpm": p.bpm,
                    "camelot_key": p.camelot_key,
                }))
            }
            Self::ExportMidiFile {
                clip_id,
                output_path,
            } => {
                let clip = engine
                    .midi_depot
                    .iter()
                    .find(|c| c.id == *clip_id)
                    .ok_or_else(|| SynthError::NotFound(clip_id.clone()))?;
                let path = PathBuf::from(output_path);
                let bytes = export_midi_smf_file(clip, &path)?;
                Ok(json!({
                    "written_mid": path.display().to_string(),
                    "clip_id": clip_id,
                    "bytes": bytes,
                }))
            }
            Self::RunDemoShowcase { demo_id } => {
                let demo = engine
                    .demos
                    .iter()
                    .find(|d| d.id == *demo_id)
                    .cloned()
                    .ok_or_else(|| SynthError::NotFound(demo_id.clone()))?;
                let _ = engine.select_patch(&demo.patch_id)?;
                let _ = Self::LoadMidiDepotClip {
                    clip_id: demo.midi_clip_id.clone(),
                }
                .dispatch(engine)?;
                {
                    let p = engine.active_patch_mut();
                    p.m1_program = demo.m1_program;
                    p.extreme_dsp = demo.extreme_dsp;
                    p.bpm = demo.bpm;
                }
                let rep = analyze_patch_spectrum(engine.active_patch(), engine.sample_rate_hz, 200);
                Ok(json!({
                    "demo": demo,
                    "rms_dbfs": rep.rms_dbfs,
                    "peak_dbfs": rep.peak_dbfs,
                    "estimated_pitch_hz": rep.estimated_pitch_hz,
                }))
            }
            Self::ListRackCables => Ok(json!({
                "cables": engine.rack_cables,
                "count": engine.rack_cables.len(),
            })),
            Self::ConnectRackCable {
                from_module,
                from_jack,
                to_module,
                to_jack,
                color_name,
                gain,
            } => {
                if from_module.trim().is_empty()
                    || from_jack.trim().is_empty()
                    || to_module.trim().is_empty()
                    || to_jack.trim().is_empty()
                {
                    return Err(SynthError::Validation(
                        "Rack cable endpoints cannot be empty".into(),
                    ));
                }
                if from_module == to_module && from_jack == to_jack {
                    return Err(SynthError::Validation(
                        "Cannot patch a jack directly to itself".into(),
                    ));
                }
                let palette = ["green", "cyan", "amber", "purple", "red"];
                let color = color_name
                    .clone()
                    .filter(|c| !c.trim().is_empty())
                    .unwrap_or_else(|| palette[engine.rack_cables.len() % palette.len()].into());
                let g = gain.unwrap_or(1.0).clamp(0.0, 2.0);
                // Replace if an identical endpoint pair already exists, otherwise append.
                if let Some(existing) = engine.rack_cables.iter_mut().find(|c| {
                    c.from_module == *from_module
                        && c.from_jack == *from_jack
                        && c.to_module == *to_module
                        && c.to_jack == *to_jack
                }) {
                    existing.color_name = color;
                    existing.gain = g;
                    let updated = existing.clone();
                    return Ok(json!({ "cable": updated, "count": engine.rack_cables.len() }));
                }
                let mut seq = engine.rack_cables.len() + 1;
                let id = loop {
                    let candidate = format!("cab_{seq}");
                    if !engine.rack_cables.iter().any(|c| c.id == candidate) {
                        break candidate;
                    }
                    seq += 1;
                };
                let cable = RackCable {
                    id,
                    from_module: from_module.clone(),
                    from_jack: from_jack.clone(),
                    to_module: to_module.clone(),
                    to_jack: to_jack.clone(),
                    color_name: color,
                    gain: g,
                };
                engine.rack_cables.push(cable.clone());
                Ok(json!({ "cable": cable, "count": engine.rack_cables.len() }))
            }
            Self::DisconnectRackCable { cable_id } => {
                let before = engine.rack_cables.len();
                engine.rack_cables.retain(|c| c.id != *cable_id);
                if engine.rack_cables.len() == before {
                    return Err(SynthError::NotFound(format!("Rack cable '{cable_id}'")));
                }
                Ok(json!({
                    "removed": cable_id,
                    "count": engine.rack_cables.len(),
                }))
            }
            Self::ResetRackCables => {
                engine.rack_cables = default_rack_cables();
                Ok(json!({
                    "reset": true,
                    "count": engine.rack_cables.len(),
                }))
            }
            Self::ListRackModules => Ok(json!({
                "modules": engine.rack_modules,
                "count": engine.rack_modules.len(),
            })),
            Self::AddRackModule { kind, row } => {
                let parsed = RackModuleKind::parse(kind).ok_or_else(|| {
                    SynthError::Validation(format!("Unknown rack module kind '{kind}'"))
                })?;
                let target_row = row.unwrap_or(1).min(2);
                let mut seq = engine.rack_modules.len() + 1;
                let id = loop {
                    let cand = format!("{}-{seq}", parsed.as_str().to_uppercase());
                    if !engine.rack_modules.iter().any(|m| m.id == cand) {
                        break cand;
                    }
                    seq += 1;
                };
                let (pa, pb) = parsed.default_params();
                let inst = RackModuleInstance {
                    id,
                    kind: parsed,
                    row: target_row,
                    bypassed: false,
                    param_a: pa,
                    param_b: pb,
                };
                engine.rack_modules.push(inst.clone());
                Ok(json!({ "module": inst, "count": engine.rack_modules.len() }))
            }
            Self::RemoveRackModule { module_id } => {
                let before = engine.rack_modules.len();
                engine.rack_modules.retain(|m| m.id != *module_id);
                if engine.rack_modules.len() == before {
                    return Err(SynthError::NotFound(format!("Rack module '{module_id}'")));
                }
                engine
                    .rack_cables
                    .retain(|c| c.from_module != *module_id && c.to_module != *module_id);
                Ok(json!({
                    "removed": module_id,
                    "count": engine.rack_modules.len(),
                }))
            }
            Self::MoveRackModule {
                module_id,
                delta_row,
                delta_slot,
            } => {
                let Some(idx) = engine.rack_modules.iter().position(|m| m.id == *module_id) else {
                    return Err(SynthError::NotFound(format!("Rack module '{module_id}'")));
                };
                if *delta_row != 0 {
                    let cur = engine.rack_modules[idx].row as i32;
                    engine.rack_modules[idx].row = (cur + *delta_row).clamp(0, 2) as usize;
                }
                if *delta_slot != 0 {
                    let row = engine.rack_modules[idx].row;
                    let row_indices: Vec<usize> = engine
                        .rack_modules
                        .iter()
                        .enumerate()
                        .filter(|(_, m)| m.row == row)
                        .map(|(i, _)| i)
                        .collect();
                    if let Some(pos) = row_indices.iter().position(|&i| i == idx) {
                        let new_pos = (pos as i32 + *delta_slot)
                            .clamp(0, row_indices.len() as i32 - 1)
                            as usize;
                        if new_pos != pos {
                            engine.rack_modules.swap(idx, row_indices[new_pos]);
                        }
                    }
                }
                Ok(json!({ "moved": module_id }))
            }
            Self::SetRackModuleParam {
                module_id,
                param_a,
                param_b,
                bypassed,
            } => {
                let Some(m) = engine.rack_modules.iter_mut().find(|m| m.id == *module_id) else {
                    return Err(SynthError::NotFound(format!("Rack module '{module_id}'")));
                };
                if let Some(a) = param_a {
                    m.param_a = a.clamp(0.0, 1.0);
                }
                if let Some(b) = param_b {
                    m.param_b = b.clamp(0.0, 1.0);
                }
                if let Some(bp) = bypassed {
                    m.bypassed = *bp;
                }
                Ok(json!({ "module": m.clone() }))
            }
            Self::ResetRackLayout => {
                engine.rack_modules = default_rack_modules();
                engine.rack_cables = default_rack_cables();
                Ok(json!({
                    "reset": true,
                    "modules": engine.rack_modules.len(),
                    "cables": engine.rack_cables.len(),
                }))
            }
            Self::ListRackPresets => Ok(json!({
                "presets": list_rack_presets(),
                "count": RACK_PRESETS.len(),
            })),
            Self::LoadRackPreset { preset_id } => {
                let info = apply_rack_preset(engine, preset_id)?;
                Ok(json!({
                    "loaded_preset": info,
                    "modules": engine.rack_modules.len(),
                    "cables": engine.rack_cables.len(),
                }))
            }
            Self::SetModRoute {
                index,
                source,
                dest,
                depth,
                enabled,
            } => {
                if *index >= 8 {
                    return Err(SynthError::Validation(format!(
                        "Modulation slot {index} out of range (0..8)"
                    )));
                }
                let src = ModSource::parse(source).ok_or_else(|| {
                    SynthError::Validation(format!("Unknown modulation source '{source}'"))
                })?;
                let dst = ModDest::parse(dest).ok_or_else(|| {
                    SynthError::Validation(format!("Unknown modulation destination '{dest}'"))
                })?;
                let route = ModRoute {
                    source: src,
                    dest: dst,
                    depth: depth.clamp(-1.0, 1.0),
                    enabled: *enabled,
                };
                let m = &mut engine.active_patch_mut().mod_matrix;
                if *index < m.len() {
                    m[*index] = route;
                } else {
                    m.push(route);
                }
                Ok(json!({ "mod_matrix": m }))
            }
            Self::ClearModMatrix => {
                engine.active_patch_mut().mod_matrix.clear();
                Ok(json!({ "cleared": true }))
            }
            Self::SetArpeggiator {
                mode,
                rate_div,
                octaves,
                gate,
                latch,
            } => {
                let parsed = ArpMode::parse(mode).ok_or_else(|| {
                    SynthError::Validation(format!("Unknown arpeggiator mode '{mode}'"))
                })?;
                let arp = &mut engine.active_patch_mut().arp;
                arp.mode = parsed;
                arp.rate_div = (*rate_div).clamp(1, 8);
                arp.octaves = (*octaves).clamp(1, 4);
                arp.gate = gate.clamp(0.1, 1.0);
                arp.latch = *latch;
                Ok(json!({ "arp": *arp }))
            }
            Self::SetTuning { preset, scala_scl } => {
                let tuning = if let Some(scl) = scala_scl {
                    let _ = parse_scala_scl(scl)?;
                    TuningPreset::CustomScala(scl.clone())
                } else {
                    TuningPreset::parse_builtin(preset).ok_or_else(|| {
                        SynthError::Validation(format!("Unknown tuning preset '{preset}'"))
                    })?
                };
                engine.active_patch_mut().tuning = tuning.clone();
                Ok(json!({ "tuning": tuning.id(), "label": tuning.label() }))
            }
            Self::LoadWavetable { spec } => {
                let wt = Wavetable::load(spec)?;
                let p = engine.active_patch_mut();
                p.wavetable = spec.clone();
                p.vco1_wave = WaveformKind::Wavetable;
                Ok(json!({
                    "wavetable": wt.name,
                    "frames": wt.frames.len(),
                    "vco1_wave": p.vco1_wave.as_str(),
                }))
            }
            Self::ImportMidiFile { path } => {
                let pat = import_midi_smf_file(Path::new(path))?;
                let p = engine.active_patch_mut();
                p.steps = pat.steps;
                if let Some(b) = pat.bpm {
                    p.bpm = b;
                }
                Ok(json!({
                    "imported_mid": path,
                    "note_count": pat.note_count,
                    "bpm": p.bpm,
                }))
            }
            Self::ConfigureClapHost {
                plugin_path,
                plugin_index,
                cv_param_id,
                mix,
                cv_depth,
            } => {
                engine.clap_host = ClapHostConfig {
                    plugin_path: plugin_path.clone(),
                    plugin_index: *plugin_index,
                    cv_param_id: *cv_param_id,
                    mix: mix.clamp(0.0, 1.0),
                    cv_depth: cv_depth.clamp(-1.0, 1.0),
                };
                // Ensure a ClapHost module exists in the rack when a plugin path is configured.
                if plugin_path.is_some()
                    && !engine
                        .rack_modules
                        .iter()
                        .any(|m| m.kind == RackModuleKind::ClapHost)
                {
                    engine.rack_modules.push(RackModuleInstance {
                        id: "CLAP-HOST-1".into(),
                        kind: RackModuleKind::ClapHost,
                        row: 1,
                        bypassed: false,
                        param_a: engine.clap_host.mix,
                        param_b: engine.clap_host.cv_depth * 0.5 + 0.5,
                    });
                } else if let Some(m) = engine
                    .rack_modules
                    .iter_mut()
                    .find(|m| m.kind == RackModuleKind::ClapHost)
                {
                    m.param_a = engine.clap_host.mix;
                    m.param_b = engine.clap_host.cv_depth * 0.5 + 0.5;
                }
                Ok(json!({ "clap_host": engine.clap_host }))
            }
            Self::ResetFactoryPatches => {
                *engine = SynthEngine::factory_seed();
                Ok(json!({
                    "reset": true,
                    "patch_count": engine.patches.len(),
                    "active_patch_id": engine.active_patch_id,
                }))
            }
        }
    }
}

/// Slow atmospheric patches built on the pad oscillator and the master effects.
fn ambient_patches(mk_steps: &dyn Fn(&[u8]) -> Vec<SequencerStep>) -> Vec<SynthPatch> {
    let base = SynthPatch {
        id: String::new(),
        name: String::new(),
        category: "Ambient Pad".into(),
        bpm: 60.0,
        camelot_key: "8A Am".into(),
        vco1_wave: WaveformKind::AtmoPad,
        vco2_wave: WaveformKind::AtmoPad,
        vco2_detune_cents: 7.0,
        vco_mix: 0.5,
        fm_index: 0.0,
        pulse_width: 0.5,
        filter_mode: FilterMode::LowPass24dB,
        cutoff_hz: 2600.0,
        resonance_q: 1.2,
        env_mod_depth: 0.15,
        attack_ms: 1800.0,
        decay_ms: 1200.0,
        sustain_level: 0.85,
        release_ms: 3200.0,
        lfo_rate_hz: 0.12,
        lfo_depth: 0.15,
        drive_saturation: 0.1,
        master_gain: 0.9,
        m1_program: None,
        m1_layer_mix: 0.5,
        extreme_dsp: ExtremeDspMode::Off,
        extreme_morph: 0.5,
        sampler: SamplerParams::default(),
        fx: FxParams {
            chorus_mix: 0.6,
            reverb_mix: 0.5,
            reverb_decay_s: 6.0,
            reverb_damping: 0.45,
            ..FxParams::default()
        },
        mod_matrix: Vec::new(),
        arp: ArpParams::default(),
        tuning: TuningPreset::default(),
        wavetable: "analog_morph".into(),
        steps: mk_steps(&[
            57, 57, 57, 57, 57, 57, 57, 57, 64, 64, 64, 64, 64, 64, 64, 64,
        ])
        .into_iter()
        .enumerate()
        .map(|(i, mut s)| {
            // Two long legato chords per bar: only steps 0 and 8 retrigger.
            s.gate_pct = 1.0;
            s.slide = i % 8 != 0;
            s.accent = false;
            s
        })
        .collect(),
    };
    let mut glass = base.clone();
    glass.id = "ambient_glass_choir".into();
    glass.name = "Glass Choir Atmosphere".into();
    let mut tide = base.clone();
    tide.id = "ambient_slow_tide".into();
    tide.name = "Slow Tide".into();
    tide.vco1_wave = WaveformKind::SuperSaw;
    tide.vco_mix = 0.35;
    tide.cutoff_hz = 1400.0;
    tide.lfo_rate_hz = 0.07;
    tide.lfo_depth = 0.35;
    tide.attack_ms = 2500.0;
    tide.release_ms = 4200.0;
    tide.fx.chorus_mix = 0.4;
    tide.fx.reverb_mix = 0.6;
    tide.fx.reverb_decay_s = 9.0;
    let mut sampled = base;
    sampled.id = "ambient_sample_pad".into();
    sampled.name = "Sample Bank Pad (load a bank on workbench 4)".into();
    sampled.category = "Sampler / Ambient".into();
    sampled.sampler.mix = 1.0;
    sampled.cutoff_hz = 12000.0;
    sampled.attack_ms = 600.0;
    sampled.release_ms = 2500.0;
    sampled.fx.chorus_mix = 0.4;
    sampled.fx.reverb_mix = 0.35;
    vec![glass, tide, sampled]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_produces_non_silent_bounded_pcm_for_all_factory_patches() {
        let eng = SynthEngine::factory_seed();
        assert_eq!(eng.patches.len(), 9);
        for p in &eng.patches {
            let buf = synthesize_audio_buffer(p, 48000, 120 + 2 * p.attack_ms as u32);
            assert!(!buf.is_empty());
            assert!(buf.iter().all(|&s| (-1.0..=1.0).contains(&s)));
            let max_amp = buf.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
            assert!(max_amp > 0.05);
        }
    }

    #[test]
    fn spectrum_analyzer_computes_32_bands_and_dbfs_metrics() {
        let eng = SynthEngine::factory_seed();
        let rep = analyze_patch_spectrum(eng.active_patch(), 48000, 200);
        assert_eq!(rep.spectrum_bands_db.len(), 32);
        assert_eq!(rep.scope_samples.len(), 128);
        assert!(rep.peak_dbfs <= 0.1 && rep.peak_dbfs > -40.0);
        assert!(rep.estimated_pitch_hz > 20.0);
    }

    #[test]
    fn clap_descriptor_and_djcraft_stem_exports_match_soundcraft_and_djcraft() {
        let eng = SynthEngine::factory_seed();
        let clap = export_clap_descriptor(eng.active_patch());
        assert_eq!(clap["clap_version"], "1.2.2");
        assert_eq!(clap["plugin_id"], "dev.sandraschi.synthcraft");
        assert_eq!(clap["parameters"].as_array().map(Vec::len), Some(10));

        let stem = export_djcraft_stem_manifest(eng.active_patch(), 4, 48000);
        assert_eq!(stem["camelot_key"], "8A Am");
        assert_eq!(stem["bars"], 4);
        assert_eq!(stem["beats"], 16);
    }

    #[test]
    fn wav_writer_creates_valid_riff_wave_header_on_disk() {
        let mut path = std::env::temp_dir();
        path.push(format!("synthcraft_test_{}.wav", std::process::id()));
        let _ = std::fs::remove_file(&path);

        let eng = SynthEngine::factory_seed();
        let count = write_wav_file(eng.active_patch(), &path, 48000, 50);
        assert!(count.is_ok());

        let bytes = std::fs::read(&path).unwrap_or_default();
        let _ = std::fs::remove_file(&path);
        assert!(bytes.len() > 44);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
    }

    #[test]
    fn sqlite_roundtrip_persists_patches_and_active_patch_id() {
        let mut path = std::env::temp_dir();
        path.push(format!("synthcraft_test_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);

        let mut eng = SynthEngine::factory_seed();
        let _ = eng.select_patch("goliath_reese_sub");
        assert!(eng.save_db(&path).is_ok());

        let loaded = load_from_db(&path).ok().flatten();
        let _ = std::fs::remove_file(&path);
        assert!(loaded.is_some());
        if let Some(l) = loaded {
            assert_eq!(l.active_patch_id, "goliath_reese_sub");
            assert_eq!(l.patches.len(), 9);
        }
    }

    #[test]
    fn rack_cable_commands_and_dsp_routing_affect_audio() {
        let mut eng = SynthEngine::factory_seed();
        let default_r = RackRouting::from_cables(&eng.rack_cables);
        assert_eq!(default_r, RackRouting::default());

        // Disconnect VCF -> VCA signal cable: engine should go silent.
        let res = Command::DisconnectRackCable {
            cable_id: "cab_vcf_vca".into(),
        }
        .dispatch(&mut eng);
        assert!(res.is_ok());

        let mut rt = SynthRt::new(48000.0, eng.active_patch().clone());
        rt.set_routing(RackRouting::from_cables(&eng.rack_cables));
        rt.note_on(60, 0.9);
        let mut silent = vec![0.0_f32; 512];
        rt.process(&mut silent);
        let silent_peak = silent.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(
            silent_peak < 1e-4,
            "expected silence without VCA signal cable, got {silent_peak}"
        );

        // Direct bypass patch: VCO-1 SAW OUT -> VCA-MASTER SIGNAL IN restores signal.
        let patched = Command::ConnectRackCable {
            from_module: "VCO-1".into(),
            from_jack: "SAW OUT".into(),
            to_module: "VCA-MASTER".into(),
            to_jack: "SIGNAL IN".into(),
            color_name: Some("amber".into()),
            gain: Some(1.0),
        }
        .dispatch(&mut eng);
        assert!(patched.is_ok());
        rt.set_routing(RackRouting::from_cables(&eng.rack_cables));
        let mut loud = vec![0.0_f32; 512];
        rt.process(&mut loud);
        let loud_peak = loud.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(
            loud_peak > 0.05,
            "expected audible signal after direct VCA patch, got {loud_peak}"
        );

        // Reset restores default cables.
        assert!(Command::ResetRackCables.dispatch(&mut eng).is_ok());
        assert_eq!(
            RackRouting::from_cables(&eng.rack_cables),
            RackRouting::default()
        );
    }
}
