// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

extern crate coreaudio_sys;
extern crate libc;

mod utils;

// cubeb_backend::{*} is referred:
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
use self::coreaudio_sys::*;
use self::utils::*;
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_void, c_char};
use std::ptr;
use std::slice;

const PRIVATE_AGGREGATE_DEVICE_NAME: &'static str = "CubebAggregateDevice";

const DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS: AudioObjectPropertyAddress =
    AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultInputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

const DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS: AudioObjectPropertyAddress =
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

const INPUT_DATA_SOURCE_PROPERTY_ADDRESS: AudioObjectPropertyAddress =
    AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyDataSource,
        mScope: kAudioDevicePropertyScopeInput,
        mElement: kAudioObjectPropertyElementMaster,
};

const OUTPUT_DATA_SOURCE_PROPERTY_ADDRESS: AudioObjectPropertyAddress =
    AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyDataSource,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMaster,
};

fn audiounit_get_default_device_id(dev_type: DeviceType) -> AudioObjectID
{
    let adr;
    if dev_type == DeviceType::OUTPUT {
        adr = &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS;
    } else if dev_type == DeviceType::INPUT {
        adr = &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS;
    } else {
        return kAudioObjectUnknown;
    }

    let mut devid: AudioDeviceID = kAudioObjectUnknown;
    let mut size = mem::size_of::<AudioDeviceID>();
    if audio_object_get_property_data(kAudioObjectSystemObject,
                                      adr, &mut size, &mut devid) != 0 {
        return kAudioObjectUnknown;
    }

    return devid;
}

fn convert_uint32_into_string(data: u32) -> CString
{
    // Simply create an empty string if no data.
    let empty = CString::default();
    if data == 0 {
        return empty;
    }

    // Reverse 0xWXYZ into 0xZYXW.
    let mut buffer = vec![b'\x00'; 4]; // 4 bytes for uint32.
    buffer[0] = (data >> 24) as u8;
    buffer[1] = (data >> 16) as u8;
    buffer[2] = (data >> 8) as u8;
    buffer[3] = (data) as u8;

    // CString::new() will consume the input bytes vec and add a '\0' at the
    // end of the bytes. The input bytes vec must not contain any 0 bytes in
    // it, in case causing memory leaks when we leak its memory to the
    // external code and then retake the ownership of its memory.
    // https://doc.rust-lang.org/std/ffi/struct.CString.html#method.new
    CString::new(buffer).unwrap_or(empty)
}

fn audiounit_get_default_device_datasource(devtype: DeviceType,
                                           data: &mut u32) -> Result<()>
{
    let id = audiounit_get_default_device_id(devtype);
    if id == kAudioObjectUnknown {
        return Err(Error::error());
    }

    let mut size = mem::size_of_val(data);
    // FIXIT: This is wrong. We will use output scope when dev_type
    //        is unknown. Change it after C version is updated!
    /* This fails with some USB headsets (e.g., Plantronic .Audio 628). */
    let r = audio_object_get_property_data(id, if devtype == DeviceType::INPUT {
                                                   &INPUT_DATA_SOURCE_PROPERTY_ADDRESS
                                               } else {
                                                   &OUTPUT_DATA_SOURCE_PROPERTY_ADDRESS
                                               }, &mut size, data);
    if r != 0 {
        *data = 0;
    }

    Ok(())
}

fn audiounit_get_default_device_name(stm: &AudioUnitStream,
                                     device: &mut ffi::cubeb_device,
                                     devtype: DeviceType) -> Result<()>
{
    let mut data: u32 = 0;
    audiounit_get_default_device_datasource(devtype, &mut data)?;

    // FIXIT: This is wrong. We will use output scope when dev_type
    //        is unknown. Change it after C version is updated!
    let name = if devtype == DeviceType::INPUT {
        &mut device.input_name
    } else {
        &mut device.output_name
    };
    // Leak the memory to the external code.
    *name = convert_uint32_into_string(data).into_raw();
    if name.is_null() {
        // FIXIT: This is wrong. We will use output scope when dev_type
        //        is unknown. Change it after C version is updated!
        cubeb_log!("({:p}) name of {} device is empty!", stm,
                   if devtype == DeviceType::INPUT { "input" } else { "output" } );
    }
    Ok(())
}

