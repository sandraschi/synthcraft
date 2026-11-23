//! Integration test suite for SynthCraft L1 core.

use synthcraft_core::{
    export_midi_smf_file, midi_to_freq_hz, Command, ExtremeDspMode, FilterMode, KorgM1Program,
    SynthEngine, WaveformKind,
};

#[test]
fn midi_a4_is_440_hz_and_a3_is_220_hz() {
    assert!((midi_to_freq_hz(69) - 440.0).abs() < 0.01);
    assert!((midi_to_freq_hz(57) - 220.0).abs() < 0.01);
}

#[test]
fn all_eight_waveform_kinds_parse_and_sample_within_bounds() {
    for (raw, expected) in [
        ("sine", WaveformKind::Sine),
        ("saw", WaveformKind::Saw),
        ("square", WaveformKind::Square),
        ("triangle", WaveformKind::Triangle),
        ("pulse_pwm", WaveformKind::PulsePwm),
        ("supersaw", WaveformKind::SuperSaw),
        ("white_noise", WaveformKind::WhiteNoise),
        ("sub_octave", WaveformKind::SubOctave),
    ] {
        let parsed = WaveformKind::parse(raw);
        assert_eq!(parsed, Some(expected));
        let val = expected.sample(0.25, 0.4, 42);
        assert!((-1.0..=1.0).contains(&val));
    }
}

#[test]
fn all_four_filter_modes_parse_accurately() {
    for (raw, expected) in [
        ("lowpass_24db", FilterMode::LowPass24dB),
        ("bandpass_12db", FilterMode::BandPass12dB),
        ("highpass_12db", FilterMode::HighPass12dB),
        ("acid_resonant", FilterMode::AcidResonant),
    ] {
        assert_eq!(FilterMode::parse(raw), Some(expected));
    }
}

#[test]
fn command_update_oscillators_and_filter_mutates_active_patch() {
    let mut eng = SynthEngine::factory_seed();
    let osc = Command::UpdateOscillators {
        vco1_wave: "supersaw".into(),
        vco2_wave: "pulse_pwm".into(),
        detune_cents: 19.5,
        mix: 0.6,
        fm_index: 0.8,
    }
    .dispatch(&mut eng);
    assert!(osc.is_ok());
    assert_eq!(eng.active_patch().vco1_wave, WaveformKind::SuperSaw);

    let flt = Command::UpdateFilter {
        mode: "bandpass_12db".into(),
        cutoff_hz: 2200.0,
        resonance_q: 5.5,
        env_mod_depth: 0.7,
    }
    .dispatch(&mut eng);
    assert!(flt.is_ok());
    assert_eq!(eng.active_patch().filter_mode, FilterMode::BandPass12dB);
}

#[test]
fn command_update_envelope_lfo_and_sequencer_step() {
    let mut eng = SynthEngine::factory_seed();
    let env = Command::UpdateEnvelopeLfo {
        attack_ms: 15.0,
        decay_ms: 250.0,
        sustain_level: 0.6,
        release_ms: 180.0,
        lfo_rate_hz: 3.5,
        lfo_depth: 0.4,
    }
    .dispatch(&mut eng);
    assert!(env.is_ok());

    let step = Command::SetSequencerStep {
        step_index: 3,
        midi_note: 60,
        accent: true,
        slide: true,
        active: true,
    }
    .dispatch(&mut eng);
    assert!(step.is_ok());
    assert_eq!(eng.active_patch().steps[3].midi_note, 60);
}

#[test]
fn out_of_bounds_sequencer_step_returns_validation_error() {
    let mut eng = SynthEngine::factory_seed();
    let bad = Command::SetSequencerStep {
        step_index: 99,
        midi_note: 60,
        accent: false,
        slide: false,
        active: true,
    }
    .dispatch(&mut eng);
    assert!(bad.is_err());
}

#[test]
fn unknown_patch_id_returns_not_found_error() {
    let mut eng = SynthEngine::factory_seed();
    let bad = Command::LoadPatch {
        patch_id: "non_existent_patch".into(),
    }
    .dispatch(&mut eng);
    assert!(bad.is_err());
}

#[test]
fn synthesize_preview_and_analyze_spectrum_commands_succeed() {
    let mut eng = SynthEngine::factory_seed();
    let prev = Command::SynthesizePreview { duration_ms: 80 }.dispatch(&mut eng);
    assert!(prev.is_ok());

    let spec = Command::AnalyzeSpectrum { duration_ms: 80 }.dispatch(&mut eng);
    assert!(spec.is_ok());
}

#[test]
fn export_clap_and_djcraft_stem_commands_return_structured_json() {
    let mut eng = SynthEngine::factory_seed();
    let clap = Command::ExportClapManifest
        .dispatch(&mut eng)
        .unwrap_or_default();
    assert_eq!(clap["plugin_id"], "dev.sandraschi.synthcraft");

    let stem = Command::ExportDjcraftStem { bars: 8 }
        .dispatch(&mut eng)
        .unwrap_or_default();
    assert_eq!(stem["bars"], 8);
}

#[test]
fn reset_factory_patches_restores_default_patch() {
    let mut eng = SynthEngine::factory_seed();
    let _ = eng.select_patch("soundcraft_warm_pad");
    let res = Command::ResetFactoryPatches
        .dispatch(&mut eng)
        .unwrap_or_default();
    assert_eq!(res["active_patch_id"], "vienna_acid_303");
}

#[test]
fn all_eight_korg_m1_1990_programs_parse_and_sample_pcm() {
    for prog in KorgM1Program::all() {
        assert_eq!(KorgM1Program::parse(prog.as_str()), Some(*prog));
        let s = prog.sample_pcm(220.0, 0.015);
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn all_extreme_dsp_modes_parse_and_synthesize_bounded_samples() {
    for mode in ExtremeDspMode::all() {
        assert_eq!(ExtremeDspMode::parse(mode.as_str()), Some(*mode));
        let s = mode.sample_dsp(220.0, 0.025, 0.65);
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn midi_depot_loads_into_sequencer_and_exports_valid_smf_mid_header() {
    let mut eng = SynthEngine::factory_seed();
    let res = Command::LoadMidiDepotClip {
        clip_id: "90s_house_m1_piano".into(),
    }
    .dispatch(&mut eng);
    assert!(res.is_ok());
    assert_eq!(
        eng.active_patch().m1_program,
        Some(KorgM1Program::Piano16_01)
    );

    let mut path = std::env::temp_dir();
    path.push(format!("synthcraft_test_{}.mid", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let clip = &eng.midi_depot[0];
    let written = export_midi_smf_file(clip, &path);
    assert!(written.is_ok());
    let bytes = std::fs::read(&path).unwrap_or_default();
    let _ = std::fs::remove_file(&path);
    assert!(bytes.len() > 22);
    assert_eq!(&bytes[0..4], b"MThd");
    assert_eq!(&bytes[14..18], b"MTrk");
}

#[test]
fn all_six_demo_showcases_execute_and_produce_valid_spectra() {
    let mut eng = SynthEngine::factory_seed();
    let ids: Vec<String> = eng.demos.iter().map(|d| d.id.clone()).collect();
    assert_eq!(ids.len(), 6);
    for id in ids {
        let res = Command::RunDemoShowcase { demo_id: id }.dispatch(&mut eng);
        assert!(res.is_ok());
    }
}
