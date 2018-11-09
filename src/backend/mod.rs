// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

extern crate coreaudio_sys;
extern crate libc;

mod async_dispatch;
mod utils;
mod owned_critical_section;

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
                    StreamOps, StreamParams, StreamParamsRef, StreamPrefs};
use self::async_dispatch::*;
use self::coreaudio_sys::*;
use self::utils::*;
use self::owned_critical_section::*;
use std::cmp;
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_void, c_char};
use std::ptr;
use std::slice;

// TODO:
// 1. We use AudioDeviceID and AudioObjectID at the same time.
//    They are actually same. Maybe it's better to use only one
//    of them so code reader don't get confused about their types.
// 2. Maybe we can merge `io_side` and `DeviceType`.
// 3. Add assertions like:
//    `assert!(devtype == DeviceType::INPUT || devtype == DeviceType::OUTPUT)`
//    if the function is only called for either input or output. Then
//    `if (devtype == DeviceType::INPUT) { ... } else { ... }`
//    makes sense. In fact, for those variables depends on DeviceType, we can
//    implement a `From` trait to get them.

const AU_OUT_BUS: AudioUnitElement = 0;
const AU_IN_BUS: AudioUnitElement = 1;

const DISPATCH_QUEUE_LABEL: &'static str = "org.mozilla.cubeb";
const PRIVATE_AGGREGATE_DEVICE_NAME: &'static str = "CubebAggregateDevice";

/* Testing empirically, some headsets report a minimal latency that is very
 * low, but this does not work in practice. Lie and say the minimum is 256
 * frames. */
const SAFE_MIN_LATENCY_FRAMES: u32 = 256;
const SAFE_MAX_LATENCY_FRAMES: u32 = 512;

// TODO: Move them into a seperate module, or add an API to generate these
//       property addressed.
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

bitflags! {
    struct device_flags: u32 {
        const DEV_UNKNOWN           = 0b00000000; /* Unknown */
        const DEV_INPUT             = 0b00000001; /* Record device like mic */
        const DEV_OUTPUT            = 0b00000010; /* Playback device like speakers */
        const DEV_SYSTEM_DEFAULT    = 0b00000100; /* System default device */
        const DEV_SELECTED_DEFAULT  = 0b00001000; /* User selected to use the system default device */
    }
}

#[derive(Debug, PartialEq)]
enum io_side {
  INPUT,
  OUTPUT,
}

#[derive(Clone, Debug)]
struct device_info {
    id: AudioDeviceID,
    flags: device_flags
}

impl device_info {
    fn new() -> Self {
        device_info {
            id: kAudioObjectUnknown,
            flags: device_flags::DEV_UNKNOWN,
        }
    }
}

impl Default for device_info {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

// 'ctx: 'stm means 'ctx outlives 'stm
struct property_listener<'addr, 'stm, 'ctx: 'stm> {
    device_id: AudioDeviceID,
    property_address: &'addr AudioObjectPropertyAddress,
    callback: audio_object_property_listener_proc,
    stream: &'stm mut AudioUnitStream<'ctx>,
}

impl<'addr, 'stm, 'ctx> property_listener<'addr, 'stm, 'ctx> {
    fn new(id: AudioDeviceID,
           address: &'addr AudioObjectPropertyAddress,
           listener: audio_object_property_listener_proc,
           stm: &'stm mut AudioUnitStream<'ctx>) -> Self {
        property_listener {
            device_id: id,
            property_address: address,
            callback: listener,
            stream: stm
        }
    }
}

fn has_input(stm: &AudioUnitStream) -> bool
{
    stm.input_stream_params.rate() > 0
}

fn has_output(stm: &AudioUnitStream) -> bool
{
    stm.output_stream_params.rate() > 0
}

fn audiounit_increment_active_streams(ctx: &mut AudioUnitContext)
{
    ctx.mutex.assert_current_thread_owns();
    ctx.active_streams += 1;
}

fn audiounit_decrement_active_streams(ctx: &mut AudioUnitContext)
{
    ctx.mutex.assert_current_thread_owns();
    ctx.active_streams -= 1;
}

fn audiounit_active_streams(ctx: &mut AudioUnitContext) -> i32
{
    ctx.mutex.assert_current_thread_owns();
    ctx.active_streams
}

fn audiounit_set_global_latency(ctx: &mut AudioUnitContext, latency_frames: u32)
{
    ctx.mutex.assert_current_thread_owns();
    assert_eq!(audiounit_active_streams(ctx), 1);
    ctx.global_latency_frames = latency_frames;
}

fn audiounit_set_device_info(stm: &mut AudioUnitStream, id: AudioDeviceID, devtype: DeviceType) -> Result<()>
{
    assert!(devtype == DeviceType::INPUT || devtype == DeviceType::OUTPUT);

    let info = if devtype == DeviceType::INPUT {
        &mut stm.input_device
    } else {
        &mut stm.output_device
    };

    *info = device_info::default();
    info.id = id;
    info.flags |= if devtype == DeviceType::INPUT {
        device_flags::DEV_INPUT
    } else {
        device_flags::DEV_OUTPUT
    };

    let default_device_id = audiounit_get_default_device_id(devtype);
    if default_device_id == kAudioObjectUnknown {
        return Err(Error::error());
    }

    if id == kAudioObjectUnknown {
        info.id = default_device_id;
        info.flags |= device_flags::DEV_SELECTED_DEFAULT;
    }

    if info.id == default_device_id {
        info.flags |= device_flags::DEV_SYSTEM_DEFAULT;
    }

    assert_ne!(info.id, kAudioObjectUnknown);
    assert!(info.flags.contains(device_flags::DEV_INPUT) && !info.flags.contains(device_flags::DEV_OUTPUT) ||
            !info.flags.contains(device_flags::DEV_INPUT) && info.flags.contains(device_flags::DEV_OUTPUT));

    Ok(())
}

fn audiounit_add_listener(listener: &mut property_listener) -> OSStatus
{
    audio_object_add_property_listener(listener.device_id,
                                       listener.property_address,
                                       listener.callback,
                                       listener.stream as *mut AudioUnitStream as *mut c_void)
}

fn audiounit_remove_listener(listener: &mut property_listener) -> OSStatus
{
    audio_object_remove_property_listener(listener.device_id,
                                          listener.property_address,
                                          listener.callback,
                                          listener.stream as *mut AudioUnitStream as *mut c_void)
}

fn audiounit_install_system_changed_callback(stm: &mut AudioUnitStream) -> Result<()>
{
    let mut r: OSStatus = 0;

    if !stm.output_unit.is_null() {
        /* This event will notify us when the default audio device changes,
         * for example when the user plugs in a USB headset and the system chooses it
         * automatically as the default, or when another device is chosen in the
         * dropdown list. */
        // stm.default_output_listener =
    }

    if !stm.input_unit.is_null() {
        /* This event will notify us when the default input device changes. */
    }

    Ok(())
}

fn audiounit_get_acceptable_latency_range(latency_range: &mut AudioValueRange) -> Result<()>
{
    let mut size: usize = 0;
    let mut r: OSStatus = 0;
    let mut output_device_id: AudioDeviceID = kAudioObjectUnknown;
    let output_device_buffer_size_range = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyBufferFrameSizeRange,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMaster,
    };

