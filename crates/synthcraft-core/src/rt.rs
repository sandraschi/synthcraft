//! Real-time polyphonic voice engine.
//!
//! `SynthRt` is the one synthesis path in SynthCraft. The audio callback in the desktop app
//! and the offline renderer (`synthesize_audio_buffer`, WAV export, analysis) both call
//! [`SynthRt::process`], so what you hear is what you export.
//!
//! The engine owns no threads and does no I/O. `process` does not allocate.

use crate::dsp::{Adsr, Chorus, DcBlocker, Drift, Ladder, Osc, Reverb};
use crate::model::midi_to_freq_hz as midi_hz;
use crate::model::SynthPatch;
use crate::sampler::{SampleBank, MAX_LAYERS};
use std::f32::consts::{FRAC_PI_4, SQRT_2, TAU};
use std::sync::Arc;

const NO_ZONE: usize = usize::MAX;

pub const MAX_VOICES: usize = 16;
const GLIDE_MS: f32 = 55.0;

/// One sounding zone of a voice (a layer).
#[derive(Debug, Clone, Copy)]
struct Layer {
    zone: usize,
    pos: f64,
    step: f32,
}

const IDLE_LAYER: Layer = Layer {
    zone: NO_ZONE,
    pos: 0.0,
    step: 0.0,
};

#[derive(Debug, Clone, Copy)]
struct Voice {
    active: bool,
    gate: bool,
    from_seq: bool,
    note: u8,
    age_order: u64,
    freq: f32,
    target_freq: f32,
    vel: f32,
    accent: bool,
    osc1: Osc,
    osc2: Osc,
    layer_m1_phase: f64,
    layer_x_phase: f64,
    layers: [Layer; MAX_LAYERS],
    n_layers: usize,
    layers_b: [Layer; MAX_LAYERS],
    n_layers_b: usize,
    dac_phase: f32,
    dac_held: f32,
    /// A one-shot zone is sounding: note-off is ignored until the sample ends.
    hold: bool,
    /// Fixed position of this voice slot in the stereo field, -1..=1.
    pan: f32,
    time_s: f32,
    env: Adsr,
    filter: Ladder,
}

impl Voice {
    fn new(i: usize) -> Self {
        Self {
            active: false,
            gate: false,
            from_seq: false,
            note: 0,
            age_order: 0,
            freq: 110.0,
            target_freq: 110.0,
            vel: 1.0,
            accent: false,
            osc1: Osc::new(0x1234_5678 ^ (i as u32 * 7919 + 1)),
            osc2: Osc::new(0x9E37_79B9 ^ (i as u32 * 104_729 + 3)),
            layer_m1_phase: 0.0,
            layer_x_phase: 0.0,
            layers: [IDLE_LAYER; MAX_LAYERS],
            n_layers: 0,
            layers_b: [IDLE_LAYER; MAX_LAYERS],
            n_layers_b: 0,
            dac_phase: 0.0,
            dac_held: 0.0,
            hold: false,
            pan: (i as f32 * 0.618_034).fract() * 2.0 - 1.0,
            time_s: 0.0,
            env: Adsr::default(),
            filter: Ladder::default(),
        }
    }
}

/// Engine-wide options the audio settings page drives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RtOptions {
    pub a4_hz: f32,
    pub max_voices: usize,
    pub headroom_db: f32,
    pub dc_blocker: bool,
    pub soft_clip: bool,
}

