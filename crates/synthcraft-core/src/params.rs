//! Host-facing parameter table shared by the CLAP plugin, the C ABI and the VCV Rack module.
//!
//! Ids are stable: saved plugin states and host automation refer to them. New parameters get
//! new ids at the end; ids are never reused. The first ten keep the ids of the CLAP descriptor
//! that `export_clap_descriptor` has always published.

use crate::model::SynthPatch;

/// One automatable parameter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamDef {
    pub id: u32,
    /// Stable machine key, also used in the Rack module and saved states.
    pub key: &'static str,
    pub name: &'static str,
    pub module: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

macro_rules! p {
    ($id:expr, $key:expr, $name:expr, $module:expr, $min:expr, $max:expr, $def:expr) => {
        ParamDef {
            id: $id,
            key: $key,
            name: $name,
            module: $module,
            min: $min,
            max: $max,
            default: $def,
        }
    };
}

/// Every host-visible parameter, in display order.
pub const PARAMS: &[ParamDef] = &[
    p!(
        0x1001,
        "vco2_detune_cents",
        "VCO2 Detune (cents)",
        "vco",
        -100.0,
        100.0,
        12.0
    ),
    p!(0x1002, "vco_mix", "VCO 1/2 Mix", "vco", 0.0, 1.0, 0.5),
    p!(
        0x1003,
        "fm_index",
        "FM Modulation Index",
        "vco",
        0.0,
        4.0,
        0.2
    ),
    p!(
        0x1004,
        "cutoff_hz",
        "VCF Cutoff (Hz)",
        "vcf",
        35.0,
        16000.0,
        1200.0
    ),
    p!(
        0x1005,
        "resonance_q",
        "VCF Resonance (Q)",
        "vcf",
        0.5,
        12.0,
        3.5
    ),
    p!(
        0x1006,
        "env_mod_depth",
        "VCF Envelope Depth",
        "vcf",
        0.0,
        1.0,
        0.65
    ),
    p!(0x1007, "attack_ms", "Attack (ms)", "env", 1.0, 2000.0, 8.0),
    p!(0x1008, "decay_ms", "Decay (ms)", "env", 5.0, 4000.0, 180.0),
    p!(
        0x1009,
        "sustain_level",
        "Sustain Level",
        "env",
        0.0,
        1.0,
        0.5
    ),
    p!(
        0x100A,
        "lfo_rate_hz",
        "LFO Rate (Hz)",
        "lfo",
        0.1,
        30.0,
        2.0
    ),
    p!(
        0x100B,
        "release_ms",
        "Release (ms)",
        "env",
        5.0,
        8000.0,
        220.0
    ),
    p!(0x100C, "lfo_depth", "LFO Depth", "lfo", 0.0, 1.0, 0.2),
    p!(0x100D, "drive_saturation", "Drive", "out", 0.0, 3.0, 0.0),
    p!(0x100E, "master_gain", "Master Gain", "out", 0.0, 1.0, 0.7),
    p!(0x100F, "pulse_width", "Pulse Width", "vco", 0.05, 0.95, 0.5),
    p!(
        0x1010,
        "m1_layer_mix",
        "PCM Layer Mix",
        "pcm",
        0.0,
        1.0,
        0.55
    ),
    p!(
        0x1011,
        "extreme_morph",
        "Extreme DSP Morph",
        "pcm",
        0.0,
        1.0,
        0.55
    ),
    p!(
        0x1012,
        "sampler_mix",
        "Sampler Mix",
        "sampler",
        0.0,
        1.0,
        0.8
    ),
    p!(0x1013, "chorus_mix", "Chorus Mix", "fx", 0.0, 1.0, 0.0),
    p!(0x1014, "reverb_mix", "Reverb Mix", "fx", 0.0, 1.0, 0.0),
    p!(
        0x1015,
        "reverb_decay_s",
        "Reverb Decay (s)",
        "fx",
        0.2,
        20.0,
        3.0
    ),
    p!(
        0x1016,
        "reverb_damping",
        "Reverb Damping",
        "fx",
        0.0,
        0.95,
        0.5
    ),
];

/// Look a parameter up by id.
pub fn param_def(id: u32) -> Option<&'static ParamDef> {
    PARAMS.iter().find(|d| d.id == id)
}

