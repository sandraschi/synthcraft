//! `CLAP_EXT_GUI` implementation: Win32 embedded child `HWND` + `egui-wgpu` hardware UI.

use crate::bridge::GuiBridge;
use crate::plugin::this;
use clap_sys::ext::gui::{
    clap_gui_resize_hints, clap_plugin_gui, clap_window, CLAP_WINDOW_API_WIN32,
};
use clap_sys::plugin::clap_plugin;
use std::ffi::{c_char, CStr};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

pub(crate) const DEFAULT_WIDTH: u32 = 900;
pub(crate) const DEFAULT_HEIGHT: u32 = 540;
pub(crate) const MIN_WIDTH: u32 = 720;
pub(crate) const MIN_HEIGHT: u32 = 440;

pub(crate) static GUI_EXT: clap_plugin_gui = clap_plugin_gui {
    is_api_supported: Some(gui_is_api_supported),
    get_preferred_api: Some(gui_get_preferred_api),
    create: Some(gui_create),
    destroy: Some(gui_destroy),
    set_scale: Some(gui_set_scale),
    get_size: Some(gui_get_size),
    can_resize: Some(gui_can_resize),
    get_resize_hints: Some(gui_get_resize_hints),
    adjust_size: Some(gui_adjust_size),
    set_size: Some(gui_set_size),
    set_parent: Some(gui_set_parent),
    set_transient: Some(gui_set_transient),
    suggest_title: Some(gui_suggest_title),
    show: Some(gui_show),
    hide: Some(gui_hide),
};

/// Active GUI session owned by `Plugin`.
pub(crate) struct GuiInstance {
    width: u32,
    height: u32,
    scale: f64,
    #[cfg(windows)]
    win: Option<win32::Win32Gui>,
}

impl GuiInstance {
    fn new() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            scale: 1.0,
            #[cfg(windows)]
            win: None,
        }
    }

    pub(crate) fn close(&mut self) {
        #[cfg(windows)]
        {
            self.win = None;
        }
    }
}

unsafe extern "C" fn gui_is_api_supported(
    _p: *const clap_plugin,
    api: *const c_char,
    is_floating: bool,
) -> bool {
    if is_floating || api.is_null() {
        return false;
    }
    #[cfg(windows)]
    {
        // SAFETY: `api` is a host-provided NUL-terminated string; null checked above.
        unsafe { CStr::from_ptr(api) == CLAP_WINDOW_API_WIN32 }
    }
    #[cfg(not(windows))]
    {
        let _ = (api, CLAP_WINDOW_API_WIN32);
        false
    }
}

unsafe extern "C" fn gui_get_preferred_api(
    _p: *const clap_plugin,
    api: *mut *const c_char,
    is_floating: *mut bool,
) -> bool {
    #[cfg(windows)]
    {
        if api.is_null() || is_floating.is_null() {
            return false;
        }
        // SAFETY: both pointers are non-null and writable per the CLAP contract.
        unsafe {
            *api = CLAP_WINDOW_API_WIN32.as_ptr();
            *is_floating = false;
        }
        true
    }
    #[cfg(not(windows))]
    {
        let _ = (api, is_floating);
        false
    }
}

unsafe extern "C" fn gui_create(
    p: *const clap_plugin,
    api: *const c_char,
    is_floating: bool,
) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: calling our own check with the host's arguments.
        if !unsafe { gui_is_api_supported(p, api, is_floating) } {
            return false;
        }
        // SAFETY: `p` is one of ours.
        let Some(pl) = (unsafe { this(p) }) else {
            return false;
        };
        let Ok(mut slot) = pl.gui.lock() else {
            return false;
        };
        if let Some(mut old) = slot.take() {
            old.close();
        }
        *slot = Some(GuiInstance::new());
        true
    }))
    .unwrap_or(false)
}

unsafe extern "C" fn gui_destroy(p: *const clap_plugin) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: `p` is one of ours.
        let Some(pl) = (unsafe { this(p) }) else {
            return;
        };
        if let Ok(mut slot) = pl.gui.lock() {
            if let Some(mut inst) = slot.take() {
                inst.close();
            }
        }
    }));
}

unsafe extern "C" fn gui_set_scale(p: *const clap_plugin, scale: f64) -> bool {
    if !scale.is_finite() || scale <= 0.0 {
        return false;
    }
    // SAFETY: `p` is one of ours.
    let Some(pl) = (unsafe { this(p) }) else {
        return false;
    };
    let Ok(mut slot) = pl.gui.lock() else {
        return false;
    };
    let Some(inst) = slot.as_mut() else {
        return false;
    };
    inst.scale = scale.clamp(0.5, 4.0);
    #[cfg(windows)]
    if let Some(w) = inst.win.as_mut() {
        w.set_scale(inst.scale as f32);
    }
    true
}

