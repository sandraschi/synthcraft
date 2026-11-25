//! Twelve curated, ready-to-play 3U Eurorack modular rigs with properly patched cables,
//! tuned expansion modules, and matching voice settings.

use crate::model::{FilterMode, WaveformKind};
use crate::rack::{default_rack_modules, RackModuleInstance, RackModuleKind};
use crate::rompler::{default_rack_cables, ExtremeDspMode, PcmProgram, RackCable};
use crate::{SynthEngine, SynthError};
use serde::{Deserialize, Serialize};

/// Metadata describing one curated 3U modular rack preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RackPresetInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub style: &'static str,
    pub description: &'static str,
}

pub const RACK_PRESETS: &[RackPresetInfo] = &[
    RackPresetInfo {
        id: "berlin_school_sequence",
        name: "1. Berlin School Hypnotic Echo",
        style: "Classic Modular",
        description: "Resonant dual-saw ladder filter with rhythmic stereo ping-pong tape delay.",
    },
    RackPresetInfo {
        id: "west_coast_wavefolder",
        name: "2. West-Coast Wavefolder Pluck",
        style: "West-Coast",
        description: "Sine oscillator driven through heavy trigonometric wavefolding + S&H FM shimmer.",
    },
    RackPresetInfo {
        id: "acid_squelch_303",
        name: "3. Acid Squelch & Euclidean Rig",
        style: "Acid / Techno",
        description: "AcidResonant ladder filter with ADSR cross-patched to Cutoff CV + high drive.",
    },
    RackPresetInfo {
        id: "random_robot_bleeps",
        name: "4. Random S&H Computer Lab",
        style: "Generative",
        description: "Fast stepped Sample & Hold patched into both Cutoff CV and VCO-1 FM CV + echo.",
    },
    RackPresetInfo {
        id: "ambient_cathedral_drift",
        name: "5. Drifting Ambient Cathedral",
        style: "Ambient / Pad",
        description: "AtmoPad + ultra-slow slewed S&H filter drift, long ping-pong delay, and deep reverb.",
    },
    RackPresetInfo {
        id: "industrial_filter_bypass",
        name: "6. Industrial Wavefolder Bypass",
        style: "Industrial",
        description: "Unplugs VCF->VCA and patches VCO-1 straight to VCA through maxed wavefolder + drive.",
    },
    RackPresetInfo {
        id: "dub_techno_chord_echo",
        name: "7. Dub Techno Chord & Tape Echo",
        style: "Dub Techno",
        description: "Detuned SuperSaw into warm LP24 ladder filter feeding lush cross-feedback stereo delay.",
    },
    RackPresetInfo {
        id: "fm_metallic_bells",
        name: "8. FM Metallic Bells & Euclidean",
        style: "FM / Percussive",
        description: "High-index VCO-2 FM modulation + wavefolder harmonics + BandPass shimmer + delay.",
    },
    RackPresetInfo {
        id: "sub_earthquake_reese",
        name: "9. Sub-Bass Reese & Attenuverter",
        style: "Bass / DnB",
        description: "SubOctave + detuned Saw with inverted Attenuverter LFO filter sweep and saturation.",
    },
    RackPresetInfo {
        id: "pcm_shimmer_workstation",
        name: "10. 1990 PCM Shimmer & Drift",
        style: "Vintage PCM",
        description: "Synthesized PCM Universe/Choir layered through VCF AUX IN + slow S&H + stereo delay.",
    },
    RackPresetInfo {
        id: "polyrhythmic_glitch_lab",
        name: "11. Polyrhythmic Glitch & S&H",
        style: "Experimental",
        description: "Euclidean E(11,16) + dual-patched S&H + wavefolder + tight metallic slapback delay.",
    },
    RackPresetInfo {
        id: "clean_studio_init",
        name: "12. Clean Studio Reference Rig",
        style: "Utility / Init",
        description: "Balanced 2-row default layout with clean subtractive routing and neutral FX.",
    },
];

pub fn list_rack_presets() -> &'static [RackPresetInfo] {
    RACK_PRESETS
}

