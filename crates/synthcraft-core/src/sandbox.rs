//! File access policy for agent entry points (AGENTS rule 18).
//!
//! An agent may only read below the read roots and only write below the write roots. Paths
//! are resolved before they are compared: `..` is refused, symlinks are followed, and the
//! comparison is by path component, so `C:\\data2` is not inside `C:\\data`.

use crate::error::SynthError;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct PathPolicy {
    read: Vec<PathBuf>,
    write: Vec<PathBuf>,
}

fn deny(msg: impl Into<String>) -> SynthError {
    SynthError::Validation(msg.into())
}

fn canonical_root(p: &Path) -> Result<PathBuf, SynthError> {
    let c = std::fs::canonicalize(p).map_err(|e| {
        deny(format!(
            "automation root {} is not usable: {e}",
            p.display()
        ))
    })?;
    if !c.is_dir() {
        return Err(deny(format!(
            "automation root {} is not a directory",
            p.display()
        )));
    }
    Ok(c)
}

/// Resolve `path` against the filesystem without requiring it to exist: the nearest existing
/// ancestor is canonicalized (following symlinks) and the missing tail is appended.
fn resolve(path: &Path) -> Result<PathBuf, SynthError> {
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(deny(format!("'..' is not allowed in {}", path.display())));
    }
    let mut existing = path.to_path_buf();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    loop {
        if let Ok(c) = std::fs::canonicalize(&existing) {
            let mut out = c;
            for part in tail.iter().rev() {
                out.push(part);
            }
            return Ok(out);
        }
        let Some(name) = existing.file_name().map(std::ffi::OsStr::to_os_string) else {
            return Err(deny(format!("cannot resolve {}", path.display())));
        };
        tail.push(name);
        if !existing.pop() {
            return Err(deny(format!("cannot resolve {}", path.display())));
        }
    }
}

impl PathPolicy {
    pub fn new(read: &[PathBuf], write: &[PathBuf]) -> Result<Self, SynthError> {
        Ok(Self {
            read: read
                .iter()
                .map(|p| canonical_root(p))
                .collect::<Result<_, _>>()?,
            write: write
                .iter()
                .map(|p| canonical_root(p))
                .collect::<Result<_, _>>()?,
        })
    }

    /// True when at least one read root and one write root are set.
    pub fn is_complete(&self) -> bool {
        !self.read.is_empty() && !self.write.is_empty()
    }

    /// True when any root is set, meaning file access must be checked.
    pub fn is_active(&self) -> bool {
        !self.read.is_empty() || !self.write.is_empty()
    }

    fn check(&self, raw: &str, roots: &[PathBuf], what: &str) -> Result<PathBuf, SynthError> {
        let Some(first) = roots.first() else {
            return Err(deny(format!(
                "{what} access is not allowed: no --automation-{what}-root was given"
            )));
        };
        let p = Path::new(raw);
        let joined = if p.is_absolute() {
            p.to_path_buf()
        } else {
            first.join(p)
        };
        let resolved = resolve(&joined)?;
        if roots.iter().any(|r| resolved.starts_with(r)) {
            Ok(resolved)
        } else {
            Err(deny(format!(
                "{what} access denied: {} is outside the allowed {what} roots",
                resolved.display()
            )))
        }
    }

    /// Resolve `raw` for reading. Relative paths are taken relative to the first read root.
    pub fn check_read(&self, raw: &str) -> Result<PathBuf, SynthError> {
        self.check(raw, &self.read, "read")
    }

    /// Resolve `raw` for writing. Relative paths are taken relative to the first write root.
    pub fn check_write(&self, raw: &str) -> Result<PathBuf, SynthError> {
        self.check(raw, &self.write, "write")
    }
}
