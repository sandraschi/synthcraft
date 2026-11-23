//! Free sample pack catalogue and installer (used by `fetch-bank` and the app).
//!
//! Download free sample packs listed in `banks/manifest.json`.
//!
//! The repository ships only the manifest (URLs pinned to a commit, SHA-256 per file, licence).
//! Every downloaded file is verified before it is kept, and an SFZ plus an attribution note are
//! written next to the samples so the sampler can load the bank by its `.sfz` path.

use crate::sampler::parse_note;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const MANIFEST: &str = include_str!("../../../banks/manifest.json");

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub banks: Vec<Bank>,
}

#[derive(Debug, Deserialize)]
pub struct Bank {
    pub id: String,
    pub name: String,
    pub license: String,
    pub author: String,
    pub repo: String,
    pub commit: String,
    pub files: Vec<BankFile>,
}

#[derive(Debug, Deserialize)]
pub struct BankFile {
    pub path: String,
    pub note: String,
    pub bytes: u64,
    pub sha256: String,
}

pub fn manifest() -> Result<Manifest, String> {
    serde_json::from_str(MANIFEST).map_err(|e| format!("bank manifest is invalid: {e}"))
}

fn local_name(f: &BankFile) -> String {
    let leaf = f.path.rsplit('/').next().unwrap_or(&f.path);
    leaf.replace('#', "s").replace(' ', "_")
}

fn raw_url(b: &Bank, f: &BankFile) -> String {
    let enc: Vec<String> = f
        .path
        .split('/')
        .map(|seg| seg.replace(' ', "%20").replace('#', "%23"))
        .collect();
    format!(
        "https://raw.githubusercontent.com/{}/{}/{}",
        b.repo,
        b.commit,
        enc.join("/")
    )
}

/// Build the SFZ text: each sample covers the keys halfway to its neighbours.
pub fn sfz_for(b: &Bank) -> Result<String, String> {
    let mut roots: Vec<(u8, &BankFile)> = Vec::new();
    for f in &b.files {
        let n = parse_note(&f.note).ok_or_else(|| format!("bad note {:?} in {}", f.note, b.id))?;
        roots.push((n, f));
    }
    roots.sort_by_key(|(n, _)| *n);
    let mut out = format!(
        "// {} ({}), {}\n<control>\n<group> loop_mode=loop_continuous\n",
        b.name, b.license, b.author
    );
    for (i, (root, f)) in roots.iter().enumerate() {
        let lo = if i == 0 {
            0
        } else {
            ((u16::from(roots[i - 1].0) + u16::from(*root)) / 2 + 1) as u8
        };
        let hi = match roots.get(i + 1) {
            None => 127,
            Some((next, _)) => ((u16::from(*root) + u16::from(*next)) / 2) as u8,
        };
        out.push_str(&format!(
            "<region> sample={} lokey={} hikey={} pitch_keycenter={}\n",
            local_name(f),
            lo,
            hi,
            root
        ));
    }
    Ok(out)
}

fn sha_hex(data: &[u8]) -> String {
    let d = Sha256::digest(data);
    d.iter().map(|b| format!("{b:02x}")).collect()
}

/// Download (or reuse) every file of `b` below `dest/<id>`, verify hashes, write SFZ and notice.
pub fn fetch_bank(
    b: &Bank,
    dest: &Path,
    download: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    log: &mut dyn FnMut(&str),
) -> Result<PathBuf, String> {
    let dir = dest.join(&b.id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    for f in &b.files {
        let target = dir.join(local_name(f));
        if let Ok(existing) = std::fs::read(&target) {
            if sha_hex(&existing) == f.sha256 {
                log(&format!("ok (cached) {}", target.display()));
                continue;
            }
        }
        let data = download(&raw_url(b, f))?;
        let got = sha_hex(&data);
        if got != f.sha256 {
            return Err(format!(
                "checksum mismatch for {}: expected {}, got {got}. Nothing was written for this file.",
                f.path, f.sha256
            ));
        }
        std::fs::write(&target, &data)
            .map_err(|e| format!("cannot write {}: {e}", target.display()))?;
        log(&format!("fetched {} ({} bytes)", target.display(), f.bytes));
    }
    let sfz_path = dir.join(format!("{}.sfz", b.id));
    std::fs::write(&sfz_path, sfz_for(b)?)
        .map_err(|e| format!("cannot write {}: {e}", sfz_path.display()))?;
    let notice = format!(
        "{}\nLicence: {}\nAuthor: {}\nSource: https://github.com/{} at commit {}\n",
        b.name, b.license, b.author, b.repo, b.commit
    );
    std::fs::write(dir.join("ATTRIBUTION.txt"), notice)
        .map_err(|e| format!("cannot write attribution: {e}"))?;
    Ok(sfz_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bank() -> Bank {
        let mk = |p: &str, n: &str, data: &[u8]| BankFile {
            path: p.into(),
            note: n.into(),
            bytes: data.len() as u64,
            sha256: sha_hex(data),
        };
        Bank {
            id: "t".into(),
            name: "Test".into(),
            license: "CC0-1.0".into(),
            author: "a".into(),
            repo: "o/r".into(),
            commit: "c".into(),
            files: vec![
                mk("S/hi C4.wav", "C4", b"two"),
                mk("S/lo F#3.wav", "F#3", b"one"),
            ],
        }
    }

    #[test]
    fn shipped_manifest_parses_and_notes_are_valid() {
        let m = manifest().unwrap_or_else(|e| unreachable_msg(&e));
        assert!(!m.banks.is_empty());
        for b in &m.banks {
            assert!(sfz_for(b).is_ok(), "{}", b.id);
            assert!(b.files.iter().all(|f| f.sha256.len() == 64));
        }
    }

    fn unreachable_msg(e: &str) -> Manifest {
        assert!(e.is_empty(), "{e}");
        Manifest { banks: vec![] }
    }

    #[test]
    fn sfz_ranges_tile_the_keyboard_without_gaps() {
        let sfz = sfz_for(&sample_bank()).unwrap_or_default();
        assert!(
            sfz.contains("sample=lo_Fs3.wav lokey=0 hikey=57 pitch_keycenter=54"),
            "{sfz}"
        );
        assert!(
            sfz.contains("sample=hi_C4.wav lokey=58 hikey=127 pitch_keycenter=60"),
            "{sfz}"
        );
    }

    #[test]
    fn fetch_verifies_hashes_and_caches() {
        let dir = std::env::temp_dir().join(format!("sc_fetch_{}", std::process::id()));
        let b = sample_bank();
        let fake = |url: &str| -> Result<Vec<u8>, String> {
            if url.contains("lo%20F%233") {
                Ok(b"one".to_vec())
            } else {
                Ok(b"two".to_vec())
            }
        };
        let mut lines = Vec::new();
        let sfz = fetch_bank(&b, &dir, &fake, &mut |s| lines.push(s.to_string()));
        assert!(sfz.is_ok(), "{sfz:?}");
        assert!(dir.join("t").join("ATTRIBUTION.txt").exists());
        let mut again = Vec::new();
        let _ = fetch_bank(&b, &dir, &fake, &mut |s| again.push(s.to_string()));
        assert!(
            again.iter().all(|l| l.starts_with("ok (cached)")),
            "{again:?}"
        );
        let bad = |_: &str| -> Result<Vec<u8>, String> { Ok(b"tampered".to_vec()) };
        let dir2 = dir.join("bad");
        let r = fetch_bank(&b, &dir2, &bad, &mut |_| {});
        assert!(r.is_err_and(|e| e.contains("checksum mismatch")));
        assert!(!dir2.join("t").join("hi_C4.wav").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
