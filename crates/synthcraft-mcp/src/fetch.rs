//! Blocking HTTPS download used by `fetch-bank`.

pub fn http_get(url: &str) -> Result<Vec<u8>, String> {
    let mut resp = ureq::get(url)
        .call()
        .map_err(|e| format!("download failed for {url}: {e}"))?;
    resp.body_mut()
        .with_config()
        .limit(200 * 1024 * 1024)
        .read_to_vec()
        .map_err(|e| format!("read failed for {url}: {e}"))
}
