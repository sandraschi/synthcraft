//! Phase 1 Next-Gen DSP:
//! - Drawable Multi-Segment Envelope Generator (`MsegParams` / `MsegRuntime`) with up to 16 spline points
//! - Dual Morphing Filter B (`Filter2Params` / `Filter2StereoState`) supporting Serial / Parallel routing,
//!   Formant Vowel (A-E-I-O-U) 3-bandpass vocal tract, Comb +/- resonator, MS-20 Sallen-Key diode screamer,
//!   and classic LP/BP/HP modes.

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

pub const MSEG_MAX_POINTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MsegMode {
    Off,
    TempoLoop,
    FreeHz,
    OneShot,
}

impl MsegMode {
    pub const ALL: [Self; 4] = [Self::Off, Self::TempoLoop, Self::FreeHz, Self::OneShot];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::TempoLoop => "Tempo Loop",
            Self::FreeHz => "Free Hz",
            Self::OneShot => "One-Shot",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MsegPoint {
    pub x: f32,     // 0.0 ..= 1.0 normalized time
    pub y: f32,     // 0.0 ..= 1.0 modulation level
    pub curve: f32, // -1.0 (log) .. 0.0 (linear) .. +1.0 (exp)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MsegParams {
    pub mode: MsegMode,
    pub rate_hz: f32,
    pub tempo_beats: f32, // e.g. 1.0, 2.0, 4.0, 8.0 beats per cycle
    pub cutoff_depth: f32,
    pub morph_depth: f32,
    pub filter2_depth: f32,
    pub points: Vec<MsegPoint>,
}

impl Default for MsegParams {
    fn default() -> Self {
        Self {
            mode: MsegMode::Off,
            rate_hz: 1.0,
            tempo_beats: 4.0,
            cutoff_depth: 0.35,
            morph_depth: 0.25,
            filter2_depth: 0.40,
            points: Self::preset_points("acid_pluck"),
        }
    }
}

impl MsegParams {
    pub fn preset_names() -> &'static [(&'static str, &'static str)] {
        &[
            ("acid_pluck", "Acid Pluck"),
            ("trance_gate", "Trance Gate"),
            ("edm_riser", "EDM Riser"),
            ("wobble_sub", "Dubstep Wobble"),
            ("swell_pad", "Cinematic Swell"),
            ("chaos_step", "Stepped Chaos"),
        ]
    }

    pub fn preset_points(id: &str) -> Vec<MsegPoint> {
        let mk = |pts: &[(f32, f32, f32)]| -> Vec<MsegPoint> {
            pts.iter()
                .map(|&(x, y, curve)| MsegPoint { x, y, curve })
                .collect()
        };
        match id {
            "trance_gate" => mk(&[
                (0.0, 1.0, 0.0),
                (0.11, 1.0, 0.0),
                (0.125, 0.08, 0.0),
                (0.24, 0.08, 0.0),
                (0.25, 0.92, 0.0),
                (0.36, 0.92, 0.0),
                (0.375, 0.05, 0.0),
                (0.50, 1.0, 0.0),
                (0.62, 0.15, 0.0),
                (0.75, 0.85, 0.0),
                (0.87, 0.10, 0.0),
                (1.0, 0.0, 0.0),
            ]),
            "edm_riser" => mk(&[
                (0.0, 0.05, 0.6),
                (0.45, 0.25, 0.5),
                (0.80, 0.72, 0.4),
                (0.95, 1.0, 0.0),
                (1.0, 0.0, 0.0),
            ]),
            "wobble_sub" => mk(&[
                (0.0, 0.1, -0.4),
                (0.125, 1.0, 0.4),
                (0.25, 0.1, -0.4),
                (0.375, 0.85, 0.4),
                (0.50, 0.1, -0.4),
                (0.625, 1.0, 0.4),
                (0.75, 0.1, -0.4),
                (0.875, 0.75, 0.4),
                (1.0, 0.1, 0.0),
            ]),
            "swell_pad" => mk(&[
                (0.0, 0.0, -0.5),
                (0.35, 0.82, -0.2),
                (0.65, 0.95, 0.2),
                (1.0, 0.05, 0.4),
            ]),
            "chaos_step" => mk(&[
                (0.0, 0.2, 0.0),
                (0.16, 0.88, -0.3),
                (0.32, 0.35, 0.3),
                (0.48, 0.96, -0.4),
                (0.64, 0.15, 0.4),
                (0.80, 0.74, 0.0),
                (1.0, 0.2, 0.0),
            ]),
            _ => mk(&[
                (0.0, 1.0, 0.55),
                (0.22, 0.28, 0.45),
                (0.55, 0.10, 0.2),
                (1.0, 0.0, 0.0),
            ]),
        }
    }