impl Default for RtOptions {
    fn default() -> Self {
        Self {
            a4_hz: 440.0,
            max_voices: 8,
            headroom_db: 0.0,
            dc_blocker: true,
            soft_clip: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Sequencer {
    playing: bool,
    step: usize,
    pos: f32,
    just_started: bool,
    gate_closed: bool,
}

/// Polyphonic synthesizer voice engine with an in-engine step sequencer.
pub struct SynthRt {
    sr: f32,
    patch: SynthPatch,
    bank: Option<Arc<SampleBank>>,
    bank_b: Option<Arc<SampleBank>>,
    chorus: Chorus,
    reverb: Reverb,
    reverb_r: Reverb,
    drift: Drift,
    drift_seed: u32,
    voices: [Voice; MAX_VOICES],
    seq: Sequencer,
    counter: u64,
    lfo_phase: f32,
    dc: DcBlocker,
    dc_r: DcBlocker,
    bend_ratio: f32,
    mod_wheel: f32,
    cutoff_s: f32,
    q_s: f32,
    gain_s: f32,
    opts: RtOptions,
    routing: crate::rompler::RackRouting,
    sh_phase: f32,
    sh_seed: u32,
    sh_held: f32,
    sh_slewed: f32,
    delay_l: Vec<f32>,
    delay_r: Vec<f32>,
    delay_pos: usize,
    delay_lp_l: f32,
    delay_lp_r: f32,
    arp: crate::arp::ArpRuntime,
    pitch_table: [f32; 128],
    pitch_tuning: crate::tuning::TuningPreset,
    pitch_a4: f32,
    wt: Arc<crate::wavetable::Wavetable>,
    wt_spec: String,
    clap_host: Option<Box<dyn crate::clap_host::ExternalClapProcessor>>,
    clap_host_cv_param_id: Option<u32>,
    clap_host_dry_l: Vec<f32>,
    clap_host_dry_r: Vec<f32>,
    drums: crate::drums::DrumSynth,
    mseg: crate::nextgen::MsegRuntime,
    filter2: crate::nextgen::Filter2StereoState,
    granular: crate::granular::GranularRuntime,
    spectral: crate::granular::SpectralRuntime,
}

impl std::fmt::Debug for SynthRt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SynthRt")
            .field("sr", &self.sr)
            .field("patch_id", &self.patch.id)
            .field("has_clap_host", &self.clap_host.is_some())
            .finish()
    }
}

impl SynthRt {
    pub fn new(sample_rate: f32, patch: SynthPatch) -> Self {
        let sr = sample_rate.clamp(8000.0, 192_000.0);
        let mut voices = [Voice::new(0); MAX_VOICES];
        for (i, v) in voices.iter_mut().enumerate() {
            *v = Voice::new(i);
        }
        let patch_seed = patch.fx.ambient_seed;
        let pitch_tuning = patch.tuning.clone();
        let pitch_a4 = RtOptions::default().a4_hz;
        let pitch_table = crate::tuning::build_pitch_table(&pitch_tuning, pitch_a4);
        let wt_spec = patch.wavetable.clone();
        let wt = crate::wavetable::Wavetable::load(&wt_spec)
            .or_else(|_| crate::wavetable::Wavetable::load("analog_morph"))
            .unwrap_or_else(|_| {
                Arc::new(crate::wavetable::Wavetable {
                    name: "fallback".into(),
                    frames: vec![[0.0; 256]],
                })
            });
        Self {
            sr,
            cutoff_s: patch.cutoff_hz,
            q_s: patch.resonance_q,
            gain_s: patch.master_gain,
            patch,
            bank: None,
            bank_b: None,
            chorus: Chorus::new(sr),
            reverb: Reverb::new(sr),
            reverb_r: Reverb::with_spread(sr, 23.0),
            drift: Drift::new(patch_seed),
            drift_seed: patch_seed,
            voices,
            seq: Sequencer::default(),
            counter: 0,
            lfo_phase: 0.0,
            dc: DcBlocker::default(),
            dc_r: DcBlocker::default(),
            bend_ratio: 1.0,
            mod_wheel: 0.0,
            opts: RtOptions::default(),
            routing: crate::rompler::RackRouting::default(),
            sh_phase: 0.0,
            sh_seed: 0x5A17_C4FE,
            sh_held: 0.0,
            sh_slewed: 0.0,
            delay_l: vec![0.0; (sr as usize).max(48_000)],
            delay_r: vec![0.0; (sr as usize).max(48_000)],
            delay_pos: 0,
            delay_lp_l: 0.0,
            delay_lp_r: 0.0,
            arp: crate::arp::ArpRuntime::default(),
            pitch_table,
            pitch_tuning,
            pitch_a4,
            wt,
            wt_spec,
            clap_host: None,
            clap_host_cv_param_id: None,
            clap_host_dry_l: vec![0.0; 2048],
            clap_host_dry_r: vec![0.0; 2048],
            drums: crate::drums::DrumSynth::new(),
            mseg: crate::nextgen::MsegRuntime::default(),
            filter2: crate::nextgen::Filter2StereoState::new(),
            granular: crate::granular::GranularRuntime::new(),
            spectral: crate::granular::SpectralRuntime::new(),
        }
    }

    /// Current live MSEG modulation value (`0.0..=1.0`) for animated rotary knob rings.
    pub fn mseg_value(&self) -> f32 {
        self.mseg.last_value
    }

    /// Audition one drum voice immediately (used by UI drum pads & MIDI drum triggers).
    pub fn trigger_drum(&mut self, voice_idx: usize, accent: bool) {
        if let Some(tr) = self.patch.drums.tracks.get(voice_idx) {
            self.drums.trigger(voice_idx, accent, tr.tune, tr.decay);
        } else {
            self.drums.trigger(voice_idx, accent, 0.0, 0.45);
        }
    }

    /// Install or clear an external 3U CLAP Host processor. Returns the previous processor
    /// so the caller can drop it off the audio thread.
    pub fn set_clap_host(
        &mut self,
        host: Option<Box<dyn crate::clap_host::ExternalClapProcessor>>,
    ) -> Option<Box<dyn crate::clap_host::ExternalClapProcessor>> {
        std::mem::replace(&mut self.clap_host, host)
    }

    /// Select which CLAP parameter ID receives CV modulation from the 3U `PARAM CV` jack.
    pub fn set_clap_host_cv_param(&mut self, param_id: Option<u32>) {
        self.clap_host_cv_param_id = param_id;
    }

