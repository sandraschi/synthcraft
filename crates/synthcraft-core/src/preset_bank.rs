//! 120-Preset Pro Factory Bank, Engine Badges, and Musical Patch Mutator.
//!
//! Covers 12 genres/disciplines including Cathedral Pipe Organ, Acid & 303, Bass & Reese,
//! Leads & SuperSaw, Pads & Ambient, Keys & 1990 PCM, 6-Op FM & Bells, West-Coast Modular,
//! Physical & Acoustic (Neural DDSP / 2D FDTD), Granular & Spectral, Vocoder & Chords,
//! and Drums & Grooves.
#![forbid(unsafe_code)]

use crate::chord_mem::{ChordMode, ScaleLock};
use crate::drums::DrumMachinePatch;
use crate::fm_unison::{Fm6Params, UnisonStackMode};
use crate::fx_rack::IrPreset;
use crate::model::{FilterMode, SequencerStep, SynthPatch, WaveformKind};
use crate::neural_ddsp::DdspTimbre;
use crate::nextgen::{Filter2Mode, Filter2Routing, MsegMode};
use crate::physical::{PhysicalMaterial, WdfCircuitMode};
use crate::rompler::{ExtremeDspMode, PcmProgram};

pub const PRESET_CATEGORIES: &[&str] = &[
    "Acid & 303",
    "Bass & Reese",
    "Leads & SuperSaw",
    "Pads & Ambient",
    "Keys & 1990 PCM",
    "Cathedral Pipe Organ",
    "6-Op FM & Bells",
    "West-Coast & Modular",
    "Physical & Acoustic",
    "Granular & Spectral",
    "Vocoder & Chords",
    "Drums & Grooves",
];

/// Derive concise engine badges for the Preset Browser UI from any `SynthPatch`.
pub fn preset_engine_badges(p: &SynthPatch) -> Vec<&'static str> {
    let mut out = Vec::new();
    if p.m1_program.is_some() {
        out.push("M1");
    }
    if p.extreme_dsp != ExtremeDspMode::Off {
        out.push("XDSP");
    }
    if p.drums.enabled {
        out.push("808");
    }
    if p.mseg.mode != MsegMode::Off {
        out.push("MSEG");
    }
    if p.filter2.mode != Filter2Mode::Off {
        out.push("FLT2");
    }
    if p.granular.enabled {
        out.push("GRAN");
    }
    if p.spectral.enabled {
        out.push("SPEC");
    }
    if p.fm6.enabled {
        out.push("FM6");
    }
    if p.unison.voices > 1 {
        out.push("UNI16");
    }
    if p.studio_fx.enabled && p.studio_fx.ott_enabled {
        out.push("OTT");
    }
    if p.studio_fx.enabled && p.studio_fx.shimmer_enabled {
        out.push("SHIM");
    }
    if p.studio_fx.enabled && p.studio_fx.conv_enabled {
        out.push("IR");
    }
    if p.studio_fx.enabled && p.studio_fx.tape_enabled {
        out.push("TAPE");
    }
    if p.physical_mesh.enabled {
        out.push("FDTD");
    }
    if p.wdf_circuit.mode != WdfCircuitMode::Off {
        out.push("WDF");
    }
    if p.ddsp.enabled {
        out.push("DDSP");
    }
    if p.chord_mem.chord_mode != ChordMode::Off {
        out.push("CHRD");
    }
    if p.vocoder.enabled {
        out.push("VOC");
    }
    if p.mastering.enabled {
        out.push("MSTR");
    }
    if out.is_empty() {
        out.push("VA");
    }
    out
}

/// Deterministic musical patch mutator (+/- `amount` in `0.02..=0.50`), keeping all parameters bounded.
pub fn mutate_patch(src: &SynthPatch, amount: f32, seed: u64) -> SynthPatch {
    let amt = amount.clamp(0.02, 0.50);
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut next_bipolar = || -> f32 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let u = (state as f32) / (u64::MAX as f32);
        (u * 2.0 - 1.0) * amt
    };

    let mut p = src.clone();
    p.cutoff_hz = (p.cutoff_hz * (1.0 + next_bipolar() * 1.4)).clamp(90.0, 16_000.0);
    p.resonance_q = (p.resonance_q * (1.0 + next_bipolar())).clamp(0.5, 11.5);
    p.vco2_detune_cents = (p.vco2_detune_cents + next_bipolar() * 28.0).clamp(-60.0, 60.0);
    p.vco_mix = (p.vco_mix + next_bipolar() * 0.5).clamp(0.0, 1.0);
    p.fm_index = (p.fm_index + next_bipolar() * 0.8).clamp(0.0, 4.0);
    p.env_mod_depth = (p.env_mod_depth + next_bipolar() * 0.5).clamp(-1.0, 1.0);
    p.attack_ms = (p.attack_ms * (1.0 + next_bipolar())).clamp(1.0, 2500.0);
    p.decay_ms = (p.decay_ms * (1.0 + next_bipolar())).clamp(20.0, 4000.0);
    p.release_ms = (p.release_ms * (1.0 + next_bipolar())).clamp(20.0, 5000.0);
    p.drive_saturation = (p.drive_saturation + next_bipolar() * 0.7).clamp(0.5, 4.5);
    p.fx.chorus_mix = (p.fx.chorus_mix + next_bipolar() * 0.35).clamp(0.0, 0.85);
    p.fx.reverb_mix = (p.fx.reverb_mix + next_bipolar() * 0.35).clamp(0.0, 0.85);
    if p.filter2.mode != Filter2Mode::Off {
        p.filter2.cutoff_hz = (p.filter2.cutoff_hz * (1.0 + next_bipolar())).clamp(120.0, 14_000.0);
        p.filter2.vowel_morph = (p.filter2.vowel_morph + next_bipolar() * 0.6).clamp(0.0, 1.0);
    }
    p
}

