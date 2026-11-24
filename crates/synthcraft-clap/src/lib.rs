//! SynthCraft as a CLAP instrument.
//!
//! Exports the standard `clap_entry` symbol. Features: stereo output, note input (CLAP note
//! events and raw MIDI), 25 automatable parameters from `synthcraft_core::params`, and
//! state save/load as the patch JSON. Audio runs on the same engine as the desktop app.
//!
//! Sample banks: the patch JSON stores the bank path, so a saved project reloads its bank.
//! To pick a bank without a GUI, set the environment variable `SYNTHCRAFT_BANK` to an
//! `.sfz`, `.sf2`, `.wav` or folder path before starting the host; it is loaded on activate.
//!
//! Threading follows the CLAP rules: `process` and `flush` run on the audio thread and never
//! block (a busy engine yields silence for that block); everything else is main-thread.

mod bridge;
mod ext;
mod gui;
mod plugin;
mod ui_view;

pub use plugin::{create_for_test, ENTRY_ID};

use clap_sys::entry::clap_plugin_entry;
use clap_sys::version::CLAP_VERSION;
use std::ffi::{c_char, c_void, CStr};

/// The symbol CLAP hosts look for.
#[no_mangle]
#[used]
pub static clap_entry: clap_plugin_entry = clap_plugin_entry {
    clap_version: CLAP_VERSION,
    init: Some(entry_init),
    deinit: Some(entry_deinit),
    get_factory: Some(entry_get_factory),
};

unsafe extern "C" fn entry_init(_plugin_path: *const c_char) -> bool {
    true
}

unsafe extern "C" fn entry_deinit() {}

unsafe extern "C" fn entry_get_factory(factory_id: *const c_char) -> *const c_void {
    if factory_id.is_null() {
        return std::ptr::null();
    }
    // SAFETY: the host passes a NUL-terminated factory id; null was handled above.
    let id = unsafe { CStr::from_ptr(factory_id) };
    if id == clap_sys::factory::plugin_factory::CLAP_PLUGIN_FACTORY_ID {
        std::ptr::from_ref(&plugin::FACTORY).cast()
    } else {
        std::ptr::null()
    }
}
