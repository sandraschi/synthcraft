//! Standard MIDI File (`.mid`, SMF Format 0 and Format 1) importer that quantizes the
//! first 16 sixteenth-note slots into `SynthPatch::steps` (plus tempo if present).

use crate::error::SynthError;
use crate::model::SequencerStep;
use std::path::Path;

/// Result of importing a `.mid` file into a 16-step sequencer pattern.
#[derive(Debug, Clone)]
pub struct ImportedMidiPattern {
    pub steps: Vec<SequencerStep>,
    pub bpm: Option<f32>,
    pub note_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct NoteEvent {
    tick: u64,
    note: u8,
    velocity: u8,
    is_on: bool,
}

fn read_vlq(data: &[u8], pos: &mut usize) -> Option<u64> {
    let mut val = 0u64;
    for _ in 0..4 {
        let b = *data.get(*pos)?;
        *pos += 1;
        val = (val << 7) | u64::from(b & 0x7F);
        if b & 0x80 == 0 {
            return Some(val);
        }
    }
    Some(val)
}

/// Parse raw `.mid` bytes into a 16-step `SequencerStep` pattern.
pub fn parse_smf_to_steps(bytes: &[u8]) -> Result<ImportedMidiPattern, SynthError> {
    if bytes.len() < 14 || &bytes[0..4] != b"MThd" {
        return Err(SynthError::Validation(
            "Not a valid Standard MIDI File (MThd)".into(),
        ));
    }
    let hdr_len = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    if hdr_len < 6 || 8 + hdr_len > bytes.len() {
        return Err(SynthError::Validation("Truncated MIDI header".into()));
    }
    let division = u16::from_be_bytes([bytes[12], bytes[13]]);
    if division == 0 || (division & 0x8000) != 0 {
        return Err(SynthError::Validation(
            "SMPTE timecode MIDI files are not supported (expected PPQN)".into(),
        ));
    }
    let ppqn = f64::from(division);

    let mut pos = 8 + hdr_len;
    let mut events: Vec<NoteEvent> = Vec::new();
    let mut bpm: Option<f32> = None;

    while pos + 8 <= bytes.len() {
        let tag = &bytes[pos..pos + 4];
        let trk_len = u32::from_be_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;
        pos += 8;
        if pos + trk_len > bytes.len() {
            break;
        }
        let trk = &bytes[pos..pos + trk_len];
        pos += trk_len;
        if tag != b"MTrk" {
            continue;
        }

        let mut tpos = 0usize;
        let mut tick = 0u64;
        let mut running_status = 0u8;
        while tpos < trk.len() {
            let Some(delta) = read_vlq(trk, &mut tpos) else {
                break;
            };
            tick = tick.saturating_add(delta);
            let Some(&first) = trk.get(tpos) else { break };
            let status = if first & 0x80 != 0 {
                tpos += 1;
                if first < 0xF0 {
                    running_status = first;
                }
                first
            } else {
                running_status
            };

            match status {
                0xFF => {
                    // Meta event
                    let Some(&meta_type) = trk.get(tpos) else {
                        break;
                    };
                    tpos += 1;
                    let Some(len) = read_vlq(trk, &mut tpos) else {
                        break;
                    };
                    let len = len as usize;
                    if tpos + len > trk.len() {
                        break;
                    }
                    if meta_type == 0x51 && len == 3 && bpm.is_none() {
                        let us_per_q = (u32::from(trk[tpos]) << 16)
                            | (u32::from(trk[tpos + 1]) << 8)
                            | u32::from(trk[tpos + 2]);
                        if us_per_q > 0 {
                            bpm = Some((60_000_000.0 / f64::from(us_per_q)) as f32);
                        }
                    }
                    tpos += len;
                }
                0xF0 | 0xF7 => {
                    let Some(len) = read_vlq(trk, &mut tpos) else {
                        break;
                    };
                    tpos = (tpos + len as usize).min(trk.len());
                }
                s if (0x80..0xF0).contains(&s) => {
                    let kind = s & 0xF0;
                    let data_len = if matches!(kind, 0xC0 | 0xD0) { 1 } else { 2 };
                    if tpos + data_len > trk.len() {
                        break;
                    }
                    let d1 = trk[tpos] & 0x7F;
                    let d2 = if data_len == 2 {
                        trk[tpos + 1] & 0x7F
                    } else {
                        0
                    };
                    tpos += data_len;
                    if kind == 0x90 && d2 > 0 {
                        events.push(NoteEvent {
                            tick,
                            note: d1,
                            velocity: d2,
                            is_on: true,
                        });
                    } else if kind == 0x80 || (kind == 0x90 && d2 == 0) {
                        events.push(NoteEvent {
                            tick,
                            note: d1,
                            velocity: 0,
                            is_on: false,
                        });
                    }
                }
                _ => break,
            }
        }
    }

    let step_ticks = (ppqn / 4.0).max(1.0);
    let mut steps: Vec<SequencerStep> = (0..16)
        .map(|i| SequencerStep {
            step_index: i,
            midi_note: 48,
            velocity: 0.75,
            gate_pct: 0.6,
            accent: false,
            slide: false,
            active: false,
        })
        .collect();

    events.sort_by_key(|e| e.tick);
    let mut note_count = 0usize;
    let mut active_notes = 0u32;
    for ev in &events {
        if ev.is_on {
            let slot = (ev.tick as f64 / step_ticks).round() as usize;
            if slot < 16 {
                let s = &mut steps[slot];
                s.midi_note = ev.note.clamp(24, 96);
                s.velocity = (f32::from(ev.velocity) / 127.0).clamp(0.1, 1.0);
                s.accent = ev.velocity >= 110;
                s.slide = active_notes > 0;
                s.active = true;
                note_count += 1;
            }
            active_notes = active_notes.saturating_add(1);
        } else {
            active_notes = active_notes.saturating_sub(1);
        }
    }

    if note_count == 0 {
        return Err(SynthError::Validation(
            "No MIDI notes found in the first bar of the file".into(),
        ));
    }

    Ok(ImportedMidiPattern {
        steps,
        bpm: bpm.map(|b| b.clamp(40.0, 240.0)),
        note_count,
    })
}

/// Read a `.mid` file from disk and quantize it into 16 sequencer steps.
pub fn import_midi_smf_file(path: &Path) -> Result<ImportedMidiPattern, SynthError> {
    let bytes = std::fs::read(path).map_err(|e| {
        SynthError::Validation(format!("Cannot read MIDI file {}: {e}", path.display()))
    })?;
    parse_smf_to_steps(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rompler::{builtin_midi_depot, export_midi_smf_file};

    #[test]
    fn round_trips_exported_midi_depot_clip_back_into_16_steps() {
        let clips = builtin_midi_depot();
        let clip = &clips[0];
        let mut path = std::env::temp_dir();
        path.push(format!("synthcraft_smf_import_{}.mid", std::process::id()));
        let _ = std::fs::remove_file(&path);
        assert!(export_midi_smf_file(clip, &path).is_ok());

        let imported = import_midi_smf_file(&path);
        let _ = std::fs::remove_file(&path);
        assert!(imported.is_ok());
        if let Ok(pat) = imported {
            assert_eq!(pat.steps.len(), 16);
            assert!(pat.note_count >= 4);
            assert!(pat.steps.iter().any(|s| s.active));
        }
    }
}
