//! 1990-style PCM Workstation Programs, Extreme Modern DSP Generators,
//! Hardware MIDI Controller Mapping, MIDI Pattern Depot, and Demo Showcase Presets.

use crate::error::SynthError;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::path::Path;

/// Iconic 1990 PCM workstation Factory PCM Programs (AI2 Synthesis: 16-bit 32 kHz PCM Attack Transient + Looped Wave).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PcmProgram {
    /// Prog 00: Universe (Choir + Glass Bell Pad + Ensemble Chorus)
    Universe00,
    /// Prog 01: Piano 16' (1990 House / Rave Bright Hammer Piano)
    Piano16_01,
    /// Prog 11: M1 Fretless Bass (Mwah Pitch-Envelope Fretless)
    Fretless11,
    /// Prog 12: M1 Slap Bass (Thumb Pop Transient + Funk Decay)
    SlapBass12,
    /// Prog 17: Organ 2 (90s house percussive 3rd-harmonic drawbar organ)
    Organ2_17,
    /// Prog 24: M1 Pan Flute (Chiff Breath Attack + Bamboo Overtones)
    PanFlute24,
    /// Prog 36: M1 Ooh/Aah Choir (3-Formant Vowel Filter Bank)
    OohAahChoir36,
    /// Prog 48: M1 Orchestra Hit (Stacked Minor-Triad Staccato Hit)
    OrchestraHit48,
}

