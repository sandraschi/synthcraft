//! Descriptor, factory and the plug-in object with its audio processing.

use crate::bridge::{GuiBridge, UiEvent};
use crate::ext;
use crate::gui::GuiInstance;
use clap_sys::events::{
    clap_event_header, clap_event_midi, clap_event_note, clap_event_note_expression,
    clap_event_param_gesture, clap_event_param_mod, clap_event_param_value,
    CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, CLAP_EVENT_NOTE_CHOKE, CLAP_EVENT_NOTE_EXPRESSION,
    CLAP_EVENT_NOTE_OFF, CLAP_EVENT_NOTE_ON, CLAP_EVENT_PARAM_GESTURE_BEGIN,
    CLAP_EVENT_PARAM_GESTURE_END, CLAP_EVENT_PARAM_MOD, CLAP_EVENT_PARAM_VALUE,
    CLAP_NOTE_EXPRESSION_BRIGHTNESS, CLAP_NOTE_EXPRESSION_PRESSURE, CLAP_NOTE_EXPRESSION_TUNING,
};
use clap_sys::ext::params::{clap_host_params, CLAP_EXT_PARAMS, CLAP_PARAM_RESCAN_VALUES};
use clap_sys::factory::plugin_factory::clap_plugin_factory;
use clap_sys::host::clap_host;
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
use clap_sys::plugin_features::{
    CLAP_PLUGIN_FEATURE_INSTRUMENT, CLAP_PLUGIN_FEATURE_STEREO, CLAP_PLUGIN_FEATURE_SYNTHESIZER,
};
use clap_sys::process::{clap_process, CLAP_PROCESS_CONTINUE, CLAP_PROCESS_ERROR};
use clap_sys::version::CLAP_VERSION;
use std::ffi::{c_char, c_void, CStr};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use synthcraft_core::plugin::PluginEngine;

/// Plug-in id reported to hosts.
pub const ENTRY_ID: &CStr = c"dev.sandraschi.synthcraft";

struct Features([*const c_char; 4]);
// SAFETY: the pointers refer to immutable static C string literals.
unsafe impl Sync for Features {}

static FEATURES: Features = Features([
    CLAP_PLUGIN_FEATURE_INSTRUMENT.as_ptr(),
    CLAP_PLUGIN_FEATURE_SYNTHESIZER.as_ptr(),
    CLAP_PLUGIN_FEATURE_STEREO.as_ptr(),
    std::ptr::null(),
]);

pub(crate) static DESCRIPTOR: clap_plugin_descriptor = clap_plugin_descriptor {
    clap_version: CLAP_VERSION,
    id: ENTRY_ID.as_ptr(),
    name: c"SynthCraft".as_ptr(),
    vendor: c"Sandra Schipal".as_ptr(),
    url: c"https://github.com/sandraschi/synthcraft".as_ptr(),
    manual_url: c"https://github.com/sandraschi/synthcraft".as_ptr(),
    support_url: c"https://github.com/sandraschi/synthcraft/issues".as_ptr(),
    version: c"0.2.0".as_ptr(),
    description: c"Polyphonic subtractive, PCM and sample-bank synthesizer".as_ptr(),
    features: FEATURES.0.as_ptr(),
};

pub(crate) static FACTORY: clap_plugin_factory = clap_plugin_factory {
    get_plugin_count: Some(factory_count),
    get_plugin_descriptor: Some(factory_descriptor),
    create_plugin: Some(factory_create),
};

unsafe extern "C" fn factory_count(_f: *const clap_plugin_factory) -> u32 {
    1
}

unsafe extern "C" fn factory_descriptor(
    _f: *const clap_plugin_factory,
    index: u32,
) -> *const clap_plugin_descriptor {
    if index == 0 {
        &DESCRIPTOR
    } else {
        std::ptr::null()
    }
}

unsafe extern "C" fn factory_create(
    _f: *const clap_plugin_factory,
    host: *const clap_host,
    plugin_id: *const c_char,
) -> *const clap_plugin {
    if plugin_id.is_null() {
        return std::ptr::null();
    }
    // SAFETY: the host passes a NUL-terminated id; null was handled above.
    if unsafe { CStr::from_ptr(plugin_id) } != ENTRY_ID {
        return std::ptr::null();
    }
    Plugin::create(host)
}