    /// Forward a parameter value change to the currently hosted CLAP plugin (if any).
    pub fn set_clap_host_param(&mut self, param_id: u32, value: f64) {
        if let Some(host) = self.clap_host.as_mut() {
            host.set_param(param_id, value);
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sr
    }

    pub fn patch(&self) -> &SynthPatch {
        &self.patch
    }

    /// Edit the patch in place. Does not allocate; the engine reads the patch every sample.
    pub fn patch_mut(&mut self) -> &mut SynthPatch {
        &mut self.patch
    }

    /// Swap the patch without disturbing sounding voices. Returns the old patch so the caller
    /// can drop it off the audio thread.
    pub fn replace_patch(&mut self, patch: SynthPatch) -> SynthPatch {
        std::mem::replace(&mut self.patch, patch)
    }

    /// Exchange the patch with `other` in place (no allocation, no drop), for the audio thread.
    pub fn swap_patch(&mut self, other: &mut SynthPatch) {
        std::mem::swap(&mut self.patch, other);
    }

    /// Install (or clear) the sample bank. Sounding voices drop their zone; new notes pick
    /// zones from the new bank. Returns the old bank so the caller can drop it off-thread.
    pub fn set_bank(&mut self, bank: Option<Arc<SampleBank>>) -> Option<Arc<SampleBank>> {
        for v in &mut self.voices {
            v.n_layers = 0;
            v.hold = false;
        }
        std::mem::replace(&mut self.bank, bank)
    }

    /// Install (or clear) the Layer B AI2 Combi sample bank. Returns the old bank for off-thread drop.
    pub fn set_bank_b(&mut self, bank: Option<Arc<SampleBank>>) -> Option<Arc<SampleBank>> {
        for v in &mut self.voices {
            v.n_layers_b = 0;
        }
        std::mem::replace(&mut self.bank_b, bank)
    }

    pub fn has_bank(&self) -> bool {
        self.bank.is_some() || self.bank_b.is_some()
    }

    pub fn set_options(&mut self, opts: RtOptions) {
        self.opts = RtOptions {
            a4_hz: opts.a4_hz.clamp(400.0, 480.0),
            max_voices: opts.max_voices.clamp(1, MAX_VOICES),
            headroom_db: opts.headroom_db.clamp(-24.0, 0.0),
            ..opts
        };
    }

    pub fn options(&self) -> RtOptions {
        self.opts
    }

    pub fn set_routing(&mut self, routing: crate::rompler::RackRouting) {
        self.routing = routing;
    }

    pub fn routing(&self) -> crate::rompler::RackRouting {
        self.routing
    }

    fn hz(&self, note: u8) -> f32 {
        self.pitch_table[(note as usize).min(127)]
    }

    fn sync_pitch_table_if_needed(&mut self) {
        if self.patch.tuning != self.pitch_tuning || (self.opts.a4_hz - self.pitch_a4).abs() > 0.01
        {
            self.pitch_tuning = self.patch.tuning.clone();
            self.pitch_a4 = self.opts.a4_hz;
            self.pitch_table = crate::tuning::build_pitch_table(&self.pitch_tuning, self.pitch_a4);
        }
        if self.patch.wavetable != self.wt_spec {
            self.wt_spec = self.patch.wavetable.clone();
            if let Ok(loaded) = crate::wavetable::Wavetable::load(&self.wt_spec) {
                self.wt = loaded;
            }
        }
    }

    pub fn set_playing(&mut self, playing: bool) {
        if playing && !self.seq.playing {
            self.seq.step = 0;
            self.seq.pos = 0.0;
            self.seq.just_started = true;
            self.seq.gate_closed = false;
        }
        if !playing && self.seq.playing {
            for v in self.voices.iter_mut().filter(|v| v.from_seq) {
                v.gate = false;
                v.env.gate_off();
            }
        }
        self.seq.playing = playing;
    }

    pub fn is_playing(&self) -> bool {
        self.seq.playing
    }

    pub fn current_step(&self) -> usize {
        self.seq.step
    }

    pub fn active_voices(&self) -> usize {
        self.voices.iter().filter(|v| v.active).count()
    }

    pub fn pitch_bend(&mut self, semitones: f32) {
        self.bend_ratio = 2.0_f32.powf(semitones / 12.0);
    }

    pub fn set_mod_wheel(&mut self, value: f32) {
        self.mod_wheel = value.clamp(0.0, 1.0);
    }

    fn allocate(&mut self) -> usize {
        let limit = self.opts.max_voices.clamp(1, MAX_VOICES);
        if let Some(i) = self.voices[..limit].iter().position(|v| !v.active) {
            return i;
        }
        // Steal the oldest releasing voice, otherwise the oldest voice.
        let releasing = self.voices[..limit]
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.gate)
            .min_by_key(|(_, v)| v.age_order)
            .map(|(i, _)| i);
        if let Some(i) = releasing {
            return i;
        }
        self.voices[..limit]
            .iter()
            .enumerate()
            .min_by_key(|(_, v)| v.age_order)
            .map_or(0, |(i, _)| i)
    }

