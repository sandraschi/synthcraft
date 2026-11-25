//! Safe, host-agnostic plugin engine.
//!
//! `PluginEngine` is what the CLAP plugin, the C ABI (`synthcraft-ffi`) and the VCV Rack
//! module all drive. It wraps [`SynthRt`], the same engine the desktop app and the offline
//! renderer use, and adds what a plug-in host needs: parameter access by stable id, block
//! processing into stereo buffers of any length, and JSON state.
//!
//! Threading: `note_*`, `set_param`, `process` and the getters never allocate and are safe to
//! call from an audio thread. `load_bank`, `load_state_json`, `state_json` and
//! `set_sample_rate` allocate and do I/O; call them from a non-audio thread while the audio
//! thread is not inside `process` (the caller owns that exclusion).
//!
//! The engine is mono internally (as is `SynthRt` today); both output channels carry the
//! same signal.

use crate::error::SynthError;
use crate::params::{get_param, set_param};
use crate::rt::SynthRt;
use crate::sampler::load_cached;
use crate::{SynthEngine, SynthPatch};

/// Largest chunk rendered in one call to the underlying engine.
pub const MAX_BLOCK: usize = 1024;

/// Host-facing synth instance.
pub struct PluginEngine {
    rt: SynthRt,
}

impl PluginEngine {
    /// New instance with the factory lead patch at `sample_rate`.
    pub fn new(sample_rate: f32) -> Self {
        Self::from_patch(
            sample_rate,
            SynthEngine::factory_seed().active_patch().clone(),
        )
    }