    pub fn sort_and_clamp(&mut self) {
        for p in &mut self.points {
            p.x = p.x.clamp(0.0, 1.0);
            p.y = p.y.clamp(0.0, 1.0);
            p.curve = p.curve.clamp(-1.0, 1.0);
        }
        self.points.sort_by(|a, b| a.x.total_cmp(&b.x));
        if self.points.len() < 2 {
            self.points = Self::preset_points("acid_pluck");
        }
        self.points.truncate(MSEG_MAX_POINTS);
        if let Some(first) = self.points.first_mut() {
            first.x = 0.0;
        }
        if let Some(last) = self.points.last_mut() {
            last.x = 1.0;
        }
    }

    /// Evaluate the MSEG spline at normalized phase `t` in `[0.0, 1.0]`. Allocation-free.
    #[inline]
    pub fn eval(&self, t: f32) -> f32 {
        let pts = &self.points;
        if pts.is_empty() {
            return 0.0;
        }
        if pts.len() == 1 {
            return pts[0].y;
        }
        let x = t.clamp(0.0, 1.0);
        for w in pts.windows(2) {
            let a = w[0];
            let b = w[1];
            if x >= a.x && x <= b.x {
                let dx = (b.x - a.x).max(1e-5);
                let mut u = ((x - a.x) / dx).clamp(0.0, 1.0);
                // Curve shaping: curve > 0 bends downward (exp), curve < 0 bends upward (log)
                let exp_pow = 2.0_f32.powf(a.curve.clamp(-1.0, 1.0) * 2.2);
                u = u.powf(exp_pow);
                return (a.y + (b.y - a.y) * u).clamp(0.0, 1.0);
            }
        }
        pts[pts.len() - 1].y
    }
}

#[derive(Debug, Clone, Default)]
pub struct MsegRuntime {
    pub phase: f32,
    pub last_value: f32,
    finished: bool,
}

impl MsegRuntime {
    pub fn retrigger(&mut self) {
        self.phase = 0.0;
        self.finished = false;
    }

    #[inline]
    pub fn tick(&mut self, params: &MsegParams, bpm: f32, sr: f32) -> f32 {
        if params.mode == MsegMode::Off {
            self.last_value = 0.0;
            return 0.0;
        }
        let freq = match params.mode {
            MsegMode::Off => 0.0,
            MsegMode::FreeHz => params.rate_hz.clamp(0.05, 32.0),
            MsegMode::TempoLoop | MsegMode::OneShot => {
                let beats = params.tempo_beats.clamp(0.25, 16.0);
                (bpm.clamp(40.0, 240.0) / 60.0) / beats
            }
        };
        let val = params.eval(self.phase);
        self.last_value = val;

        if !self.finished {
            self.phase += freq / sr.max(8000.0);
            if self.phase >= 1.0 {
                if params.mode == MsegMode::OneShot {
                    self.phase = 1.0;
                    self.finished = true;
                } else {
                    self.phase -= 1.0;
                }
            }
        }
        val
    }
}

// ---------------------------------------------------------------------------
// Dual Morphing Filter B (Serial / Parallel + Formant A-E-I-O-U + Comb + MS-20)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Filter2Mode {
    Off,
    LowPass24dB,
    BandPass12dB,
    HighPass12dB,
    FormantVowel,
    CombPositive,
    CombNegative,
    Ms20Screamer,
}

