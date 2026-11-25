//! Low-level DSP building blocks for SynthCraft: band-limited oscillators (PolyBLEP),
//! a zero-delay-feedback 4-pole ladder filter, an ADSR with real note-off release,
//! a DC blocker, and the FFT and pitch tools used by the analyser.
//!
//! Everything here is allocation-free per sample so it can run in an audio callback.

use crate::model::{FilterMode, WaveformKind};
use std::f32::consts::{PI, TAU};

/// PolyBLEP residual for a unit step discontinuity at phase 0 (`t` in `[0,1)`, `dt` = phase increment).
pub fn poly_blep(t: f32, dt: f32) -> f32 {
    if dt <= 0.0 {
        return 0.0;
    }
    if t < dt {
        let x = t / dt;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}

const SUPERSAW_CENTS: [f32; 7] = [-23.0, -14.0, -5.0, 0.0, 5.0, 14.0, 23.0];
const SUPERSAW_GAIN: [f32; 7] = [0.62, 0.72, 0.85, 1.0, 0.85, 0.72, 0.62];

/// One oscillator voice. Phase is integrated per sample, so frequency changes (glide, bend)
/// never cause phase jumps.
#[derive(Debug, Clone, Copy)]
pub struct Osc {
    phase: f32,
    flip: f32,
    rng: u32,
    ss_phase: [f32; 7],
    pad_ph: [f32; 12],
    pad_lfo: [f32; 6],
    breath: f32,
}

impl Osc {
    pub fn new(seed: u32) -> Self {
        let mut ss_phase = [0.0_f32; 7];
        for (i, p) in ss_phase.iter_mut().enumerate() {
            *p = (i as f32 * 0.137 + 0.05).rem_euclid(1.0);
        }
        let mut o = Self {
            phase: 0.0,
            flip: 1.0,
            rng: seed.max(1),
            ss_phase,
            pad_ph: [0.0; 12],
            pad_lfo: [0.0; 6],
            breath: 0.0,
        };
        for k in 0..6 {
            let r = o.next_noise();
            o.pad_lfo[k] = (r * 0.5 + 0.5).clamp(0.0, 0.999);
        }
        o
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.flip = 1.0;
        for (i, p) in self.ss_phase.iter_mut().enumerate() {
            *p = (i as f32 * 0.137 + 0.05).rem_euclid(1.0);
        }
        self.pad_ph = [0.0; 12];
        self.breath = 0.0;
    }

    /// Evolving atmospheric pad: six harmonics, each a detuned pair whose level and detune
    /// drift on their own slow LFOs, plus a breath of low-passed noise.
    fn pad_tick(&mut self, freq: f32, sr: f32) -> f32 {
        const HARM: [f32; 6] = [1.0, 2.0, 3.0, 4.0, 6.0, 8.0];
        const WEIGHT: [f32; 6] = [1.0, 0.555, 0.393, 0.308, 0.218, 0.171];
        let mut acc = 0.0_f32;
        let mut norm = 0.0_f32;
        for k in 0..6 {
            self.pad_lfo[k] += (0.045 + 0.037 * k as f32) / sr;
            if self.pad_lfo[k] >= 1.0 {
                self.pad_lfo[k] -= 1.0;
            }
            let l = (TAU * self.pad_lfo[k]).sin();
            let f = freq * HARM[k];
            if f > sr * 0.45 {
                continue;
            }
            let cents = 5.0 + 2.0 * k as f32 + 3.0 * l;
            let spread = cents * 5.776e-4;
            self.pad_ph[2 * k] += f * (1.0 + spread) / sr;
            self.pad_ph[2 * k + 1] += f * (1.0 - spread) / sr;
            for j in 0..2 {
                let p = &mut self.pad_ph[2 * k + j];
                if *p >= 1.0 {
                    *p -= 1.0;
                }
            }
            let amp = (0.55 + 0.45 * l) * WEIGHT[k];
            acc += amp
                * 0.5
                * ((TAU * self.pad_ph[2 * k]).sin() + (TAU * self.pad_ph[2 * k + 1]).sin());
            norm += WEIGHT[k];
        }
        let n = self.next_noise();
        self.breath += 0.12 * (n - self.breath);
        let breath_amp = 0.06 * (0.6 + 0.4 * (TAU * self.pad_lfo[0]).sin());
        acc / norm.max(1e-3) * 1.8 + self.breath * breath_amp * 3.0
    }

    fn next_noise(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x as f32 / 2_147_483_648.0) - 1.0
    }

    /// Current normalized oscillator phase in `[0.0, 1.0)`.
    pub fn phase(&self) -> f32 {
        self.phase
    }

    /// Produce one sample at `freq` Hz. `pm` is phase modulation in cycles (used for FM).
    pub fn tick(
        &mut self,
        kind: WaveformKind,
        freq: f32,
        sr: f32,
        pulse_width: f32,
        pm: f32,
    ) -> f32 {
        let dt = (freq / sr).clamp(0.0, 0.45);
        let p = (self.phase + pm).rem_euclid(1.0);
        let out = match kind {
            WaveformKind::Sine => (TAU * p).sin(),
            WaveformKind::Saw => 2.0 * p - 1.0 - poly_blep(p, dt),
            WaveformKind::Square => {
                let v = if p < 0.5 { 1.0 } else { -1.0 };
                v + poly_blep(p, dt) - poly_blep((p + 0.5).rem_euclid(1.0), dt)
            }
            WaveformKind::PulsePwm => {
                let w = pulse_width.clamp(0.05, 0.95);
                let v = if p < w { 1.0 } else { -1.0 };
                v + poly_blep(p, dt) - poly_blep((p + 1.0 - w).rem_euclid(1.0), dt)
            }
            WaveformKind::Triangle => 1.0 - 4.0 * (p - 0.5).abs(),
            WaveformKind::WhiteNoise => self.next_noise(),
            WaveformKind::SubOctave => (PI * p).sin() * self.flip,
            WaveformKind::AtmoPad => self.pad_tick(freq, sr),
            WaveformKind::Wavetable => (TAU * p).sin(),
            WaveformKind::SuperSaw => {
                let mut acc = 0.0_f32;
                let mut norm = 0.0_f32;
                for i in 0..7 {
                    let ratio = 2.0_f32.powf(SUPERSAW_CENTS[i] / 1200.0);
                    let d = (dt * ratio).clamp(0.0, 0.45);
                    let q = (self.ss_phase[i] + pm).rem_euclid(1.0);
                    acc += SUPERSAW_GAIN[i] * (2.0 * q - 1.0 - poly_blep(q, d));
                    norm += SUPERSAW_GAIN[i];
                    let mut np = self.ss_phase[i] + d;
                    if np >= 1.0 {
                        np -= 1.0;
                    }
                    self.ss_phase[i] = np;
                }
                acc / norm
            }
        };
        self.phase += dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.flip = -self.flip;
        }
        out
    }
}