    fn start_voice(&mut self, idx: usize, note: u8, vel: f32, accent: bool, from_seq: bool) {
        self.counter += 1;
        let counter = self.counter;
        let was_active = self.voices[idx].active;
        let f = self.hz(note);
        let v = &mut self.voices[idx];
        v.note = note;
        v.vel = vel.clamp(0.05, 1.0);
        v.accent = accent;
        v.from_seq = from_seq;
        v.age_order = counter;
        v.gate = true;
        v.freq = f;
        v.target_freq = f;
        v.time_s = 0.0;
        v.layer_m1_phase = 0.0;
        v.layer_x_phase = 0.0;
        v.n_layers = 0;
        v.n_layers_b = 0;
        v.dac_phase = 0.0;
        v.dac_held = 0.0;
        v.hold = false;
        if let Some(bank) = self.bank.as_deref() {
            let mut found = [0usize; MAX_LAYERS];
            let n = bank.find_all(note, v.vel, &mut found);
            for &zi in &found[..n] {
                if let Some(z) = bank.zones.get(zi) {
                    let root_hz = midi_hz(z.root_key) * (self.opts.a4_hz / 440.0);
                    v.layers[v.n_layers] = Layer {
                        zone: zi,
                        pos: 0.0,
                        step: z.sample_rate / self.sr * 2.0_f32.powf(z.tune_cents / 1200.0)
                            / root_hz.max(1.0),
                    };
                    v.n_layers += 1;
                    v.hold |= z.one_shot && z.loop_range.is_none();
                }
            }
        }
        if let Some(bank_b) = self.bank_b.as_deref() {
            let oct_shift = i32::from(self.patch.sampler.layer_b_octave.clamp(-2, 2)) * 12;
            let note_b = (i32::from(note) + oct_shift).clamp(0, 127) as u8;
            let detune_ratio = 2.0_f32.powf(
                (self.patch.sampler.layer_b_detune_cents.clamp(-50.0, 50.0)
                    + (oct_shift as f32) * 100.0)
                    / 1200.0,
            );
            let mut found = [0usize; MAX_LAYERS];
            let n = bank_b.find_all(note_b, v.vel, &mut found);
            for &zi in &found[..n] {
                if let Some(z) = bank_b.zones.get(zi) {
                    let root_hz = midi_hz(z.root_key) * (self.opts.a4_hz / 440.0);
                    v.layers_b[v.n_layers_b] = Layer {
                        zone: zi,
                        pos: 0.0,
                        step: z.sample_rate / self.sr
                            * 2.0_f32.powf(z.tune_cents / 1200.0)
                            * detune_ratio
                            / root_hz.max(1.0),
                    };
                    v.n_layers_b += 1;
                }
            }
        }
        if !was_active {
            v.osc1.reset();
            v.osc2.reset();
            v.filter.reset();
            v.env = Adsr::default();
        }
        v.active = true;
        v.env.gate_on();
    }

    fn trigger_voice_note_on(&mut self, note: u8, velocity: f32) {
        let existing = self
            .voices
            .iter()
            .position(|v| v.active && !v.from_seq && v.note == note);
        let idx = existing.unwrap_or_else(|| self.allocate());
        self.start_voice(idx, note, velocity, false, false);
    }

    fn trigger_voice_note_off(&mut self, note: u8) {
        for v in self
            .voices
            .iter_mut()
            .filter(|v| v.active && v.gate && !v.from_seq && v.note == note && !v.hold)
        {
            v.gate = false;
            v.env.gate_off();
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.sync_pitch_table_if_needed();
        if self.patch.arp.mode != crate::arp::ArpMode::Off {
            self.arp.key_down(note, velocity, self.patch.arp.latch);
            return;
        }
        self.trigger_voice_note_on(note, velocity);
    }

    pub fn note_off(&mut self, note: u8) {
        if self.patch.arp.mode != crate::arp::ArpMode::Off {
            self.arp.key_up(note);
            return;
        }
        self.trigger_voice_note_off(note);
    }

    pub fn all_notes_off(&mut self) {
        let _ = self.arp.clear();
        for v in self.voices.iter_mut().filter(|v| v.active) {
            v.hold = false;
            v.gate = false;
            v.env.gate_off();
        }
    }

    fn seq_voice(&self) -> Option<usize> {
        self.voices
            .iter()
            .position(|v| v.active && v.from_seq && v.gate)
    }

    fn trigger_step(&mut self) {
        // Trigger the synchronized 8-voice TR-808/909 drum machine on every sequencer step.
        self.drums.trigger_step(&self.patch.drums, self.seq.step);
        let len = self.patch.steps.len();
        if len == 0 {
            return;
        }
        let idx = self.seq.step % len;
        let step = self.patch.steps[idx].clone();
        self.seq.gate_closed = false;
        if !step.active {
            if let Some(i) = self.seq_voice() {
                self.voices[i].gate = false;
                self.voices[i].env.gate_off();
            }
            self.seq.gate_closed = true;
            return;
        }
        let note = step.midi_note.clamp(0, 127);
        if step.slide {
            if let Some(i) = self.seq_voice() {
                // Legato glide: keep the envelope running, move the pitch.
                let target = self.hz(note);
                let v = &mut self.voices[i];
                v.target_freq = target;
                v.note = note;
                v.accent = step.accent;
                return;
            }
        }
        let vel = step.velocity.clamp(0.05, 1.0);
        let i = match self.seq_voice() {
            Some(i) => i,
            None => self.allocate(),
        };
        // A new non-slide step always retriggers.
        self.start_voice(i, note, vel, step.accent, true);
    }

    fn maybe_close_gate(&mut self, step_len: f32) {
        let len = self.patch.steps.len();
        if len == 0 || self.seq.gate_closed {
            return;
        }
        let cur = self.patch.steps[self.seq.step % len].clone();
        let gate_len = step_len * cur.gate_pct.clamp(0.1, 1.0);
        if self.seq.pos < gate_len {
            return;
        }
        let next = &self.patch.steps[(self.seq.step + 1) % len];
        if next.active && next.slide {
            // Slide into the next note keeps the gate open through the step boundary.
            return;
        }
        self.seq.gate_closed = true;
        if let Some(i) = self.seq_voice() {
            self.voices[i].gate = false;
            self.voices[i].env.gate_off();
        }
    }

    /// Render `out.len()` mono samples. Allocation-free.
    /// Render a mono mix (left and right averaged). Does not allocate.
    pub fn process(&mut self, out: &mut [f32]) {
        let mut l = [0.0_f32; 256];
        let mut r = [0.0_f32; 256];
        for chunk in out.chunks_mut(256) {
            let n = chunk.len();
            self.process_stereo(&mut l[..n], &mut r[..n]);
            for ((o, a), b) in chunk.iter_mut().zip(&l[..n]).zip(&r[..n]) {
                *o = 0.5 * (a + b);
            }
        }
    }

    /// Render `left.len().min(right.len())` stereo frames. This is the one synthesis path.
    /// Does not allocate.
    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        self.process_stereo_with_ext_in(left, right, &[], &[], 0.0);
    }

