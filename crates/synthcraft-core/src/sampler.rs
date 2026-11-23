//! Sample playback data: banks of key/velocity zones loaded from WAV, SFZ or SF2 files.
//!
//! A bank spec is a path. For SF2 files a preset index can be appended as `file.sf2#3`.
//! A directory of WAV files is mapped by note names found in the file names (`pad_C3.wav`).
//! Decoded banks are cached by spec so the UI, the audio thread and the offline renderer
//! share one copy. Nothing here runs on the audio thread except [`Zone::read`].

use crate::error::SynthError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// One playable region of a bank.
#[derive(Debug, Clone)]
pub struct Zone {
    pub data: Arc<[f32]>,
    pub sample_rate: f32,
    pub key_lo: u8,
    pub key_hi: u8,
    pub vel_lo: u8,
    pub vel_hi: u8,
    pub root_key: u8,
    pub tune_cents: f32,
    pub gain: f32,
    /// Loop start and end (end exclusive) in sample frames, when the zone sustains.
    pub loop_range: Option<(usize, usize)>,
}

impl Zone {
    /// Cubic Hermite read at a fractional frame position. Out-of-range reads give silence.
    pub fn read(&self, pos: f64) -> f32 {
        let i = pos.floor();
        if i < 0.0 {
            return 0.0;
        }
        let i = i as usize;
        let t = (pos - pos.floor()) as f32;
        let d = &self.data;
        let at = |k: isize| -> f32 {
            let mut idx = i as isize + k;
            if let Some((ls, le)) = self.loop_range {
                let span = (le - ls) as isize;
                if span > 0 && idx >= le as isize {
                    idx = ls as isize + (idx - ls as isize) % span;
                }
            }
            if idx < 0 {
                return 0.0;
            }
            d.get(idx as usize).copied().unwrap_or(0.0)
        };
        let (y0, y1, y2, y3) = (at(-1), at(0), at(1), at(2));
        let c0 = y1;
        let c1 = 0.5 * (y2 - y0);
        let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
        let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);
        ((c3 * t + c2) * t + c1) * t + c0
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// A set of zones with a display name.
#[derive(Debug, Clone, Default)]
pub struct SampleBank {
    pub name: String,
    pub zones: Vec<Zone>,
}

impl SampleBank {
    /// Pick the zone for a note and velocity (0..=1). Falls back to the zone nearest in key
    /// so notes outside the mapped range still sound.
    pub fn find(&self, note: u8, velocity: f32) -> Option<usize> {
        let v = (velocity.clamp(0.0, 1.0) * 127.0).round() as u8;
        let hit = self
            .zones
            .iter()
            .position(|z| note >= z.key_lo && note <= z.key_hi && v >= z.vel_lo && v <= z.vel_hi);
        if hit.is_some() {
            return hit;
        }
        self.zones
            .iter()
            .enumerate()
            .filter(|(_, z)| v >= z.vel_lo && v <= z.vel_hi)
            .min_by_key(|(_, z)| {
                if note < z.key_lo {
                    z.key_lo - note
                } else {
                    note.saturating_sub(z.key_hi)
                }
            })
            .map(|(i, _)| i)
            .or_else(|| (!self.zones.is_empty()).then_some(0))
    }

    pub fn total_frames(&self) -> usize {
        self.zones.iter().map(Zone::len).sum()
    }
}

fn err(msg: impl Into<String>) -> SynthError {
    SynthError::Io(msg.into())
}

fn u16_at(b: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(o..o + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(o..o + 4)?.try_into().ok()?))
}

/// Iterate RIFF sub-chunks in `b`, yielding `(id, body)`.
fn chunks(b: &[u8]) -> Vec<([u8; 4], &[u8])> {
    let mut out = Vec::new();
    let mut o = 0usize;
    while o + 8 <= b.len() {
        let Some(id) = b.get(o..o + 4).and_then(|s| <[u8; 4]>::try_from(s).ok()) else {
            break;
        };
        let size = u32_at(b, o + 4).unwrap_or(0) as usize;
        let end = (o + 8 + size).min(b.len());
        if let Some(body) = b.get(o + 8..end) {
            out.push((id, body));
        }
        o = o + 8 + size + (size & 1);
    }
    out
}

// ---------------------------------------------------------------- WAV

/// A decoded WAV file, mixed down to mono.
#[derive(Debug, Clone)]
pub struct Wav {
    pub data: Vec<f32>,
    pub sample_rate: u32,
    pub root_key: Option<u8>,
    pub loop_range: Option<(usize, usize)>,
}

