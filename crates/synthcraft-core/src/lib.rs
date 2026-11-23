//! SynthCraft L1 Core Engine & Universal Command Bus.
//!
//! Pure-Rust Modular Audio Synthesizer, 16-Step Acid Sequencer, 32-Band DFT
//! Spectrum Analyzer, RIFF `.wav` Renderer, CLAP 1.2.2 Plugin Bridge for SoundCraft,
//! and BPM/Camelot Stem & Riser Forge for DJCraft.
#![forbid(unsafe_code)]

pub mod db;
pub mod error;
pub mod model;

pub use db::{default_db_path, load_from_db, save_to_db};
pub use error::SynthError;
pub use model::{
    analyze_patch_spectrum, export_clap_descriptor, export_djcraft_stem_manifest, midi_to_freq_hz,
    synthesize_audio_buffer, write_wav_file, ClapParameterSpec, FilterMode, SequencerStep,
    SpectrumAnalysisReport, SynthPatch, WaveformKind,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthEngine {
    pub active_patch_id: String,
    pub sample_rate_hz: u32,
    pub patches: Vec<SynthPatch>,
}

impl Default for SynthEngine {
    fn default() -> Self {
        Self::factory_seed()
    }
}

impl SynthEngine {
    pub fn open_or_seed(path: &Path) -> Result<Self, SynthError> {
        if let Some(loaded) = load_from_db(path)? {
            return Ok(loaded);
        }
        let seeded = Self::factory_seed();
        save_to_db(&seeded, path)?;
        Ok(seeded)
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

        let patches = vec![
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
                steps: mk_steps(&[
                    57, 57, 61, 61, 64, 64, 69, 69, 68, 68, 64, 64, 61, 61, 57, 57,
                ]),
            },
        ];

        Self {
            active_patch_id: "vienna_acid_303".into(),
            sample_rate_hz: 48000,
            patches,
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
    ExportDjcraftStem {
        bars: usize,
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
            Self::ExportDjcraftStem { bars } => {
                Ok(export_djcraft_stem_manifest(engine.active_patch(), *bars))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_produces_non_silent_bounded_pcm_for_all_factory_patches() {
        let eng = SynthEngine::factory_seed();
        assert_eq!(eng.patches.len(), 6);
        for p in &eng.patches {
            let buf = synthesize_audio_buffer(p, 48000, 120);
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

        let stem = export_djcraft_stem_manifest(eng.active_patch(), 4);
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
            assert_eq!(l.patches.len(), 6);
        }
    }
}
