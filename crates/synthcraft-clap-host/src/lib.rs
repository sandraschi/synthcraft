//! Native CLAP 1.2.2 plugin host for SynthCraft's 3U Eurorack `CLAP HOST` module.
//!
//! Dynamically loads external `.clap` shared libraries, enumerates their plugin descriptors and
//! `clap.params` parameters, and implements [`ExternalClapProcessor`] so `SynthRt` can process
//! stereo audio blocks through any installed `.clap` plugin with live CV-to-parameter modulation
//! (`CLAP_EVENT_PARAM_MOD`).

use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::entry::clap_plugin_entry;
use clap_sys::events::{
    clap_event_header, clap_event_param_mod, clap_event_param_value, clap_input_events,
    clap_output_events, CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_PARAM_MOD, CLAP_EVENT_PARAM_VALUE,
};
use clap_sys::ext::params::{clap_param_info, clap_plugin_params, CLAP_EXT_PARAMS};
use clap_sys::factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::process::clap_process;
use clap_sys::version::CLAP_VERSION;
use std::ffi::{c_char, c_void, CStr, CString};
use std::path::{Path, PathBuf};
use std::ptr;
use synthcraft_core::{ClapParamInfo, ClapPluginInfo, ExternalClapProcessor, SynthError};

/// Standard system directories searched for installed `.clap` plugin bundles.
pub fn standard_clap_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            dirs.push(
                PathBuf::from(local)
                    .join("Programs")
                    .join("Common")
                    .join("CLAP"),
            );
        }
        if let Ok(common) = std::env::var("COMMONPROGRAMFILES") {
            dirs.push(PathBuf::from(common).join("CLAP"));
        }
    }
    #[cfg(not(windows))]
    {
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(home).join(".clap"));
        }
        dirs.push(PathBuf::from("/usr/lib/clap"));
        dirs.push(PathBuf::from("/usr/local/lib/clap"));
    }
    if let Ok(extra) = std::env::var("CLAP_PATH") {
        let sep = if cfg!(windows) { ';' } else { ':' };
        for p in extra.split(sep).filter(|s| !s.trim().is_empty()) {
            dirs.push(PathBuf::from(p));
        }
    }
    dirs
}

/// Discover `.clap` files inside the given directories (or [`standard_clap_dirs`] if empty).
pub fn scan_clap_files(extra_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots = standard_clap_dirs();
    roots.extend(extra_dirs.iter().cloned());
    let mut found = Vec::new();
    for root in roots {
        collect_clap_files(&root, 3, &mut found);
    }
    found.sort();
    found.dedup();
    found
}

fn collect_clap_files(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth == 0 || !dir.is_dir() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_clap = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("clap"))
            .unwrap_or(false);
        if is_clap && path.is_file() {
            out.push(path);
        } else if path.is_dir() && !is_clap {
            collect_clap_files(&path, depth - 1, out);
        }
    }
}

/// Platform dynamic library handle.
struct DynLib {
    #[cfg(windows)]
    handle: windows_sys::Win32::Foundation::HMODULE,
    #[cfg(not(windows))]
    handle: *mut c_void,
}

// SAFETY: OS shared library handles are process-global pointers that can be moved across threads
// and freed on any thread via `FreeLibrary` / `dlclose`.
unsafe impl Send for DynLib {}