/// Decode PCM (8/16/24/32-bit) and float WAV data. Reads root key and first loop from `smpl`.
pub fn decode_wav(bytes: &[u8]) -> Result<Wav, SynthError> {
    if bytes.get(0..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"WAVE") {
        return Err(err("not a RIFF/WAVE file"));
    }
    let mut fmt: Option<(u16, u16, u32, u16)> = None;
    let mut pcm: Option<&[u8]> = None;
    let mut root_key = None;
    let mut loop_range = None;
    for (id, body) in chunks(bytes.get(12..).unwrap_or(&[])) {
        match &id {
            b"fmt " => {
                let mut tag = u16_at(body, 0).unwrap_or(0);
                let ch = u16_at(body, 2).unwrap_or(1).max(1);
                let sr = u32_at(body, 4).unwrap_or(44100);
                let bits = u16_at(body, 14).unwrap_or(16);
                if tag == 0xFFFE {
                    tag = u16_at(body, 24).unwrap_or(1);
                }
                fmt = Some((tag, ch, sr, bits));
            }
            b"data" => pcm = Some(body),
            b"smpl" => {
                let unity = u32_at(body, 12).unwrap_or(60);
                root_key = u8::try_from(unity).ok().filter(|k| *k < 128);
                if u32_at(body, 28).unwrap_or(0) > 0 {
                    let s = u32_at(body, 44).unwrap_or(0) as usize;
                    let e = u32_at(body, 48).unwrap_or(0) as usize + 1;
                    if e > s {
                        loop_range = Some((s, e));
                    }
                }
            }
            _ => {}
        }
    }
    let (tag, ch, sr, bits) = fmt.ok_or_else(|| err("WAV has no fmt chunk"))?;
    let pcm = pcm.ok_or_else(|| err("WAV has no data chunk"))?;
    let ch = usize::from(ch);
    let bytes_per = usize::from(bits / 8).max(1);
    let frame = bytes_per * ch;
    let frames = pcm.len() / frame;
    let mut data = Vec::with_capacity(frames);
    for f in 0..frames {
        let mut acc = 0.0_f32;
        for c in 0..ch {
            let o = f * frame + c * bytes_per;
            let Some(s) = pcm.get(o..o + bytes_per) else {
                break;
            };
            let v = match (tag, bits) {
                (1, 8) => (f32::from(s[0]) - 128.0) / 128.0,
                (1, 16) => f32::from(i16::from_le_bytes([s[0], s[1]])) / 32768.0,
                (1, 24) => {
                    let v = i32::from_le_bytes([0, s[0], s[1], s[2]]) >> 8;
                    v as f32 / 8_388_608.0
                }
                (1, 32) => i32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f32 / 2_147_483_648.0,
                (3, 32) => f32::from_le_bytes([s[0], s[1], s[2], s[3]]),
                (3, 64) => {
                    f64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]) as f32
                }
                _ => {
                    return Err(err(format!(
                        "unsupported WAV format tag {tag} / {bits} bit"
                    )))
                }
            };
            acc += v;
        }
        data.push(acc / ch as f32);
    }
    Ok(Wav {
        data,
        sample_rate: sr,
        root_key,
        loop_range,
    })
}

fn read_wav(path: &Path) -> Result<Wav, SynthError> {
    let bytes = std::fs::read(path).map_err(|e| err(format!("{}: {e}", path.display())))?;
    decode_wav(&bytes).map_err(|e| err(format!("{}: {e}", path.display())))
}

/// Find a seamless sustain loop in a one-shot sample and bake an equal-power crossfade into
/// `data` so jumping from the loop end back to its start has no click. Returns `None` when the
/// sample is too short or no region repeats well enough (it is then played once, unlooped).
pub fn auto_loop(data: &mut [f32], sample_rate: f32) -> Option<(usize, usize)> {
    const WIN: usize = 512;
    let n = data.len();
    let sr = sample_rate.max(8000.0);
    if n < (1.2 * sr) as usize {
        return None;
    }
    let min_len = (0.4 * sr) as usize;
    let score = |d: &[f32], s: usize, e: usize| -> f32 {
        if s < WIN || e + WIN > d.len() {
            return -1.0;
        }
        let a = &d[s - WIN..s + WIN];
        let b = &d[e - WIN..e + WIN];
        let (mut ab, mut aa, mut bb) = (0.0_f32, 0.0_f32, 0.0_f32);
        for (x, y) in a.iter().zip(b) {
            ab += x * y;
            aa += x * x;
            bb += y * y;
        }
        if aa < 1e-9 || bb < 1e-9 {
            return -1.0;
        }
        let ratio = (aa.min(bb) / aa.max(bb)).sqrt();
        ab / (aa * bb).sqrt() * ratio
    };
    let mut best: Option<(f32, usize, usize)> = None;
    for frac in [0.88_f32, 0.80, 0.72] {
        let e = (n as f32 * frac) as usize;
        let lo = ((n as f32 * 0.2) as usize).max(4096);
        let hi = e.saturating_sub(min_len);
        let mut s = lo;
        while s < hi {
            let sc = score(data, s, e);
            if best.is_none_or(|(b, _, _)| sc > b) {
                best = Some((sc, s, e));
            }
            s += 32;
        }
    }
    let (_, s0, e) = best?;
    let mut best_s = (score(data, s0, e), s0);
    for s in s0.saturating_sub(32)..(s0 + 32).min(e.saturating_sub(min_len)) {
        let sc = score(data, s, e);
        if sc > best_s.0 {
            best_s = (sc, s);
        }
    }
    if best_s.0 < 0.6 {
        return None;
    }
    let s = best_s.1;
    let xf = 4096.min(s).min((e - s) / 4);
    for i in 0..xf {
        let t = (i as f32 + 0.5) / xf as f32;
        let (g_out, g_in) = (
            (t * std::f32::consts::FRAC_PI_2).cos(),
            (t * std::f32::consts::FRAC_PI_2).sin(),
        );
        data[e - xf + i] = data[e - xf + i] * g_out + data[s - xf + i] * g_in;
    }
    Some((s, e))
}

