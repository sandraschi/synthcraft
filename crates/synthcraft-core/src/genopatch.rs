//! Phase 3 Next-Gen DSP: Multi-Core "Genopatch" Audio-to-Patch Evolutionary Matcher (Synplant 2 style).
//!
//! Uses `std::thread::scope` across available CPU cores (24 threads on Goliath's Ryzen 9 5900X)
//! to breed, mutate, synthesize, and score candidate `SynthPatch` genomes against any target
//! audio buffer (`.wav` file or built-in target sound), returning the top 6 editable DNA branches.

use crate::model::{
    analyze_patch_spectrum, synthesize_audio_buffer, FilterMode, SynthPatch, WaveformKind,
};
use crate::rompler::ExtremeDspMode;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFingerprint {
    pub bands: [f32; 16],
    pub centroid_norm: f32,
    pub attack_ratio: f32,
    pub decay_ratio: f32,
    pub zero_crossing_rate: f32,
    pub rms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenopatchCandidate {
    pub branch_id: usize,
    pub label: String,
    pub similarity_pct: f32,
    pub spectral_distance: f32,
    pub patch: SynthPatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenopatchReport {
    pub target_name: String,
    pub generations: usize,
    pub population_size: usize,
    pub threads_used: usize,
    pub total_evaluations: usize,
    pub elapsed_ms: u64,
    pub candidates: Vec<GenopatchCandidate>,
}

/// Extract a 16-band log-spectral + temporal envelope + noisiness fingerprint from a mono buffer.
pub fn extract_fingerprint(samples: &[f32], sample_rate_hz: u32) -> AudioFingerprint {
    let n = samples.len().clamp(64, 2048);
    let mut bands = [0.0_f32; 16];
    let mut sum_sq = 0.0_f32;
    let mut zcr = 0.0_f32;

    for i in 0..samples.len() {
        let s = samples[i];
        sum_sq += s * s;
        if i > 0 && ((samples[i - 1] >= 0.0) != (s >= 0.0)) {
            zcr += 1.0;
        }
    }
    let rms = (sum_sq / samples.len().max(1) as f32).sqrt();
    let zero_crossing_rate = (zcr / samples.len().max(1) as f32).clamp(0.0, 1.0);

    // Temporal envelope: first 15% (attack) vs last 35% (decay/sustain)
    let split_a = (samples.len() * 15 / 100).max(1);
    let split_d = (samples.len() * 65 / 100).min(samples.len().saturating_sub(1));
    let rms_a = (samples[..split_a].iter().map(|x| x * x).sum::<f32>() / split_a as f32).sqrt();
    let rms_d = (samples[split_d..].iter().map(|x| x * x).sum::<f32>()
        / (samples.len() - split_d).max(1) as f32)
        .sqrt();
    let denom = rms.max(1e-5);
    let attack_ratio = (rms_a / denom).clamp(0.0, 3.0);
    let decay_ratio = (rms_d / denom).clamp(0.0, 3.0);

    // 16 log-spaced DFT bins from 65 Hz to 8000 Hz
    let sr = sample_rate_hz.max(8000) as f32;
    let mut max_b = 1e-6_f32;
    let mut weighted_sum = 0.0_f32;
    let mut mag_sum = 1e-6_f32;
    for (bi, slot) in bands.iter_mut().enumerate() {
        let frac = bi as f32 / 15.0;
        let freq = 65.0 * (8000.0_f32 / 65.0).powf(frac);
        let mut re = 0.0_f32;
        let mut im = 0.0_f32;
        let stride = (n / 256).max(1);
        let mut cnt = 0usize;
        let mut i = 0usize;
        while i < n && i < samples.len() {
            let ang = TAU * freq * (i as f32) / sr;
            let s = samples[i];
            re += s * ang.cos();
            im -= s * ang.sin();
            i += stride;
            cnt += 1;
        }
        let mag = (re * re + im * im).sqrt() / cnt.max(1) as f32;
        *slot = mag;
        if mag > max_b {
            max_b = mag;
        }
        weighted_sum += frac * mag;
        mag_sum += mag;
    }
    for b in &mut bands {
        *b = (*b / max_b).clamp(0.0, 1.0);
    }

    AudioFingerprint {
        bands,
        centroid_norm: (weighted_sum / mag_sum).clamp(0.0, 1.0),
        attack_ratio,
        decay_ratio,
        zero_crossing_rate,
        rms,
    }
}

pub fn fingerprint_distance(a: &AudioFingerprint, b: &AudioFingerprint) -> f32 {
    let mut spec_d = 0.0_f32;
    for i in 0..16 {
        let diff = a.bands[i] - b.bands[i];
        spec_d += diff * diff;
    }
    let spec_rms = (spec_d / 16.0).sqrt();
    let cent_d = (a.centroid_norm - b.centroid_norm).abs();
    let env_d =
        ((a.attack_ratio - b.attack_ratio).abs() + (a.decay_ratio - b.decay_ratio).abs()) * 0.25;
    let zcr_d = (a.zero_crossing_rate - b.zero_crossing_rate).abs() * 1.5;
    spec_rms * 0.55 + cent_d * 0.20 + env_d * 0.15 + zcr_d * 0.10
}

/// Load a 16-bit PCM mono/stereo `.wav` file into normalized `f32` samples for Genopatch matching.
pub fn load_wav_mono_samples(path: &Path) -> Result<(Vec<f32>, u32), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("Not a valid RIFF/WAVE file".into());
    }
    let channels = u16::from_le_bytes([bytes[22], bytes[23]]).max(1) as usize;
    let sr = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]).max(8000);
    let bits = u16::from_le_bytes([bytes[34], bytes[35]]);
    // Locate "data" chunk
    let mut pos = 12usize;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let sz = u32::from_le_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;
        pos += 8;
        if id == b"data" {
            let end = (pos + sz).min(bytes.len());
            let mut out = Vec::new();
            if bits == 16 {
                let frame_bytes = 2 * channels;
                let mut cur = pos;
                while cur + frame_bytes <= end && out.len() < sr as usize {
                    let s = i16::from_le_bytes([bytes[cur], bytes[cur + 1]]) as f32 / 32768.0;
                    out.push(s);
                    cur += frame_bytes;
                }
            } else if bits == 24 {
                let frame_bytes = 3 * channels;
                let mut cur = pos;
                while cur + frame_bytes <= end && out.len() < sr as usize {
                    let v = ((bytes[cur] as i32)
                        | ((bytes[cur + 1] as i32) << 8)
                        | ((bytes[cur + 2] as i32) << 16))
                        << 8
                        >> 8;
                    out.push(v as f32 / 8_388_608.0);
                    cur += frame_bytes;
                }
            } else {
                return Err(format!("Unsupported WAV bit depth: {bits}"));
            }
            return Ok((out, sr));
        }
        pos = pos.saturating_add(sz);
    }
    Err("No data chunk found in WAV".into())
}

