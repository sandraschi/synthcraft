//! One-Shot #10: Polyphonic 1-Finger Chord Memory, Scale Lock Quantizer & Riff Generator.
//!
//! - Maps any single incoming MIDI note into rich voiced polyphonic chords (Power 5th, Triads,
//!   7ths, 9ths, House Stab m9, Dub Techno m11, Sus4, Vienna Baroque 6/4) with inversion & Drop-2 spread.
//! - Scale Lock Quantizer snaps notes to Chromatic, Minor, Major, Dorian, Phrygian, Harmonic Minor,
//!   Hirajoshi, or Blues scales.
//! - One-click musical riff generator populates the 16..64-step sequencer in key.

use crate::model::SequencerStep;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChordMode {
    Off,
    Power5th,
    MinorTriad,
    MajorTriad,
    Minor7th,
    Major7th,
    HouseStabM9,
    DubTechnoM11,
    TechnoSus4,
    ViennaBaroque64,
}

impl ChordMode {
    pub const ALL: [Self; 10] = [
        Self::Off,
        Self::Power5th,
        Self::MinorTriad,
        Self::MajorTriad,
        Self::Minor7th,
        Self::Major7th,
        Self::HouseStabM9,
        Self::DubTechnoM11,
        Self::TechnoSus4,
        Self::ViennaBaroque64,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off (Single Note)",
            Self::Power5th => "Power 5th (+7)",
            Self::MinorTriad => "Minor Triad",
            Self::MajorTriad => "Major Triad",
            Self::Minor7th => "Minor 7th",
            Self::Major7th => "Major 7th",
            Self::HouseStabM9 => "90s House Stab (m9)",
            Self::DubTechnoM11 => "Dub Techno (m11)",
            Self::TechnoSus4 => "Techno Sus4",
            Self::ViennaBaroque64 => "Vienna Baroque 6/4",
        }
    }

    pub fn intervals(self) -> &'static [i8] {
        match self {
            Self::Off => &[0],
            Self::Power5th => &[0, 7, 12],
            Self::MinorTriad => &[0, 3, 7],
            Self::MajorTriad => &[0, 4, 7],
            Self::Minor7th => &[0, 3, 7, 10],
            Self::Major7th => &[0, 4, 7, 11],
            Self::HouseStabM9 => &[0, 3, 7, 10, 14],
            Self::DubTechnoM11 => &[0, 3, 7, 10, 17],
            Self::TechnoSus4 => &[0, 5, 7, 12],
            Self::ViennaBaroque64 => &[-5, 0, 4, 12],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleLock {
    Chromatic,
    NaturalMinor,
    Major,
    Dorian,
    Phrygian,
    HarmonicMinor,
    Hirajoshi,
    Blues,
}

impl ScaleLock {
    pub const ALL: [Self; 8] = [
        Self::Chromatic,
        Self::NaturalMinor,
        Self::Major,
        Self::Dorian,
        Self::Phrygian,
        Self::HarmonicMinor,
        Self::Hirajoshi,
        Self::Blues,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Chromatic => "Chromatic (Off)",
            Self::NaturalMinor => "Natural Minor",
            Self::Major => "Major (Ionian)",
            Self::Dorian => "Dorian (House/Funk)",
            Self::Phrygian => "Phrygian (Acid/Techno)",
            Self::HarmonicMinor => "Harmonic Minor",
            Self::Hirajoshi => "Hirajoshi (Japanese)",
            Self::Blues => "Blues Hexatonic",
        }
    }

    pub fn degrees(self) -> &'static [u8] {
        match self {
            Self::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            Self::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            Self::Major => &[0, 2, 4, 5, 7, 9, 11],
            Self::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Self::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            Self::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            Self::Hirajoshi => &[0, 2, 3, 7, 8],
            Self::Blues => &[0, 3, 5, 6, 7, 10],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChordMemoryParams {
    pub chord_mode: ChordMode,
    pub scale_lock: ScaleLock,
    pub root_semitone: u8, // 0=C .. 9=A .. 11=B
    pub inversion: u8,     // 0..=3
    pub drop2_spread: bool,
}

impl Default for ChordMemoryParams {
    fn default() -> Self {
        Self {
            chord_mode: ChordMode::Off,
            scale_lock: ScaleLock::Chromatic,
            root_semitone: 9, // A (matches 8A Am default)
            inversion: 0,
            drop2_spread: false,
        }
    }
}

impl ChordMemoryParams {
    /// Snap a MIDI note to the active scale & root. Allocation-free.
    #[inline]
    pub fn quantize_note(&self, note: u8) -> u8 {
        if self.scale_lock == ScaleLock::Chromatic {
            return note.min(127);
        }
        let root = self.root_semitone % 12;
        let rel = (note + 12 - root) % 12;
        let degs = self.scale_lock.degrees();
        let mut best_deg = degs[0];
        let mut best_dist = 12u8;
        for &d in degs {
            let diff = rel.abs_diff(d);
            let dist = diff.min(12 - diff);
            if dist < best_dist {
                best_dist = dist;
                best_deg = d;
            }
        }
        let oct = note / 12;
        (oct * 12 + root + best_deg).min(127)
    }

    /// Expand a single root note into up to 5 voiced MIDI notes (stack-allocated, zero heap alloc).
    #[inline]
    pub fn expand_chord(&self, root_note: u8) -> ([u8; 5], usize) {
        let base = self.quantize_note(root_note);
        let ivs = self.chord_mode.intervals();
        let mut out = [base; 5];
        let mut count = 0usize;
        let inv = (self.inversion as usize).min(3);

        for (i, &iv) in ivs.iter().take(5).enumerate() {
            let mut n = i16::from(base) + i16::from(iv);
            if i < inv {
                n += 12;
            }
            if self.drop2_spread && i == 1 && ivs.len() >= 3 {
                n -= 12;
            }
            out[count] = n.clamp(24, 108) as u8;
            count += 1;
        }
        (out, count.max(1))
    }

    /// Generate a musical scale-locked riff for the active sequencer length (`16..=64` steps).
    pub fn generate_riff(&self, len: usize, base_octave: u8, style: &str) -> Vec<SequencerStep> {
        let n_steps = len.clamp(16, 64);
        let root = (base_octave.clamp(2, 5) * 12) + (self.root_semitone % 12);
        let degs = if self.scale_lock == ScaleLock::Chromatic {
            ScaleLock::NaturalMinor.degrees()
        } else {
            self.scale_lock.degrees()
        };
        let mut out = Vec::with_capacity(n_steps);
        for i in 0..n_steps {
            let bar = i / 16;
            let pos = i % 16;
            let (deg_idx, oct_add, active, accent, slide) = match style {
                "berlin_hypnotic" => {
                    let pat = [0, 0, 2, 0, 4, 0, 3, 2, 0, 0, 5, 0, 4, 3, 2, 1];
                    (
                        (pat[pos] + bar) % degs.len(),
                        if pos % 8 == 6 { 12 } else { 0 },
                        pos % 2 == 0 || pos % 5 == 0,
                        pos % 4 == 0,
                        pos % 8 == 7,
                    )
                }
                "house_chord_groove" => {
                    let pat = [0, 0, 2, 4, 0, 3, 2, 0, 4, 4, 2, 1, 3, 2, 1, 0];
                    (
                        (pat[pos] + (bar % 2) * 2) % degs.len(),
                        0,
                        matches!(pos, 0 | 3 | 6 | 10 | 12 | 14),
                        matches!(pos, 0 | 6 | 12),
                        false,
                    )
                }
                _ => {
                    // Classic Acid 303 Bassline
                    let pat = [0, 0, 0, 2, 3, 0, 4, 3, 0, 2, 5, 4, 0, 3, 2, 1];
                    (
                        (pat[pos] + bar) % degs.len(),
                        if pos == 6 || pos == 14 { 12 } else { 0 },
                        !matches!(pos, 5 | 13),
                        pos % 4 == 0 || pos == 10,
                        matches!(pos, 3 | 7 | 11 | 15),
                    )
                }
            };
            let midi_note = (root + degs[deg_idx] + oct_add).clamp(30, 90);
            out.push(SequencerStep {
                step_index: i,
                midi_note,
                velocity: if accent { 0.96 } else { 0.78 },
                gate_pct: if slide { 0.95 } else { 0.65 },
                accent,
                slide,
                active,
            });
        }
        out
    }
}
