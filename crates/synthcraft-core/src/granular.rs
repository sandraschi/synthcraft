//! Phase 2 Next-Gen DSP:
//! - Sample-Buffer / Live-Capture Granular Cloud Engine (`GranularParams` / `GranularRuntime`)
//!   with 24 concurrent Hann-windowed grains, scrub playhead, spray jitter, pitch shift, stereo scatter, and Freeze.
//! - 64-Partial Spectral Additive Resynthesis Engine (`SpectralParams` / `SpectralRuntime`)
//!   with DFT harmonic capture from any audio buffer and real-time harmonic sculpting (formant shift,
//!   odd/even tilt, harmonic warp, and spectral mix).

use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};

pub const MAX_GRAINS: usize = 24;
pub const GRANULAR_RING_FRAMES: usize = 96_000; // 2.0 seconds @ 48 kHz
pub const SPECTRAL_PARTIALS: usize = 64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GranularParams {
    pub enabled: bool,
    pub mix: f32,             // 0.0 ..= 1.0
    pub grain_size_ms: f32,   // 10.0 ..= 400.0
    pub density: u32,         // 1 ..= 24 concurrent grains
    pub position: f32,        // 0.0 ..= 1.0 buffer scrub playhead
    pub spray: f32,           // 0.0 ..= 1.0 random position scatter
    pub pitch_semitones: f32, // -24.0 ..= +24.0
    pub stereo_spread: f32,   // 0.0 ..= 1.0
    pub freeze: bool,         // freeze live capture buffer
}

