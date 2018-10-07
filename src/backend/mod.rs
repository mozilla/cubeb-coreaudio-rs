// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

// cubeb_backend::{*} is is referred:
// - ffi                : cubeb_sys::*                      (cubeb-core/lib.rs).
// - Context            : pub struct Context                (cubeb-core/context.rs).
// - ContextOps         : pub trait ContextOps              (cubeb-backend/trait.rs).
// - DeviceCollectionRef: pub struct DeviceCollectionRef    (cubeb-core/device_collection.rs).
// - DeviceId           : pub type DeviceId                 (cubeb-core/device.rs).
// - DeviceType         : pub struct DeviceType             (cubeb-core/device.rs).
// - Error              : pub struct Error                  (cubeb-core/error.rs).
// - Ops                : pub struct Ops                    (cubeb-backend/ops.rs).
// - Result             : pub type Result<T>                (cubeb-core/error.rs).
// - Stream             : pub struct Stream                 (cubeb-core/stream.rs)
// - StreamOps          : pub trait StreamOps               (cubeb-backend/traits.rs)
// - StreamParams       : pub struct StreamParams           (cubeb-core/stream.rs)
// - StreamParamsRef    : pub struct StreamParamsRef        (cubeb-core/stream.rs)
use cubeb_backend::{ffi, Context, ContextOps, DeviceCollectionRef, DeviceId,
                    DeviceRef, DeviceType, Error, Ops, Result, Stream,
                    StreamOps, StreamParams, StreamParamsRef};
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::c_void;
use std::ptr;

fn audiounit_create_device_from_hwdev(
    dev_info: &mut ffi::cubeb_device_info,
    devid: u32,
    devtype: DeviceType
) -> Result<()> {
    // Leak the memory of these strings to the external code.
    let device_id_c = CString::new(devid.to_string() + " device_id").unwrap().into_raw();
    let friendly_name_c = CString::new(devid.to_string() + " friendly_name").unwrap().into_raw();
    let group_id_c = CString::new(devid.to_string() + " group_id").unwrap().into_raw();
    let vendor_name_c = CString::new(devid.to_string() + " vendor_name").unwrap().into_raw();

    dev_info.devid = devid as *const c_void;
    dev_info.device_id = device_id_c;
    dev_info.friendly_name = friendly_name_c;
    dev_info.group_id = group_id_c;
    dev_info.vendor_name = vendor_name_c;

    dev_info.device_type = ffi::CUBEB_DEVICE_TYPE_UNKNOWN;
    dev_info.state = ffi::CUBEB_DEVICE_STATE_UNPLUGGED;
    dev_info.preferred = ffi::CUBEB_DEVICE_PREF_NONE;

    dev_info.format = ffi::CUBEB_DEVICE_FMT_ALL;
    dev_info.default_format = ffi::CUBEB_DEVICE_FMT_F32LE;
    dev_info.max_channels = 2;
    dev_info.min_rate = 1;
    dev_info.max_rate = 2;
    dev_info.default_rate = 44100;

    dev_info.latency_lo = 0;
    dev_info.latency_hi = 0;

    Ok(())
}

pub const OPS: Ops = capi_new!(AudioUnitContext, AudioUnitStream);

pub struct AudioUnitContext {
    pub ops: *const Ops,
}

impl ContextOps for AudioUnitContext {
    fn init(_context_name: Option<&CStr>) -> Result<Context> {
        let ctx = Box::new(AudioUnitContext {
            ops: &OPS as *const _,
        });
        Ok(unsafe { Context::from_ptr(Box::into_raw(ctx) as *mut _) })
    }