struct XorShift(u64);
impl XorShift {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0.max(1);
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 16) as u32
    }
    fn f01(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }
    fn signed(&mut self) -> f32 {
        self.f01() * 2.0 - 1.0
    }
}

fn mutate_patch(parent: &SynthPatch, rng: &mut XorShift, scale: f32) -> SynthPatch {
    const WAVES: [WaveformKind; 8] = [
        WaveformKind::Sine,
        WaveformKind::Saw,
        WaveformKind::Square,
        WaveformKind::Triangle,
        WaveformKind::PulsePwm,
        WaveformKind::SuperSaw,
        WaveformKind::WhiteNoise,
        WaveformKind::SubOctave,
    ];
    const FILTERS: [FilterMode; 4] = [
        FilterMode::LowPass24dB,
        FilterMode::BandPass12dB,
        FilterMode::HighPass12dB,
        FilterMode::AcidResonant,
    ];
    let extreme = ExtremeDspMode::all();

    let mut c = parent.clone();
    if rng.f01() < 0.25 * scale {
        c.vco1_wave = WAVES[(rng.next_u32() as usize) % WAVES.len()];
    }
    if rng.f01() < 0.25 * scale {
        c.vco2_wave = WAVES[(rng.next_u32() as usize) % WAVES.len()];
    }
    if rng.f01() < 0.20 * scale {
        c.filter_mode = FILTERS[(rng.next_u32() as usize) % FILTERS.len()];
    }
    if rng.f01() < 0.20 * scale && !extreme.is_empty() {
        c.extreme_dsp = extreme[(rng.next_u32() as usize) % extreme.len()];
    }
    c.vco2_detune_cents = (c.vco2_detune_cents + rng.signed() * 25.0 * scale).clamp(-50.0, 50.0);
    c.vco_mix = (c.vco_mix + rng.signed() * 0.35 * scale).clamp(0.0, 1.0);
    c.fm_index = (c.fm_index + rng.signed() * 1.4 * scale).clamp(0.0, 4.0);
    c.pulse_width = (c.pulse_width + rng.signed() * 0.25 * scale).clamp(0.08, 0.92);
    c.cutoff_hz = (c.cutoff_hz * 2.0_f32.powf(rng.signed() * 2.0 * scale)).clamp(60.0, 12000.0);
    c.resonance_q = (c.resonance_q + rng.signed() * 3.5 * scale).clamp(0.5, 11.5);
    c.env_mod_depth = (c.env_mod_depth + rng.signed() * 0.35 * scale).clamp(0.0, 1.0);
    c.drive_saturation = (c.drive_saturation + rng.signed() * 0.9 * scale).clamp(0.0, 3.0);
    c.extreme_morph = (c.extreme_morph + rng.signed() * 0.35 * scale).clamp(0.0, 1.0);
    c.attack_ms = (c.attack_ms * 2.0_f32.powf(rng.signed() * 1.2 * scale)).clamp(1.0, 450.0);
    c.decay_ms = (c.decay_ms * 2.0_f32.powf(rng.signed() * 1.2 * scale)).clamp(25.0, 1500.0);
    c.sustain_level = (c.sustain_level + rng.signed() * 0.30 * scale).clamp(0.0, 1.0);
    c
}

