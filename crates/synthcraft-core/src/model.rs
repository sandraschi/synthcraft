//! Domain models for Modular Audio Synthesis: Dual VCOs, 4-Pole Resonant Ladder VCF,
//! ADSR Envelope, Tempo-Synced LFO, 16-Step Acid Sequencer, 32-Band DFT Spectrum
//! Analyzer, RIFF PCM `.wav` Renderer, CLAP 1.2.2 Plugin Bridge (for SoundCraft),
//! and BPM/Camelot Stem & Riser Exporter (for DJCraft).

use crate::error::SynthError;
use crate::rompler::{ExtremeDspMode, PcmProgram};
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
    AtmoPad,
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
            Self::AtmoPad => "atmo_pad",
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
            "atmo_pad" | "pad" | "atmo" => Some(Self::AtmoPad),
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
            // `phase` is the phase of the sub itself: one full sine period, no DC offset.
            Self::SubOctave => (2.0 * PI * phase).sin(),
            Self::AtmoPad => (2.0 * PI * phase).sin(),
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
    #[serde(default)]
    pub m1_program: Option<PcmProgram>,
    #[serde(default = "default_half")]
    pub m1_layer_mix: f32,
    #[serde(default = "default_extreme_off")]
    pub extreme_dsp: ExtremeDspMode,
    #[serde(default = "default_half")]
    pub extreme_morph: f32,
    #[serde(default)]
    pub sampler: SamplerParams,
    #[serde(default)]
    pub fx: FxParams,
    pub steps: Vec<SequencerStep>,
}

/// Sample bank layer: a bank spec (see `sampler::load_bank`) and how much of it to mix in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SamplerParams {
    pub bank: Option<String>,
    pub mix: f32,
}

/// Master effects applied after the voices. All zero means bypassed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FxParams {
    pub chorus_mix: f32,
    pub reverb_mix: f32,
    pub reverb_decay_s: f32,
    pub reverb_damping: f32,
}

impl Default for FxParams {
    fn default() -> Self {
        Self {
            chorus_mix: 0.0,
            reverb_mix: 0.0,
            reverb_decay_s: 3.0,
            reverb_damping: 0.5,
        }
    }
}

fn default_half() -> f32 {
    0.55
}

fn default_extreme_off() -> ExtremeDspMode {
    ExtremeDspMode::Off
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

/// Longest offline render in milliseconds (5 minutes).
pub const MAX_RENDER_MS: u32 = 300_000;

/// Render mono audio (`[-1.0, 1.0]`) for `duration_ms` by running the sequencer through the
/// same [`crate::rt::SynthRt`] engine the live audio callback uses.
pub fn synthesize_audio_buffer(patch: &SynthPatch, sample_rate: u32, duration_ms: u32) -> Vec<f32> {
    let sr = sample_rate.clamp(8000, 96000);
    let total = ((f64::from(sr) * f64::from(duration_ms.clamp(20, MAX_RENDER_MS))) / 1000.0).round()
        as usize;
    let mut out = vec![0.0_f32; total];
    let mut rt = crate::rt::SynthRt::new(sr as f32, patch.clone());
    if let Some(spec) = patch.sampler.bank.as_deref() {
        // A bank that fails to load renders as the plain synth; the UI reports the error.
        rt.set_bank(crate::sampler::load_cached(spec).ok());
    }
    rt.set_playing(true);
    for chunk in out.chunks_mut(512) {
        rt.process(chunk);
    }
    for s in &mut out {
        *s = s.clamp(-1.0, 1.0);
    }
    out
}

/// Analyse a rendered patch: RMS/peak over the whole render, then a Hann-windowed FFT of the
/// loudest block for 32 log-spaced bands (40 Hz to 12 kHz), a scope slice that starts on a
/// rising zero crossing, and an autocorrelation pitch estimate (0.0 when not periodic).
pub fn analyze_patch_spectrum(
    patch: &SynthPatch,
    sample_rate: u32,
    duration_ms: u32,
) -> SpectrumAnalysisReport {
    let samples = synthesize_audio_buffer(patch, sample_rate, duration_ms);
    let n = samples.len().max(1);
    let sr = sample_rate.clamp(8000, 96000) as f32;

    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    let peak = samples.iter().fold(1e-6_f32, |m, s| m.max(s.abs()));
    let rms = (sum_sq / n as f32).sqrt().max(1e-6);
    let rms_dbfs = round2(20.0 * rms.log10());
    let peak_dbfs = round2(20.0 * peak.log10());
    let crest_factor_db = round2(peak_dbfs - rms_dbfs);

    // Loudest power-of-two block.
    let mut win = 4096usize;
    while win > samples.len() && win > 64 {
        win /= 2;
    }
    let win = win.min(samples.len().max(1));
    let hop = (win / 4).max(1);
    let mut best_start = 0usize;
    let mut best_energy = -1.0_f32;
    let mut start = 0usize;
    while start + win <= samples.len() {
        let e: f32 = samples[start..start + win].iter().map(|s| s * s).sum();
        if e > best_energy {
            best_energy = e;
            best_start = start;
        }
        start += hop;
    }
    let block = samples
        .get(best_start..best_start + win)
        .unwrap_or(&samples[..]);

    let estimated_pitch_hz = round2(crate::dsp::autocorrelation_pitch(block, sr));

    let scope_from = block
        .windows(2)
        .position(|w| w[0] < 0.0 && w[1] >= 0.0)
        .unwrap_or(0);
    let scope_samples: Vec<f32> = block
        .iter()
        .skip(scope_from)
        .take(128)
        .map(|&x| (x * 1000.0).round() / 1000.0)
        .collect();

    let mag_db = if block.len().is_power_of_two() {
        crate::dsp::magnitude_db(block)
    } else {
        Vec::new()
    };
    let spectrum_bands_db = crate::dsp::log_bands(&mag_db, sr / block.len().max(1) as f32);

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

/// Duration in ms of a stem of `bars` bars (4 beats each) at the patch tempo.
pub fn stem_duration_ms(patch: &SynthPatch, bars: usize) -> u32 {
    let beats = bars.clamp(1, 32) * 4;
    let sec = (beats as f32) * (60.0 / patch.bpm.clamp(40.0, 240.0));
    ((sec * 1000.0).round() as u32).min(MAX_RENDER_MS)
}

/// Export BPM-locked, Camelot-keyed stem metadata and bar-grid cue points for **DJCraft** &
/// **SoundCraft**. Cue points are derived from the tempo: the loop start, every fourth bar
/// (phrase boundaries) and the loop end.
pub fn export_djcraft_stem_manifest(patch: &SynthPatch, bars: usize, sample_rate: u32) -> Value {
    let clamped_bars = bars.clamp(1, 32);
    let beats = clamped_bars * 4;
    let duration_ms = stem_duration_ms(patch, clamped_bars);
    let bar_ms = f64::from(duration_ms) / clamped_bars as f64;
    let mut cues = vec![json!({ "label": "LOOP_IN", "ms": 0 })];
    let mut bar = 4;
    while bar < clamped_bars {
        cues.push(json!({ "label": format!("PHRASE_BAR_{}", bar + 1), "ms": (bar_ms * bar as f64).round() as u32 }));
        bar += 4;
    }
    cues.push(json!({ "label": "LOOP_OUT", "ms": duration_ms }));

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
        "sample_rate_hz": sample_rate,
        "channels": 1,
        "cue_points_ms": cues,
        "clap_plugin_id": "dev.sandraschi.synthcraft"
    })
}

fn round2(v: f32) -> f32 {
    (v * 100.0).round() / 100.0
}