impl PcmProgram {
    pub fn all() -> &'static [Self] {
        &[
            Self::Universe00,
            Self::Piano16_01,
            Self::Organ2_17,
            Self::Fretless11,
            Self::SlapBass12,
            Self::PanFlute24,
            Self::OohAahChoir36,
            Self::OrchestraHit48,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Universe00 => "m1_00_universe",
            Self::Piano16_01 => "m1_01_piano16",
            Self::Fretless11 => "m1_11_fretless",
            Self::SlapBass12 => "m1_12_slapbass",
            Self::Organ2_17 => "m1_17_organ2",
            Self::PanFlute24 => "m1_24_panflute",
            Self::OohAahChoir36 => "m1_36_choir",
            Self::OrchestraHit48 => "m1_48_orchhit",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Universe00 => "Prog 00: Universe (Choir + Glass Bell)",
            Self::Piano16_01 => "Prog 01: Piano 16' (1990 House/Rave Piano)",
            Self::Fretless11 => "Prog 11: Fretless Bass (Mwah Scoop)",
            Self::SlapBass12 => "Prog 12: Slap Bass (Thumb Pop PCM)",
            Self::Organ2_17 => "Prog 17: Organ 2 (90s house organ)",
            Self::PanFlute24 => "Prog 24: Pan Flute (Breathy Chiff Attack)",
            Self::OohAahChoir36 => "Prog 36: Ooh/Aah Choir (Formant Vocal)",
            Self::OrchestraHit48 => "Prog 48: Orchestra Hit (Stab)",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "m1_00_universe" | "universe" | "00" => Some(Self::Universe00),
            "m1_01_piano16" | "piano16" | "piano" | "01" => Some(Self::Piano16_01),
            "m1_11_fretless" | "fretless" | "11" => Some(Self::Fretless11),
            "m1_12_slapbass" | "slapbass" | "slap" | "12" => Some(Self::SlapBass12),
            "m1_17_organ2" | "organ2" | "organ" | "17" => Some(Self::Organ2_17),
            "m1_24_panflute" | "panflute" | "flute" | "24" => Some(Self::PanFlute24),
            "m1_36_choir" | "choir" | "36" => Some(Self::OohAahChoir36),
            "m1_48_orchhit" | "orchhit" | "hit" | "48" => Some(Self::OrchestraHit48),
            _ => None,
        }
    }

    /// One 16-bit, 32 kHz-quantised sample of this additive voice. `phase_cycles` is the
    /// integrated oscillator phase (continuous through glide and bend); `time_sec` is time since note-on.
    pub fn sample_pcm(self, phase_cycles: f32, time_sec: f32) -> f32 {
        // Quantize time to 32,000 Hz DAC clock like the original 1988 PCM workstation DAC chip
        let t = (time_sec * 32000.0).floor() / 32000.0;
        let w = 2.0 * PI * phase_cycles;

        let raw = match self {
            Self::Piano16_01 => {
                // Bright metallic hammer transient (first 25ms) + inharmonic stretched piano partials
                let hammer = (-t * 95.0).exp() * (w * 7.0 + (w * 3.0).sin() * 2.2).sin() * 0.45;
                let body = (-t * 4.5).exp()
                    * ((w).sin() * 0.55
                        + (w * 2.001).sin() * 0.32
                        + (w * 3.004).sin() * 0.18
                        + (w * 4.01).sin() * 0.10);
                hammer + body
            }
            Self::Organ2_17 => {
                // 90s House Organ 2: Fundamental (8') + Octave (4') + Loud 12th/Nazat (2 2/3' = 3*f0) + 8ms key-click
                let click = (-t * 180.0).exp() * (w * 9.5).sin() * 0.40;
                let env = (-t * 5.2).exp() * 0.65 + 0.35;
                let drawbars = (w).sin() * 0.50 + (w * 2.0).sin() * 0.25 + (w * 2.998).sin() * 0.48;
                (click + drawbars * env).tanh()
            }
            Self::Universe00 => {
                // glass bell transient + slow choir formant pad
                let bell = (-t * 7.0).exp() * (w * 3.5 + (w * 1.414).sin() * 1.8).sin() * 0.35;
                let pad_env = 1.0 - (-t * 3.5).exp();
                let choir =
                    ((w * 0.998).sin() + (w * 1.003).sin() + (w * 2.002).sin() * 0.4) * 0.35;
                bell + choir * pad_env
            }
            Self::Fretless11 => {
                // Pitch scoop on attack + warm fretless mwah harmonics
                let scoop = 1.0 - 0.015 * (-t * 28.0).exp();
                let ws = w * scoop;
                let env = (-t * 3.8).exp();
                ((ws).sin() * 0.6 + (ws * 2.0).sin() * 0.35 + (ws * 3.0).cos() * 0.2) * env
            }
            Self::SlapBass12 => {
                let pop = (-t * 140.0).exp() * (w * 8.0 + (w * 5.0).sin() * 3.0).sin() * 0.55;
                let thumb = (-t * 6.0).exp() * ((w).sin() + (w * 2.0).sin() * 0.5).tanh() * 0.7;
                pop + thumb
            }
            Self::PanFlute24 => {
                // Breathy chiff noise burst + odd-harmonic bamboo pipe
                let noise = ((t * 123456.7).sin() * 43758.547).fract() * 2.0 - 1.0;
                let chiff = (-t * 45.0).exp() * noise * 0.45;
                let vibrato = 1.0 + 0.006 * (2.0 * PI * 5.2 * t).sin();
                let pipe = ((w * vibrato).sin() * 0.75 + (w * 3.0 * vibrato).sin() * 0.22)
                    * (1.0 - (-t * 18.0).exp());
                chiff + pipe
            }
            Self::OohAahChoir36 => {
                // 3-formant vocal approximation (F1 ~ 700 Hz, F2 ~ 1200 Hz, F3 ~ 2600 Hz)
                let f1 = (2.0 * PI * 720.0 * t).sin() * (w).sin();
                let f2 = (2.0 * PI * 1240.0 * t).sin() * (w * 0.5).cos();
                let env = 1.0 - (-t * 6.0).exp();
                (f1 * 0.55 + f2 * 0.35 + (w).sin() * 0.4) * env
            }
            Self::OrchestraHit48 => {
                // Minor triad stack (Root + Minor 3rd 1.1892 + Fifth 1.4983) with sharp staccato decay
                let env = (-t * 8.5).exp();
                let root = (w).sin() + (w * 2.0).sin() * 0.5;
                let m3 = (w * 1.1892).sin() + (w * 2.3784).sin() * 0.5;
                let fifth = (w * 1.4983).sin() + (w * 2.9966).sin() * 0.5;
                ((root + m3 + fifth) * 0.35 * env).tanh()
            }
        };

        // 16-bit PCM quantization step
        ((raw.clamp(-1.0, 1.0) * 32767.0).round()) / 32767.0
    }
}

/// Extreme Modern DSP Sound Generators beyond classic analog VCOs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtremeDspMode {
    /// Disabled (standard VCO1 + VCO2 subtractive/FM chain)
    Off,
    /// Physical Modeling Karplus-Strong Plucked String Waveguide
    KarplusStrongPluck,
    /// Microsound Granular Shimmer Cloud (overlapping Hann-windowed grains)
    GranularCloud,
    /// 3D Wavetable Morpher (Sine -> Formant -> Digital Fold -> HyperSaw)
    WavetableMorph,
    /// West-coast multi-stage Trigonometric Wavefolder + LPG
    WestCoastWavefolder,
    /// Phase Distortion (Non-Linear Phase Angle Warping)
    PhaseDistortionCz,
    /// Modal Physical Modeling Struck Metal / Glass Bar (Euler-Bernoulli Eigenmodes)
    ModalStruckBar,
}

