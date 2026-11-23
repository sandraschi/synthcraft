//! C ABI for the SynthCraft plugin engine.
//!
//! Every function is a thin, null-checked wrapper over
//! [`synthcraft_core::plugin::PluginEngine`]. The header is `include/synthcraft.h`; keep the
//! two in step (`tests/abi.rs` checks the struct layout and the version constant).
//!
//! Rules for callers:
//! - One `ScEngine` is used from one thread at a time. `sc_process`, `sc_note_*`,
//!   `sc_set_param` and `sc_get_param` are real-time safe (no allocation, no locks).
//! - `sc_create`, `sc_destroy`, `sc_set_sample_rate`, `sc_load_bank`, `sc_clear_bank`,
//!   `sc_state_json` and `sc_load_state_json` allocate; call them off the audio thread and
//!   never while another thread is inside `sc_process` on the same engine.
//! - Strings are UTF-8, NUL-terminated. Strings returned by `sc_state_json` must be freed
//!   with `sc_free_string`. `sc_last_error` is owned by the engine and valid until the next
//!   call that can fail.

use std::ffi::{c_char, c_float, c_int, c_uint, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use synthcraft_core::params::{param_def, PARAMS};
use synthcraft_core::plugin::PluginEngine;

/// ABI version. Bump on any incompatible change to the functions or structs below.
pub const SC_ABI_VERSION: u32 = 1;

/// Opaque engine handle (`sc_engine` in the header).
pub struct ScEngine {
    inner: PluginEngine,
    last_error: CString,
}

impl ScEngine {
    fn set_error(&mut self, msg: &str) {
        let clean: String = msg.chars().filter(|c| *c != '\0').collect();
        self.last_error = CString::new(clean).unwrap_or_default();
    }
}

/// Parameter description (`sc_param_info` in the header).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScParamInfo {
    pub id: u32,
    pub min: c_float,
    pub max: c_float,
    pub default_value: c_float,
    pub key: [c_char; 32],
    pub name: [c_char; 48],
    pub module: [c_char; 16],
}

fn fill<const N: usize>(dst: &mut [c_char; N], src: &str) {
    for (d, b) in dst.iter_mut().zip(src.bytes().take(N - 1)) {
        *d = b as c_char;
    }
}

fn guard<T>(default: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(default)
}

/// Version of the ABI this library implements.
#[no_mangle]
pub extern "C" fn sc_abi_version() -> u32 {
    SC_ABI_VERSION
}

/// Create an engine. Returns null on failure.
#[no_mangle]
pub extern "C" fn sc_create(sample_rate: c_float) -> *mut ScEngine {
    guard(ptr::null_mut(), || {
        Box::into_raw(Box::new(ScEngine {
            inner: PluginEngine::new(sample_rate),
            last_error: CString::default(),
        }))
    })
}

/// Destroy an engine created by `sc_create`. Null is ignored.
///
/// # Safety
/// `e` must be null or a pointer returned by `sc_create` that has not been destroyed, and no
/// other thread may be using it.
#[no_mangle]
pub unsafe extern "C" fn sc_destroy(e: *mut ScEngine) {
    if !e.is_null() {
        // SAFETY: the caller guarantees `e` came from `sc_create` and is not used again.
        drop(unsafe { Box::from_raw(e) });
    }
}

/// Mutable access to the engine behind a handle, or `None` for null.
///
/// # Safety
/// `e` must be null or a live pointer from `sc_create`, used by one thread at a time.
unsafe fn eng<'a>(e: *mut ScEngine) -> Option<&'a mut ScEngine> {
    // SAFETY: forwarded from the caller's contract.
    unsafe { e.as_mut() }
}

/// Change the sample rate. Sounding notes are cut. Not real-time safe.
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_set_sample_rate(e: *mut ScEngine, sample_rate: c_float) {
    // SAFETY: forwarded from the caller's contract.
    if let Some(e) = unsafe { eng(e) } {
        guard((), || e.inner.set_sample_rate(sample_rate));
    }
}

/// Start a note. `velocity` is 0..1; 0 acts as note off.
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_note_on(e: *mut ScEngine, note: u8, velocity: c_float) {
    // SAFETY: forwarded from the caller's contract.
    if let Some(e) = unsafe { eng(e) } {
        guard((), || e.inner.note_on(note, velocity));
    }
}