/// Sample data with a baked loop crossfade, plus its loop range.
type LoopedSample = (Arc<[f32]>, Option<(usize, usize)>);

fn wav_zone(w: Wav, key_lo: u8, key_hi: u8, root: u8) -> Zone {
    Zone {
        sample_rate: w.sample_rate as f32,
        data: w.data.into(),
        key_lo,
        key_hi,
        vel_lo: 0,
        vel_hi: 127,
        root_key: root,
        tune_cents: 0.0,
        gain: 1.0,
        loop_range: w.loop_range,
    }
}

// ---------------------------------------------------------------- notes

/// Parse `60` or a note name such as `c4`, `F#2`, `db3` (middle C = c4 = 60).
pub fn parse_note(s: &str) -> Option<u8> {
    let s = s.trim();
    if let Ok(n) = s.parse::<i32>() {
        return u8::try_from(n).ok().filter(|v| *v < 128);
    }
    let mut chars = s.chars();
    let base = match chars.next()?.to_ascii_lowercase() {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => return None,
    };
    let rest: String = chars.collect();
    let (acc, oct) = if let Some(r) = rest.strip_prefix('#') {
        (1, r)
    } else if let Some(r) = rest.strip_prefix('b') {
        (-1, r)
    } else {
        (0, rest.as_str())
    };
    let oct: i32 = oct.parse().ok()?;
    u8::try_from((oct + 1) * 12 + base + acc)
        .ok()
        .filter(|v| *v < 128)
}

// ---------------------------------------------------------------- SFZ

fn sfz_words(text: &str) -> Vec<(String, String)> {
    // Returns (opcode-or-header, value) pairs; headers are stored as ("<region>", "").
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        let line = line.split("//").next().unwrap_or("");
        for raw in line.split_whitespace() {
            let mut word = raw;
            while !word.is_empty() {
                if word.starts_with('#') {
                    break;
                }
                if let Some(rest) = word.strip_prefix('<') {
                    if let Some(end) = rest.find('>') {
                        out.push((format!("<{}>", &rest[..end]), String::new()));
                        word = &rest[end + 1..];
                        continue;
                    }
                    break;
                }
                if let Some(eq) = word.find('=') {
                    let name = &word[..eq];
                    if !name.is_empty()
                        && name
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
                    {
                        out.push((name.to_ascii_lowercase(), word[eq + 1..].to_string()));
                        break;
                    }
                }
                if let Some(last) = out.last_mut() {
                    if !last.0.starts_with('<') {
                        last.1.push(' ');
                        last.1.push_str(word);
                    }
                }
                break;
            }
        }
    }
    out
}

fn sfz_f(ops: &HashMap<String, String>, k: &str) -> Option<f32> {
    ops.get(k).and_then(|v| v.trim().parse().ok())
}

fn sfz_key(ops: &HashMap<String, String>, k: &str) -> Option<u8> {
    ops.get(k).and_then(|v| parse_note(v))
}