/// The plug-in instance behind `clap_plugin.plugin_data`.
pub(crate) struct Plugin {
    clap: clap_plugin,
    /// The host that created us; used to request a parameter rescan after state loads.
    host: *const clap_host,
    pub(crate) bridge: Arc<GuiBridge>,
    pub(crate) gui: Mutex<Option<GuiInstance>>,
}

impl Plugin {
    fn create(host: *const clap_host) -> *const clap_plugin {
        let boxed = Box::new(Plugin {
            clap: clap_plugin {
                desc: &DESCRIPTOR,
                plugin_data: std::ptr::null_mut(),
                init: Some(p_init),
                destroy: Some(p_destroy),
                activate: Some(p_activate),
                deactivate: Some(p_deactivate),
                start_processing: Some(p_start),
                stop_processing: Some(p_stop),
                reset: Some(p_reset),
                process: Some(p_process),
                get_extension: Some(p_get_extension),
                on_main_thread: Some(p_main_thread),
            },
            host,
            bridge: Arc::new(GuiBridge::new(48_000.0)),
            gui: Mutex::new(None),
        });
        let raw = Box::into_raw(boxed);
        // SAFETY: `raw` was just produced by `Box::into_raw`, so it is valid and unique here.
        unsafe {
            (*raw).clap.plugin_data = raw.cast();
            std::ptr::addr_of!((*raw).clap)
        }
    }

    /// Tell the host that parameter values changed outside its knowledge (after a state
    /// load), as CLAP requires. Main thread only. A missing host or extension is ignored.
    pub(crate) fn request_value_rescan(&self) {
        // SAFETY: `host` is the pointer the host gave `create_plugin`; CLAP keeps it valid for
        // the life of the plug-in. Null is handled.
        let Some(host) = (unsafe { self.host.as_ref() }) else {
            return;
        };
        let Some(get_ext) = host.get_extension else {
            return;
        };
        // SAFETY: calling the host's own callback with its own pointer.
        let ext = unsafe { get_ext(self.host, CLAP_EXT_PARAMS.as_ptr()) };
        // SAFETY: a non-null result is the host's `clap_host_params` table.
        if let Some(hp) = unsafe { ext.cast::<clap_host_params>().as_ref() } {
            if let Some(rescan) = hp.rescan {
                // SAFETY: as above.
                unsafe { rescan(self.host, CLAP_PARAM_RESCAN_VALUES) };
            }
        }
    }

    /// Mirror the engine's parameter values into the atomics (after a state load).
    pub(crate) fn sync_values(&self, engine: &PluginEngine) {
        self.bridge.sync_values(engine);
    }

    fn set_value(&self, engine: &mut PluginEngine, id: u32, value: f64) {
        if let Some(v) = engine.set_param(id, value as f32) {
            self.bridge.store_value_by_id(id, v);
        }
    }

    /// Apply one event at its time position.
    pub(crate) fn handle_event_pub(&self, engine: &mut PluginEngine, h: &clap_event_header) {
        self.handle_event(engine, h);
    }