fn seq(notes: &[u8]) -> Vec<SequencerStep> {
    notes
        .iter()
        .enumerate()
        .map(|(i, &n)| SequencerStep {
            step_index: i,
            midi_note: n,
            velocity: if i % 4 == 0 { 0.94 } else { 0.78 },
            gate_pct: 72.0,
            accent: i % 4 == 0 || i % 7 == 3,
            slide: i % 5 == 3,
            active: true,
        })
        .collect()
}

fn base(
    template: &SynthPatch,
    id: &str,
    name: &str,
    category: &str,
    bpm: f32,
    key: &str,
) -> SynthPatch {
    let mut p = template.clone();
    p.id = id.to_string();
    p.name = name.to_string();
    p.category = category.to_string();
    p.bpm = bpm;
    p.camelot_key = key.to_string();
    p.m1_program = None;
    p.extreme_dsp = ExtremeDspMode::Off;
    p.drums = DrumMachinePatch::default();
    p.mseg.mode = MsegMode::Off;
    p.filter2.mode = Filter2Mode::Off;
    p.granular.enabled = false;
    p.spectral.enabled = false;
    p.fm6 = Fm6Params::default();
    p.unison.voices = 1;
    p.studio_fx.enabled = false;
    p.studio_fx.ott_enabled = false;
    p.studio_fx.conv_enabled = false;
    p.studio_fx.shimmer_enabled = false;
    p.studio_fx.tape_enabled = false;
    p.physical_mesh.enabled = false;
    p.wdf_circuit.mode = WdfCircuitMode::Off;
    p.ddsp.enabled = false;
    p.chord_mem.chord_mode = ChordMode::Off;
    p.vocoder.enabled = false;
    p.mastering.enabled = true;
    p
}

