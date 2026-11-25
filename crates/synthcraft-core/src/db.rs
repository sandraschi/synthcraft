//! Bundled SQLite persistence for SynthCraft (`rusqlite` 0.40).
//!
//! Rules this module keeps:
//! - The schema is versioned (`PRAGMA user_version`). A database written by a newer
//!   SynthCraft is refused, never overwritten. Older ones are backed up, then migrated.
//! - Saves are per-row upserts inside one transaction. Only patches that no longer exist in
//!   the engine are deleted, and an engine with no patches is refused.
//! - Rows that cannot be read are moved to `synth_quarantine`, never dropped, and reported in
//!   `SynthEngine::load_warnings`.
//! - Open errors are returned to the caller. Nothing here falls back to defaults silently.

use crate::error::SynthError;
use crate::model::SynthPatch;
use crate::rack::{default_rack_modules, RackModuleInstance};
use crate::rompler::{
    builtin_demo_showcases, builtin_midi_depot, default_rack_cables, DemoShowcaseItem,
    MidiDepotClip, MidiHardwareConfig, RackCable,
};
use crate::SynthEngine;
use rusqlite::{params, Connection};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Current schema version stored in `PRAGMA user_version`.
pub const SCHEMA_VERSION: i64 = 2;

pub fn default_db_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("synthcraft").join("synthcraft.db")
}

fn st(ctx: &'static str) -> impl Fn(rusqlite::Error) -> SynthError {
    move |e| SynthError::Storage(format!("{ctx}: {e}"))
}

fn user_version(conn: &Connection) -> Result<i64, SynthError> {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(st("read schema version"))
}

fn has_table(conn: &Connection, name: &str) -> Result<bool, SynthError> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
        params![name],
        |r| r.get(0),
    )
    .map_err(st("table check"))
}

fn has_column(conn: &Connection, table: &str, col: &str) -> Result<bool, SynthError> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(st("table_info"))?;
    let names = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .map_err(st("table_info query"))?;
    for n in names {
        if n.map_err(st("table_info row"))? == col {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Open the database, refuse newer schemas, and bring older ones up to date.
fn open_migrated(path: &Path) -> Result<Connection, SynthError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SynthError::Storage(format!("create {}: {e}", parent.display())))?;
        }
    }
    let mut conn = Connection::open(path)
        .map_err(|e| SynthError::Storage(format!("open {}: {e}", path.display())))?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(st("busy timeout"))?;
    // WAL keeps readers (CLI, MCP) from blocking the app. A failure here is not fatal.
    let _ = conn.query_row("PRAGMA journal_mode=WAL", [], |r| r.get::<_, String>(0));

    let version = user_version(&conn)?;
    if version > SCHEMA_VERSION {
        return Err(SynthError::Storage(format!(
            "{} was written by a newer SynthCraft (schema {version}, this build understands {SCHEMA_VERSION}); refusing to open it. Update SynthCraft or use another --db path.",
            path.display()
        )));
    }
    if version == SCHEMA_VERSION {
        return Ok(conn);
    }

    let legacy = has_table(&conn, "synth_patches")?;
    if legacy {
        let backup = path.with_extension(format!("db.bak-v{version}"));
        if !backup.exists() {
            // Checkpoint so the copy contains everything, then copy the main file.
            let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| {
                r.get::<_, i64>(0)
            });
            std::fs::copy(path, &backup).map_err(|e| {
                SynthError::Storage(format!(
                    "backup before migration to {}: {e}",
                    backup.display()
                ))
            })?;
        }
    }

    let tx = conn.transaction().map_err(st("migration start"))?;
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS synth_patches (
            id TEXT PRIMARY KEY,
            patch_json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS synth_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            active_patch_id TEXT NOT NULL,
            sample_rate_hz INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS synth_kv (
            key TEXT PRIMARY KEY,
            json TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS synth_quarantine (
            id TEXT NOT NULL,
            kind TEXT NOT NULL,
            raw TEXT NOT NULL,
            error TEXT NOT NULL,
            quarantined_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )
    .map_err(st("migration schema"))?;
    if !has_column(&tx, "synth_patches", "position")? {
        tx.execute_batch(
            "ALTER TABLE synth_patches ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
             UPDATE synth_patches SET position = rowid;",
        )
        .map_err(st("migration position column"))?;
    }
    tx.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION};"))
        .map_err(st("set schema version"))?;
    tx.commit().map_err(st("migration commit"))?;
    Ok(conn)
}