impl DynLib {
    fn open(path: &Path) -> Result<Self, SynthError> {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::System::LibraryLoader::LoadLibraryW;
            let wide: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            // SAFETY: `wide` is a valid NUL-terminated UTF-16 path.
            let handle = unsafe { LoadLibraryW(wide.as_ptr()) };
            if handle.is_null() {
                return Err(SynthError::Io(format!(
                    "LoadLibraryW failed for '{}'",
                    path.display()
                )));
            }
            Ok(Self { handle })
        }
        #[cfg(not(windows))]
        {
            extern "C" {
                fn dlopen(filename: *const c_char, flags: i32) -> *mut c_void;
            }
            const RTLD_NOW: i32 = 2;
            let cpath = CString::new(path.to_string_lossy().as_bytes())
                .map_err(|_| SynthError::Validation("CLAP path contains interior NUL".into()))?;
            // SAFETY: `cpath` is a valid NUL-terminated C string.
            let handle = unsafe { dlopen(cpath.as_ptr(), RTLD_NOW) };
            if handle.is_null() {
                return Err(SynthError::Io(format!(
                    "dlopen failed for '{}'",
                    path.display()
                )));
            }
            Ok(Self { handle })
        }
    }

    fn symbol(&self, name: &CStr) -> Option<*const c_void> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::LibraryLoader::GetProcAddress;
            // SAFETY: `self.handle` is a live module handle and `name` is NUL-terminated.
            let proc = unsafe { GetProcAddress(self.handle, name.as_ptr().cast()) }?;
            Some(proc as *const c_void)
        }
        #[cfg(not(windows))]
        {
            extern "C" {
                fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
            }
            // SAFETY: `self.handle` is a live dlopen handle and `name` is NUL-terminated.
            let ptr = unsafe { dlsym(self.handle, name.as_ptr()) };
            if ptr.is_null() {
                None
            } else {
                Some(ptr.cast_const())
            }
        }
    }
}

impl Drop for DynLib {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::FreeLibrary;
            // SAFETY: `self.handle` was returned by `LoadLibraryW` and is freed once here.
            unsafe {
                let _ = FreeLibrary(self.handle);
            }
        }
        #[cfg(not(windows))]
        {
            extern "C" {
                fn dlclose(handle: *mut c_void) -> i32;
            }
            // SAFETY: `self.handle` was returned by `dlopen` and is closed once here.
            unsafe {
                let _ = dlclose(self.handle);
            }
        }
    }
}

/// Helper to read an optional C string pointer into a Rust `String`.
///
/// # Safety
/// `ptr` must either be null or point to a valid NUL-terminated UTF-8/ASCII C string.
unsafe fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // SAFETY: Caller guarantees `ptr` is a valid NUL-terminated C string.
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