impl Filter2Mode {
    pub const ALL: [Self; 8] = [
        Self::Off,
        Self::LowPass24dB,
        Self::BandPass12dB,
        Self::HighPass12dB,
        Self::FormantVowel,
        Self::CombPositive,
        Self::CombNegative,
        Self::Ms20Screamer,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::LowPass24dB => "LP 24dB",
            Self::BandPass12dB => "BP 12dB",
            Self::HighPass12dB => "HP 12dB",
            Self::FormantVowel => "Formant (A-E-I-O-U)",
            Self::CombPositive => "Comb (+)",
            Self::CombNegative => "Comb (-)",
            Self::Ms20Screamer => "MS-20 Screamer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Filter2Routing {
    Serial,
    Parallel,
}

impl Filter2Routing {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Serial => "Serial (A -> B)",
            Self::Parallel => "Parallel (A + B)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Filter2Params {
    pub mode: Filter2Mode,
    pub routing: Filter2Routing,
    pub cutoff_hz: f32,
    pub resonance: f32,   // 0.0 ..= 1.0
    pub vowel_morph: f32, // 0.0 (A) .. 1.0 (E) .. 2.0 (I) .. 3.0 (O) .. 4.0 (U)
    pub mix: f32,         // 0.0 ..= 1.0 wet/dry
}

impl Default for Filter2Params {
    fn default() -> Self {
        Self {
            mode: Filter2Mode::Off,
            routing: Filter2Routing::Serial,
            cutoff_hz: 1400.0,
            resonance: 0.55,
            vowel_morph: 0.0,
            mix: 0.75,
        }
    }
}

/// 2-pole State Variable Filter primitive used for Formant vocal tract & MS-20 Sallen-Key.
#[derive(Debug, Clone, Copy, Default)]
struct Svf2 {
    lp: f32,
    bp: f32,
}

impl Svf2 {
    #[inline]
    fn tick(&mut self, input: f32, freq_hz: f32, q: f32, sr: f32) -> (f32, f32, f32) {
        let f = (2.0 * (std::f32::consts::PI * freq_hz.clamp(30.0, sr * 0.42) / sr).sin())
            .clamp(0.001, 0.95);
        let damp = (1.0 / q.clamp(0.5, 16.0)).clamp(0.06, 2.0);
        let hp = input - self.lp - damp * self.bp;
        self.bp += f * hp;
        self.lp += f * self.bp;
        (self.lp, self.bp, hp)
    }
}

#[derive(Debug, Clone)]
struct Filter2Channel {
    svf1: Svf2,
    svf2: Svf2,
    svf3: Svf2,
    comb_buf: Vec<f32>,
    comb_pos: usize,
}

impl Filter2Channel {
    fn new() -> Self {
        Self {
            svf1: Svf2::default(),
            svf2: Svf2::default(),
            svf3: Svf2::default(),
            comb_buf: vec![0.0; 4096],
            comb_pos: 0,
        }
    }