/// Load an SFZ instrument (`sample`, key/vel ranges, `pitch_keycenter`, `tune`, `transpose`,
/// `volume`, `loop_mode`, `loop_start`, `loop_end`, `<global>/<master>/<group>/<region>`).
pub fn load_sfz(path: &Path) -> Result<SampleBank, SynthError> {
    let text =
        std::fs::read_to_string(path).map_err(|e| err(format!("{}: {e}", path.display())))?;
    let dir = path.parent().unwrap_or(Path::new("."));
    let mut global = HashMap::new();
    let mut master = HashMap::new();
    let mut group = HashMap::new();
    let mut region: Option<HashMap<String, String>> = None;
    let mut regions: Vec<HashMap<String, String>> = Vec::new();
    let mut default_path = String::new();
    let mut scope = String::new();
    let mut flush = |region: &mut Option<HashMap<String, String>>,
                     global: &HashMap<String, String>,
                     master: &HashMap<String, String>,
                     group: &HashMap<String, String>| {
        if let Some(r) = region.take() {
            let mut m = global.clone();
            m.extend(master.clone());
            m.extend(group.clone());
            m.extend(r);
            regions.push(m);
        }
    };
    for (k, v) in sfz_words(&text) {
        if k.starts_with('<') {
            flush(&mut region, &global, &master, &group);
            scope = k.clone();
            match k.as_str() {
                "<global>" => {
                    global.clear();
                    master.clear();
                    group.clear();
                }
                "<master>" => {
                    master.clear();
                    group.clear();
                }
                "<group>" => group.clear(),
                "<region>" => region = Some(HashMap::new()),
                _ => {}
            }
            continue;
        }
        match scope.as_str() {
            "<control>" => {
                if k == "default_path" {
                    default_path = v.replace('\\', "/");
                }
            }
            "<global>" => {
                global.insert(k, v);
            }
            "<master>" => {
                master.insert(k, v);
            }
            "<group>" => {
                group.insert(k, v);
            }
            "<region>" => {
                if let Some(r) = region.as_mut() {
                    r.insert(k, v);
                }
            }
            _ => {}
        }
    }
    flush(&mut region, &global, &master, &group);

    let mut cache: HashMap<PathBuf, (Arc<[f32]>, Wav)> = HashMap::new();
    let mut looped: HashMap<PathBuf, LoopedSample> = HashMap::new();
    let mut zones = Vec::new();
    for ops in &regions {
        let Some(sample) = ops.get("sample") else {
            continue;
        };
        if sample.starts_with('*') {
            continue;
        }
        let rel = format!("{}{}", default_path, sample.replace('\\', "/"));
        let full = dir.join(rel);
        if !cache.contains_key(&full) {
            let mut w = read_wav(&full)?;
            let arc: Arc<[f32]> = std::mem::take(&mut w.data).into();
            cache.insert(full.clone(), (arc, w));
        }
        let Some((arc, meta)) = cache.get(&full) else {
            continue;
        };
        let key = sfz_key(ops, "key");
        let lo = key.or_else(|| sfz_key(ops, "lokey")).unwrap_or(0);
        let hi = key.or_else(|| sfz_key(ops, "hikey")).unwrap_or(127);
        let root = sfz_key(ops, "pitch_keycenter")
            .or(key)
            .or(meta.root_key)
            .unwrap_or(60);
        let mode = ops.get("loop_mode").map(|s| s.trim().to_ascii_lowercase());
        let looping = match mode.as_deref() {
            Some("loop_continuous" | "loop_sustain") => true,
            Some(_) => false,
            None => meta.loop_range.is_some(),
        };
        let mut data = arc.clone();
        let loop_range = if looping {
            let s = sfz_f(ops, "loop_start").map(|v| v as usize);
            let e = sfz_f(ops, "loop_end").map(|v| v as usize + 1);
            match (s, e, meta.loop_range) {
                (Some(s), Some(e), _) if e > s => Some((s, e)),
                (_, _, Some(r)) => Some(r),
                _ => {
                    if let Some((d, r)) = looped.get(&full) {
                        data = d.clone();
                        *r
                    } else {
                        let mut v = arc.to_vec();
                        let r = auto_loop(&mut v, meta.sample_rate as f32).or(Some((0, v.len())));
                        let d: Arc<[f32]> = v.into();
                        looped.insert(full.clone(), (d.clone(), r));
                        data = d;
                        r
                    }
                }
            }
        } else {
            None
        };
        let vol_db = sfz_f(ops, "volume").unwrap_or(0.0);
        zones.push(Zone {
            data,
            sample_rate: meta.sample_rate as f32,
            key_lo: lo.min(hi),
            key_hi: hi.max(lo),
            vel_lo: sfz_f(ops, "lovel").map_or(0, |v| v.clamp(0.0, 127.0) as u8),
            vel_hi: sfz_f(ops, "hivel").map_or(127, |v| v.clamp(0.0, 127.0) as u8),
            root_key: root,
            tune_cents: sfz_f(ops, "tune").unwrap_or(0.0)
                + sfz_f(ops, "transpose").unwrap_or(0.0) * 100.0,
            gain: 10.0_f32.powf(vol_db / 20.0),
            loop_range,
        });
    }
    if zones.is_empty() {
        return Err(err(format!("{}: no playable regions", path.display())));
    }
    Ok(SampleBank {
        name: file_stem(path),
        zones,
    })
}

// ---------------------------------------------------------------- SF2

#[derive(Debug, Clone, Copy)]
struct Gen {
    oper: u16,
    lo: u8,
    hi: u8,
    amt: i16,
}

fn parse_gens(b: &[u8]) -> Vec<Gen> {
    b.as_chunks::<4>()
        .0
        .iter()
        .map(|c| Gen {
            oper: u16::from_le_bytes([c[0], c[1]]),
            lo: c[2],
            hi: c[3],
            amt: i16::from_le_bytes([c[2], c[3]]),
        })
        .collect()
}

fn parse_bags(b: &[u8]) -> Vec<usize> {
    b.as_chunks::<4>()
        .0
        .iter()
        .map(|c| usize::from(u16::from_le_bytes([c[0], c[1]])))
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct Acc {
    key: (u8, u8),
    vel: (u8, u8),
    coarse: i32,
    fine: i32,
    atten: i32,
    root: Option<i32>,
    modes: i32,
}

impl Default for Acc {
    fn default() -> Self {
        Self {
            key: (0, 127),
            vel: (0, 127),
            coarse: 0,
            fine: 0,
            atten: 0,
            root: None,
            modes: 0,
        }
    }
}

fn apply_gens(acc: &mut Acc, gens: &[Gen]) {
    for g in gens {
        match g.oper {
            43 => acc.key = (g.lo, g.hi),
            44 => acc.vel = (g.lo, g.hi),
            48 => acc.atten = i32::from(g.amt),
            51 => acc.coarse = i32::from(g.amt),
            52 => acc.fine = i32::from(g.amt),
            54 => acc.modes = i32::from(g.amt),
            58 => acc.root = Some(i32::from(g.amt)),
            _ => {}
        }
    }
}

fn zone_slices<'a>(gens: &'a [Gen], bags: &[usize], first: usize, last: usize) -> Vec<&'a [Gen]> {
    (first..last)
        .filter_map(|i| {
            let a = *bags.get(i)?;
            let b = *bags.get(i + 1)?;
            gens.get(a..b)
        })
        .collect()
}

