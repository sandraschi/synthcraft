//! Live DJCraft cooperation bridge:
//! - Loopback UDP Link follower (`127.0.0.1:20809`, `DJLK` 40-byte beacon) for real-time BPM,
//!   beat phase, play/stop state, and Camelot key sync from a running DJCraft instance.
//! - Auto-Camelot harmonic transposition (`camelot_to_root_semitone` + `transpose_patch_to_camelot`)
//!   so SynthCraft's 16-step sequencer follows DJCraft's active deck key with zero clashing notes.
//! - Direct JSON-over-TCP IPC client to DJCraft's `%LOCALAPPDATA%\djcraft\live.port` for
//!   live status queries and 1-click "Forge & Load onto DJCraft Deck A/B/C/D".

#[cfg(not(target_arch = "wasm32"))]
use crate::SynthError;
use crate::SynthPatch;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use serde_json::{json, Value};
#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufRead, BufReader, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream, UdpSocket};
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

/// Loopback UDP port where DJCraft mirrors its `DJLK` Link beacons for local Craft apps.
pub const DJCRAFT_LINK_LOOPBACK_PORT: u16 = 20809;
const MAGIC: &[u8; 4] = b"DJLK";

/// Decoded live Link beacon from DJCraft.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DjcraftBeacon {
    pub node_id: u64,
    pub bpm: f64,
    pub beat_phase: f64,
    pub playing: bool,
    pub camelot_key: String,
}

impl DjcraftBeacon {
    /// Decode a 28-byte (legacy) or 40-byte (v2 with playing + Camelot key) `DJLK` beacon.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 28 || &bytes[0..4] != MAGIC {
            return None;
        }
        let node_id = u64::from_le_bytes(bytes[4..12].try_into().ok()?);
        let bpm = f64::from_le_bytes(bytes[12..20].try_into().ok()?);
        let beat_phase = f64::from_le_bytes(bytes[20..28].try_into().ok()?);
        if !bpm.is_finite() || !(20.0..=300.0).contains(&bpm) || !beat_phase.is_finite() {
            return None;
        }
        let mut playing = false;
        let mut camelot_key = String::new();
        if bytes.len() >= 38 {
            playing = bytes[28] != 0;
            let len = usize::from(bytes[29]).min(8);
            if let Ok(s) = std::str::from_utf8(&bytes[30..30 + len]) {
                camelot_key = s.trim_end_matches('\0').trim().to_string();
            }
        }
        Some(Self {
            node_id,
            bpm,
            beat_phase: beat_phase.rem_euclid(4.0),
            playing,
            camelot_key,
        })
    }
}

/// Map a Camelot wheel code (`1A`..`12A`, `1B`..`12B`, optionally followed by musical name like
/// `"8A Am"`) or standalone musical key (`"Am"`, `"F#m"`, `"C"`) to its root pitch class (`0..=11`,
/// where `C = 0`, `C# = 1`, ..., `B = 11`).
pub fn camelot_to_root_semitone(key: &str) -> Option<i32> {
    let token = key.split_whitespace().next()?.trim().to_ascii_uppercase();
    match token.as_str() {
        // Minor (A) wheel
        "1A" | "ABM" | "G#M" => Some(8),
        "2A" | "EBM" | "D#M" => Some(3),
        "3A" | "BBM" | "A#M" => Some(10),
        "4A" | "FM" => Some(5),
        "5A" | "CM" => Some(0),
        "6A" | "GM" => Some(7),
        "7A" | "DM" => Some(2),
        "8A" | "AM" => Some(9),
        "9A" | "EM" => Some(4),
        "10A" | "BM" => Some(11),
        "11A" | "F#M" | "GBM" => Some(6),
        "12A" | "C#M" | "DBM" => Some(1),
        // Major (B) wheel
        "1B" | "B" => Some(11),
        "2B" | "F#" | "GB" => Some(6),
        "3B" | "DB" | "C#" => Some(1),
        "4B" | "AB" | "G#" => Some(8),
        "5B" | "EB" | "D#" => Some(3),
        "6B" | "BB" | "A#" => Some(10),
        "7B" | "F" => Some(5),
        "8B" | "C" => Some(0),
        "9B" | "G" => Some(7),
        "10B" | "D" => Some(2),
        "11B" | "A" => Some(9),
        "12B" | "E" => Some(4),
        _ => None,
    }
}

