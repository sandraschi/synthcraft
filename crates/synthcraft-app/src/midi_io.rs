//! Hardware MIDI input (midir). Notes go straight to the audio thread for low latency;
//! controllers, pitch bend and channel filtering are applied here, and CC events that change
//! patch parameters are forwarded to the UI thread, which owns the patch.

use crate::audio_io::AudioMsg;
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use synthcraft_core::midi::MidiEvent;

/// Settings the MIDI thread reads live.
pub struct MidiShared {
    /// 0 = omni, 1..=16 = one channel.
    pub channel: AtomicU8,
    /// Pitch-bend range in semitones.
    pub bend_range: AtomicU8,
}

impl MidiShared {
    pub fn new(channel: u8, bend_range: u8) -> Arc<Self> {
        Arc::new(Self {
            channel: AtomicU8::new(channel),
            bend_range: AtomicU8::new(bend_range),
        })
    }
}

pub struct MidiHandle {
    _conn: MidiInputConnection<()>,
    pub port_name: String,
}

pub fn list_ports() -> Vec<String> {
    let Ok(input) = MidiInput::new("synthcraft-list") else {
        return Vec::new();
    };
    input
        .ports()
        .iter()
        .filter_map(|p| input.port_name(p).ok())
        .collect()
}

/// Connect to the port called `name`. Note messages go to `audio`; other events to `ui`.
pub fn connect(
    name: &str,
    audio: Sender<AudioMsg>,
    ui: Sender<MidiEvent>,
    shared: Arc<MidiShared>,
    repaint: egui::Context,
) -> Result<MidiHandle, String> {
    let mut input = MidiInput::new("synthcraft").map_err(|e| format!("MIDI init failed: {e}"))?;
    input.ignore(Ignore::All);
    let ports = input.ports();
    let port = ports
        .iter()
        .find(|p| input.port_name(p).map(|n| n == name).unwrap_or(false))
        .ok_or_else(|| format!("MIDI port '{name}' not found"))?;
    let port_name = input.port_name(port).unwrap_or_else(|_| name.to_string());
    let conn = input
        .connect(
            port,
            "synthcraft-in",
            move |_stamp, bytes, _| {
                let Some(ev) = MidiEvent::parse(bytes) else {
                    return;
                };
                let want = shared.channel.load(Ordering::Relaxed);
                if want != 0 && ev.channel() != want {
                    return;
                }
                match ev {
                    MidiEvent::NoteOn { note, velocity, .. } => {
                        let _ = audio.send(AudioMsg::NoteOn(note, velocity));
                        let _ = ui.send(ev);
                        repaint.request_repaint();
                    }
                    MidiEvent::NoteOff { note, .. } => {
                        let _ = audio.send(AudioMsg::NoteOff(note));
                    }
                    MidiEvent::AllNotesOff { .. } => {
                        let _ = audio.send(AudioMsg::AllNotesOff);
                    }
                    MidiEvent::PitchBend { value, .. } => {
                        let range = f32::from(shared.bend_range.load(Ordering::Relaxed));
                        let _ = audio.send(AudioMsg::PitchBend(value * range));
                    }
                    MidiEvent::Controller { cc: 1, value, .. } => {
                        let _ = audio.send(AudioMsg::ModWheel(value));
                        let _ = ui.send(ev);
                        repaint.request_repaint();
                    }
                    MidiEvent::Controller { .. } => {
                        let _ = ui.send(ev);
                        repaint.request_repaint();
                    }
                }
            },
            (),
        )
        .map_err(|e| format!("cannot open MIDI port: {e}"))?;
    Ok(MidiHandle {
        _conn: conn,
        port_name,
    })
}
