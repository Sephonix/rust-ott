//! Linux X11 LV2 UI. Edits cross a bounded atomic mailbox; host callbacks run
//! only in idle(), on the host's UI thread, never on baseview's event thread.
use super::{Feature, CONTROL_OFFSET, METER_OFFSET, URI};
use baseview::{Size, WindowHandle, WindowOpenOptions, WindowScalePolicy};
use egui_baseview::EguiWindow;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, XcbWindowHandle};
use sps_echo_dsp::{params::*, AtomicValue, MeterBus};
use std::ffi::{c_char, c_void, CStr};
use std::ptr;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

type Write = unsafe extern "C" fn(*mut c_void, u32, u32, u32, *const c_void);
#[repr(C)]
struct UiDescriptor {
    uri: *const c_char,
    instantiate: unsafe extern "C" fn(
        *const UiDescriptor,
        *const c_char,
        *const c_char,
        Option<Write>,
        *mut c_void,
        *mut *mut c_void,
        *const *const Feature,
    ) -> *mut c_void,
    cleanup: unsafe extern "C" fn(*mut c_void),
    port_event: unsafe extern "C" fn(*mut c_void, u32, u32, u32, *const c_void),
    extension_data: unsafe extern "C" fn(*const c_char) -> *const c_void,
}
// SAFETY: static pointers only target immutable strings/functions.
unsafe impl Sync for UiDescriptor {}
#[repr(C)]
struct IdleInterface {
    idle: unsafe extern "C" fn(*mut c_void) -> i32,
}
#[repr(C)]
struct Resize {
    handle: *mut c_void,
    resize: unsafe extern "C" fn(*mut c_void, i32, i32) -> i32,
}
struct Shared {
    values: [AtomicValue; PARAM_COUNT],
    pending: [AtomicValue; PARAM_COUNT],
    dirty: AtomicU64,
    meters: MeterBus,
}
impl Default for Shared {
    fn default() -> Self {
        Self {
            values: std::array::from_fn(|i| AtomicValue::new(SPECS[i].default)),
            pending: std::array::from_fn(|i| AtomicValue::new(SPECS[i].default)),
            dirty: AtomicU64::new(0),
            meters: MeterBus::default(),
        }
    }
}
struct Ui {
    window: WindowHandle,
    shared: Arc<Shared>,
    write: Write,
    controller: *mut c_void,
}
struct Parent(u32);
// SAFETY: parent XID is supplied by the host's ui:parent feature.
unsafe impl HasRawWindowHandle for Parent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = XcbWindowHandle::empty();
        handle.window = self.0;
        RawWindowHandle::Xcb(handle)
    }
}
unsafe extern "C" fn instantiate(
    _: *const UiDescriptor,
    plugin_uri: *const c_char,
    _: *const c_char,
    write: Option<Write>,
    controller: *mut c_void,
    widget: *mut *mut c_void,
    features: *const *const Feature,
) -> *mut c_void {
    if plugin_uri.is_null() || widget.is_null() || features.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: LV2 URI and feature strings are NUL terminated for this callback.
    if unsafe { CStr::from_ptr(plugin_uri) }.to_bytes() != URI.as_bytes() {
        return ptr::null_mut();
    }
    let Some(write) = write else {
        return ptr::null_mut();
    };
    let mut parent = 0;
    let mut resize: *const Resize = ptr::null();
    let mut cursor = features;
    unsafe {
        while !(*cursor).is_null() {
            let feature = &**cursor;
            if !feature.uri.is_null() {
                match CStr::from_ptr(feature.uri).to_bytes() {
                    b"http://lv2plug.in/ns/extensions/ui#parent" => {
                        parent = feature.data as usize as u32
                    }
                    b"http://lv2plug.in/ns/extensions/ui#resize" => resize = feature.data.cast(),
                    _ => {}
                }
            }
            cursor = cursor.add(1);
        }
    }
    if parent == 0 {
        return ptr::null_mut();
    }
    let shared = Arc::new(Shared::default());
    let gui_shared = shared.clone();
    let window = EguiWindow::open_parented(
        &Parent(parent),
        WindowOpenOptions {
            title: "SPS Echo".to_owned(),
            size: Size::new(sps_echo_gui::WIDTH as f64, sps_echo_gui::HEIGHT as f64),
            scale: WindowScalePolicy::ScaleFactor(1.0),
            gl_config: Some(Default::default()),
        },
        Default::default(),
        sps_echo_gui::UiState::default(),
        |ctx, _, _| sps_echo_gui::configure(ctx),
        move |ctx, _, state| {
            let values = std::array::from_fn(|i| gui_shared.values[i].load());
            sps_echo_gui::draw(
                ctx,
                state,
                &values,
                &gui_shared.meters.snapshot(),
                |i, v| {
                    gui_shared.values[i].store(v);
                    gui_shared.pending[i].store(v);
                    gui_shared.dirty.fetch_or(1u64 << i, Ordering::Relaxed);
                },
            );
        },
    );
    let xid = match window.raw_window_handle() {
        RawWindowHandle::Xcb(h) => h.window as usize,
        RawWindowHandle::Xlib(h) => h.window as usize,
        _ => 0,
    };
    if xid == 0 {
        return ptr::null_mut();
    }
    unsafe {
        *widget = xid as *mut c_void;
        if !resize.is_null() {
            ((*resize).resize)(
                (*resize).handle,
                sps_echo_gui::WIDTH as i32,
                sps_echo_gui::HEIGHT as i32,
            );
        }
    }
    Box::into_raw(Box::new(Ui {
        window,
        shared,
        write,
        controller,
    }))
    .cast()
}
unsafe extern "C" fn cleanup(handle: *mut c_void) {
    if !handle.is_null() {
        // SAFETY: the host guarantees no more port_event/idle calls after cleanup.
        let mut ui = unsafe { Box::from_raw(handle.cast::<Ui>()) };
        ui.window.close();
    }
}
unsafe extern "C" fn port_event(
    handle: *mut c_void,
    port: u32,
    size: u32,
    format: u32,
    buffer: *const c_void,
) {
    if buffer.is_null() || format != 0 || size != 4 {
        return;
    }
    // SAFETY: handle is live and floatProtocol supplies one readable f32.
    let Some(ui) = (unsafe { handle.cast::<Ui>().as_ref() }) else {
        return;
    };
    let v = unsafe { buffer.cast::<f32>().read_unaligned() };
    let port = port as usize;
    if (CONTROL_OFFSET..METER_OFFSET).contains(&port) {
        let i = port - CONTROL_OFFSET;
        if ui.shared.dirty.load(Ordering::Relaxed) & (1u64 << i) == 0 {
            ui.shared.values[i].store(SPECS[i].sanitize(v));
        }
    } else if let Some(meter) = port
        .checked_sub(METER_OFFSET)
        .and_then(|i| ui.shared.meters.0.get(i))
    {
        if v.is_finite() {
            meter.store(v);
        }
    }
}
unsafe extern "C" fn idle(handle: *mut c_void) -> i32 {
    // SAFETY: host calls this on the UI thread for a live instance.
    let Some(ui) = (unsafe { handle.cast::<Ui>().as_ref() }) else {
        return 1;
    };
    let dirty = ui.shared.dirty.swap(0, Ordering::Relaxed);
    for i in 0..PARAM_COUNT {
        if dirty & (1u64 << i) != 0 {
            let v = ui.shared.pending[i].load();
            unsafe {
                (ui.write)(
                    ui.controller,
                    (CONTROL_OFFSET + i) as u32,
                    4,
                    0,
                    (&v as *const f32).cast(),
                );
            }
        }
    }
    if ui.window.is_open() {
        0
    } else {
        1
    }
}
unsafe extern "C" fn extension_data(uri: *const c_char) -> *const c_void {
    if !uri.is_null()
        && unsafe { CStr::from_ptr(uri) }.to_bytes()
            == b"http://lv2plug.in/ns/extensions/ui#idleInterface"
    {
        (&IDLE as *const IdleInterface).cast()
    } else {
        ptr::null()
    }
}
static IDLE: IdleInterface = IdleInterface { idle };
static DESCRIPTOR: UiDescriptor = UiDescriptor {
    uri: c"https://sephonix.com/plugins/sps-echo#ui".as_ptr(),
    instantiate,
    cleanup,
    port_event,
    extension_data,
};
/// # Safety
/// Callbacks follow the LV2 UI contract; the descriptor has static lifetime.
#[no_mangle]
pub unsafe extern "C" fn lv2ui_descriptor(index: u32) -> *const c_void {
    if index == 0 {
        (&DESCRIPTOR as *const UiDescriptor).cast()
    } else {
        ptr::null()
    }
}
