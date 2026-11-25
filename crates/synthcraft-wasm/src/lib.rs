//! WebAssembly (`wasm32-unknown-unknown`) AudioWorklet engine bridge for SynthCraft.
//!
//! Exports a minimal, bundler-free C ABI over WASM linear memory so any browser
//! `AudioWorkletProcessor` can instantiate `synthcraft_wasm.wasm` directly via
//! `WebAssembly.instantiate(bytes)` without `wasm-bindgen` or `TextDecoder`.

use synthcraft_core::params::PARAMS;
use synthcraft_core::{SynthEngine, SynthRt};

/// Opaque handle owning a live `SynthRt` instance plus pre-allocated stereo render buffers
/// inside WASM linear memory (so AudioWorklet `process()` is 100% allocation-free).
pub struct WasmSynth {
    rt: SynthRt,
    left: Vec<f32>,
    right: Vec<f32>,
}

/// Create a new SynthCraft WASM engine at `sample_rate` Hz with pre-allocated stereo buffers of
/// `block_frames` (typically `128` for Web Audio `AudioWorklet`).
#[no_mangle]
pub extern "C" fn scw_create(sample_rate: f32, block_frames: u32) -> *mut WasmSynth {
    let eng = SynthEngine::factory_seed();
    let patch = eng.active_patch().clone();
    let rt = SynthRt::new(sample_rate.max(8000.0), patch);
    let cap = (block_frames as usize).clamp(16, 4096);
    Box::into_raw(Box::new(WasmSynth {
        rt,
        left: vec![0.0; cap],
        right: vec![0.0; cap],
    }))
}