/// Inspect a `.clap` file without activating audio processing, returning its plugin descriptors
/// and (for `plugin_index`) its automatable parameters.
pub fn inspect_clap_file(
    path: &Path,
    plugin_index: u32,
) -> Result<(Vec<ClapPluginInfo>, Vec<ClapParamInfo>), SynthError> {
    let lib = DynLib::open(path)?;
    let entry_sym = CString::new("clap_entry")
        .map_err(|_| SynthError::Validation("invalid symbol name".into()))?;
    let entry_ptr = lib
        .symbol(&entry_sym)
        .ok_or_else(|| SynthError::Validation("missing clap_entry symbol".into()))?
        .cast::<clap_plugin_entry>();

    let cpath = CString::new(path.to_string_lossy().as_bytes())
        .map_err(|_| SynthError::Validation("path contains interior NUL".into()))?;

    // SAFETY: `entry_ptr` is the exported `clap_plugin_entry` symbol from the loaded `.clap` module.
    let entry = unsafe { &*entry_ptr };
    let init_fn = entry
        .init
        .ok_or_else(|| SynthError::Validation("clap_entry.init is null".into()))?;
    // SAFETY: `cpath` is a valid NUL-terminated path string.
    if !unsafe { init_fn(cpath.as_ptr()) } {
        return Err(SynthError::Io("clap_entry.init returned false".into()));
    }

    let result = (|| -> Result<(Vec<ClapPluginInfo>, Vec<ClapParamInfo>), SynthError> {
        let get_factory = entry
            .get_factory
            .ok_or_else(|| SynthError::Validation("clap_entry.get_factory is null".into()))?;
        // SAFETY: `CLAP_PLUGIN_FACTORY_ID` is a valid static CStr from clap-sys.
        let factory_ptr =
            unsafe { get_factory(CLAP_PLUGIN_FACTORY_ID.as_ptr()) }.cast::<clap_plugin_factory>();
        if factory_ptr.is_null() {
            return Err(SynthError::Validation(
                "CLAP bundle exposes no plugin factory".into(),
            ));
        }
        // SAFETY: `factory_ptr` is non-null and valid while `clap_entry` is initialized.
        let factory = unsafe { &*factory_ptr };
        let count_fn = factory
            .get_plugin_count
            .ok_or_else(|| SynthError::Validation("get_plugin_count is null".into()))?;
        let desc_fn = factory
            .get_plugin_descriptor
            .ok_or_else(|| SynthError::Validation("get_plugin_descriptor is null".into()))?;
        // SAFETY: Calling factory method with valid `factory_ptr`.
        let count = unsafe { count_fn(factory_ptr) };
        let mut plugins = Vec::with_capacity(count as usize);
        for idx in 0..count {
            // SAFETY: `idx < count` and `factory_ptr` is valid.
            let dptr = unsafe { desc_fn(factory_ptr, idx) };
            if dptr.is_null() {
                continue;
            }
            // SAFETY: `dptr` is non-null and points to a valid `clap_plugin_descriptor`.
            let d = unsafe { &*dptr };
            // SAFETY: Descriptor strings are valid NUL-terminated C strings or null.
            plugins.push(unsafe {
                ClapPluginInfo {
                    id: cstr_to_string(d.id),
                    name: cstr_to_string(d.name),
                    vendor: cstr_to_string(d.vendor),
                    version: cstr_to_string(d.version),
                    path: path.to_string_lossy().into_owned(),
                    plugin_index: idx,
                }
            });
        }

        // Briefly instantiate `plugin_index` to query `clap.params` metadata.
        let mut params = Vec::new();
        if let Some(target) = plugins.get(plugin_index as usize) {
            if let Some(create_fn) = factory.create_plugin {
                if let Ok(cid) = CString::new(target.id.as_str()) {
                    let host = make_clap_host();
                    // SAFETY: `factory_ptr`, `&host`, and `cid` are valid for the call.
                    let plugin_ptr = unsafe { create_fn(factory_ptr, &host, cid.as_ptr()) };
                    if !plugin_ptr.is_null() {
                        // SAFETY: `plugin_ptr` was returned by `create_plugin`.
                        let plugin = unsafe { &*plugin_ptr };
                        let ok = plugin
                            .init
                            // SAFETY: `plugin_ptr` is valid.
                            .map(|f| unsafe { f(plugin_ptr) })
                            .unwrap_or(false);
                        if ok {
                            // SAFETY: `plugin_ptr` is initialized.
                            params = unsafe { query_plugin_params(plugin_ptr) };
                        }
                        if let Some(destroy) = plugin.destroy {
                            // SAFETY: `plugin_ptr` is destroyed exactly once.
                            unsafe { destroy(plugin_ptr) };
                        }
                    }
                }
            }
        }
        Ok((plugins, params))
    })();

    if let Some(deinit) = entry.deinit {
        // SAFETY: Balances the `entry.init` call above before `DynLib` drops.
        unsafe { deinit() };
    }
    result
}

/// Query all automatable parameters from an initialized `clap_plugin`.
///
/// # Safety
/// `plugin_ptr` must be a valid, initialized `clap_plugin` pointer.
unsafe fn query_plugin_params(plugin_ptr: *const clap_plugin) -> Vec<ClapParamInfo> {
    let mut out = Vec::new();
    // SAFETY: Caller guarantees `plugin_ptr` is valid.
    let plugin = unsafe { &*plugin_ptr };
    let Some(get_ext) = plugin.get_extension else {
        return out;
    };
    // SAFETY: `CLAP_EXT_PARAMS` is a static CStr.
    let ext_ptr =
        unsafe { get_ext(plugin_ptr, CLAP_EXT_PARAMS.as_ptr()) }.cast::<clap_plugin_params>();
    if ext_ptr.is_null() {
        return out;
    }
    // SAFETY: `ext_ptr` is non-null and valid for the lifetime of `plugin_ptr`.
    let ext = unsafe { &*ext_ptr };
    let (Some(count_fn), Some(info_fn)) = (ext.count, ext.get_info) else {
        return out;
    };
    // SAFETY: `plugin_ptr` is initialized.
    let n = unsafe { count_fn(plugin_ptr) };
    for i in 0..n {
        let mut raw = clap_param_info {
            id: 0,
            flags: 0,
            cookie: ptr::null_mut(),
            name: [0; 256],
            module: [0; 1024],
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
        };
        // SAFETY: `raw` is properly initialized and `i < n`.
        if unsafe { info_fn(plugin_ptr, i, &mut raw) } {
            let bytes: Vec<u8> = raw
                .name
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| c as u8)
                .collect();
            let name = String::from_utf8_lossy(&bytes).into_owned();
            out.push(ClapParamInfo {
                id: raw.id,
                name,
                min_value: raw.min_value,
                max_value: raw.max_value,
                default_value: raw.default_value,
            });
        }
    }
    out
}