/// Release a note.
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_note_off(e: *mut ScEngine, note: u8) {
    // SAFETY: forwarded from the caller's contract.
    if let Some(e) = unsafe { eng(e) } {
        guard((), || e.inner.note_off(note));
    }
}

/// Release every note.
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_all_notes_off(e: *mut ScEngine) {
    // SAFETY: forwarded from the caller's contract.
    if let Some(e) = unsafe { eng(e) } {
        guard((), || e.inner.all_notes_off());
    }
}

/// Pitch bend in semitones.
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_pitch_bend(e: *mut ScEngine, semitones: c_float) {
    // SAFETY: forwarded from the caller's contract.
    if let Some(e) = unsafe { eng(e) } {
        guard((), || e.inner.pitch_bend(semitones));
    }
}

/// Mod wheel, 0..1.
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_mod_wheel(e: *mut ScEngine, value: c_float) {
    // SAFETY: forwarded from the caller's contract.
    if let Some(e) = unsafe { eng(e) } {
        guard((), || e.inner.mod_wheel(value));
    }
}

/// Limit polyphony to `voices` (1 to 16).
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_set_max_voices(e: *mut ScEngine, voices: c_uint) {
    // SAFETY: forwarded from the caller's contract.
    if let Some(e) = unsafe { eng(e) } {
        guard((), || e.inner.set_max_voices(voices as usize));
    }
}

/// Number of sounding voices.
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_active_voices(e: *mut ScEngine) -> c_uint {
    // SAFETY: forwarded from the caller's contract.
    unsafe { eng(e) }.map_or(0, |e| e.inner.active_voices() as c_uint)
}

/// Number of host parameters.
#[no_mangle]
pub extern "C" fn sc_param_count() -> c_uint {
    PARAMS.len() as c_uint
}

/// Describe parameter `index` (0 to `sc_param_count() - 1`). Returns 1 on success, 0 if the
/// index is out of range or `out` is null.
///
/// # Safety
/// `out` must be null or point to writable memory for one `ScParamInfo`.
#[no_mangle]
pub unsafe extern "C" fn sc_param_info_at(index: c_uint, out: *mut ScParamInfo) -> c_int {
    let Some(d) = PARAMS.get(index as usize) else {
        return 0;
    };
    if out.is_null() {
        return 0;
    }
    let mut info = ScParamInfo {
        id: d.id,
        min: d.min,
        max: d.max,
        default_value: d.default,
        key: [0; 32],
        name: [0; 48],
        module: [0; 16],
    };
    fill(&mut info.key, d.key);
    fill(&mut info.name, d.name);
    fill(&mut info.module, d.module);
    // SAFETY: `out` is non-null and the caller guarantees it is writable for one struct.
    unsafe { out.write(info) };
    1
}

/// Set a parameter by id. Returns 1 if the id exists, else 0. The value is clamped.
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_set_param(e: *mut ScEngine, id: c_uint, value: c_float) -> c_int {
    // SAFETY: forwarded from the caller's contract.
    let Some(e) = (unsafe { eng(e) }) else {
        return 0;
    };
    guard(0, || c_int::from(e.inner.set_param(id, value).is_some()))
}

/// Read a parameter by id. Returns NaN for an unknown id or a null engine.
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_get_param(e: *mut ScEngine, id: c_uint) -> c_float {
    // SAFETY: forwarded from the caller's contract.
    let Some(e) = (unsafe { eng(e) }) else {
        return c_float::NAN;
    };
    guard(c_float::NAN, || {
        e.inner.get_param(id).unwrap_or(c_float::NAN)
    })
}

/// Default value of a parameter, or NaN for an unknown id. Needs no engine.
#[no_mangle]
pub extern "C" fn sc_param_default(id: c_uint) -> c_float {
    param_def(id).map_or(c_float::NAN, |d| d.default)
}