    fn handle_event(&self, engine: &mut PluginEngine, h: &clap_event_header) {
        if h.space_id != CLAP_CORE_EVENT_SPACE_ID {
            return;
        }
        match h.type_ {
            CLAP_EVENT_NOTE_ON | CLAP_EVENT_NOTE_OFF | CLAP_EVENT_NOTE_CHOKE => {
                if (h.size as usize) < std::mem::size_of::<clap_event_note>() {
                    return;
                }
                // SAFETY: type and size were checked, and CLAP guarantees the event bytes are
                // a `clap_event_note` for these types.
                let ev = unsafe { &*std::ptr::from_ref(h).cast::<clap_event_note>() };
                let Ok(key) = u8::try_from(ev.key) else {
                    // key -1 is a wildcard: only "all notes off" makes sense for it
                    if h.type_ != CLAP_EVENT_NOTE_ON {
                        engine.all_notes_off();
                    }
                    return;
                };
                if h.type_ == CLAP_EVENT_NOTE_ON {
                    engine.note_on(key, ev.velocity as f32);
                } else {
                    engine.note_off(key);
                }
            }
            CLAP_EVENT_PARAM_VALUE => {
                if (h.size as usize) < std::mem::size_of::<clap_event_param_value>() {
                    return;
                }
                // SAFETY: type and size were checked.
                let ev = unsafe { &*std::ptr::from_ref(h).cast::<clap_event_param_value>() };
                self.set_value(engine, ev.param_id, ev.value);
            }
            CLAP_EVENT_PARAM_MOD => {
                if (h.size as usize) < std::mem::size_of::<clap_event_param_mod>() {
                    return;
                }
                // SAFETY: type and size were checked.
                let ev = unsafe { &*std::ptr::from_ref(h).cast::<clap_event_param_mod>() };
                let note = u8::try_from(ev.key).ok();
                engine.set_poly_param_mod(note, ev.param_id, ev.amount as f32);
            }
            CLAP_EVENT_NOTE_EXPRESSION => {
                if (h.size as usize) < std::mem::size_of::<clap_event_note_expression>() {
                    return;
                }
                // SAFETY: type and size were checked.
                let ev = unsafe { &*std::ptr::from_ref(h).cast::<clap_event_note_expression>() };
                let note = u8::try_from(ev.key).ok();
                let val = ev.value as f32;
                match ev.expression_id {
                    CLAP_NOTE_EXPRESSION_TUNING => {
                        engine.set_note_expression(note, Some(val), None, None)
                    }
                    CLAP_NOTE_EXPRESSION_BRIGHTNESS => {
                        engine.set_note_expression(note, None, Some(val), None)
                    }
                    CLAP_NOTE_EXPRESSION_PRESSURE => {
                        engine.set_note_expression(note, None, None, Some(val))
                    }
                    _ => {}
                }
            }
            CLAP_EVENT_MIDI => {
                if (h.size as usize) < std::mem::size_of::<clap_event_midi>() {
                    return;
                }
                // SAFETY: type and size were checked.
                let ev = unsafe { &*std::ptr::from_ref(h).cast::<clap_event_midi>() };
                handle_midi(engine, ev.data);
            }
            _ => {}
        }
    }
}

fn handle_midi(engine: &mut PluginEngine, d: [u8; 3]) {
    let (status, a, b) = (d[0] & 0xF0, d[1] & 0x7F, d[2] & 0x7F);
    match status {
        0x90 if b > 0 => engine.note_on(a, f32::from(b) / 127.0),
        0x90 | 0x80 => engine.note_off(a),
        0xA0 => engine.set_note_expression(Some(a), None, None, Some(f32::from(b) / 127.0)),
        0xB0 if a == 1 => engine.mod_wheel(f32::from(b) / 127.0),
        0xB0 if a == 74 => engine.set_note_expression(None, None, Some(f32::from(b) / 127.0), None),
        0xB0 if a == 123 || a == 120 => engine.all_notes_off(),
        0xD0 => engine.set_note_expression(None, None, None, Some(f32::from(a) / 127.0)),
        0xE0 => {
            let v = i32::from(a) | (i32::from(b) << 7);
            engine.pitch_bend((v - 8192) as f32 / 8192.0 * 2.0);
        }
        _ => {}
    }
}

/// The plug-in object behind a `clap_plugin` pointer.
///
/// # Safety
/// `p` must be a pointer this module returned from `factory_create` that is not destroyed.
pub(crate) unsafe fn this<'a>(p: *const clap_plugin) -> Option<&'a Plugin> {
    // SAFETY: forwarded from the caller; `plugin_data` is the `Plugin` that owns this struct.
    unsafe { p.as_ref() }.and_then(|c| unsafe { c.plugin_data.cast::<Plugin>().as_ref() })
}

unsafe extern "C" fn p_init(_p: *const clap_plugin) -> bool {
    true
}

unsafe extern "C" fn p_destroy(p: *const clap_plugin) {
    // SAFETY: `p` is ours; `plugin_data` was set to the leaked box in `create`.
    if let Some(c) = unsafe { p.as_ref() } {
        let data = c.plugin_data.cast::<Plugin>();
        if !data.is_null() {
            // SAFETY: the host calls destroy once; this reclaims the box made in `create`.
            drop(unsafe { Box::from_raw(data) });
        }
    }
}

