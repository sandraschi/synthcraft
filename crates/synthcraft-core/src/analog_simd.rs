//! One-Shot #4 & #5:
//! - Golden-Ear Analog Calibration: Per-voice component tolerance & slow thermal drift
//!   (`VintageSlopParams` / `VoiceTolerance`, inspired by Prophet-5 Rev4 "Vintage" & u-he Diva)
//!   plus bass-preserving ZDF resonance compensation.
//! - Compact Pure-Rust Neural DDSP Formant-Velocity MLP (`ddsp_velocity_harmonic_weights`)
//! - 4-Wide Auto-Vectorizable SIMD Stereo Voice Accumulator (`Simd4StereoMixer`, `#![forbid(unsafe_code)]`).
#![forbid(unsafe_code)]

use crate::neural_ddsp::DDSP_HARMONICS;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// Prophet-5 Rev4 / u-he Diva-style per-voice analog component tolerance & thermal drift.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VintageSlopParams {
    /// 0.0 = pristine digital precision, 1.0 = warm 1978 analog voice-to-voice tolerance.
    pub amount: f32,
    /// Bass-preserving resonance compensation on the 4-pole ladder filter (0.0 ..= 1.0).
    pub bass_comp: f32,
}

impl Default for VintageSlopParams {
    fn default() -> Self {
        Self {
            amount: 0.25,
            bass_comp: 0.45,
        }
    }
}

/// Per-voice fixed resistor/capacitor tolerance + slow thermal drift state.
#[derive(Debug, Clone, Copy)]
pub struct VoiceTolerance {
    pub pitch_cents_base: f32,
    pub cutoff_mult_base: f32,
    pub env_rate_mult: f32,
    drift_phase: f32,
    drift_hz: f32,
}

impl VoiceTolerance {
    /// Deterministic hardware-like component spread for voice index `0..MAX_VOICES`.
    pub fn for_voice(voice_idx: usize) -> Self {
        let mut h = (voice_idx as u32).wrapping_mul(0x9E37_79B9) ^ 0xA511_E9B3;
        let mut next_signed = || -> f32 {
            h ^= h << 13;
            h ^= h >> 17;
            h ^= h << 5;
            ((h as f32) / (u32::MAX as f32)) * 2.0 - 1.0
        };
        Self {
            pitch_cents_base: next_signed() * 9.5, // up to +/- 9.5 cents at slop=1.0
            cutoff_mult_base: 1.0 + next_signed() * 0.085, // +/- 8.5% capacitor tolerance
            env_rate_mult: 1.0 + next_signed() * 0.075, // +/- 7.5% envelope timing spread
            drift_phase: (next_signed() * 0.5 + 0.5).clamp(0.0, 0.999),
            drift_hz: 0.07 + (voice_idx as f32) * 0.019,
        }
    }

    /// Advance slow thermal oscillator drift and return `(pitch_ratio, cutoff_ratio)`.
    #[inline]
    pub fn tick(&mut self, slop: f32, sr: f32) -> (f32, f32) {
        let amt = slop.clamp(0.0, 1.0);
        if amt <= 0.0001 {
            return (1.0, 1.0);
        }
        self.drift_phase += self.drift_hz / sr.max(8000.0);
        if self.drift_phase >= 1.0 {
            self.drift_phase -= 1.0;
        }
        let thermal = (TAU * self.drift_phase).sin();
        let cents = (self.pitch_cents_base + thermal * 3.2) * amt;
        let pitch_ratio = 2.0_f32.powf(cents / 1200.0);
        let cutoff_ratio = 1.0 + (self.cutoff_mult_base - 1.0 + thermal * 0.025) * amt;
        (pitch_ratio, cutoff_ratio.clamp(0.75, 1.30))
    }
}

/// Bass-preserving input gain boost for 4-pole resonant ladder filters (prevents sub-bass drop
/// as resonance `q` climbs from `0.7` to `11.0`).
#[inline]
pub fn zdf_bass_compensation_gain(resonance_q: f32, bass_comp: f32) -> f32 {
    let q_norm = ((resonance_q - 0.7) / 10.3).clamp(0.0, 1.0);
    1.0 + q_norm * bass_comp.clamp(0.0, 1.0) * 0.65
}