    pub fn from_patch(sample_rate: f32, patch: SynthPatch) -> Self {
        Self {
            rt: SynthRt::new(sample_rate, patch),
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.rt.sample_rate()
    }

    pub fn patch(&self) -> &SynthPatch {
        self.rt.patch()
    }

    pub fn patch_mut(&mut self) -> &mut SynthPatch {
        self.rt.patch_mut()
    }

    /// Replace the active patch in place. Call from the main thread while `process` is not running.
    pub fn replace_patch(&mut self, patch: SynthPatch) {
        let _ = self.rt.replace_patch(patch);
    }

    /// Rebuild the engine at a new rate, keeping patch and bank. Sounding notes are cut.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        if (sample_rate - self.rt.sample_rate()).abs() < 0.5 {
            return;
        }
        let patch = self.rt.patch().clone();
        let opts = self.rt.options();
        let bank = self.rt.set_bank(None);
        self.rt = SynthRt::new(sample_rate, patch);
        self.rt.set_options(opts);
        self.rt.set_bank(bank);
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        if velocity <= 0.0 {
            self.rt.note_off(note);
        } else {
            self.rt.note_on(note.min(127), velocity);
        }
    }

    pub fn note_off(&mut self, note: u8) {
        self.rt.note_off(note.min(127));
    }

    pub fn all_notes_off(&mut self) {
        self.rt.all_notes_off();
    }

    /// Pitch bend in semitones.
    pub fn pitch_bend(&mut self, semitones: f32) {
        self.rt.pitch_bend(semitones);
    }

    pub fn mod_wheel(&mut self, value: f32) {
        self.rt.set_mod_wheel(value);
    }

    /// Limit polyphony (1 to `MAX_VOICES`). Allocation-free.
    pub fn set_max_voices(&mut self, voices: usize) {
        let mut opts = self.rt.options();
        opts.max_voices = voices;
        self.rt.set_options(opts);
    }

    pub fn active_voices(&self) -> usize {
        self.rt.active_voices()
    }

    pub fn set_note_expression(
        &mut self,
        note: Option<u8>,
        pitch_st: Option<f32>,
        timbre: Option<f32>,
        pressure: Option<f32>,
    ) {
        self.rt
            .set_note_expression(note, pitch_st, timbre, pressure);
    }

    pub fn set_poly_param_mod(&mut self, note: Option<u8>, param_id: u32, delta: f32) {
        self.rt.set_poly_param_mod(note, param_id, delta);
    }

    /// Set a parameter by id. Returns the clamped value, or `None` for an unknown id.
    pub fn set_param(&mut self, id: u32, value: f32) -> Option<f32> {
        set_param(self.rt.patch_mut(), id, value)
    }

    pub fn get_param(&self, id: u32) -> Option<f32> {
        get_param(self.rt.patch(), id)
    }

    /// Render `left.len().min(right.len())` frames. Does not allocate.
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let frames = left.len().min(right.len());
        let mut done = 0;
        while done < frames {
            let n = (frames - done).min(MAX_BLOCK);
            self.rt
                .process_stereo(&mut left[done..done + n], &mut right[done..done + n]);
            done += n;
        }
    }

    /// Load a sample bank (WAV, folder, SFZ or SF2 path) and select it. Not real-time safe.
    pub fn load_bank(&mut self, spec: &str) -> Result<(), SynthError> {
        let bank = load_cached(spec)?;
        self.rt.set_bank(Some(bank));
        self.rt.patch_mut().sampler.bank = Some(spec.to_string());
        Ok(())
    }

    /// Remove the sample bank.
    pub fn clear_bank(&mut self) {
        self.rt.set_bank(None);
        self.rt.patch_mut().sampler.bank = None;
    }

    pub fn has_bank(&self) -> bool {
        self.rt.has_bank()
    }

    /// The whole patch as JSON. Hosts store this as plugin state.
    pub fn state_json(&self) -> Result<String, SynthError> {
        serde_json::to_string(self.rt.patch()).map_err(|e| SynthError::Validation(e.to_string()))
    }

    /// Restore a patch from [`Self::state_json`]. If the patch names a sample bank it is
    /// loaded; a bank that fails to load leaves the patch in place without a bank and returns
    /// the error text in `Ok(Some(..))` so the caller can report it.
    pub fn load_state_json(&mut self, json: &str) -> Result<Option<String>, SynthError> {
        let patch: SynthPatch = serde_json::from_str(json)
            .map_err(|e| SynthError::Validation(format!("invalid state: {e}")))?;
        let spec = patch.sampler.bank.clone();
        self.rt.set_bank(None);
        self.rt.replace_patch(patch);
        let mut warning = None;
        if let Some(spec) = spec {
            match load_cached(&spec) {
                Ok(b) => {
                    self.rt.set_bank(Some(b));
                }
                Err(e) => warning = Some(format!("bank '{spec}' not loaded: {e}")),
            }
        }
        Ok(warning)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::PARAMS;

    fn energy(l: &[f32]) -> f32 {
        l.iter().map(|x| x * x).sum::<f32>()
    }

    #[test]
    fn silent_until_a_note_then_sounds_then_releases() {
        let mut e = PluginEngine::new(48_000.0);
        let (mut l, mut r) = (vec![0.0; 3000], vec![0.0; 3000]);
        e.process(&mut l, &mut r);
        assert!(energy(&l) < 1e-9);
        e.note_on(60, 0.9);
        e.process(&mut l, &mut r);
        assert!(energy(&l) > 0.01);
        assert!(energy(&r) > 0.01);
        e.note_off(60);
        for _ in 0..40 {
            e.process(&mut l, &mut r);
        }
        assert_eq!(e.active_voices(), 0);
    }

    #[test]
    fn long_buffers_are_chunked_and_match_one_shot_rendering() {
        let mut a = PluginEngine::new(44_100.0);
        let mut b = PluginEngine::new(44_100.0);
        a.note_on(48, 0.8);
        b.note_on(48, 0.8);
        let n = MAX_BLOCK * 2 + 123;
        let (mut al, mut ar) = (vec![0.0; n], vec![0.0; n]);
        a.process(&mut al, &mut ar);
        let (mut bl, mut br) = (vec![0.0; n], vec![0.0; n]);
        let mut off = 0;
        while off < n {
            let m = 100.min(n - off);
            b.process(&mut bl[off..off + m], &mut br[off..off + m]);
            off += m;
        }
        let max_diff = al
            .iter()
            .zip(&bl)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f32::max);
        assert!(max_diff < 1e-4, "block size changed the sound: {max_diff}");
    }

    #[test]
    fn params_change_the_sound_and_are_readable() {
        let mut e = PluginEngine::new(48_000.0);
        assert_eq!(e.set_param(0x1004, 99_999.0), Some(16_000.0));
        assert_eq!(e.get_param(0x1004), Some(16_000.0));
        assert_eq!(e.set_param(0xBEEF, 1.0), None);
        let render = |cut: f32| {
            let mut e = PluginEngine::new(48_000.0);
            e.set_param(0x1004, cut);
            e.set_param(0x1006, 0.0);
            e.note_on(60, 1.0);
            let (mut l, mut r) = (vec![0.0; 9600], vec![0.0; 9600]);
            e.process(&mut l, &mut r);
            energy(&l)
        };
        assert!(render(16_000.0) > render(80.0) * 1.5);
        for d in PARAMS {
            assert!(e.get_param(d.id).is_some());
        }
    }

    #[test]
    fn state_round_trips_through_json() {
        let mut a = PluginEngine::new(48_000.0);
        a.set_param(0x1004, 777.0);
        a.set_param(0x1014, 0.4);
        let json = a.state_json().unwrap_or_default();
        let mut b = PluginEngine::new(48_000.0);
        assert_eq!(b.load_state_json(&json).ok(), Some(None));
        assert_eq!(b.get_param(0x1004), Some(777.0));
        assert_eq!(b.get_param(0x1014), Some(0.4));
        assert!(b.load_state_json("{not json").is_err());
    }

    #[test]
    fn polyphony_limit_is_clamped_and_respected() {
        let mut e = PluginEngine::new(48_000.0);
        e.set_max_voices(2);
        for n in [60, 64, 67, 71] {
            e.note_on(n, 0.8);
        }
        assert_eq!(e.active_voices(), 2);
        e.set_max_voices(10_000);
        for n in 40..60 {
            e.note_on(n, 0.8);
        }
        assert!(e.active_voices() <= crate::MAX_VOICES);
    }

    #[test]
    fn sample_rate_change_keeps_patch() {
        let mut e = PluginEngine::new(48_000.0);
        e.set_param(0x1005, 7.0);
        e.set_sample_rate(96_000.0);
        assert_eq!(e.sample_rate(), 96_000.0);
        assert_eq!(e.get_param(0x1005), Some(7.0));
    }

    #[test]
    fn missing_bank_is_an_error_not_a_panic() {
        let mut e = PluginEngine::new(48_000.0);
        assert!(e.load_bank("definitely/not/here.sfz").is_err());
        assert!(!e.has_bank());
    }
}
