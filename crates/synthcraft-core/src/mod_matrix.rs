//! General Modulation Matrix: route any modulation source to any synthesis destination
//! with bipolar depth (`-1.0..=1.0`), evaluated allocation-free inside `SynthRt`.

use serde::{Deserialize, Serialize};

/// Modulation source inside the real-time voice/global engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModSource {
    Lfo,
    AdsrEnv,
    Velocity,
    ModWheel,
    SampleHold,
    AmbientDrift,
}

impl ModSource {
    pub const ALL: [Self; 6] = [
        Self::Lfo,
        Self::AdsrEnv,
        Self::Velocity,
        Self::ModWheel,
        Self::SampleHold,
        Self::AmbientDrift,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lfo => "lfo",
            Self::AdsrEnv => "adsr_env",
            Self::Velocity => "velocity",
            Self::ModWheel => "mod_wheel",
            Self::SampleHold => "sample_hold",
            Self::AmbientDrift => "ambient_drift",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Lfo => "LFO",
            Self::AdsrEnv => "ADSR Env",
            Self::Velocity => "Velocity",
            Self::ModWheel => "Mod Wheel",
            Self::SampleHold => "S&H CV",
            Self::AmbientDrift => "Ambient Drift",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "lfo" => Some(Self::Lfo),
            "adsr_env" | "adsr" | "env" => Some(Self::AdsrEnv),
            "velocity" | "vel" => Some(Self::Velocity),
            "mod_wheel" | "modwheel" | "cc1" => Some(Self::ModWheel),
            "sample_hold" | "sh" | "s&h" => Some(Self::SampleHold),
            "ambient_drift" | "drift" | "ambient" => Some(Self::AmbientDrift),
            _ => None,
        }
    }
}

/// Modulation target parameter inside the real-time engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModDest {
    Cutoff,
    Resonance,
    FmIndex,
    PulseWidth,
    Vco2Detune,
    Wavefold,
    StereoPan,
    ExtremeMorph,
    ReverbMix,
}

impl ModDest {
    pub const ALL: [Self; 9] = [
        Self::Cutoff,
        Self::Resonance,
        Self::FmIndex,
        Self::PulseWidth,
        Self::Vco2Detune,
        Self::Wavefold,
        Self::StereoPan,
        Self::ExtremeMorph,
        Self::ReverbMix,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cutoff => "cutoff",
            Self::Resonance => "resonance",
            Self::FmIndex => "fm_index",
            Self::PulseWidth => "pulse_width",
            Self::Vco2Detune => "vco2_detune",
            Self::Wavefold => "wavefold",
            Self::StereoPan => "stereo_pan",
            Self::ExtremeMorph => "extreme_morph",
            Self::ReverbMix => "reverb_mix",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Cutoff => "VCF Cutoff",
            Self::Resonance => "VCF Resonance",
            Self::FmIndex => "FM Index",
            Self::PulseWidth => "Pulse Width",
            Self::Vco2Detune => "VCO2 Detune",
            Self::Wavefold => "Wavefolder",
            Self::StereoPan => "Stereo Pan",
            Self::ExtremeMorph => "X-DSP Morph",
            Self::ReverbMix => "Reverb Mix",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "cutoff" | "cutoff_hz" => Some(Self::Cutoff),
            "resonance" | "resonance_q" | "res" => Some(Self::Resonance),
            "fm_index" | "fm" => Some(Self::FmIndex),
            "pulse_width" | "pwm" => Some(Self::PulseWidth),
            "vco2_detune" | "detune" => Some(Self::Vco2Detune),
            "wavefold" | "fold" => Some(Self::Wavefold),
            "stereo_pan" | "pan" => Some(Self::StereoPan),
            "extreme_morph" | "morph" => Some(Self::ExtremeMorph),
            "reverb_mix" | "reverb" => Some(Self::ReverbMix),
            _ => None,
        }
    }
}

/// One modulation routing slot (persisted in `SynthPatch::mod_matrix`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ModRoute {
    pub source: ModSource,
    pub dest: ModDest,
    /// Bipolar modulation depth in `-1.0..=1.0`.
    pub depth: f32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Instantaneous per-voice source values fed into the matrix evaluator.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModInputs {
    pub lfo: f32,
    pub env: f32,
    pub vel: f32,
    pub mod_wheel: f32,
    pub sh: f32,
    pub drift: f32,
}

impl ModInputs {
    #[inline]
    pub fn get(self, src: ModSource) -> f32 {
        match src {
            ModSource::Lfo => self.lfo,
            ModSource::AdsrEnv => self.env,
            ModSource::Velocity => self.vel,
            ModSource::ModWheel => self.mod_wheel,
            ModSource::SampleHold => self.sh,
            ModSource::AmbientDrift => self.drift,
        }
    }
}

/// Summed modulation offsets for one voice sample.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModOutputs {
    pub cutoff_oct: f32,
    pub resonance_q: f32,
    pub fm_index: f32,
    pub pulse_width: f32,
    pub detune_cents: f32,
    pub wavefold: f32,
    pub pan: f32,
    pub morph: f32,
    pub reverb_mix: f32,
}

/// Evaluate up to 8 modulation routes with zero allocations.
#[inline]
pub fn evaluate_mod_matrix(routes: &[ModRoute], inputs: ModInputs) -> ModOutputs {
    let mut out = ModOutputs::default();
    for r in routes.iter().take(8) {
        if !r.enabled || r.depth.abs() < 1e-5 {
            continue;
        }
        let v = inputs.get(r.source) * r.depth.clamp(-1.0, 1.0);
        match r.dest {
            ModDest::Cutoff => out.cutoff_oct += v * 3.0,
            ModDest::Resonance => out.resonance_q += v * 6.0,
            ModDest::FmIndex => out.fm_index += v * 2.0,
            ModDest::PulseWidth => out.pulse_width += v * 0.4,
            ModDest::Vco2Detune => out.detune_cents += v * 50.0,
            ModDest::Wavefold => out.wavefold += v * 0.8,
            ModDest::StereoPan => out.pan += v,
            ModDest::ExtremeMorph => out.morph += v * 0.5,
            ModDest::ReverbMix => out.reverb_mix += v * 0.5,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_evaluates_bipolar_routes_and_skips_disabled() {
        let routes = [
            ModRoute {
                source: ModSource::Velocity,
                dest: ModDest::Cutoff,
                depth: 0.5,
                enabled: true,
            },
            ModRoute {
                source: ModSource::Lfo,
                dest: ModDest::StereoPan,
                depth: -1.0,
                enabled: true,
            },
            ModRoute {
                source: ModSource::AdsrEnv,
                dest: ModDest::FmIndex,
                depth: 1.0,
                enabled: false,
            },
        ];
        let out = evaluate_mod_matrix(
            &routes,
            ModInputs {
                lfo: 0.6,
                env: 1.0,
                vel: 0.8,
                ..Default::default()
            },
        );
        assert!((out.cutoff_oct - 1.2).abs() < 1e-4);
        assert!((out.pan + 0.6).abs() < 1e-4);
        assert_eq!(out.fm_index, 0.0);
    }
}