fn put_kv<T: Serialize>(
    tx: &rusqlite::Transaction<'_>,
    key: &str,
    value: &T,
) -> Result<(), SynthError> {
    let json =
        serde_json::to_string(value).map_err(|e| SynthError::Serialization(e.to_string()))?;
    tx.execute(
        "INSERT INTO synth_kv (key, json) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET json = excluded.json WHERE json IS NOT excluded.json",
        params![key, json],
    )
    .map_err(st("write setting"))?;
    Ok(())
}

/// Write the whole engine. Existing rows are updated in place, unchanged rows are not touched.
pub fn save_to_db(engine: &SynthEngine, path: &Path) -> Result<(), SynthError> {
    if engine.patches.is_empty() {
        return Err(SynthError::Validation(
            "refusing to save an engine with no patches".into(),
        ));
    }
    let mut conn = open_migrated(path)?;
    let tx = conn.transaction().map_err(st("save start"))?;

    for (pos, p) in engine.patches.iter().enumerate() {
        let pj = serde_json::to_string(p).map_err(|e| SynthError::Serialization(e.to_string()))?;
        tx.execute(
            "INSERT INTO synth_patches (id, patch_json, position) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET patch_json = excluded.patch_json, position = excluded.position
             WHERE patch_json IS NOT excluded.patch_json OR position IS NOT excluded.position",
            params![p.id, pj, pos as i64],
        )
        .map_err(|e| SynthError::Storage(format!("save patch {}: {e}", p.id)))?;
    }

    let stored: Vec<String> = {
        let mut stmt = tx
            .prepare("SELECT id FROM synth_patches")
            .map_err(st("list patches"))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(st("list patches query"))?;
        rows.collect::<Result<_, _>>()
            .map_err(st("list patches row"))?
    };
    for id in stored {
        if !engine.patches.iter().any(|p| p.id == id) {
            tx.execute("DELETE FROM synth_patches WHERE id = ?1", params![id])
                .map_err(st("delete removed patch"))?;
        }
    }

    tx.execute(
        "INSERT INTO synth_state (id, active_patch_id, sample_rate_hz) VALUES (1, ?1, ?2)
         ON CONFLICT(id) DO UPDATE SET active_patch_id = excluded.active_patch_id,
             sample_rate_hz = excluded.sample_rate_hz",
        params![engine.active_patch_id, i64::from(engine.sample_rate_hz)],
    )
    .map_err(st("save state"))?;

    put_kv(&tx, "rack_modules", &engine.rack_modules)?;
    put_kv(&tx, "rack_cables", &engine.rack_cables)?;
    put_kv(&tx, "midi_depot", &engine.midi_depot)?;
    put_kv(&tx, "demos", &engine.demos)?;
    put_kv(&tx, "midi_hw", &engine.midi_hw)?;

    tx.commit().map_err(st("save commit"))
}

fn quarantine(conn: &Connection, id: &str, kind: &str, raw: &str, error: &str) {
    let _ = conn.execute(
        "INSERT INTO synth_quarantine (id, kind, raw, error) VALUES (?1, ?2, ?3, ?4)",
        params![id, kind, raw, error],
    );
}

fn get_kv<T: DeserializeOwned>(
    conn: &Connection,
    key: &str,
    default: impl FnOnce() -> T,
    warnings: &mut Vec<String>,
) -> T {
    let raw: Option<String> = conn
        .query_row(
            "SELECT json FROM synth_kv WHERE key = ?1",
            params![key],
            |r| r.get(0),
        )
        .ok();
    let Some(raw) = raw else {
        return default();
    };
    match serde_json::from_str::<T>(&raw) {
        Ok(v) => v,
        Err(e) => {
            quarantine(conn, key, "setting", &raw, &e.to_string());
            warnings.push(format!(
                "Setting '{key}' could not be read ({e}); defaults are in use and the stored value was kept in the quarantine table."
            ));
            default()
        }
    }
}