/// Host callback stubs required by `clap_host_t`.
unsafe extern "C" fn host_get_extension(
    _host: *const clap_host,
    _id: *const c_char,
) -> *const c_void {
    ptr::null()
}
unsafe extern "C" fn host_request_restart(_host: *const clap_host) {}
unsafe extern "C" fn host_request_process(_host: *const clap_host) {}
unsafe extern "C" fn host_request_callback(_host: *const clap_host) {}

fn make_clap_host() -> clap_host {
    clap_host {
        clap_version: CLAP_VERSION,
        host_data: ptr::null_mut(),
        name: c"SynthCraft 3U CLAP Host".as_ptr(),
        vendor: c"Sandra Schipal".as_ptr(),
        url: c"https://github.com/sandraschi/synthcraft".as_ptr(),
        version: c"0.3.0".as_ptr(),
        get_extension: Some(host_get_extension),
        request_restart: Some(host_request_restart),
        request_process: Some(host_request_process),
        request_callback: Some(host_request_callback),
    }
}

/// Event union passed to `clap_input_events`.
#[repr(C)]
enum PendingEvent {
    ParamValue(clap_event_param_value),
    ParamMod(clap_event_param_mod),
}

struct EventList {
    events: Vec<PendingEvent>,
}

unsafe extern "C" fn in_events_size(list: *const clap_input_events) -> u32 {
    if list.is_null() {
        return 0;
    }
    // SAFETY: `ctx` points to our stack-pinned `EventList` for the duration of `process`.
    let ctx = unsafe { (*list).ctx.cast::<EventList>() };
    if ctx.is_null() {
        return 0;
    }
    // SAFETY: `ctx` is valid for the duration of `process`.
    unsafe { (*ctx).events.len() as u32 }
}

unsafe extern "C" fn in_events_get(
    list: *const clap_input_events,
    index: u32,
) -> *const clap_event_header {
    if list.is_null() {
        return ptr::null();
    }
    // SAFETY: `ctx` points to our stack-pinned `EventList` for the duration of `process`.
    let ctx = unsafe { (*list).ctx.cast::<EventList>() };
    if ctx.is_null() {
        return ptr::null();
    }
    // SAFETY: `ctx` is valid for the duration of `process`.
    let events = unsafe { &(*ctx).events };
    match events.get(index as usize) {
        Some(PendingEvent::ParamValue(e)) => &e.header as *const clap_event_header,
        Some(PendingEvent::ParamMod(e)) => &e.header as *const clap_event_header,
        None => ptr::null(),
    }
}

unsafe extern "C" fn out_events_try_push(
    _list: *const clap_output_events,
    _event: *const clap_event_header,
) -> bool {
    true
}

/// Live, activated external `.clap` plugin instance ready for real-time audio processing.
pub struct ClapHostInstance {
    // Keep `_host` boxed at a stable address because the plugin stores `*const clap_host`.
    _host: Box<clap_host>,
    entry_ptr: *const clap_plugin_entry,
    plugin_ptr: *const clap_plugin,
    info: ClapPluginInfo,
    params: Vec<ClapParamInfo>,
    pending_params: Vec<(u32, f64)>,
    out_l: Vec<f32>,
    out_r: Vec<f32>,
    steady_time: i64,
    max_frames: usize,
    // Dropped LAST so the shared library stays mapped until `plugin.destroy` & `entry.deinit` finish.
    _lib: DynLib,
}

// SAFETY: `ClapHostInstance` owns its plugin pointer exclusively and moves from the UI/loader
// thread to the audio thread once (and back for drop); it is never accessed concurrently.
unsafe impl Send for ClapHostInstance {}

