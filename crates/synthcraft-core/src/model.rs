//! Domain models for Modular Audio Synthesis: Dual VCOs, 4-Pole Resonant Ladder VCF,
//! ADSR Envelope, Tempo-Synced LFO, 16-Step TB-303 Acid Sequencer, 32-Band DFT Spectrum
//! Analyzer, RIFF PCM `.wav` Renderer, CLAP 1.2.2 Plugin Bridge (for SoundCraft),
//! and BPM/Camelot Stem & Riser Exporter (for DJCraft).

use crate::error::SynthError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::f32::consts::PI;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaveformKind {
    Sine,
    Saw,
    Square,
    Triangle,
    PulsePwm,
    SuperSaw,
    WhiteNoise,
    SubOctave,
}

impl WaveformKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sine => "sine",
            Self::Saw => "saw",
            Self::Square => "square",
            Self::Triangle => "triangle",
            Self::PulsePwm => "pulse_pwm",
            Self::SuperSaw => "supersaw",
            Self::WhiteNoise => "white_noise",
            Self::SubOctave => "sub_octave",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "sine" | "sin" => Some(Self::Sine),
            "saw" | "sawtooth" => Some(Self::Saw),
            "square" | "sq" => Some(Self::Square),
            "triangle" | "tri" => Some(Self::Triangle),
            "pulse_pwm" | "pwm" | "pulse" => Some(Self::PulsePwm),
            "supersaw" | "super_saw" => Some(Self::SuperSaw),
            "white_noise" | "noise" => Some(Self::WhiteNoise),
            "sub_octave" | "sub" => Some(Self::SubOctave),
            _ => None,
        }
    }

    /// Sample oscillator waveform at normalized phase `p in [0.0, 1.0)`.
    pub fn sample(self, p: f32, pulse_width: f32, noise_seed: u32) -> f32 {
        let phase = p.rem_euclid(1.0);
        match self {
            Self::Sine => (2.0 * PI * phase).sin(),
            Self::Saw => 2.0 * phase - 1.0,
            Self::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::Triangle => 1.0 - 4.0 * (phase - 0.5).abs(),
            Self::PulsePwm => {
                let pw = pulse_width.clamp(0.05, 0.95);
                if phase < pw {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::SuperSaw => {
                let s0 = 2.0 * phase - 1.0;
                let s1 = 2.0 * (phase * 1.007).rem_euclid(1.0) - 1.0;
                let s2 = 2.0 * (phase * 0.993).rem_euclid(1.0) - 1.0;
                let s3 = 2.0 * (phase * 1.014).rem_euclid(1.0) - 1.0;
                (s0 + 0.75 * (s1 + s2) + 0.5 * s3) / 3.0
            }
            Self::WhiteNoise => {
                // Deterministic xorshift hash for reproducible audio tests
                let mut x = noise_seed.wrapping_add((phase * 65536.0) as u32).max(1);
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                ((x & 0xFFFF) as f32 / 32768.0) - 1.0
            }
            Self::SubOctave => (PI * phase).sin(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterMode {
    LowPass24dB,
    BandPass12dB,
    HighPass12dB,
    AcidResonant,
}

impl FilterMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LowPass24dB => "lowpass_24db",
            Self::BandPass12dB => "bandpass_12db",
            Self::HighPass12dB => "highpass_12db",
            Self::AcidResonant => "acid_resonant",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "lowpass_24db" | "lowpass" | "lp" => Some(Self::LowPass24dB),
            "bandpass_12db" | "bandpass" | "bp" => Some(Self::BandPass12dB),
            "highpass_12db" | "highpass" | "hp" => Some(Self::HighPass12dB),
            "acid_resonant" | "acid" | "303" => Some(Self::AcidResonant),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerStep {
    pub step_index: usize,
    pub midi_note: u8,
    pub velocity: f32,
    pub gate_pct: f32,
    pub accent: bool,
    pub slide: bool,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthPatch {
    pub id: String,
    pub name: String,
    pub category: String,
    pub bpm: f32,
    pub camelot_key: String, // e.g. "8A Am", "4A Fm", "11B A"
    pub vco1_wave: WaveformKind,
    pub vco2_wave: WaveformKind,
    pub vco2_detune_cents: f32,
    pub vco_mix: f32, // 0.0 = VCO1, 1.0 = VCO2
    pub fm_index: f32,
    pub pulse_width: f32,
    pub filter_mode: FilterMode,
    pub cutoff_hz: f32,
    pub resonance_q: f32,
    pub env_mod_depth: f32,
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub sustain_level: f32,
    pub release_ms: f32,
    pub lfo_rate_hz: f32,
    pub lfo_depth: f32,
    pub drive_saturation: f32,
    pub master_gain: f32,
    pub steps: Vec<SequencerStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumAnalysisReport {
    pub patch_id: String,
    pub sample_rate_hz: u32,
    pub sample_count: usize,
    pub rms_dbfs: f32,
    pub peak_dbfs: f32,
    pub crest_factor_db: f32,
    pub estimated_pitch_hz: f32,
    pub scope_samples: Vec<f32>, // 128 waveform samples for oscilloscope display
    pub spectrum_bands_db: Vec<(f32, f32)>, // 32 (center_freq_hz, magnitude_db) pairs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClapParameterSpec {
    pub clap_id: String,
    pub name: String,
    pub module: String,
    pub min_val: f32,
    pub max_val: f32,
    pub default_val: f32,
    pub current_val: f32,
    pub poly_modulatable: bool,
}

pub fn midi_to_freq_hz(midi_note: u8) -> f32 {
    440.0 * 2.0_f32.powf((f32::from(midi_note) - 69.0) / 12.0)
}

/// Synthesize mono PCM floating-point audio (`[-1.0, 1.0]`) for `duration_ms` at `sample_rate`.
pub fn synthesize_audio_buffer(patch: &SynthPatch, sample_rate: u32, duration_ms: u32) -> Vec<f32> {
    let sr = sample_rate.clamp(8000, 96000) as f32;
    let total_samples = ((sr * (duration_ms.clamp(20, 15000) as f32)) / 1000.0).round() as usize;
    let mut out = Vec::with_capacity(total_samples);

    let step_dur_sec = (60.0 / patch.bpm.clamp(40.0, 240.0)) * 0.25; // 1/16th note
    let step_samples = (step_dur_sec * sr).max(1.0) as usize;

    let mut phase1 = 0.0_f32;
    let mut phase2 = 0.0_f32;
    let mut lfo_phase = 0.0_f32;
    let mut lp_state1 = 0.0_f32;
    let mut lp_state2 = 0.0_f32;
    let mut bp_state = 0.0_f32;
    let mut current_freq = 110.0_f32;

    let detune_ratio = 2.0_f32.powf(patch.vco2_detune_cents / 1200.0);
    let atk_s = (patch.attack_ms.max(1.0) / 1000.0) * sr;
    let dec_s = (patch.decay_ms.max(5.0) / 1000.0) * sr;
    let rel_s = (patch.release_ms.max(5.0) / 1000.0) * sr;

    for idx in 0..total_samples {
        let step_idx = if patch.steps.is_empty() {
            0
        } else {
            (idx / step_samples) % patch.steps.len()
        };
        let pos_in_step = (idx % step_samples) as f32;

        let (target_note, vel, gate_pct, accent, slide, active) = patch
            .steps
            .get(step_idx)
            .map(|s| {
                (
                    s.midi_note,
                    s.velocity,
                    s.gate_pct,
                    s.accent,
                    s.slide,
                    s.active,
                )
            })
            .unwrap_or((45, 0.85, 0.75, false, false, true));

        let target_freq = midi_to_freq_hz(target_note);
        if idx == 0 || !slide {
            current_freq = target_freq;
        } else {
            current_freq += (target_freq - current_freq) * 0.008;
        }

        // ADSR envelope within step
        let gate_samples = (step_samples as f32 * gate_pct.clamp(0.1, 1.0)).max(1.0);
        let env = if !active {
            0.0
        } else if pos_in_step < atk_s {
            pos_in_step / atk_s
        } else if pos_in_step < atk_s + dec_s {
            let t = (pos_in_step - atk_s) / dec_s;
            1.0 - t * (1.0 - patch.sustain_level.clamp(0.0, 1.0))
        } else if pos_in_step < gate_samples {
            patch.sustain_level.clamp(0.0, 1.0)
        } else {
            let rel_t = ((pos_in_step - gate_samples) / rel_s).clamp(0.0, 1.0);
            patch.sustain_level.clamp(0.0, 1.0) * (1.0 - rel_t)
        };

        let accent_boost = if accent { 1.35 } else { 1.0 };

        // LFO modulation
        let lfo_val = (2.0 * PI * lfo_phase).sin() * patch.lfo_depth.clamp(0.0, 1.0);
        lfo_phase = (lfo_phase + patch.lfo_rate_hz.clamp(0.05, 40.0) / sr).rem_euclid(1.0);

        // VCO2 + FM -> VCO1
        let s2 = patch
            .vco2_wave
            .sample(phase2, patch.pulse_width, idx as u32);
        let fm_offset = s2 * patch.fm_index.clamp(0.0, 4.0) * 0.08;
        let s1 = patch
            .vco1_wave
            .sample(phase1 + fm_offset, patch.pulse_width, idx as u32);

        phase1 = (phase1 + current_freq / sr).rem_euclid(1.0);
        phase2 = (phase2 + (current_freq * detune_ratio) / sr).rem_euclid(1.0);

        let mix = patch.vco_mix.clamp(0.0, 1.0);
        let raw_osc = s1 * (1.0 - mix) + s2 * mix;

        // 4-Pole Resonant State-Variable / Ladder VCF approximation
        let mod_cutoff =
            (patch.cutoff_hz + env * patch.env_mod_depth * 3200.0 * accent_boost + lfo_val * 900.0)
                .clamp(35.0, sr * 0.44);
        let f = (2.0 * (PI * mod_cutoff / sr).sin()).clamp(0.005, 0.92);
        let q_fb = (patch.resonance_q.clamp(0.5, 12.0) / 12.0) * 0.88;

        let hp = raw_osc - lp_state1 - (1.0 - q_fb) * bp_state;
        bp_state += f * hp;
        lp_state1 += f * bp_state;
        lp_state2 += f * (lp_state1 - lp_state2);

        let filtered = match patch.filter_mode {
            FilterMode::LowPass24dB => lp_state2,
            FilterMode::BandPass12dB => bp_state,
            FilterMode::HighPass12dB => hp,
            FilterMode::AcidResonant => (lp_state2 + 0.45 * bp_state * accent_boost).tanh(),
        };

        let drive = (1.0 + patch.drive_saturation.clamp(0.0, 3.0) * 2.5) * accent_boost;
        let sample = (filtered * drive).tanh()
            * env
            * vel.clamp(0.1, 1.0)
            * patch.master_gain.clamp(0.05, 1.0);
        out.push(sample.clamp(-1.0, 1.0));
    }

    out
}

/// Analyze synthesized audio: RMS/Peak dBFS, crest factor, pitch estimate, 128-sample scope, and 32-band DFT spectrum.
pub fn analyze_patch_spectrum(
    patch: &SynthPatch,
    sample_rate: u32,
    duration_ms: u32,
) -> SpectrumAnalysisReport {
    let samples = synthesize_audio_buffer(patch, sample_rate, duration_ms);
    let n = samples.len().max(1);

    let mut sum_sq = 0.0_f32;
    let mut peak = 1e-6_f32;
    let mut zero_crossings = 0usize;

    for i in 0..samples.len() {
        let s = samples[i];
        sum_sq += s * s;
        let a = s.abs();
        if a > peak {
            peak = a;
        }
        if i > 0 && ((samples[i - 1] < 0.0 && s >= 0.0) || (samples[i - 1] >= 0.0 && s < 0.0)) {
            zero_crossings += 1;
        }
    }

    let rms = (sum_sq / (n as f32)).sqrt().max(1e-6);
    let rms_dbfs = round2(20.0 * rms.log10());
    let peak_dbfs = round2(20.0 * peak.log10());
    let crest_factor_db = round2(peak_dbfs - rms_dbfs);
    let dur_sec = (n as f32) / (sample_rate.max(1) as f32);
    let estimated_pitch_hz = round2((zero_crossings as f32) / (2.0 * dur_sec.max(0.01)));

    let scope_len = 128.min(samples.len());
    let scope_samples: Vec<f32> = samples[..scope_len]
        .iter()
        .map(|&x| (x * 1000.0).round() / 1000.0)
        .collect();

    // 32 log-spaced frequency bands from 40 Hz to 12,000 Hz evaluated over a 512-sample window
    let win_len = 512.min(samples.len()).max(1);
    let sr = sample_rate.max(8000) as f32;
    let mut spectrum_bands_db = Vec::with_capacity(32);

    for b in 0..32 {
        let frac = (b as f32) / 31.0;
        let freq = 40.0 * (12000.0_f32 / 40.0).powf(frac);
        let omega = 2.0 * PI * freq / sr;
        let mut re = 0.0_f32;
        let mut im = 0.0_f32;
        for (k, &s) in samples[..win_len].iter().enumerate() {
            let angle = omega * (k as f32);
            re += s * angle.cos();
            im -= s * angle.sin();
        }
        let mag = ((re * re + im * im).sqrt() / (win_len as f32)).max(1e-5);
        let db = (20.0 * mag.log10()).clamp(-80.0, 6.0);
        spectrum_bands_db.push((round2(freq), round2(db)));
    }

    SpectrumAnalysisReport {
        patch_id: patch.id.clone(),
        sample_rate_hz: sample_rate,
        sample_count: samples.len(),
        rms_dbfs,
        peak_dbfs,
        crest_factor_db,
        estimated_pitch_hz,
        scope_samples,
        spectrum_bands_db,
    }
}

/// Write a standard 16-bit mono RIFF PCM `.wav` file to disk for SoundCraft or DJCraft.
pub fn write_wav_file(
    patch: &SynthPatch,
    path: &Path,
    sample_rate: u32,
    duration_ms: u32,
) -> Result<usize, SynthError> {
    let samples = synthesize_audio_buffer(patch, sample_rate, duration_ms);
    let data_bytes = (samples.len() * 2) as u32;
    let riff_size = 36 + data_bytes;
    let byte_rate = sample_rate * 2;

    let mut bytes = Vec::with_capacity((44 + data_bytes) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes()); // PCM = 1
    bytes.extend_from_slice(&1_u16.to_le_bytes()); // 1 channel
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes()); // block align
    bytes.extend_from_slice(&16_u16.to_le_bytes()); // 16-bit
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());

    for s in &samples {
        let pcm = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SynthError::Io(format!("create_dir_all failed: {e}")))?;
        }
    }
    std::fs::write(path, &bytes).map_err(|e| SynthError::Io(format!("write wav failed: {e}")))?;
    Ok(samples.len())
}

/// Export CLAP (`1.2.2`, `u-he` / Bitwig Clever Audio Plug-in ABI) descriptor & parameter automation table for **SoundCraft**.
pub fn export_clap_descriptor(patch: &SynthPatch) -> Value {
    let params = vec![
        ClapParameterSpec {
            clap_id: "0x1001".into(),
            name: "VCO2 Detune (cents)".into(),
            module: "vco".into(),
            min_val: -100.0,
            max_val: 100.0,
            default_val: 12.0,
            current_val: patch.vco2_detune_cents,
            poly_modulatable: true,
        },
        ClapParameterSpec {
            clap_id: "0x1002".into(),
            name: "VCO 1/2 Mix".into(),
            module: "vco".into(),
            min_val: 0.0,
            max_val: 1.0,
            default_val: 0.5,
            current_val: patch.vco_mix,
            poly_modulatable: true,
        },
        ClapParameterSpec {
            clap_id: "0x1003".into(),
            name: "FM Modulation Index".into(),
            module: "vco".into(),
            min_val: 0.0,
            max_val: 4.0,
            default_val: 0.2,
            current_val: patch.fm_index,
            poly_modulatable: true,
        },
        ClapParameterSpec {
            clap_id: "0x1004".into(),
            name: "VCF Cutoff Frequency (Hz)".into(),
            module: "vcf".into(),
            min_val: 35.0,
            max_val: 16000.0,
            default_val: 1200.0,
            current_val: patch.cutoff_hz,
            poly_modulatable: true,
        },
        ClapParameterSpec {
            clap_id: "0x1005".into(),
            name: "VCF Resonance (Q)".into(),
            module: "vcf".into(),
            min_val: 0.5,
            max_val: 12.0,
            default_val: 3.5,
            current_val: patch.resonance_q,
            poly_modulatable: true,
        },
        ClapParameterSpec {
            clap_id: "0x1006".into(),
            name: "VCF Envelope Mod Depth".into(),
            module: "vcf".into(),
            min_val: 0.0,
            max_val: 1.0,
            default_val: 0.65,
            current_val: patch.env_mod_depth,
            poly_modulatable: true,
        },
        ClapParameterSpec {
            clap_id: "0x1007".into(),
            name: "ADSR Attack (ms)".into(),
            module: "env".into(),
            min_val: 1.0,
            max_val: 2000.0,
            default_val: 8.0,
            current_val: patch.attack_ms,
            poly_modulatable: false,
        },
        ClapParameterSpec {
            clap_id: "0x1008".into(),
            name: "ADSR Decay (ms)".into(),
            module: "env".into(),
            min_val: 5.0,
            max_val: 4000.0,
            default_val: 180.0,
            current_val: patch.decay_ms,
            poly_modulatable: false,
        },
        ClapParameterSpec {
            clap_id: "0x1009".into(),
            name: "ADSR Sustain Level".into(),
            module: "env".into(),
            min_val: 0.0,
            max_val: 1.0,
            default_val: 0.5,
            current_val: patch.sustain_level,
            poly_modulatable: false,
        },
        ClapParameterSpec {
            clap_id: "0x100A".into(),
            name: "LFO Rate (Hz)".into(),
            module: "lfo".into(),
            min_val: 0.1,
            max_val: 30.0,
            default_val: 2.0,
            current_val: patch.lfo_rate_hz,
            poly_modulatable: true,
        },
    ];

    json!({
        "clap_version": "1.2.2",
        "plugin_id": "dev.sandraschi.synthcraft",
        "name": "SynthCraft Modular DSP Engine",
        "vendor": "Sandra Schipal (sandraschi)",
        "features": ["instrument", "synthesizer", "mono", "stereo", "note-effect"],
        "abi_entry_symbol": "clap_entry",
        "target_hosts": ["SoundCraft", "DJCraft", "Bitwig Studio", "REAPER", "Ardour"],
        "active_patch": patch.id,
        "parameters": params,
    })
}

/// Export BPM-locked, Camelot-keyed stem metadata and cue points for **DJCraft** & **SoundCraft**.
pub fn export_djcraft_stem_manifest(patch: &SynthPatch, bars: usize) -> Value {
    let clamped_bars = bars.clamp(1, 32);
    let beats = clamped_bars * 4;
    let duration_sec = (beats as f32) * (60.0 / patch.bpm.clamp(40.0, 240.0));
    let duration_ms = (duration_sec * 1000.0).round() as u32;

    json!({
        "target_host": "DJCraft & SoundCraft",
        "stem_id": format!("stem_{}_{}bars", patch.id, clamped_bars),
        "patch_name": patch.name,
        "category": patch.category,
        "bpm": patch.bpm,
        "camelot_key": patch.camelot_key,
        "bars": clamped_bars,
        "beats": beats,
        "duration_ms": duration_ms,
        "sample_rate_hz": 48000,
        "cue_points_ms": [
            { "label": "LOOP_IN", "ms": 0 },
            { "label": "FILTER_PEAK", "ms": duration_ms / 2 },
            { "label": "DROP_OUT", "ms": duration_ms }
        ],
        "clap_plugin_id": "dev.sandraschi.synthcraft"
    })
}

fn round2(v: f32) -> f32 {
    (v * 100.0).round() / 100.0
}