/// Render `frames` frames of audio into `left` and `right` (both mono copies today).
///
/// # Safety
/// `e` must be null or a live engine pointer; `left` and `right` must each be null or point
/// to `frames` writable floats that do not overlap.
#[no_mangle]
pub unsafe extern "C" fn sc_process(
    e: *mut ScEngine,
    left: *mut c_float,
    right: *mut c_float,
    frames: c_uint,
) {
    // SAFETY: forwarded from the caller's contract.
    let Some(e) = (unsafe { eng(e) }) else {
        return;
    };
    if left.is_null() || right.is_null() || frames == 0 {
        return;
    }
    let n = frames as usize;
    // SAFETY: both buffers are non-null and the caller guarantees `frames` writable,
    // non-overlapping floats each.
    let (l, r) = unsafe {
        (
            std::slice::from_raw_parts_mut(left, n),
            std::slice::from_raw_parts_mut(right, n),
        )
    };
    guard((), || e.inner.process(l, r));
}

/// Load a sample bank. Returns 0 on success; on failure returns 1 and sets `sc_last_error`.
///
/// # Safety
/// `e` must be null or a live engine pointer; `path` must be null or a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn sc_load_bank(e: *mut ScEngine, path: *const c_char) -> c_int {
    // SAFETY: forwarded from the caller's contract.
    let Some(e) = (unsafe { eng(e) }) else {
        return 1;
    };
    if path.is_null() {
        e.set_error("null path");
        return 1;
    }
    // SAFETY: `path` is non-null and NUL-terminated per the caller's contract.
    let spec = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .into_owned();
    match guard(Err("internal error".to_string()), || {
        e.inner.load_bank(&spec).map_err(|err| err.to_string())
    }) {
        Ok(()) => 0,
        Err(msg) => {
            e.set_error(&msg);
            1
        }
    }
}

/// Remove the sample bank.
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_clear_bank(e: *mut ScEngine) {
    // SAFETY: forwarded from the caller's contract.
    if let Some(e) = unsafe { eng(e) } {
        guard((), || e.inner.clear_bank());
    }
}

/// The current patch as a JSON string, or null on failure. Free with `sc_free_string`.
///
/// # Safety
/// `e` must be null or a live engine pointer.
#[no_mangle]
pub unsafe extern "C" fn sc_state_json(e: *mut ScEngine) -> *mut c_char {
    // SAFETY: forwarded from the caller's contract.
    let Some(e) = (unsafe { eng(e) }) else {
        return ptr::null_mut();
    };
    match guard(None, || e.inner.state_json().ok()) {
        Some(s) => CString::new(s).map_or(ptr::null_mut(), CString::into_raw),
        None => {
            e.set_error("could not serialise state");
            ptr::null_mut()
        }
    }
}

/// Free a string returned by `sc_state_json`. Null is ignored.
///
/// # Safety
/// `s` must be null or a pointer returned by `sc_state_json` that has not been freed.
#[no_mangle]
pub unsafe extern "C" fn sc_free_string(s: *mut c_char) {
    if !s.is_null() {
        // SAFETY: the caller guarantees `s` came from `CString::into_raw` in this library.
        drop(unsafe { CString::from_raw(s) });
    }
}

/// Restore a patch from JSON. Returns 0 on success, 1 if the JSON is invalid (engine
/// unchanged), 2 if the state loaded but its sample bank did not (see `sc_last_error`).
///
/// # Safety
/// `e` must be null or a live engine pointer; `json` must be null or NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn sc_load_state_json(e: *mut ScEngine, json: *const c_char) -> c_int {
    // SAFETY: forwarded from the caller's contract.
    let Some(e) = (unsafe { eng(e) }) else {
        return 1;
    };
    if json.is_null() {
        e.set_error("null state");
        return 1;
    }
    // SAFETY: `json` is non-null and NUL-terminated per the caller's contract.
    let text = unsafe { CStr::from_ptr(json) }
        .to_string_lossy()
        .into_owned();
    match guard(Err("internal error".to_string()), || {
        e.inner
            .load_state_json(&text)
            .map_err(|err| err.to_string())
    }) {
        Ok(None) => 0,
        Ok(Some(warning)) => {
            e.set_error(&warning);
            2
        }
        Err(msg) => {
            e.set_error(&msg);
            1
        }
    }
}

/// Text of the last error, valid until the next call that can fail. Empty if none.
///
/// # Safety
/// `e` must be null or a live engine pointer. The result must not be freed.
#[no_mangle]
pub unsafe extern "C" fn sc_last_error(e: *mut ScEngine) -> *const c_char {
    // SAFETY: forwarded from the caller's contract.
    match unsafe { eng(e) } {
        Some(e) => e.last_error.as_ptr(),
        None => c"null engine".as_ptr(),
    }
}