fn sf2_chunks(bytes: &[u8]) -> Result<HashMap<[u8; 4], &[u8]>, SynthError> {
    if bytes.get(0..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"sfbk") {
        return Err(err("not an SF2 file"));
    }
    let mut map = HashMap::new();
    for (id, body) in chunks(bytes.get(12..).unwrap_or(&[])) {
        if &id == b"LIST" {
            for (sid, sbody) in chunks(body.get(4..).unwrap_or(&[])) {
                map.insert(sid, sbody);
            }
        }
    }
    Ok(map)
}

/// Names of the presets in an SF2 file, in index order.
pub fn list_sf2_presets(path: &Path) -> Result<Vec<String>, SynthError> {
    let bytes = std::fs::read(path).map_err(|e| err(format!("{}: {e}", path.display())))?;
    let map = sf2_chunks(&bytes)?;
    let phdr = map.get(b"phdr").ok_or_else(|| err("SF2 has no phdr"))?;
    let n = (phdr.len() / 38).saturating_sub(1);
    Ok((0..n)
        .map(|i| {
            let raw = phdr.get(i * 38..i * 38 + 20).unwrap_or(&[]);
            let end = raw.iter().position(|b| *b == 0).unwrap_or(raw.len());
            String::from_utf8_lossy(raw.get(..end).unwrap_or(&[]))
                .trim()
                .to_string()
        })
        .collect())
}

/// Load one preset of an SF2 file (`None` = first melodic preset). Stereo pairs are reduced
/// to their left channel. ROM samples are skipped.
pub fn load_sf2(path: &Path, preset: Option<usize>) -> Result<SampleBank, SynthError> {
    let bytes = std::fs::read(path).map_err(|e| err(format!("{}: {e}", path.display())))?;
    let map = sf2_chunks(&bytes)?;
    let need = |k: &[u8; 4]| -> Result<&[u8], SynthError> {
        map.get(k)
            .copied()
            .ok_or_else(|| err(format!("SF2 missing {} chunk", String::from_utf8_lossy(k))))
    };
    let smpl = need(b"smpl")?;
    let phdr = need(b"phdr")?;
    let pbag = parse_bags(need(b"pbag")?);
    let pgen = parse_gens(need(b"pgen")?);
    let inst = need(b"inst")?;
    let ibag = parse_bags(need(b"ibag")?);
    let igen = parse_gens(need(b"igen")?);
    let shdr = need(b"shdr")?;

    let n_presets = (phdr.len() / 38).saturating_sub(1);
    let pi = match preset {
        Some(p) => p,
        None => (0..n_presets)
            .find(|i| u16_at(phdr, i * 38 + 22) == Some(0))
            .unwrap_or(0),
    };
    if pi >= n_presets {
        return Err(err(format!(
            "SF2 preset {pi} out of range (0..{n_presets})"
        )));
    }
    let raw_name = phdr.get(pi * 38..pi * 38 + 20).unwrap_or(&[]);
    let nend = raw_name
        .iter()
        .position(|b| *b == 0)
        .unwrap_or(raw_name.len());
    let name = String::from_utf8_lossy(raw_name.get(..nend).unwrap_or(&[]))
        .trim()
        .to_string();
    let pb0 = usize::from(u16_at(phdr, pi * 38 + 24).unwrap_or(0));
    let pb1 = usize::from(u16_at(phdr, (pi + 1) * 38 + 24).unwrap_or(0));

    let mut decoded: HashMap<usize, Arc<[f32]>> = HashMap::new();
    let mut zones = Vec::new();
    let mut pglobal = Acc::default();
    for (zi, pz) in zone_slices(&pgen, &pbag, pb0, pb1).into_iter().enumerate() {
        let inst_id = pz
            .iter()
            .rev()
            .find(|g| g.oper == 41)
            .map(|g| g.amt as usize);
        let mut pacc = pglobal;
        apply_gens(&mut pacc, pz);
        let Some(ii) = inst_id else {
            if zi == 0 {
                pglobal = pacc;
            }
            continue;
        };
        let ib0 = usize::from(u16_at(inst, ii * 22 + 20).unwrap_or(0));
        let ib1 = usize::from(u16_at(inst, (ii + 1) * 22 + 20).unwrap_or(0));
        let mut iglobal = Acc::default();
        for (izi, iz) in zone_slices(&igen, &ibag, ib0, ib1).into_iter().enumerate() {
            let sid = iz
                .iter()
                .rev()
                .find(|g| g.oper == 53)
                .map(|g| g.amt as usize);
            let mut iacc = iglobal;
            apply_gens(&mut iacc, iz);
            let Some(sid) = sid else {
                if izi == 0 {
                    iglobal = iacc;
                }
                continue;
            };
            let key = (iacc.key.0.max(pacc.key.0), iacc.key.1.min(pacc.key.1));
            let vel = (iacc.vel.0.max(pacc.vel.0), iacc.vel.1.min(pacc.vel.1));
            if key.0 > key.1 || vel.0 > vel.1 {
                continue;
            }
            let h = sid * 46;
            let stype = u16_at(shdr, h + 44).unwrap_or(0);
            if stype & 0x8000 != 0 || stype == 4 {
                continue;
            }
            let start = u32_at(shdr, h + 20).unwrap_or(0) as usize;
            let end = u32_at(shdr, h + 24).unwrap_or(0) as usize;
            let ls = u32_at(shdr, h + 28).unwrap_or(0) as usize;
            let le = u32_at(shdr, h + 32).unwrap_or(0) as usize;
            let sr = u32_at(shdr, h + 36).unwrap_or(44100);
            let orig = shdr.get(h + 40).copied().unwrap_or(60);
            let corr = shdr.get(h + 41).map_or(0, |b| i32::from(*b as i8));
            if end <= start || end * 2 > smpl.len() {
                continue;
            }
            let data = if let Some(d) = decoded.get(&sid) {
                d.clone()
            } else {
                let seg = smpl.get(start * 2..end * 2).unwrap_or(&[]);
                let v: Arc<[f32]> = seg
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| f32::from(i16::from_le_bytes([c[0], c[1]])) / 32768.0)
                    .collect::<Vec<_>>()
                    .into();
                decoded.insert(sid, v.clone());
                v
            };
            let modes = iacc.modes;
            let loop_range = (modes & 3 == 1 || modes & 3 == 3)
                .then(|| (ls.saturating_sub(start), le.saturating_sub(start)))
                .filter(|(a, b)| b > a && *b <= data.len());
            let root = iacc
                .root
                .filter(|r| (0..128).contains(r))
                .unwrap_or(if orig == 255 { 60 } else { i32::from(orig) });
            let atten = (iacc.atten + pacc.atten).clamp(0, 1440);
            zones.push(Zone {
                data,
                sample_rate: sr as f32,
                key_lo: key.0,
                key_hi: key.1,
                vel_lo: vel.0,
                vel_hi: vel.1,
                root_key: root as u8,
                tune_cents: ((iacc.coarse + pacc.coarse) * 100 + iacc.fine + pacc.fine + corr)
                    as f32,
                gain: 10.0_f32.powf(-(atten as f32) / 200.0),
                loop_range,
            });
        }
    }
    if zones.is_empty() {
        return Err(err(format!(
            "SF2 preset {pi} ({name}) has no playable zones"
        )));
    }
    Ok(SampleBank { name, zones })
}