    output_device_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if output_device_id == kAudioObjectUnknown {
        cubeb_log!("Could not get default output device id.");
        return Err(Error::error());
    }

    /* Get the buffer size range this device supports */
    size = mem::size_of_val(latency_range);

    r = audio_object_get_property_data(output_device_id,
                                       &output_device_buffer_size_range,
                                       &mut size,
                                       latency_range);
    if r != 0 {
        cubeb_log!("AudioObjectGetPropertyData/buffer size range rv={}", r);
        return Err(Error::error());
    }

    Ok(())
}

fn audiounit_get_default_device_id(devtype: DeviceType) -> AudioObjectID
{
    let adr;
    if devtype == DeviceType::OUTPUT {
        adr = &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS;
    } else if devtype == DeviceType::INPUT {
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

fn get_device_name(id: AudioDeviceID) -> CFStringRef
{
    let mut size = mem::size_of::<CFStringRef>();
    let mut UIname: CFStringRef = ptr::null();
    let address_uuid = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyDeviceUID,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster
    };
    let err = audio_object_get_property_data(id, &address_uuid, &mut size, &mut UIname);
    if err == 0 { UIname } else { ptr::null() }
}

// fn get_device_name(id: AudioDeviceID) -> CString
// {
//     let mut size = mem::size_of::<CFStringRef>();
//     let mut UIname: CFStringRef = ptr::null();
//     let address_uuid = AudioObjectPropertyAddress {
//         mSelector: kAudioDevicePropertyDeviceUID,
//         mScope: kAudioObjectPropertyScopeGlobal,
//         mElement: kAudioObjectPropertyElementMaster
//     };
//     let err = audio_object_get_property_data(id, &address_uuid, &mut size, &mut UIname);
//     if err != 0 {
//         UIname = ptr::null();
//     }
//     audiounit_strref_to_cstr_utf8(UIname)
// }

#[cfg(target_os = "ios")]
fn audiounit_new_unit_instance(unit: &mut AudioUnit, _: &device_info) -> Result<()>
{
    assert_eq!(*unit, ptr::null_mut());

    let mut desc = AudioComponentDescription::default();
    let mut comp: AudioComponent;
    let mut rv: OSStatus = 0;

    desc.componentType = kAudioUnitType_Output;
    desc.componentSubType = kAudioUnitSubType_RemoteIO;

    desc.componentManufacturer = kAudioUnitManufacturer_Apple;
    desc.componentFlags = 0;
    desc.componentFlagsMask = 0;
    comp = unsafe { AudioComponentFindNext(ptr::null_mut(), &desc) };
    if comp.is_null() {
        cubeb_log!("Could not find matching audio hardware.");
        return Err(Error::error());
    }

    rv = unsafe { AudioComponentInstanceNew(comp, unit) };
    if rv != 0 {
        cubeb_log!("AudioComponentInstanceNew rv={}", rv);
        return Err(Error::error());
    }
    Ok(())
}

#[cfg(not(target_os = "ios"))]
fn audiounit_new_unit_instance(unit: &mut AudioUnit, device: &device_info) -> Result<()>
{
    assert_eq!(*unit, ptr::null_mut());

    let mut desc = AudioComponentDescription::default();
    let mut comp: AudioComponent = ptr::null_mut();
    let mut rv: OSStatus = 0;

    desc.componentType = kAudioUnitType_Output;
    // Use the DefaultOutputUnit for output when no device is specified
    // so we retain automatic output device switching when the default
    // changes.  Once we have complete support for device notifications
    // and switching, we can use the AUHAL for everything.
    if device.flags.contains(device_flags::DEV_SYSTEM_DEFAULT |
                             device_flags::DEV_OUTPUT) {
        desc.componentSubType = kAudioUnitSubType_DefaultOutput;
    } else {
        desc.componentSubType = kAudioUnitSubType_HALOutput;
    }

    desc.componentManufacturer = kAudioUnitManufacturer_Apple;
    desc.componentFlags = 0;
    desc.componentFlagsMask = 0;
    comp = unsafe { AudioComponentFindNext(ptr::null_mut(), &desc) };
    if comp.is_null() {
        cubeb_log!("Could not find matching audio hardware.");
        return Err(Error::error());
    }

    rv = unsafe { AudioComponentInstanceNew(comp, unit as *mut AudioUnit) };
    if rv != 0 {
        cubeb_log!("AudioComponentInstanceNew rv={}", rv);
        return Err(Error::error());
    }
    Ok(())
}

#[derive(PartialEq)]
enum enable_state {
  DISABLE,
  ENABLE,
}

fn audiounit_enable_unit_scope(unit: &AudioUnit, side: io_side, state: enable_state) -> Result<()>
{
    assert_ne!(*unit, ptr::null_mut());

    let mut rv: OSStatus = 0;
    let enable: u32 = if state == enable_state::DISABLE { 0 } else { 1 };
    rv = audio_unit_set_property(unit, kAudioOutputUnitProperty_EnableIO,
                                 if side == io_side::INPUT { kAudioUnitScope_Input } else { kAudioUnitScope_Output },
                                 if side == io_side::INPUT { AU_IN_BUS } else { AU_OUT_BUS },
                                 &enable,
                                 mem::size_of::<u32>());
    if rv != 0 {
        cubeb_log!("AudioUnitSetProperty/kAudioOutputUnitProperty_EnableIO rv={}", rv);
        return Err(Error::error());
    }
    Ok(())
}

fn audiounit_create_unit(unit: &mut AudioUnit, device: &device_info) -> Result<()>
{
    assert_eq!(*unit, ptr::null_mut());

    let mut rv: OSStatus = 0;
    audiounit_new_unit_instance(unit, device)?;
    assert_ne!(*unit, ptr::null_mut());

    if device.flags.contains(device_flags::DEV_SYSTEM_DEFAULT | device_flags::DEV_OUTPUT) {
        return Ok(());
    }

    if device.flags.contains(device_flags::DEV_INPUT) {
        if let Err(r) = audiounit_enable_unit_scope(unit, io_side::INPUT, enable_state::ENABLE) {
            cubeb_log!("Failed to enable audiounit input scope ");
            return Err(r);
        }
        if let Err(r) = audiounit_enable_unit_scope(unit, io_side::OUTPUT, enable_state::DISABLE) {
            cubeb_log!("Failed to disable audiounit output scope ");
            return Err(r);
        }
    } else if device.flags.contains(device_flags::DEV_OUTPUT) {
        if let Err(r) = audiounit_enable_unit_scope(unit, io_side::OUTPUT, enable_state::ENABLE) {
            cubeb_log!("Failed to enable audiounit output scope ");
            return Err(r);
        }
        if let Err(r) = audiounit_enable_unit_scope(unit, io_side::INPUT, enable_state::DISABLE) {
            cubeb_log!("Failed to disable audiounit input scope ");
            return Err(r);
        }
    } else {
        assert!(false);
    }

    rv = audio_unit_set_property(unit,
                                 kAudioOutputUnitProperty_CurrentDevice,
                                 kAudioUnitScope_Global,
                                 0,
                                 &device.id,
                                 mem::size_of::<AudioDeviceID>());
    if rv != 0 {
        cubeb_log!("AudioUnitSetProperty/kAudioOutputUnitProperty_CurrentDevice rv={}", rv);
        return Err(Error::error());
    }

    Ok(())
}

// TODO: 1. Change to audiounit_clamp_latency(stm: &mut AudioUnitStream)
//          latency_frames is actually equal to stm.latency_frames.
//       2. Merge the value clamp for boundary.
fn audiounit_clamp_latency(stm: &mut AudioUnitStream, latency_frames: u32) -> u32
{
    // For the 1st stream set anything within safe min-max
    assert!(audiounit_active_streams(stm.context) > 0);
    if audiounit_active_streams(stm.context) == 1 {
        return cmp::max(cmp::min(latency_frames, SAFE_MAX_LATENCY_FRAMES),
                        SAFE_MIN_LATENCY_FRAMES);
    }
    // TODO: Should we check this even for 1 stream case ?
    //       Do we need to set latency if there is no output unit ?
    assert_ne!(stm.output_unit, ptr::null_mut());

    // If more than one stream operates in parallel
    // allow only lower values of latency
    let mut r: OSStatus = 0;
    let mut output_buffer_size: UInt32 = 0;
    let mut size = mem::size_of_val(&output_buffer_size);
    // TODO: Why we check `output_unit` here? We already have an assertions above!
    if !stm.output_unit.is_null() {
        r = audio_unit_get_property(&stm.output_unit,
                                    kAudioDevicePropertyBufferFrameSize,
                                    kAudioUnitScope_Output,
                                    AU_OUT_BUS,
                                    &mut output_buffer_size,
                                    &mut size);
        if r != 0 {
            cubeb_log!("AudioUnitGetProperty/output/kAudioDevicePropertyBufferFrameSize rv={}", r);
            // TODO: Shouldn't it return something in range between
            //       SAFE_MIN_LATENCY_FRAMES and SAFE_MAX_LATENCY_FRAMES ?
            return 0;
        }

        output_buffer_size = cmp::max(cmp::min(output_buffer_size, SAFE_MAX_LATENCY_FRAMES),
                                      SAFE_MIN_LATENCY_FRAMES);
    }

    let mut input_buffer_size: UInt32 = 0;
    if !stm.input_unit.is_null() {
        r = audio_unit_get_property(&stm.input_unit,
                                    kAudioDevicePropertyBufferFrameSize,
                                    kAudioUnitScope_Input,
                                    AU_IN_BUS,
                                    &mut input_buffer_size,
                                    &mut size);
        if r != 0 {
            cubeb_log!("AudioUnitGetProperty/input/kAudioDevicePropertyBufferFrameSize rv={}", r);
            // TODO: Shouldn't it return something in range between
            //       SAFE_MIN_LATENCY_FRAMES and SAFE_MAX_LATENCY_FRAMES ?
            return 0;
        }

        input_buffer_size = cmp::max(cmp::min(input_buffer_size, SAFE_MAX_LATENCY_FRAMES),
                                     SAFE_MIN_LATENCY_FRAMES);
    }

    // Every following active streams can only set smaller latency
    let upper_latency_limit = if input_buffer_size != 0 && output_buffer_size != 0 {
        cmp::min(input_buffer_size, output_buffer_size)
    } else if input_buffer_size != 0 {
        input_buffer_size
    } else if output_buffer_size != 0 {
        output_buffer_size
    } else {
        SAFE_MAX_LATENCY_FRAMES
    };

    cmp::max(cmp::min(latency_frames, upper_latency_limit),
             SAFE_MIN_LATENCY_FRAMES)
}

fn audiounit_setup_stream(stm: &mut AudioUnitStream) -> Result<()>
{
    stm.mutex.assert_current_thread_owns();

    if stm.input_stream_params.prefs().contains(StreamPrefs::LOOPBACK) ||
       stm.output_stream_params.prefs().contains(StreamPrefs::LOOPBACK) {
        cubeb_log!("({:p}) Loopback not supported for audiounit.", stm);
        return Err(Error::not_supported());
    }

    let in_dev_info = stm.input_device.clone();
    let out_dev_info = stm.output_device.clone();

    if has_input(stm) && has_output(stm) &&
       stm.input_device.id != stm.output_device.id {
        // Create aggregate device ...
    }

    if has_input(stm) {
        if let Err(r) = audiounit_create_unit(&mut stm.input_unit, &in_dev_info) {
            cubeb_log!("({:p}) AudioUnit creation for input failed.", stm);
            return Err(r);
        }
    }

    if has_output(stm) {
        if let Err(r) = audiounit_create_unit(&mut stm.output_unit, &out_dev_info) {
            cubeb_log!("({:p}) AudioUnit creation for output failed.", stm);
            return Err(r);
        }
    }

    /* Latency cannot change if another stream is operating in parallel. In this case
     * latecy is set to the other stream value. */
    if audiounit_active_streams(stm.context) > 1 {
        cubeb_log!("({:p}) More than one active stream, use global latency.", stm);
        stm.latency_frames = stm.context.global_latency_frames;
    } else {
        /* Silently clamp the latency down to the platform default, because we
         * synthetize the clock from the callbacks, and we want the clock to update
         * often. */
        // Create a `latency_frames` here to avoid the borrowing issue.
        let latency_frames = stm.latency_frames;
        // TODO: Change `audiounit_clamp_latency` to audiounit_clamp_latency(stm)!
        stm.latency_frames = audiounit_clamp_latency(stm, latency_frames);
        assert!(stm.latency_frames > 0); // Ungly error check
        audiounit_set_global_latency(stm.context, stm.latency_frames);
    }

    Ok(())
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
    // TODO: devtype includes input, output, in-out, and unknown. This is a
    //       bad style to check type, although this function will early return
    //       for in-out and unknown type since audiounit_get_default_device_id
    //       will gives a kAudioObjectUnknown for unknown type.
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

// TODO: This actually is the name converted from the bytes of the data source
//       (kAudioDevicePropertyDataSource), rather than the name of the audio
//       device(kAudioObjectPropertyName). The naming here is vague.
fn audiounit_get_default_device_name(stm: &AudioUnitStream,
                                     device: &mut ffi::cubeb_device,
                                     devtype: DeviceType) -> Result<()>
{
    let mut data: u32 = 0;
    audiounit_get_default_device_datasource(devtype, &mut data)?;

    // TODO: devtype includes input, output, in-out, and unknown. This is a
    //       bad style to check type, although this function will early return
    //       for in-out and unknown type since
    //       audiounit_get_default_device_datasource will throw an error for
    //       in-out and unknown type.
    let name = if devtype == DeviceType::INPUT {
        &mut device.input_name
    } else {
        &mut device.output_name
    };
    // Leak the memory to the external code.
    *name = convert_uint32_into_string(data).into_raw();
    if name.is_null() {
        // TODO: Bad style to use scope as the above.
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
        return empty;
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
            // Cannot dereference *ptr to a AudioBufferList directly
            // since it's a variable-size struct: https://bit.ly/2CYFhJ0
            // `let list: = unsafe { *ptr }` will copy the `*ptr` whose type
            // is AudioBufferList to a list. However, it contains only one
            // `UInt32` and only one `AudioBuffer`, while the memory pointed
            // by `ptr` may have one `UInt32` and lots of `AudioBuffer`s.
            // See reference:
            // https://bit.ly/2O2MJE4
            let list: &AudioBufferList = unsafe { &(*ptr) };
            let ptr = list.mBuffers.as_ptr() as *const AudioBuffer;
            let len = list.mNumberBuffers as usize;
            if len == 0 {
                return 0;
            }
            let buffers = unsafe {
                slice::from_raw_parts(ptr, len)
            };
            for buffer in buffers {
                count += buffer.mNumberChannels;
            }
        }
    }
    count
}

// TODO: It seems that it works no matter what scope is(see test.rs). Is it ok?
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

// TODO: Rename to is_private_aggregate_device ?
//       Is it possible to have a public aggregate device ?
fn is_aggregate_device(device_info: &ffi::cubeb_device_info) -> bool
{
    assert!(!device_info.friendly_name.is_null());
    let private_name_ptr = PRIVATE_AGGREGATE_DEVICE_NAME.as_ptr() as *const c_char;
    unsafe {
        libc::strncmp(device_info.friendly_name, private_name_ptr,
                      libc::strlen(private_name_ptr)) == 0
    }
}

fn audiounit_get_devices_of_type(devtype: DeviceType) -> Vec<AudioObjectID>
{
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

    // Remove the aggregate device from the list of devices (if any).
    devices.retain(|&device| {
        let name = get_device_name(device);
        if name.is_null() {
            return true;
        }
        let private_device = cfstringref_from_static_string(PRIVATE_AGGREGATE_DEVICE_NAME);
        unsafe {
            let found = CFStringFind(name, private_device, 0).location;
            CFRelease(private_device as *const c_void);
            // TODO: release name here ? (Sync with C version here.)
            // CFRelease(name as *const c_void);
            found == kCFNotFound
        }
    });

    // devices.retain(|&device| {
    //     let name = get_device_name(device);
    //     let private_name = CString::new(PRIVATE_AGGREGATE_DEVICE_NAME).unwrap();
    //     name != private_name
    // });

    /* Expected sorted but did not find anything in the docs. */
    devices.sort();
    if devtype.contains(DeviceType::INPUT | DeviceType::OUTPUT) {
        return devices;
    }

    // FIXIT: This is wrong. We will use output scope when devtype
    //        is unknown. Change it after C version is updated!
    let scope = if devtype == DeviceType::INPUT {
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

extern fn audiounit_collection_changed_callback(_inObjectID: AudioObjectID,
                                                _inNumberAddresses: u32,
                                                _inAddresses: *const AudioObjectPropertyAddress,
                                                inClientData: *mut c_void) -> OSStatus
{
    let context = inClientData as *mut AudioUnitContext;

    // Rust compilter doesn't allow a pointer to be passed across threads.
    // A hacky way to do that is to cast the pointer into a value, then
    // the value, which is actually an address, can be copied into threads.
    let ctx_ptr = context as usize;

    unsafe {
        // This can be called from inside an AudioUnit function, dispatch to another queue.
        async_dispatch((*context).serial_queue, move || {
            // The scope of `lock` is a critical section.
            let ctx = ctx_ptr as *mut AudioUnitContext;
            let _lock = AutoLock::new(&mut (*ctx).mutex);

            if (*ctx).input_collection_changed_callback.is_none() &&
               (*ctx).output_collection_changed_callback.is_none() {
                return;
            }
            if (*ctx).input_collection_changed_callback.is_some() {
                let devices = audiounit_get_devices_of_type(DeviceType::INPUT);
                /* Elements in the vector expected sorted. */
                if (*ctx).input_device_array != devices {
                    (*ctx).input_device_array = devices;
                    (*ctx).input_collection_changed_callback.unwrap()(ctx as *mut _, (*ctx).input_collection_changed_user_ptr);
                }
            }
            if (*ctx).output_collection_changed_callback.is_some() {
                let devices = audiounit_get_devices_of_type(DeviceType::OUTPUT);
                /* Elements in the vector expected sorted. */
                if (*ctx).output_device_array != devices {
                    (*ctx).output_device_array = devices;
                    (*ctx).output_collection_changed_callback.unwrap()(ctx as *mut _, (*ctx).output_collection_changed_user_ptr);
                }
            }
        });
    }

    0 // noErr.
}

fn audiounit_add_device_listener(context: *mut AudioUnitContext,
                                 devtype: DeviceType,
                                 collection_changed_callback: ffi::cubeb_device_collection_changed_callback,
                                 user_ptr: *mut c_void) -> OSStatus
{
    unsafe {
        (*context).mutex.assert_current_thread_owns();
    }
    assert!(devtype.intersects(DeviceType::INPUT | DeviceType::OUTPUT));
    // TODO: We should add an assertion here! (Sync with C verstion.)
    // assert!(collection_changed_callback.is_some());
    unsafe {
        /* Note: second register without unregister first causes 'nope' error.
         * Current implementation requires unregister before register a new cb. */
        assert!(devtype.contains(DeviceType::INPUT) && (*context).input_collection_changed_callback.is_none() ||
                devtype.contains(DeviceType::OUTPUT) && (*context).output_collection_changed_callback.is_none());

        if (*context).input_collection_changed_callback.is_none() &&
           (*context).output_collection_changed_callback.is_none() {
            let ret = audio_object_add_property_listener(kAudioObjectSystemObject,
                                                         &DEVICES_PROPERTY_ADDRESS,
                                                         audiounit_collection_changed_callback,
                                                         context as *mut c_void);
            if ret != 0 {
                return ret;
            }
        }

        if devtype.contains(DeviceType::INPUT) {
            /* Expected empty after unregister. */
            assert!((*context).input_device_array.is_empty());
            (*context).input_device_array = audiounit_get_devices_of_type(DeviceType::INPUT);
            (*context).input_collection_changed_callback = collection_changed_callback;
            (*context).input_collection_changed_user_ptr = user_ptr;
        }

        if devtype.contains(DeviceType::OUTPUT) {
            /* Expected empty after unregister. */
            assert!((*context).output_device_array.is_empty());
            (*context).output_device_array = audiounit_get_devices_of_type(DeviceType::OUTPUT);
            (*context).output_collection_changed_callback = collection_changed_callback;
            (*context).output_collection_changed_user_ptr = user_ptr;
        }
    }

    0 // noErr.
}

fn audiounit_remove_device_listener(context: *mut AudioUnitContext, devtype: DeviceType) -> OSStatus
{
    unsafe {
        (*context).mutex.assert_current_thread_owns();
    }
    // TODO: We should add an assertion here! (Sync with C verstion.)
    // assert!(devtype.intersects(DeviceType::INPUT | DeviceType::OUTPUT));
    unsafe {
        if devtype.contains(DeviceType::INPUT) {
            (*context).input_collection_changed_callback = None;
            (*context).input_collection_changed_user_ptr = ptr::null_mut();
            (*context).input_device_array.clear();
        }

        if devtype.contains(DeviceType::OUTPUT) {
            (*context).output_collection_changed_callback = None;
            (*context).output_collection_changed_user_ptr = ptr::null_mut();
            (*context).output_device_array.clear();
        }

        if (*context).input_collection_changed_callback.is_some() ||
           (*context).output_collection_changed_callback.is_some() {
            return 0; // noErr.
        }
    }

    /* Note: unregister a non registered cb is not a problem, not checking. */
    audio_object_remove_property_listener(kAudioObjectSystemObject,
                                          &DEVICES_PROPERTY_ADDRESS,
                                          audiounit_collection_changed_callback,
                                          context as *mut c_void)
}

pub const OPS: Ops = capi_new!(AudioUnitContext, AudioUnitStream);

#[derive(Debug)]
pub struct AudioUnitContext {
    _ops: *const Ops,
    mutex: OwnedCriticalSection,
    active_streams: i32, // TODO: Shouldn't it be u32?
    global_latency_frames: u32,
    input_collection_changed_callback: ffi::cubeb_device_collection_changed_callback,
    input_collection_changed_user_ptr: *mut c_void,
    output_collection_changed_callback: ffi::cubeb_device_collection_changed_callback,
    output_collection_changed_user_ptr: *mut c_void,
    // Store list of devices to detect changes
    input_device_array: Vec<AudioObjectID>,
    output_device_array: Vec<AudioObjectID>,
    // The queue is asynchronously deallocated once all references to it are released
    serial_queue: dispatch_queue_t,
}

impl AudioUnitContext {
    fn new() -> Self {
        AudioUnitContext {
            _ops: &OPS as *const _,
            mutex: OwnedCriticalSection::new(),
            active_streams: 0,
            global_latency_frames: 0,
            input_collection_changed_callback: None,
            input_collection_changed_user_ptr: ptr::null_mut(),
            output_collection_changed_callback: None,
            output_collection_changed_user_ptr: ptr::null_mut(),
            input_device_array: Vec::new(),
            output_device_array: Vec::new(),
            serial_queue: create_dispatch_queue(
                DISPATCH_QUEUE_LABEL,
                DISPATCH_QUEUE_SERIAL
            )
        }
    }

    fn init(&mut self) {
        self.mutex.init();
    }
}

impl ContextOps for AudioUnitContext {
    fn init(_context_name: Option<&CStr>) -> Result<Context> {
        let mut ctx = Box::new(AudioUnitContext::new());
        ctx.init();
        Ok(unsafe { Context::from_ptr(Box::into_raw(ctx) as *mut _) })
    }

    fn backend_id(&mut self) -> &'static CStr {
        unsafe { CStr::from_ptr(b"audiounit-rust\0".as_ptr() as *const _) }
    }
    #[cfg(target_os = "ios")]
    fn max_channel_count(&mut self) -> Result<u32> {
        //TODO: [[AVAudioSession sharedInstance] maximumOutputNumberOfChannels]
        Ok(2u32)
    }
    #[cfg(not(target_os = "ios"))]
    fn max_channel_count(&mut self) -> Result<u32> {
        let mut size: usize = 0;
        let mut r: OSStatus = 0;
        let mut output_device_id: AudioDeviceID = kAudioObjectUnknown;
        let mut stream_format = AudioStreamBasicDescription::default();
        let stream_format_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreamFormat,
            mScope: kAudioDevicePropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMaster
        };

        output_device_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
        if output_device_id == kAudioObjectUnknown {
            return Err(Error::error());
        }

        size = mem::size_of_val(&stream_format);

        r = audio_object_get_property_data(output_device_id,
                                           &stream_format_address,
                                           &mut size,
                                           &mut stream_format);

        if r != 0 {
            cubeb_log!("AudioObjectPropertyAddress/StreamFormat rv={}", r);
            return Err(Error::error());
        }

        Ok(stream_format.mChannelsPerFrame)
    }
    #[cfg(target_os = "ios")]
    fn min_latency(&mut self, _params: StreamParams) -> Result<u32> {
        Err(not_supported());
    }
    #[cfg(not(target_os = "ios"))]
    fn min_latency(&mut self, _params: StreamParams) -> Result<u32> {
        let mut latency_range = AudioValueRange::default();
        if let Err(_) = audiounit_get_acceptable_latency_range(&mut latency_range) {
            cubeb_log!("Could not get acceptable latency range.");
            return Err(Error::error()); // TODO: return the error we get instead?
        }

        Ok(cmp::max(latency_range.mMinimum as u32,
                    SAFE_MIN_LATENCY_FRAMES))
    }
    #[cfg(target_os = "ios")]
    fn preferred_sample_rate(&mut self) -> Result<u32> {
        Err(not_supported());
    }
    #[cfg(not(target_os = "ios"))]
    fn preferred_sample_rate(&mut self) -> Result<u32> {
        let mut size: usize = 0;
        let mut r: OSStatus = 0;
        let mut fsamplerate: f64 = 0.0;
        let mut output_device_id: AudioDeviceID = kAudioObjectUnknown;
        let samplerate_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyNominalSampleRate,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMaster
        };

        output_device_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
        if output_device_id == kAudioObjectUnknown {
            return Err(Error::error());
        }

        size = mem::size_of_val(&fsamplerate);
        r = audio_object_get_property_data(output_device_id,
                                           &samplerate_address,
                                           &mut size,
                                           &mut fsamplerate);

        if r != 0 {
            return Err(Error::error());
        }

        Ok(fsamplerate as u32)
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
            let (ptr, len) = leak_vec(devices);
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
                    device.device_id = ptr::null_mut();
                }
                if !device.friendly_name.is_null() {
                    let _ = CString::from_raw(device.friendly_name as *mut _);
                    device.friendly_name = ptr::null_mut();
                }
                if !device.vendor_name.is_null() {
                    let _ = CString::from_raw(device.vendor_name as *mut _);
                    device.vendor_name = ptr::null_mut();
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
        input_device: DeviceId,
        input_stream_params: Option<&StreamParamsRef>,
        output_device: DeviceId,
        output_stream_params: Option<&StreamParamsRef>,
        latency_frames: u32,
        data_callback: ffi::cubeb_data_callback,
        state_callback: ffi::cubeb_state_callback,
        user_ptr: *mut c_void,
    ) -> Result<Stream> {
        // Since we cannot call `AutoLock::new(&mut self.mutex)` and
        // `AudioUnitStream::new(self, ...)` at the same time.
        // (`self` cannot be borrowed immutably after it's borrowed as mutable.),
        // we take the pointer to `self.mutex` first and then dereference it to
        // the mutex to avoid this problem for now.
        let mutex_ptr: *mut OwnedCriticalSection;
        {
            mutex_ptr = &mut self.mutex as *mut OwnedCriticalSection;
        }
        // The scope of `_context_lock` is a critical section.
        let _context_lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(self);
        let mut boxed_stream = Box::new(
            AudioUnitStream::new(
                self,
                user_ptr,
                data_callback,
                state_callback,
                latency_frames
            )
        );
        boxed_stream.init();
        // TODO: Shouldn't this be put at the first so we don't need to perform
        //       any action if the check fails? (Sync with C version)
        assert!(latency_frames > 0);
        // TODO: Shouldn't this be put at the first so we don't need to perform
        //       any action if the check fails? (Sync with C version)
        if (!input_device.is_null() && input_stream_params.is_none()) ||
           (!output_device.is_null() && output_stream_params.is_none()) {
            return Err(Error::invalid_parameter());
        }
        // TODO: Add a method `to_owned` in `StreamParamsRef`.
        if let Some(stream_params_ref) = input_stream_params {
            assert!(!stream_params_ref.as_ptr().is_null());
            boxed_stream.input_stream_params = StreamParams::from(unsafe { (*stream_params_ref.as_ptr()) });
            if let Err(r) = audiounit_set_device_info(boxed_stream.as_mut(), input_device as AudioDeviceID, DeviceType::INPUT) {
                cubeb_log!("({:p}) Fail to set device info for input.", boxed_stream.as_ref());
                return Err(r);
            }
        }
        if let Some(stream_params_ref) = output_stream_params {
            assert!(!stream_params_ref.as_ptr().is_null());
            boxed_stream.output_stream_params = StreamParams::from(unsafe { *(stream_params_ref.as_ptr()) });
            if let Err(r) = audiounit_set_device_info(boxed_stream.as_mut(), output_device as AudioDeviceID, DeviceType::OUTPUT) {
                cubeb_log!("({:p}) Fail to set device info for output.", boxed_stream.as_ref());
                return Err(r);
            }
        }

        if let Err(r) = {
            // It's not critical to lock here, because no other thread has been started
            // yet, but it allows to assert that the lock has been taken in
            // `audiounit_setup_stream`.

            // Since we cannot borrow boxed_stream as mutable twice
            // (for boxed_stream.mutex and boxed_stream itself), we store
            // the pointer to boxed_stream.mutex(it's a value) and convert it
            // to a reference as the workaround to borrow as mutable twice.
            // Same as what we did above for AudioUnitContext.mutex.
            let mutex_ptr: *mut OwnedCriticalSection;
            {
                mutex_ptr = &mut boxed_stream.mutex as *mut OwnedCriticalSection;
            }
            // The scope of `_lock` is a critical section.
            let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
            audiounit_setup_stream(boxed_stream.as_mut())
        } {
            cubeb_log!("({:p}) Could not setup the audiounit stream.", boxed_stream.as_ref());
            return Err(r);
        }

        if let Err(r) = audiounit_install_system_changed_callback(boxed_stream.as_mut()) {
            cubeb_log!("({:p}) Could not install the device change callback.", boxed_stream.as_ref());
            return Err(r);
        }

        println!("<Initialize> stream @ {:p}\nstream.context @ {:p}\n{:?}",
                 boxed_stream.as_ref(), boxed_stream.context, boxed_stream.as_ref());
        let cubeb_stream = unsafe {
            Stream::from_ptr(Box::into_raw(boxed_stream) as *mut _)
        };
        Ok(cubeb_stream)
    }
    fn register_device_collection_changed(
        &mut self,
        devtype: DeviceType,
        collection_changed_callback: ffi::cubeb_device_collection_changed_callback,
        user_ptr: *mut c_void,
    ) -> Result<()> {
        if devtype == DeviceType::UNKNOWN {
            return Err(Error::invalid_parameter());
        }
        let mut ret = 0; // noErr.
        let ctx_ptr = self as *mut AudioUnitContext;
        // The scope of `lock` is a critical section.
        let _lock = AutoLock::new(&mut self.mutex);
        if collection_changed_callback.is_some() {
            ret = audiounit_add_device_listener(ctx_ptr,
                                                devtype,
                                                collection_changed_callback,
                                                user_ptr);
        } else {
            ret = audiounit_remove_device_listener(ctx_ptr, devtype);
        }
        if ret == 0 {
            Ok(())
        } else {
            Err(Error::error())
        }
    }
}