impl Default for GranularParams {
    fn default() -> Self {
        Self {
            enabled: false,
            mix: 0.45,
            grain_size_ms: 90.0,
            density: 10,
            position: 0.25,
            spray: 0.18,
            pitch_semitones: 0.0,
            stereo_spread: 0.75,
            freeze: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Grain {
    active: bool,
    read_pos: f32,
    rate: f32,
    age: f32,
    length: f32,
    pan: f32,
}

#[derive(Debug, Clone)]
pub struct GranularRuntime {
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    write_pos: usize,
    grains: [Grain; MAX_GRAINS],
    spawn_accum: f32,
    rng: u32,
}

impl Default for GranularRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl GranularRuntime {
    pub fn new() -> Self {
        Self {
            buf_l: vec![0.0; GRANULAR_RING_FRAMES],
            buf_r: vec![0.0; GRANULAR_RING_FRAMES],
            write_pos: 0,
            grains: [Grain::default(); MAX_GRAINS],
            spawn_accum: 0.0,
            rng: 0x67A1_5EED,
        }
    }

    #[inline]
    fn rand01(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x as f32) / (u32::MAX as f32)
    }

    /// Load mono `.wav` samples directly into the 24-grain stereo ring buffer for real-audio granular synthesis.
    pub fn load_wav_samples(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        let cap = self.buf_l.len();
        let copy_len = samples.len().min(cap);
        for i in 0..cap {
            let s = samples[i % copy_len];
            self.buf_l[i] = s;
            self.buf_r[i] = samples[(i + 7) % copy_len];
        }
        self.write_pos = copy_len % cap;
    }

    /// Process one stereo frame through the live-capture 24-grain cloud. Allocation-free.
    #[inline]
    pub fn tick_stereo(
        &mut self,
        in_l: f32,
        in_r: f32,
        params: &GranularParams,
        sr: f32,
    ) -> (f32, f32) {
        let cap = self.buf_l.len();
        if !params.freeze {
            self.buf_l[self.write_pos] = in_l;
            self.buf_r[self.write_pos] = in_r;
            self.write_pos = (self.write_pos + 1) % cap;
        }
        if !params.enabled || params.mix <= 0.001 {
            return (in_l, in_r);
        }

        let grain_len = (params.grain_size_ms.clamp(10.0, 400.0) * 0.001 * sr).max(64.0);
        let density = params.density.clamp(1, MAX_GRAINS as u32) as f32;
        let spawn_interval = (grain_len / density).max(32.0);

        self.spawn_accum += 1.0;
        if self.spawn_accum >= spawn_interval {
            self.spawn_accum -= spawn_interval;
            let r_pos = self.rand01();
            let r_pan = self.rand01();
            if let Some(g) = self.grains.iter_mut().find(|g| !g.active) {
                let lookback = (params.position.clamp(0.0, 0.95) * (cap as f32 * 0.85))
                    + r_pos * params.spray.clamp(0.0, 1.0) * (cap as f32 * 0.25);
                let start = (self.write_pos as f32 - lookback).rem_euclid(cap as f32);
                g.active = true;
                g.read_pos = start;
                g.rate = 2.0_f32.powf(params.pitch_semitones.clamp(-24.0, 24.0) / 12.0);
                g.age = 0.0;
                g.length = grain_len;
                g.pan = (r_pan * 2.0 - 1.0) * params.stereo_spread.clamp(0.0, 1.0);
            }
        }

        let mut wet_l = 0.0_f32;
        let mut wet_r = 0.0_f32;
        let mut active_cnt = 0.0_f32;

        for g in &mut self.grains {
            if !g.active {
                continue;
            }
            let u = g.age / g.length;
            if u >= 1.0 {
                g.active = false;
                continue;
            }
            // Raised-cosine Hann window (click-free grains)
            let win = 0.5 * (1.0 - (TAU * u).cos());
            let idx0 = (g.read_pos as usize) % cap;
            let idx1 = (idx0 + 1) % cap;
            let frac = g.read_pos.fract();
            let sl = self.buf_l[idx0] + (self.buf_l[idx1] - self.buf_l[idx0]) * frac;
            let sr_s = self.buf_r[idx0] + (self.buf_r[idx1] - self.buf_r[idx0]) * frac;

            let gl = ((1.0 - g.pan) * 0.5).sqrt();
            let gr = ((1.0 + g.pan) * 0.5).sqrt();
            wet_l += sl * win * gl;
            wet_r += sr_s * win * gr;
            active_cnt += 1.0;

            g.read_pos = (g.read_pos + g.rate).rem_euclid(cap as f32);
            g.age += 1.0;
        }

        let norm = 1.0 / active_cnt.max(1.0).sqrt();
        let mix = params.mix.clamp(0.0, 1.0);
        (
            in_l * (1.0 - mix) + wet_l * norm * mix,
            in_r * (1.0 - mix) + wet_r * norm * mix,
        )
    }
}

// ---------------------------------------------------------------------------
// 64-Partial Spectral Additive Resynthesis Engine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpectralParams {
    pub enabled: bool,
    pub mix: f32,
    pub formant_shift_st: f32, // -12.0 ..= +12.0 semitones
    pub odd_even_tilt: f32,    // -1.0 (odd/clarinet) .. 0.0 (balanced) .. +1.0 (even/octave)
    pub harmonic_warp: f32,    // -0.5 ..= +0.5 (inharmonic bell/glass stretch)
    pub partials: Vec<f32>,    // 64 partial amplitudes in [0.0, 1.0]
}

impl Default for SpectralParams {
    fn default() -> Self {
        let mut partials = Vec::with_capacity(SPECTRAL_PARTIALS);
        for k in 1..=SPECTRAL_PARTIALS {
            // Warm harmonic saw/organ decay profile
            partials.push((1.0 / (k as f32).powf(0.85)).clamp(0.0, 1.0));
        }
        Self {
            enabled: false,
            mix: 0.35,
            formant_shift_st: 0.0,
            odd_even_tilt: 0.0,
            harmonic_warp: 0.0,
            partials,
        }
    }
}

impl SpectralParams {
    pub fn ensure_len(&mut self) {
        if self.partials.len() != SPECTRAL_PARTIALS {
            self.partials.resize(SPECTRAL_PARTIALS, 0.0);
        }
    }

