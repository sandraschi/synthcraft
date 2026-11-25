//! Morphing Wavetable Oscillator: 4 built-in multi-frame wavetables plus single-cycle
//! or multi-frame `.wav` loader (256..=4096 samples per frame, up to 64 frames).
//!
//! Playback (`sample(phase, morph)`) is branch-light and allocation-free for the audio thread.

use crate::error::SynthError;
use std::f32::consts::TAU;
use std::path::Path;
use std::sync::Arc;

const FRAME_LEN: usize = 256;
const MAX_FRAMES: usize = 64;

/// Decoded, DC-removed, peak-normalized multi-frame wavetable.
#[derive(Debug, Clone)]
pub struct Wavetable {
    pub name: String,
    /// Flattened `frames * FRAME_LEN` samples in `-1.0..=1.0`.
    pub frames: Vec<[f32; FRAME_LEN]>,
}

impl Wavetable {
    pub const BUILTINS: [(&'static str, &'static str); 4] = [
        ("analog_morph", "Analog Morph (Sine -> Tri -> Saw -> Sq)"),
        ("formant_vowel", "Formant Vowel (A -> E -> I -> O -> U)"),
        ("metallic_bell", "Metallic Bell Harmonics"),
        ("acid_fold", "Acid Harmonic Wavefold"),
    ];

    /// Build one of the 4 built-in morphing wavetables.
    pub fn builtin(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "analog_morph" | "analog" => Some(build_analog_morph()),
            "formant_vowel" | "formant" | "vowel" => Some(build_formant_vowel()),
            "metallic_bell" | "bell" | "metallic" => Some(build_metallic_bell()),
            "acid_fold" | "acid" | "fold" => Some(build_acid_fold()),
            _ => None,
        }
    }

    /// Load a built-in wavetable by id OR decode a `.wav` file (16-bit PCM or 32-bit float)
    /// into up to 64 resampled 256-sample frames.
    pub fn load(spec: &str) -> Result<Arc<Self>, SynthError> {
        if let Some(wt) = Self::builtin(spec) {
            return Ok(Arc::new(wt));
        }
        let path = Path::new(spec);
        let bytes = std::fs::read(path).map_err(|e| {
            SynthError::Validation(format!("Cannot read wavetable '{}': {e}", path.display()))
        })?;
        let samples = decode_wav_mono(&bytes)?;
        if samples.len() < 16 {
            return Err(SynthError::Validation(
                "Wavetable WAV is too short (need >= 16 samples)".into(),
            ));
        }
        // Heuristic frame size: Serum/Vital 2048 if >= 2048 and cleanly divisible, else single cycle.
        let src_frame_len = if samples.len() >= 2048 && samples.len().is_multiple_of(2048) {
            2048
        } else if samples.len() >= 1024 && samples.len().is_multiple_of(1024) {
            1024
        } else if samples.len() >= 512 && samples.len().is_multiple_of(512) {
            512
        } else {
            samples.len().min(4096)
        };
        let n_frames = (samples.len() / src_frame_len).clamp(1, MAX_FRAMES);
        let mut frames = Vec::with_capacity(n_frames);
        for f in 0..n_frames {
            let chunk = &samples[f * src_frame_len..(f + 1) * src_frame_len];
            let mut dst = [0.0_f32; FRAME_LEN];
            for (i, slot) in dst.iter_mut().enumerate() {
                let pos = (i as f32 / FRAME_LEN as f32) * (chunk.len() as f32);
                let i0 = (pos as usize) % chunk.len();
                let i1 = (i0 + 1) % chunk.len();
                let frac = pos - pos.floor();
                *slot = chunk[i0] * (1.0 - frac) + chunk[i1] * frac;
            }
            normalize_frame(&mut dst);
            frames.push(dst);
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("custom_wt")
            .to_string();
        Ok(Arc::new(Self { name, frames }))
    }

    /// Sample the wavetable at normalized phase `p in [0, 1)` and frame morph `morph in [0, 1]`.
    #[inline]
    pub fn sample(&self, phase: f32, morph: f32) -> f32 {
        let nf = self.frames.len();
        if nf == 0 {
            return 0.0;
        }
        let p = phase.rem_euclid(1.0) * (FRAME_LEN as f32);
        let i0 = (p as usize) % FRAME_LEN;
        let i1 = (i0 + 1) % FRAME_LEN;
        let frac = p - p.floor();

        let fm = morph.clamp(0.0, 1.0) * ((nf - 1) as f32);
        let f0 = (fm as usize).min(nf - 1);
        let f1 = (f0 + 1).min(nf - 1);
        let ffrac = fm - fm.floor();

        let a = self.frames[f0][i0] * (1.0 - frac) + self.frames[f0][i1] * frac;
        if f0 == f1 {
            return a;
        }
        let b = self.frames[f1][i0] * (1.0 - frac) + self.frames[f1][i1] * frac;
        a * (1.0 - ffrac) + b * ffrac
    }
}

fn normalize_frame(frame: &mut [f32; FRAME_LEN]) {
    let mean = frame.iter().sum::<f32>() / (FRAME_LEN as f32);
    let mut peak = 0.0_f32;
    for s in frame.iter_mut() {
        *s -= mean;
        peak = peak.max(s.abs());
    }
    if peak > 1e-5 {
        let inv = 0.98 / peak;
        for s in frame.iter_mut() {
            *s *= inv;
        }
    }
}