impl ExtremeDspMode {
    pub fn all() -> &'static [Self] {
        &[
            Self::Off,
            Self::KarplusStrongPluck,
            Self::GranularCloud,
            Self::WavetableMorph,
            Self::WestCoastWavefolder,
            Self::PhaseDistortionCz,
            Self::ModalStruckBar,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::KarplusStrongPluck => "karplus_strong_pluck",
            Self::GranularCloud => "granular_cloud",
            Self::WavetableMorph => "wavetable_morph",
            Self::WestCoastWavefolder => "west_coast_wavefolder",
            Self::PhaseDistortionCz => "phase_distortion_cz",
            Self::ModalStruckBar => "modal_struck_bar",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Off => "Off (Standard Analog VCO 1+2)",
            Self::KarplusStrongPluck => "Karplus-Strong Waveguide Plucked String",
            Self::GranularCloud => "Granular Microsound Shimmer Cloud",
            Self::WavetableMorph => "3D Wavetable Morpher (Serum/Vital Style)",
            Self::WestCoastWavefolder => "West-Coast Wavefolder + LPG",
            Self::PhaseDistortionCz => "Non-Linear Phase Distortion",
            Self::ModalStruckBar => "Modal Physical Struck Metal/Glass Bar",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "off" | "none" => Some(Self::Off),
            "karplus_strong_pluck" | "karplus" | "pluck" => Some(Self::KarplusStrongPluck),
            "granular_cloud" | "granular" | "cloud" => Some(Self::GranularCloud),
            "wavetable_morph" | "wavetable" | "wt" => Some(Self::WavetableMorph),
            "west_coast_wavefolder" | "wavefolder" | "buchla" => Some(Self::WestCoastWavefolder),
            "phase_distortion_cz" | "cz" | "phase_distortion" => Some(Self::PhaseDistortionCz),
            "modal_struck_bar" | "modal" | "struck_bar" => Some(Self::ModalStruckBar),
            _ => None,
        }
    }

    /// Sample the generator. `phase_cycles` is the integrated oscillator phase, `time_sec` the time
    /// since note-on, `morph` the macro in `[0.0, 1.0]`.
    pub fn sample_dsp(self, phase_cycles: f32, time_sec: f32, morph: f32) -> f32 {
        let m = morph.clamp(0.0, 1.0);
        let w = 2.0 * PI * phase_cycles;

        match self {
            Self::Off => (w).sin(),
            Self::KarplusStrongPluck => {
                // High harmonics decay faster than fundamental (dispersion + waveguide damping)
                let mut acc = 0.0_f32;
                for h in 1..=10 {
                    let hf = h as f32;
                    let damp = (-time_sec * (2.5 + hf * hf * (1.2 - 0.8 * m))).exp();
                    acc += ((w * hf + hf * 0.7).sin() / hf) * damp;
                }
                (acc * 0.85).clamp(-1.0, 1.0)
            }
            Self::GranularCloud => {
                // 4 overlapping Hann-windowed micro-grains with octave shimmer
                let grain_rate = 12.0 + m * 28.0;
                let mut acc = 0.0_f32;
                for g in 0..4 {
                    let gf = g as f32;
                    let g_phase = (time_sec * grain_rate + gf * 0.25).rem_euclid(1.0);
                    let hann = 0.5 * (1.0 - (2.0 * PI * g_phase).cos());
                    let ratio = match g {
                        0 => 1.0,
                        1 => 1.004,
                        2 => 2.0,   // octave shimmer
                        _ => 1.498, // fifth grain
                    };
                    acc += hann * (w * ratio + gf * 1.3).sin();
                }
                (acc * 0.45).clamp(-1.0, 1.0)
            }
            Self::WavetableMorph => {
                let s_sine = (w).sin();
                let s_formant = (w).sin() * (w * 3.0).cos();
                let s_fold = ((w).sin() * 3.8).sin();
                let s_hyper = 2.0 * phase_cycles.rem_euclid(1.0) - 1.0;
                let out = if m < 0.33 {
                    let t = m / 0.33;
                    s_sine * (1.0 - t) + s_formant * t
                } else if m < 0.66 {
                    let t = (m - 0.33) / 0.33;
                    s_formant * (1.0 - t) + s_fold * t
                } else {
                    let t = (m - 0.66) / 0.34;
                    s_fold * (1.0 - t) + s_hyper * t
                };
                out.clamp(-1.0, 1.0)
            }
            Self::WestCoastWavefolder => {
                // 5-stage trigonometric wavefolder + vactrol LPG decay
                let folds = 1.5 + m * 7.5;
                let carrier = (w).sin() + 0.25 * (w * 2.0).sin();
                let folded = (carrier * folds).sin() - 0.3 * (carrier * folds * 1.7).cos();
                let vactrol_lpg = (-time_sec * (4.0 - 2.5 * m)).exp();
                (folded * vactrol_lpg * 0.85).clamp(-1.0, 1.0)
            }
            Self::PhaseDistortionCz => {
                // Non-linear phase knee warping + hard-synced resonant window
                let p = phase_cycles.rem_euclid(1.0);
                let knee = (0.5 - m * 0.42).clamp(0.05, 0.5);
                let warped = if p < knee {
                    0.5 * (p / knee)
                } else {
                    0.5 + 0.5 * ((p - knee) / (1.0 - knee))
                };
                let res_freq = 1.0 + m * 6.0;
                let window = 1.0 - p;
                ((2.0 * PI * warped).cos() * 0.5 + (2.0 * PI * p * res_freq).sin() * window * 0.5)
                    .clamp(-1.0, 1.0)
            }
            Self::ModalStruckBar => {
                // Euler-Bernoulli uniform bar eigenmode ratios: 1.0, 2.756, 5.404, 8.933
                let ratios = [1.0_f32, 2.756, 5.404, 8.933];
                let amps = [0.60_f32, 0.30, 0.18, 0.10];
                let mut acc = 0.0_f32;
                for (i, (&r, &a)) in ratios.iter().zip(amps.iter()).enumerate() {
                    let decay = (-time_sec * (3.0 + (i as f32) * (4.5 - 3.0 * m))).exp();
                    acc += (w * r).sin() * a * decay;
                }
                acc.clamp(-1.0, 1.0)
            }
        }
    }
}