/// Compact pure-Rust 2-layer MLP harmonic shaper for Neural DDSP timbre transfer.
/// Maps `(pitch_norm, velocity, bow_pressure)` -> 32 dynamic harmonic multipliers in `[0.0, 1.5]`.
pub fn ddsp_velocity_harmonic_weights(
    midi_note: f32,
    velocity: f32,
    bow_pressure: f32,
) -> [f32; DDSP_HARMONICS] {
    let pitch_n = ((midi_note - 48.0) / 36.0).clamp(-1.0, 1.5);
    let vel = velocity.clamp(0.05, 1.0);
    let bow = bow_pressure.clamp(0.0, 1.0);

    // Compact 4-neuron hidden layer (tanh activations) modeling acoustic bridge & formant coupling
    let h0 = (0.85 * vel + 0.60 * bow - 0.35 * pitch_n).tanh();
    let h1 = (1.10 * bow - 0.50 * vel + 0.45 * pitch_n).tanh();
    let h2 = (-0.65 * pitch_n + 0.75 * vel + 0.20).tanh();
    let h3 = (0.90 * vel * bow - 0.25 * pitch_n).tanh();

    let mut weights = [1.0_f32; DDSP_HARMONICS];
    for (i, w) in weights.iter_mut().enumerate() {
        let k = (i as f32) / (DDSP_HARMONICS as f32);
        let low_body = (1.0 - k).powf(1.5) * (0.35 * h2 + 0.20 * h0);
        let mid_formant = (-((k - 0.28) / 0.16).powi(2)).exp() * (0.42 * h3 + 0.22 * h0);
        let upper_air = k.powf(1.2) * (0.50 * h1 + 0.30 * h3);
        *w = (0.78 + low_body + mid_formant + upper_air).clamp(0.05, 1.50);
    }
    weights
}

/// 4-wide auto-vectorizable stereo voice accumulator (compiles cleanly to SSE/AVX2 in pure safe Rust).
#[derive(Debug, Clone, Copy, Default)]
pub struct Simd4StereoMixer {
    pub lane_l: [f32; 4],
    pub lane_r: [f32; 4],
}

impl Simd4StereoMixer {
    #[inline]
    pub fn reset(&mut self) {
        self.lane_l = [0.0; 4];
        self.lane_r = [0.0; 4];
    }

    /// Accumulate a batch of 4 mono voice samples with 4 stereo pan gains (`-1.0..=1.0`).
    #[inline]
    pub fn add_quad(&mut self, samples: [f32; 4], pans: [f32; 4], gains: [f32; 4]) {
        for i in 0..4 {
            let s = samples[i] * gains[i];
            let p = pans[i].clamp(-1.0, 1.0);
            let gl = (0.5 * (1.0 - p)).sqrt();
            let gr = (0.5 * (1.0 + p)).sqrt();
            self.lane_l[i] += s * gl;
            self.lane_r[i] += s * gr;
        }
    }

    /// Horizontal reduction of the 4 SIMD lanes with soft analog saturation.
    #[inline]
    pub fn reduce_stereo(&self, master_gain: f32) -> (f32, f32) {
        let l = (self.lane_l[0] + self.lane_l[1] + self.lane_l[2] + self.lane_l[3]) * master_gain;
        let r = (self.lane_r[0] + self.lane_r[1] + self.lane_r[2] + self.lane_r[3]) * master_gain;
        (l.tanh(), r.tanh())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vintage_slop_is_zero_when_disabled_and_bounded_when_active() {
        let mut v0 = VoiceTolerance::for_voice(0);
        let mut v1 = VoiceTolerance::for_voice(1);
        assert_eq!(v0.tick(0.0, 48_000.0), (1.0, 1.0));
        let (p0, c0) = v0.tick(1.0, 48_000.0);
        let (p1, c1) = v1.tick(1.0, 48_000.0);
        assert!((0.98..=1.02).contains(&p0));
        assert!((0.75..=1.30).contains(&c0));
        assert!((p0 - p1).abs() > 1e-6 || (c0 - c1).abs() > 1e-6);
    }

    #[test]
    fn ddsp_mlp_and_simd4_mixer_produce_bounded_results() {
        let w_soft = ddsp_velocity_harmonic_weights(60.0, 0.2, 0.2);
        let w_loud = ddsp_velocity_harmonic_weights(60.0, 0.95, 0.9);
        assert!(w_loud[10] > w_soft[10]);

        let mut mix = Simd4StereoMixer::default();
        mix.add_quad([0.5, -0.25, 0.4, 0.1], [-0.8, 0.8, 0.0, 0.2], [1.0; 4]);
        let (l, r) = mix.reduce_stereo(0.8);
        assert!(l.is_finite() && r.is_finite());
        assert!((-1.0..=1.0).contains(&l) && (-1.0..=1.0).contains(&r));
        assert!(zdf_bass_compensation_gain(8.0, 0.5) > 1.1);
    }
}