impl ClapHostInstance {
    /// Load, initialize, activate, and start processing an external `.clap` plugin.
    pub fn load(
        path: &Path,
        plugin_index: u32,
        sample_rate: f64,
        max_frames: u32,
    ) -> Result<Self, SynthError> {
        let lib = DynLib::open(path)?;
        let entry_sym = CString::new("clap_entry")
            .map_err(|_| SynthError::Validation("invalid symbol name".into()))?;
        let entry_ptr = lib
            .symbol(&entry_sym)
            .ok_or_else(|| SynthError::Validation("missing clap_entry symbol".into()))?
            .cast::<clap_plugin_entry>();

        let cpath = CString::new(path.to_string_lossy().as_bytes())
            .map_err(|_| SynthError::Validation("path contains interior NUL".into()))?;

        // SAFETY: `entry_ptr` is the exported `clap_plugin_entry` symbol.
        let entry = unsafe { &*entry_ptr };
        let init_fn = entry
            .init
            .ok_or_else(|| SynthError::Validation("clap_entry.init is null".into()))?;
        // SAFETY: `cpath` is a valid NUL-terminated path.
        if !unsafe { init_fn(cpath.as_ptr()) } {
            return Err(SynthError::Io("clap_entry.init returned false".into()));
        }

        let get_factory = entry
            .get_factory
            .ok_or_else(|| SynthError::Validation("clap_entry.get_factory is null".into()))?;
        // SAFETY: `CLAP_PLUGIN_FACTORY_ID` is a static CStr.
        let factory_ptr =
            unsafe { get_factory(CLAP_PLUGIN_FACTORY_ID.as_ptr()) }.cast::<clap_plugin_factory>();
        if factory_ptr.is_null() {
            if let Some(deinit) = entry.deinit {
                // SAFETY: Balances `entry.init`.
                unsafe { deinit() };
            }
            return Err(SynthError::Validation(
                "CLAP bundle has no plugin factory".into(),
            ));
        }

        // SAFETY: `factory_ptr` is non-null.
        let factory = unsafe { &*factory_ptr };
        let count = factory
            .get_plugin_count
            // SAFETY: `factory_ptr` is valid.
            .map(|f| unsafe { f(factory_ptr) })
            .unwrap_or(0);
        if plugin_index >= count {
            if let Some(deinit) = entry.deinit {
                // SAFETY: Balances `entry.init`.
                unsafe { deinit() };
            }
            return Err(SynthError::NotFound(format!(
                "plugin index {plugin_index} out of range (bundle has {count} plugins)"
            )));
        }

        let dptr = factory
            .get_plugin_descriptor
            // SAFETY: `plugin_index < count`.
            .map(|f| unsafe { f(factory_ptr, plugin_index) })
            .unwrap_or(ptr::null());
        if dptr.is_null() {
            if let Some(deinit) = entry.deinit {
                // SAFETY: Balances `entry.init`.
                unsafe { deinit() };
            }
            return Err(SynthError::Validation("null plugin descriptor".into()));
        }
        // SAFETY: `dptr` is non-null.
        let d = unsafe { &*dptr };
        // SAFETY: Descriptor strings are valid C strings or null.
        let info = unsafe {
            ClapPluginInfo {
                id: cstr_to_string(d.id),
                name: cstr_to_string(d.name),
                vendor: cstr_to_string(d.vendor),
                version: cstr_to_string(d.version),
                path: path.to_string_lossy().into_owned(),
                plugin_index,
            }
        };

        let host = Box::new(make_clap_host());
        let create_fn = factory
            .create_plugin
            .ok_or_else(|| SynthError::Validation("create_plugin is null".into()))?;
        let cid = CString::new(info.id.as_str())
            .map_err(|_| SynthError::Validation("plugin id contains NUL".into()))?;
        // SAFETY: `factory_ptr`, `&*host`, and `cid` are valid; `host` stays pinned in `Box`.
        let plugin_ptr = unsafe { create_fn(factory_ptr, &*host, cid.as_ptr()) };
        if plugin_ptr.is_null() {
            if let Some(deinit) = entry.deinit {
                // SAFETY: Balances `entry.init`.
                unsafe { deinit() };
            }
            return Err(SynthError::Io("create_plugin returned null".into()));
        }

        // SAFETY: `plugin_ptr` is non-null.
        let plugin = unsafe { &*plugin_ptr };
        let inited = plugin
            .init
            // SAFETY: `plugin_ptr` is valid.
            .map(|f| unsafe { f(plugin_ptr) })
            .unwrap_or(false);
        if !inited {
            if let Some(destroy) = plugin.destroy {
                // SAFETY: Destroy failed plugin.
                unsafe { destroy(plugin_ptr) };
            }
            if let Some(deinit) = entry.deinit {
                // SAFETY: Balances `entry.init`.
                unsafe { deinit() };
            }
            return Err(SynthError::Io("plugin.init returned false".into()));
        }

        // SAFETY: `plugin_ptr` is initialized.
        let params = unsafe { query_plugin_params(plugin_ptr) };
        let mf = max_frames.max(64);
        let activated = plugin
            .activate
            // SAFETY: `plugin_ptr` is initialized; `sample_rate` and frame bounds are positive.
            .map(|f| unsafe { f(plugin_ptr, sample_rate.max(8000.0), 1, mf) })
            .unwrap_or(false);
        if !activated {
            if let Some(destroy) = plugin.destroy {
                // SAFETY: Destroy unactivated plugin.
                unsafe { destroy(plugin_ptr) };
            }
            if let Some(deinit) = entry.deinit {
                // SAFETY: Balances `entry.init`.
                unsafe { deinit() };
            }
            return Err(SynthError::Io("plugin.activate returned false".into()));
        }

        if let Some(start) = plugin.start_processing {
            // SAFETY: `plugin_ptr` is activated.
            let _ = unsafe { start(plugin_ptr) };
        }

        Ok(Self {
            _host: host,
            entry_ptr,
            plugin_ptr,
            info,
            params,
            pending_params: Vec::with_capacity(16),
            out_l: vec![0.0; mf as usize],
            out_r: vec![0.0; mf as usize],
            steady_time: 0,
            max_frames: mf as usize,
            _lib: lib,
        })
    }
}