/// Load the engine. `Ok(None)` means there is no database (or no SynthCraft data in it yet).
/// Any error means the file exists but could not be used, and it is left untouched.
pub fn load_from_db(path: &Path) -> Result<Option<SynthEngine>, SynthError> {
    if !path.exists() {
        return Ok(None);
    }
    let conn = open_migrated(path)?;
    let mut warnings: Vec<String> = Vec::new();

    let rows: Vec<(String, String)> = {
        let mut stmt = conn
            .prepare("SELECT id, patch_json FROM synth_patches ORDER BY position ASC, rowid ASC")
            .map_err(st("prepare patches"))?;
        let it = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(st("query patches"))?;
        it.collect::<Result<_, _>>().map_err(st("patch row"))?
    };

    let mut patches: Vec<SynthPatch> = Vec::new();
    let mut bad = 0usize;
    for (id, raw) in rows {
        match serde_json::from_str::<SynthPatch>(&raw) {
            Ok(p) => patches.push(p),
            Err(e) => {
                bad += 1;
                quarantine(&conn, &id, "patch", &raw, &e.to_string());
                let _ = conn.execute("DELETE FROM synth_patches WHERE id = ?1", params![id]);
                warnings.push(format!(
                    "Patch '{id}' could not be read ({e}); it was moved to the quarantine table, not deleted."
                ));
            }
        }
    }

    if patches.is_empty() {
        if bad == 0 {
            return Ok(None);
        }
        let mut seeded = SynthEngine::factory_seed();
        seeded.load_warnings = warnings;
        return Ok(Some(seeded));
    }

    let state: Option<(String, i64)> = conn
        .query_row(
            "SELECT active_patch_id, sample_rate_hz FROM synth_state WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();
    let (mut active_patch_id, sample_rate_hz) = match state {
        Some((id, sr)) => (id, sr.clamp(8000, 96000) as u32),
        None => (patches[0].id.clone(), 48000),
    };
    if !patches.iter().any(|p| p.id == active_patch_id) {
        warnings.push(format!(
            "Active patch '{active_patch_id}' no longer exists; switched to '{}'.",
            patches[0].id
        ));
        active_patch_id = patches[0].id.clone();
    }

    let rack_modules: Vec<RackModuleInstance> =
        get_kv(&conn, "rack_modules", default_rack_modules, &mut warnings);
    let rack_cables: Vec<RackCable> =
        get_kv(&conn, "rack_cables", default_rack_cables, &mut warnings);
    let midi_depot: Vec<MidiDepotClip> =
        get_kv(&conn, "midi_depot", builtin_midi_depot, &mut warnings);
    let demos: Vec<DemoShowcaseItem> =
        get_kv(&conn, "demos", builtin_demo_showcases, &mut warnings);
    let midi_hw: MidiHardwareConfig =
        get_kv(&conn, "midi_hw", MidiHardwareConfig::default, &mut warnings);

    Ok(Some(SynthEngine {
        active_patch_id,
        sample_rate_hz,
        patches,
        rack_modules,
        rack_cables,
        midi_depot,
        demos,
        midi_hw,
        load_warnings: warnings,
    }))
}

/// Store an opaque JSON string under `key` (used by the desktop app for UI preferences).
pub fn save_setting(path: &Path, key: &str, json: &str) -> Result<(), SynthError> {
    let conn = open_migrated(path)?;
    conn.execute(
        "INSERT INTO synth_kv (key, json) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET json = excluded.json WHERE json IS NOT excluded.json",
        params![key, json],
    )
    .map_err(st("save setting"))?;
    Ok(())
}

/// Read a string stored with [`save_setting`]. `Ok(None)` when the key or database is absent.
pub fn load_setting(path: &Path, key: &str) -> Result<Option<String>, SynthError> {
    if !path.exists() {
        return Ok(None);
    }
    let conn = open_migrated(path)?;
    match conn.query_row(
        "SELECT json FROM synth_kv WHERE key = ?1",
        params![key],
        |r| r.get::<_, String>(0),
    ) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(SynthError::Storage(format!("load setting {key}: {e}"))),
    }
}
