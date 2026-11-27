//! Phase 4 Next-Gen DSP:
//! - 6-Operator Thru-Zero FM Routing Matrix (`Fm6Params` / `Fm6Runtime`) with interactive 6x6
//!   modulation matrix, self-feedback, harmonic ratios, and classic DX7/FM8/Opsix algorithms.
//! - Per-Voice 16-Voice Hyper-Unison Stack (`UnisonParams` / `UnisonRuntime`) with SuperSaw,
//!   Octave, Fifth, Minor7 Chord, and Golden-Ratio detune spreads + stereo width.

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

pub const FM_OPS: usize = 6;
pub const MAX_UNISON_VOICES: usize = 16;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fm6Params {
    pub enabled: bool,
    pub mix: f32,
    pub preset_name: String,
    pub ratios: [f32; FM_OPS],
    pub levels: [f32; FM_OPS],           // carrier output level per op
    pub matrix: [[f32; FM_OPS]; FM_OPS], // matrix[src][dst] = FM mod depth (0.0..4.0)
}

impl Default for Fm6Params {
    fn default() -> Self {
        let mut p = Self::preset("dx7_epiano");
        p.enabled = false;
        p
    }
}

impl Fm6Params {
    pub fn preset_names() -> &'static [(&'static str, &'static str)] {
        &[
            ("dx7_epiano", "DX7 Classic E-Piano"),
            ("berlin_fm_bell", "Berlin FM Metallic Bell"),
            ("lately_bass", "1990 Lately Solid Bass"),
            ("neuro_growl", "Neuro Reese FM Growl"),
            ("glass_choir", "Inharmonic Glass Choir"),
        ]
    }

    pub fn preset(id: &str) -> Self {
        let mut matrix = [[0.0_f32; FM_OPS]; FM_OPS];
        let (preset_name, ratios, levels) = match id {
            "berlin_fm_bell" => {
                matrix[1][0] = 1.85;
                matrix[2][1] = 0.90;
                matrix[4][3] = 2.10;
                matrix[5][4] = 0.75;
                matrix[5][5] = 0.45; // self-feedback
                (
                    "Berlin FM Metallic Bell".into(),
                    [1.0, 3.5, 7.0, 1.0, 2.76, 5.42],
                    [0.85, 0.0, 0.0, 0.75, 0.0, 0.0],
                )
            }
            "lately_bass" => {
                matrix[1][0] = 2.40;
                matrix[2][1] = 1.25;
                matrix[3][2] = 0.85;
                matrix[3][3] = 0.55;
                matrix[5][4] = 1.60;
                (
                    "1990 Lately Solid Bass".into(),
                    [0.5, 1.0, 2.0, 1.0, 0.5, 3.0],
                    [0.95, 0.0, 0.0, 0.0, 0.65, 0.0],
                )
            }
            "neuro_growl" => {
                matrix[1][0] = 2.80;
                matrix[2][0] = 1.45;
                matrix[3][1] = 1.90;
                matrix[4][2] = 1.60;
                matrix[5][3] = 1.20;
                matrix[0][0] = 0.35;
                (
                    "Neuro Reese FM Growl".into(),
                    [1.0, 1.01, 2.0, 3.02, 0.5, 4.0],
                    [0.90, 0.25, 0.0, 0.0, 0.35, 0.0],
                )
            }
            "glass_choir" => {
                matrix[1][0] = 0.85;
                matrix[3][2] = 1.15;
                matrix[5][4] = 1.40;
                matrix[2][0] = 0.35;
                (
                    "Inharmonic Glass Choir".into(),
                    [1.0, 2.0, 1.5, 3.0, 2.01, 6.0],
                    [0.75, 0.0, 0.65, 0.0, 0.55, 0.0],
                )
            }
            _ => {
                // Classic DX7 Electric Piano (Algorithm 5: three 2-op pairs)
                matrix[1][0] = 1.35; // warm body
                matrix[3][2] = 2.20; // tine attack
                matrix[5][4] = 1.60; // bell shimmer
                matrix[5][5] = 0.30;
                (
                    "DX7 Classic E-Piano".into(),
                    [1.0, 1.0, 1.0, 14.0, 1.0, 1.0],
                    [0.90, 0.0, 0.65, 0.0, 0.45, 0.0],
                )
            }
        };
        Self {
            enabled: true,
            mix: 0.65,
            preset_name,
            ratios,
            levels,
            matrix,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fm6Runtime {
    phases: [f32; FM_OPS],
    prev_out: [f32; FM_OPS],
}

impl Default for Fm6Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Fm6Runtime {
    pub fn new() -> Self {
        Self {
            phases: [0.0; FM_OPS],
            prev_out: [0.0; FM_OPS],
        }
    }

    /// Synthesize one stereo frame of the 6-Op Thru-Zero FM Matrix. Allocation-free.
    #[inline]
    pub fn tick_stereo(&mut self, base_freq_hz: f32, params: &Fm6Params, sr: f32) -> (f32, f32) {
        if !params.enabled || params.mix <= 0.001 {
            return (0.0, 0.0);
        }
        let dt = 1.0 / sr.max(8000.0);
        let mut cur_out = [0.0_f32; FM_OPS];
        let mut sum_l = 0.0_f32;
        let mut sum_r = 0.0_f32;

        for (dst, out_slot) in cur_out.iter_mut().enumerate().take(FM_OPS) {
            let mut mod_sum = 0.0_f32;
            for src in 0..FM_OPS {
                let depth = params.matrix[src][dst];
                if depth.abs() > 1e-4 {
                    mod_sum += self.prev_out[src] * depth;
                }
            }
            let freq =
                (base_freq_hz * params.ratios[dst].clamp(0.125, 24.0)).clamp(10.0, sr * 0.44);
            // Thru-zero phase modulation: instant phase can move smoothly through zero
            self.phases[dst] = (self.phases[dst] + freq * dt).rem_euclid(1.0);
            let sample = (TAU * self.phases[dst] + mod_sum).sin();
            *out_slot = sample;

            let lvl = params.levels[dst].clamp(0.0, 1.5);
            if lvl > 0.001 {
                let pan = (dst as f32 / (FM_OPS as f32 - 1.0)) * 0.7 - 0.35;
                let gl = ((1.0 - pan) * 0.5).sqrt();
                let gr = ((1.0 + pan) * 0.5).sqrt();
                sum_l += sample * lvl * gl;
                sum_r += sample * lvl * gr;
            }
        }
        self.prev_out = cur_out;

        let g = 0.32 * params.mix.clamp(0.0, 1.0);
        (sum_l * g, sum_r * g)
    }
}

// ---------------------------------------------------------------------------
// 16-Voice Hyper-Unison Stack
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnisonStackMode {
    SuperSaw,
    OctaveStack,
    FifthPower,
    Minor7Chord,
    GoldenRatio,
}

impl UnisonStackMode {
    pub const ALL: [Self; 5] = [
        Self::SuperSaw,
        Self::OctaveStack,
        Self::FifthPower,
        Self::Minor7Chord,
        Self::GoldenRatio,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SuperSaw => "SuperSaw",
            Self::OctaveStack => "Octave Stack",
            Self::FifthPower => "Fifth Power",
            Self::Minor7Chord => "Minor 7th Chord",
            Self::GoldenRatio => "Golden Ratio",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UnisonParams {
    pub enabled: bool,
    pub voices: u32, // 2..=16
    pub detune_cents: f32,
    pub stereo_spread: f32,
    pub stack_mode: UnisonStackMode,
    pub mix: f32,
}

impl Default for UnisonParams {
    fn default() -> Self {
        Self {
            enabled: false,
            voices: 7,
            detune_cents: 22.0,
            stereo_spread: 0.85,
            stack_mode: UnisonStackMode::SuperSaw,
            mix: 0.55,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnisonRuntime {
    phases: [f32; MAX_UNISON_VOICES],
}

impl Default for UnisonRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl UnisonRuntime {
    pub fn new() -> Self {
        // Stagger initial phases so unison chords don't create a huge phase-aligned transient spike
        let mut phases = [0.0_f32; MAX_UNISON_VOICES];
        for (i, p) in phases.iter_mut().enumerate() {
            *p = (i as f32 * 0.618_034).fract();
        }
        Self { phases }
    }

    #[inline]
    pub fn tick_stereo(&mut self, base_freq_hz: f32, params: &UnisonParams, sr: f32) -> (f32, f32) {
        if !params.enabled || params.mix <= 0.001 {
            return (0.0, 0.0);
        }
        let count = (params.voices as usize).clamp(2, MAX_UNISON_VOICES);
        let dt = 1.0 / sr.max(8000.0);
        let spread = params.stereo_spread.clamp(0.0, 1.0);
        let mut sum_l = 0.0_f32;
        let mut sum_r = 0.0_f32;

        for (i, phase) in self.phases.iter_mut().enumerate().take(count) {
            let norm = if count > 1 {
                (i as f32 / (count - 1) as f32) * 2.0 - 1.0
            } else {
                0.0
            };
            let stack_semitones = match params.stack_mode {
                UnisonStackMode::SuperSaw => 0.0,
                UnisonStackMode::OctaveStack => match i % 3 {
                    0 => 0.0,
                    1 => 12.0,
                    _ => -12.0,
                },
                UnisonStackMode::FifthPower => match i % 3 {
                    0 => 0.0,
                    1 => 7.0,
                    _ => 12.0,
                },
                UnisonStackMode::Minor7Chord => match i % 4 {
                    0 => 0.0,
                    1 => 3.0,
                    2 => 7.0,
                    _ => 10.0,
                },
                UnisonStackMode::GoldenRatio => {
                    let phi = (1.0_f32 + 5.0_f32.sqrt()) * 0.5;
                    ((i as f32 * phi).fract() - 0.5) * 2.0
                }
            };
            let cents = norm * params.detune_cents.clamp(0.0, 100.0) + stack_semitones * 100.0;
            let freq = (base_freq_hz * 2.0_f32.powf(cents / 1200.0)).clamp(15.0, sr * 0.44);
            *phase = (*phase + freq * dt).fract();
            // Band-softened sawtooth
            let s = 2.0 * *phase - 1.0;
            let pan = norm * spread;
            let gl = ((1.0 - pan) * 0.5).sqrt();
            let gr = ((1.0 + pan) * 0.5).sqrt();
            sum_l += s * gl;
            sum_r += s * gr;
        }

        let norm_g = (0.35 * params.mix.clamp(0.0, 1.0)) / (count as f32).sqrt();
        (sum_l * norm_g, sum_r * norm_g)
    }
}