    fn backend_id(&mut self) -> &'static CStr {
        unsafe { CStr::from_ptr(b"audiounit-rust\0".as_ptr() as *const _) }
    }
    fn max_channel_count(&mut self) -> Result<u32> {
        Ok(0u32)
    }
    fn min_latency(&mut self, _params: StreamParams) -> Result<u32> {
        Ok(0u32)
    }
    fn preferred_sample_rate(&mut self) -> Result<u32> {
        Ok(0u32)
    }
    fn enumerate_devices(
        &mut self,
        _devtype: DeviceType,
        collection: &DeviceCollectionRef,
    ) -> Result<()> {
        let mut devices = Vec::<ffi::cubeb_device_info>::new();
        for i in 0..3 {
            let mut device = ffi::cubeb_device_info::default();
            audiounit_create_device_from_hwdev(&mut device, i, DeviceType::UNKNOWN)?;
            devices.push(device);
        }
        devices.shrink_to_fit(); // Make sure the capacity is same as the length.
        let coll = unsafe { &mut *collection.as_ptr() };
        coll.device = devices.as_mut_ptr();
        coll.count = devices.len();
        mem::forget(devices); // Leak the memory of devices to the external code.
        Ok(())
    }
    fn device_collection_destroy(&mut self, collection: &mut DeviceCollectionRef) -> Result<()> {
        let coll = unsafe { &mut *collection.as_ptr() };
        // Retake the ownership of the previous leaked memory from the external code.
        let mut devices = unsafe {
            Vec::from_raw_parts(
                coll.device,
                coll.count,
                coll.count
            )
        };
        for device in &mut devices {
            // This should be mapped to the memory allocation in
            // audiounit_create_device_from_hwdev.
            unsafe {
                // Retake the memory of these strings from the external code.
                if !device.device_id.is_null() {
                    let _ = CString::from_raw(device.device_id as *mut _);
                }
                if !device.friendly_name.is_null() {
                    let _ = CString::from_raw(device.friendly_name as *mut _);
                }
                if !device.group_id.is_null() {
                    let _ = CString::from_raw(device.group_id as *mut _);
                }
                if !device.vendor_name.is_null() {
                    let _ = CString::from_raw(device.vendor_name as *mut _);
                }
            }
        }
        drop(devices); // Release the memory.
        coll.device = ptr::null_mut();
        coll.count = 0;
        Ok(())
    }
    fn stream_init(
        &mut self,
        _stream_name: Option<&CStr>,
        _input_device: DeviceId,
        _input_stream_params: Option<&StreamParamsRef>,
        _output_device: DeviceId,
        _output_stream_params: Option<&StreamParamsRef>,
        _latency_frame: u32,
        _data_callback: ffi::cubeb_data_callback,
        _state_callback: ffi::cubeb_state_callback,
        _user_ptr: *mut c_void,
    ) -> Result<Stream> {
        Ok(unsafe { Stream::from_ptr(0xDEAD_BEEF as *mut _) })
    }
    fn register_device_collection_changed(
        &mut self,
        _dev_type: DeviceType,
        _collection_changed_callback: ffi::cubeb_device_collection_changed_callback,
        _user_ptr: *mut c_void,
    ) -> Result<()> {
        Ok(())
    }
}

struct AudioUnitStream {}

impl StreamOps for AudioUnitStream {
    fn start(&mut self) -> Result<()> {
        Ok(())
    }
    fn stop(&mut self) -> Result<()> {
        Ok(())
    }
    fn reset_default_device(&mut self) -> Result<()> {
        Ok(())
    }
    fn position(&mut self) -> Result<u64> {
        Ok(0u64)
    }
    fn latency(&mut self) -> Result<u32> {
        Ok(0u32)
    }
    fn set_volume(&mut self, volume: f32) -> Result<()> {
        assert_eq!(volume, 0.5);
        Ok(())
    }
    fn set_panning(&mut self, panning: f32) -> Result<()> {
        assert_eq!(panning, 0.5);
        Ok(())
    }
    fn current_device(&mut self) -> Result<&DeviceRef> {
        Ok(unsafe { DeviceRef::from_ptr(0xDEAD_BEEF as *mut _) })
    }
    fn device_destroy(&mut self, device: &DeviceRef) -> Result<()> {
        assert_eq!(device.as_ptr(), 0xDEAD_BEEF as *mut _);
        Ok(())
    }
    fn register_device_changed_callback(
        &mut self,
        _: ffi::cubeb_device_changed_callback,
    ) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test_ops_context_init() {
    let mut c: *mut ffi::cubeb = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.init.unwrap()(&mut c, ptr::null()) },
        ffi::CUBEB_OK
    );
    unsafe { OPS.destroy.unwrap()(c) }
}

