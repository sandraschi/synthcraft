//! Real-time hardware-style Arpeggiator (Up, Down, UpDown, Random) running allocation-free
//! inside `SynthRt`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArpMode {
    #[default]
    Off,
    Up,
    Down,
    UpDown,
    Random,
}

impl ArpMode {
    pub const ALL: [Self; 5] = [Self::Off, Self::Up, Self::Down, Self::UpDown, Self::Random];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Up => "up",
            Self::Down => "down",
            Self::UpDown => "up_down",
            Self::Random => "random",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Up => "Up",
            Self::Down => "Down",
            Self::UpDown => "Up / Down",
            Self::Random => "Random",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "off" | "none" => Some(Self::Off),
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            "up_down" | "updown" | "pingpong" => Some(Self::UpDown),
            "random" | "rand" => Some(Self::Random),
            _ => None,
        }
    }
}

/// Persisted arpeggiator settings on `SynthPatch`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ArpParams {
    pub mode: ArpMode,
    /// Note division per quarter note: 1 = 1/4, 2 = 1/8, 4 = 1/16, 8 = 1/32.
    pub rate_div: u8,
    /// Octave span (`1..=4`).
    pub octaves: u8,
    /// Gate fraction of step (`0.1..=1.0`).
    pub gate: f32,
    /// Keep arpeggiating after keys are released until a new chord is pressed.
    pub latch: bool,
}

impl Default for ArpParams {
    fn default() -> Self {
        Self {
            mode: ArpMode::Off,
            rate_div: 4, // 1/16th notes
            octaves: 2,
            gate: 0.65,
            latch: false,
        }
    }
}

/// Allocation-free runtime state for the arpeggiator inside `SynthRt`.
#[derive(Debug, Clone)]
pub struct ArpRuntime {
    held: [bool; 128],
    latched: [bool; 128],
    held_vel: f32,
    step_pos: f32,
    step_idx: usize,
    seed: u32,
    current_note: Option<u8>,
    gate_open: bool,
}

impl Default for ArpRuntime {
    fn default() -> Self {
        Self {
            held: [false; 128],
            latched: [false; 128],
            held_vel: 0.85,
            step_pos: 0.0,
            step_idx: 0,
            seed: 0xA9F1_3C2B,
            current_note: None,
            gate_open: false,
        }
    }
}

/// Action emitted by one tick of `ArpRuntime`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArpAction {
    None,
    NoteOn(u8, f32),
    NoteOff(u8),
}

impl ArpRuntime {
    pub fn key_down(&mut self, note: u8, vel: f32, latch: bool) {
        let n = (note as usize).min(127);
        let had_held = self.held.iter().any(|&h| h);
        if latch && !had_held {
            // Starting a new latched chord replaces the previous latched set.
            self.latched = [false; 128];
        }
        self.held[n] = true;
        if latch {
            self.latched[n] = true;
        }
        self.held_vel = vel.clamp(0.05, 1.0);
        if !had_held {
            self.step_pos = 0.0;
            self.step_idx = 0;
        }
    }

    pub fn key_up(&mut self, note: u8) {
        let n = (note as usize).min(127);
        self.held[n] = false;
    }

    pub fn clear(&mut self) -> Option<u8> {
        self.held = [false; 128];
        self.latched = [false; 128];
        self.gate_open = false;
        self.current_note.take()
    }

    fn collect_notes(&self, params: ArpParams, buf: &mut [u8; 64]) -> usize {
        let src = if params.latch && self.latched.iter().any(|&h| h) {
            &self.latched
        } else {
            &self.held
        };
        let octs = params.octaves.clamp(1, 4);
        let mut count = 0usize;
        for oct in 0..octs {
            let offset = oct * 12;
            for (note, &on) in src.iter().enumerate() {
                if on && note + usize::from(offset) <= 127 && count < buf.len() {
                    buf[count] = (note as u8) + offset;
                    count += 1;
                }
            }
        }
        count
    }

    /// Advance the arpeggiator by 1 sample. Returns a note event if one fires on this sample.
    pub fn tick(&mut self, params: ArpParams, bpm: f32, sr: f32) -> ArpAction {
        if params.mode == ArpMode::Off {
            if let Some(n) = self.current_note.take() {
                self.gate_open = false;
                return ArpAction::NoteOff(n);
            }
            return ArpAction::None;
        }
        let mut pool = [0u8; 64];
        let count = self.collect_notes(params, &mut pool);
        if count == 0 {
            if let Some(n) = self.current_note.take() {
                self.gate_open = false;
                return ArpAction::NoteOff(n);
            }
            return ArpAction::None;
        }

        let div = f32::from(params.rate_div.clamp(1, 8));
        let step_len = (60.0 / bpm.clamp(40.0, 240.0) * sr / div).max(64.0);
        let gate_len = step_len * params.gate.clamp(0.1, 0.98);

        let action = if self.step_pos == 0.0 {
            let picked = match params.mode {
                ArpMode::Off => pool[0],
                ArpMode::Up => pool[self.step_idx % count],
                ArpMode::Down => pool[count - 1 - (self.step_idx % count)],
                ArpMode::UpDown => {
                    if count == 1 {
                        pool[0]
                    } else {
                        let cycle = count * 2 - 2;
                        let p = self.step_idx % cycle;
                        if p < count {
                            pool[p]
                        } else {
                            pool[cycle - p]
                        }
                    }
                }
                ArpMode::Random => {
                    let mut x = self.seed.max(1);
                    x ^= x << 13;
                    x ^= x >> 17;
                    x ^= x << 5;
                    self.seed = x;
                    pool[(x as usize) % count]
                }
            };
            self.step_idx = self.step_idx.wrapping_add(1);
            self.current_note = Some(picked);
            self.gate_open = true;
            ArpAction::NoteOn(picked, self.held_vel)
        } else if self.gate_open && self.step_pos >= gate_len {
            self.gate_open = false;
            match self.current_note.take() {
                Some(n) => ArpAction::NoteOff(n),
                None => ArpAction::None,
            }
        } else {
            ArpAction::None
        };

        self.step_pos += 1.0;
        if self.step_pos >= step_len {
            self.step_pos = 0.0;
        }
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arpeggiator_steps_up_across_two_octaves_for_held_chord() {
        let mut arp = ArpRuntime::default();
        let params = ArpParams {
            mode: ArpMode::Up,
            rate_div: 4,
            octaves: 2,
            gate: 0.5,
            latch: false,
        };
        arp.key_down(60, 0.9, false);
        arp.key_down(64, 0.9, false);
        let mut triggered = Vec::new();
        for _ in 0..32_000 {
            if let ArpAction::NoteOn(n, _) = arp.tick(params, 120.0, 48_000.0) {
                triggered.push(n);
            }
        }
        assert!(
            triggered.len() >= 4,
            "expected >=4 arp notes, got {:?}",
            triggered
        );
        assert_eq!(&triggered[..4], &[60, 64, 72, 76]);
    }
}
