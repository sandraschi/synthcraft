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
        let s = prog.sample_pcm(3.3, 0.015);
        assert!((-1.0..=1.0).contains(&s));
    }
}

#[test]
fn all_extreme_dsp_modes_parse_and_synthesize_bounded_samples() {
    for mode in ExtremeDspMode::all() {
        assert_eq!(ExtremeDspMode::parse(mode.as_str()), Some(*mode));
        let s = mode.sample_dsp(5.5, 0.025, 0.65);
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

// ---------------------------------------------------------------------------
// Real-time engine behaviour
// ---------------------------------------------------------------------------

use synthcraft_core::dsp::{autocorrelation_pitch, magnitude_db, Ladder, Osc};
use synthcraft_core::{SynthPatch, SynthRt};

const SR: f32 = 48_000.0;

fn rms(x: &[f32]) -> f32 {
    (x.iter().map(|s| s * s).sum::<f32>() / x.len().max(1) as f32).sqrt()
}

fn plain_patch() -> SynthPatch {
    let mut p = SynthEngine::factory_seed().active_patch().clone();
    p.vco1_wave = WaveformKind::Saw;
    p.vco2_wave = WaveformKind::Saw;
    p.vco_mix = 0.0;
    p.vco2_detune_cents = 0.0;
    p.fm_index = 0.0;
    p.filter_mode = FilterMode::LowPass24dB;
    p.cutoff_hz = 9000.0;
    p.resonance_q = 0.5;
    p.env_mod_depth = 0.0;
    p.lfo_depth = 0.0;
    p.drive_saturation = 0.0;
    p.attack_ms = 2.0;
    p.decay_ms = 50.0;
    p.sustain_level = 1.0;
    p.release_ms = 80.0;
    p.m1_program = None;
    p.extreme_dsp = ExtremeDspMode::Off;
    p.master_gain = 0.8;
    p
}

fn render_held(patch: &SynthPatch, note: u8, held_ms: u32, tail_ms: u32) -> (Vec<f32>, Vec<f32>) {
    let mut rt = SynthRt::new(SR, patch.clone());
    let mut held = vec![0.0; (SR as u32 * held_ms / 1000) as usize];
    let mut tail = vec![0.0; (SR as u32 * tail_ms / 1000) as usize];
    rt.note_on(note, 1.0);
    rt.process(&mut held);
    rt.note_off(note);
    rt.process(&mut tail);
    (held, tail)
}

#[test]
fn band_limited_saw_has_less_alias_energy_than_a_naive_saw() {
    // 529 cycles in 8192 samples puts every true harmonic at a known bin; any energy away
    // from those bins is aliasing.
    const N: usize = 8192;
    const K: usize = 529;
    let f0 = SR * K as f32 / N as f32;
    let mut osc = Osc::new(1);
    let blep: Vec<f32> = (0..N)
        .map(|_| osc.tick(WaveformKind::Saw, f0, SR, 0.5, 0.0))
        .collect();
    let naive: Vec<f32> = (0..N)
        .map(|i| 2.0 * ((i as f32 * K as f32 / N as f32).rem_euclid(1.0)) - 1.0)
        .collect();
    let alias_power = |x: &[f32]| -> f32 {
        let db = magnitude_db(x);
        db.iter()
            .enumerate()
            .filter(|(bin, _)| (1..=7).all(|h| (*bin as i64 - (h * K) as i64).abs() > 3))
            .map(|(_, d)| 10.0_f32.powf(d / 10.0))
            .sum()
    };
    assert!(alias_power(&blep) < 0.5 * alias_power(&naive));
}

#[test]
fn ladder_modes_shape_the_spectrum_as_named() {
    fn gain(mode: FilterMode, freq: f32, q: f32) -> f32 {
        let mut f = Ladder::default();
        let n = 9600;
        let out: Vec<f32> = (0..n)
            .map(|i| {
                let x = 0.3 * (std::f32::consts::TAU * freq * i as f32 / SR).sin();
                f.process(x, 1000.0, SR, q, mode)
            })
            .collect();
        rms(&out[n / 2..]) / (0.3 / 2.0_f32.sqrt())
    }
    let lo = |m| gain(m, 125.0, 0.5);
    let mid = |m| gain(m, 1000.0, 0.5);
    let hi = |m| gain(m, 8000.0, 0.5);
    // Low-pass: three octaves above cutoff is at least 40 dB below the passband.
    assert!(hi(FilterMode::LowPass24dB) < lo(FilterMode::LowPass24dB) * 0.01);
    // High-pass: low content is far below high content.
    assert!(lo(FilterMode::HighPass12dB) < hi(FilterMode::HighPass12dB) * 0.1);
    // Band-pass: the middle beats both edges.
    assert!(mid(FilterMode::BandPass12dB) > lo(FilterMode::BandPass12dB));
    assert!(mid(FilterMode::BandPass12dB) > hi(FilterMode::BandPass12dB));
    // Resonance lifts the level at cutoff.
    assert!(gain(FilterMode::LowPass24dB, 1000.0, 10.0) > 1.5 * mid(FilterMode::LowPass24dB));
}

#[test]
fn held_notes_sound_the_right_pitch() {
    let patch = plain_patch();
    for note in [45_u8, 57, 69, 81] {
        let (held, _) = render_held(&patch, note, 400, 0);
        let hz = autocorrelation_pitch(&held[held.len() - 4096..], SR);
        let want = midi_to_freq_hz(note);
        assert!(
            (hz / want - 1.0).abs() < 0.01,
            "note {note}: got {hz}, want {want}"
        );
    }
}

#[test]
fn note_off_releases_and_a_shorter_release_is_quieter_sooner() {
    let mut p = plain_patch();
    p.release_ms = 400.0;
    let (held, slow) = render_held(&p, 57, 300, 600);
    p.release_ms = 20.0;
    let (_, fast) = render_held(&p, 57, 300, 600);
    let sustain = rms(&held[held.len() - 2000..]);
    let at = |x: &[f32], ms: usize| rms(&x[ms * 48..(ms + 20) * 48]);
    assert!(
        at(&fast, 150) < 0.05 * sustain,
        "fast release should be gone by 150 ms"
    );
    assert!(at(&slow, 150) > 3.0 * at(&fast, 150));
    assert!(at(&slow, 550) < 0.2 * sustain, "slow release still ends");
}

#[test]
fn chords_are_polyphonic_and_voices_free_up_after_release() {
    let patch = plain_patch();
    let mut one = SynthRt::new(SR, patch.clone());
    let mut chord = SynthRt::new(SR, patch);
    one.note_on(57, 1.0);
    for n in [57, 61, 64] {
        chord.note_on(n, 1.0);
    }
    let mut a = vec![0.0; 9600];
    let mut b = vec![0.0; 9600];
    one.process(&mut a);
    chord.process(&mut b);
    assert_eq!(chord.active_voices(), 3);
    assert!(rms(&b[4800..]) > rms(&a[4800..]));
    chord.all_notes_off();
    let mut tail = vec![0.0; 48_000];
    chord.process(&mut tail);
    assert_eq!(chord.active_voices(), 0);
}

#[test]
fn engine_is_deterministic_and_the_output_has_no_dc_offset() {
    let eng = SynthEngine::factory_seed();
    let mut p = eng.active_patch().clone();
    p.m1_program = Some(KorgM1Program::Piano16_01);
    p.m1_layer_mix = 0.8;
    let a = synthcraft_core::synthesize_audio_buffer(&p, 48_000, 600);
    let b = synthcraft_core::synthesize_audio_buffer(&p, 48_000, 600);
    assert_eq!(a, b);
    let mean = a.iter().sum::<f32>() / a.len() as f32;
    assert!(mean.abs() < 0.01, "dc offset {mean}");
}

#[test]
fn a_sounding_sequencer_step_stops_when_playback_stops() {
    let mut rt = SynthRt::new(SR, plain_patch());
    rt.set_playing(true);
    let mut buf = vec![0.0; 9600];
    rt.process(&mut buf);
    assert!(rms(&buf) > 0.01);
    rt.set_playing(false);
    let mut tail = vec![0.0; 48_000];
    rt.process(&mut tail);
    assert_eq!(rt.active_voices(), 0);
}
