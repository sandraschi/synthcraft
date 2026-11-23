//! Background download of free sample banks (see `synthcraft_core::banks`).

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

pub enum FetchMsg {
    Progress(String),
    /// Finished: the path of the generated `.sfz`, or an error message.
    Done(Result<String, String>),
}

fn http_get(url: &str) -> Result<Vec<u8>, String> {
    let mut resp = ureq::get(url)
        .call()
        .map_err(|e| format!("download failed for {url}: {e}"))?;
    resp.body_mut()
        .with_config()
        .limit(200 * 1024 * 1024)
        .read_to_vec()
        .map_err(|e| format!("read failed for {url}: {e}"))
}

/// Start downloading bank `id` into `dest` on a worker thread. The bank is also decoded once so
/// the shared cache is warm and the later load on the UI side is instant.
pub fn start(id: String, dest: PathBuf) -> Receiver<FetchMsg> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let result = (|| {
            let manifest = synthcraft_core::banks::manifest()?;
            let bank = manifest
                .banks
                .iter()
                .find(|b| b.id == id)
                .ok_or_else(|| format!("unknown bank {id}"))?;
            let total = bank.files.len();
            let mut done = 0usize;
            let sfz = synthcraft_core::banks::fetch_bank(bank, &dest, &http_get, &mut |_| {
                done += 1;
                let _ = tx.send(FetchMsg::Progress(format!(
                    "Downloading {}: {done}/{total} files",
                    bank.name
                )));
            })?;
            let spec = sfz.to_string_lossy().into_owned();
            let _ = tx.send(FetchMsg::Progress(format!(
                "Preparing {} (finding loops)",
                bank.name
            )));
            synthcraft_core::sampler::load_cached(&spec).map_err(|e| e.to_string())?;
            Ok(spec)
        })();
        let _ = tx.send(FetchMsg::Done(result));
    });
    rx
}

/// Decode a bank spec on a worker thread (warming the shared cache) so loading a big SF2 does
/// not freeze the UI. Finishes with the same messages as [`start`].
pub fn prime(spec: String) -> Receiver<FetchMsg> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let r = synthcraft_core::sampler::load_cached(&spec)
            .map(|_| spec.clone())
            .map_err(|e| e.to_string());
        let _ = tx.send(FetchMsg::Done(r));
    });
    rx
}