#[test]
fn test_ops_context_max_channel_count() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut max_channel_count = u32::max_value();
    assert_eq!(
        unsafe { OPS.get_max_channel_count.unwrap()(c, &mut max_channel_count) },
        ffi::CUBEB_OK
    );
    assert_eq!(max_channel_count, 0);
}

#[test]
fn test_ops_context_min_latency() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let params: ffi::cubeb_stream_params = unsafe { ::std::mem::zeroed() };
    let mut latency = u32::max_value();
    assert_eq!(
        unsafe { OPS.get_min_latency.unwrap()(c, params, &mut latency) },
        ffi::CUBEB_OK
    );
    assert_eq!(latency, 0);
}

#[test]
fn test_ops_context_preferred_sample_rate() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut rate = u32::max_value();
    assert_eq!(
        unsafe { OPS.get_preferred_sample_rate.unwrap()(c, &mut rate) },
        ffi::CUBEB_OK
    );
    assert_eq!(rate, 0);
}

#[test]
fn test_ops_context_enumerate_devices() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut coll = ffi::cubeb_device_collection {
        device: ptr::null_mut(),
        count: 0,
    };
    assert_eq!(
        unsafe { OPS.enumerate_devices.unwrap()(c, 0, &mut coll) },
        ffi::CUBEB_OK
    );
    assert_eq!(coll.device, 0xDEAD_BEEF as *mut _);
    assert_eq!(coll.count, usize::max_value())
}

#[test]
fn test_ops_context_device_collection_destroy() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut coll = ffi::cubeb_device_collection {
        device: 0xDEAD_BEEF as *mut _,
        count: usize::max_value(),
    };
    assert_eq!(
        unsafe { OPS.device_collection_destroy.unwrap()(c, &mut coll) },
        ffi::CUBEB_OK
    );
    assert_eq!(coll.device, ptr::null_mut());
    assert_eq!(coll.count, 0);
}

// stream_init: Some($crate::capi::capi_stream_init::<$ctx>),
// stream_destroy: Some($crate::capi::capi_stream_destroy::<$stm>),
// stream_start: Some($crate::capi::capi_stream_start::<$stm>),
// stream_stop: Some($crate::capi::capi_stream_stop::<$stm>),
// stream_get_position: Some($crate::capi::capi_stream_get_position::<$stm>),

#[test]
fn test_ops_stream_latency() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    let mut latency = u32::max_value();
    assert_eq!(
        unsafe { OPS.stream_get_latency.unwrap()(s, &mut latency) },
        ffi::CUBEB_OK
    );
    assert_eq!(latency, 0);
}

#[test]
fn test_ops_stream_set_volume() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    unsafe {
        OPS.stream_set_volume.unwrap()(s, 0.5);
    }
}

#[test]
fn test_ops_stream_set_panning() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    unsafe {
        OPS.stream_set_panning.unwrap()(s, 0.5);
    }
}

#[test]
fn test_ops_stream_current_device() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    let mut device: *mut ffi::cubeb_device = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.stream_get_current_device.unwrap()(s, &mut device) },
        ffi::CUBEB_OK
    );
    assert_eq!(device, 0xDEAD_BEEF as *mut _);
}

#[test]
fn test_ops_stream_device_destroy() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    unsafe {
        OPS.stream_device_destroy.unwrap()(s, 0xDEAD_BEEF as *mut _);
    }
}