unsafe extern "C" fn p_activate(
    p: *const clap_plugin,
    sample_rate: f64,
    _min_frames: u32,
    _max_frames: u32,
) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: `p` comes from the host and is one of ours.
        let Some(pl) = (unsafe { this(p) }) else {
            return false;
        };
        let Ok(mut e) = pl.bridge.engine.lock() else {
            return false;
        };
        e.set_sample_rate(sample_rate as f32);
        if !e.has_bank() {
            if let Ok(spec) = std::env::var("SYNTHCRAFT_BANK") {
                // A bank that fails to load leaves the plain synth; the host has no channel
                // for the message.
                let _ = e.load_bank(&spec);
            }
        }
        true
    }))
    .unwrap_or(false)
}

unsafe extern "C" fn p_deactivate(_p: *const clap_plugin) {}

unsafe extern "C" fn p_start(_p: *const clap_plugin) -> bool {
    true
}

unsafe extern "C" fn p_stop(_p: *const clap_plugin) {}

unsafe extern "C" fn p_reset(p: *const clap_plugin) {
    // SAFETY: `p` comes from the host and is one of ours.
    if let Some(pl) = unsafe { this(p) } {
        if let Ok(mut e) = pl.bridge.engine.lock() {
            e.all_notes_off();
        }
    }
}

unsafe extern "C" fn p_main_thread(_p: *const clap_plugin) {}

unsafe extern "C" fn p_get_extension(_p: *const clap_plugin, id: *const c_char) -> *const c_void {
    if id.is_null() {
        return std::ptr::null();
    }
    // SAFETY: NUL-terminated extension id from the host; null handled above.
    ext::lookup(unsafe { CStr::from_ptr(id) })
}

unsafe extern "C" fn p_process(p: *const clap_plugin, process: *const clap_process) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: `p` and `process` come from the host for the duration of this call.
        let (Some(pl), Some(pr)) = (unsafe { this(p) }, unsafe { process.as_ref() }) else {
            return CLAP_PROCESS_ERROR;
        };
        // SAFETY: the host guarantees the event lists and buffers in `pr` are valid here.
        unsafe { render(pl, pr) }
    }))
    .unwrap_or(CLAP_PROCESS_ERROR)
}

