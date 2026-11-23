//! Background saving for the desktop app.
//!
//! The UI thread never touches SQLite. It hands a snapshot to a worker thread; the worker
//! coalesces queued snapshots to the newest one and reports back. Dropping the worker waits
//! for the queue to drain, so the last edit is on disk before the app exits.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use synthcraft_core::db::{save_setting, save_to_db};
use synthcraft_core::SynthEngine;

pub struct SaveJob {
    pub path: PathBuf,
    pub engine: SynthEngine,
    /// `(key, json)` pairs written alongside the engine.
    pub settings: Vec<(String, String)>,
}

pub fn run_job(job: &SaveJob) -> Result<(), String> {
    save_to_db(&job.engine, &job.path).map_err(|e| e.to_string())?;
    for (k, v) in &job.settings {
        save_setting(&job.path, k, v).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub struct SaveWorker {
    tx: Option<Sender<SaveJob>>,
    results: Receiver<Result<(), String>>,
    handle: Option<JoinHandle<()>>,
}

impl SaveWorker {
    pub fn spawn() -> Self {
        let (tx, rx) = channel::<SaveJob>();
        let (res_tx, results) = channel();
        let handle = std::thread::spawn(move || {
            while let Ok(mut job) = rx.recv() {
                // Only the newest queued snapshot matters.
                while let Ok(newer) = rx.try_recv() {
                    job = newer;
                }
                let _ = res_tx.send(run_job(&job));
            }
        });
        Self {
            tx: Some(tx),
            results,
            handle: Some(handle),
        }
    }

    pub fn submit(&self, job: SaveJob) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(job);
        }
    }

    /// Results of finished saves since the last call.
    pub fn poll(&self) -> Vec<Result<(), String>> {
        self.results.try_iter().collect()
    }
}

impl Drop for SaveWorker {
    fn drop(&mut self) {
        self.tx = None;
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}
