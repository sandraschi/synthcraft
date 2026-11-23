//! Bundled SQLite persistence for SynthCraft (`rusqlite` 0.40).

use crate::error::SynthError;
use crate::model::SynthPatch;
use crate::SynthEngine;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

pub fn default_db_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("synthcraft").join("synthcraft.db")
}

pub fn save_to_db(engine: &SynthEngine, path: &Path) -> Result<(), SynthError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SynthError::Storage(format!("create_dir_all failed: {e}")))?;
        }
    }
    let mut conn =
        Connection::open(path).map_err(|e| SynthError::Storage(format!("open db failed: {e}")))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS synth_patches (
            id TEXT PRIMARY KEY,
            patch_json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS synth_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            active_patch_id TEXT NOT NULL,
            sample_rate_hz INTEGER NOT NULL
        );",
    )
    .map_err(|e| SynthError::Storage(format!("schema init failed: {e}")))?;

    let tx = conn
        .transaction()
        .map_err(|e| SynthError::Storage(format!("tx start failed: {e}")))?;

    tx.execute("DELETE FROM synth_patches", [])
        .map_err(|e| SynthError::Storage(format!("clear synth_patches: {e}")))?;
    tx.execute("DELETE FROM synth_state", [])
        .map_err(|e| SynthError::Storage(format!("clear synth_state: {e}")))?;

    for p in &engine.patches {
        let pj = serde_json::to_string(p).map_err(|e| SynthError::Serialization(e.to_string()))?;
        tx.execute(
            "INSERT INTO synth_patches (id, patch_json) VALUES (?1, ?2)",
            params![p.id, pj],
        )
        .map_err(|e| SynthError::Storage(format!("insert patch {}: {e}", p.id)))?;
    }

    tx.execute(
        "INSERT INTO synth_state (id, active_patch_id, sample_rate_hz) VALUES (1, ?1, ?2)",
        params![engine.active_patch_id, i64::from(engine.sample_rate_hz)],
    )
    .map_err(|e| SynthError::Storage(format!("insert synth_state: {e}")))?;

    tx.commit()
        .map_err(|e| SynthError::Storage(format!("tx commit: {e}")))?;
    Ok(())
}

pub fn load_from_db(path: &Path) -> Result<Option<SynthEngine>, SynthError> {
    if !path.exists() {
        return Ok(None);
    }
    let conn =
        Connection::open(path).map_err(|e| SynthError::Storage(format!("open db failed: {e}")))?;

    let has_table: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='synth_patches')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| SynthError::Storage(format!("table check failed: {e}")))?;
    if !has_table {
        return Ok(None);
    }

    let mut stmt = conn
        .prepare("SELECT patch_json FROM synth_patches ORDER BY rowid ASC")
        .map_err(|e| SynthError::Storage(format!("prepare patches: {e}")))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| SynthError::Storage(format!("query patches: {e}")))?;

    let mut patches = Vec::new();
    for r in rows {
        let raw = r.map_err(|e| SynthError::Storage(format!("patch row: {e}")))?;
        if let Ok(p) = serde_json::from_str::<SynthPatch>(&raw) {
            patches.push(p);
        }
    }
    if patches.is_empty() {
        return Ok(None);
    }

    let state: Option<(String, i64)> = conn
        .query_row(
            "SELECT active_patch_id, sample_rate_hz FROM synth_state WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    let (active_patch_id, sample_rate_hz) = match state {
        Some((id, sr)) => (id, sr.clamp(8000, 96000) as u32),
        None => (patches[0].id.clone(), 48000),
    };

    Ok(Some(SynthEngine {
        active_patch_id,
        sample_rate_hz,
        patches,
        rack_cables: crate::rompler::default_rack_cables(),
        midi_depot: crate::rompler::builtin_midi_depot(),
        demos: crate::rompler::builtin_demo_showcases(),
        midi_hw: crate::rompler::MidiHardwareConfig::default(),
    }))
}