#[derive(Debug)]
struct AudioUnitStream<'ctx> {
    context: &'ctx mut AudioUnitContext,
    user_ptr: *mut c_void,

    data_callback: ffi::cubeb_data_callback,
    state_callback: ffi::cubeb_state_callback,
    /* Stream creation parameters */
    input_stream_params: StreamParams,
    output_stream_params: StreamParams,
    input_device: device_info,
    output_device: device_info,
    /* I/O AudioUnits */
    input_unit: AudioUnit,
    output_unit: AudioUnit,
    mutex: OwnedCriticalSection,
    /* Latency requested by the user. */
    latency_frames: u32,
}

impl<'ctx> AudioUnitStream<'ctx> {
    fn new(
        context: &'ctx mut AudioUnitContext,
        user_ptr: *mut c_void,
        data_callback: ffi::cubeb_data_callback,
        state_callback: ffi::cubeb_state_callback,
        latency_frames: u32,
    ) -> Self {
        AudioUnitStream {
            context,
            user_ptr,
            data_callback,
            state_callback,
            input_stream_params: StreamParams::from(
                ffi::cubeb_stream_params {
                    format: ffi::CUBEB_SAMPLE_FLOAT32NE,
                    rate: 0,
                    channels: 0,
                    layout: ffi::CUBEB_LAYOUT_UNDEFINED,
                    prefs: ffi::CUBEB_STREAM_PREF_NONE
                }
            ),
            output_stream_params: StreamParams::from(
                ffi::cubeb_stream_params {
                    format: ffi::CUBEB_SAMPLE_FLOAT32NE,
                    rate: 0,
                    channels: 0,
                    layout: ffi::CUBEB_LAYOUT_UNDEFINED,
                    prefs: ffi::CUBEB_STREAM_PREF_NONE
                }
            ),
            input_device: device_info::new(),
            output_device: device_info::new(),
            input_unit: ptr::null_mut(),
            output_unit: ptr::null_mut(),
            mutex: OwnedCriticalSection::new(),
            latency_frames
        }
    }
    fn init(&mut self) {
        self.mutex.init();
    }
}

impl<'ctx> Drop for AudioUnitStream<'ctx> {
    fn drop(&mut self) {
        println!("<Drop> stream @ {:p}\nstream.context @ {:p}\n{:?}",
                 self, self.context, self);
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