/// Zero-delay-feedback 4-pole ladder (four TPT one-poles with global resonance feedback).
#[derive(Debug, Clone, Copy, Default)]
pub struct Ladder {
    s: [f32; 4],
}

impl Ladder {
    pub fn reset(&mut self) {
        self.s = [0.0; 4];
    }

    /// `q` is resonance in the patch range `0.5..=12.0`.
    pub fn process(&mut self, x: f32, cutoff_hz: f32, sr: f32, q: f32, mode: FilterMode) -> f32 {
        let fc = cutoff_hz.clamp(20.0, sr * 0.45);
        let g = (PI * fc / sr).tan();
        let gg = g / (1.0 + g);
        let k_max = if mode == FilterMode::AcidResonant {
            4.1
        } else {
            3.9
        };
        let k = ((q - 0.5) / 11.5).clamp(0.0, 1.0) * k_max;

        let g2 = gg * gg;
        let g3 = g2 * gg;
        let g4 = g3 * gg;
        let sigma = (1.0 - gg) * (g3 * self.s[0] + g2 * self.s[1] + gg * self.s[2] + self.s[3]);
        let u = ((x - k * sigma) / (1.0 + k * g4)).tanh();

        let mut input = u;
        let mut y = [0.0_f32; 4];
        for (stage, out) in self.s.iter_mut().zip(y.iter_mut()) {
            let v = (input - *stage) * gg;
            let o = v + *stage;
            *stage = o + v;
            *out = o;
            input = o;
        }

        // Make-up gain so heavy resonance does not remove all low-frequency energy.
        let comp = 1.0 + 0.35 * (k / k_max);
        match mode {
            FilterMode::LowPass24dB => y[3] * comp,
            FilterMode::AcidResonant => (y[3] * comp * 1.15).tanh(),
            FilterMode::BandPass12dB => 2.0 * (y[0] - y[1]),
            FilterMode::HighPass12dB => u - 2.0 * y[0] + y[1],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// ADSR with a true release stage that starts from the current level on note-off.
#[derive(Debug, Clone, Copy)]
pub struct Adsr {
    pub stage: EnvStage,
    pub level: f32,
}

impl Default for Adsr {
    fn default() -> Self {
        Self {
            stage: EnvStage::Idle,
            level: 0.0,
        }
    }
}

impl Adsr {
    pub fn gate_on(&mut self) {
        self.stage = EnvStage::Attack;
    }

    pub fn gate_off(&mut self) {
        if self.stage != EnvStage::Idle {
            self.stage = EnvStage::Release;
        }
    }

    pub fn is_idle(&self) -> bool {
        self.stage == EnvStage::Idle
    }

    pub fn tick(
        &mut self,
        attack_ms: f32,
        decay_ms: f32,
        sustain: f32,
        release_ms: f32,
        sr: f32,
    ) -> f32 {
        let sustain = sustain.clamp(0.0, 1.0);
        match self.stage {
            EnvStage::Idle => self.level = 0.0,
            EnvStage::Attack => {
                let n = (attack_ms.max(0.5) * 0.001 * sr).max(1.0);
                self.level += 1.0 / n;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = EnvStage::Decay;
                }
            }
            EnvStage::Decay => {
                let c = (-6.9 / (decay_ms.max(1.0) * 0.001 * sr)).exp();
                self.level = sustain + (self.level - sustain) * c;
                if (self.level - sustain).abs() < 1e-4 {
                    self.level = sustain;
                    self.stage = EnvStage::Sustain;
                }
            }
            EnvStage::Sustain => self.level = sustain,
            EnvStage::Release => {
                let c = (-6.9 / (release_ms.max(1.0) * 0.001 * sr)).exp();
                self.level *= c;
                if self.level < 1e-4 {
                    self.level = 0.0;
                    self.stage = EnvStage::Idle;
                }
            }
        }
        self.level
    }
}

/// First-order DC blocker (about 10 Hz corner at 48 kHz).
#[derive(Debug, Clone, Copy, Default)]
pub struct DcBlocker {
    x1: f32,
    y1: f32,
}

impl DcBlocker {
    pub fn process(&mut self, x: f32) -> f32 {
        let y = x - self.x1 + 0.9987 * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
}

/// In-place iterative radix-2 FFT. `re.len()` must equal `im.len()` and be a power of two.
pub fn fft_in_place(re: &mut [f32], im: &mut [f32]) {
    let n = re.len().min(im.len());
    if n < 2 || !n.is_power_of_two() {
        return;
    }
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2usize;
    while len <= n {
        let ang = -TAU / len as f32;
        let (wr, wi) = (ang.cos(), ang.sin());
        let mut start = 0usize;
        while start < n {
            let (mut cr, mut ci) = (1.0_f32, 0.0_f32);
            for k in 0..len / 2 {
                let a = start + k;
                let b = a + len / 2;
                let tr = re[b] * cr - im[b] * ci;
                let ti = re[b] * ci + im[b] * cr;
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                let ncr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = ncr;
            }
            start += len;
        }
        len <<= 1;
    }
}

pub fn hann(i: usize, n: usize) -> f32 {
    0.5 - 0.5 * (TAU * i as f32 / n.max(1) as f32).cos()
}

/// Magnitude spectrum in dBFS of a Hann-windowed block (power-of-two length). A full-scale
/// sine reads close to 0 dBFS. Returns `n / 2` bins.
pub fn magnitude_db(block: &[f32]) -> Vec<f32> {
    let n = block.len();
    if n < 2 || !n.is_power_of_two() {
        return Vec::new();
    }
    let mut re: Vec<f32> = block
        .iter()
        .enumerate()
        .map(|(i, &s)| s * hann(i, n))
        .collect();
    let mut im = vec![0.0_f32; n];
    fft_in_place(&mut re, &mut im);
    let norm = n as f32 * 0.25;
    re.iter()
        .zip(im.iter())
        .take(n / 2)
        .map(|(&r, &i)| 20.0 * ((r * r + i * i).sqrt() / norm).max(1e-6).log10())
        .collect()
}

/// Normalised autocorrelation pitch estimate in Hz, or `0.0` when the block is not clearly periodic.
pub fn autocorrelation_pitch(block: &[f32], sr: f32) -> f32 {
    let n = block.len();
    let min_lag = (sr / 2000.0).max(2.0) as usize;
    let max_lag = ((sr / 40.0) as usize).min(n / 2);
    if n < 64 || max_lag <= min_lag + 2 {
        return 0.0;
    }
    let energy: f32 = block.iter().map(|s| s * s).sum();
    if energy < 1e-6 {
        return 0.0;
    }
    let span = n - max_lag;
    let mut acf = vec![0.0_f32; max_lag + 2];
    for (lag, slot) in acf.iter_mut().enumerate().take(max_lag + 1).skip(1) {
        let mut num = 0.0_f32;
        let mut e2 = 0.0_f32;
        for i in 0..span {
            num += block[i] * block[i + lag];
            e2 += block[i + lag] * block[i + lag];
        }
        let e1: f32 = block[..span].iter().map(|s| s * s).sum();
        let den = (e1 * e2).sqrt().max(1e-9);
        *slot = num / den;
    }
    // Skip the initial lobe: start after the first time the ACF goes negative or below 0.
    let mut start = min_lag;
    while start < max_lag && acf[start] > 0.0 {
        start += 1;
    }
    let best = acf[start..=max_lag]
        .iter()
        .copied()
        .fold(f32::MIN, f32::max);
    if best < 0.5 {
        return 0.0;
    }
    let threshold = best * 0.9;
    for lag in start.max(min_lag + 1)..max_lag {
        if acf[lag] >= threshold && acf[lag] >= acf[lag - 1] && acf[lag] >= acf[lag + 1] {
            let (a, b, c) = (acf[lag - 1], acf[lag], acf[lag + 1]);
            let denom = a - 2.0 * b + c;
            let shift = if denom.abs() > 1e-9 {
                0.5 * (a - c) / denom
            } else {
                0.0
            };
            return sr / (lag as f32 + shift);
        }
    }
    0.0
}

/// Fold a magnitude spectrum (dBFS per bin, `bin_hz` apart) into 32 log-spaced bands from
/// 40 Hz to 12 kHz. Returns `(centre_hz, dBFS)` pairs; empty input gives a floor of -80 dB.
pub fn log_bands(mag_db: &[f32], bin_hz: f32) -> Vec<(f32, f32)> {
    let ratio = (12000.0_f32 / 40.0).powf(1.0 / 31.0);
    (0..32)
        .map(|b| {
            let freq = 40.0 * ratio.powi(b);
            let lo = ((freq / ratio.sqrt()) / bin_hz.max(1e-3)).floor() as usize;
            let hi = (((freq * ratio.sqrt()) / bin_hz.max(1e-3)).ceil() as usize).max(lo + 1);
            let mut db = -80.0_f32;
            for k in lo..=hi {
                if let Some(&v) = mag_db.get(k) {
                    db = db.max(v);
                }
            }
            (
                (freq * 100.0).round() / 100.0,
                (db.clamp(-80.0, 6.0) * 100.0).round() / 100.0,
            )
        })
        .collect()
}

/// Two-tap modulated-delay chorus.
#[derive(Debug, Clone)]
pub struct Chorus {
    buf: Vec<f32>,
    w: usize,
    ph: [f32; 2],
    sr: f32,
}

impl Chorus {
    pub fn new(sr: f32) -> Self {
        Self {
            buf: vec![0.0; (sr * 0.05) as usize + 8],
            w: 0,
            ph: [0.0, 0.5],
            sr,
        }
    }

    /// Mono in, mono out.
    pub fn process(&mut self, x: f32, mix: f32) -> f32 {
        let t = self.write_and_taps(x);
        x * (1.0 - 0.5 * mix) + (t[0] + t[1]) * 0.5 * mix * 0.75
    }

    /// Stereo in, stereo out: the two modulated taps go to opposite channels, so the chorus
    /// widens the sound. The mono sum equals [`Chorus::process`] of the mono sum.
    pub fn process_stereo(&mut self, l: f32, r: f32, mix: f32) -> (f32, f32) {
        let t = self.write_and_taps((l + r) * 0.5);
        (
            l * (1.0 - 0.5 * mix) + t[0] * 0.75 * mix,
            r * (1.0 - 0.5 * mix) + t[1] * 0.75 * mix,
        )
    }

    fn write_and_taps(&mut self, x: f32) -> [f32; 2] {
        let len = self.buf.len();
        if let Some(slot) = self.buf.get_mut(self.w) {
            *slot = x;
        }
        let mut taps = [0.0_f32; 2];
        for (i, rate) in [0.31_f32, 0.43].into_iter().enumerate() {
            self.ph[i] += rate / self.sr;
            if self.ph[i] >= 1.0 {
                self.ph[i] -= 1.0;
            }
            let delay = (0.013 + 0.004 * (TAU * self.ph[i]).sin()) * self.sr;
            let pos = (self.w as f32 - delay).rem_euclid(len as f32);
            let i0 = pos.floor() as usize % len;
            let i1 = (i0 + 1) % len;
            let fr = pos - pos.floor();
            let a = self.buf.get(i0).copied().unwrap_or(0.0);
            let b = self.buf.get(i1).copied().unwrap_or(0.0);
            taps[i] = a + (b - a) * fr;
        }
        self.w = (self.w + 1) % len;
        taps
    }
}

#[derive(Debug, Clone)]
struct Comb {
    buf: Vec<f32>,
    i: usize,
    store: f32,
}

#[derive(Debug, Clone)]
struct Allpass {
    buf: Vec<f32>,
    i: usize,
}

const COMB_TUNING: [f32; 8] = [
    1116.0, 1188.0, 1277.0, 1356.0, 1422.0, 1491.0, 1557.0, 1617.0,
];
const ALLPASS_TUNING: [f32; 4] = [556.0, 441.0, 341.0, 225.0];

/// Mono Freeverb-style reverb with a decay time (RT60) control and high-frequency damping.
#[derive(Debug, Clone)]
pub struct Reverb {
    combs: Vec<Comb>,
    allpass: Vec<Allpass>,
    gains: [f32; 8],
    sr: f32,
    decay: f32,
    damp: f32,
}

impl Reverb {
    pub fn new(sr: f32) -> Self {
        Self::with_spread(sr, 0.0)
    }

    /// Like [`Reverb::new`] with every delay line lengthened by `spread` samples (at 44.1 kHz).
    /// Two reverbs with different spreads make a decorrelated stereo pair.
    pub fn with_spread(sr: f32, spread: f32) -> Self {
        let scale = sr / 44100.0;
        let mut r = Self {
            combs: COMB_TUNING
                .iter()
                .map(|t| Comb {
                    buf: vec![0.0; ((t + spread) * scale).round().max(2.0) as usize],
                    i: 0,
                    store: 0.0,
                })
                .collect(),
            allpass: ALLPASS_TUNING
                .iter()
                .map(|t| Allpass {
                    buf: vec![0.0; ((t + spread) * scale).round().max(2.0) as usize],
                    i: 0,
                })
                .collect(),
            gains: [0.8; 8],
            sr,
            decay: -1.0,
            damp: 0.5,
        };
        r.set(3.0, 0.5);
        r
    }

    /// `rt60_s` in seconds (0.2..=20), `damping` 0..=1 (more = darker tail).
    pub fn set(&mut self, rt60_s: f32, damping: f32) {
        let rt = rt60_s.clamp(0.2, 20.0);
        self.damp = damping.clamp(0.0, 0.95);
        if (rt - self.decay).abs() < 1e-4 {
            return;
        }
        self.decay = rt;
        for (g, c) in self.gains.iter_mut().zip(&self.combs) {
            *g = 10.0_f32
                .powf(-3.0 * c.buf.len() as f32 / (rt * self.sr))
                .min(0.985);
        }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let input = x * 0.015;
        let mut sum = 0.0_f32;
        for (c, g) in self.combs.iter_mut().zip(self.gains) {
            let n = c.buf.len();
            let out = c.buf.get(c.i).copied().unwrap_or(0.0);
            c.store = out * (1.0 - self.damp) + c.store * self.damp;
            if let Some(slot) = c.buf.get_mut(c.i) {
                *slot = input + c.store * g;
            }
            c.i = (c.i + 1) % n;
            sum += out;
        }
        for a in &mut self.allpass {
            let n = a.buf.len();
            let bo = a.buf.get(a.i).copied().unwrap_or(0.0);
            let out = -sum + bo;
            if let Some(slot) = a.buf.get_mut(a.i) {
                *slot = sum + bo * 0.5;
            }
            a.i = (a.i + 1) % n;
            sum = out;
        }
        sum * 3.0
    }
}

/// Slow, seeded, bounded drift for the ambient mode. Three outputs in -1..=1, each the sum of
/// two incommensurate sines, so the pattern does not audibly repeat. Same seed, same drift.
#[derive(Debug, Clone)]
pub struct Drift {
    phase: [f32; 6],
}

impl Drift {
    const RATIOS: [f32; 6] = [1.0, 0.618, 0.731, 0.377, 0.467, 0.887];

    pub fn new(seed: u32) -> Self {
        let mut s = seed ^ 0x9E37_79B9;
        let mut phase = [0.0_f32; 6];
        for p in &mut phase {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            s ^= s >> 15;
            *p = (s >> 8) as f32 / 16_777_216.0;
        }
        Self { phase }
    }

    /// Advance one sample at `rate_hz` and return the three drift values.
    pub fn tick(&mut self, rate_hz: f32, sr: f32) -> [f32; 3] {
        let inc = rate_hz.clamp(0.001, 2.0) / sr;
        for (p, k) in self.phase.iter_mut().zip(Self::RATIOS) {
            *p += inc * k;
            if *p >= 1.0 {
                *p -= 1.0;
            }
        }
        let s = |i: usize| (TAU * self.phase.get(i).copied().unwrap_or(0.0)).sin();
        [
            0.5 * (s(0) + s(1)),
            0.5 * (s(2) + s(3)),
            0.5 * (s(4) + s(5)),
        ]
    }
}
