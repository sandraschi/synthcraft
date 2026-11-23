//! Exercises the C ABI exactly as a C caller would, through raw pointers.
#![allow(unsafe_code)]

use std::ffi::{CStr, CString};
use synthcraft_ffi::*;

#[test]
fn version_and_header_agree() {
    assert_eq!(sc_abi_version(), SC_ABI_VERSION);
    let header = include_str!("../include/synthcraft.h");
    assert!(header.contains(&format!("#define SC_ABI_VERSION {SC_ABI_VERSION}u")));
    // Every exported function appears in the header.
    for name in [
        "sc_abi_version",
        "sc_create",
        "sc_destroy",
        "sc_set_sample_rate",
        "sc_note_on",
        "sc_note_off",
        "sc_all_notes_off",
        "sc_pitch_bend",
        "sc_mod_wheel",
        "sc_active_voices",
        "sc_set_max_voices",
        "sc_set_param",
        "sc_get_param",
        "sc_process",
        "sc_param_count",
        "sc_param_info_at",
        "sc_param_default",
        "sc_load_bank",
        "sc_clear_bank",
        "sc_state_json",
        "sc_free_string",
        "sc_load_state_json",
        "sc_last_error",
    ] {
        assert!(header.contains(name), "header misses {name}");
    }
}

#[test]
fn param_struct_layout_is_the_documented_one() {
    assert_eq!(std::mem::size_of::<ScParamInfo>(), 4 * 4 + 32 + 48 + 16);
    assert_eq!(std::mem::align_of::<ScParamInfo>(), 4);
}

#[test]
fn play_a_note_through_the_c_api() {
    // SAFETY: all pointers come from sc_create and local buffers that outlive the calls.
    unsafe {
        let e = sc_create(48_000.0);
        assert!(!e.is_null());
        let (mut l, mut r) = (vec![0.0_f32; 4800], vec![0.0_f32; 4800]);
        sc_set_max_voices(e, 1);
        sc_note_on(e, 60, 0.9);
        sc_note_on(e, 64, 0.9);
        assert_eq!(sc_active_voices(e), 1);
        sc_process(e, l.as_mut_ptr(), r.as_mut_ptr(), 4800);
        assert!(l.iter().map(|x| x * x).sum::<f32>() > 0.01);
        assert_eq!(l, r);
        sc_note_off(e, 60);
        sc_all_notes_off(e);
        sc_destroy(e);
    }
}

#[test]
fn params_enumerate_set_and_get() {
    // SAFETY: handles and out-structs are valid for the duration of the calls.
    unsafe {
        let e = sc_create(44_100.0);
        let n = sc_param_count();
        assert!(n >= 22);
        for i in 0..n {
            let mut info = std::mem::zeroed::<ScParamInfo>();
            assert_eq!(sc_param_info_at(i, &mut info), 1);
            let key = CStr::from_ptr(info.key.as_ptr())
                .to_string_lossy()
                .into_owned();
            assert!(!key.is_empty());
            assert!(info.min < info.max);
            assert_eq!(sc_param_default(info.id), info.default_value);
            let mid = (info.min + info.max) * 0.5;
            assert_eq!(sc_set_param(e, info.id, mid), 1);
            assert_eq!(sc_get_param(e, info.id), mid);
        }
        let mut info = std::mem::zeroed::<ScParamInfo>();
        assert_eq!(sc_param_info_at(n, &mut info), 0);
        assert_eq!(sc_set_param(e, 0xFFFF, 1.0), 0);
        assert!(sc_get_param(e, 0xFFFF).is_nan());
        sc_destroy(e);
    }
}

#[test]
fn null_handles_and_buffers_are_ignored() {
    // SAFETY: null is an explicitly supported argument for every call here.
    unsafe {
        sc_destroy(std::ptr::null_mut());
        sc_note_on(std::ptr::null_mut(), 60, 1.0);
        sc_process(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            64,
        );
        assert_eq!(sc_set_param(std::ptr::null_mut(), 0x1004, 1.0), 0);
        assert!(sc_state_json(std::ptr::null_mut()).is_null());
        assert_eq!(sc_load_bank(std::ptr::null_mut(), std::ptr::null()), 1);
        let e = sc_create(48_000.0);
        sc_process(e, std::ptr::null_mut(), std::ptr::null_mut(), 64);
        assert_eq!(sc_load_bank(e, std::ptr::null()), 1);
        sc_destroy(e);
    }
}

#[test]
fn state_round_trip_and_errors() {
    // SAFETY: strings are NUL-terminated and freed with the matching function.
    unsafe {
        let a = sc_create(48_000.0);
        sc_set_param(a, 0x1004, 640.0);
        let s = sc_state_json(a);
        assert!(!s.is_null());
        let b = sc_create(48_000.0);
        assert_eq!(sc_load_state_json(b, s), 0);
        assert_eq!(sc_get_param(b, 0x1004), 640.0);
        sc_free_string(s);

        let bad = CString::new("{nope").unwrap_or_default();
        assert_eq!(sc_load_state_json(b, bad.as_ptr()), 1);
        assert!(!CStr::from_ptr(sc_last_error(b)).to_bytes().is_empty());
        assert_eq!(
            sc_get_param(b, 0x1004),
            640.0,
            "failed load must not change state"
        );

        let nope = CString::new("no/such/bank.sfz").unwrap_or_default();
        assert_eq!(sc_load_bank(b, nope.as_ptr()), 1);
        assert!(!CStr::from_ptr(sc_last_error(b)).to_bytes().is_empty());
        sc_free_string(std::ptr::null_mut());
        sc_destroy(a);
        sc_destroy(b);
    }
}