fn mod_inst(
    id: &str,
    kind: RackModuleKind,
    row: usize,
    param_a: f32,
    param_b: f32,
) -> RackModuleInstance {
    RackModuleInstance {
        id: id.into(),
        kind,
        row,
        bypassed: false,
        param_a,
        param_b,
    }
}

fn cable(
    id: &str,
    from_m: &str,
    from_j: &str,
    to_m: &str,
    to_j: &str,
    color: &str,
    gain: f32,
) -> RackCable {
    RackCable {
        id: id.into(),
        from_module: from_m.into(),
        from_jack: from_j.into(),
        to_module: to_m.into(),
        to_jack: to_j.into(),
        color_name: color.into(),
        gain,
    }
}

fn core_row_0() -> Vec<RackModuleInstance> {
    vec![
        mod_inst("VCO-1", RackModuleKind::Vco1, 0, 0.5, 0.5),
        mod_inst("VCO-2 / X-DSP", RackModuleKind::Vco2Xdsp, 0, 0.5, 0.5),
        mod_inst(
            "PCM-WORKSTATION",
            RackModuleKind::PcmWorkstation,
            0,
            0.5,
            0.5,
        ),
        mod_inst("VCF-LADDER", RackModuleKind::VcfLadder, 0, 0.5, 0.5),
        mod_inst("LFO / ADSR", RackModuleKind::LfoAdsr, 0, 0.5, 0.5),
        mod_inst("VCA-MASTER", RackModuleKind::VcaMaster, 0, 0.5, 0.5),
    ]
}

