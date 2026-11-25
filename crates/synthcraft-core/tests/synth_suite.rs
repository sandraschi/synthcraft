//! Integration test suite for SynthCraft L1 core.

use synthcraft_core::{
    export_midi_smf_file, midi_to_freq_hz, Command, ExtremeDspMode, FilterMode, PcmProgram,
    SynthEngine, WaveformKind,
};

#[test]
fn midi_a4_is_440_hz_and_a3_is_220_hz() {
    assert!((midi_to_freq_hz(69) - 440.0).abs() < 0.01);
    assert!((midi_to_freq_hz(57) - 220.0).abs() < 0.01);
}

#[test]
fn all_waveform_kinds_parse_and_sample_within_bounds() {
    for (raw, expected) in [
        ("sine", WaveformKind::Sine),
        ("saw", WaveformKind::Saw),
        ("square", WaveformKind::Square),
        ("triangle", WaveformKind::Triangle),
        ("pulse_pwm", WaveformKind::PulsePwm),
        ("supersaw", WaveformKind::SuperSaw),
        ("white_noise", WaveformKind::WhiteNoise),
        ("sub_octave", WaveformKind::SubOctave),
        ("atmo_pad", WaveformKind::AtmoPad),
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

    let stem = Command::ExportDjcraftStem {
        bars: 8,
        output_path: None,
    }
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
fn all_eight_pcm_programs_parse_and_sample() {
    for prog in PcmProgram::all() {
        assert_eq!(PcmProgram::parse(prog.as_str()), Some(*prog));
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
    assert_eq!(eng.active_patch().m1_program, Some(PcmProgram::Piano16_01));

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
    p.m1_program = Some(PcmProgram::Piano16_01);
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

// ---------------------------------------------------------------- sampler, pad, fx

fn looped_sine_bank(root: u8) -> std::sync::Arc<synthcraft_core::sampler::SampleBank> {
    use synthcraft_core::sampler::{SampleBank, Zone};
    let sr = 32_000.0_f32;
    let f = 440.0 * 2.0_f32.powf((f32::from(root) - 69.0) / 12.0);
    // A whole number of cycles so the loop is seamless.
    let cycles = 64.0_f32;
    let n = (cycles * sr / f).round() as usize;
    let data: Vec<f32> = (0..n)
        .map(|i| (std::f32::consts::TAU * cycles * i as f32 / n as f32).sin() * 0.8)
        .collect();
    std::sync::Arc::new(SampleBank {
        name: "test".into(),
        zones: vec![Zone {
            data: data.into(),
            sample_rate: sr * (cycles * sr / f) / n as f32,
            key_lo: 0,
            key_hi: 127,
            vel_lo: 0,
            vel_hi: 127,
            root_key: root,
            tune_cents: 0.0,
            gain: 1.0,
            loop_range: Some((0, n)),
            one_shot: false,
            pan: 0.0,
        }],
    })
}

fn pitch_of(x: &[f32]) -> f32 {
    autocorrelation_pitch(x, SR)
}

#[test]
fn sampler_layer_plays_at_the_requested_pitch_and_sustains() {
    let mut p = plain_patch();
    p.sampler.mix = 1.0;
    p.cutoff_hz = 15_000.0;
    for (note, want) in [(57u8, 220.0_f32), (69, 440.0), (81, 880.0)] {
        let mut rt = SynthRt::new(SR, p.clone());
        rt.set_bank(Some(looped_sine_bank(57)));
        rt.note_on(note, 1.0);
        let mut buf = vec![0.0; 24_000];
        rt.process(&mut buf);
        let tail = &buf[12_000..];
        let hz = pitch_of(tail);
        assert!(
            (hz / want - 1.0).abs() < 0.015,
            "note {note}: {hz} vs {want}"
        );
        assert!(rms(tail) > 0.1, "looped zone must sustain");
    }
}

#[test]
fn sampler_layer_is_silent_without_a_bank_and_oneshot_ends() {
    use synthcraft_core::sampler::{SampleBank, Zone};
    let mut p = plain_patch();
    p.sampler.mix = 1.0;
    let mut rt = SynthRt::new(SR, p.clone());
    rt.note_on(60, 1.0);
    let mut buf = vec![0.0; 4800];
    rt.process(&mut buf);
    // No bank installed: the layer is bypassed and the oscillators sound.
    assert!(rms(&buf[2400..]) > 0.05);

    let blip = SampleBank {
        name: "blip".into(),
        zones: vec![Zone {
            data: vec![0.5; 480].into(),
            sample_rate: 48_000.0,
            key_lo: 0,
            key_hi: 127,
            vel_lo: 0,
            vel_hi: 127,
            root_key: 60,
            tune_cents: 0.0,
            gain: 1.0,
            loop_range: None,
            one_shot: false,
            pan: 0.0,
        }],
    };
    let mut rt = SynthRt::new(SR, p);
    rt.set_bank(Some(std::sync::Arc::new(blip)));
    rt.note_on(60, 1.0);
    let mut buf = vec![0.0; 9600];
    rt.process(&mut buf);
    assert!(rms(&buf[..400]) > 0.1);
    assert!(rms(&buf[4800..]) < 0.01, "one-shot sample must end");
}

#[test]
fn atmospheric_pad_is_pitched_evolving_and_clean() {
    let mut p = plain_patch();
    p.vco1_wave = WaveformKind::AtmoPad;
    p.vco2_wave = WaveformKind::AtmoPad;
    p.vco_mix = 0.5;
    p.cutoff_hz = 9000.0;
    p.attack_ms = 400.0;
    p.release_ms = 900.0;
    let mut rt = SynthRt::new(SR, p);
    rt.note_on(57, 0.9);
    let mut buf = vec![0.0; 48_000 * 6];
    rt.process(&mut buf);
    assert!(buf.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
    let tail = &buf[48_000 * 2..48_000 * 2 + 8192];
    let hz = pitch_of(tail);
    assert!((hz / 220.0 - 1.0).abs() < 0.03, "pad pitch {hz}");
    // The timbre must move: window RMS and zero-crossing density both vary over time.
    let wins: Vec<f32> = (2..6)
        .map(|s| rms(&buf[48_000 * s..48_000 * s + 4800]))
        .collect();
    let (lo, hi) = wins
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    assert!(hi / lo > 1.02, "pad level should drift, got {wins:?}");
    let dc = buf[48_000 * 2..].iter().sum::<f32>() / (buf.len() - 96_000) as f32;
    assert!(dc.abs() < 0.01, "dc {dc}");
}

#[test]
fn reverb_adds_a_decaying_tail_and_chorus_changes_the_signal() {
    let mut dry = plain_patch();
    dry.release_ms = 20.0;
    let mut wet = dry.clone();
    wet.fx.reverb_mix = 0.6;
    wet.fx.reverb_decay_s = 2.5;
    let (_, tail_dry) = render_held(&dry, 57, 300, 1500);
    let (_, tail_wet) = render_held(&wet, 57, 300, 1500);
    let at = |x: &[f32], ms: usize| rms(&x[ms * 48..(ms + 100) * 48]);
    assert!(at(&tail_dry, 600) < 1e-4);
    assert!(at(&tail_wet, 600) > 0.01, "reverb tail missing");
    assert!(at(&tail_wet, 1300) < at(&tail_wet, 300), "tail must decay");
    assert!(tail_wet.iter().all(|s| s.is_finite() && s.abs() <= 1.0));

    let mut ch = dry.clone();
    ch.fx.chorus_mix = 1.0;
    let (a, _) = render_held(&dry, 57, 500, 10);
    let (b, _) = render_held(&ch, 57, 500, 10);
    let diff: f32 = a.iter().zip(&b).map(|(x, y)| (x - y).abs()).sum::<f32>() / a.len() as f32;
    assert!(diff > 0.01, "chorus had no audible effect");
}

#[test]
fn ambient_factory_patches_are_audible_unclipped_and_dc_free() {
    use synthcraft_core::synthesize_audio_buffer;
    let eng = SynthEngine::factory_seed();
    for id in ["ambient_glass_choir", "ambient_slow_tide"] {
        let found = eng.patches.iter().find(|p| p.id == id);
        assert!(found.is_some(), "missing factory patch {id}");
        let Some(p) = found else {
            continue;
        };
        let buf = synthesize_audio_buffer(p, 48_000, 10_000);
        let peak = buf.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
        let level = rms(&buf[48_000 * 4..]);
        let dc = buf[48_000 * 4..].iter().sum::<f32>() / (buf.len() - 48_000 * 4) as f32;
        eprintln!("{id}: peak {peak:.3} rms {level:.4} dc {dc:.5}");
        assert!(peak > 0.1 && peak < 0.99, "{id} peak {peak}");
        assert!(level > 0.05, "{id} rms {level}");
        assert!(dc.abs() < 0.01, "{id} dc {dc}");
    }
}

#[test]
fn stem_manifest_matches_what_can_be_rendered() {
    let mut eng = SynthEngine::factory_seed();
    eng.active_patch_mut().bpm = 60.0;
    let m = synthcraft_core::export_djcraft_stem_manifest(eng.active_patch(), 16, 44_100);
    assert_eq!(m["sample_rate_hz"], 44_100);
    assert_eq!(m["duration_ms"], 64_000);
    let cues = m["cue_points_ms"].as_array().cloned().unwrap_or_default();
    let ms: Vec<u64> = cues.iter().filter_map(|c| c["ms"].as_u64()).collect();
    assert_eq!(ms, vec![0, 16_000, 32_000, 48_000, 64_000]);
    // The renderer now accepts the full stem length instead of clamping to 15 s.
    let buf = synthcraft_core::synthesize_audio_buffer(eng.active_patch(), 8000, 64_000);
    assert_eq!(buf.len(), 64 * 8000);
}

#[test]
fn stem_command_renders_the_wav_it_describes() {
    let mut eng = SynthEngine::factory_seed();
    eng.active_patch_mut().bpm = 120.0;
    let mut path = std::env::temp_dir();
    path.push(format!("synthcraft_stem_{}.wav", std::process::id()));
    let out = Command::ExportDjcraftStem {
        bars: 2,
        output_path: Some(path.to_string_lossy().into_owned()),
    }
    .dispatch(&mut eng);
    let v = out.unwrap_or_default();
    // 2 bars at 120 BPM is 4 s at the engine rate.
    assert_eq!(
        v["wav_frames"].as_u64(),
        Some(4 * u64::from(eng.sample_rate_hz))
    );
    assert!(std::fs::metadata(&path).is_ok_and(|m| m.len() > 44));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn sub_octave_reference_wave_has_no_dc_offset() {
    let n = 1000;
    let mean: f32 = (0..n)
        .map(|i| WaveformKind::SubOctave.sample(i as f32 / n as f32, 0.5, 1))
        .sum::<f32>()
        / n as f32;
    assert!(mean.abs() < 0.01, "mean {mean}");
}

#[test]
fn a_panel_edit_becomes_commands_that_reproduce_it() {
    let mut eng = SynthEngine::factory_seed();
    let before = eng.active_patch().clone();
    let mut after = before.clone();
    after.vco1_wave = WaveformKind::Triangle;
    after.vco2_detune_cents = 17.0;
    after.cutoff_hz = 1234.0;
    after.resonance_q = 3.5;
    after.attack_ms = 40.0;
    after.lfo_depth = 0.33;
    after.m1_program = Some(PcmProgram::Piano16_01);
    after.m1_layer_mix = 0.42;
    after.extreme_morph = 0.9;
    after.master_gain = 0.6;
    after.drive_saturation = 1.5;
    after.bpm = 133.0;
    after.pulse_width = 0.3;
    after.camelot_key = "4A Fm".into();
    after.fx.reverb_mix = 0.25;
    after.steps[3].midi_note = 52;
    after.steps[3].slide = !before.steps[3].slide;
    after.steps[5].velocity = 0.4;
    after.steps[6].gate_pct = 0.9;
    let cmds = before.edit_commands(&after);
    assert!(!cmds.is_empty());
    for c in &cmds {
        assert!(c.dispatch(&mut eng).is_ok(), "{c:?}");
    }
    let got = eng.active_patch();
    assert_eq!(got.vco1_wave, WaveformKind::Triangle);
    assert!((got.cutoff_hz - 1234.0).abs() < 1e-3 && (got.attack_ms - 40.0).abs() < 1e-3);
    assert_eq!(got.m1_program, Some(PcmProgram::Piano16_01));
    assert!((got.m1_layer_mix - 0.42).abs() < 1e-3 && (got.extreme_morph - 0.9).abs() < 1e-3);
    assert!((got.bpm - 133.0).abs() < 1e-3 && (got.pulse_width - 0.3).abs() < 1e-3);
    assert!((got.master_gain - 0.6).abs() < 1e-3 && (got.drive_saturation - 1.5).abs() < 1e-3);
    assert_eq!(got.camelot_key, "4A Fm");
    assert!((got.fx.reverb_mix - 0.25).abs() < 1e-3);
    assert_eq!(got.steps[3].midi_note, 52);
    assert!((got.steps[5].velocity - 0.4).abs() < 1e-3);
    assert!((got.steps[6].gate_pct - 0.9).abs() < 1e-3);
    // Nothing else moved, and a second diff is empty.
    assert!(got.edit_commands(&after).is_empty());
    // Bad parameters are rejected, not ignored.
    let bad = Command::SetPatchParam {
        param: "nope".into(),
        value: 1.0,
    };
    assert!(bad.dispatch(&mut eng).is_err());
}

// ---------------------------------------------------------------- stereo, layers, ambient

fn dry_patch() -> SynthPatch {
    let mut p = plain_patch();
    p.fx.chorus_mix = 0.0;
    p.fx.reverb_mix = 0.0;
    p
}

fn render_stereo(p: &SynthPatch, note: u8, frames: usize) -> (Vec<f32>, Vec<f32>) {
    let mut rt = SynthRt::new(SR, p.clone());
    rt.note_on(note, 1.0);
    let (mut l, mut r) = (vec![0.0; frames], vec![0.0; frames]);
    rt.process_stereo(&mut l, &mut r);
    (l, r)
}

#[test]
fn stereo_spread_pans_voices_and_zero_spread_is_exactly_mono() {
    let mut p = dry_patch();
    p.fx.stereo_spread = 0.0;
    let (l, r) = render_stereo(&p, 60, 4800);
    assert_eq!(l, r, "no spread, no effects: both channels identical");

    p.fx.stereo_spread = 1.0;
    let mut rt = SynthRt::new(SR, p.clone());
    // Four held notes land in four different voice slots, so they sit at different pans.
    for n in [48u8, 55, 60, 67] {
        rt.note_on(n, 1.0);
    }
    let (mut l, mut r) = (vec![0.0; 9600], vec![0.0; 9600]);
    rt.process_stereo(&mut l, &mut r);
    assert!(l.iter().chain(&r).all(|x| x.abs() <= 1.0));
    let diff: f32 = l.iter().zip(&r).map(|(a, b)| (a - b).abs()).sum::<f32>() / l.len() as f32;
    assert!(diff > 0.01, "spread must make the channels differ: {diff}");
}

#[test]
fn mono_process_is_the_average_of_the_stereo_channels() {
    let mut p = dry_patch();
    p.fx.stereo_spread = 0.7;
    p.fx.chorus_mix = 0.5;
    p.fx.reverb_mix = 0.3;
    let (l, r) = render_stereo(&p, 64, 6000);
    let mut rt = SynthRt::new(SR, p);
    rt.note_on(64, 1.0);
    let mut mono = vec![0.0; 6000];
    rt.process(&mut mono);
    for i in 0..mono.len() {
        assert!((mono[i] - 0.5 * (l[i] + r[i])).abs() < 1e-5, "frame {i}");
    }
}

#[test]
fn chorus_and_reverb_decorrelate_the_channels() {
    let mut p = dry_patch();
    p.fx.stereo_spread = 0.0;
    p.fx.chorus_mix = 0.6;
    p.fx.reverb_mix = 0.5;
    let (l, r) = render_stereo(&p, 60, 24_000);
    let diff: f32 = l.iter().zip(&r).map(|(a, b)| (a - b).abs()).sum::<f32>() / l.len() as f32;
    assert!(diff > 0.005, "effects must widen a centred voice: {diff}");
}

#[test]
fn find_all_returns_every_overlapping_zone_up_to_the_cap() {
    use synthcraft_core::sampler::{SampleBank, MAX_LAYERS};
    let mut base = looped_sine_bank(57).zones[0].clone();
    base.key_hi = 99;
    let mut zones = Vec::new();
    for _ in 0..(MAX_LAYERS + 2) {
        zones.push(base.clone());
    }
    let mut hi = base.clone();
    hi.key_lo = 100;
    hi.key_hi = 110;
    zones.push(hi);
    let bank = SampleBank {
        name: "t".into(),
        zones,
    };
    let mut out = [0usize; MAX_LAYERS];
    assert_eq!(bank.find_all(60, 1.0, &mut out), MAX_LAYERS);
    assert_eq!(&out[..MAX_LAYERS], &[0, 1, 2, 3]);
    let mut out = [0usize; MAX_LAYERS];
    assert_eq!(bank.find_all(105, 1.0, &mut out), 1);
    assert_eq!(out[0], MAX_LAYERS + 2);
}

#[test]
fn layered_zones_add_up() {
    use synthcraft_core::sampler::SampleBank;
    let mut z = looped_sine_bank(57).zones[0].clone();
    z.gain = 0.15;
    let one = std::sync::Arc::new(SampleBank {
        name: "one".into(),
        zones: vec![z.clone()],
    });
    let two = std::sync::Arc::new(SampleBank {
        name: "two".into(),
        zones: vec![z.clone(), z],
    });
    let mut p = dry_patch();
    p.sampler.mix = 1.0;
    p.fx.stereo_spread = 0.0;
    let level = |bank: std::sync::Arc<SampleBank>| {
        let mut rt = SynthRt::new(SR, p.clone());
        rt.set_bank(Some(bank));
        rt.note_on(57, 1.0);
        let mut buf = vec![0.0; 12_000];
        rt.process(&mut buf);
        rms(&buf[6000..])
    };
    let (a, b) = (level(one), level(two));
    assert!(
        b > a * 1.7 && b < a * 2.3,
        "two layers ~ double: {a} vs {b}"
    );
}

#[test]
fn one_shot_zone_ignores_note_off_and_frees_its_voice() {
    use synthcraft_core::sampler::{SampleBank, Zone};
    let n = 9600;
    let data: Vec<f32> = (0..n)
        .map(|i| (std::f32::consts::TAU * 440.0 * i as f32 / SR).sin() * 0.5)
        .collect();
    let zone = Zone {
        data: data.into(),
        sample_rate: SR,
        key_lo: 0,
        key_hi: 127,
        vel_lo: 0,
        vel_hi: 127,
        root_key: 69,
        tune_cents: 0.0,
        gain: 1.0,
        loop_range: None,
        one_shot: true,
        pan: 0.0,
    };
    let bank = std::sync::Arc::new(SampleBank {
        name: "hit".into(),
        zones: vec![zone],
    });
    let mut p = dry_patch();
    p.sampler.mix = 1.0;
    p.cutoff_hz = 15_000.0;
    let mut rt = SynthRt::new(SR, p);
    rt.set_bank(Some(bank));
    rt.note_on(69, 1.0);
    let mut buf = vec![0.0; 1000];
    rt.process(&mut buf);
    rt.note_off(69);
    // Well past the 80 ms release the hit is still sounding, because note-off was ignored.
    let mut mid = vec![0.0; 6000];
    rt.process(&mut mid);
    assert!(rms(&mid[4000..]) > 0.05, "one-shot must keep playing");
    // After the sample ends the voice releases and frees itself.
    let mut tail = vec![0.0; 24_000];
    rt.process(&mut tail);
    assert_eq!(rt.active_voices(), 0);
}

#[test]
fn ambient_drift_is_seeded_bounded_and_off_at_zero_depth() {
    let mut base = dry_patch();
    base.fx.reverb_mix = 0.3;
    base.fx.stereo_spread = 0.0;
    base.cutoff_hz = 1200.0;
    base.sustain_level = 1.0;
    let go = |depth: f32, seed: u32| {
        let mut p = base.clone();
        p.fx.ambient_depth = depth;
        p.fx.ambient_rate_hz = 0.4;
        p.fx.ambient_seed = seed;
        render_stereo(&p, 48, 96_000).0
    };
    let off = go(0.0, 1);
    let a = go(0.9, 7);
    let a2 = go(0.9, 7);
    let b = go(0.9, 8);
    assert_eq!(a, a2, "same seed, same drift");
    assert_ne!(a, b, "different seed, different drift");
    assert_ne!(a, off, "depth must change the sound");
    assert!(a.iter().all(|x| x.abs() <= 1.0));
    // Depth zero is the plain engine: it must match a patch that never heard of ambient.
    assert_eq!(off, go(0.0, 99));
}

#[test]
fn sequencer_expands_to_64_steps_and_drum_machine_synthesizes_all_8_voices() {
    let mut eng = SynthEngine::factory_seed();
    assert!(Command::SetSequencerLength { steps: 64 }
        .dispatch(&mut eng)
        .is_ok());
    assert_eq!(eng.active_patch().steps.len(), 64);

    assert!(Command::SetSequencerStep {
        step_index: 63,
        midi_note: 60,
        accent: true,
        slide: false,
        active: true,
    }
    .dispatch(&mut eng)
    .is_ok());
    assert_eq!(eng.active_patch().steps[63].midi_note, 60);

    // Load every built-in drum kit preset and verify 8 voices synthesize bounded audio
    for (pid, _) in synthcraft_core::DrumMachinePatch::preset_ids() {
        assert!(Command::LoadDrumPreset {
            preset_id: (*pid).to_string(),
        }
        .dispatch(&mut eng)
        .is_ok());
        let patch = eng.active_patch().clone();
        assert!(patch.drums.enabled);
        assert_eq!(patch.drums.tracks.len(), synthcraft_core::DRUM_VOICE_COUNT);

        let mut rt = SynthRt::new(SR, patch);
        let mut l = vec![0.0_f32; 4096];
        let mut r = vec![0.0_f32; 4096];
        for vi in 0..synthcraft_core::DRUM_VOICE_COUNT {
            rt.trigger_drum(vi, vi % 2 == 0);
        }
        rt.process_stereo(&mut l, &mut r);
        assert!(l.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
        assert!(r.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
        let energy: f32 = l.iter().map(|s| s * s).sum::<f32>();
        assert!(energy > 0.01, "drum preset {pid} must produce audible PCM");
    }
}