impl ExternalClapProcessor for ClapHostInstance {
    fn info(&self) -> &ClapPluginInfo {
        &self.info
    }

    fn params(&self) -> &[ClapParamInfo] {
        &self.params
    }

    fn set_param(&mut self, param_id: u32, value: f64) {
        if self.pending_params.len() < 32 {
            self.pending_params.push((param_id, value));
        }
    }

    fn process_block(&mut self, left: &mut [f32], right: &mut [f32], cv_mod: Option<(u32, f64)>) {
        let total = left.len().min(right.len());
        if total == 0 {
            return;
        }
        // SAFETY: `self.plugin_ptr` is alive and activated for the lifetime of `self`.
        let plugin = unsafe { &*self.plugin_ptr };
        let Some(process_fn) = plugin.process else {
            return;
        };

        let mut ev_list = EventList {
            events: Vec::with_capacity(self.pending_params.len() + 1),
        };
        for (param_id, value) in self.pending_params.drain(..) {
            ev_list
                .events
                .push(PendingEvent::ParamValue(clap_event_param_value {
                    header: clap_event_header {
                        size: std::mem::size_of::<clap_event_param_value>() as u32,
                        time: 0,
                        space_id: CLAP_CORE_EVENT_SPACE_ID,
                        type_: CLAP_EVENT_PARAM_VALUE,
                        flags: 0,
                    },
                    param_id,
                    cookie: ptr::null_mut(),
                    note_id: -1,
                    port_index: -1,
                    channel: -1,
                    key: -1,
                    value,
                }));
        }
        if let Some((param_id, amount)) = cv_mod {
            ev_list
                .events
                .push(PendingEvent::ParamMod(clap_event_param_mod {
                    header: clap_event_header {
                        size: std::mem::size_of::<clap_event_param_mod>() as u32,
                        time: 0,
                        space_id: CLAP_CORE_EVENT_SPACE_ID,
                        type_: CLAP_EVENT_PARAM_MOD,
                        flags: 0,
                    },
                    param_id,
                    cookie: ptr::null_mut(),
                    note_id: -1,
                    port_index: -1,
                    channel: -1,
                    key: -1,
                    amount,
                }));
        }

        let in_events = clap_input_events {
            ctx: (&mut ev_list as *mut EventList).cast::<c_void>(),
            size: Some(in_events_size),
            get: Some(in_events_get),
        };
        let out_events = clap_output_events {
            ctx: ptr::null_mut(),
            try_push: Some(out_events_try_push),
        };

        let mut offset = 0usize;
        while offset < total {
            let chunk = (total - offset).min(self.max_frames);
            let mut in_ptrs = [
                left[offset..offset + chunk].as_mut_ptr(),
                right[offset..offset + chunk].as_mut_ptr(),
            ];
            let mut out_ptrs = [self.out_l.as_mut_ptr(), self.out_r.as_mut_ptr()];
            let audio_in = clap_audio_buffer {
                data32: in_ptrs.as_mut_ptr(),
                data64: ptr::null_mut(),
                channel_count: 2,
                latency: 0,
                constant_mask: 0,
            };
            let mut audio_out = clap_audio_buffer {
                data32: out_ptrs.as_mut_ptr(),
                data64: ptr::null_mut(),
                channel_count: 2,
                latency: 0,
                constant_mask: 0,
            };
            let process = clap_process {
                steady_time: self.steady_time,
                frames_count: chunk as u32,
                transport: ptr::null(),
                audio_inputs: &audio_in,
                audio_outputs: &mut audio_out,
                audio_inputs_count: 1,
                audio_outputs_count: 1,
                in_events: &in_events,
                out_events: &out_events,
            };

            // SAFETY: All audio buffers (`chunk <= max_frames`) and event lists are valid for the
            // duration of the `process` call, and `self.plugin_ptr` is activated & processing.
            unsafe {
                let _ = process_fn(self.plugin_ptr, &process);
            }

            left[offset..offset + chunk].copy_from_slice(&self.out_l[..chunk]);
            right[offset..offset + chunk].copy_from_slice(&self.out_r[..chunk]);
            self.steady_time += chunk as i64;
            offset += chunk;
        }
    }
}