/// Process one block: interleave event handling with rendering at sample-accurate positions.
///
/// # Safety
/// The pointers inside `pr` must be valid as the CLAP process contract requires.
unsafe fn render(pl: &Plugin, pr: &clap_process) -> i32 {
    let frames = pr.frames_count as usize;
    if pr.audio_outputs.is_null() || pr.audio_outputs_count == 0 {
        return CLAP_PROCESS_ERROR;
    }
    // SAFETY: at least one output buffer exists (checked above).
    let out = unsafe { &*pr.audio_outputs };
    if out.data32.is_null() || out.channel_count == 0 {
        return CLAP_PROCESS_ERROR;
    }
    // SAFETY: `data32` holds `channel_count` channel pointers.
    let ch0 = unsafe { *out.data32 };
    let ch1 = if out.channel_count > 1 {
        // SAFETY: channel_count > 1, so index 1 is in range.
        unsafe { *out.data32.add(1) }
    } else {
        std::ptr::null_mut()
    };
    // The declared port layout is stereo, so a host must hand us two distinct channels.
    if ch0.is_null() || ch1.is_null() || ch0 == ch1 {
        return CLAP_PROCESS_ERROR;
    }
    // SAFETY: the host provides `frames` writable floats in each of two distinct channels.
    let (left, right) = unsafe {
        (
            std::slice::from_raw_parts_mut(ch0, frames),
            std::slice::from_raw_parts_mut(ch1, frames),
        )
    };
    left.fill(0.0);
    right.fill(0.0);

    let Ok(mut guard) = pl.bridge.engine.try_lock() else {
        return CLAP_PROCESS_CONTINUE; // engine busy on the main thread: one silent block
    };
    let engine = &mut *guard;

    // Drain any parameter gestures or audition notes queued by the embedded GUI.
    let mut ui_ev = Vec::new();
    pl.bridge.drain_ui_events_try(&mut ui_ev);
    // SAFETY: host guarantees `out_events` is valid if non-null during `process`.
    let out_events = unsafe { pr.out_events.as_ref() };
    for ev in ui_ev {
        match ev {
            UiEvent::ParamGestureBegin(id) => {
                if let Some(list) = out_events {
                    if let Some(push) = list.try_push {
                        let msg = clap_event_param_gesture {
                            header: clap_event_header {
                                size: std::mem::size_of::<clap_event_param_gesture>() as u32,
                                time: 0,
                                space_id: CLAP_CORE_EVENT_SPACE_ID,
                                type_: CLAP_EVENT_PARAM_GESTURE_BEGIN,
                                flags: 0,
                            },
                            param_id: id,
                        };
                        // SAFETY: `msg` is a valid `clap_event_param_gesture` with matching header size.
                        unsafe {
                            push(list, std::ptr::addr_of!(msg).cast());
                        }
                    }
                }
            }
            UiEvent::ParamValue(id, val) => {
                pl.set_value(engine, id, f64::from(val));
                if let Some(list) = out_events {
                    if let Some(push) = list.try_push {
                        let msg = clap_event_param_value {
                            header: clap_event_header {
                                size: std::mem::size_of::<clap_event_param_value>() as u32,
                                time: 0,
                                space_id: CLAP_CORE_EVENT_SPACE_ID,
                                type_: CLAP_EVENT_PARAM_VALUE,
                                flags: 0,
                            },
                            param_id: id,
                            cookie: std::ptr::null_mut(),
                            note_id: -1,
                            port_index: -1,
                            channel: -1,
                            key: -1,
                            value: f64::from(val),
                        };
                        // SAFETY: `msg` is a valid `clap_event_param_value` with matching header size.
                        unsafe {
                            push(list, std::ptr::addr_of!(msg).cast());
                        }
                    }
                }
            }
            UiEvent::ParamGestureEnd(id) => {
                if let Some(list) = out_events {
                    if let Some(push) = list.try_push {
                        let msg = clap_event_param_gesture {
                            header: clap_event_header {
                                size: std::mem::size_of::<clap_event_param_gesture>() as u32,
                                time: 0,
                                space_id: CLAP_CORE_EVENT_SPACE_ID,
                                type_: CLAP_EVENT_PARAM_GESTURE_END,
                                flags: 0,
                            },
                            param_id: id,
                        };
                        // SAFETY: `msg` is a valid `clap_event_param_gesture` with matching header size.
                        unsafe {
                            push(list, std::ptr::addr_of!(msg).cast());
                        }
                    }
                }
            }
            UiEvent::NoteOn(note, vel) => engine.note_on(note, vel),
            UiEvent::NoteOff(note) => engine.note_off(note),
        }
    }

    let mut cursor = 0usize;
    // SAFETY: the host guarantees `in_events` and its callbacks are valid.
    let events = unsafe { pr.in_events.as_ref() };
    if let Some(list) = events {
        // SAFETY: calling the host's own callbacks with the list it handed us.
        let n = list.size.map_or(0, |f| unsafe { f(list) });
        let get = list.get;
        for i in 0..n {
            let Some(get) = get else { break };
            // SAFETY: `i < size`, so the host returns a valid event header.
            let Some(h) = (unsafe { get(list, i).as_ref() }) else {
                continue;
            };
            let at = (h.time as usize).min(frames);
            if at > cursor {
                engine.process(&mut left[cursor..at], &mut right[cursor..at]);
                cursor = at;
            }
            pl.handle_event(engine, h);
        }
    }
    if cursor < frames {
        engine.process(&mut left[cursor..], &mut right[cursor..]);
    }
    pl.bridge.update_meters(left, right, engine.active_voices());
    CLAP_PROCESS_CONTINUE
}