unsafe extern "C" fn gui_get_size(
    p: *const clap_plugin,
    width: *mut u32,
    height: *mut u32,
) -> bool {
    if width.is_null() || height.is_null() {
        return false;
    }
    // SAFETY: `p` is one of ours.
    let Some(pl) = (unsafe { this(p) }) else {
        return false;
    };
    let Ok(slot) = pl.gui.lock() else {
        return false;
    };
    let (w, h) = slot
        .as_ref()
        .map_or((DEFAULT_WIDTH, DEFAULT_HEIGHT), |i| (i.width, i.height));
    // SAFETY: `width` and `height` are non-null writable pointers.
    unsafe {
        *width = w;
        *height = h;
    }
    true
}

unsafe extern "C" fn gui_can_resize(_p: *const clap_plugin) -> bool {
    true
}

unsafe extern "C" fn gui_get_resize_hints(
    _p: *const clap_plugin,
    hints: *mut clap_gui_resize_hints,
) -> bool {
    if hints.is_null() {
        return false;
    }
    // SAFETY: `hints` is non-null writable storage from the host.
    unsafe {
        *hints = clap_gui_resize_hints {
            can_resize_horizontally: true,
            can_resize_vertically: true,
            preserve_aspect_ratio: false,
            aspect_ratio_width: 0,
            aspect_ratio_height: 0,
        };
    }
    true
}

unsafe extern "C" fn gui_adjust_size(
    _p: *const clap_plugin,
    width: *mut u32,
    height: *mut u32,
) -> bool {
    if width.is_null() || height.is_null() {
        return false;
    }
    // SAFETY: `width` and `height` are non-null writable pointers.
    unsafe {
        *width = (*width).max(MIN_WIDTH);
        *height = (*height).max(MIN_HEIGHT);
    }
    true
}

unsafe extern "C" fn gui_set_size(p: *const clap_plugin, width: u32, height: u32) -> bool {
    // SAFETY: `p` is one of ours.
    let Some(pl) = (unsafe { this(p) }) else {
        return false;
    };
    let Ok(mut slot) = pl.gui.lock() else {
        return false;
    };
    let Some(inst) = slot.as_mut() else {
        return false;
    };
    inst.width = width.max(MIN_WIDTH);
    inst.height = height.max(MIN_HEIGHT);
    #[cfg(windows)]
    if let Some(w) = inst.win.as_mut() {
        w.resize(inst.width, inst.height);
    }
    true
}

unsafe extern "C" fn gui_set_parent(p: *const clap_plugin, window: *const clap_window) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: `p` and `window` come from the host.
        let (Some(pl), Some(win)) = (unsafe { this(p) }, unsafe { window.as_ref() }) else {
            return false;
        };
        if win.api.is_null() {
            return false;
        }
        #[cfg(windows)]
        {
            // SAFETY: `win.api` is a host-provided NUL-terminated string.
            if unsafe { CStr::from_ptr(win.api) } != CLAP_WINDOW_API_WIN32 {
                return false;
            }
            // SAFETY: `win.api == CLAP_WINDOW_API_WIN32`, so `specific.win32` is valid.
            let parent = unsafe { win.specific.win32 };
            if parent.is_null() {
                return false;
            }
            let Ok(mut slot) = pl.gui.lock() else {
                return false;
            };
            let Some(inst) = slot.as_mut() else {
                return false;
            };
            match win32::Win32Gui::create(
                parent,
                inst.width,
                inst.height,
                inst.scale as f32,
                Arc::clone(&pl.bridge),
            ) {
                Ok(w) => {
                    inst.win = Some(w);
                    true
                }
                Err(_) => false,
            }
        }
        #[cfg(not(windows))]
        {
            let _ = (pl, win);
            false
        }
    }))
    .unwrap_or(false)
}

unsafe extern "C" fn gui_set_transient(
    _p: *const clap_plugin,
    _window: *const clap_window,
) -> bool {
    false
}

unsafe extern "C" fn gui_suggest_title(_p: *const clap_plugin, _title: *const c_char) {}