// ---------------------------------------------------------------- folder + dispatch

fn file_stem(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Find a note name token at the end of a file stem (`pad_C3`, `strings-f#2`, `piano 60`).
fn note_from_name(stem: &str) -> Option<u8> {
    stem.rsplit(['_', '-', ' ', '.'])
        .find_map(|tok| (!tok.is_empty()).then(|| parse_note(tok)).flatten())
}

fn load_wav_folder(dir: &Path) -> Result<SampleBank, SynthError> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| err(format!("{}: {e}", dir.display())))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("wav")))
        .collect();
    files.sort();
    let mut items: Vec<(u8, Wav)> = Vec::new();
    for f in &files {
        let w = read_wav(f)?;
        let root = note_from_name(&file_stem(f)).or(w.root_key).unwrap_or(60);
        items.push((root, w));
    }
    items.sort_by_key(|(k, _)| *k);
    if items.is_empty() {
        return Err(err(format!("{}: no .wav files", dir.display())));
    }
    let roots: Vec<u8> = items.iter().map(|(k, _)| *k).collect();
    let mut zones = Vec::new();
    for (i, (root, w)) in items.into_iter().enumerate() {
        let lo = if i == 0 {
            0
        } else {
            ((u16::from(roots[i - 1]) + u16::from(root)) / 2 + 1) as u8
        };
        let hi = if i + 1 == roots.len() {
            127
        } else {
            ((u16::from(root) + u16::from(roots[i + 1])) / 2) as u8
        };
        zones.push(wav_zone(w, lo, hi, root));
    }
    Ok(SampleBank {
        name: file_stem(dir),
        zones,
    })
}

/// Load a bank from a spec: `.sf2` (optionally `#preset`), `.sfz`, `.wav`, or a folder of WAVs.
pub fn load_bank(spec: &str) -> Result<SampleBank, SynthError> {
    let (path_s, preset) = match spec.rsplit_once('#') {
        Some((p, n)) if n.trim().parse::<usize>().is_ok() => (p, n.trim().parse::<usize>().ok()),
        _ => (spec, None),
    };
    let path = Path::new(path_s);
    if path.is_dir() {
        return load_wav_folder(path);
    }
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "sf2" => load_sf2(path, preset),
        "sfz" => load_sfz(path),
        "wav" => {
            let w = read_wav(path)?;
            let root = w.root_key.unwrap_or(60);
            Ok(SampleBank {
                name: file_stem(path),
                zones: vec![wav_zone(w, 0, 127, root)],
            })
        }
        other => Err(err(format!(
            "unsupported sample bank type '.{other}' (use .sf2, .sfz, .wav or a folder of wavs)"
        ))),
    }
}

type BankCache = Mutex<HashMap<String, Arc<SampleBank>>>;

fn cache() -> &'static BankCache {
    static CACHE: OnceLock<BankCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Load a bank once and share it. Safe to call from any thread except the audio thread.
