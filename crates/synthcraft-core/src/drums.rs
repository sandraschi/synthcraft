//! Full 8-Voice Pure-Rust TR-808 / TR-909 Analog & FM Drum Machine (`#![forbid(unsafe_code)]`).
//!
//! Voices:
//! 0. `BD` -- Kick Drum (pitch-swept sine/triangle + transient click + sub punch)
//! 1. `SD` -- Snare Drum (dual resonant body oscillators + bandpass snare-wire noise)
//! 2. `CP` -- Hand Clap (4 staggered micro-delayed bandpass bursts + diffused tail)
//! 3. `CH` -- Closed Hi-Hat (6-oscillator 808 metallic square cluster + HPF; chokes OH)
//! 4. `OH` -- Open Hi-Hat (6-oscillator 808 metallic cluster, long sizzle; choked by CH)
//! 5. `LT` -- Low Tom / Conga (pitch-swept warm sine shell)
//! 6. `CB` -- Rimshot / 808 Cowbell (540 Hz + 800 Hz square-wave bandpass metallic hit)
//! 7. `CY` -- Crash / Ride Cymbal (inharmonic metallic noise shimmer + long decay)
//!
//! Supports up to 64 steps (4 x 16-step bars: A, B, C, D) with per-step Off/Hit/Accent,
//! per-voice Level/Tune/Decay/Pan/Mute/Solo, Swing (50%..75%), Euclidean rhythm fills,
//! and 6 built-in club/classic kit presets.

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

pub const DRUM_VOICE_COUNT: usize = 8;
pub const DRUM_MAX_STEPS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumVoiceKind {
    Kick = 0,
    Snare = 1,
    Clap = 2,
    ClosedHat = 3,
    OpenHat = 4,
    LowTom = 5,
    Cowbell = 6,
    Cymbal = 7,
}

impl DrumVoiceKind {
    pub const ALL: [Self; DRUM_VOICE_COUNT] = [
        Self::Kick,
        Self::Snare,
        Self::Clap,
        Self::ClosedHat,
        Self::OpenHat,
        Self::LowTom,
        Self::Cowbell,
        Self::Cymbal,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::Kick => "BD",
            Self::Snare => "SD",
            Self::Clap => "CP",
            Self::ClosedHat => "CH",
            Self::OpenHat => "OH",
            Self::LowTom => "LT",
            Self::Cowbell => "CB",
            Self::Cymbal => "CY",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Kick => "Kick (808/909)",
            Self::Snare => "Snare",
            Self::Clap => "Hand Clap",
            Self::ClosedHat => "Closed Hat",
            Self::OpenHat => "Open Hat",
            Self::LowTom => "Low Tom",
            Self::Cowbell => "Cowbell / Rim",
            Self::Cymbal => "Crash / Ride",
        }
    }

    pub fn default_pan(self) -> f32 {
        match self {
            Self::Kick => 0.0,
            Self::Snare => 0.05,
            Self::Clap => -0.15,
            Self::ClosedHat => 0.28,
            Self::OpenHat => 0.34,
            Self::LowTom => -0.30,
            Self::Cowbell => -0.22,
            Self::Cymbal => 0.40,
        }
    }

    pub fn default_decay(self) -> f32 {
        match self {
            Self::Kick => 0.55,
            Self::Snare => 0.40,
            Self::Clap => 0.45,
            Self::ClosedHat => 0.16,
            Self::OpenHat => 0.62,
            Self::LowTom => 0.48,
            Self::Cowbell => 0.30,
            Self::Cymbal => 0.78,
        }
    }
}

/// One lane of the 8-voice drum machine (up to 64 steps).
/// Step values: `0` = Off, `1` = Normal Hit, `2` = Accent Hit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DrumTrack {
    pub level: f32,
    pub tune: f32,  // -12.0 .. +12.0 semitones
    pub decay: f32, // 0.05 .. 1.0
    pub pan: f32,   // -1.0 .. +1.0
    pub mute: bool,
    pub solo: bool,
    pub steps: Vec<u8>,
}

impl Default for DrumTrack {
    fn default() -> Self {
        Self {
            level: 0.85,
            tune: 0.0,
            decay: 0.45,
            pan: 0.0,
            mute: false,
            solo: false,
            steps: vec![0; DRUM_MAX_STEPS],
        }
    }
}