/// Apply one of the 12 curated modular rack rigs (modules, cables, and active patch voicing).
pub fn apply_rack_preset(
    engine: &mut SynthEngine,
    preset_id: &str,
) -> Result<&'static RackPresetInfo, SynthError> {
    let Some(info) = RACK_PRESETS.iter().find(|p| p.id == preset_id) else {
        return Err(SynthError::NotFound(format!("Rack preset '{preset_id}'")));
    };

    let mut mods = core_row_0();
    let mut cabs = default_rack_cables();
    let p = engine.active_patch_mut();

    match preset_id {
        "berlin_school_sequence" => {
            mods.push(mod_inst(
                "DELAY-1",
                RackModuleKind::StereoDelay,
                1,
                0.42,
                0.62,
            ));
            mods.push(mod_inst(
                "EUC-1",
                RackModuleKind::EuclideanClock,
                1,
                0.5,
                0.5,
            ));
            mods.push(mod_inst(
                "SCOPE-1",
                RackModuleKind::InlineScope,
                1,
                0.5,
                0.5,
            ));
            cabs.push(cable(
                "cab_vcf_dly",
                "VCF-LADDER",
                "LP24 OUT",
                "DELAY-1",
                "AUDIO IN",
                "cyan",
                1.0,
            ));
            cabs.push(cable(
                "cab_dly_scope",
                "DELAY-1",
                "DLY OUT",
                "SCOPE-1",
                "SIG IN",
                "amber",
                1.0,
            ));
            p.vco1_wave = WaveformKind::Saw;
            p.vco2_wave = WaveformKind::Saw;
            p.vco2_detune_cents = 14.0;
            p.vco_mix = 0.45;
            p.fm_index = 0.1;
            p.filter_mode = FilterMode::LowPass24dB;
            p.cutoff_hz = 1350.0;
            p.resonance_q = 4.2;
            p.env_mod_depth = 0.72;
            p.attack_ms = 6.0;
            p.decay_ms = 240.0;
            p.sustain_level = 0.35;
            p.release_ms = 320.0;
            p.lfo_rate_hz = 0.45;
            p.lfo_depth = 0.32;
            p.drive_saturation = 0.35;
            p.m1_program = None;
            p.extreme_dsp = ExtremeDspMode::Off;
            p.fx.chorus_mix = 0.25;
            p.fx.reverb_mix = 0.32;
            p.fx.reverb_decay_s = 3.8;
            p.fx.stereo_spread = 0.45;
        }
        "west_coast_wavefolder" => {
            mods.push(mod_inst(
                "WAVEFOLDER-1",
                RackModuleKind::WavefolderLpg,
                1,
                0.82,
                0.68,
            ));
            mods.push(mod_inst(
                "SH-1",
                RackModuleKind::SampleHoldSlew,
                1,
                0.32,
                0.18,
            ));
            mods.push(mod_inst(
                "DELAY-1",
                RackModuleKind::StereoDelay,
                1,
                0.30,
                0.48,
            ));
            mods.push(mod_inst(
                "SCOPE-1",
                RackModuleKind::InlineScope,
                1,
                0.5,
                0.5,
            ));
            cabs.push(cable(
                "cab_sh_fm",
                "SH-1",
                "S&H OUT",
                "VCO-1",
                "FM CV",
                "purple",
                0.65,
            ));
            cabs.push(cable(
                "cab_wf_in",
                "VCO-1",
                "SAW OUT",
                "WAVEFOLDER-1",
                "AUDIO IN",
                "amber",
                1.0,
            ));
            cabs.push(cable(
                "cab_dly_in",
                "VCF-LADDER",
                "LP24 OUT",
                "DELAY-1",
                "AUDIO IN",
                "cyan",
                1.0,
            ));
            p.vco1_wave = WaveformKind::Sine;
            p.vco2_wave = WaveformKind::Triangle;
            p.vco2_detune_cents = 5.0;
            p.vco_mix = 0.30;
            p.fm_index = 0.85;
            p.filter_mode = FilterMode::LowPass24dB;
            p.cutoff_hz = 4200.0;
            p.resonance_q = 2.0;
            p.env_mod_depth = 0.60;
            p.attack_ms = 3.0;
            p.decay_ms = 260.0;
            p.sustain_level = 0.15;
            p.release_ms = 420.0;
            p.drive_saturation = 0.25;
            p.m1_program = None;
            p.extreme_dsp = ExtremeDspMode::WestCoastWavefolder;
            p.extreme_morph = 0.55;
            p.fx.chorus_mix = 0.2;
            p.fx.reverb_mix = 0.38;
        }
        "acid_squelch_303" => {
            mods.push(mod_inst(
                "EUC-1",
                RackModuleKind::EuclideanClock,
                1,
                1.0,
                0.32,
            ));
            mods.push(mod_inst(
                "WAVEFOLDER-1",
                RackModuleKind::WavefolderLpg,
                1,
                0.22,
                0.55,
            ));
            mods.push(mod_inst(
                "SCOPE-1",
                RackModuleKind::InlineScope,
                1,
                0.5,
                0.5,
            ));
            cabs.push(cable(
                "cab_adsr_cut",
                "LFO / ADSR",
                "ENV OUT",
                "VCF-LADDER",
                "CUTOFF CV",
                "red",
                0.85,
            ));
            p.vco1_wave = WaveformKind::Saw;
            p.vco2_wave = WaveformKind::Square;
            p.vco_mix = 0.15;
            p.fm_index = 0.0;
            p.filter_mode = FilterMode::AcidResonant;
            p.cutoff_hz = 480.0;
            p.resonance_q = 8.8;
            p.env_mod_depth = 0.90;
            p.attack_ms = 2.0;
            p.decay_ms = 190.0;
            p.sustain_level = 0.10;
            p.release_ms = 120.0;
            p.lfo_rate_hz = 1.5;
            p.lfo_depth = 0.15;
            p.drive_saturation = 1.65;
            p.m1_program = None;
            p.extreme_dsp = ExtremeDspMode::Off;
            p.fx.chorus_mix = 0.0;
            p.fx.reverb_mix = 0.15;
        }
        "random_robot_bleeps" => {
            mods.push(mod_inst(
                "SH-1",
                RackModuleKind::SampleHoldSlew,
                1,
                0.72,
                0.0,
            ));
            mods.push(mod_inst(
                "DELAY-1",
                RackModuleKind::StereoDelay,
                1,
                0.24,
                0.65,
            ));
            mods.push(mod_inst(
                "EUC-1",
                RackModuleKind::EuclideanClock,
                1,
                0.75,
                0.45,
            ));
            mods.push(mod_inst(
                "SCOPE-1",
                RackModuleKind::InlineScope,
                1,
                0.5,
                0.5,
            ));
            cabs.push(cable(
                "cab_sh_cut",
                "SH-1",
                "S&H OUT",
                "VCF-LADDER",
                "CUTOFF CV",
                "amber",
                1.35,
            ));
            cabs.push(cable(
                "cab_sh_fm",
                "SH-1",
                "S&H OUT",
                "VCO-1",
                "FM CV",
                "purple",
                1.25,
            ));
            cabs.push(cable(
                "cab_dly_in",
                "VCF-LADDER",
                "LP24 OUT",
                "DELAY-1",
                "AUDIO IN",
                "cyan",
                1.0,
            ));
            p.vco1_wave = WaveformKind::Square;
            p.vco2_wave = WaveformKind::Sine;
            p.vco_mix = 0.25;
            p.fm_index = 1.4;
            p.filter_mode = FilterMode::BandPass12dB;
            p.cutoff_hz = 1400.0;
            p.resonance_q = 6.5;
            p.attack_ms = 2.0;
            p.decay_ms = 140.0;
            p.sustain_level = 0.45;
            p.release_ms = 180.0;
            p.drive_saturation = 0.4;
            p.m1_program = None;
            p.extreme_dsp = ExtremeDspMode::Off;
            p.fx.reverb_mix = 0.35;
        }
        "ambient_cathedral_drift" => {
            mods.push(mod_inst(
                "SH-1",
                RackModuleKind::SampleHoldSlew,
                1,
                0.08,
                0.92,
            ));
            mods.push(mod_inst(
                "ATTEN-1",
                RackModuleKind::Attenuverter,
                1,
                0.72,
                0.5,
            ));
            mods.push(mod_inst(
                "DELAY-1",
                RackModuleKind::StereoDelay,
                1,
                0.72,
                0.72,
            ));
            mods.push(mod_inst(
                "SCOPE-1",
                RackModuleKind::InlineScope,
                1,
                0.5,
                0.5,
            ));
            cabs.push(cable(
                "cab_sh_cut",
                "SH-1",
                "S&H OUT",
                "VCF-LADDER",
                "CUTOFF CV",
                "purple",
                0.95,
            ));
            cabs.push(cable(
                "cab_dly_in",
                "VCF-LADDER",
                "LP24 OUT",
                "DELAY-1",
                "AUDIO IN",
                "cyan",
                1.0,
            ));
            p.vco1_wave = WaveformKind::AtmoPad;
            p.vco2_wave = WaveformKind::AtmoPad;
            p.vco2_detune_cents = 9.0;
            p.vco_mix = 0.5;
            p.fm_index = 0.05;
            p.filter_mode = FilterMode::LowPass24dB;
            p.cutoff_hz = 1900.0;
            p.resonance_q = 1.6;
            p.env_mod_depth = 0.22;
            p.attack_ms = 1400.0;
            p.decay_ms = 1600.0;
            p.sustain_level = 0.85;
            p.release_ms = 3200.0;
            p.lfo_rate_hz = 0.14;
            p.lfo_depth = 0.35;
            p.drive_saturation = 0.1;
            p.m1_program = None;
            p.extreme_dsp = ExtremeDspMode::Off;
            p.fx.chorus_mix = 0.65;
            p.fx.reverb_mix = 0.68;
            p.fx.reverb_decay_s = 11.0;
            p.fx.reverb_damping = 0.42;
            p.fx.stereo_spread = 0.85;
            p.fx.ambient_depth = 0.65;
            p.fx.ambient_rate_hz = 0.06;
        }
        "industrial_filter_bypass" => {
            mods.push(mod_inst(
                "WAVEFOLDER-1",
                RackModuleKind::WavefolderLpg,
                1,
                0.95,
                0.85,
            ));
            mods.push(mod_inst(
                "SH-1",
                RackModuleKind::SampleHoldSlew,
                1,
                0.55,
                0.05,
            ));
            mods.push(mod_inst(
                "SCOPE-1",
                RackModuleKind::InlineScope,
                1,
                0.5,
                0.5,
            ));
            // Unplug VCF->VCA and patch VCO-1 straight into VCA for raw unfiltered wavefolder grit!
            cabs.retain(|c| c.id != "cab_vcf_vca");
            cabs.push(cable(
                "cab_direct_vca",
                "VCO-1",
                "SAW OUT",
                "VCA-MASTER",
                "SIGNAL IN",
                "red",
                1.15,
            ));
            cabs.push(cable(
                "cab_sh_fm",
                "SH-1",
                "S&H OUT",
                "VCO-1",
                "FM CV",
                "amber",
                0.9,
            ));
            p.vco1_wave = WaveformKind::Saw;
            p.vco2_wave = WaveformKind::Square;
            p.vco2_detune_cents = -28.0;
            p.vco_mix = 0.4;
            p.fm_index = 1.9;
            p.drive_saturation = 2.6;
            p.attack_ms = 3.0;
            p.decay_ms = 220.0;
            p.sustain_level = 0.7;
            p.release_ms = 160.0;
            p.m1_program = None;
            p.extreme_dsp = ExtremeDspMode::Off;
            p.fx.reverb_mix = 0.22;
        }
        "dub_techno_chord_echo" => {
            mods.push(mod_inst(
                "DELAY-1",
                RackModuleKind::StereoDelay,
                1,
                0.48,
                0.76,
            ));
            mods.push(mod_inst(
                "ATTEN-1",
                RackModuleKind::Attenuverter,
                1,
                0.68,
                0.5,
            ));
            mods.push(mod_inst(
                "SCOPE-1",
                RackModuleKind::InlineScope,
                1,
                0.5,
                0.5,
            ));
            cabs.push(cable(
                "cab_dly_in",
                "VCF-LADDER",
                "LP24 OUT",
                "DELAY-1",
                "AUDIO IN",
                "cyan",
                1.0,
            ));
            p.vco1_wave = WaveformKind::SuperSaw;
            p.vco2_wave = WaveformKind::Saw;
            p.vco2_detune_cents = 18.0;
            p.vco_mix = 0.4;
            p.filter_mode = FilterMode::LowPass24dB;
            p.cutoff_hz = 950.0;
            p.resonance_q = 3.8;
            p.env_mod_depth = 0.65;
            p.attack_ms = 12.0;
            p.decay_ms = 280.0;
            p.sustain_level = 0.18;
            p.release_ms = 450.0;
            p.lfo_rate_hz = 0.28;
            p.lfo_depth = 0.42;
            p.drive_saturation = 0.65;
            p.m1_program = None;
            p.extreme_dsp = ExtremeDspMode::Off;
            p.fx.chorus_mix = 0.45;
            p.fx.reverb_mix = 0.52;
            p.fx.reverb_decay_s = 6.5;
            p.fx.stereo_spread = 0.65;
        }
        "fm_metallic_bells" => {
            mods.push(mod_inst(
                "WAVEFOLDER-1",
                RackModuleKind::WavefolderLpg,
                1,
                0.45,
                0.35,
            ));
            mods.push(mod_inst(
                "EUC-1",
                RackModuleKind::EuclideanClock,
                1,
                1.0,
                0.44,
            ));
            mods.push(mod_inst(
                "DELAY-1",
                RackModuleKind::StereoDelay,
                1,
                0.36,
                0.58,
            ));
            cabs.push(cable(
                "cab_dly_in",
                "VCF-LADDER",
                "LP24 OUT",
                "DELAY-1",
                "AUDIO IN",
                "purple",
                1.0,
            ));
            p.vco1_wave = WaveformKind::Sine;
            p.vco2_wave = WaveformKind::Sine;
            p.vco2_detune_cents = 42.0;
            p.vco_mix = 0.2;
            p.fm_index = 2.8;
            p.filter_mode = FilterMode::BandPass12dB;
            p.cutoff_hz = 2600.0;
            p.resonance_q = 3.2;
            p.env_mod_depth = 0.55;
            p.attack_ms = 2.0;
            p.decay_ms = 650.0;
            p.sustain_level = 0.08;
            p.release_ms = 1400.0;
            p.drive_saturation = 0.2;
            p.m1_program = None;
            p.extreme_dsp = ExtremeDspMode::ModalStruckBar;
            p.extreme_morph = 0.6;
            p.fx.reverb_mix = 0.48;
            p.fx.reverb_decay_s = 5.5;
        }
        "sub_earthquake_reese" => {
            mods.push(mod_inst(
                "ATTEN-1",
                RackModuleKind::Attenuverter,
                1,
                0.15,
                0.5,
            ));
            mods.push(mod_inst(
                "WAVEFOLDER-1",
                RackModuleKind::WavefolderLpg,
                1,
                0.18,
                0.5,
            ));
            mods.push(mod_inst(
                "SCOPE-1",
                RackModuleKind::InlineScope,
                1,
                0.5,
                0.5,
            ));
            p.vco1_wave = WaveformKind::SubOctave;
            p.vco2_wave = WaveformKind::Saw;
            p.vco2_detune_cents = 24.0;
            p.vco_mix = 0.55;
            p.fm_index = 0.15;
            p.filter_mode = FilterMode::LowPass24dB;
            p.cutoff_hz = 520.0;
            p.resonance_q = 4.6;
            p.env_mod_depth = 0.45;
            p.attack_ms = 18.0;
            p.decay_ms = 520.0;
            p.sustain_level = 0.78;
            p.release_ms = 340.0;
            p.lfo_rate_hz = 0.85;
            p.lfo_depth = 0.55;
            p.drive_saturation = 1.45;
            p.m1_program = None;
            p.extreme_dsp = ExtremeDspMode::Off;
            p.fx.chorus_mix = 0.35;
            p.fx.reverb_mix = 0.12;
        }
        "pcm_shimmer_workstation" => {
            mods.push(mod_inst(
                "SH-1",
                RackModuleKind::SampleHoldSlew,
                1,
                0.14,
                0.85,
            ));
            mods.push(mod_inst(
                "DELAY-1",
                RackModuleKind::StereoDelay,
                1,
                0.55,
                0.60,
            ));
            mods.push(mod_inst(
                "SCOPE-1",
                RackModuleKind::InlineScope,
                1,
                0.5,
                0.5,
            ));
            cabs.push(cable(
                "cab_sh_cut",
                "SH-1",
                "S&H OUT",
                "VCF-LADDER",
                "CUTOFF CV",
                "purple",
                0.7,
            ));
            cabs.push(cable(
                "cab_dly_in",
                "VCF-LADDER",
                "LP24 OUT",
                "DELAY-1",
                "AUDIO IN",
                "cyan",
                1.0,
            ));
            p.vco1_wave = WaveformKind::AtmoPad;
            p.vco2_wave = WaveformKind::Triangle;
            p.vco_mix = 0.35;
            p.m1_program =
                PcmProgram::parse("m1_00_universe").or_else(|| PcmProgram::all().first().copied());
            p.m1_layer_mix = 0.72;
            p.extreme_dsp = ExtremeDspMode::GranularCloud;
            p.extreme_morph = 0.48;
            p.filter_mode = FilterMode::LowPass24dB;
            p.cutoff_hz = 3200.0;
            p.resonance_q = 1.8;
            p.attack_ms = 420.0;
            p.decay_ms = 900.0;
            p.sustain_level = 0.75;
            p.release_ms = 1800.0;
            p.fx.chorus_mix = 0.55;
            p.fx.reverb_mix = 0.55;
            p.fx.reverb_decay_s = 7.0;
        }
        "polyrhythmic_glitch_lab" => {
            mods.push(mod_inst(
                "EUC-1",
                RackModuleKind::EuclideanClock,
                1,
                1.0,
                0.69,
            ));
            mods.push(mod_inst(
                "SH-1",
                RackModuleKind::SampleHoldSlew,
                1,
                0.88,
                0.04,
            ));
            mods.push(mod_inst(
                "WAVEFOLDER-1",
                RackModuleKind::WavefolderLpg,
                1,
                0.72,
                0.75,
            ));
            mods.push(mod_inst(
                "DELAY-1",
                RackModuleKind::StereoDelay,
                1,
                0.10,
                0.74,
            ));
            mods.push(mod_inst(
                "SCOPE-1",
                RackModuleKind::InlineScope,
                2,
                0.5,
                0.5,
            ));
            cabs.push(cable(
                "cab_sh_cut",
                "SH-1",
                "S&H OUT",
                "VCF-LADDER",
                "CUTOFF CV",
                "amber",
                1.5,
            ));
            cabs.push(cable(
                "cab_sh_fm",
                "SH-1",
                "S&H OUT",
                "VCO-1",
                "FM CV",
                "red",
                1.4,
            ));
            cabs.push(cable(
                "cab_dly_in",
                "VCF-LADDER",
                "LP24 OUT",
                "DELAY-1",
                "AUDIO IN",
                "cyan",
                1.0,
            ));
            p.vco1_wave = WaveformKind::PulsePwm;
            p.pulse_width = 0.22;
            p.vco2_wave = WaveformKind::WhiteNoise;
            p.vco_mix = 0.18;
            p.fm_index = 2.1;
            p.filter_mode = FilterMode::AcidResonant;
            p.cutoff_hz = 1600.0;
            p.resonance_q = 7.4;
            p.attack_ms = 2.0;
            p.decay_ms = 120.0;
            p.sustain_level = 0.35;
            p.release_ms = 160.0;
            p.drive_saturation = 1.1;
            p.m1_program = None;
            p.extreme_dsp = ExtremeDspMode::Off;
            p.fx.reverb_mix = 0.28;
        }
        _ => {
            // clean_studio_init
            mods = default_rack_modules();
            cabs = default_rack_cables();
            p.vco1_wave = WaveformKind::Saw;
            p.vco2_wave = WaveformKind::Saw;
            p.vco2_detune_cents = 10.0;
            p.vco_mix = 0.5;
            p.fm_index = 0.1;
            p.filter_mode = FilterMode::LowPass24dB;
            p.cutoff_hz = 2200.0;
            p.resonance_q = 2.2;
            p.env_mod_depth = 0.45;
            p.attack_ms = 8.0;
            p.decay_ms = 240.0;
            p.sustain_level = 0.6;
            p.release_ms = 280.0;
            p.lfo_rate_hz = 1.5;
            p.lfo_depth = 0.12;
            p.drive_saturation = 0.0;
            p.m1_program = None;
            p.extreme_dsp = ExtremeDspMode::Off;
            p.fx.chorus_mix = 0.0;
            p.fx.reverb_mix = 0.15;
            p.fx.ambient_depth = 0.0;
        }
    }

    engine.rack_modules = mods;
    engine.rack_cables = cabs;
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rack::compute_rack_routing;
    use crate::SynthRt;

    #[test]
    fn all_12_rack_presets_produce_audible_bounded_stereo_audio() {
        assert_eq!(RACK_PRESETS.len(), 12);
        for info in RACK_PRESETS {
            let mut eng = SynthEngine::factory_seed();
            assert!(apply_rack_preset(&mut eng, info.id).is_ok());
            assert!(!eng.rack_modules.is_empty());
            assert!(!eng.rack_cables.is_empty());

            let routing = compute_rack_routing(&eng.rack_modules, &eng.rack_cables);
            let mut rt = SynthRt::new(48_000.0, eng.active_patch().clone());
            rt.set_routing(routing);
            rt.note_on(60, 0.9);
            let n = ((eng.active_patch().attack_ms * 48.0) as usize).max(4096) + 2048;
            let mut l = vec![0.0_f32; n];
            let mut r = vec![0.0_f32; n];
            rt.process_stereo(&mut l, &mut r);
            let peak = l
                .iter()
                .chain(r.iter())
                .map(|s| s.abs())
                .fold(0.0_f32, f32::max);
            assert!(
                peak > 0.01 && peak <= 1.0,
                "preset {} peak out of bounds: {peak}",
                info.id
            );
        }
    }
}
