//! CLAP extensions: params, audio-ports, note-ports, state.

use crate::plugin::{this, N_PARAMS};
use clap_sys::events::{clap_input_events, clap_output_events};
use clap_sys::ext::audio_ports::{
    clap_audio_port_info, clap_plugin_audio_ports, CLAP_AUDIO_PORT_IS_MAIN, CLAP_EXT_AUDIO_PORTS,
    CLAP_PORT_STEREO,
};
use clap_sys::ext::note_ports::{
    clap_note_port_info, clap_plugin_note_ports, CLAP_EXT_NOTE_PORTS, CLAP_NOTE_DIALECT_CLAP,
    CLAP_NOTE_DIALECT_MIDI,
};
use clap_sys::ext::params::{
    clap_param_info, clap_plugin_params, CLAP_EXT_PARAMS, CLAP_PARAM_IS_AUTOMATABLE,
};
use clap_sys::ext::state::{clap_plugin_state, CLAP_EXT_STATE};
use clap_sys::id::clap_id;
use clap_sys::plugin::clap_plugin;
use clap_sys::stream::{clap_istream, clap_ostream};
use std::ffi::{c_char, c_void, CStr};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::Ordering;
use synthcraft_core::params::{param_def, PARAMS};

/// Largest state we accept, to bound what a hostile or corrupt project can make us read.
const MAX_STATE_BYTES: usize = 4 * 1024 * 1024;

static PARAMS_EXT: clap_plugin_params = clap_plugin_params {
    count: Some(params_count),
    get_info: Some(params_get_info),
    get_value: Some(params_get_value),
    value_to_text: Some(params_value_to_text),
    text_to_value: Some(params_text_to_value),
    flush: Some(params_flush),
};

static AUDIO_PORTS_EXT: clap_plugin_audio_ports = clap_plugin_audio_ports {
    count: Some(ports_count),
    get: Some(ports_get),
};

static NOTE_PORTS_EXT: clap_plugin_note_ports = clap_plugin_note_ports {
    count: Some(note_ports_count),
    get: Some(note_ports_get),
};

static STATE_EXT: clap_plugin_state = clap_plugin_state {
    save: Some(state_save),
    load: Some(state_load),
};

/// The extension table for `get_extension`.
pub(crate) fn lookup(id: &CStr) -> *const c_void {
    if id == CLAP_EXT_PARAMS {
        std::ptr::from_ref(&PARAMS_EXT).cast()
    } else if id == CLAP_EXT_AUDIO_PORTS {
        std::ptr::from_ref(&AUDIO_PORTS_EXT).cast()
    } else if id == CLAP_EXT_NOTE_PORTS {
        std::ptr::from_ref(&NOTE_PORTS_EXT).cast()
    } else if id == CLAP_EXT_STATE {
        std::ptr::from_ref(&STATE_EXT).cast()
    } else {
        std::ptr::null()
    }
}

fn copy_str<const N: usize>(dst: &mut [c_char; N], src: &str) {
    dst.fill(0);
    for (d, b) in dst.iter_mut().zip(src.bytes().take(N - 1)) {
        *d = b as c_char;
    }
}

unsafe extern "C" fn params_count(_p: *const clap_plugin) -> u32 {
    N_PARAMS as u32
}

unsafe extern "C" fn params_get_info(
    _p: *const clap_plugin,
    index: u32,
    info: *mut clap_param_info,
) -> bool {
    let Some(d) = PARAMS.get(index as usize) else {
        return false;
    };
    if info.is_null() {
        return false;
    }
    // SAFETY: `info` is non-null and the host provides writable storage for one struct.
    let info = unsafe { &mut *info };
    info.id = d.id;
    info.flags = CLAP_PARAM_IS_AUTOMATABLE;
    info.cookie = std::ptr::null_mut();
    copy_str(&mut info.name, d.name);
    copy_str(&mut info.module, d.module);
    info.min_value = f64::from(d.min);
    info.max_value = f64::from(d.max);
    info.default_value = f64::from(d.default);
    true
}

unsafe extern "C" fn params_get_value(p: *const clap_plugin, id: clap_id, out: *mut f64) -> bool {
    // SAFETY: `p` is one of ours; `out` is checked for null.
    let (Some(pl), Some(i)) = (unsafe { this(p) }, PARAMS.iter().position(|d| d.id == id)) else {
        return false;
    };
    if out.is_null() {
        return false;
    }
    let v = f32::from_bits(pl.values[i].load(Ordering::Relaxed));
    // SAFETY: `out` is non-null and writable per the CLAP contract.
    unsafe { *out = f64::from(v) };
    true
}

unsafe extern "C" fn params_value_to_text(
    _p: *const clap_plugin,
    id: clap_id,
    value: f64,
    buf: *mut c_char,
    cap: u32,
) -> bool {
    if param_def(id).is_none() || buf.is_null() || cap == 0 {
        return false;
    }
    let text = if value.abs() >= 100.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    };
    // SAFETY: the host provides `cap` writable bytes at `buf`.
    let dst = unsafe { std::slice::from_raw_parts_mut(buf.cast::<u8>(), cap as usize) };
    let n = text.len().min(dst.len() - 1);
    dst[..n].copy_from_slice(&text.as_bytes()[..n]);
    dst[n] = 0;
    true
}