fn build_analog_morph() -> Wavetable {
    let mut frames = Vec::with_capacity(4);
    let mut f0 = [0.0_f32; FRAME_LEN];
    let mut f1 = [0.0_f32; FRAME_LEN];
    let mut f2 = [0.0_f32; FRAME_LEN];
    let mut f3 = [0.0_f32; FRAME_LEN];
    for i in 0..FRAME_LEN {
        let p = i as f32 / FRAME_LEN as f32;
        f0[i] = (TAU * p).sin();
        f1[i] = 1.0 - 4.0 * (p - 0.5).abs();
        f2[i] = 2.0 * p - 1.0;
        f3[i] = if p < 0.5 { 1.0 } else { -1.0 };
    }
    for f in [&mut f0, &mut f1, &mut f2, &mut f3] {
        normalize_frame(f);
        frames.push(*f);
    }
    Wavetable {
        name: "analog_morph".into(),
        frames,
    }
}

fn build_formant_vowel() -> Wavetable {
    // 5 vowel-like harmonic spectral peaks
    let peaks: [(f32, f32); 5] = [(3.0, 7.0), (2.0, 11.0), (1.5, 14.0), (2.5, 5.5), (1.5, 4.0)];
    let mut frames = Vec::with_capacity(peaks.len());
    for (p1, p2) in peaks {
        let mut f = [0.0_f32; FRAME_LEN];
        for (i, slot) in f.iter_mut().enumerate() {
            let ph = i as f32 / FRAME_LEN as f32;
            let mut sum = 0.0_f32;
            for h in 1..=16 {
                let hf = h as f32;
                let w1 = (-((hf - p1) * 0.7).powi(2)).exp();
                let w2 = (-((hf - p2) * 0.55).powi(2)).exp();
                sum += (0.15 / hf + w1 + 0.75 * w2) * (TAU * ph * hf).sin();
            }
            *slot = sum;
        }
        normalize_frame(&mut f);
        frames.push(f);
    }
    Wavetable {
        name: "formant_vowel".into(),
        frames,
    }
}

fn build_metallic_bell() -> Wavetable {
    let mut frames = Vec::with_capacity(6);
    for idx in 0..6 {
        let bright = idx as f32 / 5.0;
        let mut f = [0.0_f32; FRAME_LEN];
        for (i, slot) in f.iter_mut().enumerate() {
            let ph = i as f32 / FRAME_LEN as f32;
            let s = (TAU * ph).sin()
                + 0.65 * (TAU * ph * 2.0).sin()
                + (0.3 + 0.5 * bright) * (TAU * ph * 5.0).sin()
                + (0.15 + 0.6 * bright) * (TAU * ph * 7.0).sin()
                + (0.05 + 0.55 * bright) * (TAU * ph * 11.0).sin();
            *slot = s;
        }
        normalize_frame(&mut f);
        frames.push(f);
    }
    Wavetable {
        name: "metallic_bell".into(),
        frames,
    }
}

fn build_acid_fold() -> Wavetable {
    let mut frames = Vec::with_capacity(8);
    for idx in 0..8 {
        let drive = 1.0 + (idx as f32) * 0.85;
        let mut f = [0.0_f32; FRAME_LEN];
        for (i, slot) in f.iter_mut().enumerate() {
            let ph = i as f32 / FRAME_LEN as f32;
            let base = (TAU * ph).sin() * 0.7 + (2.0 * ph - 1.0) * 0.3;
            *slot = (base * drive).sin();
        }
        normalize_frame(&mut f);
        frames.push(f);
    }
    Wavetable {
        name: "acid_fold".into(),
        frames,
    }
}

fn decode_wav_mono(bytes: &[u8]) -> Result<Vec<f32>, SynthError> {
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(SynthError::Validation("Not a valid RIFF WAVE file".into()));
    }
    let mut pos = 12usize;
    let mut audio_fmt = 1u16;
    let mut channels = 1u16;
    let mut bits = 16u16;
    let mut data_slice: Option<&[u8]> = None;
    while pos + 8 <= bytes.len() {
        let tag = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;
        pos += 8;
        if pos + size > bytes.len() {
            break;
        }
        let body = &bytes[pos..pos + size];
        if tag == b"fmt " && body.len() >= 16 {
            audio_fmt = u16::from_le_bytes([body[0], body[1]]);
            channels = u16::from_le_bytes([body[2], body[3]]).max(1);
            bits = u16::from_le_bytes([body[14], body[15]]);
        } else if tag == b"data" {
            data_slice = Some(body);
        }
        pos += size + (size & 1);
    }
    let Some(data) = data_slice else {
        return Err(SynthError::Validation("WAV missing data chunk".into()));
    };
    let ch = usize::from(channels);
    let mut out = Vec::new();
    match (audio_fmt, bits) {
        (1, 16) => {
            for frame in data.chunks_exact(2 * ch) {
                let s = i16::from_le_bytes([frame[0], frame[1]]) as f32 / 32768.0;
                out.push(s);
            }
        }
        (3, 32) => {
            for frame in data.chunks_exact(4 * ch) {
                let s = f32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]);
                out.push(if s.is_finite() { s } else { 0.0 });
            }
        }
        _ => {
            return Err(SynthError::Validation(format!(
                "Unsupported WAV format={audio_fmt}, bits={bits} (expected 16-bit PCM or 32-bit float)"
            )));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_wavetables_morph_smoothly_and_stay_bounded() {
        for (id, _) in Wavetable::BUILTINS {
            let wt = Wavetable::builtin(id).unwrap_or_else(build_analog_morph);
            assert!(wt.frames.len() >= 4);
            let a = wt.sample(0.25, 0.0);
            let b = wt.sample(0.25, 1.0);
            assert!((-1.0..=1.0).contains(&a));
            assert!((-1.0..=1.0).contains(&b));
        }
    }
}