impl Drop for ClapHostInstance {
    fn drop(&mut self) {
        // SAFETY: `self.plugin_ptr` was created and activated in `load` and is torn down once here
        // before `_lib` is dropped (Rust drops fields in declaration order).
        unsafe {
            let plugin = &*self.plugin_ptr;
            if let Some(stop) = plugin.stop_processing {
                stop(self.plugin_ptr);
            }
            if let Some(deactivate) = plugin.deactivate {
                deactivate(self.plugin_ptr);
            }
            if let Some(destroy) = plugin.destroy {
                destroy(self.plugin_ptr);
            }
            let entry = &*self.entry_ptr;
            if let Some(deinit) = entry.deinit {
                deinit();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_clap_dirs_returns_platform_paths() {
        let dirs = standard_clap_dirs();
        assert!(!dirs.is_empty());
    }

    #[test]
    fn loading_missing_clap_file_returns_error_not_panic() {
        let err =
            ClapHostInstance::load(Path::new("nonexistent_plugin_12345.clap"), 0, 48000.0, 512);
        assert!(err.is_err());
    }

    #[test]
    fn can_inspect_and_host_installed_synthcraft_clap_if_present() {
        let candidate = standard_clap_dirs()
            .into_iter()
            .map(|d| d.join("SynthCraft.clap"))
            .find(|p| p.is_file());
        let Some(path) = candidate else {
            return;
        };
        let inspected = inspect_clap_file(&path, 0);
        assert!(inspected.is_ok(), "inspect failed: {:?}", inspected.err());
        if let Ok((plugins, params)) = inspected {
            assert!(!plugins.is_empty());
            assert!(!params.is_empty());

            let loaded = ClapHostInstance::load(&path, 0, 48000.0, 512);
            assert!(loaded.is_ok(), "load failed: {:?}", loaded.err());
            if let Ok(mut host) = loaded {
                let mut l = vec![0.0_f32; 256];
                let mut r = vec![0.0_f32; 256];
                host.process_block(&mut l, &mut r, Some((params[0].id, 0.15)));
                assert!(l.iter().all(|s| s.is_finite()));
                assert!(r.iter().all(|s| s.is_finite()));
            }
        }
    }
}
