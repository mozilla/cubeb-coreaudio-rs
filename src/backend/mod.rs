// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

use cubeb_backend::{
    ffi, Context, ContextOps, DeviceCollectionRef, DeviceId, DeviceRef, DeviceType, Error, Ops,
    Result, Stream, StreamOps, StreamParams, StreamParamsRef,
};
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr;

pub const OPS: Ops = capi_new!(AudioUnitContext, AudioUnitStream);

pub struct AudioUnitContext {
    pub ops: *const Ops,
}

impl ContextOps for AudioUnitContext {
    fn init(_context_name: Option<&CStr>) -> Result<Context> {
        // The context must be boxed since capi_destroy releases the context
        // by Box::from_raw.
        let ctx = Box::new(AudioUnitContext {
            ops: &OPS as *const Ops,
        });
        Ok(unsafe { Context::from_ptr(Box::into_raw(ctx) as *mut ffi::cubeb) })
    }

    fn backend_id(&mut self) -> &'static CStr {
        unsafe { CStr::from_ptr(b"audiounit-rust\0".as_ptr() as *const c_char) }
    }
    fn max_channel_count(&mut self) -> Result<u32> {
        Ok(256u32)
    }
    fn min_latency(&mut self, _params: StreamParams) -> Result<u32> {
        Ok(256u32)
    }
    fn preferred_sample_rate(&mut self) -> Result<u32> {
        Ok(48000u32)
    }
    fn enumerate_devices(
        &mut self,
        _devtype: DeviceType,
        collection: &DeviceCollectionRef,
    ) -> Result<()> {
        let coll = unsafe { &mut *collection.as_ptr() };
        coll.device = 0xDEAD_BEEF as *mut ffi::cubeb_device_info;
        coll.count = usize::max_value();
        Err(Error::not_supported())
    }
    fn device_collection_destroy(&mut self, collection: &mut DeviceCollectionRef) -> Result<()> {
        let coll = unsafe { &mut *collection.as_ptr() };
        assert_eq!(coll.device, 0xDEAD_BEEF as *mut ffi::cubeb_device_info);
        assert_eq!(coll.count, usize::max_value());
        coll.device = ptr::null_mut();
        coll.count = 0;
        Err(Error::not_supported())
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
        Err(Error::not_supported())
    }
    fn register_device_collection_changed(
        &mut self,
        dev_type: DeviceType,
        collection_changed_callback: ffi::cubeb_device_collection_changed_callback,
        user_ptr: *mut c_void,
    ) -> Result<()> {
        assert!(dev_type.contains(DeviceType::INPUT | DeviceType::OUTPUT));
        assert!(collection_changed_callback.is_some());
        assert_eq!(user_ptr, 0xDEAD_BEEF as *mut c_void);
        Err(Error::not_supported())
    }
}

struct AudioUnitStream {}

impl StreamOps for AudioUnitStream {
    fn start(&mut self) -> Result<()> {
        Err(Error::not_supported())
    }
    fn stop(&mut self) -> Result<()> {
        Err(Error::not_supported())
    }
    fn reset_default_device(&mut self) -> Result<()> {
        Err(Error::not_supported())
    }
    fn position(&mut self) -> Result<u64> {
        Ok(0u64)
    }
    fn latency(&mut self) -> Result<u32> {
        Ok(0u32)
    }
    fn set_volume(&mut self, volume: f32) -> Result<()> {
        // Most floating-point numbers are slightly imprecise. Using EPSILON
        // to check the equalitiy might not be correct all th time, but it's
        // enough for our case for now.
        use std::f32;
        assert!((volume - 0.5f32).abs() < f32::EPSILON);
        Err(Error::not_supported())
    }
    fn set_panning(&mut self, panning: f32) -> Result<()> {
        // The reason for using EPSILON is same as above.
        use std::f32;
        assert!((panning - 0.5f32).abs() < f32::EPSILON);
        Err(Error::not_supported())
    }
    fn current_device(&mut self) -> Result<&DeviceRef> {
        Ok(unsafe { DeviceRef::from_ptr(0xDEAD_BEEF as *mut ffi::cubeb_device) })
    }
    fn device_destroy(&mut self, device: &DeviceRef) -> Result<()> {
        assert_eq!(device.as_ptr(), 0xDEAD_BEEF as *mut ffi::cubeb_device);
        Err(Error::not_supported())
    }
    fn register_device_changed_callback(
        &mut self,
        device_changed_callback: ffi::cubeb_device_changed_callback,
    ) -> Result<()> {
        assert!(device_changed_callback.is_some());
        Err(Error::not_supported())
    }
}

#[cfg(test)]
mod test;
