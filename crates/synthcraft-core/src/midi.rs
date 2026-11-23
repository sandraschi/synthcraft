//! Parsing of raw MIDI bytes into engine events. Pure and device-free so it is unit tested.

/// A decoded channel-voice message the synthesizer reacts to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MidiEvent {
    NoteOn {
        channel: u8,
        note: u8,
        velocity: f32,
    },
    NoteOff {
        channel: u8,
        note: u8,
    },
    Controller {
        channel: u8,
        cc: u8,
        value: f32,
    },
    /// Pitch wheel, `-1.0..=1.0`.
    PitchBend {
        channel: u8,
        value: f32,
    },
    AllNotesOff {
        channel: u8,
    },
}

impl MidiEvent {
    /// Channel as `1..=16`.
    pub fn channel(self) -> u8 {
        match self {
            Self::NoteOn { channel, .. }
            | Self::NoteOff { channel, .. }
            | Self::Controller { channel, .. }
            | Self::PitchBend { channel, .. }
            | Self::AllNotesOff { channel } => channel,
        }
    }

    /// Decode one MIDI message. Running status, system and realtime messages are not decoded.
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let status = *bytes.first()?;
        if !(0x80..0xF0).contains(&status) {
            return None;
        }
        let channel = (status & 0x0F) + 1;
        let d1 = bytes.get(1).copied().unwrap_or(0) & 0x7F;
        let d2 = bytes.get(2).copied().unwrap_or(0) & 0x7F;
        match status & 0xF0 {
            0x80 => Some(Self::NoteOff { channel, note: d1 }),
            0x90 if d2 == 0 => Some(Self::NoteOff { channel, note: d1 }),
            0x90 => Some(Self::NoteOn {
                channel,
                note: d1,
                velocity: f32::from(d2) / 127.0,
            }),
            0xB0 if d1 == 123 => Some(Self::AllNotesOff { channel }),
            0xB0 => Some(Self::Controller {
                channel,
                cc: d1,
                value: f32::from(d2) / 127.0,
            }),
            0xE0 => {
                let raw = (i32::from(d2) << 7) | i32::from(d1);
                Some(Self::PitchBend {
                    channel,
                    value: ((raw - 8192) as f32 / 8192.0).clamp(-1.0, 1.0),
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_on_with_zero_velocity_is_a_note_off() {
        assert_eq!(
            MidiEvent::parse(&[0x92, 60, 0]),
            Some(MidiEvent::NoteOff {
                channel: 3,
                note: 60
            })
        );
    }

    #[test]
    fn pitch_bend_centre_is_zero_and_extremes_reach_plus_minus_one() {
        let v = |lsb: u8, msb: u8| match MidiEvent::parse(&[0xE0, lsb, msb]) {
            Some(MidiEvent::PitchBend { value, .. }) => value,
            other => panic_free_fail(other),
        };
        assert!(v(0, 64).abs() < 1e-6);
        assert!(v(0, 0) <= -0.999);
        assert!(v(127, 127) > 0.999);
    }

    fn panic_free_fail(_: Option<MidiEvent>) -> f32 {
        f32::NAN
    }

    #[test]
    fn realtime_and_truncated_messages_are_ignored() {
        assert_eq!(MidiEvent::parse(&[0xF8]), None);
        assert_eq!(MidiEvent::parse(&[]), None);
        assert_eq!(MidiEvent::parse(&[0x40, 1, 2]), None);
    }
}