unsafe extern "C" fn params_text_to_value(
    _p: *const clap_plugin,
    id: clap_id,
    text: *const c_char,
    out: *mut f64,
) -> bool {
    if param_def(id).is_none() || text.is_null() || out.is_null() {
        return false;
    }
    // SAFETY: NUL-terminated text from the host; null handled above.
    let s = unsafe { CStr::from_ptr(text) }.to_string_lossy();
    match s.trim().parse::<f64>() {
        Ok(v) if v.is_finite() => {
            // SAFETY: `out` is non-null and writable.
            unsafe { *out = v };
            true
        }
        _ => false,
    }
}

unsafe extern "C" fn params_flush(
    p: *const clap_plugin,
    input: *const clap_input_events,
    _output: *const clap_output_events,
) {
    // SAFETY: `p` and `input` come from the host and are valid for this call.
    let (Some(pl), Some(list)) = (unsafe { this(p) }, unsafe { input.as_ref() }) else {
        return;
    };
    let Ok(mut guard) = pl.engine.try_lock() else {
        return;
    };
    // SAFETY: calling the host's own callbacks with its own list.
    let n = list.size.map_or(0, |f| unsafe { f(list) });
    for i in 0..n {
        let Some(get) = list.get else { break };
        // SAFETY: `i < size`.
        if let Some(h) = unsafe { get(list, i).as_ref() } {
            pl.handle_event_pub(&mut guard, h);
        }
    }
}

unsafe extern "C" fn ports_count(_p: *const clap_plugin, is_input: bool) -> u32 {
    u32::from(!is_input)
}

unsafe extern "C" fn ports_get(
    _p: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool {
    if is_input || index != 0 || info.is_null() {
        return false;
    }
    // SAFETY: `info` is non-null and writable.
    let info = unsafe { &mut *info };
    info.id = 0;
    copy_str(&mut info.name, "Main");
    info.flags = CLAP_AUDIO_PORT_IS_MAIN;
    info.channel_count = 2;
    info.port_type = CLAP_PORT_STEREO.as_ptr();
    info.in_place_pair = u32::MAX;
    true
}

unsafe extern "C" fn note_ports_count(_p: *const clap_plugin, is_input: bool) -> u32 {
    u32::from(is_input)
}

unsafe extern "C" fn note_ports_get(
    _p: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_note_port_info,
) -> bool {
    if !is_input || index != 0 || info.is_null() {
        return false;
    }
    // SAFETY: `info` is non-null and writable.
    let info = unsafe { &mut *info };
    info.id = 0;
    info.supported_dialects = CLAP_NOTE_DIALECT_CLAP | CLAP_NOTE_DIALECT_MIDI;
    info.preferred_dialect = CLAP_NOTE_DIALECT_CLAP;
    copy_str(&mut info.name, "Notes");
    true
}

unsafe extern "C" fn state_save(p: *const clap_plugin, stream: *const clap_ostream) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: `p` and `stream` come from the host and are valid for this call.
        let (Some(pl), Some(st)) = (unsafe { this(p) }, unsafe { stream.as_ref() }) else {
            return false;
        };
        let Some(write) = st.write else {
            return false;
        };
        let Ok(json) = pl.engine.lock().map(|e| e.state_json()) else {
            return false;
        };
        let Ok(json) = json else {
            return false;
        };
        let bytes = json.as_bytes();
        let mut off = 0;
        while off < bytes.len() {
            // SAFETY: the slice is valid for the length passed.
            let n = unsafe {
                write(
                    stream,
                    bytes[off..].as_ptr().cast(),
                    (bytes.len() - off) as u64,
                )
            };
            if n <= 0 {
                return false;
            }
            off += n as usize;
        }
        true
    }))
    .unwrap_or(false)
}

unsafe extern "C" fn state_load(p: *const clap_plugin, stream: *const clap_istream) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: `p` and `stream` come from the host and are valid for this call.
        let (Some(pl), Some(st)) = (unsafe { this(p) }, unsafe { stream.as_ref() }) else {
            return false;
        };
        let Some(read) = st.read else {
            return false;
        };
        let mut data: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            // SAFETY: `chunk` is writable for its full length.
            let n = unsafe { read(stream, chunk.as_mut_ptr().cast(), chunk.len() as u64) };
            if n < 0 {
                return false;
            }
            if n == 0 {
                break;
            }
            data.extend_from_slice(&chunk[..n as usize]);
            if data.len() > MAX_STATE_BYTES {
                return false;
            }
        }
        let Ok(text) = String::from_utf8(data) else {
            return false;
        };
        let Ok(mut e) = pl.engine.lock() else {
            return false;
        };
        if e.load_state_json(&text).is_err() {
            return false;
        }
        pl.sync_values(&e);
        drop(e);
        pl.request_value_rescan();
        true
    }))
    .unwrap_or(false)
}