/// Virtual Eurorack / patch cable connecting two 3U module jacks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RackCable {
    pub id: String,
    pub from_module: String,
    pub from_jack: String,
    pub to_module: String,
    pub to_jack: String,
    pub color_name: String,
    pub gain: f32,
}

/// Allocation-free routing gains derived from the 3U rack's patch cables.
///
/// `RackRouting::default()` matches [`default_rack_cables()`] so unpatched `SynthRt`
/// instances (CLAP, VCV Rack, offline tests) keep identical default signal flow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RackRouting {
    pub vco1_to_vcf: f32,
    pub vco2_to_fm: f32,
    pub vco2_to_vcf: f32,
    pub pcm_to_vcf: f32,
    pub lfo_to_cutoff: f32,
    pub lfo_to_fm: f32,
    pub adsr_to_vca: f32,
    pub adsr_to_cutoff: f32,
    pub vcf_to_vca: f32,
    pub direct_to_vca: f32,
    pub fold_amount: f32,
    pub fold_symmetry: f32,
    pub sh_rate_hz: f32,
    pub sh_slew: f32,
    pub sh_to_cutoff: f32,
    pub sh_to_fm: f32,
    pub delay_time_s: f32,
    pub delay_feedback: f32,
    pub delay_mix: f32,
    pub clap_host_mix: f32,
    pub clap_host_cv_depth: f32,
    /// `0` = LFO, `1` = Sample & Hold slew CV patched into `CLAP HOST` `PARAM CV`.
    pub clap_host_cv_lfo: f32,
    pub clap_host_cv_sh: f32,
}

impl Default for RackRouting {
    fn default() -> Self {
        Self {
            vco1_to_vcf: 1.0,
            vco2_to_fm: 1.0,
            vco2_to_vcf: 0.0,
            pcm_to_vcf: 1.0,
            lfo_to_cutoff: 1.0,
            lfo_to_fm: 0.0,
            adsr_to_vca: 1.0,
            adsr_to_cutoff: 0.0,
            vcf_to_vca: 1.0,
            direct_to_vca: 0.0,
            fold_amount: 0.0,
            fold_symmetry: 0.0,
            sh_rate_hz: 4.0,
            sh_slew: 0.2,
            sh_to_cutoff: 0.0,
            sh_to_fm: 0.0,
            delay_time_s: 0.3,
            delay_feedback: 0.35,
            delay_mix: 0.0,
            clap_host_mix: 0.0,
            clap_host_cv_depth: 0.5,
            clap_host_cv_lfo: 0.0,
            clap_host_cv_sh: 0.0,
        }
    }
}

