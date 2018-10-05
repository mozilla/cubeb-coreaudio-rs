// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

extern crate coreaudio_sys;
use self::coreaudio_sys::*;

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
use std::mem;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::slice;

mod utils;
use self::utils::*;

// Implementation
// ============================================================================
pub const DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS: AudioObjectPropertyAddress =
    AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultInputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

pub const DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS: AudioObjectPropertyAddress =
    AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultOutputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

const DEVICES_PROPERTY_ADDRESS: AudioObjectPropertyAddress =
    AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDevices,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

fn audiounit_get_default_device_id(
    dev_type: DeviceType
) -> AudioObjectID {
    let mut adr;
    if dev_type == DeviceType::OUTPUT {
        adr = &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS;
    } else if dev_type == DeviceType::INPUT {
        adr = &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS;
    } else {
        return kAudioObjectUnknown;
    }

    let mut dev_id: AudioDeviceID = kAudioObjectUnknown;
    let mut size = mem::size_of::<AudioDeviceID>();
    if audio_object_get_property_data(
        kAudioObjectSystemObject,
        adr,
        &mut size,
        &mut dev_id
    ) != 0 {
        return kAudioObjectUnknown;
    }

    return dev_id;
}

fn audiounit_get_channel_count(
    dev_id: AudioObjectID,
    scope: AudioObjectPropertyScope
) -> u32 {
    let mut count: u32 = 0;
    let mut size: usize = 0;

    let adr = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyStreamConfiguration,
        mScope: scope,
        mElement: kAudioObjectPropertyElementMaster
    };

    if audio_object_get_property_data_size(dev_id, &adr, &mut size) == 0 && size > 0 {
        let mut data: Vec<u8> = allocate_array_by_size(size);
        let ptr = data.as_mut_ptr() as *mut AudioBufferList;
        if audio_object_get_property_data(dev_id, &adr, &mut size, ptr) == 0 {
            let list = unsafe { *ptr };
            let buffers = unsafe {
                let ptr = list.mBuffers.as_ptr() as *mut AudioBuffer;
                let len = list.mNumberBuffers as usize;
                slice::from_raw_parts_mut(ptr, len)
            };
            for buffer in buffers {
                count += buffer.mNumberChannels;
            }
        }
    }
    count
}

fn audiounit_get_devices_of_type(dev_type: DeviceType) -> Vec<AudioObjectID> {
    let mut size: usize = 0;
    let mut ret = audio_object_get_property_data_size(
        kAudioObjectSystemObject,
        &DEVICES_PROPERTY_ADDRESS,
        &mut size
    );
    if ret != 0 {
        return Vec::new();
    }
    /* Total number of input and output devices. */
    let mut devices: Vec<AudioObjectID> = allocate_array_by_size(size);
    ret = audio_object_get_property_data(
        kAudioObjectSystemObject,
        &DEVICES_PROPERTY_ADDRESS,
        &mut size,
        devices.as_mut_ptr(),
    );
    if ret != 0 {
        return Vec::new();
    }
    /* Expected sorted but did not find anything in the docs. */
    devices.sort();
    if dev_type.contains(DeviceType::INPUT | DeviceType::OUTPUT) {
        return devices;
    }

    // FIXIT: This is wrong. We will return the output devices when dev_type
    //       is unknown. Change it after C version is updated!
    let scope = if dev_type == DeviceType::INPUT {
        kAudioDevicePropertyScopeInput
    } else {
        kAudioDevicePropertyScopeOutput
    };
    let mut devices_in_scope = Vec::new();
    for device in devices {
        if audiounit_get_channel_count(device, scope) > 0 {
            devices_in_scope.push(device);
        }
    }

    return devices_in_scope;
}