/// Run a multi-core evolutionary `(mu + lambda)` Genopatch search across available CPU threads.
pub fn evolve_matching_patches(
    target_name: &str,
    target_samples: &[f32],
    seed_patch: &SynthPatch,
    sample_rate_hz: u32,
    generations: usize,
    population_size: usize,
) -> GenopatchReport {
    let t0 = std::time::Instant::now();
    let target_fp = extract_fingerprint(target_samples, sample_rate_hz);
    let gens = generations.clamp(2, 24);
    let pop_size = population_size.clamp(12, 96);
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(pop_size);

    let mut rng = XorShift(0x5EED_6E40_2026);
    let mut elites: Vec<(f32, SynthPatch)> = vec![(f32::MAX, seed_patch.clone())];
    let mut total_evals = 0usize;

    for gen in 0..gens {
        let mut_scale = (1.0 - (gen as f32 / gens as f32) * 0.65).clamp(0.25, 1.0);
        let mut pool: Vec<SynthPatch> = Vec::with_capacity(pop_size);
        for (_, ep) in elites.iter().take(6) {
            pool.push(ep.clone());
        }
        while pool.len() < pop_size {
            let parent = &elites[pool.len() % elites.len().min(6)].1;
            pool.push(mutate_patch(parent, &mut rng, mut_scale));
        }

        // Parallel scoring across CPU cores using `std::thread::scope`
        let chunk_size = pool.len().div_ceil(threads).max(1);
        let scored: Vec<(f32, SynthPatch)> = std::thread::scope(|s| {
            let mut handles = Vec::new();
            for chunk in pool.chunks(chunk_size) {
                let fp_ref = &target_fp;
                handles.push(s.spawn(move || {
                    let mut local = Vec::with_capacity(chunk.len());
                    for cand in chunk {
                        let rendered = synthesize_audio_buffer(cand, sample_rate_hz, 120);
                        let fp = extract_fingerprint(&rendered, sample_rate_hz);
                        let d = fingerprint_distance(fp_ref, &fp);
                        local.push((d, cand.clone()));
                    }
                    local
                }));
            }
            let mut merged = Vec::with_capacity(pool.len());
            for h in handles {
                if let Ok(v) = h.join() {
                    merged.extend(v);
                }
            }
            merged
        });

        total_evals += scored.len();
        elites = scored;
        elites.sort_by(|a, b| a.0.total_cmp(&b.0));
        elites.truncate(6);
    }

    let candidates: Vec<GenopatchCandidate> = elites
        .into_iter()
        .enumerate()
        .map(|(i, (dist, mut patch))| {
            let sim = ((1.0 - dist * 1.45) * 100.0).clamp(5.0, 99.8);
            patch.id = format!("genopatch_branch_{}", i + 1);
            patch.name = format!("Geno #{} ({:.1}% -- {})", i + 1, sim, target_name);
            let _ = analyze_patch_spectrum(&patch, sample_rate_hz, 60);
            GenopatchCandidate {
                branch_id: i + 1,
                label: format!(
                    "DNA Branch #{} | {}+{} | {} | {:.0} Hz",
                    i + 1,
                    patch.vco1_wave.as_str(),
                    patch.vco2_wave.as_str(),
                    patch.filter_mode.as_str(),
                    patch.cutoff_hz
                ),
                similarity_pct: sim,
                spectral_distance: dist,
                patch,
            }
        })
        .collect();

    GenopatchReport {
        target_name: target_name.to_string(),
        generations: gens,
        population_size: pop_size,
        threads_used: threads,
        total_evaluations: total_evals,
        elapsed_ms: t0.elapsed().as_millis() as u64,
        candidates,
    }
}
