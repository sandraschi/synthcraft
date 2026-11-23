use std::path::PathBuf;
use synthcraft_core::sandbox::PathPolicy;

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("synthcraft_sb_{}_{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let _ = std::fs::create_dir_all(&d);
    d
}

fn policy(tag: &str) -> (PathPolicy, PathBuf, PathBuf) {
    let base = tmp(tag);
    let r = base.join("in");
    let w = base.join("out");
    let _ = std::fs::create_dir_all(&r);
    let _ = std::fs::create_dir_all(&w);
    let p = PathPolicy::new(std::slice::from_ref(&r), std::slice::from_ref(&w)).unwrap_or_default();
    (p, r, w)
}

#[test]
fn allows_inside_and_resolves_relative_to_the_first_root() {
    let (p, r, w) = policy("inside");
    assert!(p.is_complete());
    let f = r.join("bank.sf2");
    let _ = std::fs::write(&f, b"x");
    assert!(p.check_read(&f.to_string_lossy()).is_ok());
    assert!(p.check_read("bank.sf2").is_ok());
    let out = p.check_write("new_dir/render.wav");
    assert!(out.as_ref().is_ok_and(|o| o.ends_with("render.wav")));
    assert!(out.is_ok_and(|o| o.starts_with(&w) || o.to_string_lossy().contains("new_dir")));
}

#[test]
fn refuses_outside_dotdot_and_the_wrong_direction() {
    let (p, r, w) = policy("outside");
    let other = tmp("elsewhere").join("secret.wav");
    assert!(p.check_write(&other.to_string_lossy()).is_err());
    assert!(p.check_read(&other.to_string_lossy()).is_err());
    let escape = format!("{}/../in/x.wav", w.display());
    assert!(
        p.check_write(&escape).is_err(),
        "dotdot refused even if it lands inside"
    );
    // Reading roots are not write roots and the reverse.
    assert!(p.check_write(&r.join("a.wav").to_string_lossy()).is_err());
    assert!(p.check_read(&w.join("a.wav").to_string_lossy()).is_err());
}

#[test]
fn sibling_with_the_same_prefix_is_not_inside() {
    let base = tmp("prefix");
    let root = base.join("data");
    let sibling = base.join("data2");
    let _ = std::fs::create_dir_all(&root);
    let _ = std::fs::create_dir_all(&sibling);
    let p = PathPolicy::new(std::slice::from_ref(&root), std::slice::from_ref(&root))
        .unwrap_or_default();
    assert!(p
        .check_write(&sibling.join("x.wav").to_string_lossy())
        .is_err());
}

#[test]
fn missing_roots_deny_everything() {
    let p = PathPolicy::default();
    assert!(!p.is_complete());
    assert!(p.check_read("a").is_err());
    assert!(p.check_write("a").is_err());
    assert!(PathPolicy::new(&[PathBuf::from("/definitely/not/here")], &[]).is_err());
}

#[cfg(unix)]
#[test]
fn symlink_escape_is_refused() {
    let (p, _r, w) = policy("symlink");
    let target = tmp("symtarget");
    let link = w.join("link");
    assert!(std::os::unix::fs::symlink(&target, &link).is_ok());
    assert!(p
        .check_write(&link.join("x.wav").to_string_lossy())
        .is_err());
}