/// Create a plug-in instance the way a host would, for tests.
pub fn create_for_test() -> *const clap_plugin {
    Plugin::create(std::ptr::null())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{clap_entry, ext};
    use clap_sys::audio_buffer::clap_audio_buffer;
    use clap_sys::events::clap_input_events;
    use clap_sys::ext::gui::{clap_plugin_gui, CLAP_EXT_GUI, CLAP_WINDOW_API_WIN32};
    use clap_sys::ext::params::{clap_param_info, clap_plugin_params, CLAP_EXT_PARAMS};
    use clap_sys::ext::state::{clap_plugin_state, CLAP_EXT_STATE};
    use clap_sys::factory::plugin_factory::CLAP_PLUGIN_FACTORY_ID;
    use clap_sys::stream::{clap_istream, clap_ostream};
    use synthcraft_core::params::PARAMS;

    struct Events {
        notes: Vec<clap_event_note>,
        params: Vec<clap_event_param_value>,
    }

    unsafe extern "C" fn ev_size(l: *const clap_input_events) -> u32 {
        // SAFETY: ctx points at the `Events` owned by the test for the whole call.
        let e = unsafe { &*(*l).ctx.cast::<Events>() };
        (e.notes.len() + e.params.len()) as u32
    }

    unsafe extern "C" fn ev_get(l: *const clap_input_events, i: u32) -> *const clap_event_header {
        // SAFETY: as above.
        let e = unsafe { &*(*l).ctx.cast::<Events>() };
        let i = i as usize;
        if i < e.notes.len() {
            &e.notes[i].header
        } else {
            &e.params[i - e.notes.len()].header
        }
    }

    fn header(size: usize, time: u32, type_: u16) -> clap_event_header {
        clap_event_header {
            size: size as u32,
            time,
            space_id: CLAP_CORE_EVENT_SPACE_ID,
            type_,
            flags: 0,
        }
    }

    fn note_event(time: u32, key: i16, ty: u16) -> clap_event_note {
        clap_event_note {
            header: header(std::mem::size_of::<clap_event_note>(), time, ty),
            note_id: -1,
            port_index: 0,
            channel: 0,
            key,
            velocity: 0.9,
        }
    }

    fn run_block(
        p: *const clap_plugin,
        frames: usize,
        events: &Events,
    ) -> (Vec<f32>, Vec<f32>, i32) {
        let (mut l, mut r) = (vec![0.5_f32; frames], vec![0.5_f32; frames]);
        let mut chans = [l.as_mut_ptr(), r.as_mut_ptr()];
        let mut out = clap_audio_buffer {
            data32: chans.as_mut_ptr(),
            data64: std::ptr::null_mut(),
            channel_count: 2,
            latency: 0,
            constant_mask: 0,
        };
        let list = clap_input_events {
            ctx: std::ptr::from_ref(events).cast_mut().cast(),
            size: Some(ev_size),
            get: Some(ev_get),
        };
        let pr = clap_process {
            steady_time: 0,
            frames_count: frames as u32,
            transport: std::ptr::null(),
            audio_inputs: std::ptr::null(),
            audio_outputs: &mut out,
            audio_inputs_count: 0,
            audio_outputs_count: 1,
            in_events: &list,
            out_events: std::ptr::null(),
        };
        // SAFETY: `p` is a live plug-in and `pr` points at valid local buffers.
        let status = unsafe { (*p).process.map_or(CLAP_PROCESS_ERROR, |f| f(p, &pr)) };
        (l, r, status)
    }

    fn energy(v: &[f32]) -> f32 {
        v.iter().map(|x| x * x).sum()
    }

    fn make() -> *const clap_plugin {
        // SAFETY: calling our own entry points the way a host does.
        unsafe {
            assert!(clap_entry.init.is_some_and(|f| f(std::ptr::null())));
            let get = clap_entry
                .get_factory
                .map(|f| f(CLAP_PLUGIN_FACTORY_ID.as_ptr()));
            let fac = get
                .unwrap_or(std::ptr::null())
                .cast::<clap_plugin_factory>();
            assert!(!fac.is_null());
            let f = &*fac;
            assert_eq!(f.get_plugin_count.map(|c| c(fac)), Some(1));
            let d = f
                .get_plugin_descriptor
                .map(|c| c(fac, 0))
                .unwrap_or(std::ptr::null());
            assert_eq!(CStr::from_ptr((*d).id), ENTRY_ID);
            assert!(f
                .get_plugin_descriptor
                .map(|c| c(fac, 1))
                .is_some_and(<*const _>::is_null));
            let p = f
                .create_plugin
                .map(|c| c(fac, std::ptr::null(), ENTRY_ID.as_ptr()))
                .unwrap_or(std::ptr::null());
            assert!(!p.is_null());
            assert!(f
                .create_plugin
                .map(|c| c(fac, std::ptr::null(), c"other.id".as_ptr()))
                .is_some_and(<*const _>::is_null));
            assert!((*p).init.is_some_and(|i| i(p)));
            assert!((*p).activate.is_some_and(|a| a(p, 48_000.0, 16, 4096)));
            p
        }
    }

    fn destroy(p: *const clap_plugin) {
        // SAFETY: `p` is live and destroyed exactly once.
        unsafe {
            if let Some(d) = (*p).destroy {
                d(p);
            }
        }
    }

    #[test]
    fn enumerates_all_parameters_with_stable_ids() {
        let p = make();
        // SAFETY: `p` is live; the extension pointer is our static table.
        unsafe {
            let ext = (*p).get_extension.map(|g| g(p, CLAP_EXT_PARAMS.as_ptr()));
            let params = &*ext.unwrap_or(std::ptr::null()).cast::<clap_plugin_params>();
            let n = params.count.map_or(0, |c| c(p));
            assert_eq!(n as usize, PARAMS.len());
            let mut ids = Vec::new();
            for i in 0..n {
                let mut info = std::mem::zeroed::<clap_param_info>();
                assert!(params.get_info.is_some_and(|g| g(p, i, &mut info)));
                assert!(info.min_value < info.max_value);
                ids.push(info.id);
                let mut v = 0.0;
                assert!(params.get_value.is_some_and(|g| g(p, info.id, &mut v)));
                assert!((v - info.default_value).abs() < 1e-6);
            }
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), PARAMS.len());
            let mut info = std::mem::zeroed::<clap_param_info>();
            assert!(!params.get_info.is_some_and(|g| g(p, n, &mut info)));
            assert!((*p)
                .get_extension
                .is_some_and(|g| g(p, c"clap.nope".as_ptr()).is_null()));
            let ports = (*p)
                .get_extension
                .map(|g| g(p, c"clap.audio-ports".as_ptr()));
            assert!(!ports.unwrap_or(std::ptr::null()).is_null());
        }
        destroy(p);
    }

    #[test]
    fn notes_are_sample_accurate_and_params_apply() {
        let p = make();
        let silent = Events {
            notes: vec![],
            params: vec![],
        };
        let (l, r, st) = run_block(p, 512, &silent);
        assert_eq!(st, CLAP_PROCESS_CONTINUE);
        assert!(
            energy(&l) < 1e-9 && energy(&r) < 1e-9,
            "must overwrite the host buffer"
        );

        let ev = Events {
            notes: vec![note_event(200, 60, CLAP_EVENT_NOTE_ON)],
            params: vec![clap_event_param_value {
                header: header(
                    std::mem::size_of::<clap_event_param_value>(),
                    0,
                    CLAP_EVENT_PARAM_VALUE,
                ),
                param_id: 0x1004,
                cookie: std::ptr::null_mut(),
                note_id: -1,
                port_index: 0,
                channel: 0,
                key: -1,
                value: 777.0,
            }],
        };
        let (l, r, _) = run_block(p, 2048, &ev);
        assert!(energy(&l[..200]) < 1e-9, "silent before the note event");
        assert!(energy(&l[200..]) > 0.01, "sounds from the event onward");
        assert!(energy(&r[200..]) > 0.01, "right channel sounds too");

        // SAFETY: `p` is live.
        unsafe {
            let ext = (*p).get_extension.map(|g| g(p, CLAP_EXT_PARAMS.as_ptr()));
            let params = &*ext.unwrap_or(std::ptr::null()).cast::<clap_plugin_params>();
            let mut v = 0.0;
            assert!(params.get_value.is_some_and(|g| g(p, 0x1004, &mut v)));
            assert_eq!(v, 777.0);
        }

        let off = Events {
            notes: vec![note_event(0, 60, CLAP_EVENT_NOTE_OFF)],
            params: vec![],
        };
        run_block(p, 256, &off);
        for _ in 0..200 {
            run_block(p, 512, &silent);
        }
        let (l, _, _) = run_block(p, 512, &silent);
        assert!(energy(&l) < 1e-6, "note released and decayed");
        destroy(p);
    }

    struct Sink(Vec<u8>, usize);

    unsafe extern "C" fn sink_write(s: *const clap_ostream, buf: *const c_void, size: u64) -> i64 {
        // SAFETY: ctx is the test's `Sink`; `buf` has `size` readable bytes.
        let sink = unsafe { &mut *(*s).ctx.cast::<Sink>() };
        // SAFETY: the stream contract gives `size` readable bytes at `buf`.
        let chunk = unsafe { std::slice::from_raw_parts(buf.cast::<u8>(), size as usize) };
        let n = chunk.len().min(1000); // short writes, as real hosts do
        sink.0.extend_from_slice(&chunk[..n]);
        n as i64
    }

    unsafe extern "C" fn src_read(s: *const clap_istream, buf: *mut c_void, size: u64) -> i64 {
        // SAFETY: ctx is the test's `Sink` used as a source; `buf` has `size` writable bytes.
        let src = unsafe { &mut *(*s).ctx.cast::<Sink>() };
        let n = (src.0.len() - src.1).min(size as usize).min(777);
        // SAFETY: bounds were computed above.
        unsafe { std::ptr::copy_nonoverlapping(src.0[src.1..].as_ptr(), buf.cast::<u8>(), n) };
        src.1 += n;
        n as i64
    }

    #[test]
    fn state_round_trips_through_host_streams() {
        let a = make();
        let b = make();
        let mut sink = Sink(Vec::new(), 0);
        // SAFETY: both plug-ins are live; the streams reference `sink` for the calls only.
        unsafe {
            let ext = (*a).get_extension.map(|g| g(a, CLAP_EXT_STATE.as_ptr()));
            let st = &*ext.unwrap_or(std::ptr::null()).cast::<clap_plugin_state>();
            let pa = this(a).map(|pl| {
                pl.bridge
                    .engine
                    .lock()
                    .map(|mut e| e.set_param(0x1005, 9.0))
                    .ok();
            });
            assert!(pa.is_some());
            let os = clap_ostream {
                ctx: std::ptr::from_mut(&mut sink).cast(),
                write: Some(sink_write),
            };
            assert!(st.save.is_some_and(|f| f(a, &os)));
            assert!(sink.0.len() > 1000);
            let is = clap_istream {
                ctx: std::ptr::from_mut(&mut sink).cast(),
                read: Some(src_read),
            };
            assert!(st.load.is_some_and(|f| f(b, &is)));
            let ext = (*b).get_extension.map(|g| g(b, CLAP_EXT_PARAMS.as_ptr()));
            let params = &*ext.unwrap_or(std::ptr::null()).cast::<clap_plugin_params>();
            let mut v = 0.0;
            assert!(params.get_value.is_some_and(|g| g(b, 0x1005, &mut v)));
            assert_eq!(v, 9.0, "mirror updated after load");

            sink.0 = b"{broken".to_vec();
            sink.1 = 0;
            let is = clap_istream {
                ctx: std::ptr::from_mut(&mut sink).cast(),
                read: Some(src_read),
            };
            assert!(!st.load.is_some_and(|f| f(b, &is)), "bad state is rejected");
        }
        let _ = ext::lookup(CLAP_EXT_STATE);
        destroy(a);
        destroy(b);
    }

    #[test]
    fn gui_extension_lifecycle_and_sizing_contracts_hold() {
        let p = make();
        // SAFETY: `p` is a valid live plugin created by `make()`.
        unsafe {
            let raw = (*p).get_extension.map(|g| g(p, CLAP_EXT_GUI.as_ptr()));
            let gui = &*raw.unwrap_or(std::ptr::null()).cast::<clap_plugin_gui>();
            #[cfg(windows)]
            {
                assert!(gui.is_api_supported.is_some_and(|f| f(
                    p,
                    CLAP_WINDOW_API_WIN32.as_ptr(),
                    false
                )));
                assert!(!gui.is_api_supported.is_some_and(|f| f(
                    p,
                    CLAP_WINDOW_API_WIN32.as_ptr(),
                    true
                )));
                assert!(gui
                    .create
                    .is_some_and(|f| f(p, CLAP_WINDOW_API_WIN32.as_ptr(), false)));
                let (mut w, mut h) = (0u32, 0u32);
                assert!(gui.get_size.is_some_and(|f| f(p, &mut w, &mut h)));
                assert!(w >= 720 && h >= 440);
                let (mut small_w, mut small_h) = (100u32, 100u32);
                assert!(gui
                    .adjust_size
                    .is_some_and(|f| f(p, &mut small_w, &mut small_h)));
                assert!(small_w >= 720 && small_h >= 440);
                assert!(gui.set_size.is_some_and(|f| f(p, 1024, 640)));
                assert!(gui.get_size.is_some_and(|f| f(p, &mut w, &mut h)));
                assert_eq!((w, h), (1024, 640));
                if let Some(d) = gui.destroy {
                    d(p);
                }
            }
        }
        destroy(p);
    }
}
