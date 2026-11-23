//! Turn a UI edit of a patch into [`Command`]s (AGENTS rule 14: a panel can do nothing an
//! agent cannot do through the same commands).

use crate::{Command, SynthError, SynthPatch};

/// Scalar patch parameters settable through `Command::SetPatchParam`, with their limits.
pub const PATCH_PARAMS: &[(&str, f32, f32)] = &[
    ("bpm", 40.0, 240.0),
    ("pulse_width", 0.05, 0.95),
    ("master_gain", 0.0, 1.0),
    ("drive_saturation", 0.0, 3.0),
    ("m1_layer_mix", 0.0, 1.0),
    ("extreme_morph", 0.0, 1.0),
];

/// Set one scalar parameter (or `stepN.velocity` / `stepN.gate_pct`). Returns the stored value.
pub fn set_patch_param(patch: &mut SynthPatch, param: &str, value: f32) -> Result<f32, SynthError> {
    if !value.is_finite() {
        return Err(SynthError::Validation(format!(
            "{param}: value is not finite"
        )));
    }
    if let Some(rest) = param.strip_prefix("step") {
        let (idx, field) = rest
            .split_once('.')
            .ok_or_else(|| SynthError::Validation(format!("Unknown parameter '{param}'")))?;
        let i: usize = idx
            .parse()
            .map_err(|_| SynthError::Validation(format!("Unknown parameter '{param}'")))?;
        let step = patch
            .steps
            .get_mut(i)
            .ok_or_else(|| SynthError::Validation(format!("Step index {i} out of bounds")))?;
        return match field {
            "velocity" => {
                step.velocity = value.clamp(0.0, 1.0);
                Ok(step.velocity)
            }
            "gate_pct" => {
                step.gate_pct = value.clamp(0.1, 1.0);
                Ok(step.gate_pct)
            }
            _ => Err(SynthError::Validation(format!(
                "Unknown parameter '{param}'"
            ))),
        };
    }
    let Some((_, lo, hi)) = PATCH_PARAMS.iter().find(|(n, _, _)| *n == param) else {
        let names: Vec<&str> = PATCH_PARAMS.iter().map(|(n, _, _)| *n).collect();
        return Err(SynthError::Validation(format!(
            "Unknown parameter '{param}'. Expected one of {names:?}, stepN.velocity or stepN.gate_pct."
        )));
    };
    let v = value.clamp(*lo, *hi);
    match param {
        "bpm" => patch.bpm = v,
        "pulse_width" => patch.pulse_width = v,
        "master_gain" => patch.master_gain = v,
        "drive_saturation" => patch.drive_saturation = v,
        "m1_layer_mix" => patch.m1_layer_mix = v,
        _ => patch.extreme_morph = v,
    }
    Ok(v)
}

fn changed(a: f32, b: f32) -> bool {
    (a - b).abs() > f32::EPSILON
}

impl SynthPatch {
    /// The commands that turn `self` into `new`, for the fields a panel can edit.
    pub fn edit_commands(&self, new: &SynthPatch) -> Vec<Command> {
        let mut out = Vec::new();
        let (a, b) = (self, new);
        if a.vco1_wave != b.vco1_wave
            || a.vco2_wave != b.vco2_wave
            || changed(a.vco2_detune_cents, b.vco2_detune_cents)
            || changed(a.vco_mix, b.vco_mix)
            || changed(a.fm_index, b.fm_index)
        {
            out.push(Command::UpdateOscillators {
                vco1_wave: b.vco1_wave.as_str().to_string(),
                vco2_wave: b.vco2_wave.as_str().to_string(),
                detune_cents: b.vco2_detune_cents,
                mix: b.vco_mix,
                fm_index: b.fm_index,
            });
        }
        if a.filter_mode != b.filter_mode
            || changed(a.cutoff_hz, b.cutoff_hz)
            || changed(a.resonance_q, b.resonance_q)
            || changed(a.env_mod_depth, b.env_mod_depth)
        {
            out.push(Command::UpdateFilter {
                mode: b.filter_mode.as_str().to_string(),
                cutoff_hz: b.cutoff_hz,
                resonance_q: b.resonance_q,
                env_mod_depth: b.env_mod_depth,
            });
        }
        if changed(a.attack_ms, b.attack_ms)
            || changed(a.decay_ms, b.decay_ms)
            || changed(a.sustain_level, b.sustain_level)
            || changed(a.release_ms, b.release_ms)
            || changed(a.lfo_rate_hz, b.lfo_rate_hz)
            || changed(a.lfo_depth, b.lfo_depth)
        {
            out.push(Command::UpdateEnvelopeLfo {
                attack_ms: b.attack_ms,
                decay_ms: b.decay_ms,
                sustain_level: b.sustain_level,
                release_ms: b.release_ms,
                lfo_rate_hz: b.lfo_rate_hz,
                lfo_depth: b.lfo_depth,
            });
        }
        if a.m1_program != b.m1_program || changed(a.m1_layer_mix, b.m1_layer_mix) {
            out.push(Command::LoadPcmProgram {
                program: b.m1_program.map_or("off", |p| p.as_str()).to_string(),
                mix: b.m1_layer_mix,
            });
        }
        if a.extreme_dsp != b.extreme_dsp || changed(a.extreme_morph, b.extreme_morph) {
            out.push(Command::SetExtremeDsp {
                mode: b.extreme_dsp.as_str().to_string(),
                morph: b.extreme_morph,
            });
        }
        if a.sampler != b.sampler {
            out.push(Command::SetSamplerBank {
                bank: b.sampler.bank.clone(),
                mix: b.sampler.mix,
            });
        }
        if a.fx != b.fx {
            out.push(Command::SetFx {
                chorus_mix: b.fx.chorus_mix,
                reverb_mix: b.fx.reverb_mix,
                reverb_decay_s: b.fx.reverb_decay_s,
                reverb_damping: b.fx.reverb_damping,
            });
        }
        for (name, x, y) in [
            ("bpm", a.bpm, b.bpm),
            ("pulse_width", a.pulse_width, b.pulse_width),
            ("master_gain", a.master_gain, b.master_gain),
            ("drive_saturation", a.drive_saturation, b.drive_saturation),
        ] {
            if changed(x, y) {
                out.push(Command::SetPatchParam {
                    param: name.to_string(),
                    value: y,
                });
            }
        }
        if a.camelot_key != b.camelot_key {
            out.push(Command::SetCamelotKey {
                key: b.camelot_key.clone(),
            });
        }
        for (i, (x, y)) in a.steps.iter().zip(&b.steps).enumerate() {
            if x.midi_note != y.midi_note
                || x.accent != y.accent
                || x.slide != y.slide
                || x.active != y.active
            {
                out.push(Command::SetSequencerStep {
                    step_index: i,
                    midi_note: y.midi_note,
                    accent: y.accent,
                    slide: y.slide,
                    active: y.active,
                });
            }
            if changed(x.velocity, y.velocity) {
                out.push(Command::SetPatchParam {
                    param: format!("step{i}.velocity"),
                    value: y.velocity,
                });
            }
            if changed(x.gate_pct, y.gate_pct) {
                out.push(Command::SetPatchParam {
                    param: format!("step{i}.gate_pct"),
                    value: y.gate_pct,
                });
            }
        }
        out
    }
}