/// Free a `WasmSynth` created by [`scw_create`].
///
/// # Safety
/// `ptr` must be null or a pointer returned by [`scw_create`] that has not been freed yet.
#[no_mangle]
pub unsafe extern "C" fn scw_destroy(ptr: *mut WasmSynth) {
    if !ptr.is_null() {
        // SAFETY: Caller guarantees `ptr` came from `Box::into_raw` in `scw_create`.
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

/// Pointer to the left-channel render buffer in WASM linear memory.
///
/// # Safety
/// `ptr` must be a valid non-null `WasmSynth` handle.
#[no_mangle]
pub unsafe extern "C" fn scw_left_ptr(ptr: *mut WasmSynth) -> *const f32 {
    if ptr.is_null() {
        return core::ptr::null();
    }
    // SAFETY: Caller guarantees `ptr` is valid.
    unsafe { (*ptr).left.as_ptr() }
}

/// Pointer to the right-channel render buffer in WASM linear memory.
///
/// # Safety
/// `ptr` must be a valid non-null `WasmSynth` handle.
#[no_mangle]
pub unsafe extern "C" fn scw_right_ptr(ptr: *mut WasmSynth) -> *const f32 {
    if ptr.is_null() {
        return core::ptr::null();
    }
    // SAFETY: Caller guarantees `ptr` is valid.
    unsafe { (*ptr).right.as_ptr() }
}

/// Render `frames` stereo samples into the internal left/right buffers. Returns the number of
/// frames rendered (clamped to the allocated block size).
///
/// # Safety
/// `ptr` must be a valid non-null `WasmSynth` handle.
#[no_mangle]
pub unsafe extern "C" fn scw_process(ptr: *mut WasmSynth, frames: u32) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees exclusive access to `ptr`.
    let synth = unsafe { &mut *ptr };
    let n = (frames as usize)
        .min(synth.left.len())
        .min(synth.right.len());
    synth
        .rt
        .process_stereo(&mut synth.left[..n], &mut synth.right[..n]);
    n as u32
}

/// Trigger a MIDI note (`0..=127`) with velocity (`0.0..=1.0`).
///
/// # Safety
/// `ptr` must be a valid non-null `WasmSynth` handle.
#[no_mangle]
pub unsafe extern "C" fn scw_note_on(ptr: *mut WasmSynth, note: u8, velocity: f32) {
    // SAFETY: Caller guarantees `ptr` is valid or null.
    let synth = unsafe { ptr.as_mut() };
    if let Some(synth) = synth {
        synth.rt.note_on(note, velocity);
    }
}

/// Release a MIDI note (`0..=127`).
///
/// # Safety
/// `ptr` must be a valid non-null `WasmSynth` handle.
#[no_mangle]
pub unsafe extern "C" fn scw_note_off(ptr: *mut WasmSynth, note: u8) {
    // SAFETY: Caller guarantees `ptr` is valid or null.
    let synth = unsafe { ptr.as_mut() };
    if let Some(synth) = synth {
        synth.rt.note_off(note);
    }
}

/// Silence all voices immediately.
///
/// # Safety
/// `ptr` must be a valid non-null `WasmSynth` handle.
#[no_mangle]
pub unsafe extern "C" fn scw_all_notes_off(ptr: *mut WasmSynth) {
    // SAFETY: Caller guarantees `ptr` is valid or null.
    let synth = unsafe { ptr.as_mut() };
    if let Some(synth) = synth {
        synth.rt.all_notes_off();
    }
}

/// Start (`1`) or stop (`0`) the built-in 16-step sequencer.
///
/// # Safety
/// `ptr` must be a valid non-null `WasmSynth` handle.
#[no_mangle]
pub unsafe extern "C" fn scw_set_playing(ptr: *mut WasmSynth, playing: u32) {
    // SAFETY: Caller guarantees `ptr` is valid or null.
    let synth = unsafe { ptr.as_mut() };
    if let Some(synth) = synth {
        synth.rt.set_playing(playing != 0);
    }
}

/// Load a factory patch by index (`0..scw_patch_count()`). Returns `1` on success, `0` otherwise.
///
/// # Safety
/// `ptr` must be a valid non-null `WasmSynth` handle.
#[no_mangle]
pub unsafe extern "C" fn scw_load_patch_index(ptr: *mut WasmSynth, index: u32) -> u32 {
    // SAFETY: Caller guarantees `ptr` is valid or null.
    let Some(synth) = (unsafe { ptr.as_mut() }) else {
        return 0;
    };
    let eng = SynthEngine::factory_seed();
    if let Some(p) = eng.patches.get(index as usize) {
        let _ = synth.rt.replace_patch(p.clone());
        1
    } else {
        0
    }
}

/// Number of factory patches shipped in `synthcraft-core`.
#[no_mangle]
pub extern "C" fn scw_patch_count() -> u32 {
    SynthEngine::factory_seed().patches.len() as u32
}

/// Number of automatable engine parameters (`synthcraft_core::params::PARAMS`).
#[no_mangle]
pub extern "C" fn scw_param_count() -> u32 {
    PARAMS.len() as u32
}

/// Set an engine parameter by index (`0..scw_param_count()`) to its plain value.
///
/// # Safety
/// `ptr` must be a valid non-null `WasmSynth` handle.
#[no_mangle]
pub unsafe extern "C" fn scw_set_param(ptr: *mut WasmSynth, param_index: u32, value: f32) -> u32 {
    // SAFETY: Caller guarantees `ptr` is valid or null.
    let Some(synth) = (unsafe { ptr.as_mut() }) else {
        return 0;
    };
    let Some(spec) = PARAMS.get(param_index as usize) else {
        return 0;
    };
    if synthcraft_core::params::set_param(synth.rt.patch_mut(), spec.id, value).is_some() {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_c_abi_renders_audible_bounded_stereo_audio() {
        let ptr = scw_create(48000.0, 128);
        assert!(!ptr.is_null());
        // SAFETY: `ptr` was just created by `scw_create` and is destroyed at the end of the test.
        unsafe {
            assert!(scw_patch_count() >= 6);
            assert!(scw_param_count() >= 20);
            assert_eq!(scw_load_patch_index(ptr, 0), 1);
            assert_eq!(scw_set_param(ptr, 3, 2400.0), 1);
            scw_note_on(ptr, 60, 0.9);
            let mut peak = 0.0_f32;
            for _ in 0..8 {
                let n = scw_process(ptr, 128) as usize;
                assert_eq!(n, 128);
                let l = core::slice::from_raw_parts(scw_left_ptr(ptr), n);
                let r = core::slice::from_raw_parts(scw_right_ptr(ptr), n);
                for (&sl, &sr) in l.iter().zip(r.iter()) {
                    assert!(sl.is_finite() && (-1.0..=1.0).contains(&sl));
                    assert!(sr.is_finite() && (-1.0..=1.0).contains(&sr));
                    peak = peak.max(sl.abs()).max(sr.abs());
                }
            }
            assert!(peak > 0.05, "expected audible output, peak={peak}");
            scw_note_off(ptr, 60);
            scw_destroy(ptr);
        }
    }
}