fn canonical_module(m: &str) -> &str {
    match m {
        "LFO-MOD" | "ADSR-ENV" => "LFO / ADSR",
        other => other,
    }
}

impl RackRouting {
    /// Compute live DSP routing gains from the current 3U patch cables.
    pub fn from_cables(cables: &[RackCable]) -> Self {
        let mut r = Self {
            vco1_to_vcf: 0.0,
            vco2_to_fm: 0.0,
            vco2_to_vcf: 0.0,
            pcm_to_vcf: 0.0,
            lfo_to_cutoff: 0.0,
            lfo_to_fm: 0.0,
            adsr_to_vca: 0.0,
            adsr_to_cutoff: 0.0,
            vcf_to_vca: 0.0,
            direct_to_vca: 0.0,
            fold_amount: 0.0,
            fold_symmetry: 0.0,
            sh_rate_hz: 4.0,
            sh_slew: 0.2,
            sh_to_cutoff: 0.0,
            sh_to_fm: 0.0,
            delay_time_s: 0.3,
            delay_feedback: 0.35,
            delay_mix: 0.0,
            clap_host_mix: 0.0,
            clap_host_cv_depth: 0.5,
            clap_host_cv_lfo: 0.0,
            clap_host_cv_sh: 0.0,
        };
        for c in cables {
            let fm = canonical_module(&c.from_module);
            let tm = canonical_module(&c.to_module);
            let g = c.gain.clamp(0.0, 2.0);
            match (fm, c.from_jack.as_str(), tm, c.to_jack.as_str()) {
                ("VCO-1", "SAW OUT", "VCF-LADDER", "AUDIO IN" | "AUX IN") => {
                    r.vco1_to_vcf = (r.vco1_to_vcf + g).clamp(0.0, 2.0);
                }
                ("VCO-2 / X-DSP", "DSP OUT", "VCO-1", "FM CV") => {
                    // Default cable gain is 0.5 -> normalized to 1.0 so existing patches match.
                    r.vco2_to_fm = (r.vco2_to_fm + g * 2.0).clamp(0.0, 2.0);
                }
                ("VCO-2 / X-DSP", "DSP OUT", "VCF-LADDER", "AUDIO IN" | "AUX IN") => {
                    r.vco2_to_vcf = (r.vco2_to_vcf + g).clamp(0.0, 2.0);
                }
                ("PCM-WORKSTATION", "AI2 ROM OUT", "VCF-LADDER", "AUX IN" | "AUDIO IN") => {
                    // Default cable gain is 0.85 -> normalized to 1.0.
                    r.pcm_to_vcf = (r.pcm_to_vcf + g / 0.85).clamp(0.0, 2.0);
                }
                ("LFO / ADSR", "SINE OUT", "VCF-LADDER", "CUTOFF CV") => {
                    // Default cable gain is 0.4 -> normalized to 1.0.
                    r.lfo_to_cutoff = (r.lfo_to_cutoff + g / 0.4).clamp(0.0, 2.0);
                }
                ("LFO / ADSR", "SINE OUT", "VCO-1", "FM CV") => {
                    r.lfo_to_fm = (r.lfo_to_fm + g).clamp(0.0, 2.0);
                }
                ("LFO / ADSR", "ENV OUT", "VCA-MASTER", "GAIN CV") => {
                    r.adsr_to_vca = (r.adsr_to_vca + g).clamp(0.0, 2.0);
                }
                ("LFO / ADSR", "ENV OUT", "VCF-LADDER", "CUTOFF CV") => {
                    r.adsr_to_cutoff = (r.adsr_to_cutoff + g).clamp(0.0, 2.0);
                }
                ("VCF-LADDER", "LP24 OUT", "VCA-MASTER", "SIGNAL IN") => {
                    r.vcf_to_vca = (r.vcf_to_vca + g).clamp(0.0, 2.0);
                }
                (
                    "VCO-1" | "VCO-2 / X-DSP" | "PCM-WORKSTATION",
                    "SAW OUT" | "DSP OUT" | "AI2 ROM OUT",
                    "VCA-MASTER",
                    "SIGNAL IN",
                ) => {
                    r.direct_to_vca = (r.direct_to_vca + g).clamp(0.0, 2.0);
                }
                _ => {}
            }
        }
        r
    }
}

/// MIDI Depot clip entry with note sequence and Standard MIDI File (`.mid`) export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiDepotClip {
    pub id: String,
    pub title: String,
    pub genre: String,
    pub bpm: f32,
    pub camelot_key: String,
    pub recommended_m1: Option<String>,
    pub recommended_extreme_dsp: String,
    pub notes: Vec<u8>,
}