/// Build the 120-preset Pro Factory Bank (10 presets x 12 categories).
pub fn pro_bank_patches(template: &SynthPatch) -> Vec<SynthPatch> {
    let mut out = Vec::with_capacity(120);

    // 1. Acid & 303 (10)
    let acid_specs = [
        (
            "acid_tb303_diode",
            "TB-303 Diode Squelch",
            132.0,
            "8A Am",
            540.0,
            8.6,
            true,
            false,
        ),
        (
            "acid_chicago_87",
            "Chicago 1987 Phuture",
            126.0,
            "4A Fm",
            620.0,
            9.2,
            true,
            false,
        ),
        (
            "acid_ms20_scream",
            "MS-20 Screaming Acid",
            138.0,
            "9A Em",
            890.0,
            7.5,
            false,
            true,
        ),
        (
            "acid_berlin_bunker",
            "Berlin Bunker 303",
            140.0,
            "2A Ebm",
            480.0,
            9.5,
            true,
            true,
        ),
        (
            "acid_liquid_silver",
            "Liquid Silver Line",
            130.0,
            "7A Dm",
            1100.0,
            6.8,
            true,
            false,
        ),
        (
            "acid_rubber_sub",
            "Rubber Sub Acid",
            128.0,
            "5A Cm",
            390.0,
            7.2,
            true,
            false,
        ),
        (
            "acid_hypnotic_4bar",
            "Hypnotic 4-Bar Acid",
            134.0,
            "6A Gm",
            760.0,
            8.0,
            true,
            false,
        ),
        (
            "acid_overdrive_cp3",
            "CP3 Overdriven Acid",
            136.0,
            "10A Bm",
            670.0,
            8.4,
            true,
            false,
        ),
        (
            "acid_formant_growl",
            "Formant Acid Growl",
            135.0,
            "4A Fm",
            820.0,
            7.8,
            false,
            true,
        ),
        (
            "acid_psy_trance",
            "Psy-Trance 142 Acid",
            142.0,
            "11A F#m",
            950.0,
            8.9,
            true,
            true,
        ),
    ];
    for (i, &(id, name, bpm, key, cut, res, wdf, f2)) in acid_specs.iter().enumerate() {
        let mut p = base(template, id, name, "Acid & 303", bpm, key);
        p.vco1_wave = if i % 3 == 0 {
            WaveformKind::Square
        } else {
            WaveformKind::Saw
        };
        p.vco2_wave = WaveformKind::SubOctave;
        p.filter_mode = FilterMode::AcidResonant;
        p.cutoff_hz = cut;
        p.resonance_q = res;
        p.env_mod_depth = 0.88;
        p.decay_ms = 175.0 + (i as f32) * 18.0;
        p.drive_saturation = 1.6 + (i as f32) * 0.14;
        if wdf {
            p.wdf_circuit.mode = if i == 7 {
                WdfCircuitMode::MoogCp3Overdrive
            } else {
                WdfCircuitMode::Tb303DiodeLadder
            };
            p.wdf_circuit.drive = 1.4;
            p.wdf_circuit.mix = 0.80;
        }
        if f2 {
            p.filter2.mode = if i == 8 {
                Filter2Mode::FormantVowel
            } else {
                Filter2Mode::Ms20Screamer
            };
            p.filter2.cutoff_hz = cut * 1.4;
            p.filter2.mix = 0.55;
        }
        p.steps = seq(&[
            36, 36, 48, 36, 43, 36, 41, 39, 36, 48, 43, 36, 46, 43, 41, 39,
        ]);
        out.push(p);
    }

    // 2. Bass & Reese (10)
    let bass_specs = [
        (
            "bass_neuro_reese",
            "Neurofunk Monster Reese",
            174.0,
            "4A Fm",
            WaveformKind::SuperSaw,
            420.0,
            true,
        ),
        (
            "bass_moog_cp3_sub",
            "Moog CP3 Fat Sub",
            124.0,
            "8A Am",
            WaveformKind::Triangle,
            280.0,
            false,
        ),
        (
            "bass_m1_organ2_club",
            "1993 House Organ 2 Bass",
            124.0,
            "5A Cm",
            WaveformKind::Square,
            1800.0,
            false,
        ),
        (
            "bass_uk_garage_wobble",
            "UK Garage 2-Step Wobble",
            132.0,
            "7A Dm",
            WaveformKind::Saw,
            560.0,
            true,
        ),
        (
            "bass_dubstep_formant",
            "Formant Talking Bass",
            140.0,
            "4A Fm",
            WaveformKind::Wavetable,
            720.0,
            true,
        ),
        (
            "bass_dx7_lately",
            "DX7 Lately FM Bass",
            120.0,
            "9A Em",
            WaveformKind::Sine,
            2400.0,
            false,
        ),
        (
            "bass_donk_bounce",
            "Scouse FM Donk Bass",
            145.0,
            "8A Am",
            WaveformKind::Sine,
            2100.0,
            false,
        ),
        (
            "bass_detroit_sub",
            "Detroit Deep Sub",
            128.0,
            "6A Gm",
            WaveformKind::SubOctave,
            320.0,
            false,
        ),
        (
            "bass_italo_octave",
            "Italo Disco Octave Bass",
            118.0,
            "8A Am",
            WaveformKind::Saw,
            980.0,
            false,
        ),
        (
            "bass_tape_analog",
            "Warm Tape Moog Bass",
            112.0,
            "10A Bm",
            WaveformKind::Saw,
            610.0,
            false,
        ),
    ];
    for (i, &(id, name, bpm, key, wave, cut, ott)) in bass_specs.iter().enumerate() {
        let mut p = base(template, id, name, "Bass & Reese", bpm, key);
        p.vco1_wave = wave;
        p.vco2_wave = WaveformKind::SubOctave;
        p.vco2_detune_cents = if i == 0 { 24.0 } else { 8.0 };
        p.cutoff_hz = cut;
        p.resonance_q = 2.8 + (i as f32) * 0.25;
        p.attack_ms = 3.0;
        p.decay_ms = 240.0;
        p.sustain_level = 0.65;
        if i == 0 || i == 4 {
            p.unison.voices = 8;
            p.unison.detune_cents = 26.0;
            p.unison.stereo_spread = 0.7;
        }
        if i == 1 || i == 9 {
            p.wdf_circuit.mode = WdfCircuitMode::MoogCp3Overdrive;
            p.wdf_circuit.drive = 1.6;
            p.wdf_circuit.mix = 0.85;
        }
        if i == 2 {
            p.m1_program = Some(PcmProgram::Organ2_17);
            p.m1_layer_mix = 0.85;
        }
        if i == 4 {
            p.filter2.mode = Filter2Mode::FormantVowel;
            p.filter2.vowel_morph = 0.45;
            p.filter2.mix = 0.75;
        }
        if i == 5 || i == 6 {
            p.fm6 = Fm6Params::preset("lately_bass");
            p.fm6.enabled = true;
            p.fm6.mix = 0.80;
        }
        if i == 3 {
            p.mseg.mode = MsegMode::TempoLoop;
            p.mseg.tempo_beats = 1.0;
            p.mseg.cutoff_depth = 0.75;
        }
        if i == 9 {
            p.studio_fx.enabled = true;
            p.studio_fx.tape_enabled = true;
            p.studio_fx.tape_drive = 0.45;
        }
        if ott {
            p.studio_fx.enabled = true;
            p.studio_fx.ott_enabled = true;
            p.studio_fx.ott_depth = 0.55;
        }
        p.steps = seq(&[
            33, 33, 33, 36, 38, 38, 36, 33, 33, 33, 40, 38, 36, 36, 35, 33,
        ]);
        out.push(p);
    }

    // 3. Leads & SuperSaw (10)
    let lead_specs = [
        (
            "lead_jp8000_trance",
            "JP-8000 16-Voice SuperSaw",
            138.0,
            "8A Am",
            16,
            32.0,
            true,
        ),
        (
            "lead_minimoog_sing",
            "Minimoog Singing Lead",
            120.0,
            "9A Em",
            1,
            5.0,
            false,
        ),
        (
            "lead_blade_runner",
            "CS-80 Blade Runner Brass",
            90.0,
            "7A Dm",
            6,
            18.0,
            false,
        ),
        (
            "lead_ms20_punk",
            "MS-20 Punk Screamer",
            130.0,
            "4A Fm",
            2,
            12.0,
            true,
        ),
        (
            "lead_hyperpop_ott",
            "Hyperpop OTT Saw",
            150.0,
            "11B A",
            12,
            24.0,
            true,
        ),
        (
            "lead_fm_sync_razor",
            "Thru-Zero FM Razor",
            136.0,
            "6A Gm",
            4,
            14.0,
            true,
        ),
        (
            "lead_cyber_pluck",
            "Cyberpunk MSEG Pluck",
            124.0,
            "5A Cm",
            8,
            20.0,
            true,
        ),
        (
            "lead_chip_pulse",
            "Retro Pulse PWM Lead",
            128.0,
            "8B C",
            1,
            3.0,
            false,
        ),
        (
            "lead_stadium_anthem",
            "Stadium Festival Anthem",
            128.0,
            "10B D",
            16,
            38.0,
            true,
        ),
        (
            "lead_velvet_glide",
            "Velvet Glide Solo",
            110.0,
            "7B F",
            4,
            10.0,
            false,
        ),
    ];
    for (i, &(id, name, bpm, key, uni, det, ott)) in lead_specs.iter().enumerate() {
        let mut p = base(template, id, name, "Leads & SuperSaw", bpm, key);
        p.vco1_wave = if i == 7 {
            WaveformKind::PulsePwm
        } else {
            WaveformKind::SuperSaw
        };
        p.vco2_wave = WaveformKind::Saw;
        p.cutoff_hz = 3200.0 + (i as f32) * 280.0;
        p.resonance_q = 2.2;
        p.unison.voices = uni;
        p.unison.detune_cents = det;
        p.unison.stereo_spread = 0.88;
        p.unison.stack_mode = UnisonStackMode::SuperSaw;
        if ott {
            p.studio_fx.enabled = true;
            p.studio_fx.ott_enabled = true;
            p.studio_fx.ott_depth = 0.48;
        }
        if i == 0 || i == 2 || i == 8 {
            p.studio_fx.enabled = true;
            p.studio_fx.shimmer_enabled = true;
            p.studio_fx.shimmer_mix = 0.28;
        }
        if i == 3 {
            p.filter2.mode = Filter2Mode::Ms20Screamer;
            p.filter2.mix = 0.7;
        }
        if i == 5 {
            p.fm6 = Fm6Params::preset("neuro_growl");
            p.fm6.enabled = true;
            p.fm6.mix = 0.65;
        }
        if i == 6 {
            p.mseg.mode = MsegMode::TempoLoop;
            p.mseg.cutoff_depth = 0.7;
        }
        p.steps = seq(&[
            60, 64, 67, 72, 71, 67, 64, 62, 60, 64, 67, 69, 67, 64, 62, 60,
        ]);
        out.push(p);
    }

    // 4. Pads & Ambient (10)
    let pad_specs = [
        (
            "pad_eno_apollo",
            "Apollo Shimmer Cathedral",
            76.0,
            "8B C",
            true,
            true,
            false,
        ),
        (
            "pad_juno_warmth",
            "Juno-106 Warm Chorus Pad",
            96.0,
            "8A Am",
            false,
            false,
            true,
        ),
        (
            "pad_frozen_aurora",
            "Frozen 24-Grain Aurora",
            72.0,
            "10B D",
            true,
            true,
            false,
        ),
        (
            "pad_boards_tape",
            "Boards Analog Tape Pad",
            88.0,
            "7B F",
            false,
            false,
            true,
        ),
        (
            "pad_universe_1990",
            "1990 Universe Choir Pad",
            104.0,
            "9A Em",
            false,
            true,
            false,
        ),
        (
            "pad_deep_space_drone",
            "Deep Space Spectral Drone",
            68.0,
            "4A Fm",
            true,
            true,
            false,
        ),
        (
            "pad_glass_horizon",
            "Glass Horizon Wavetable",
            92.0,
            "11B A",
            false,
            true,
            false,
        ),
        (
            "pad_vienna_mist",
            "Danube Morning Mist",
            82.0,
            "8A Am",
            true,
            false,
            true,
        ),
        (
            "pad_silk_strings",
            "Silk Analog Strings",
            98.0,
            "9B G",
            false,
            false,
            false,
        ),
        (
            "pad_cosmic_drift",
            "Cosmic Ambient Drift",
            74.0,
            "6A Gm",
            true,
            true,
            true,
        ),
    ];
    for (i, &(id, name, bpm, key, gran, shim, tape)) in pad_specs.iter().enumerate() {
        let mut p = base(template, id, name, "Pads & Ambient", bpm, key);
        p.vco1_wave = WaveformKind::AtmoPad;
        p.vco2_wave = WaveformKind::Wavetable;
        p.cutoff_hz = 1650.0 + (i as f32) * 160.0;
        p.attack_ms = 220.0 + (i as f32) * 35.0;
        p.decay_ms = 900.0;
        p.sustain_level = 0.82;
        p.release_ms = 1400.0 + (i as f32) * 120.0;
        p.fx.chorus_mix = 0.48;
        p.fx.reverb_mix = 0.52;
        p.unison.voices = 6;
        p.unison.detune_cents = 16.0;
        p.unison.stereo_spread = 0.95;
        if gran {
            p.granular.enabled = true;
            p.granular.mix = 0.55;
            p.granular.density = 16;
        }
        if shim {
            p.studio_fx.enabled = true;
            p.studio_fx.shimmer_enabled = true;
            p.studio_fx.shimmer_mix = 0.42;
        }
        if tape {
            p.studio_fx.enabled = true;
            p.studio_fx.tape_enabled = true;
            p.studio_fx.tape_drive = 0.35;
        }
        if i == 4 {
            p.m1_program = Some(PcmProgram::Universe00);
            p.m1_layer_mix = 0.80;
        }
        if i == 5 {
            p.spectral.enabled = true;
            p.spectral.mix = 0.65;
        }
        p.steps = seq(&[
            48, 48, 52, 52, 55, 55, 59, 59, 57, 57, 55, 55, 52, 52, 48, 48,
        ]);
        out.push(p);
    }

    // 5. Keys & 1990 PCM (10)
    let keys_specs = [
        (
            "keys_m1_ibiza_piano",
            "1990 M1 Ibiza House Piano",
            124.0,
            "8A Am",
            Some(PcmProgram::Piano16_01),
            false,
        ),
        (
            "keys_dx7_epiano1",
            "1983 DX7 Electric Piano 1",
            96.0,
            "8B C",
            None,
            true,
        ),
        (
            "keys_m1_choir_aah",
            "1990 Ooh/Aah Vocal Choir",
            108.0,
            "9A Em",
            Some(PcmProgram::OohAahChoir36),
            false,
        ),
        (
            "keys_m1_pan_flute",
            "1990 Breathy Pan Flute",
            102.0,
            "7A Dm",
            Some(PcmProgram::PanFlute24),
            false,
        ),
        (
            "keys_m1_orch_hit",
            "1988 Fairlight Orch Hit",
            126.0,
            "4A Fm",
            Some(PcmProgram::OrchestraHit48),
            false,
        ),
        (
            "keys_m1_fretless",
            "1990 M1 Fretless Mwah",
            110.0,
            "5A Cm",
            Some(PcmProgram::Fretless11),
            false,
        ),
        (
            "keys_m1_slap_funk",
            "1990 M1 Slap Funk",
            116.0,
            "9A Em",
            Some(PcmProgram::SlapBass12),
            false,
        ),
        (
            "keys_lofi_rhodes",
            "Lo-Fi Tape Rhodes",
            86.0,
            "7B F",
            None,
            true,
        ),
        (
            "keys_baroque_cembalo",
            "Baroque Plucked Cembalo",
            112.0,
            "10B D",
            None,
            false,
        ),
        (
            "keys_wurlitzer_bark",
            "Vintage Wurlitzer Bark",
            104.0,
            "8B C",
            None,
            true,
        ),
    ];
    for (i, &(id, name, bpm, key, prog, fm)) in keys_specs.iter().enumerate() {
        let mut p = base(template, id, name, "Keys & 1990 PCM", bpm, key);
        p.m1_program = prog;
        p.m1_layer_mix = 0.88;
        p.cutoff_hz = 4800.0;
        p.attack_ms = 2.0;
        p.decay_ms = 420.0;
        p.sustain_level = 0.42;
        p.release_ms = 280.0;
        if fm {
            p.fm6 = Fm6Params::preset("dx7_epiano");
            p.fm6.enabled = true;
            p.fm6.mix = 0.78;
        }
        if i == 7 {
            p.studio_fx.enabled = true;
            p.studio_fx.tape_enabled = true;
            p.studio_fx.tape_drive = 0.42;
        }
        if i == 8 {
            p.extreme_dsp = ExtremeDspMode::KarplusStrongPluck;
            p.extreme_morph = 0.72;
        }
        p.steps = seq(&[
            60, 64, 67, 64, 60, 67, 64, 60, 62, 65, 69, 65, 62, 69, 65, 62,
        ]);
        out.push(p);
    }

    // 6. Cathedral Pipe Organ (10) -- dedicated for Sandra!
    let organ_specs = [
        (
            "organ_silbermann_pleno",
            "Silbermann Baroque Principal 8'+4'+2'",
            92.0,
            "7A Dm",
            0.0,
            0.15,
            0.0,
            0.52,
        ),
        (
            "organ_bach_toccata_tutti",
            "Bach Toccata & Fugue Pleno Tutti",
            96.0,
            "7A Dm",
            3.0,
            0.05,
            0.04,
            0.60,
        ),
        (
            "organ_cavaille_coll_swell",
            "Cavaille-Coll Romantic Swell 8'",
            78.0,
            "8B C",
            -2.5,
            -0.15,
            0.0,
            0.56,
        ),
        (
            "organ_bourdon_vespers",
            "Bourdon 16' + Gedackt 8' Vespers",
            72.0,
            "7B F",
            -5.0,
            -0.65,
            0.0,
            0.55,
        ),
        (
            "organ_voix_celeste",
            "Voix Celeste 8' Undulating Swell",
            70.0,
            "9B G",
            -2.0,
            -0.25,
            0.01,
            0.58,
        ),
        (
            "organ_trompete_fanfare",
            "Trompete 8' + Clairon 4' Reed Fanfare",
            104.0,
            "10B D",
            4.5,
            0.35,
            0.02,
            0.50,
        ),
        (
            "organ_subbass_pedal",
            "Subbass 16' + Posaune 32' Cathedral Pedal",
            68.0,
            "8A Am",
            -7.0,
            -0.40,
            0.0,
            0.58,
        ),
        (
            "organ_positiv_flute4",
            "Positiv Rohrflote 8' + Blockflote 4'",
            98.0,
            "8B C",
            -1.5,
            -0.55,
            0.0,
            0.45,
        ),
        (
            "organ_mixtur_crown",
            "Mixtur IV + Zimbel III Baroque Crown",
            108.0,
            "9B G",
            6.0,
            0.25,
            0.03,
            0.55,
        ),
        (
            "organ_stephansdom_midnight",
            "Stephansdom Midnight Cathedral Organ",
            74.0,
            "7A Dm",
            1.5,
            0.05,
            0.02,
            0.68,
        ),
    ];
    for (i, &(id, name, bpm, key, fshift, odd_even, warp, conv_mix)) in
        organ_specs.iter().enumerate()
    {
        let mut p = base(template, id, name, "Cathedral Pipe Organ", bpm, key);
        p.vco1_wave = WaveformKind::Triangle;
        p.vco2_wave = WaveformKind::SubOctave;
        p.vco_mix = if i == 6 { 0.75 } else { 0.35 };
        p.vco2_detune_cents = if i == 4 { 14.0 } else { 3.5 };
        p.cutoff_hz = (5200.0_f32 + fshift * 400.0).clamp(1200.0, 11_000.0);
        p.resonance_q = 0.8;
        p.attack_ms = 24.0 + (i as f32) * 3.0;
        p.decay_ms = 120.0;
        p.sustain_level = 0.96;
        p.release_ms = 320.0;
        p.spectral.enabled = true;
        p.spectral.formant_shift_st = fshift;
        p.spectral.odd_even_tilt = odd_even;
        p.spectral.harmonic_warp = warp;
        p.spectral.mix = 0.88;
        p.studio_fx.enabled = true;
        p.studio_fx.conv_enabled = true;
        p.studio_fx.conv_preset = IrPreset::StephansdomCathedral;
        p.studio_fx.conv_mix = conv_mix;
        if i == 1 || i == 9 {
            p.unison.voices = 4;
            p.unison.detune_cents = 7.0;
            p.unison.stereo_spread = 0.75;
        }
        if i == 9 {
            p.studio_fx.shimmer_enabled = true;
            p.studio_fx.shimmer_mix = 0.22;
        }
        p.fx.reverb_mix = 0.42;
        p.steps = seq(&[
            69, 67, 69, 69, 67, 65, 64, 62, 61, 62, 62, 50, 53, 57, 62, 50,
        ]);
        out.push(p);
    }

    // 7. 6-Op FM & Bells (10)
    let fm_specs = [
        (
            "fm_tubular_bell",
            "DX7 Tubular Cathedral Bell",
            100.0,
            "8B C",
            "berlin_fm_bell",
            0.35,
        ),
        (
            "fm_crystal_celesta",
            "Crystal FM Celesta",
            112.0,
            "9B G",
            "glass_choir",
            0.25,
        ),
        (
            "fm_metallic_gong",
            "Inharmonic Cyber Gong",
            84.0,
            "4A Fm",
            "berlin_fm_bell",
            0.55,
        ),
        (
            "fm_wood_marimba",
            "Thru-Zero Wood Marimba",
            118.0,
            "10B D",
            "dx7_epiano",
            0.15,
        ),
        (
            "fm_glass_harp",
            "Glass Armonica FM",
            90.0,
            "8A Am",
            "glass_choir",
            0.40,
        ),
        (
            "fm_cz_hollow_bass",
            "CZ-101 Phase Distortion Bass",
            124.0,
            "5A Cm",
            "lately_bass",
            0.10,
        ),
        (
            "fm_brass_stab",
            "1985 DX7 Brass Stab",
            122.0,
            "11B A",
            "dx7_epiano",
            0.20,
        ),
        (
            "fm_chime_cascade",
            "Wind Chime Cascade",
            94.0,
            "7B F",
            "berlin_fm_bell",
            0.50,
        ),
        (
            "fm_industrial_clang",
            "Industrial FM Clang",
            136.0,
            "2A Ebm",
            "neuro_growl",
            0.30,
        ),
        (
            "fm_velvet_tines",
            "Velvet Tines EP",
            92.0,
            "8B C",
            "dx7_epiano",
            0.28,
        ),
    ];
    for (i, &(id, name, bpm, key, preset_id, rev)) in fm_specs.iter().enumerate() {
        let mut p = base(template, id, name, "6-Op FM & Bells", bpm, key);
        p.fm6 = Fm6Params::preset(preset_id);
        p.fm6.enabled = true;
        p.fm6.mix = 0.82;
        p.fm_index = 1.4 + (i as f32) * 0.22;
        p.attack_ms = 2.0;
        p.decay_ms = 550.0 + (i as f32) * 80.0;
        p.sustain_level = 0.18;
        p.release_ms = 900.0;
        p.fx.reverb_mix = rev;
        if i == 5 {
            p.extreme_dsp = ExtremeDspMode::PhaseDistortionCz;
            p.extreme_morph = 0.62;
        }
        p.steps = seq(&[
            60, 67, 72, 76, 72, 67, 64, 60, 62, 69, 74, 77, 74, 69, 65, 62,
        ]);
        out.push(p);
    }

    // 8. West-Coast & Modular (10)
    let wc_specs = [
        (
            "wc_buchla_292_bongo",
            "Buchla 292 Vactrol Bongo",
            122.0,
            "8A Am",
            ExtremeDspMode::WestCoastWavefolder,
            true,
        ),
        (
            "wc_subotnick_silver",
            "Silver Apples Wavefolder",
            116.0,
            "7A Dm",
            ExtremeDspMode::WestCoastWavefolder,
            true,
        ),
        (
            "wc_karplus_koto",
            "Karplus-Strong Kyoto Koto",
            104.0,
            "9A Em",
            ExtremeDspMode::KarplusStrongPluck,
            false,
        ),
        (
            "wc_modal_vibraphone",
            "Modal Struck Vibraphone",
            110.0,
            "8B C",
            ExtremeDspMode::ModalStruckBar,
            false,
        ),
        (
            "wc_ciani_harp",
            "Suzanne Ciani Buchla Harp",
            98.0,
            "7B F",
            ExtremeDspMode::WestCoastWavefolder,
            true,
        ),
        (
            "wc_rubber_pluck",
            "Rubbery 5-Stage Fold Pluck",
            128.0,
            "4A Fm",
            ExtremeDspMode::WestCoastWavefolder,
            true,
        ),
        (
            "wc_euclidean_bells",
            "Euclidean Modal Bells",
            126.0,
            "10B D",
            ExtremeDspMode::ModalStruckBar,
            false,
        ),
        (
            "wc_slew_arp",
            "ARP 2600 S&H Slew Line",
            130.0,
            "6A Gm",
            ExtremeDspMode::Off,
            false,
        ),
        (
            "wc_glass_marimba",
            "Glass Bar Resonator",
            114.0,
            "9B G",
            ExtremeDspMode::ModalStruckBar,
            false,
        ),
        (
            "wc_desert_oud",
            "Desert Plucked Oud",
            106.0,
            "5A Cm",
            ExtremeDspMode::KarplusStrongPluck,
            false,
        ),
    ];
    for (i, &(id, name, bpm, key, xmode, lpg)) in wc_specs.iter().enumerate() {
        let mut p = base(template, id, name, "West-Coast & Modular", bpm, key);
        p.vco1_wave = WaveformKind::Sine;
        p.extreme_dsp = xmode;
        p.extreme_morph = 0.45 + (i as f32) * 0.04;
        p.attack_ms = 2.0;
        p.decay_ms = 260.0 + (i as f32) * 25.0;
        p.sustain_level = 0.12;
        p.release_ms = 340.0;
        if lpg {
            p.wdf_circuit.mode = WdfCircuitMode::Buchla292VactrolLpg;
            p.wdf_circuit.drive = 1.3;
            p.wdf_circuit.mix = 0.85;
        }
        p.fx.reverb_mix = 0.30;
        p.steps = seq(&[
            55, 62, 60, 67, 55, 65, 62, 69, 55, 62, 60, 67, 65, 62, 60, 55,
        ]);
        out.push(p);
    }

    // 9. Physical & Acoustic (10)
    let phys_specs = [
        (
            "phys_ddsp_violin",
            "Neural DDSP Solo Violin",
            88.0,
            "7A Dm",
            Some(DdspTimbre::SoloViolinBow),
            None,
        ),
        (
            "phys_ddsp_flute",
            "Neural DDSP Breath Flute",
            94.0,
            "8B C",
            Some(DdspTimbre::BreathFlute),
            None,
        ),
        (
            "phys_ddsp_horn",
            "Neural DDSP Vienna Brass Horn",
            102.0,
            "7B F",
            Some(DdspTimbre::ViennaBrassHorn),
            None,
        ),
        (
            "phys_ddsp_cello",
            "Neural DDSP Cello Sul Ponticello",
            76.0,
            "8A Am",
            Some(DdspTimbre::CelloSulPonticello),
            None,
        ),
        (
            "phys_ddsp_gamelan",
            "Neural DDSP Gamelan Metal",
            116.0,
            "10B D",
            Some(DdspTimbre::GamelanMetal),
            None,
        ),
        (
            "phys_ddsp_choir",
            "Neural DDSP Alien Vocal Choir",
            80.0,
            "9B G",
            Some(DdspTimbre::AlienVocalChoir),
            None,
        ),
        (
            "phys_fdtd_titanium",
            "2D FDTD Vibrating Titanium Plate",
            120.0,
            "4A Fm",
            None,
            Some(PhysicalMaterial::TitaniumPlate),
        ),
        (
            "phys_fdtd_glass",
            "2D FDTD Glass Marimba",
            90.0,
            "8B C",
            None,
            Some(PhysicalMaterial::GlassMarimba),
        ),
        (
            "phys_fdtd_membrane",
            "2D FDTD Kettle Membrane",
            128.0,
            "8A Am",
            None,
            Some(PhysicalMaterial::KettleMembrane),
        ),
        (
            "phys_fdtd_cello",
            "2D FDTD Bowed Cello Body",
            112.0,
            "9A Em",
            None,
            Some(PhysicalMaterial::BowedCelloBody),
        ),
    ];
    for &(id, name, bpm, key, ddsp, mesh) in phys_specs.iter() {
        let mut p = base(template, id, name, "Physical & Acoustic", bpm, key);
        if let Some(t) = ddsp {
            p.ddsp.enabled = true;
            p.ddsp.timbre_a = t;
            p.ddsp.timbre_b = t;
            p.ddsp.morph = 0.0;
            p.ddsp.bow_pressure = 0.72;
            p.ddsp.breath_noise = 0.24;
            p.ddsp.mix = 0.85;
            p.attack_ms = 45.0;
            p.release_ms = 360.0;
        }
        if let Some(m) = mesh {
            p.physical_mesh.enabled = true;
            p.physical_mesh.material = m;
            p.physical_mesh.tension = 0.30;
            p.physical_mesh.damping = 0.008;
            p.physical_mesh.mix = 0.80;
        }
        p.studio_fx.enabled = true;
        p.studio_fx.conv_enabled = true;
        p.studio_fx.conv_preset = IrPreset::ViennaStudioRoom;
        p.studio_fx.conv_mix = 0.30;
        p.steps = seq(&[
            60, 62, 64, 67, 69, 67, 64, 62, 60, 64, 67, 72, 71, 67, 64, 60,
        ]);
        out.push(p);
    }

    // 10. Granular & Spectral (10)
    let gran_specs = [
        (
            "gran_crystal_cloud",
            "24-Grain Crystal Cloud",
            86.0,
            "8B C",
            true,
            false,
            12.0,
        ),
        (
            "gran_shimmer_mist",
            "Octave Shimmer Mist",
            78.0,
            "9B G",
            true,
            false,
            12.0,
        ),
        (
            "gran_sub_freeze",
            "Frozen Sub-Octave Haze",
            82.0,
            "4A Fm",
            true,
            false,
            -12.0,
        ),
        (
            "gran_fifth_halo",
            "Fifth Harmonic Halo",
            92.0,
            "10B D",
            true,
            false,
            7.0,
        ),
        (
            "gran_micro_rain",
            "Microsound Rain Drops",
            104.0,
            "8A Am",
            true,
            false,
            0.0,
        ),
        (
            "spec_64_clarinet",
            "64-Partial Odd Clarinet",
            100.0,
            "7B F",
            false,
            true,
            0.0,
        ),
        (
            "spec_64_bell_stretch",
            "64-Partial Stretched Bells",
            96.0,
            "11B A",
            false,
            true,
            0.0,
        ),
        (
            "spec_64_choir_wash",
            "64-Partial Spectral Wash",
            84.0,
            "9A Em",
            true,
            true,
            0.0,
        ),
        (
            "spec_64_glass_organ",
            "64-Partial Glass Drawbars",
            108.0,
            "8B C",
            false,
            true,
            0.0,
        ),
        (
            "spec_64_comb_alien",
            "Spectral Comb Resonance",
            124.0,
            "6A Gm",
            false,
            true,
            0.0,
        ),
    ];
    for (i, &(id, name, bpm, key, gran, spec, pitch)) in gran_specs.iter().enumerate() {
        let mut p = base(template, id, name, "Granular & Spectral", bpm, key);
        p.attack_ms = 90.0;
        p.release_ms = 900.0;
        if gran {
            p.granular.enabled = true;
            p.granular.density = 18;
            p.granular.grain_size_ms = 95.0;
            p.granular.pitch_semitones = pitch;
            p.granular.mix = 0.72;
        }
        if spec {
            p.spectral.enabled = true;
            p.spectral.formant_shift_st = if i == 5 { -3.0 } else { 2.0 };
            p.spectral.odd_even_tilt = if i == 5 { -0.90 } else { 0.0 };
            p.spectral.harmonic_warp = if i == 6 { 0.22 } else { 0.02 };
            p.spectral.mix = 0.82;
        }
        if i == 9 {
            p.filter2.mode = Filter2Mode::CombPositive;
            p.filter2.routing = Filter2Routing::Parallel;
            p.filter2.mix = 0.65;
        }
        p.studio_fx.enabled = true;
        p.studio_fx.shimmer_enabled = true;
        p.studio_fx.shimmer_mix = 0.34;
        p.steps = seq(&[
            57, 60, 64, 69, 67, 64, 60, 57, 55, 59, 62, 67, 64, 62, 59, 55,
        ]);
        out.push(p);
    }

    // 11. Vocoder & Chords (10)
    let voc_specs = [
        (
            "voc_kraftwerk_robots",
            "Kraftwerk 16-Band Robots",
            118.0,
            "8A Am",
            true,
            ChordMode::MinorTriad,
            0.0,
        ),
        (
            "voc_daft_around",
            "Daft Punk Saw Vocoder",
            121.0,
            "8A Am",
            true,
            ChordMode::Minor7th,
            2.0,
        ),
        (
            "voc_giant_cyborg",
            "Giant Cyborg Formant",
            114.0,
            "4A Fm",
            true,
            ChordMode::Power5th,
            -5.0,
        ),
        (
            "voc_cyber_soprano",
            "Cyber Pop Soprano",
            126.0,
            "10B D",
            true,
            ChordMode::Major7th,
            5.0,
        ),
        (
            "voc_imogen_choir",
            "Imogen Harmony Choir",
            86.0,
            "8B C",
            true,
            ChordMode::TechnoSus4,
            1.0,
        ),
        (
            "chrd_basic_dub11",
            "Basic Channel Dub Minor 11",
            124.0,
            "5A Cm",
            false,
            ChordMode::DubTechnoM11,
            0.0,
        ),
        (
            "chrd_detroit_stab",
            "Detroit 1988 House Stab (m9)",
            126.0,
            "8A Am",
            false,
            ChordMode::HouseStabM9,
            0.0,
        ),
        (
            "chrd_baroque_64",
            "Vienna Baroque 6/4 Consort",
            96.0,
            "7B F",
            false,
            ChordMode::ViennaBaroque64,
            0.0,
        ),
        (
            "chrd_deep_minor7",
            "Chicago Deep Minor 7th",
            122.0,
            "9A Em",
            false,
            ChordMode::Minor7th,
            0.0,
        ),
        (
            "chrd_trance_major7",
            "Euphoric Major 7th Swell",
            136.0,
            "10B D",
            false,
            ChordMode::Major7th,
            0.0,
        ),
    ];
    for &(id, name, bpm, key, voc, cmode, fshift) in voc_specs.iter() {
        let mut p = base(template, id, name, "Vocoder & Chords", bpm, key);
        p.vco1_wave = WaveformKind::SuperSaw;
        p.cutoff_hz = if voc { 4200.0 } else { 1450.0 };
        p.chord_mem.chord_mode = cmode;
        p.chord_mem.scale_lock = ScaleLock::NaturalMinor;
        if voc {
            p.vocoder.enabled = true;
            p.vocoder.formant_shift_st = fshift;
            p.vocoder.mix = 0.88;
        } else {
            p.fx.chorus_mix = 0.45;
            p.fx.reverb_mix = 0.42;
        }
        p.steps = seq(&[
            48, 48, 51, 51, 53, 53, 55, 55, 53, 53, 51, 51, 48, 48, 46, 48,
        ]);
        out.push(p);
    }

    // 12. Drums & Grooves (10)
    let drum_specs = [
        (
            "drum_808_classic",
            "TR-808 Electro Funk Groove",
            96.0,
            "8A Am",
            "tr808_electro",
            0.56,
        ),
        (
            "drum_909_chicago",
            "1990 House Classic 909 Kit",
            124.0,
            "8A Am",
            "house_1990_classic",
            0.54,
        ),
        (
            "drum_berlin_techno",
            "Berlin 136 BPM Techno Kit",
            136.0,
            "4A Fm",
            "tr909_berlin_techno",
            0.52,
        ),
        (
            "drum_dnb_amen",
            "Drum & Bass 174 Two-Step",
            174.0,
            "9A Em",
            "dnb_amen_174",
            0.50,
        ),
        (
            "drum_acid_warehouse",
            "Acid 303 Warehouse Kit",
            132.0,
            "5A Cm",
            "acid_303_groove",
            0.53,
        ),
        (
            "drum_polyrhythm_64",
            "64-Step 4-Bar Club Groove",
            128.0,
            "2A Ebm",
            "polyrhythm_64",
            0.52,
        ),
        (
            "drum_acid_909_jam",
            "Acid 303 + 909 Full Jam",
            132.0,
            "8A Am",
            "tr909_berlin_techno",
            0.53,
        ),
        (
            "drum_lofi_tape_beat",
            "Lo-Fi Analog Tape Beat",
            88.0,
            "7A Dm",
            "tr808_electro",
            0.62,
        ),
        (
            "drum_ott_festival",
            "OTT Festival Big-Room Kit",
            128.0,
            "10B D",
            "tr909_berlin_techno",
            0.50,
        ),
        (
            "drum_dj_riser_kit",
            "DJCraft Tension Riser + Kit",
            128.0,
            "8A Am",
            "house_1990_classic",
            0.52,
        ),
    ];
    for (i, &(id, name, bpm, key, kit, swing)) in drum_specs.iter().enumerate() {
        let mut p = base(template, id, name, "Drums & Grooves", bpm, key);
        let mut dm = DrumMachinePatch::preset(kit);
        dm.enabled = true;
        dm.swing = swing;
        dm.master_level = 0.90;
        p.drums = dm;
        if i == 7 {
            p.studio_fx.enabled = true;
            p.studio_fx.tape_enabled = true;
            p.studio_fx.tape_drive = 0.48;
        }
        if i == 8 {
            p.studio_fx.enabled = true;
            p.studio_fx.ott_enabled = true;
            p.studio_fx.ott_depth = 0.55;
        }
        p.steps = seq(&[
            36, 36, 48, 36, 43, 36, 41, 39, 36, 36, 48, 36, 46, 43, 41, 39,
        ]);
        out.push(p);
    }

    out
}