    /// Analyze a mono audio buffer via 64-bin DFT and populate `self.partials` with normalized amplitudes.
    pub fn capture_from_buffer(&mut self, samples: &[f32]) {
        self.ensure_len();
        if samples.len() < 128 {
            return;
        }
        let n = samples.len().min(1024);
        let mut mags = [0.0_f32; SPECTRAL_PARTIALS];
        let mut max_mag = 1e-6_f32;
        for (k, slot) in mags.iter_mut().enumerate() {
            let bin = (k + 1) as f32;
            let mut re = 0.0_f32;
            let mut im = 0.0_f32;
            for (i, &s) in samples.iter().take(n).enumerate() {
                let win = 0.5 * (1.0 - (TAU * i as f32 / n as f32).cos());
                let ang = TAU * bin * (i as f32) / (n as f32);
                re += s * win * ang.cos();
                im -= s * win * ang.sin();
            }
            let mag = (re * re + im * im).sqrt();
            *slot = mag;
            if mag > max_mag {
                max_mag = mag;
            }
        }
        for (i, &m) in mags.iter().enumerate() {
            self.partials[i] = (m / max_mag).powf(0.65).clamp(0.0, 1.0);
        }
    }

    pub fn load_preset(&mut self, name: &str) {
        self.ensure_len();
        for k in 0..SPECTRAL_PARTIALS {
            let h = (k + 1) as f32;
            self.partials[k] = match name {
                "vocal_choir" => {
                    let f1 = (-((h - 4.0) / 2.2).powi(2)).exp();
                    let f2 = (-((h - 11.0) / 3.0).powi(2)).exp() * 0.85;
                    let f3 = (-((h - 22.0) / 4.0).powi(2)).exp() * 0.45;
                    (f1 + f2 + f3).clamp(0.0, 1.0)
                }
                "glass_bell" => {
                    if matches!(k + 1, 1 | 2 | 3 | 5 | 7 | 11 | 15 | 21 | 29) {
                        (1.0 / h.sqrt()).clamp(0.0, 1.0)
                    } else {
                        0.02
                    }
                }
                "cathedral_organ" => {
                    if matches!(k + 1, 1 | 2 | 3 | 4 | 6 | 8 | 12 | 16) {
                        0.85
                    } else {
                        0.04
                    }
                }
                _ => (1.0 / h.powf(0.85)).clamp(0.0, 1.0),
            };
        }
        let _ = PI;
    }
}

#[derive(Debug, Clone)]
pub struct SpectralRuntime {
    phases: [f32; SPECTRAL_PARTIALS],
}

impl Default for SpectralRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralRuntime {
    pub fn new() -> Self {
        Self {
            phases: [0.0; SPECTRAL_PARTIALS],
        }
    }

    /// Synthesize one stereo sample from the 64-partial additive bank at `base_freq_hz`. Allocation-free.
    #[inline]
    pub fn tick_stereo(
        &mut self,
        base_freq_hz: f32,
        params: &SpectralParams,
        sr: f32,
    ) -> (f32, f32) {
        if !params.enabled || params.mix <= 0.001 || params.partials.is_empty() {
            return (0.0, 0.0);
        }
        let formant_mul = 2.0_f32.powf(params.formant_shift_st.clamp(-12.0, 12.0) / 12.0);
        let tilt = params.odd_even_tilt.clamp(-1.0, 1.0);
        let warp = params.harmonic_warp.clamp(-0.5, 0.5);
        let nyquist = sr * 0.44;
        let dt = 1.0 / sr.max(8000.0);

        let mut l = 0.0_f32;
        let mut r = 0.0_f32;
        let n = params.partials.len().min(SPECTRAL_PARTIALS);

        for k in 0..n {
            let amp_raw = params.partials[k];
            if amp_raw <= 0.002 {
                continue;
            }
            let h = (k + 1) as f32;
            let parity_gain = if (k + 1).is_multiple_of(2) {
                (1.0 + tilt).clamp(0.0, 1.8)
            } else {
                (1.0 - tilt).clamp(0.0, 1.8)
            };
            let warped_h = h.powf(1.0 + warp * 0.45);
            let freq = base_freq_hz * warped_h * formant_mul;
            if freq >= nyquist {
                continue;
            }
            self.phases[k] = (self.phases[k] + freq * dt).fract();
            let s = (TAU * self.phases[k]).sin() * amp_raw * parity_gain;
            // Subtle odd/even stereo spread
            if k.is_multiple_of(2) {
                l += s * 0.85;
                r += s * 0.55;
            } else {
                l += s * 0.55;
                r += s * 0.85;
            }
        }

        let norm = 0.14 * params.mix.clamp(0.0, 1.0);
        (l * norm, r * norm)
    }
}