unsafe extern "C" fn gui_show(p: *const clap_plugin) -> bool {
    // SAFETY: `p` is one of ours.
    let Some(pl) = (unsafe { this(p) }) else {
        return false;
    };
    let Ok(mut slot) = pl.gui.lock() else {
        return false;
    };
    let Some(inst) = slot.as_mut() else {
        return false;
    };
    #[cfg(windows)]
    if let Some(w) = inst.win.as_mut() {
        w.set_visible(true);
    }
    true
}

unsafe extern "C" fn gui_hide(p: *const clap_plugin) -> bool {
    // SAFETY: `p` is one of ours.
    let Some(pl) = (unsafe { this(p) }) else {
        return false;
    };
    let Ok(mut slot) = pl.gui.lock() else {
        return false;
    };
    let Some(inst) = slot.as_mut() else {
        return false;
    };
    #[cfg(windows)]
    if let Some(w) = inst.win.as_mut() {
        w.set_visible(false);
    }
    true
}

#[cfg(windows)]
mod win32 {
    use super::*;
    use crate::ui_view::ClapUiState;
    use raw_window_handle::{
        RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
    };
    use std::ffi::c_void;
    use std::num::NonZeroIsize;
    use std::sync::Once;
    use std::time::Instant;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetKeyState, ReleaseCapture, SetCapture, VK_SHIFT,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongPtrW, KillTimer, LoadCursorW,
        RegisterClassW, SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWLP_USERDATA,
        IDC_ARROW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER, SW_HIDE, SW_SHOW, WM_LBUTTONDBLCLK,
        WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP,
        WM_SIZE, WM_TIMER, WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_VISIBLE,
    };

    const TIMER_ID: usize = 1;
    const CLASS_NAME: &[u16] = &[
        b'S' as u16,
        b'y' as u16,
        b'n' as u16,
        b't' as u16,
        b'h' as u16,
        b'C' as u16,
        b'r' as u16,
        b'a' as u16,
        b'f' as u16,
        b't' as u16,
        b'C' as u16,
        b'l' as u16,
        b'a' as u16,
        b'p' as u16,
        b'G' as u16,
        b'u' as u16,
        b'i' as u16,
        0,
    ];

    static REGISTER_ONCE: Once = Once::new();

    struct WndState {
        bridge: Arc<GuiBridge>,
        ui: ClapUiState,
        ctx: egui::Context,
        events: Vec<egui::Event>,
        start_time: Instant,
        width: u32,
        height: u32,
        scale: f32,
        last_click_time: Instant,
        last_click_pos: egui::Pos2,
        gpu: GpuRenderer,
    }

    struct GpuRenderer {
        _instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        renderer: egui_wgpu::Renderer,
    }

    impl GpuRenderer {
        fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self, String> {
            let instance =
                wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
            let Some(nz) = NonZeroIsize::new(hwnd as isize) else {
                return Err("null HWND".into());
            };
            let raw_win = RawWindowHandle::Win32(Win32WindowHandle::new(nz));
            let raw_disp = RawDisplayHandle::Windows(WindowsDisplayHandle::new());
            // SAFETY: `hwnd` is a valid Win32 window we own; `Win32Gui::drop` drops `GpuRenderer`
            // before calling `DestroyWindow(hwnd)`.
            let surface = unsafe {
                instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: Some(raw_disp),
                    raw_window_handle: raw_win,
                })
            }
            .map_err(|e| format!("create_surface: {e}"))?;

            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                    apply_limit_buckets: false,
                }))
                .map_err(|e| format!("request_adapter: {e}"))?;

            let (device, queue) =
                pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                    label: Some("synthcraft-clap-gui"),
                    ..Default::default()
                }))
                .map_err(|e| format!("request_device: {e}"))?;

            let caps = surface.get_capabilities(&adapter);
            let format = caps
                .formats
                .iter()
                .copied()
                .find(wgpu::TextureFormat::is_srgb)
                .or_else(|| caps.formats.first().copied())
                .ok_or_else(|| "no supported surface format".to_string())?;

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: width.max(1),
                height: height.max(1),
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: caps
                    .alpha_modes
                    .first()
                    .copied()
                    .unwrap_or(wgpu::CompositeAlphaMode::Opaque),
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
                color_space: wgpu::SurfaceColorSpace::Auto,
            };
            surface.configure(&device, &config);

            let renderer =
                egui_wgpu::Renderer::new(&device, format, egui_wgpu::RendererOptions::default());

            Ok(Self {
                _instance: instance,
                surface,
                device,
                queue,
                config,
                renderer,
            })
        }

        fn resize(&mut self, width: u32, height: u32) {
            if width == 0 || height == 0 {
                return;
            }
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }

        fn paint(&mut self, ctx: &egui::Context, output: egui::FullOutput, scale: f32) {
            let frame = match self.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(f)
                | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    self.surface.configure(&self.device, &self.config);
                    return;
                }
                _ => return,
            };
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("synthcraft-clap-enc"),
                });

            for (id, deltas) in &output.textures_delta.set {
                for delta in deltas {
                    self.renderer
                        .update_texture(&self.device, &self.queue, *id, delta);
                }
            }

            let tris = ctx.tessellate(output.shapes, output.pixels_per_point);
            let screen = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: scale.max(0.5),
            };
            let user_cmds = self.renderer.update_buffers(
                &self.device,
                &self.queue,
                &mut encoder,
                &tris,
                &screen,
            );

            {
                let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("synthcraft-clap-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.035,
                                g: 0.055,
                                b: 0.102,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
                let mut pass = pass.forget_lifetime();
                self.renderer.render(&mut pass, &tris, &screen);
            }

            self.queue.submit(
                user_cmds
                    .into_iter()
                    .chain(std::iter::once(encoder.finish())),
            );
            self.queue.present(frame);

            for id in &output.textures_delta.free {
                self.renderer.free_texture(id);
            }
        }
    }

    pub(crate) struct Win32Gui {
        hwnd: HWND,
        state_ptr: *mut WndState,
    }

    // SAFETY: CLAP guarantees all `clap_plugin_gui` calls happen on the host's main thread.
    unsafe impl Send for Win32Gui {}

    impl Win32Gui {
        pub(crate) fn create(
            parent: *mut c_void,
            width: u32,
            height: u32,
            scale: f32,
            bridge: Arc<GuiBridge>,
        ) -> Result<Self, String> {
            // SAFETY: Win32 module handle and class registration on the main thread.
            let hinst = unsafe { GetModuleHandleW(std::ptr::null()) };
            REGISTER_ONCE.call_once(|| {
                let wc = WNDCLASSW {
                    style: 0x0008, // CS_DBLCLKS
                    lpfnWndProc: Some(wnd_proc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: hinst,
                    hIcon: std::ptr::null_mut(),
                    // SAFETY: standard Win32 cursor resource.
                    hCursor: unsafe { LoadCursorW(std::ptr::null_mut(), IDC_ARROW) },
                    hbrBackground: std::ptr::null_mut(),
                    lpszMenuName: std::ptr::null(),
                    lpszClassName: CLASS_NAME.as_ptr(),
                };
                // SAFETY: `wc` is valid and `CLASS_NAME` is static NUL-terminated UTF-16.
                unsafe {
                    RegisterClassW(&wc);
                }
            });

            // SAFETY: `parent` is a valid host HWND passed by `set_parent`.
            let hwnd = unsafe {
                CreateWindowExW(
                    0,
                    CLASS_NAME.as_ptr(),
                    CLASS_NAME.as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
                    0,
                    0,
                    width as i32,
                    height as i32,
                    parent as HWND,
                    std::ptr::null_mut(),
                    hinst,
                    std::ptr::null(),
                )
            };
            if hwnd.is_null() {
                return Err("CreateWindowExW failed".into());
            }

            let gpu = match GpuRenderer::new(hwnd, width, height) {
                Ok(g) => g,
                Err(e) => {
                    // SAFETY: `hwnd` was just created and has no user data yet.
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                    return Err(e);
                }
            };

            let state = Box::new(WndState {
                bridge,
                ui: ClapUiState::default(),
                ctx: egui::Context::default(),
                events: Vec::with_capacity(32),
                start_time: Instant::now(),
                width,
                height,
                scale: scale.max(0.5),
                last_click_time: Instant::now(),
                last_click_pos: egui::Pos2::ZERO,
                gpu,
            });
            let state_ptr = Box::into_raw(state);
            // SAFETY: `hwnd` is valid; `state_ptr` stays alive until `Win32Gui::drop`.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
                SetTimer(hwnd, TIMER_ID, 16, None);
            }

            Ok(Self { hwnd, state_ptr })
        }

        pub(crate) fn resize(&mut self, width: u32, height: u32) {
            // SAFETY: `hwnd` and `state_ptr` are valid while `self` is alive.
            unsafe {
                SetWindowPos(
                    self.hwnd,
                    std::ptr::null_mut(),
                    0,
                    0,
                    width as i32,
                    height as i32,
                    SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
                if let Some(st) = self.state_ptr.as_mut() {
                    st.width = width;
                    st.height = height;
                    st.gpu.resize(width, height);
                }
            }
        }

        pub(crate) fn set_scale(&mut self, scale: f32) {
            // SAFETY: `state_ptr` is valid while `self` is alive.
            if let Some(st) = unsafe { self.state_ptr.as_mut() } {
                st.scale = scale.max(0.5);
            }
        }

        pub(crate) fn set_visible(&mut self, visible: bool) {
            // SAFETY: `hwnd` is valid while `self` is alive.
            unsafe {
                ShowWindow(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
            }
        }
    }

    impl Drop for Win32Gui {
        fn drop(&mut self) {
            // SAFETY: clear userdata & timer first, drop WGPU surface before destroying HWND.
            unsafe {
                KillTimer(self.hwnd, TIMER_ID);
                SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
                if !self.state_ptr.is_null() {
                    drop(Box::from_raw(self.state_ptr));
                    self.state_ptr = std::ptr::null_mut();
                }
                DestroyWindow(self.hwnd);
            }
        }
    }

    fn pointer_pos(lparam: LPARAM, scale: f32) -> egui::Pos2 {
        let x = (lparam & 0xFFFF) as i16 as f32 / scale;
        let y = ((lparam >> 16) & 0xFFFF) as i16 as f32 / scale;
        egui::Pos2::new(x, y)
    }

    fn current_modifiers() -> egui::Modifiers {
        // SAFETY: `GetKeyState` is a safe Win32 query on the message thread.
        let shift = unsafe { GetKeyState(i32::from(VK_SHIFT)) } < 0;
        egui::Modifiers {
            shift,
            ..Default::default()
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // SAFETY: retrieving our `WndState` pointer set in `Win32Gui::create`.
        let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WndState };
        // SAFETY: `ptr` is either null or the live `Box<WndState>` owned by `Win32Gui` on this main thread.
        let Some(st) = (unsafe { ptr.as_mut() }) else {
            // SAFETY: standard fallback when no state is attached.
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        match msg {
            WM_MOUSEMOVE => {
                let pos = pointer_pos(lparam, st.scale);
                st.events.push(egui::Event::PointerMoved(pos));
                0
            }
            WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
                let pos = pointer_pos(lparam, st.scale);
                // SAFETY: capture mouse during drag on our own HWND.
                unsafe {
                    SetCapture(hwnd);
                }
                st.last_click_time = Instant::now();
                st.last_click_pos = pos;
                st.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: current_modifiers(),
                });
                0
            }
            WM_LBUTTONUP => {
                let pos = pointer_pos(lparam, st.scale);
                // SAFETY: release mouse capture.
                unsafe {
                    ReleaseCapture();
                }
                st.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: current_modifiers(),
                });
                0
            }
            WM_RBUTTONDOWN => {
                let pos = pointer_pos(lparam, st.scale);
                st.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Secondary,
                    pressed: true,
                    modifiers: current_modifiers(),
                });
                0
            }
            WM_RBUTTONUP => {
                let pos = pointer_pos(lparam, st.scale);
                st.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Secondary,
                    pressed: false,
                    modifiers: current_modifiers(),
                });
                0
            }
            WM_MOUSEWHEEL => {
                let delta = ((wparam >> 16) as i16) as f32 / 120.0;
                st.events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line,
                    delta: egui::Vec2::new(0.0, delta * 3.0),
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Wheel,
                    modifiers: current_modifiers(),
                });
                0
            }
            WM_SIZE => {
                let w = (lparam & 0xFFFF) as u32;
                let h = ((lparam >> 16) & 0xFFFF) as u32;
                if w > 0 && h > 0 {
                    st.width = w;
                    st.height = h;
                    st.gpu.resize(w, h);
                }
                0
            }
            WM_TIMER if wparam == TIMER_ID => {
                let raw = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::Vec2::new(st.width as f32 / st.scale, st.height as f32 / st.scale),
                    )),
                    time: Some(st.start_time.elapsed().as_secs_f64()),
                    events: std::mem::take(&mut st.events),
                    ..Default::default()
                };
                let bridge = Arc::clone(&st.bridge);
                let out = st.ctx.run_ui(raw, |ui| {
                    st.ui.draw(ui, &bridge);
                });
                st.gpu.paint(&st.ctx, out, st.scale);
                0
            }
            // SAFETY: default Win32 message handling for unhandled messages.
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }
}
