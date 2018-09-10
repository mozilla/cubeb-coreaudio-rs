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
// - Result             : pub type Result<T>                (cubeb-core/error.rs).
// - Stream             : pub struct Stream                 (cubeb-core/stream.rs)
// - StreamParams       : pub struct StreamParams           (cubeb-core/stream.rs)
// - StreamParamsRef    : pub struct StreamParamsRef        (cubeb-core/stream.rs)
use cubeb_backend::{ffi, Context, ContextOps, DeviceCollectionRef, DeviceId,
                    DeviceType, Error, Result, Stream, StreamParams,
                    StreamParamsRef};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;

pub struct AudioUnitContext {
    pub context_name: Option<CString>,
}

impl AudioUnitContext {
    fn new(name: Option<&CStr>) -> Result<Box<Self>> {
        let name = name.map(|s| s.to_owned());
        let ctx = Box::new(AudioUnitContext {
             context_name: name,
        });
        Ok(ctx)
    }
}

impl ContextOps for AudioUnitContext {
    fn init(context_name: Option<&CStr>) -> Result<Context> {
        let ctx = AudioUnitContext::new(context_name)?;
        Ok(unsafe { Context::from_ptr(Box::into_raw(ctx) as *mut _) }) // _ is ffi::cubeb.
    }

    fn backend_id(&mut self) -> &'static CStr {
        unsafe { CStr::from_ptr(b"audiounit-rust\0".as_ptr() as *const _) } // _ is c_char.
    }

    fn max_channel_count(&mut self) -> Result<u32> {
        Err(Error::not_supported())
    }

    fn min_latency(&mut self, params: StreamParams) -> Result<u32> {
        Err(Error::not_supported())
    }

    fn preferred_sample_rate(&mut self) -> Result<u32> {
        Err(Error::not_supported())
    }

    fn enumerate_devices(
        &mut self,
        devtype: DeviceType,
        collection: &DeviceCollectionRef,
    ) -> Result<()> {
        Err(Error::not_supported())
    }

    fn device_collection_destroy(&mut self, collection: &mut DeviceCollectionRef) -> Result<()> {
        Err(Error::not_supported())
    }

    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
    fn stream_init(
        &mut self,
        stream_name: Option<&CStr>,
        input_device: DeviceId,
        input_stream_params: Option<&StreamParamsRef>,
        output_device: DeviceId,
        output_stream_params: Option<&StreamParamsRef>,
        latency_frames: u32,
        data_callback: ffi::cubeb_data_callback,
        state_callback: ffi::cubeb_state_callback,
        user_ptr: *mut c_void,
    ) -> Result<Stream> {
        Err(Error::not_supported())
    }

    fn register_device_collection_changed(
        &mut self,
        devtype: DeviceType,
        cb: ffi::cubeb_device_collection_changed_callback,
        user_ptr: *mut c_void,
    ) -> Result<()> {
        Err(Error::not_supported())
    }
}