    #[inline]
    fn process(&mut self, input: f32, params: &Filter2Params, mod_val: f32, sr: f32) -> f32 {
        let res = params.resonance.clamp(0.0, 0.98);
        let cut = (params.cutoff_hz * 2.0_f32.powf(mod_val * 2.5)).clamp(40.0, 16000.0);
        let wet = match params.mode {
            Filter2Mode::Off => return input,
            Filter2Mode::LowPass24dB => {
                let q = 0.7 + res * 7.5;
                let (lp1, _, _) = self.svf1.tick(input, cut, q, sr);
                let (lp2, _, _) = self.svf2.tick(lp1, cut, q * 0.85, sr);
                lp2
            }
            Filter2Mode::BandPass12dB => {
                let q = 0.8 + res * 9.0;
                let (_, bp, _) = self.svf1.tick(input, cut, q, sr);
                bp * 1.4
            }
            Filter2Mode::HighPass12dB => {
                let q = 0.7 + res * 7.0;
                let (_, _, hp) = self.svf1.tick(input, cut, q, sr);
                hp
            }
            Filter2Mode::FormantVowel => {
                // 3-formant vocal tract table: A, E, I, O, U (F1, F2, F3 in Hz)
                const VOWELS: [[f32; 3]; 5] = [
                    [730.0, 1090.0, 2440.0], // A
                    [530.0, 1840.0, 2480.0], // E
                    [270.0, 2290.0, 3010.0], // I
                    [570.0, 840.0, 2410.0],  // O
                    [300.0, 870.0, 2240.0],  // U
                ];
                let vm = (params.vowel_morph + mod_val * 2.0).clamp(0.0, 4.0);
                let idx = (vm.floor() as usize).min(3);
                let frac = vm - idx as f32;
                let shift = (cut / 1400.0).clamp(0.5, 2.2);
                let f1 = (VOWELS[idx][0] + (VOWELS[idx + 1][0] - VOWELS[idx][0]) * frac) * shift;
                let f2 = (VOWELS[idx][1] + (VOWELS[idx + 1][1] - VOWELS[idx][1]) * frac) * shift;
                let f3 = (VOWELS[idx][2] + (VOWELS[idx + 1][2] - VOWELS[idx][2]) * frac) * shift;
                let q = 3.5 + res * 9.0;
                let (_, b1, _) = self.svf1.tick(input, f1, q, sr);
                let (_, b2, _) = self.svf2.tick(input, f2, q * 1.1, sr);
                let (_, b3, _) = self.svf3.tick(input, f3, q * 1.2, sr);
                ((b1 * 1.1 + b2 * 0.9 + b3 * 0.55) * 1.3).tanh()
            }
            Filter2Mode::CombPositive | Filter2Mode::CombNegative => {
                let cap = self.comb_buf.len();
                let delay = (sr / cut.clamp(45.0, 4000.0)) as usize;
                let d = delay.clamp(8, cap - 1);
                let read_idx = (self.comb_pos + cap - d) % cap;
                let delayed = self.comb_buf[read_idx];
                let sign = if params.mode == Filter2Mode::CombPositive {
                    1.0
                } else {
                    -1.0
                };
                let fb = (0.15 + res * 0.80) * sign;
                let out = (input + delayed * fb).tanh();
                self.comb_buf[self.comb_pos] = out;
                self.comb_pos = (self.comb_pos + 1) % cap;
                out
            }
            Filter2Mode::Ms20Screamer => {
                // Korg MS-20 style Sallen-Key diode-clipped resonance feedback
                let k = res * 2.15;
                let fb = (self.svf1.bp * k).tanh();
                let driven = (input * (1.0 + res * 0.9) - fb * 0.65).tanh();
                let (lp, _, _) = self.svf1.tick(driven, cut, 1.0 + res * 10.0, sr);
                let (lp2, _, _) = self.svf2.tick(lp, cut * 1.05, 1.2, sr);
                (lp2 * 1.35).tanh()
            }
        };

        let mix = params.mix.clamp(0.0, 1.0);
        let _ = TAU;
        input * (1.0 - mix) + wet * mix
    }
}

#[derive(Debug, Clone)]
pub struct Filter2StereoState {
    left: Filter2Channel,
    right: Filter2Channel,
}

impl Default for Filter2StereoState {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter2StereoState {
    pub fn new() -> Self {
        Self {
            left: Filter2Channel::new(),
            right: Filter2Channel::new(),
        }
    }

    #[inline]
    pub fn process_stereo(
        &mut self,
        l: f32,
        r: f32,
        params: &Filter2Params,
        mod_val: f32,
        sr: f32,
    ) -> (f32, f32) {
        if params.mode == Filter2Mode::Off || params.mix <= 0.001 {
            return (l, r);
        }
        (
            self.left.process(l, params, mod_val, sr),
            self.right.process(r, params, mod_val, sr),
        )
    }
}
