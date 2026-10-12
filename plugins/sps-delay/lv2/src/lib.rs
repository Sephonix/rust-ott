//! Minimal LV2 core ABI. The host owns port buffers; the instance owns the DSP.
#![deny(unsafe_op_in_unsafe_fn)]
use sps_delay_dsp::{params::*, Engine, METER_COUNT};
use std::ffi::{c_char, c_void};
use std::ptr;

#[cfg(target_os = "linux")]
mod ui;

pub const URI: &str = "https://sephonix.com/plugins/sps-delay";
pub const CONTROL_OFFSET: usize = 4;
pub const METER_OFFSET: usize = CONTROL_OFFSET + PARAM_COUNT;
pub const TRANSPORT_PORT: usize = METER_OFFSET + METER_COUNT;
pub const PORT_COUNT: usize = TRANSPORT_PORT + 1;

pub use sps_lv2::Feature;
use sps_lv2::TransportIds;

#[repr(C)]
pub struct Descriptor {
    pub uri: *const c_char,
    pub instantiate: unsafe extern "C" fn(
        *const Descriptor,
        f64,
        *const c_char,
        *const *const Feature,
    ) -> *mut c_void,
    pub connect_port: unsafe extern "C" fn(*mut c_void, u32, *mut c_void),
    pub activate: unsafe extern "C" fn(*mut c_void),
    pub run: unsafe extern "C" fn(*mut c_void, u32),
    pub deactivate: Option<unsafe extern "C" fn(*mut c_void)>,
    pub cleanup: unsafe extern "C" fn(*mut c_void),
    pub extension_data: unsafe extern "C" fn(*const c_char) -> *const c_void,
}
// SAFETY: static descriptor pointers refer only to static strings and functions.
unsafe impl Sync for Descriptor {}

struct Instance {
    transport: TransportIds,
    engine: Engine,
    ports: [*mut f32; PORT_COUNT],
}
impl Instance {
    fn controls(&self) -> [f32; PARAM_COUNT] {
        std::array::from_fn(|i| {
            let port = self.ports[CONTROL_OFFSET + i];
            if port.is_null() {
                SPECS[i].default
            }
            // SAFETY: host control ports are readable single floats during run/activate.
            else {
                unsafe { *port }
            }
        })
    }
}
unsafe extern "C" fn instantiate(
    _: *const Descriptor,
    rate: f64,
    _: *const c_char,
    features: *const *const Feature,
) -> *mut c_void {
    if !rate.is_finite() || !(8000.0..=768000.0).contains(&rate) {
        return ptr::null_mut();
    }
    Box::into_raw(Box::new(Instance {
        engine: Engine::new(rate as f32),
        // SAFETY: instantiate features follow the host LV2 lifetime contract.
        transport: unsafe { TransportIds::from_features(features) },
        ports: [ptr::null_mut(); PORT_COUNT],
    }))
    .cast()
}
unsafe extern "C" fn connect_port(handle: *mut c_void, port: u32, data: *mut c_void) {
    // SAFETY: the host passes a live handle returned by instantiate.
    let Some(instance) = (unsafe { handle.cast::<Instance>().as_mut() }) else {
        return;
    };
    if let Some(slot) = instance.ports.get_mut(port as usize) {
        *slot = data.cast();
    }
}
unsafe extern "C" fn activate(handle: *mut c_void) {
    // SAFETY: the host serializes lifecycle calls for a live instance.
    let Some(instance) = (unsafe { handle.cast::<Instance>().as_mut() }) else {
        return;
    };
    instance.engine.set_parameters(&instance.controls());
    instance.engine.reset();
}
unsafe extern "C" fn run(handle: *mut c_void, count: u32) {
    // SAFETY: all host port buffers remain valid for count samples during this call.
    let Some(instance) = (unsafe { handle.cast::<Instance>().as_mut() }) else {
        return;
    };
    instance.engine.set_parameters(&instance.controls());
    // SAFETY: the optional atom input is host-owned and valid throughout run.
    let mut events = unsafe {
        instance
            .transport
            .events(instance.ports[TRANSPORT_PORT].cast())
    };
    let mut tempo_event = events.next();
    for i in 0..count as usize {
        while let Some((frame, bpm)) = tempo_event {
            if frame as usize > i {
                break;
            }
            instance.engine.set_tempo(bpm);
            tempo_event = events.next();
        }
        let read = |port: usize| {
            if instance.ports[port].is_null() {
                0.0
            } else {
                unsafe { *instance.ports[port].add(i) }
            }
        };
        // Read both channels before writing, so input/output aliasing is supported.
        let output = instance.engine.process_frame([read(0), read(1)], 2);
        for (channel, value) in output.iter().enumerate() {
            if !instance.ports[2 + channel].is_null() {
                unsafe {
                    *instance.ports[2 + channel].add(i) = *value;
                }
            }
        }
    }
    for (i, value) in instance.engine.meters().iter().enumerate() {
        let port = instance.ports[METER_OFFSET + i];
        if !port.is_null() {
            unsafe {
                *port = *value;
            }
        }
    }
}
unsafe extern "C" fn cleanup(handle: *mut c_void) {
    if !handle.is_null() {
        // SAFETY: host calls cleanup once, after all callbacks have finished.
        drop(unsafe { Box::from_raw(handle.cast::<Instance>()) });
    }
}
unsafe extern "C" fn extension_data(_: *const c_char) -> *const c_void {
    ptr::null()
}
static DESCRIPTOR: Descriptor = Descriptor {
    uri: c"https://sephonix.com/plugins/sps-delay".as_ptr(),
    instantiate,
    connect_port,
    activate,
    run,
    deactivate: None,
    cleanup,
    extension_data,
};
/// # Safety
/// The returned descriptor is immutable; its callbacks follow the LV2 core contract.
#[no_mangle]
pub unsafe extern "C" fn lv2_descriptor(index: u32) -> *const Descriptor {
    if index == 0 {
        &DESCRIPTOR
    } else {
        ptr::null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lv2_in_place_and_reactivation() {
        unsafe {
            assert!(lv2_descriptor(1).is_null());
            assert!(instantiate(&DESCRIPTOR, f64::NAN, ptr::null(), ptr::null()).is_null());
            let handle = instantiate(&DESCRIPTOR, 48000.0, ptr::null(), ptr::null());
            let mut controls = defaults();
            controls[BYPASS] = 1.0;
            let mut left = [0.1f32; 128];
            let mut right = [-0.1f32; 128];
            for (port, buffer) in [
                (0, left.as_mut_ptr()),
                (1, right.as_mut_ptr()),
                (2, left.as_mut_ptr()),
                (3, right.as_mut_ptr()),
            ] {
                connect_port(handle, port, buffer.cast());
            }
            for (i, v) in controls.iter_mut().enumerate() {
                connect_port(handle, (CONTROL_OFFSET + i) as u32, (v as *mut f32).cast());
            }
            activate(handle);
            run(handle, 128);
            assert_eq!(left, [0.1; 128]);
            assert_eq!(right, [-0.1; 128]);
            controls[BYPASS] = 0.0;
            assert_eq!(controls[BYPASS], 0.0);
            activate(handle);
            run(handle, 128);
            assert!(left.iter().all(|v| v.is_finite()));
            assert_ne!(left, [0.1; 128]);
            run(handle, 0);
            cleanup(handle);
        }
    }
}