/// Transpose all 16 sequencer steps of `patch` by the shortest signed semitone interval
/// (`-6..=+6`) from `patch.camelot_key` to `target_camelot`, updating `patch.camelot_key`.
/// Returns the applied semitone shift (`0` if already in the same key or unrecognized).
pub fn transpose_patch_to_camelot(patch: &mut SynthPatch, target_camelot: &str) -> i32 {
    let (Some(from_pc), Some(to_pc)) = (
        camelot_to_root_semitone(&patch.camelot_key),
        camelot_to_root_semitone(target_camelot),
    ) else {
        return 0;
    };
    let mut delta = (to_pc - from_pc).rem_euclid(12);
    if delta > 6 {
        delta -= 12;
    }
    if delta != 0 {
        for step in &mut patch.steps {
            let shifted = (i32::from(step.midi_note) + delta).clamp(24, 96);
            step.midi_note = shifted as u8;
        }
    }
    patch.camelot_key = target_camelot.trim().to_string();
    delta
}

/// Non-blocking UDP listener for DJCraft's `DJLK` loopback beacons (`127.0.0.1:20809`).
#[cfg(not(target_arch = "wasm32"))]
pub struct DjcraftLinkReceiver {
    socket: Option<UdpSocket>,
    last_beacon: Option<DjcraftBeacon>,
    last_seen: Option<Instant>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for DjcraftLinkReceiver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl DjcraftLinkReceiver {
    pub fn new() -> Self {
        Self {
            socket: None,
            last_beacon: None,
            last_seen: None,
        }
    }

    pub fn ensure_bound(&mut self) -> bool {
        if self.socket.is_some() {
            return true;
        }
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, DJCRAFT_LINK_LOOPBACK_PORT);
        if let Ok(sock) = UdpSocket::bind(addr) {
            let _ = sock.set_nonblocking(true);
            self.socket = Some(sock);
            true
        } else {
            false
        }
    }

    /// Drain pending UDP packets and return the latest decoded `DjcraftBeacon`, if any arrived.
    pub fn poll(&mut self) -> Option<DjcraftBeacon> {
        let sock = self.socket.as_ref()?;
        let mut latest = None;
        let mut buf = [0u8; 64];
        while let Ok((n, _)) = sock.recv_from(&mut buf) {
            if let Some(b) = DjcraftBeacon::decode(&buf[..n]) {
                latest = Some(b);
            }
        }
        if let Some(ref b) = latest {
            self.last_beacon = Some(b.clone());
            self.last_seen = Some(Instant::now());
        }
        latest
    }

    pub fn last_beacon(&self) -> Option<&DjcraftBeacon> {
        self.last_beacon.as_ref()
    }

    pub fn is_connected(&self) -> bool {
        self.last_seen
            .is_some_and(|t| t.elapsed() < Duration::from_secs(2))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn djcraft_live_port_path() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)?;
    Some(base.join("djcraft").join("live.port"))
}

/// Read the active TCP IPC port of a running `djcraft.exe` GUI from `%LOCALAPPDATA%\djcraft\live.port`.
#[cfg(not(target_arch = "wasm32"))]
pub fn djcraft_live_port() -> Result<u16, SynthError> {
    let path = djcraft_live_port_path()
        .ok_or_else(|| SynthError::Io("cannot locate LOCALAPPDATA/XDG_CONFIG_HOME".into()))?;
    let raw = std::fs::read_to_string(&path).map_err(|_| {
        SynthError::Io(format!(
            "DJCraft live.port not found at {} (is DJCraft running?)",
            path.display()
        ))
    })?;
    let val: Value = serde_json::from_str(&raw)
        .map_err(|e| SynthError::Serialization(format!("invalid djcraft live.port: {e}")))?;
    val.get("port")
        .and_then(Value::as_u64)
        .map(|p| p as u16)
        .ok_or_else(|| SynthError::Serialization("missing 'port' in djcraft live.port".into()))
}