    /// Render stereo into `left` and `right`, optionally mixing an external stereo audio input
    /// (`ext_l`, `ext_r`, e.g. DJCraft's Auxiliary FX Send) directly into SynthCraft's 3U
    /// Modular FX chain (Wavefolder, Chorus, Stereo Ping-Pong Delay, Reverb, and 3U CLAP Host).
    pub fn process_stereo_with_ext_in(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        ext_l: &[f32],
        ext_r: &[f32],
        ext_gain: f32,
    ) {
        self.sync_pitch_table_if_needed();
        let sr = self.sr;
        let glide_coef = 1.0 - (-1.0 / (GLIDE_MS * 0.001 * sr)).exp();
        let smooth = 1.0 - (-1.0 / (0.008 * sr)).exp();
        let step_len = 60.0 / self.patch.bpm.clamp(40.0, 240.0) * sr * 0.25;
        let headroom = 10.0_f32.powf(self.opts.headroom_db / 20.0);
        let fx = self.patch.fx;
        self.reverb.set(fx.reverb_decay_s, fx.reverb_damping);
        self.reverb_r.set(fx.reverb_decay_s, fx.reverb_damping);
        if fx.ambient_seed != self.drift_seed {
            self.drift = Drift::new(fx.ambient_seed);
            self.drift_seed = fx.ambient_seed;
        }
        let spread = fx.stereo_spread.clamp(0.0, 1.0);
        let ambient = fx.ambient_depth.clamp(0.0, 1.0);

        for (sample_idx, (left_out, right_out)) in left.iter_mut().zip(right.iter_mut()).enumerate()
        {
            match self.arp.tick(self.patch.arp, self.patch.bpm, sr) {
                crate::arp::ArpAction::NoteOn(n, vel) => self.trigger_voice_note_on(n, vel),
                crate::arp::ArpAction::NoteOff(n) => self.trigger_voice_note_off(n),
                crate::arp::ArpAction::None => {}
            }
            if self.seq.playing {
                if self.seq.just_started {
                    self.seq.just_started = false;
                    self.trigger_step();
                }
                self.maybe_close_gate(step_len);
            }

            self.cutoff_s += (self.patch.cutoff_hz - self.cutoff_s) * smooth;
            self.q_s += (self.patch.resonance_q - self.q_s) * smooth;
            self.gain_s += (self.patch.master_gain - self.gain_s) * smooth;

            let lfo = (TAU * self.lfo_phase).sin()
                * (self.patch.lfo_depth + self.mod_wheel * (1.0 - self.patch.lfo_depth))
                    .clamp(0.0, 1.0);
            self.lfo_phase =
                (self.lfo_phase + self.patch.lfo_rate_hz.clamp(0.05, 40.0) / sr).rem_euclid(1.0);

            // Sample & Hold + Slew Limiter (driven by S&H + SLEW 3U module)
            let r = self.routing;
            self.sh_phase += r.sh_rate_hz.clamp(0.2, 30.0) / sr;
            if self.sh_phase >= 1.0 {
                self.sh_phase -= 1.0;
                let mut x = self.sh_seed.max(1);
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                self.sh_seed = x;
                self.sh_held = ((x & 0xFFFF) as f32 / 32768.0) - 1.0;
            }
            let slew_coef = if r.sh_slew <= 0.01 {
                1.0
            } else {
                1.0 - (-1.0 / (r.sh_slew * 0.25 * sr)).exp()
            };
            self.sh_slewed += (self.sh_held - self.sh_slewed) * slew_coef;
            let sh_cv = self.sh_slewed;

            let p = &self.patch;
            let detune_ratio = 2.0_f32.powf(p.vco2_detune_cents / 1200.0);
            let mix = p.vco_mix.clamp(0.0, 1.0);
            let drive_in = 1.0 + p.drive_saturation.clamp(0.0, 3.0) * 0.8;
            let drive_out = 1.0 + p.drive_saturation.clamp(0.0, 3.0) * 1.2;
            let sampler_mix = if self.bank.is_some() || self.bank_b.is_some() {
                p.sampler.mix.clamp(0.0, 1.0)
            } else {
                0.0
            };
            let (d_cut, d_morph, d_verb) = if ambient > 0.0 {
                let d = self.drift.tick(fx.ambient_rate_hz, sr);
                (d[0] * ambient, d[1] * ambient, d[2] * ambient)
            } else {
                (0.0, 0.0, 0.0)
            };
            let morph = (p.extreme_morph + d_morph * 0.35).clamp(0.0, 1.0);
            let mut sum_l = 0.0_f32;
            let mut sum_r = 0.0_f32;

            for v in self.voices.iter_mut().filter(|v| v.active) {
                v.freq += (v.target_freq - v.freq) * glide_coef;
                let f = v.freq * self.bend_ratio;
                let env = v
                    .env
                    .tick(p.attack_ms, p.decay_ms, p.sustain_level, p.release_ms, sr);
                if v.env.is_idle() {
                    v.active = false;
                    continue;
                }
                let acc = if v.accent { 1.35 } else { 1.0 };
                let mo = crate::mod_matrix::evaluate_mod_matrix(
                    &p.mod_matrix,
                    crate::mod_matrix::ModInputs {
                        lfo,
                        env,
                        vel: v.vel,
                        mod_wheel: self.mod_wheel,
                        sh: sh_cv,
                        drift: d_cut,
                    },
                );
                let voice_detune = if mo.detune_cents.abs() > 1e-4 {
                    2.0_f32.powf((p.vco2_detune_cents + mo.detune_cents) / 1200.0)
                } else {
                    detune_ratio
                };
                let pw = (p.pulse_width + mo.pulse_width).clamp(0.05, 0.95);
                let r = self.routing;
                let wt_morph = (morph + mo.morph).clamp(0.0, 1.0);
                let s2_raw = v.osc2.tick(p.vco2_wave, f * voice_detune, sr, pw, 0.0);
                let s2 = if p.vco2_wave == crate::model::WaveformKind::Wavetable {
                    self.wt.sample(v.osc2.phase(), wt_morph)
                } else {
                    s2_raw
                };
                let fm = s2 * (p.fm_index + mo.fm_index).clamp(0.0, 4.0) * 0.08 * r.vco2_to_fm
                    + lfo * r.lfo_to_fm * 0.15
                    + sh_cv * r.sh_to_fm * 0.12;
                let s1_raw = v.osc1.tick(p.vco1_wave, f, sr, pw, fm);
                let s1 = if p.vco1_wave == crate::model::WaveformKind::Wavetable {
                    self.wt
                        .sample((v.osc1.phase() + fm).rem_euclid(1.0), wt_morph)
                } else {
                    s1_raw
                };
                let mut raw = s1 * (1.0 - mix) * r.vco1_to_vcf + s2 * (mix + r.vco2_to_vcf * 0.5);
                let fold = (r.fold_amount + mo.wavefold).clamp(0.0, 1.0);
                if fold > 0.01 {
                    let gain = 1.0 + fold * 4.5;
                    let folded = (raw * gain + r.fold_symmetry).sin();
                    raw = raw * (1.0 - fold) + folded * fold;
                }

                let dphase = f64::from(f / sr);
                if p.extreme_dsp != crate::rompler::ExtremeDspMode::Off {
                    v.layer_x_phase += dphase;
                    let x = p.extreme_dsp.sample_dsp(
                        v.layer_x_phase as f32,
                        v.time_s,
                        (morph + mo.morph).clamp(0.0, 1.0),
                    );
                    raw = raw * 0.45 + x * 0.55;
                }
                if let Some(m1) = p.m1_program {
                    v.layer_m1_phase += dphase;
                    let pcm = m1.sample_pcm(v.layer_m1_phase as f32, v.time_s);
                    let m = (p.m1_layer_mix * r.pcm_to_vcf).clamp(0.0, 1.0);
                    raw = raw * (1.0 - m) + pcm * m;
                }
                let mut layer_pan = 0.0_f32;
                if sampler_mix > 0.0 || v.hold {
                    let mut smp_a = 0.0_f32;
                    let mut smp_b = 0.0_f32;
                    let mut alive_a = 0_u32;
                    let mut alive_b = 0_u32;
                    let mut pan_acc = 0.0_f32;
                    if let Some(bank) = self.bank.as_deref() {
                        for l in &mut v.layers[..v.n_layers] {
                            let Some(z) = bank.zones.get(l.zone) else {
                                continue;
                            };
                            smp_a += z.read(l.pos) * z.gain;
                            pan_acc += z.pan;
                            alive_a += 1;
                            l.pos += f64::from(f * l.step);
                            match z.loop_range {
                                Some((ls, le)) if l.pos >= le as f64 && le > ls => {
                                    l.pos -= (le - ls) as f64;
                                }
                                None if l.pos >= z.len() as f64 => l.zone = NO_ZONE,
                                _ => {}
                            }
                        }
                    }
                    if let Some(bank_b) = self.bank_b.as_deref() {
                        for l in &mut v.layers_b[..v.n_layers_b] {
                            let Some(z) = bank_b.zones.get(l.zone) else {
                                continue;
                            };
                            smp_b += z.read(l.pos) * z.gain;
                            pan_acc += z.pan;
                            alive_b += 1;
                            l.pos += f64::from(f * l.step);
                            match z.loop_range {
                                Some((ls, le)) if l.pos >= le as f64 && le > ls => {
                                    l.pos -= (le - ls) as f64;
                                }
                                None if l.pos >= z.len() as f64 => l.zone = NO_ZONE,
                                _ => {}
                            }
                        }
                    }
                    let alive = alive_a + alive_b;
                    if alive > 0 {
                        layer_pan = pan_acc / alive as f32;
                    }
                    if v.hold && alive == 0 {
                        v.hold = false;
                        v.gate = false;
                        v.env.gate_off();
                    }
                    let b_bal = p.sampler.layer_b_mix.clamp(0.0, 1.0);
                    let mut smp = match (self.bank.is_some(), self.bank_b.is_some()) {
                        (true, true) => smp_a * (1.0 - b_bal) + smp_b * b_bal,
                        (true, false) => smp_a,
                        (false, true) => smp_b,
                        (false, false) => 0.0,
                    };
                    if p.sampler.vintage_dac {
                        // Authentic 1988 32 kHz sample-and-hold + 12-bit DAC quantization
                        v.dac_phase += 32_000.0 / sr;
                        if v.dac_phase >= 1.0 {
                            v.dac_phase -= v.dac_phase.floor();
                            v.dac_held = (smp.clamp(-1.0, 1.0) * 2048.0).round() / 2048.0;
                        }
                        smp = v.dac_held;
                    }
                    raw = raw * (1.0 - sampler_mix) + smp * sampler_mix;
                }
                v.time_s += 1.0 / sr;

                let octaves = env * p.env_mod_depth.clamp(0.0, 1.0) * 3.0 * acc
                    + env * r.adsr_to_cutoff * 2.5
                    + lfo * 1.2 * r.lfo_to_cutoff
                    + sh_cv * 1.6 * r.sh_to_cutoff
                    + d_cut * 1.5
                    + mo.cutoff_oct;
                let cutoff = self.cutoff_s * 2.0_f32.powf(octaves);
                let q = (self.q_s + mo.resonance_q).clamp(0.5, 12.0);
                let filtered = v
                    .filter
                    .process(raw * drive_in, cutoff, sr, q, p.filter_mode);
                let vca_in = filtered * r.vcf_to_vca + raw * r.direct_to_vca;
                let x = (vca_in * drive_out).tanh() * env * r.adsr_to_vca * v.vel * acc;
                let pan = (v.pan * spread + layer_pan + mo.pan).clamp(-1.0, 1.0);
                if pan == 0.0 {
                    sum_l += x;
                    sum_r += x;
                } else {
                    let ang = (pan + 1.0) * FRAC_PI_4;
                    sum_l += x * ang.cos() * SQRT_2;
                    sum_r += x * ang.sin() * SQRT_2;
                }
            }

            let g = self.gain_s.clamp(0.0, 1.0) * headroom;
            let mut out_l = sum_l * g;
            let mut out_r = sum_r * g;
            if ext_gain > 0.001 && sample_idx < ext_l.len() && sample_idx < ext_r.len() {
                let mut el = ext_l[sample_idx] * ext_gain;
                let mut er = ext_r[sample_idx] * ext_gain;
                if r.fold_amount > 0.01 {
                    let fg = 1.0 + r.fold_amount * 4.5;
                    el = (el * fg + r.fold_symmetry).sin();
                    er = (er * fg + r.fold_symmetry).sin();
                }
                out_l += el;
                out_r += er;
            }
            if fx.chorus_mix > 0.001 {
                (out_l, out_r) =
                    self.chorus
                        .process_stereo(out_l, out_r, fx.chorus_mix.clamp(0.0, 1.0));
            }
            if r.delay_mix > 0.001 && !self.delay_l.is_empty() {
                let cap = self.delay_l.len();
                let d_samps = ((r.delay_time_s * sr) as usize).clamp(64, cap - 1);
                let r_samps = ((d_samps as f32 * 0.75) as usize).clamp(64, cap - 1);
                let idx_l = (self.delay_pos + cap - d_samps) % cap;
                let idx_r = (self.delay_pos + cap - r_samps) % cap;
                let tap_l = self.delay_l[idx_l];
                let tap_r = self.delay_r[idx_r];
                self.delay_lp_l += (tap_l - self.delay_lp_l) * 0.35;
                self.delay_lp_r += (tap_r - self.delay_lp_r) * 0.35;
                let fb = r.delay_feedback.clamp(0.0, 0.88);
                // Ping-pong cross-feedback
                self.delay_l[self.delay_pos] = (out_l + self.delay_lp_r * fb).tanh();
                self.delay_r[self.delay_pos] = (out_r + self.delay_lp_l * fb).tanh();
                self.delay_pos = (self.delay_pos + 1) % cap;
                let dm = r.delay_mix.clamp(0.0, 1.0);
                out_l = out_l * (1.0 - dm * 0.4) + tap_l * dm;
                out_r = out_r * (1.0 - dm * 0.4) + tap_r * dm;
            }
            if fx.reverb_mix > 0.001 {
                let m = (fx.reverb_mix + d_verb * 0.25).clamp(0.0, 1.0);
                out_l = out_l * (1.0 - 0.4 * m) + self.reverb.process(out_l) * m;
                out_r = out_r * (1.0 - 0.4 * m) + self.reverb_r.process(out_r) * m;
            }
            let mseg_val = self.mseg.tick(&self.patch.mseg, self.patch.bpm, sr);
            let active_freq = self
                .voices
                .iter()
                .find(|v| v.active)
                .map(|v| v.freq)
                .unwrap_or(110.0);
            let (spec_l, spec_r) = self
                .spectral
                .tick_stereo(active_freq, &self.patch.spectral, sr);
            out_l += spec_l;
            out_r += spec_r;

            let f2_mod = mseg_val * self.patch.mseg.filter2_depth;
            if self.patch.filter2.routing == crate::nextgen::Filter2Routing::Parallel {
                let (fl, fr) =
                    self.filter2
                        .process_stereo(out_l, out_r, &self.patch.filter2, f2_mod, sr);
                out_l = (out_l + fl) * 0.5;
                out_r = (out_r + fr) * 0.5;
            } else {
                (out_l, out_r) =
                    self.filter2
                        .process_stereo(out_l, out_r, &self.patch.filter2, f2_mod, sr);
            }

            (out_l, out_r) = self
                .granular
                .tick_stereo(out_l, out_r, &self.patch.granular, sr);

            let (drum_l, drum_r) = self.drums.tick_stereo(&self.patch.drums, sr);
            out_l += drum_l;
            out_r += drum_r;

            if self.opts.dc_blocker {
                out_l = self.dc.process(out_l);
                out_r = self.dc_r.process(out_r);
            }
            if self.opts.soft_clip {
                *left_out = (out_l * 0.9).tanh();
                *right_out = (out_r * 0.9).tanh();
            } else {
                *left_out = out_l.clamp(-1.0, 1.0);
                *right_out = out_r.clamp(-1.0, 1.0);
            }

            if self.seq.playing {
                self.seq.pos += 1.0;
                // Apply classic TR/MPC swing to even/odd 16th-note pairs when drum swing > 0.50
                let swing = self.patch.drums.swing.clamp(0.50, 0.75);
                let cur_step_len = if self.seq.step.is_multiple_of(2) {
                    step_len * (swing * 2.0)
                } else {
                    step_len * ((1.0 - swing) * 2.0)
                };
                if self.seq.pos >= cur_step_len {
                    self.seq.pos -= cur_step_len;
                    let synth_len = self.patch.steps.len().max(1);
                    let drum_len = if self.patch.drums.enabled {
                        self.patch
                            .drums
                            .length
                            .clamp(1, crate::drums::DRUM_MAX_STEPS)
                    } else {
                        synth_len
                    };
                    let cycle_len = synth_len.max(drum_len);
                    self.seq.step = (self.seq.step + 1) % cycle_len;
                    self.trigger_step();
                }
            }
        }

        // 3U CLAP Host external plugin insert (runs block-aligned with CV-to-param modulation).
        let mix = self.routing.clap_host_mix.clamp(0.0, 1.0);
        if mix > 0.001 && !left.is_empty() {
            let cv_lfo = (TAU * self.lfo_phase).sin() * self.routing.clap_host_cv_lfo;
            let cv_sh = self.sh_slewed * self.routing.clap_host_cv_sh;
            let cv_amount = f64::from((cv_lfo + cv_sh) * self.routing.clap_host_cv_depth);
            let cv_mod = self.clap_host_cv_param_id.map(|id| (id, cv_amount));

            if let Some(host) = self.clap_host.as_mut() {
                let n = left.len().min(right.len());
                if mix < 0.999 && n <= self.clap_host_dry_l.len() {
                    self.clap_host_dry_l[..n].copy_from_slice(&left[..n]);
                    self.clap_host_dry_r[..n].copy_from_slice(&right[..n]);
                    host.process_block(&mut left[..n], &mut right[..n], cv_mod);
                    let dry = 1.0 - mix;
                    for i in 0..n {
                        left[i] = (self.clap_host_dry_l[i] * dry + left[i] * mix).clamp(-1.0, 1.0);
                        right[i] =
                            (self.clap_host_dry_r[i] * dry + right[i] * mix).clamp(-1.0, 1.0);
                    }
                } else {
                    host.process_block(&mut left[..n], &mut right[..n], cv_mod);
                }
            }
        }
    }
}