/// Curated 1-Click Demo Showcase item combining SynthPatch + PCM workstation + Extreme DSP + MIDI Clip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoShowcaseItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub patch_id: String,
    pub m1_program: Option<PcmProgram>,
    pub extreme_dsp: ExtremeDspMode,
    pub midi_clip_id: String,
    pub bpm: f32,
}

/// Hardware MIDI Controller configuration and CC mapping table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiHardwareConfig {
    pub selected_port: String,
    pub available_ports: Vec<String>,
    pub midi_channel: u8, // 0 = Omni, 1..=16 = Channel
    pub velocity_curve: String,
    pub pitch_bend_semitones: u8,
    pub last_triggered_note: u8,
    pub cc_mappings: Vec<(u8, String)>,
}

impl Default for MidiHardwareConfig {
    fn default() -> Self {
        Self {
            selected_port: "USB MIDI Keyboard (WinMM Auto-Detect / Omni)".into(),
            available_ports: vec![
                "USB MIDI Keyboard (WinMM Auto-Detect / Omni)".into(),
                "Arturia KeyStep 37 / MiniLab 3 (USB MIDI)".into(),
                "Akai MPK Mini Mk3 (USB MIDI)".into(),
                "USB MIDI keyboard (MIDI IN)".into(),
                "Novation Launchkey / Native Instruments Komplete Kontrol".into(),
                "Virtual On-Screen 25-Key Studio Keyboard".into(),
            ],
            midi_channel: 0,
            velocity_curve: "Linear".into(),
            pitch_bend_semitones: 2,
            last_triggered_note: 60, // C4
            cc_mappings: vec![
                (1, "Mod Wheel -> LFO Filter Depth (0x100A)".into()),
                (74, "Brightness (CC#74) -> VCF Cutoff Hz (0x1004)".into()),
                (71, "Timbre (CC#71) -> VCF Resonance Q (0x1005)".into()),
                (73, "Attack (CC#73) -> ADSR Attack ms (0x1007)".into()),
                (72, "Release (CC#72) -> ADSR Release ms".into()),
                (91, "FX Depth (CC#91) -> Extreme DSP Morph Macro".into()),
            ],
        }
    }
}

pub fn default_rack_cables() -> Vec<RackCable> {
    vec![
        RackCable {
            id: "cab_vco1_vcf".into(),
            from_module: "VCO-1".into(),
            from_jack: "SAW OUT".into(),
            to_module: "VCF-LADDER".into(),
            to_jack: "AUDIO IN".into(),
            color_name: "green".into(),
            gain: 1.0,
        },
        RackCable {
            id: "cab_vco2_fm".into(),
            from_module: "VCO-2 / X-DSP".into(),
            from_jack: "DSP OUT".into(),
            to_module: "VCO-1".into(),
            to_jack: "FM CV".into(),
            color_name: "amber".into(),
            gain: 0.5,
        },
        RackCable {
            id: "cab_m1_vcf".into(),
            from_module: "PCM-WORKSTATION".into(),
            from_jack: "AI2 ROM OUT".into(),
            to_module: "VCF-LADDER".into(),
            to_jack: "AUX IN".into(),
            color_name: "purple".into(),
            gain: 0.85,
        },
        RackCable {
            id: "cab_lfo_cutoff".into(),
            from_module: "LFO-MOD".into(),
            from_jack: "SINE OUT".into(),
            to_module: "VCF-LADDER".into(),
            to_jack: "CUTOFF CV".into(),
            color_name: "cyan".into(),
            gain: 0.4,
        },
        RackCable {
            id: "cab_adsr_vca".into(),
            from_module: "ADSR-ENV".into(),
            from_jack: "ENV OUT".into(),
            to_module: "VCA-MASTER".into(),
            to_jack: "GAIN CV".into(),
            color_name: "red".into(),
            gain: 1.0,
        },
        RackCable {
            id: "cab_vcf_vca".into(),
            from_module: "VCF-LADDER".into(),
            from_jack: "LP24 OUT".into(),
            to_module: "VCA-MASTER".into(),
            to_jack: "SIGNAL IN".into(),
            color_name: "green".into(),
            gain: 1.0,
        },
    ]
}