pub fn load_cached(spec: &str) -> Result<Arc<SampleBank>, SynthError> {
    if let Ok(map) = cache().lock() {
        if let Some(b) = map.get(spec) {
            return Ok(b.clone());
        }
    }
    let bank = Arc::new(load_bank(spec)?);
    if let Ok(mut map) = cache().lock() {
        map.insert(spec.to_string(), bank.clone());
    }
    Ok(bank)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav_bytes(frames: &[i16], sr: u32, root: Option<u8>, lp: Option<(u32, u32)>) -> Vec<u8> {
        let mut data = Vec::new();
        for f in frames {
            data.extend_from_slice(&f.to_le_bytes());
        }
        let mut fmt = Vec::new();
        fmt.extend_from_slice(&1u16.to_le_bytes());
        fmt.extend_from_slice(&1u16.to_le_bytes());
        fmt.extend_from_slice(&sr.to_le_bytes());
        fmt.extend_from_slice(&(sr * 2).to_le_bytes());
        fmt.extend_from_slice(&2u16.to_le_bytes());
        fmt.extend_from_slice(&16u16.to_le_bytes());
        let mut body = Vec::new();
        body.extend_from_slice(b"WAVE");
        let mut push = |id: &[u8; 4], c: &[u8]| {
            body.extend_from_slice(id);
            body.extend_from_slice(&(c.len() as u32).to_le_bytes());
            body.extend_from_slice(c);
            if c.len() % 2 == 1 {
                body.push(0);
            }
        };
        push(b"fmt ", &fmt);
        if let Some(r) = root {
            let mut s = vec![0u8; 36];
            s[12..16].copy_from_slice(&u32::from(r).to_le_bytes());
            if let Some((a, b)) = lp {
                s[28..32].copy_from_slice(&1u32.to_le_bytes());
                s.extend_from_slice(&0u32.to_le_bytes());
                s.extend_from_slice(&0u32.to_le_bytes());
                s.extend_from_slice(&a.to_le_bytes());
                s.extend_from_slice(&b.to_le_bytes());
                s.extend_from_slice(&0u32.to_le_bytes());
                s.extend_from_slice(&0u32.to_le_bytes());
            }
            push(b"smpl", &s);
        }
        push(b"data", &data);
        let mut out = b"RIFF".to_vec();
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&body);
        out
    }

    fn sine(n: usize, cycles: f32) -> Vec<i16> {
        (0..n)
            .map(|i| {
                ((i as f32 / n as f32 * cycles * std::f32::consts::TAU).sin() * 20000.0) as i16
            })
            .collect()
    }

    #[test]
    fn wav_decodes_with_root_and_loop() {
        let w = decode_wav(&wav_bytes(
            &sine(400, 4.0),
            22050,
            Some(57),
            Some((100, 299)),
        ))
        .unwrap_or_else(|_| Wav {
            data: vec![],
            sample_rate: 0,
            root_key: None,
            loop_range: None,
        });
        assert_eq!(w.data.len(), 400);
        assert_eq!(w.sample_rate, 22050);
        assert_eq!(w.root_key, Some(57));
        assert_eq!(w.loop_range, Some((100, 300)));
        assert!(w.data.iter().any(|v| v.abs() > 0.5));
    }

    #[test]
    fn note_names_parse() {
        assert_eq!(parse_note("c4"), Some(60));
        assert_eq!(parse_note("A3"), Some(57));
        assert_eq!(parse_note("f#2"), Some(42));
        assert_eq!(parse_note("db3"), Some(49));
        assert_eq!(parse_note("72"), Some(72));
        assert_eq!(parse_note("zz"), None);
        assert_eq!(note_from_name("pad_C3"), Some(48));
    }

    #[test]
    fn hermite_reads_loop_and_interpolate() {
        let z = Zone {
            data: vec![0.0, 1.0, 0.0, -1.0].into(),
            sample_rate: 4.0,
            key_lo: 0,
            key_hi: 127,
            vel_lo: 0,
            vel_hi: 127,
            root_key: 60,
            tune_cents: 0.0,
            gain: 1.0,
            loop_range: Some((0, 4)),
        };
        assert!((z.read(1.0) - 1.0).abs() < 1e-6);
        assert!((z.read(5.0) - 1.0).abs() < 1e-6, "loop wraps");
        assert!(z.read(0.5) > 0.3 && z.read(0.5) < 0.9);
    }

    #[test]
    fn sfz_maps_regions_with_spaces_and_inheritance() {
        let dir = std::env::temp_dir().join(format!("sc_sfz_{}", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join("my samples"));
        let _ = std::fs::write(
            dir.join("my samples").join("a one.wav"),
            wav_bytes(&sine(300, 3.0), 32000, None, None),
        );
        let _ = std::fs::write(
            dir.join("my samples").join("b.wav"),
            wav_bytes(&sine(300, 6.0), 32000, Some(72), Some((10, 200))),
        );
        let sfz = "<control> default_path=my samples/\n<group> volume=-6 loop_mode=loop_continuous\n<region> sample=a one.wav lokey=c2 hikey=b3 pitch_keycenter=c3 // comment\n<region> sample=b.wav lokey=c4 hikey=c6 lovel=64 tune=12\n";
        let p = dir.join("t.sfz");
        let _ = std::fs::write(&p, sfz);
        let bank = load_sfz(&p).unwrap_or_default();
        assert_eq!(bank.zones.len(), 2);
        let a = &bank.zones[0];
        assert_eq!((a.key_lo, a.key_hi, a.root_key), (36, 59, 48));
        assert!(a.loop_range.is_some());
        assert!((a.gain - 0.501).abs() < 0.01);
        let b = &bank.zones[1];
        assert_eq!((b.vel_lo, b.vel_hi, b.root_key), (64, 127, 72));
        assert!((b.tune_cents - 12.0).abs() < 1e-3);
        assert_eq!(bank.find(40, 0.9), Some(0));
        assert_eq!(bank.find(70, 0.9), Some(1));
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn rec(chunks: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
        let mut out = Vec::new();
        for (id, c) in chunks {
            out.extend_from_slice(*id);
            out.extend_from_slice(&(c.len() as u32).to_le_bytes());
            out.extend_from_slice(c);
        }
        out
    }

    fn gen(oper: u16, lo: u8, hi: u8) -> Vec<u8> {
        let mut v = oper.to_le_bytes().to_vec();
        v.push(lo);
        v.push(hi);
        v
    }

    fn bag(g: u16) -> Vec<u8> {
        let mut v = g.to_le_bytes().to_vec();
        v.extend_from_slice(&0u16.to_le_bytes());
        v
    }

    #[test]
    fn sf2_flattens_preset_instrument_sample() {
        let frames = sine(200, 2.0);
        let mut smpl = Vec::new();
        for f in &frames {
            smpl.extend_from_slice(&f.to_le_bytes());
        }
        smpl.extend(vec![0u8; 92]);
        let mut phdr = Vec::new();
        for (name, bag_ndx) in [("Test Pad", 0u16), ("EOP", 1u16)] {
            let mut n = name.as_bytes().to_vec();
            n.resize(20, 0);
            phdr.extend(n);
            phdr.extend_from_slice(&0u16.to_le_bytes());
            phdr.extend_from_slice(&0u16.to_le_bytes());
            phdr.extend_from_slice(&bag_ndx.to_le_bytes());
            phdr.extend_from_slice(&[0u8; 12]);
        }
        let pbag = [bag(0), bag(1)].concat();
        let pgen = [gen(41, 0, 0), gen(0, 0, 0)].concat();
        let mut inst = Vec::new();
        for (name, b) in [("Inst", 0u16), ("EOI", 1u16)] {
            let mut n = name.as_bytes().to_vec();
            n.resize(20, 0);
            inst.extend(n);
            inst.extend_from_slice(&b.to_le_bytes());
        }
        let ibag = [bag(0), bag(3)].concat();
        let igen = [gen(43, 36, 84), gen(54, 1, 0), gen(53, 0, 0), gen(0, 0, 0)].concat();
        let mut shdr = Vec::new();
        for (nm, s, e, ls, le, sr, orig) in [
            ("smp", 0u32, 200u32, 20u32, 180u32, 32000u32, 57u8),
            ("EOS", 0, 0, 0, 0, 0, 0),
        ] {
            let mut n = nm.as_bytes().to_vec();
            n.resize(20, 0);
            shdr.extend(n);
            for v in [s, e, ls, le, sr] {
                shdr.extend_from_slice(&v.to_le_bytes());
            }
            shdr.push(orig);
            shdr.push(0);
            shdr.extend_from_slice(&0u16.to_le_bytes());
            shdr.extend_from_slice(&1u16.to_le_bytes());
        }
        let sdta = [b"sdta".to_vec(), rec(&[(b"smpl", smpl)])].concat();
        let pdta = [
            b"pdta".to_vec(),
            rec(&[
                (b"phdr", phdr),
                (b"pbag", pbag),
                (b"pmod", vec![0; 10]),
                (b"pgen", pgen),
                (b"inst", inst),
                (b"ibag", ibag),
                (b"imod", vec![0; 10]),
                (b"igen", igen),
                (b"shdr", shdr),
            ]),
        ]
        .concat();
        let body = [b"sfbk".to_vec(), rec(&[(b"LIST", sdta), (b"LIST", pdta)])].concat();
        let mut file = b"RIFF".to_vec();
        file.extend_from_slice(&(body.len() as u32).to_le_bytes());
        file.extend(body);
        let p = std::env::temp_dir().join(format!("sc_test_{}.sf2", std::process::id()));
        let _ = std::fs::write(&p, &file);
        let names = list_sf2_presets(&p).unwrap_or_default();
        assert_eq!(names, vec!["Test Pad".to_string()]);
        let bank = load_sf2(&p, None).unwrap_or_default();
        assert_eq!(bank.zones.len(), 1);
        let z = &bank.zones[0];
        assert_eq!((z.key_lo, z.key_hi, z.root_key), (36, 84, 57));
        assert_eq!(z.loop_range, Some((20, 180)));
        assert_eq!(z.len(), 200);
        assert!((z.sample_rate - 32000.0).abs() < 1.0);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn auto_loop_finds_a_clickless_sustain_loop() {
        let sr = 48_000.0_f32;
        let n = (3.0 * sr) as usize;
        let mut d: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / sr;
                let env = 0.3 + 0.7 * (-t * 0.4).exp();
                env * ((std::f32::consts::TAU * 220.0 * t).sin()
                    + 0.4 * (std::f32::consts::TAU * 660.0 * t).sin())
            })
            .collect();
        let r = auto_loop(&mut d, sr);
        let (s, e) = r.unwrap_or((0, 0));
        assert!(e > s && e - s >= (0.4 * sr) as usize, "loop {r:?}");
        assert!(
            (d[e - 1] - d[s - 1]).abs() < 0.08,
            "splice click {}",
            (d[e - 1] - d[s - 1]).abs()
        );
        let mut short = vec![0.1_f32; 1000];
        assert!(auto_loop(&mut short, sr).is_none());
    }
}
