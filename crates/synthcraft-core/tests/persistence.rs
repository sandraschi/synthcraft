//! Persistence rules: versioning, no silent data loss, nothing overwritten on error.

use rusqlite::{params, Connection};
use std::path::PathBuf;
use synthcraft_core::db::{load_from_db, save_to_db, SCHEMA_VERSION};
use synthcraft_core::SynthEngine;

fn temp_db(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("synthcraft_pt_{}_{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    dir.join("t.db")
}

fn json(e: &SynthEngine) -> String {
    serde_json::to_string(e).unwrap_or_default()
}

#[test]
fn roundtrip_keeps_patches_settings_cables_and_order() {
    let path = temp_db("roundtrip");
    let mut e = SynthEngine::factory_seed();
    e.patches[1].name = "Renamed Reese".into();
    e.patches.swap(0, 2);
    e.active_patch_id = e.patches[3].id.clone();
    e.midi_hw.midi_channel = 5;
    e.midi_hw.pitch_bend_semitones = 7;
    if let Some(c) = e.rack_cables.first_mut() {
        c.from_module = "edited-module".into();
    }
    assert!(save_to_db(&e, &path).is_ok());
    let loaded = load_from_db(&path).ok().flatten();
    assert!(loaded.is_some());
    if let Some(l) = loaded {
        assert_eq!(json(&l), json(&e));
        assert!(l.load_warnings.is_empty());
    }
}

#[test]
fn saving_twice_and_removing_a_patch_works_per_row() {
    let path = temp_db("rows");
    let mut e = SynthEngine::factory_seed();
    assert!(save_to_db(&e, &path).is_ok());
    assert!(save_to_db(&e, &path).is_ok());
    let removed = e.patches.remove(2).id;
    assert!(save_to_db(&e, &path).is_ok());
    let l = load_from_db(&path).ok().flatten();
    assert!(l.is_some_and(
        |l| l.patches.iter().all(|p| p.id != removed) && l.patches.len() == e.patches.len()
    ));
}

#[test]
fn empty_engine_is_refused_and_does_not_wipe_the_database() {
    let path = temp_db("empty");
    let e = SynthEngine::factory_seed();
    assert!(save_to_db(&e, &path).is_ok());
    let mut empty = e.clone();
    empty.patches.clear();
    assert!(save_to_db(&empty, &path).is_err());
    let l = load_from_db(&path).ok().flatten();
    assert!(l.is_some_and(|l| l.patches.len() == e.patches.len()));
}

#[test]
fn newer_schema_is_refused_and_left_untouched() {
    let path = temp_db("newer");
    assert!(save_to_db(&SynthEngine::factory_seed(), &path).is_ok());
    {
        let Ok(conn) = Connection::open(&path) else {
            return;
        };
        let _ = conn.execute_batch(&format!("PRAGMA user_version = {};", SCHEMA_VERSION + 5));
    }
    let before = std::fs::read(&path).unwrap_or_default();
    let err = load_from_db(&path)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(err.contains("newer SynthCraft"), "{err}");
    assert!(save_to_db(&SynthEngine::factory_seed(), &path).is_err());
    let after = std::fs::read(&path).unwrap_or_default();
    assert_eq!(before.len(), after.len());
}

#[test]
fn a_file_that_is_not_a_database_is_an_error_and_is_not_overwritten() {
    let path = temp_db("garbage");
    let junk = b"this is definitely not sqlite, it is somebody's notes".repeat(40);
    assert!(std::fs::write(&path, &junk).is_ok());
    assert!(load_from_db(&path).is_err());
    assert!(SynthEngine::open_or_seed(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap_or_default(), junk);
}

#[test]
fn unreadable_rows_are_quarantined_and_reported_not_dropped() {
    let path = temp_db("quarantine");
    assert!(save_to_db(&SynthEngine::factory_seed(), &path).is_ok());
    {
        let Ok(conn) = Connection::open(&path) else {
            return;
        };
        let _ = conn.execute(
            "INSERT INTO synth_patches (id, patch_json, position) VALUES (?1, ?2, 99)",
            params!["broken_patch", "{\"id\": \"broken_patch\", \"name\": 12"],
        );
    }
    let l = load_from_db(&path).ok().flatten();
    assert!(l.is_some());
    let Some(l) = l else {
        return;
    };
    assert!(l.load_warnings.iter().any(|w| w.contains("broken_patch")));
    assert!(l.patches.iter().all(|p| p.id != "broken_patch"));
    // A save after loading must not erase the quarantined copy.
    assert!(save_to_db(&l, &path).is_ok());
    let Ok(conn) = Connection::open(&path) else {
        return;
    };
    let kept: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM synth_quarantine WHERE id = 'broken_patch'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    assert_eq!(kept, 1);
}

#[test]
fn legacy_unversioned_database_is_backed_up_then_migrated_without_loss() {
    let path = temp_db("legacy");
    let seed = SynthEngine::factory_seed();
    {
        let Ok(conn) = Connection::open(&path) else {
            return;
        };
        let _ = conn.execute_batch(
            "CREATE TABLE synth_patches (id TEXT PRIMARY KEY, patch_json TEXT NOT NULL);
             CREATE TABLE synth_state (id INTEGER PRIMARY KEY CHECK (id = 1),
                active_patch_id TEXT NOT NULL, sample_rate_hz INTEGER NOT NULL);",
        );
        for p in seed.patches.iter().take(2) {
            let _ = conn.execute(
                "INSERT INTO synth_patches (id, patch_json) VALUES (?1, ?2)",
                params![p.id, serde_json::to_string(p).unwrap_or_default()],
            );
        }
        let _ = conn.execute(
            "INSERT INTO synth_state (id, active_patch_id, sample_rate_hz) VALUES (1, ?1, 44100)",
            params![seed.patches[1].id],
        );
    }
    let e = SynthEngine::open_or_seed(&path);
    assert!(e.is_ok());
    let Ok(e) = e else {
        return;
    };
    assert_eq!(e.active_patch_id, seed.patches[1].id);
    assert_eq!(e.sample_rate_hz, 44100);
    assert_eq!(e.patches[0].id, seed.patches[0].id, "order preserved");
    assert_eq!(
        e.patches.len(),
        seed.patches.len(),
        "missing factory patches added"
    );
    let backup = path.with_extension("db.bak-v0");
    assert!(backup.exists(), "backup before migration");
    let Ok(conn) = Connection::open(&path) else {
        return;
    };
    let v: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap_or(0);
    assert_eq!(v, SCHEMA_VERSION);
}