/// Look a parameter up by its stable key.
pub fn param_by_key(key: &str) -> Option<&'static ParamDef> {
    PARAMS.iter().find(|d| d.key == key)
}

/// Read a parameter from a patch.
pub fn get_param(patch: &SynthPatch, id: u32) -> Option<f32> {
    Some(match id {
        0x1001 => patch.vco2_detune_cents,
        0x1002 => patch.vco_mix,
        0x1003 => patch.fm_index,
        0x1004 => patch.cutoff_hz,
        0x1005 => patch.resonance_q,
        0x1006 => patch.env_mod_depth,
        0x1007 => patch.attack_ms,
        0x1008 => patch.decay_ms,
        0x1009 => patch.sustain_level,
        0x100A => patch.lfo_rate_hz,
        0x100B => patch.release_ms,
        0x100C => patch.lfo_depth,
        0x100D => patch.drive_saturation,
        0x100E => patch.master_gain,
        0x100F => patch.pulse_width,
        0x1010 => patch.m1_layer_mix,
        0x1011 => patch.extreme_morph,
        0x1012 => patch.sampler.mix,
        0x1013 => patch.fx.chorus_mix,
        0x1014 => patch.fx.reverb_mix,
        0x1015 => patch.fx.reverb_decay_s,
        0x1016 => patch.fx.reverb_damping,
        _ => return None,
    })
}

/// Write a parameter into a patch, clamped to its range. Returns the stored value, or `None`
/// for an unknown id or a non-finite value. Does not allocate.
pub fn set_param(patch: &mut SynthPatch, id: u32, value: f32) -> Option<f32> {
    let def = param_def(id)?;
    if !value.is_finite() {
        return None;
    }
    let v = value.clamp(def.min, def.max);
    match id {
        0x1001 => patch.vco2_detune_cents = v,
        0x1002 => patch.vco_mix = v,
        0x1003 => patch.fm_index = v,
        0x1004 => patch.cutoff_hz = v,
        0x1005 => patch.resonance_q = v,
        0x1006 => patch.env_mod_depth = v,
        0x1007 => patch.attack_ms = v,
        0x1008 => patch.decay_ms = v,
        0x1009 => patch.sustain_level = v,
        0x100A => patch.lfo_rate_hz = v,
        0x100B => patch.release_ms = v,
        0x100C => patch.lfo_depth = v,
        0x100D => patch.drive_saturation = v,
        0x100E => patch.master_gain = v,
        0x100F => patch.pulse_width = v,
        0x1010 => patch.m1_layer_mix = v,
        0x1011 => patch.extreme_morph = v,
        0x1012 => patch.sampler.mix = v,
        0x1013 => patch.fx.chorus_mix = v,
        0x1014 => patch.fx.reverb_mix = v,
        0x1015 => patch.fx.reverb_decay_s = v,
        _ => patch.fx.reverb_damping = v,
    }
    Some(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SynthEngine;

    #[test]
    fn ids_are_unique_and_ranges_sane() {
        for (i, a) in PARAMS.iter().enumerate() {
            assert!(a.min < a.max, "{}", a.key);
            assert!(a.default >= a.min && a.default <= a.max, "{}", a.key);
            for b in &PARAMS[i + 1..] {
                assert_ne!(a.id, b.id);
                assert_ne!(a.key, b.key);
            }
        }
    }

    #[test]
    fn every_param_round_trips_and_clamps() {
        let mut patch = SynthEngine::factory_seed().active_patch().clone();
        for d in PARAMS {
            let mid = (d.min + d.max) * 0.5;
            assert_eq!(set_param(&mut patch, d.id, mid), Some(mid), "{}", d.key);
            assert_eq!(get_param(&patch, d.id), Some(mid), "{}", d.key);
            assert_eq!(set_param(&mut patch, d.id, d.max + 1000.0), Some(d.max));
            assert_eq!(set_param(&mut patch, d.id, d.min - 1000.0), Some(d.min));
            assert_eq!(set_param(&mut patch, d.id, f32::NAN), None);
        }
        assert_eq!(set_param(&mut patch, 0xDEAD, 1.0), None);
        assert_eq!(get_param(&patch, 0xDEAD), None);
        assert_eq!(param_by_key("cutoff_hz").map(|d| d.id), Some(0x1004));
    }
}