impl DrumTrack {
    pub fn for_voice(kind: DrumVoiceKind) -> Self {
        Self {
            level: match kind {
                DrumVoiceKind::Kick => 0.95,
                DrumVoiceKind::Snare | DrumVoiceKind::Clap => 0.82,
                DrumVoiceKind::ClosedHat | DrumVoiceKind::OpenHat => 0.62,
                DrumVoiceKind::Cymbal => 0.55,
                _ => 0.72,
            },
            tune: 0.0,
            decay: kind.default_decay(),
            pan: kind.default_pan(),
            mute: false,
            solo: false,
            steps: vec![0; DRUM_MAX_STEPS],
        }
    }

    pub fn ensure_len(&mut self) {
        if self.steps.len() < DRUM_MAX_STEPS {
            self.steps.resize(DRUM_MAX_STEPS, 0);
        }
    }

    pub fn step_state(&self, idx: usize) -> u8 {
        self.steps.get(idx % DRUM_MAX_STEPS).copied().unwrap_or(0)
    }

    pub fn set_step(&mut self, idx: usize, val: u8) {
        self.ensure_len();
        if idx < self.steps.len() {
            self.steps[idx] = val.min(2);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DrumMachinePatch {
    pub enabled: bool,
    pub length: usize, // 16, 32, 48, or 64
    pub swing: f32,    // 0.50 (straight) .. 0.75 (heavy MPC swing)
    pub master_level: f32,
    pub kit_name: String,
    pub tracks: Vec<DrumTrack>,
}

impl Default for DrumMachinePatch {
    fn default() -> Self {
        let mut p = Self::preset("tr909_berlin_techno");
        p.enabled = false;
        p
    }
}

impl DrumMachinePatch {
    pub fn preset_ids() -> &'static [(&'static str, &'static str)] {
        &[
            ("tr909_berlin_techno", "TR-909 Berlin Techno (16-Step)"),
            ("tr808_electro", "TR-808 Electro Funk (32-Step)"),
            ("house_1990_classic", "1990 House Classic (32-Step)"),
            ("dnb_amen_174", "Drum & Bass Two-Step (32-Step)"),
            ("acid_303_groove", "Acid 303 Warehouse (32-Step)"),
            ("polyrhythm_64", "64-Step 4-Bar Club Progression"),
        ]
    }

    pub fn ensure_valid(&mut self) {
        self.length = match self.length {
            1..=16 => 16,
            17..=32 => 32,
            33..=48 => 48,
            _ => 64,
        };
        while self.tracks.len() < DRUM_VOICE_COUNT {
            let kind = DrumVoiceKind::ALL[self.tracks.len()];
            self.tracks.push(DrumTrack::for_voice(kind));
        }
        self.tracks.truncate(DRUM_VOICE_COUNT);
        for t in &mut self.tracks {
            t.ensure_len();
        }
    }

    /// Fill a single track's active length with a Euclidean pulse pattern.
    pub fn fill_euclidean(&mut self, voice_idx: usize, pulses: usize) {
        self.ensure_valid();
        let len = self.length.clamp(1, DRUM_MAX_STEPS);
        if let Some(tr) = self.tracks.get_mut(voice_idx) {
            for (s, step) in tr.steps.iter_mut().enumerate().take(len) {
                let hit = if pulses == 0 {
                    false
                } else {
                    ((s * pulses) % len) < pulses
                };
                *step = if hit {
                    if s.is_multiple_of(4) {
                        2
                    } else {
                        1
                    }
                } else {
                    0
                };
            }
        }
    }

    pub fn preset(id: &str) -> Self {
        let mut tracks: Vec<DrumTrack> = DrumVoiceKind::ALL
            .iter()
            .copied()
            .map(DrumTrack::for_voice)
            .collect();

        let set_bar = |tr: &mut DrumTrack, bar: usize, pat: &str| {
            let base = bar * 16;
            for (i, ch) in pat.chars().take(16).enumerate() {
                tr.steps[base + i] = match ch {
                    'X' => 2,
                    'x' => 1,
                    _ => 0,
                };
            }
        };

        let (kit_name, length, swing) = match id {
            "tr808_electro" => {
                for bar in 0..2 {
                    set_bar(&mut tracks[0], bar, "X..x..x...X.....");
                    set_bar(&mut tracks[1], bar, "....X.......X...");
                    set_bar(&mut tracks[2], bar, "....x.......X..x");
                    set_bar(&mut tracks[3], bar, "x.x.x.x.x.x.x.x.");
                    set_bar(&mut tracks[4], bar, "..x.......x.....");
                    set_bar(&mut tracks[6], bar, "x..x..x...x.x...");
                }
                set_bar(&mut tracks[5], 1, "............x.xX");
                ("TR-808 Electro Funk".to_string(), 32, 0.56)
            }
            "house_1990_classic" => {
                for bar in 0..2 {
                    set_bar(&mut tracks[0], bar, "X...X...X...X...");
                    set_bar(&mut tracks[2], bar, "....X.......X...");
                    set_bar(&mut tracks[3], bar, "x.x.x.x.x.x.x.x.");
                    set_bar(&mut tracks[4], bar, "..X...X...X...X.");
                }
                set_bar(&mut tracks[1], 1, "....x.......x.x.");
                set_bar(&mut tracks[7], 0, "X...............");
                ("1990 House Classic".to_string(), 32, 0.54)
            }
            "dnb_amen_174" => {
                for bar in 0..2 {
                    set_bar(&mut tracks[0], bar, "X.........X.....");
                    set_bar(&mut tracks[1], bar, "....X..x.x..X...");
                    set_bar(&mut tracks[3], bar, "x.x.x.x.x.x.x.x.");
                    set_bar(&mut tracks[4], bar, "......x.......x.");
                }
                set_bar(&mut tracks[0], 1, "X.......x.X.....");
                set_bar(&mut tracks[7], 0, "X...............");
                ("Drum & Bass Two-Step".to_string(), 32, 0.50)
            }
            "acid_303_groove" => {
                for bar in 0..2 {
                    set_bar(&mut tracks[0], bar, "X...X...X...X...");
                    set_bar(&mut tracks[1], bar, "....X.......X...");
                    set_bar(&mut tracks[2], bar, "....x.......x..x");
                    set_bar(&mut tracks[3], bar, "x.xx.xxx.xxx.xxx");
                    set_bar(&mut tracks[4], bar, "..x.......x.....");
                    set_bar(&mut tracks[6], bar, "..x..x....x.....");
                }
                set_bar(&mut tracks[5], 1, "..........x.xx.X");
                ("Acid 303 Warehouse".to_string(), 32, 0.53)
            }
            "polyrhythm_64" => {
                for bar in 0..4 {
                    set_bar(&mut tracks[0], bar, "X...X...X...X...");
                    set_bar(&mut tracks[3], bar, "x.xxx.xxx.xxx.xx");
                    set_bar(&mut tracks[4], bar, "..x...x...x...x.");
                    if bar >= 1 {
                        set_bar(&mut tracks[2], bar, "....X.......X...");
                    }
                    if bar >= 2 {
                        set_bar(&mut tracks[1], bar, "....x.......x..x");
                        set_bar(&mut tracks[6], bar, "x..x..x..x..x...");
                    }
                }
                set_bar(&mut tracks[5], 3, "........x.x.xxxX");
                set_bar(&mut tracks[7], 0, "X...............");
                set_bar(&mut tracks[7], 2, "X...............");
                ("64-Step 4-Bar Club Progression".to_string(), 64, 0.52)
            }
            _ => {
                // Default: TR-909 Berlin Techno (16-step, mirrored to bar 2 for easy expansion)
                for bar in 0..2 {
                    set_bar(&mut tracks[0], bar, "X...X...X...X...");
                    set_bar(&mut tracks[1], bar, "....X.......X...");
                    set_bar(&mut tracks[2], bar, "....x.......X...");
                    set_bar(&mut tracks[3], bar, "x.x.x.x.x.x.x.x.");
                    set_bar(&mut tracks[4], bar, "..X...X...X...X.");
                    set_bar(&mut tracks[5], bar, "..........x..x..");
                }
                set_bar(&mut tracks[7], 0, "X...............");
                ("TR-909 Berlin Techno".to_string(), 16, 0.52)
            }
        };

        Self {
            enabled: true,
            length,
            swing,
            master_level: 0.88,
            kit_name,
            tracks,
        }
    }
}

// ---------------------------------------------------------------------------
// Allocation-free real-time 8-voice analog/FM percussion synthesizer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct DrumVoiceState {
    active: bool,
    age_s: f32,
    phase: f32,
    phase2: f32,
    vel: f32,
    tune_ratio: f32,
    decay_param: f32,
    hp_prev_in: f32,
    hp_prev_out: f32,
    bp_lp: f32,
    bp_hp: f32,
}

impl Default for DrumVoiceState {
    fn default() -> Self {
        Self {
            active: false,
            age_s: 0.0,
            phase: 0.0,
            phase2: 0.0,
            vel: 1.0,
            tune_ratio: 1.0,
            decay_param: 0.5,
            hp_prev_in: 0.0,
            hp_prev_out: 0.0,
            bp_lp: 0.0,
            bp_hp: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrumSynth {
    voices: [DrumVoiceState; DRUM_VOICE_COUNT],
    rng: u32,
}

impl Default for DrumSynth {
    fn default() -> Self {
        Self::new()
    }
}

impl DrumSynth {
    pub fn new() -> Self {
        Self {
            voices: std::array::from_fn(|_| DrumVoiceState::default()),
            rng: 0x8080_9091,
        }
    }

    #[inline]
    fn next_noise(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        ((x as f32) / (u32::MAX as f32)) * 2.0 - 1.0
    }

    pub fn trigger(&mut self, voice_idx: usize, accent: bool, tune_semitones: f32, decay: f32) {
        if voice_idx >= DRUM_VOICE_COUNT {
            return;
        }
        // Classic TR-808/909 Hi-Hat Choke: Closed Hat silences Open Hat immediately.
        if voice_idx == DrumVoiceKind::ClosedHat as usize {
            self.voices[DrumVoiceKind::OpenHat as usize].active = false;
        }
        let v = &mut self.voices[voice_idx];
        v.active = true;
        v.age_s = 0.0;
        v.phase = 0.0;
        v.phase2 = 0.0;
        v.vel = if accent { 1.25 } else { 0.85 };
        v.tune_ratio = 2.0_f32.powf(tune_semitones.clamp(-12.0, 12.0) / 12.0);
        v.decay_param = decay.clamp(0.05, 1.0);
    }

    pub fn trigger_step(&mut self, patch: &DrumMachinePatch, step: usize) {
        if !patch.enabled || patch.tracks.is_empty() {
            return;
        }
        let any_solo = patch.tracks.iter().any(|t| t.solo);
        let len = patch.length.clamp(1, DRUM_MAX_STEPS);
        let idx = step % len;
        for (vi, tr) in patch.tracks.iter().enumerate().take(DRUM_VOICE_COUNT) {
            if tr.mute || (any_solo && !tr.solo) {
                continue;
            }
            let st = tr.step_state(idx);
            if st > 0 {
                self.trigger(vi, st >= 2, tr.tune, tr.decay);
            }
        }
    }

    /// Render one stereo frame of all 8 active percussion voices. Allocation-free.
    #[inline]
    pub fn tick_stereo(&mut self, patch: &DrumMachinePatch, sr: f32) -> (f32, f32) {
        let mut sum_l = 0.0_f32;
        let mut sum_r = 0.0_f32;
        let dt = 1.0 / sr.max(8000.0);
        let master = patch.master_level.clamp(0.0, 1.5);

        for vi in 0..DRUM_VOICE_COUNT {
            if !self.voices[vi].active {
                continue;
            }
            let noise = self.next_noise();
            let v = &mut self.voices[vi];
            let t = v.age_s;
            let tr_ratio = v.tune_ratio;
            let dec = v.decay_param;

            let sample = match vi {
                // 0: Kick (808/909 pitch-swept sine + transient click)
                0 => {
                    let dur = 0.14 + dec * 0.65;
                    if t > dur {
                        v.active = false;
                        0.0
                    } else {
                        let pitch_env = (-t * 36.0).exp();
                        let freq = (48.0 + 135.0 * pitch_env) * tr_ratio;
                        v.phase = (v.phase + freq * dt).fract();
                        let body = (TAU * v.phase).sin();
                        let click = (-t * 280.0).exp() * noise * 0.45;
                        let amp = (-t / (0.06 + dec * 0.22)).exp();
                        ((body * 1.35 + click) * amp).tanh() * 0.95
                    }
                }
                // 1: Snare (dual shell tones + snappy bandpass noise)
                1 => {
                    let dur = 0.10 + dec * 0.36;
                    if t > dur {
                        v.active = false;
                        0.0
                    } else {
                        let f1 = 185.0 * tr_ratio * (1.0 + 0.35 * (-t * 45.0).exp());
                        let f2 = 330.0 * tr_ratio;
                        v.phase = (v.phase + f1 * dt).fract();
                        v.phase2 = (v.phase2 + f2 * dt).fract();
                        let body = ((TAU * v.phase).sin() * 0.65 + (TAU * v.phase2).sin() * 0.35)
                            * (-t * 24.0).exp();
                        // Simple bandpass on noise
                        v.bp_lp += (noise - v.bp_lp) * 0.35;
                        v.bp_hp += (v.bp_lp - v.bp_hp) * 0.08;
                        let wire = (v.bp_lp - v.bp_hp) * (-t / (0.04 + dec * 0.14)).exp();
                        (body * 0.75 + wire * 1.15).tanh() * 0.85
                    }
                }
                // 2: Hand Clap (4 staggered bursts + tail)
                2 => {
                    let dur = 0.14 + dec * 0.38;
                    if t > dur {
                        v.active = false;
                        0.0
                    } else {
                        let burst = if t < 0.036 {
                            let local = (t % 0.011) / 0.011;
                            (1.0 - local).powf(2.2)
                        } else {
                            (-((t - 0.036) / (0.05 + dec * 0.16))).exp() * 0.72
                        };
                        v.bp_lp += (noise - v.bp_lp) * 0.28;
                        v.bp_hp += (v.bp_lp - v.bp_hp) * 0.10;
                        ((v.bp_lp - v.bp_hp) * burst * 2.2).tanh() * 0.80
                    }
                }
                // 3 & 4: Closed / Open Hi-Hat (6-oscillator 808 metallic square cluster + HPF)
                3 | 4 => {
                    let dur = if vi == 3 {
                        0.035 + dec * 0.11
                    } else {
                        0.18 + dec * 0.62
                    };
                    if t > dur {
                        v.active = false;
                        0.0
                    } else {
                        let base_f = 310.0 * tr_ratio;
                        v.phase = (v.phase + base_f * dt).fract();
                        // 6 inharmonic metallic partials folded from phase + noise
                        let mut metal = 0.0_f32;
                        for r in [1.0_f32, 1.342, 1.617, 2.053, 2.681, 3.319] {
                            let s = if ((v.phase * r) % 1.0) < 0.5 {
                                1.0
                            } else {
                                -1.0
                            };
                            metal += s;
                        }
                        let raw = metal * 0.12 + noise * 0.38;
                        // 1-pole highpass
                        let hp = 0.84 * (v.hp_prev_out + raw - v.hp_prev_in);
                        v.hp_prev_in = raw;
                        v.hp_prev_out = hp;
                        let tau = if vi == 3 {
                            0.014 + dec * 0.04
                        } else {
                            0.07 + dec * 0.24
                        };
                        hp * (-t / tau).exp() * 0.75
                    }
                }
                // 5: Low Tom / Acid Conga
                5 => {
                    let dur = 0.14 + dec * 0.45;
                    if t > dur {
                        v.active = false;
                        0.0
                    } else {
                        let freq = (95.0 + 95.0 * (-t * 18.0).exp()) * tr_ratio;
                        v.phase = (v.phase + freq * dt).fract();
                        let amp = (-t / (0.06 + dec * 0.18)).exp();
                        ((TAU * v.phase).sin() * 1.15 * amp).tanh() * 0.85
                    }
                }
                // 6: 808 Cowbell / Rimshot (540 Hz + 800 Hz square-wave bandpass)
                6 => {
                    let dur = 0.08 + dec * 0.28;
                    if t > dur {
                        v.active = false;
                        0.0
                    } else {
                        v.phase = (v.phase + 540.0 * tr_ratio * dt).fract();
                        v.phase2 = (v.phase2 + 800.0 * tr_ratio * dt).fract();
                        let sq1 = if v.phase < 0.5 { 1.0 } else { -1.0 };
                        let sq2 = if v.phase2 < 0.5 { 1.0 } else { -1.0 };
                        let raw = (sq1 + sq2) * 0.5;
                        v.bp_lp += (raw - v.bp_lp) * 0.25;
                        v.bp_hp += (v.bp_lp - v.bp_hp) * 0.08;
                        let amp = (-t / (0.025 + dec * 0.09)).exp();
                        (v.bp_lp - v.bp_hp) * amp * 0.95
                    }
                }
                // 7: Crash / Ride Cymbal
                _ => {
                    let dur = 0.35 + dec * 1.25;
                    if t > dur {
                        v.active = false;
                        0.0
                    } else {
                        v.phase = (v.phase + 420.0 * tr_ratio * dt).fract();
                        let ring = ((TAU * v.phase * 1.47).sin() * (TAU * v.phase * 2.83).sin())
                            * 0.35
                            + noise * 0.65;
                        let hp = 0.88 * (v.hp_prev_out + ring - v.hp_prev_in);
                        v.hp_prev_in = ring;
                        v.hp_prev_out = hp;
                        let amp = (-t / (0.14 + dec * 0.45)).exp();
                        hp * amp * 0.65
                    }
                }
            };

            v.age_s += dt;
            if let Some(tr) = patch.tracks.get(vi) {
                let gain = sample * v.vel * tr.level.clamp(0.0, 1.5) * master;
                let pan = tr.pan.clamp(-1.0, 1.0);
                let gl = ((1.0 - pan) * 0.5).sqrt();
                let gr = ((1.0 + pan) * 0.5).sqrt();
                sum_l += gain * gl;
                sum_r += gain * gr;
            }
        }

        (sum_l, sum_r)
    }
}
