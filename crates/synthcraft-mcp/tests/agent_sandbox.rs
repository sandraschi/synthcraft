//! The MCP entry point must not start without automation roots and must keep file access
//! inside them (AGENTS rule 18).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_synthcraft-mcp")
}

fn dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("sc_mcp_{}_{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let _ = std::fs::create_dir_all(&d);
    d
}

#[test]
fn mcp_refuses_to_start_without_both_roots() {
    let base = dir("noroots");
    let db = base.join("t.db");
    let out = Command::new(bin())
        .args(["--db", &db.to_string_lossy(), "mcp"])
        .stdin(Stdio::null())
        .output();
    let Ok(out) = out else {
        return;
    };
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("automation-read-root"));

    let out = Command::new(bin())
        .args([
            "--db",
            &db.to_string_lossy(),
            "--automation-read-root",
            &base.to_string_lossy(),
            "mcp",
        ])
        .stdin(Stdio::null())
        .output();
    assert_eq!(out.ok().and_then(|o| o.status.code()), Some(2));
}

fn call(base: &Path, line: &str) -> String {
    let db = base.join("t.db");
    let rd = base.join("in");
    let wr = base.join("out");
    let _ = std::fs::create_dir_all(&rd);
    let _ = std::fs::create_dir_all(&wr);
    let child = Command::new(bin())
        .args([
            "--db",
            &db.to_string_lossy(),
            "--automation-read-root",
            &rd.to_string_lossy(),
            "--automation-write-root",
            &wr.to_string_lossy(),
            "mcp",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    let Ok(mut child) = child else {
        return String::new();
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = writeln!(stdin, "{line}");
    }
    let out = child.wait_with_output();
    out.map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

#[test]
fn render_outside_the_write_root_is_refused_and_inside_works() {
    let base = dir("render");
    let outside = base.join("elsewhere.wav");
    let req = |p: &Path| {
        format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"render_wav_file","arguments":{{"output_path":{},"duration_ms":200}}}}}}"#,
            serde_json::to_string(&p.to_string_lossy()).unwrap_or_default()
        )
    };
    let resp = call(&base, &req(&outside));
    assert!(resp.contains("\"isError\":true"), "{resp}");
    assert!(resp.contains("write access denied"), "{resp}");
    assert!(!outside.exists());

    let inside = base.join("out").join("ok.wav");
    let resp = call(&base, &req(&inside));
    assert!(resp.contains("\"isError\":false"), "{resp}");
    assert!(inside.exists());
}

#[test]
fn sampler_bank_outside_the_read_root_is_refused() {
    let base = dir("bank");
    let line = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"set_sampler_bank","arguments":{{"bank":{}}}}}}}"#,
        serde_json::to_string(&base.join("secret.sf2").to_string_lossy()).unwrap_or_default()
    );
    let resp = call(&base, &line);
    assert!(resp.contains("read access denied"), "{resp}");
}