#[cfg(not(target_arch = "wasm32"))]
fn send_djcraft_ipc(req: &Value) -> Result<Value, SynthError> {
    let port = djcraft_live_port()?;
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let mut stream =
        TcpStream::connect_timeout(&addr.into(), Duration::from_millis(800)).map_err(|e| {
            SynthError::Io(format!(
                "cannot connect to DJCraft on 127.0.0.1:{port}: {e}"
            ))
        })?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let line = serde_json::to_string(req)
        .map_err(|e| SynthError::Serialization(format!("encode IPC request failed: {e}")))?;
    writeln!(stream, "{line}")
        .map_err(|e| SynthError::Io(format!("write to DJCraft IPC failed: {e}")))?;
    let _ = stream.flush();

    let mut resp_line = String::new();
    let mut reader = BufReader::new(stream);
    reader
        .read_line(&mut resp_line)
        .map_err(|e| SynthError::Io(format!("read from DJCraft IPC failed: {e}")))?;
    serde_json::from_str(resp_line.trim())
        .map_err(|e| SynthError::Serialization(format!("invalid DJCraft IPC response: {e}")))
}

/// Query a running `djcraft.exe` GUI for its live 4-deck BPM, playing state, and Camelot keys.
#[cfg(not(target_arch = "wasm32"))]
pub fn query_djcraft_live_status() -> Result<Value, SynthError> {
    send_djcraft_ipc(&json!({ "op": "live_status" }))
}

/// Load a rendered WAV stem directly onto Deck A/B/C/D of a running `djcraft.exe` GUI.
#[cfg(not(target_arch = "wasm32"))]
pub fn send_stem_to_djcraft_deck(deck: &str, wav_path: &Path) -> Result<Value, SynthError> {
    let deck_enum = match deck.trim().to_ascii_lowercase().as_str() {
        "a" | "1" | "decka" | "deck_a" => "DeckA",
        "b" | "2" | "deckb" | "deck_b" => "DeckB",
        "c" | "3" | "deckc" | "deck_c" => "DeckC",
        "d" | "4" | "deckd" | "deck_d" => "DeckD",
        other => {
            return Err(SynthError::Validation(format!(
                "invalid DJCraft deck '{other}' (expected A, B, C, or D)"
            )));
        }
    };
    let abs = std::fs::canonicalize(wav_path).unwrap_or_else(|_| wav_path.to_path_buf());
    send_djcraft_ipc(&json!({
        "op": "load_file",
        "deck": deck_enum,
        "path": abs.to_string_lossy(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SynthEngine;

    #[test]
    fn decodes_40_byte_djcraft_beacon_with_playing_and_camelot() {
        let mut buf = [0u8; 40];
        buf[0..4].copy_from_slice(b"DJLK");
        buf[4..12].copy_from_slice(&42_u64.to_le_bytes());
        buf[12..20].copy_from_slice(&128.0_f64.to_le_bytes());
        buf[20..28].copy_from_slice(&1.5_f64.to_le_bytes());
        buf[28] = 1;
        buf[29] = 5;
        buf[30..35].copy_from_slice(b"9A Em");

        let b = DjcraftBeacon::decode(&buf);
        assert!(b.is_some());
        if let Some(b) = b {
            assert_eq!(b.node_id, 42);
            assert!((b.bpm - 128.0).abs() < 1e-6);
            assert!(b.playing);
            assert_eq!(b.camelot_key, "9A Em");
        }
    }

    #[test]
    fn camelot_transpose_shifts_shortest_interval_and_updates_steps() {
        let mut eng = SynthEngine::factory_seed();
        let p = eng.active_patch_mut();
        p.camelot_key = "8A Am".into(); // A = 9
        let orig_note = p.steps[0].midi_note;
        // 8A (A=9) -> 9A (E=4): shortest shift is -5 semitones
        let shift = transpose_patch_to_camelot(p, "9A Em");
        assert_eq!(shift, -5);
        assert_eq!(p.camelot_key, "9A Em");
        assert_eq!(
            p.steps[0].midi_note,
            (i32::from(orig_note) - 5).clamp(24, 96) as u8
        );
    }
}