pub fn builtin_midi_depot() -> Vec<MidiDepotClip> {
    vec![
        MidiDepotClip {
            id: "90s_house_m1_piano".into(),
            title: "1990 Ibiza Rave Piano 16' Anthem".into(),
            genre: "90s House / Rave".into(),
            bpm: 126.0,
            camelot_key: "8A Am".into(),
            recommended_m1: Some("m1_01_piano16".into()),
            recommended_extreme_dsp: "off".into(),
            notes: vec![
                57, 57, 60, 57, 62, 60, 64, 62, 65, 65, 64, 62, 60, 57, 55, 57,
            ],
        },
        MidiDepotClip {
            id: "club_organ_bass".into(),
            title: "Club Organ 2 Bass".into(),
            genre: "Classic 90s Club House".into(),
            bpm: 124.0,
            camelot_key: "4A Fm".into(),
            recommended_m1: Some("m1_17_organ2".into()),
            recommended_extreme_dsp: "off".into(),
            notes: vec![
                41, 41, 48, 41, 44, 41, 51, 48, 41, 41, 53, 41, 46, 48, 44, 41,
            ],
        },
        MidiDepotClip {
            id: "vienna_acid_warehouse".into(),
            title: "Vienna Gürtel Warehouse 303 Acid Line".into(),
            genre: "Acid Techno".into(),
            bpm: 132.0,
            camelot_key: "8A Am".into(),
            recommended_m1: None,
            recommended_extreme_dsp: "off".into(),
            notes: vec![
                45, 45, 57, 45, 48, 45, 55, 52, 45, 57, 60, 45, 50, 52, 55, 57,
            ],
        },
        MidiDepotClip {
            id: "m1_universe_cosmic_pad".into(),
            title: "PCM workstation '00 Universe' Cosmic Glass Sequence".into(),
            genre: "90s Ambient / Electronica".into(),
            bpm: 108.0,
            camelot_key: "9A Em".into(),
            recommended_m1: Some("m1_00_universe".into()),
            recommended_extreme_dsp: "granular_cloud".into(),
            notes: vec![
                52, 55, 59, 64, 66, 64, 59, 55, 50, 54, 57, 62, 64, 62, 57, 54,
            ],
        },
        MidiDepotClip {
            id: "buchla_westcoast_bongo".into(),
            title: "West-Coast Wavefolder Polyrhythm".into(),
            genre: "West-Coast Modular".into(),
            bpm: 128.0,
            camelot_key: "2A D#m".into(),
            recommended_m1: None,
            recommended_extreme_dsp: "west_coast_wavefolder".into(),
            notes: vec![
                51, 58, 54, 63, 51, 61, 58, 54, 51, 58, 66, 63, 58, 54, 51, 46,
            ],
        },
        MidiDepotClip {
            id: "karplus_acoustic_harp".into(),
            title: "Karplus-Strong Plucked String Arpeggio".into(),
            genre: "Physical Modeling".into(),
            bpm: 118.0,
            camelot_key: "11B A".into(),
            recommended_m1: Some("m1_24_panflute".into()),
            recommended_extreme_dsp: "karplus_strong_pluck".into(),
            notes: vec![
                57, 61, 64, 69, 73, 69, 64, 61, 59, 62, 66, 71, 74, 71, 66, 62,
            ],
        },
        MidiDepotClip {
            id: "modal_glass_marimba".into(),
            title: "Euler-Bernoulli Modal Struck Glass Bar".into(),
            genre: "IDM / Cinematic".into(),
            bpm: 122.0,
            camelot_key: "8A Am".into(),
            recommended_m1: Some("m1_36_choir".into()),
            recommended_extreme_dsp: "modal_struck_bar".into(),
            notes: vec![
                69, 72, 76, 79, 81, 79, 76, 72, 67, 71, 74, 79, 76, 74, 71, 67,
            ],
        },
        MidiDepotClip {
            id: "goliath_neuro_reese_drop".into(),
            title: "Goliath 174 BPM Neurofunk Wavetable Drop".into(),
            genre: "Drum & Bass".into(),
            bpm: 174.0,
            camelot_key: "4A Fm".into(),
            recommended_m1: Some("m1_48_orchhit".into()),
            recommended_extreme_dsp: "wavetable_morph".into(),
            notes: vec![
                41, 41, 41, 53, 44, 44, 44, 56, 39, 39, 39, 51, 36, 36, 39, 41,
            ],
        },
    ]
}

