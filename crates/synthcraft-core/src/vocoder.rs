//! One-Shot #11: 16-Band Analog Vocoder & Robotic Talkbox Engine (`#![forbid(unsafe_code)]`).
//!
//! Classic Sennheiser VSM-201 / Roland VP-330 16-band filterbank vocoder:
//! - 16 log-spaced bandpass analysis filters (`110 Hz .. 7600 Hz`) with envelope followers
//! - 16 matched carrier synthesis bandpass filters with Formant Shift (`-12..+12 st`)
//! - Built-in tempo-synced synthetic speech/robotic vowel modulator phrases (works immediately
//!   with zero microphone setup) OR external `DJFX` audio modulator!

use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};

pub const VOCODER_BANDS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VocoderPhrase {
    SynthCraftVienna,
    VowelCycleAeiou,
    DaftRobotFunk,
    WhisperChoir,
}

impl VocoderPhrase {
    pub const ALL: [Self; 4] = [
        Self::SynthCraftVienna,
        Self::VowelCycleAeiou,
        Self::DaftRobotFunk,
        Self::WhisperChoir,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::SynthCraftVienna => "SYNTH-CRAFT-VI-EN-NA",
            Self::VowelCycleAeiou => "A - E - I - O - U Cycle",
            Self::DaftRobotFunk => "RO-BOT-FUNK-ACID",
            Self::WhisperChoir => "Whispering Formant Choir",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VocoderParams {
    pub enabled: bool,
    pub phrase: VocoderPhrase,
    pub formant_shift_st: f32, // -12.0 ..= +12.0 semitones
    pub attack_ms: f32,        // 2.0 ..= 60.0 ms
    pub release_ms: f32,       // 15.0 ..= 300.0 ms
    pub sibilance_hf: f32,     // 0.0 ..= 1.0 consonant crispness
    pub mix: f32,              // 0.0 ..= 1.0
}

impl Default for VocoderParams {
    fn default() -> Self {
        Self {
            enabled: false,
            phrase: VocoderPhrase::SynthCraftVienna,
            formant_shift_st: 0.0,
            attack_ms: 8.0,
            release_ms: 65.0,
            sibilance_hf: 0.35,
            mix: 0.70,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct BandSvf {
    lp: f32,
    bp: f32,
}

impl BandSvf {
    #[inline]
    fn tick_bp(&mut self, input: f32, freq_hz: f32, q: f32, sr: f32) -> f32 {
        let f = (2.0 * (PI * freq_hz.clamp(60.0, sr * 0.42) / sr).sin()).clamp(0.002, 0.92);
        let damp = (1.0 / q.clamp(1.0, 14.0)).clamp(0.07, 1.0);
        let hp = input - self.lp - damp * self.bp;
        self.bp += f * hp;
        self.lp += f * self.bp;
        self.bp
    }
}

#[derive(Debug, Clone)]
pub struct VocoderRuntime {
    mod_bands: [BandSvf; VOCODER_BANDS],
    car_bands_l: [BandSvf; VOCODER_BANDS],
    car_bands_r: [BandSvf; VOCODER_BANDS],
    envelopes: [f32; VOCODER_BANDS],
    mod_phase: f32,
    syllable_phase: f32,
    rng: u32,
}

impl Default for VocoderRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl VocoderRuntime {
    pub fn new() -> Self {
        Self {
            mod_bands: [BandSvf::default(); VOCODER_BANDS],
            car_bands_l: [BandSvf::default(); VOCODER_BANDS],
            car_bands_r: [BandSvf::default(); VOCODER_BANDS],
            envelopes: [0.0; VOCODER_BANDS],
            mod_phase: 0.0,
            syllable_phase: 0.0,
            rng: 0x00C0_DE16,
        }
    }

    #[inline]
    fn next_noise(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    #[inline]
    fn synth_modulator(&mut self, phrase: VocoderPhrase, bpm: f32, sibilance: f32, sr: f32) -> f32 {
        // Syllable clock locked to 8th-notes at current BPM
        let syl_hz = (bpm.clamp(60.0, 220.0) / 60.0) * 2.0;
        self.syllable_phase = (self.syllable_phase + syl_hz / sr).fract();
        let step = (self.syllable_phase * 8.0) as usize;
        let local = (self.syllable_phase * 8.0).fract();

        // Glottal buzz oscillator (~135 Hz) + syllable-onset consonant sibilance burst
        self.mod_phase = (self.mod_phase + 135.0 / sr).fract();
        let buzz = 2.0 * self.mod_phase - 1.0;
        let consonant = if local < 0.12 {
            self.next_noise() * sibilance * 1.4
        } else {
            self.next_noise() * sibilance * 0.12
        };

        // Modulate spectral tilt per syllable to articulate vowels across the 16 bands
        let vowel_shape = match phrase {
            VocoderPhrase::SynthCraftVienna => {
                [0.9, 0.3, 0.7, 0.2, 0.85, 0.4, 0.95, 0.15][step % 8]
            }
            VocoderPhrase::VowelCycleAeiou => [0.2, 0.4, 0.6, 0.8, 1.0, 0.75, 0.5, 0.3][step % 8],
            VocoderPhrase::DaftRobotFunk => [1.0, 0.15, 0.9, 0.25, 0.95, 0.1, 0.8, 0.3][step % 8],
            VocoderPhrase::WhisperChoir => 0.5 + 0.4 * (TAU * self.syllable_phase).sin(),
        };
        let syll_env = (1.0 - (local * 1.25).min(1.0)).powf(0.45);
        (buzz * vowel_shape + consonant) * syll_env
    }

    /// Process stereo carrier through the 16-band analog vocoder. Allocation-free.
    #[inline]
    pub fn process_stereo(
        &mut self,
        car_l: f32,
        car_r: f32,
        ext_mod: Option<f32>,
        params: &VocoderParams,
        bpm: f32,
        sr: f32,
    ) -> (f32, f32) {
        if !params.enabled || params.mix <= 0.001 {
            return (car_l, car_r);
        }
        let mod_sig = ext_mod
            .unwrap_or_else(|| self.synth_modulator(params.phrase, bpm, params.sibilance_hf, sr));
        let atk = (1.0 / (params.attack_ms.clamp(2.0, 60.0) * 0.001 * sr)).clamp(0.001, 0.5);
        let rel = (1.0 / (params.release_ms.clamp(15.0, 300.0) * 0.001 * sr)).clamp(0.0002, 0.2);
        let shift = 2.0_f32.powf(params.formant_shift_st.clamp(-12.0, 12.0) / 12.0);

        let mut wet_l = 0.0_f32;
        let mut wet_r = 0.0_f32;

        for b in 0..VOCODER_BANDS {
            let frac = b as f32 / (VOCODER_BANDS - 1) as f32;
            let center_hz = 110.0 * (7600.0_f32 / 110.0).powf(frac);
            let mod_band = self.mod_bands[b].tick_bp(mod_sig, center_hz, 5.5, sr);
            let rect = mod_band.abs();
            let c = if rect > self.envelopes[b] { atk } else { rel };
            self.envelopes[b] += (rect - self.envelopes[b]) * c;

            let car_hz = (center_hz * shift).clamp(70.0, sr * 0.42);
            let cl = self.car_bands_l[b].tick_bp(car_l, car_hz, 5.5, sr);
            let cr = self.car_bands_r[b].tick_bp(car_r, car_hz, 5.5, sr);
            let env = (self.envelopes[b] * 4.5).min(2.5);
            wet_l += cl * env;
            wet_r += cr * env;
        }

        let m = params.mix.clamp(0.0, 1.0);
        (
            car_l * (1.0 - m) + (wet_l * 0.75).tanh() * m,
            car_r * (1.0 - m) + (wet_r * 0.75).tanh() * m,
        )
    }
}