fn audiounit_create_device_from_hwdev(
    dev_info: &mut ffi::cubeb_device_info,
    dev_id: AudioObjectID,
    dev_type: DeviceType,
) -> Result<()> {
    let mut adr = AudioObjectPropertyAddress {
        mSelector: 0,
        mScope: 0,
        mElement: kAudioObjectPropertyElementMaster
    };
    let mut size: usize = 0;

    if dev_type == DeviceType::OUTPUT {
        adr.mScope = kAudioDevicePropertyScopeOutput;
    } else if dev_type == DeviceType::INPUT {
        adr.mScope = kAudioDevicePropertyScopeInput;
    } else {
        return Err(Error::error());
    }

    let ch = audiounit_get_channel_count(dev_id, adr.mScope);
    if ch == 0 {
        return Err(Error::error());
    }

    let device_id_c = CString::new("device id !").unwrap();
    let devid_c = CString::new("devid !").unwrap();
    let friendly_name_c = CString::new("friendly name !").unwrap();
    let group_id_c = CString::new("group id !").unwrap();
    let vendor_name_c = CString::new("group id !").unwrap();

    dev_info.device_id = device_id_c.into_raw();
    dev_info.devid = devid_c.into_raw() as ffi::cubeb_devid;
    dev_info.friendly_name = friendly_name_c.into_raw();
    dev_info.group_id = group_id_c.into_raw();
    dev_info.vendor_name = vendor_name_c.into_raw();
    dev_info.device_type = ffi::CUBEB_DEVICE_TYPE_UNKNOWN;
    dev_info.state = ffi::CUBEB_DEVICE_STATE_UNPLUGGED;
    dev_info.preferred = ffi::CUBEB_DEVICE_PREF_NONE;
    dev_info.format = ffi::CUBEB_DEVICE_FMT_ALL;
    dev_info.default_format = ffi::CUBEB_DEVICE_FMT_F32LE;
    dev_info.max_channels = ch;
    dev_info.min_rate = 1;
    dev_info.max_rate = 2;
    dev_info.default_rate = 0;
    dev_info.latency_lo = 0;
    dev_info.latency_hi = 0;

    Ok(())
}

// Interface
// ============================================================================
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
        let mut input_devs = Vec::<AudioObjectID>::new();
        let mut output_devs = Vec::<AudioObjectID>::new();

        // Count number of input and output devices.  This is not
        // necessarily the same as the count of raw devices supported by the
        // system since, for example, with Soundflower installed, some
        // devices may report as being both input *and* output and cubeb
        // separates those into two different devices.

        if devtype.contains(DeviceType::OUTPUT) {
            output_devs = audiounit_get_devices_of_type(DeviceType::OUTPUT);
        }

        if devtype.contains(DeviceType::INPUT) {
            input_devs = audiounit_get_devices_of_type(DeviceType::OUTPUT);
        }

        let mut devices: Vec<ffi::cubeb_device_info> = allocate_array(
            output_devs.len() + input_devs.len()
        );

        let mut count = 0;
        if devtype.contains(DeviceType::OUTPUT) {
            for dev in output_devs {
                audiounit_create_device_from_hwdev(&mut devices[count], dev, DeviceType::OUTPUT)?;
                // is_aggregate_device ?
                count += 1;
            }
        }

        if devtype.contains(DeviceType::INPUT) {
            for dev in input_devs {
                audiounit_create_device_from_hwdev(&mut devices[count], dev, DeviceType::INPUT)?;
                // is_aggregate_device ?
                count += 1;
            }
        }

        let coll = unsafe { &mut *collection.as_ptr() };
        if (count > 0) {
            let mut tmp = Vec::new();
            mem::swap(&mut devices, &mut tmp);
            let mut devs = tmp.into_boxed_slice();
            coll.device = devs.as_mut_ptr();
            coll.count = devs.len();
            // Giving away the memory owned by devices.  Don't free it!
            mem::forget(devs);
        } else {
            coll.device = ptr::null_mut();
            coll.count = 0;
        }

        Ok(())
    }

    fn device_collection_destroy(&mut self, collection: &mut DeviceCollectionRef) -> Result<()> {
        debug_assert!(!collection.as_ptr().is_null()); // TODO: This fails if there is no device.
        unsafe {
            let coll = &mut *collection.as_ptr();
            let mut devices = Vec::from_raw_parts(
                coll.device as *mut ffi::cubeb_device_info,
                coll.count,
                coll.count,
            );
            for dev in &mut devices {
                // These should be paired with what we create in audiounit_create_device_from_hwdev.
                if !dev.device_id.is_null() {
                    let _ = CString::from_raw(dev.device_id as *mut _);
                }
                if !dev.devid.is_null() {
                    let _ = CString::from_raw(dev.devid as *mut _);
                }
                if !dev.friendly_name.is_null() {
                    let _ = CString::from_raw(dev.friendly_name as *mut _);
                }
                if !dev.group_id.is_null() {
                    let _ = CString::from_raw(dev.group_id as *mut _);
                }
                if !dev.vendor_name.is_null() {
                    let _ = CString::from_raw(dev.vendor_name as *mut _);
                }
            }
            coll.device = ptr::null_mut();
            coll.count = 0;
        }
        Ok(())
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

#[cfg(test)]
mod test;