fn audiounit_strref_to_cstr_utf8(strref: CFStringRef) -> CString
{
    let empty = CString::default();
    if strref.is_null() {
        return empty;
    }

    let len = unsafe {
        CFStringGetLength(strref)
    };
    // Add 1 to size to allow for '\0' termination character.
    let size = unsafe {
        CFStringGetMaximumSizeForEncoding(len, kCFStringEncodingUTF8) + 1
    };
    let mut buffer = vec![b'\x00'; size as usize];

    let success = unsafe {
        CFStringGetCString(
            strref,
            buffer.as_mut_ptr() as *mut c_char,
            size,
            kCFStringEncodingUTF8
        ) != 0
    };
    if !success {
        buffer.clear();
    }

    // CString::new() will consume the input bytes vec and add a '\0' at the
    // end of the bytes. We need to remove the '\0' from the bytes data
    // returned from CFStringGetCString by ourselves to avoid memory leaks.
    // https://doc.rust-lang.org/std/ffi/struct.CString.html#method.new
    // The size returned from CFStringGetMaximumSizeForEncoding is always
    // greater than or equal to the string length, where the string length
    // is the number of characters from the beginning to nul-terminator('\0'),
    // so we should shrink the string vector to fit that size.
    let str_len = unsafe {
        libc::strlen(buffer.as_ptr() as *mut c_char)
    };
    buffer.truncate(str_len); // Drop the elements from '\0'(including '\0').

    CString::new(buffer).unwrap_or(empty)
}