pub fn builtin_demo_showcases() -> Vec<DemoShowcaseItem> {
    vec![
        DemoShowcaseItem {
            id: "demo_1990_m1_piano".into(),
            title: "1990 PCM workstation '01 Piano 16' Ibiza Rave Anthem".into(),
            description: "The legendary 1990 PCM workstation Piano 16' PCM attack + house chord progression at 126 BPM.".into(),
            patch_id: "vienna_acid_303".into(),
            m1_program: Some(PcmProgram::Piano16_01),
            extreme_dsp: ExtremeDspMode::Off,
            midi_clip_id: "90s_house_m1_piano".into(),
            bpm: 126.0,
        },
        DemoShowcaseItem {
            id: "demo_club_organ_bass".into(),
            title: "1990 PCM workstation '17 Organ 2' Club Bassline".into(),
            description: "Percussive 3rd-harmonic Organ 2 drawbar bass with 8ms key-click transient.".into(),
            patch_id: "danube_dub_chord".into(),
            m1_program: Some(PcmProgram::Organ2_17),
            extreme_dsp: ExtremeDspMode::Off,
            midi_clip_id: "club_organ_bass".into(),
            bpm: 124.0,
        },
        DemoShowcaseItem {
            id: "demo_m1_universe_granular".into(),
            title: "PCM workstation '00 Universe' + Granular Shimmer Cloud".into(),
            description: "Combines the iconic M1 glass choir/bell PCM with a 4-grain octave shimmer cloud.".into(),
            patch_id: "soundcraft_warm_pad".into(),
            m1_program: Some(PcmProgram::Universe00),
            extreme_dsp: ExtremeDspMode::GranularCloud,
            midi_clip_id: "m1_universe_cosmic_pad".into(),
            bpm: 108.0,
        },
        DemoShowcaseItem {
            id: "demo_buchla_wavefolder".into(),
            title: "West-Coast Wavefolder + Vactrol LPG".into(),
            description: "5-stage trigonometric wavefolding with organic vactrol low-pass gate decay.".into(),
            patch_id: "vienna_acid_303".into(),
            m1_program: None,
            extreme_dsp: ExtremeDspMode::WestCoastWavefolder,
            midi_clip_id: "buchla_westcoast_bongo".into(),
            bpm: 128.0,
        },
        DemoShowcaseItem {
            id: "demo_karplus_panflute".into(),
            title: "Karplus-Strong Plucked String + M1 Pan Flute".into(),
            description: "Waveguide physical string model layered with the 1990 PCM workstation breathy Pan Flute.".into(),
            patch_id: "stephansdom_bell_fm".into(),
            m1_program: Some(PcmProgram::PanFlute24),
            extreme_dsp: ExtremeDspMode::KarplusStrongPluck,
            midi_clip_id: "karplus_acoustic_harp".into(),
            bpm: 118.0,
        },
        DemoShowcaseItem {
            id: "demo_modal_glass_choir".into(),
            title: "Modal Struck Glass Bar + M1 Ooh/Aah Choir".into(),
            description: "Euler-Bernoulli inharmonic bar eigenmodes layered over the M1 formant choir.".into(),
            patch_id: "stephansdom_bell_fm".into(),
            m1_program: Some(PcmProgram::OohAahChoir36),
            extreme_dsp: ExtremeDspMode::ModalStruckBar,
            midi_clip_id: "modal_glass_marimba".into(),
            bpm: 122.0,
        },
    ]
}

/// Write a Standard MIDI File (`.mid`, SMF Format 0, 480 PPQN) to disk from a `MidiDepotClip`.
pub fn export_midi_smf_file(clip: &MidiDepotClip, path: &Path) -> Result<usize, SynthError> {
    let mut track_data: Vec<u8> = Vec::new();
    let step_ticks: u8 = 120; // 1/16th note at 480 PPQN

    for &note in &clip.notes {
        // Delta = 0, Note On Ch 0 (0x90), note, vel 100
        track_data.extend_from_slice(&[0x00, 0x90, note.clamp(0, 127), 100]);
        // Delta = 120 ticks, Note Off Ch 0 (0x80), note, vel 64
        track_data.extend_from_slice(&[step_ticks, 0x80, note.clamp(0, 127), 64]);
    }
    // End of Track meta-event
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    let mut bytes = Vec::with_capacity(22 + track_data.len());
    // MThd header (Format 0, 1 track, 480 PPQN = 0x01E0)
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&6_u32.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&1_u16.to_be_bytes());
    bytes.extend_from_slice(&480_u16.to_be_bytes());

    // MTrk chunk
    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&track_data);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SynthError::Io(format!("create_dir_all failed: {e}")))?;
        }
    }
    std::fs::write(path, &bytes).map_err(|e| SynthError::Io(format!("write mid failed: {e}")))?;
    Ok(bytes.len())
}