fn audiounit_get_channel_count(devid: AudioObjectID, scope: AudioObjectPropertyScope) -> u32
{
    let mut count: u32 = 0;
    let mut size: usize = 0;

    let adr = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyStreamConfiguration,
        mScope: scope,
        mElement: kAudioObjectPropertyElementMaster
    };

    if audio_object_get_property_data_size(devid, &adr, &mut size) == 0 && size > 0 {
        let mut data: Vec<u8> = allocate_array_by_size(size);
        let ptr = data.as_mut_ptr() as *mut AudioBufferList;
        if audio_object_get_property_data(devid, &adr, &mut size, ptr) == 0 {
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

fn audiounit_get_available_samplerate(devid: AudioObjectID, scope: AudioObjectPropertyScope,
                                      min: &mut u32, max: &mut u32, def: &mut u32)
{
    let mut adr = AudioObjectPropertyAddress {
        mSelector: 0,
        mScope: scope,
        mElement: kAudioObjectPropertyElementMaster
    };

    adr.mSelector = kAudioDevicePropertyNominalSampleRate;
    if audio_object_has_property(devid, &adr) {
        let mut size = mem::size_of::<f64>();
        let mut fvalue: f64 = 0.0;
        if audio_object_get_property_data(devid, &adr, &mut size, &mut fvalue) == 0 {
            *def = fvalue as u32;
        }
    }

    adr.mSelector = kAudioDevicePropertyAvailableNominalSampleRates;
    let mut size = 0;
    let mut range = AudioValueRange::default();
    if audio_object_has_property(devid, &adr) &&
       audio_object_get_property_data_size(devid, &adr, &mut size) == 0 {
        let mut ranges: Vec<AudioValueRange> = allocate_array_by_size(size);
        range.mMinimum = 9999999999.0; // TODO: why not f64::MAX?
        range.mMaximum = 0.0; // TODO: why not f64::MIN?
        if audio_object_get_property_data(devid, &adr, &mut size, ranges.as_mut_ptr()) == 0 {
            for rng in &ranges {
                if rng.mMaximum > range.mMaximum {
                    range.mMaximum = rng.mMaximum;
                }
                if rng.mMinimum < range.mMinimum {
                    range.mMinimum = rng.mMinimum;
                }
            }
        }
        *max = range.mMaximum as u32;
        *min = range.mMinimum as u32;
    } else {
        *max = 0;
        *min = 0;
    }
}

fn audiounit_get_device_presentation_latency(devid: AudioObjectID, scope: AudioObjectPropertyScope) -> u32
{
    let mut adr = AudioObjectPropertyAddress {
        mSelector: 0,
        mScope: scope,
        mElement: kAudioObjectPropertyElementMaster
    };
    let mut size: usize = 0;
    let mut dev: u32 = 0;
    let mut stream: u32 = 0;
    let mut sid: Vec<AudioStreamID> = allocate_array(1);

    adr.mSelector = kAudioDevicePropertyLatency;
    size = mem::size_of::<u32>();
    if audio_object_get_property_data(devid, &adr, &mut size, &mut dev) != 0 {
        dev = 0;
    }

    adr.mSelector = kAudioDevicePropertyStreams;
    size = mem::size_of_val(&sid);
    if audio_object_get_property_data(devid, &adr, &mut size, sid.as_mut_ptr()) == 0 {
        adr.mSelector = kAudioStreamPropertyLatency;
        size = mem::size_of::<u32>();
        audio_object_get_property_data(sid[0], &adr, &mut size, &mut stream);
    }

    dev + stream
}

fn audiounit_create_device_from_hwdev(dev_info: &mut ffi::cubeb_device_info, devid: AudioObjectID, devtype: DeviceType) -> Result<()>
{
    let mut adr = AudioObjectPropertyAddress {
        mSelector: 0,
        mScope: 0,
        mElement: kAudioObjectPropertyElementMaster
    };
    let mut size: usize = 0;

    if devtype == DeviceType::OUTPUT {
        adr.mScope = kAudioDevicePropertyScopeOutput;
    } else if devtype == DeviceType::INPUT {
        adr.mScope = kAudioDevicePropertyScopeInput;
    } else {
        return Err(Error::error());
    }

    let ch = audiounit_get_channel_count(devid, adr.mScope);
    if ch == 0 {
        return Err(Error::error());
    }

    // Set all data in dev_info to zero(its default data is zero):
    // https://github.com/djg/cubeb-rs/blob/78ed9459b8ac2ca50ea37bb72f8a06847eb8d379/cubeb-sys/src/device.rs#L129
    *dev_info = ffi::cubeb_device_info::default();

    let mut device_id_str: CFStringRef = ptr::null();
    size = mem::size_of::<CFStringRef>();
    adr.mSelector = kAudioDevicePropertyDeviceUID;
    let mut ret = audio_object_get_property_data(devid, &adr, &mut size, &mut device_id_str);
    if ret == 0 && !device_id_str.is_null() {
        let c_string = audiounit_strref_to_cstr_utf8(device_id_str);
        // Leak the memory to the external code.
        dev_info.device_id = c_string.into_raw();

        // TODO: Why we set devid here? Does it has relationship with device_id_str?
        assert!(mem::size_of::<ffi::cubeb_devid>() >= mem::size_of_val(&devid),
                "cubeb_devid can't represent devid");
        dev_info.devid = devid as ffi::cubeb_devid;

        dev_info.group_id = dev_info.device_id;

        unsafe {
            CFRelease(device_id_str as *const c_void);
        }
        // TODO: device_id_str is a danlging pointer now.
        //       Find a way to prevent it from being used.
    }

    let mut friendly_name_str: CFStringRef = ptr::null();
    let mut ds: u32 = 0;
    size = mem::size_of::<u32>();
    adr.mSelector = kAudioDevicePropertyDataSource;
    ret = audio_object_get_property_data(devid, &adr, &mut size, &mut ds);
    if ret == 0 {
        let mut trl = AudioValueTranslation {
            mInputData: &mut ds as *mut u32 as *mut c_void,
            mInputDataSize: mem::size_of_val(&ds) as u32,
            mOutputData: &mut friendly_name_str as *mut CFStringRef as *mut c_void,
            mOutputDataSize: mem::size_of::<CFStringRef>() as u32,
        };
        adr.mSelector = kAudioDevicePropertyDataSourceNameForIDCFString;
        size = mem::size_of::<AudioValueTranslation>();
        audio_object_get_property_data(devid, &adr, &mut size, &mut trl);
    }

    // If there is no datasource for this device, fall back to the
    // device name.
    if friendly_name_str.is_null() {
        size = mem::size_of::<CFStringRef>();
        adr.mSelector = kAudioObjectPropertyName;
        audio_object_get_property_data(devid, &adr, &mut size, &mut friendly_name_str);
    }

    if friendly_name_str.is_null() {
        // Couldn't get a datasource name nor a device name, return a
        // valid string of length 0.
        let c_string = CString::default();
        dev_info.friendly_name = c_string.into_raw();
    } else {
        let c_string = audiounit_strref_to_cstr_utf8(friendly_name_str);
        // Leak the memory to the external code.
        dev_info.friendly_name = c_string.into_raw();
        unsafe {
            CFRelease(friendly_name_str as *const c_void);
        }
        // TODO: friendly_name_str is a danlging pointer now.
        //       Find a way to prevent it from being used.
    };

    let mut vendor_name_str: CFStringRef = ptr::null();
    size = mem::size_of::<CFStringRef>();
    adr.mSelector = kAudioObjectPropertyManufacturer;
    ret = audio_object_get_property_data(devid, &adr, &mut size, &mut vendor_name_str);
    if ret == 0 && !vendor_name_str.is_null() {
        let c_string = audiounit_strref_to_cstr_utf8(vendor_name_str);
        // Leak the memory to the external code.
        dev_info.vendor_name = c_string.into_raw();
        unsafe {
            CFRelease(vendor_name_str as *const c_void);
        }
        // TODO: vendor_name_str is a danlging pointer now.
        //       Find a way to prevent it from being used.
    }

    // TODO: Implement From trait for enum cubeb_device_type so we can use
    // `devtype.into()` to get `ffi::CUBEB_DEVICE_TYPE_*`.
    dev_info.device_type = if devtype == DeviceType::OUTPUT {
        ffi::CUBEB_DEVICE_TYPE_OUTPUT
    } else if devtype == DeviceType::INPUT {
        ffi::CUBEB_DEVICE_TYPE_INPUT
    } else {
        ffi::CUBEB_DEVICE_TYPE_UNKNOWN
    };
    dev_info.state = ffi::CUBEB_DEVICE_STATE_ENABLED;
    dev_info.preferred = if devid == audiounit_get_default_device_id(devtype) {
        ffi::CUBEB_DEVICE_PREF_ALL
    } else {
        ffi::CUBEB_DEVICE_PREF_NONE
    };

    dev_info.max_channels = ch;
    dev_info.format = ffi::CUBEB_DEVICE_FMT_ALL;
    dev_info.default_format = ffi::CUBEB_DEVICE_FMT_F32NE;
    audiounit_get_available_samplerate(devid, adr.mScope,
                                       &mut dev_info.min_rate, &mut dev_info.max_rate, &mut dev_info.default_rate);

    let latency = audiounit_get_device_presentation_latency(devid, adr.mScope);
    let mut range = AudioValueRange::default();
    adr.mSelector = kAudioDevicePropertyBufferFrameSizeRange;
    size = mem::size_of::<AudioValueRange>();
    ret = audio_object_get_property_data(devid, &adr, &mut size, &mut range);
    if ret == 0 {
        dev_info.latency_lo = latency + range.mMinimum as u32;
        dev_info.latency_hi = latency + range.mMaximum as u32;
    } else {
        dev_info.latency_lo = 10 * dev_info.default_rate / 1000;    /* Default to 10ms */
        dev_info.latency_hi = 100 * dev_info.default_rate / 1000;   /* Default to 10ms */
    }

    Ok(())
}

fn is_aggregate_device(device_info: &ffi::cubeb_device_info) -> bool {
    assert!(!device_info.friendly_name.is_null());
    unsafe {
        libc::strncmp(device_info.friendly_name, PRIVATE_AGGREGATE_DEVICE_NAME.as_ptr() as *const c_char,
                      libc::strlen(PRIVATE_AGGREGATE_DEVICE_NAME.as_ptr() as *const c_char)) == 0
    }
}

fn audiounit_get_devices_of_type(dev_type: DeviceType) -> Vec<AudioObjectID> {
    let mut size: usize = 0;
    let mut ret = audio_object_get_property_data_size(kAudioObjectSystemObject,
                                                      &DEVICES_PROPERTY_ADDRESS,
                                                      &mut size
    );
    if ret != 0 {
        return Vec::new();
    }
    /* Total number of input and output devices. */
    let mut devices: Vec<AudioObjectID> = allocate_array_by_size(size);
    ret = audio_object_get_property_data(kAudioObjectSystemObject,
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

    // FIXIT: This is wrong. We will use output scope when dev_type
    //        is unknown. Change it after C version is updated!
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
            input_devs = audiounit_get_devices_of_type(DeviceType::INPUT);
        }

        let mut devices: Vec<ffi::cubeb_device_info> = allocate_array(
            output_devs.len() + input_devs.len()
        );

        let mut count = 0;
        if devtype.contains(DeviceType::OUTPUT) {
            for dev in output_devs {
                let device = &mut devices[count];
                if audiounit_create_device_from_hwdev(device, dev, DeviceType::OUTPUT).is_err() ||
                   is_aggregate_device(device) {
                    continue;
                }
                count += 1;
            }
        }

        if devtype.contains(DeviceType::INPUT) {
            for dev in input_devs {
                let device = &mut devices[count];
                if audiounit_create_device_from_hwdev(device, dev, DeviceType::INPUT).is_err() ||
                   is_aggregate_device(device) {
                    continue;
                }
                count += 1;
            }
        }

        let coll = unsafe { &mut *collection.as_ptr() };
        if count > 0 {
            let (ptr, len) = get_leaked_vec(devices);
            coll.device = ptr;
            coll.count = len;
        } else {
            coll.device = ptr::null_mut();
            coll.count = 0;
        }

        Ok(())
    }
    fn device_collection_destroy(&mut self, collection: &mut DeviceCollectionRef) -> Result<()> {
        let coll = unsafe { &mut *collection.as_ptr() };
        if coll.device.is_null() {
            return Ok(());
        }

        // Retake the ownership of the previous leaked memory from the external code.
        let mut devices = retake_leaked_vec(coll.device, coll.count);
        for device in &mut devices {
            // This should be mapped to the memory allocation in
            // audiounit_create_device_from_hwdev.
            unsafe {
                // Retake the memory of these strings from the external code.
                if !device.device_id.is_null() {
                    // group_id is a mirror to device_id, so we could skip it.
                    assert!(!device.group_id.is_null());
                    assert_eq!(device.device_id, device.group_id);
                    let _ = CString::from_raw(device.device_id as *mut _);
                }
                if !device.friendly_name.is_null() {
                    let _ = CString::from_raw(device.friendly_name as *mut _);
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
        let boxed_stream = AudioUnitStream::new(self)?;
        let cubeb_stream = unsafe {
            Stream::from_ptr(Box::into_raw(boxed_stream) as *mut _)
        };
        Ok(cubeb_stream)
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

struct AudioUnitStream<'ctx> {
    context: &'ctx AudioUnitContext,
    state: ffi::cubeb_state,
}

impl<'ctx> AudioUnitStream<'ctx> {
    fn new(
        context: &'ctx AudioUnitContext,
    ) -> Result<Box<Self>> {
         let stm = AudioUnitStream {
             context,
             state: ffi::CUBEB_STATE_ERROR,
         };
         let boxed_stm = Box::new(stm);
         println!("stream @ {:p} is initialized!", boxed_stm.as_ref());
         Ok(boxed_stm)
    }
}

impl<'ctx> Drop for AudioUnitStream<'ctx> {
    fn drop(&mut self) {
        println!("stream @ {:p} is dropped!", self);
    }
}

impl<'ctx> StreamOps for AudioUnitStream<'ctx> {
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
    #[cfg(target_os = "ios")]
    fn current_device(&mut self) -> Result<&DeviceRef> {
        Err(not_supported())
    }
    #[cfg(not(target_os = "ios"))]
    fn current_device(&mut self) -> Result<&DeviceRef> {
        let mut device: Box<ffi::cubeb_device> = Box::new(unsafe { mem::zeroed() });
        audiounit_get_default_device_name(self, device.as_mut(), DeviceType::OUTPUT)?;
        audiounit_get_default_device_name(self, device.as_mut(), DeviceType::INPUT)?;
        Ok(unsafe { DeviceRef::from_ptr(Box::into_raw(device) as *mut _) })
    }
    #[cfg(target_os = "ios")]
    fn device_destroy(&mut self, device: &DeviceRef) -> Result<()> {
        Err(not_supported())
    }
    #[cfg(not(target_os = "ios"))]
    fn device_destroy(&mut self, device: &DeviceRef) -> Result<()> {
        if device.as_ptr().is_null() {
            Err(Error::error())
        } else {
            unsafe {
                let mut dev: Box<ffi::cubeb_device> = Box::from_raw(device.as_ptr() as *mut _);
                if !dev.output_name.is_null() {
                    let _ = CString::from_raw(dev.output_name as *mut _);
                    dev.output_name = ptr::null_mut();
                }
                if !dev.input_name.is_null() {
                    let _ = CString::from_raw(dev.input_name as *mut _);
                    dev.input_name = ptr::null_mut();
                }
                drop(dev);
            }
            Ok(())
        }
    }
    fn register_device_changed_callback(
        &mut self,
        _: ffi::cubeb_device_changed_callback,
    ) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test;
