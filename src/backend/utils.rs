// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

extern crate coreaudio_sys as sys;

use std::mem;
use std::os::raw::c_void;
use std::ptr;

pub fn allocate_array_by_size<T>(size: usize) -> Vec<T> {
    let elements = size / mem::size_of::<T>();
    allocate_array::<T>(elements)
}

pub fn allocate_array<T>(elements: usize) -> Vec<T> {
    let mut array = Vec::<T>::with_capacity(elements);
    unsafe {
        array.set_len(elements);
    }
    array
}

pub fn leak_vec<T>(mut v: Vec<T>) -> (*mut T, usize) {
    v.shrink_to_fit(); // Make sure the capacity is same as the length.
    let ptr_and_len = (v.as_mut_ptr(), v.len());
    mem::forget(v); // Leak the memory to the external code.
    ptr_and_len
}

pub fn retake_leaked_vec<T>(ptr: *mut T, len: usize) -> Vec<T> {
    unsafe {
        Vec::from_raw_parts(
            ptr,
            len,
            len
        )
    }
}

// CFSTR doesn't be implemented in core-foundation-sys, so we create a function
// to replace it.
pub fn cfstringref_from_static_string(string: &'static str) -> sys::CFStringRef {
    // References:
    // https://developer.apple.com/documentation/corefoundation/1543597-cfstringcreatewithbytesnocopy?language=objc
    // https://github.com/opensource-apple/CF/blob/3cc41a76b1491f50813e28a4ec09954ffa359e6f/CFString.c#L1605
    // https://github.com/servo/core-foundation-rs/blob/2aac8fb85b5b114673280e273c04219c0c360e54/core-foundation/src/string.rs#L125
    // https://github.com/servo/core-foundation-rs/blob/2aac8fb85b5b114673280e273c04219c0c360e54/io-surface/src/lib.rs#L48
    // Set deallocator to kCFAllocatorNull to prevent the the memory of the
    // parameter `string` from being released by CFRelease.
    // We manage the string memory by ourselves.
    unsafe {
        sys::CFStringCreateWithBytesNoCopy(
            sys::kCFAllocatorDefault,
            string.as_ptr(),
            string.len() as sys::CFIndex,
            sys::kCFStringEncodingUTF8,
            false as sys::Boolean,
            sys::kCFAllocatorNull
        )
    }
}

// pub fn cfstringref_from_string(string: &str) -> sys::CFStringRef {
//     // References:
//     // https://developer.apple.com/documentation/corefoundation/1543419-cfstringcreatewithbytes?language=objc
//     // https://github.com/opensource-apple/CF/blob/3cc41a76b1491f50813e28a4ec09954ffa359e6f/CFString.c#L1597
//     // https://github.com/servo/core-foundation-rs/blob/2aac8fb85b5b114673280e273c04219c0c360e54/core-foundation/src/string.rs#L111
//     // https://github.com/servo/core-foundation-rs/blob/2aac8fb85b5b114673280e273c04219c0c360e54/io-surface/src/lib.rs#L48
//     unsafe {
//         sys::CFStringCreateWithBytes(
//             sys::kCFAllocatorDefault,
//             string.as_ptr(),
//             string.len() as sys::CFIndex,
//             sys::kCFStringEncodingUTF8,
//             false as sys::Boolean
//         )
//     }
// }

pub fn audio_object_has_property(
    id: sys::AudioObjectID,
    address: &sys::AudioObjectPropertyAddress,
) -> bool {
    unsafe {
        sys::AudioObjectHasProperty(
            id,
            address,
        ) != 0
    }
}

pub fn audio_object_get_property_data<T>(
    id: sys::AudioObjectID,
    address: &sys::AudioObjectPropertyAddress,
    size: *mut usize,
    data: *mut T,
) -> sys::OSStatus {
    unsafe {
        sys::AudioObjectGetPropertyData(
            id,
            address,
            0,
            ptr::null(),
            size as *mut sys::UInt32,
            data as *mut c_void,
        )
    }
}

pub fn audio_object_get_property_data_size(
    id: sys::AudioObjectID,
    address: &sys::AudioObjectPropertyAddress,
    size: *mut usize,
) -> sys::OSStatus {
    unsafe {
        sys::AudioObjectGetPropertyDataSize(
            id,
            address,
            0,
            ptr::null(),
            size as *mut sys::UInt32,
        )
    }
}

pub fn audio_object_set_property_data<T>(
    id: sys::AudioObjectID,
    address: &sys::AudioObjectPropertyAddress,
    size: usize,
    data: *const T,
) -> sys::OSStatus {
    unsafe {
        sys::AudioObjectSetPropertyData(
            id,
            address,
            0,
            ptr::null(),
            size as sys::UInt32,
            data as *const c_void,
        )
    }
}

// Referece:
// https://gist.github.com/ChunMinChang/f0f4a71f78d1e1c6390493ab1c9d10d3
pub type audio_object_property_listener_proc = extern fn(
    sys::AudioObjectID,
    u32,
    *const sys::AudioObjectPropertyAddress,
    *mut c_void,
) -> sys::OSStatus;

pub fn audio_object_add_property_listener(
    id: sys::AudioObjectID,
    address: &sys::AudioObjectPropertyAddress,
    listener: audio_object_property_listener_proc,
    data: *mut c_void,
) -> sys::OSStatus {
    unsafe {
        sys::AudioObjectAddPropertyListener(
            id,
            address,
            Some(listener),
            data
        )
    }
}

pub fn audio_object_remove_property_listener(
    id: sys::AudioObjectID,
    address: &sys::AudioObjectPropertyAddress,
    listener: audio_object_property_listener_proc,
    data: *mut c_void,
) -> sys::OSStatus {
    unsafe {
        sys::AudioObjectRemovePropertyListener(
            id,
            address,
            Some(listener),
            data
        )
    }
}

pub fn audio_unit_get_property<T>(
    unit: &sys::AudioUnit,
    property: sys::AudioUnitPropertyID,
    scope: sys::AudioUnitScope,
    element: sys::AudioUnitElement,
    data: *mut T,
    size: *mut usize,
) -> sys::OSStatus {
    unsafe {
        sys::AudioUnitGetProperty(
            *unit,
            property,
            scope,
            element,
            data as *mut c_void,
            size as *mut sys::UInt32
        )
    }
}

pub fn audio_unit_set_property<T>(
    unit: &sys::AudioUnit,
    property: sys::AudioUnitPropertyID,
    scope: sys::AudioUnitScope,
    element: sys::AudioUnitElement,
    data: *const T,
    size: usize,
) -> sys::OSStatus {
    unsafe {
        sys::AudioUnitSetProperty(
            *unit,
            property,
            scope,
            element,
            data as *const c_void,
            size as sys::UInt32,
        )
    }
}

pub fn audio_unit_get_parameter(
    unit: &sys::AudioUnit,
    id: sys:: AudioUnitParameterID,
    scope: sys::AudioUnitScope,
    element: sys::AudioUnitElement,
    value: &mut sys::AudioUnitParameterValue,
) -> sys::OSStatus {
    unsafe {
        sys::AudioUnitGetParameter(
            *unit,
            id,
            scope,
            element,
            value as *mut sys::AudioUnitParameterValue
        )
    }
}

// https://developer.apple.com/documentation/audiotoolbox/1440111-audiounitaddpropertylistener?language=objc
pub type audio_unit_property_listener_proc = extern fn(
    *mut c_void,
    sys::AudioUnit,
    sys::AudioUnitPropertyID,
    sys::AudioUnitScope,
    sys::AudioUnitElement
);

pub fn audio_unit_add_property_listener(
    unit: &sys::AudioUnit,
    id: sys::AudioUnitPropertyID,
    listener: audio_unit_property_listener_proc,
    data: *mut c_void,
) -> sys::OSStatus {
    unsafe {
        sys::AudioUnitAddPropertyListener(
            *unit,
            id,
            Some(listener),
            data
        )
    }
}

pub fn audio_unit_remove_property_listener_with_user_data(
    unit: &sys::AudioUnit,
    id: sys::AudioUnitPropertyID,
    listener: audio_unit_property_listener_proc,
    data: *mut c_void,
) -> sys::OSStatus {
    unsafe {
        sys::AudioUnitRemovePropertyListenerWithUserData(
            *unit,
            id,
            Some(listener),
            data
        )
    }
}

pub fn audio_unit_set_parameter(
    unit: &sys::AudioUnit,
    id: sys:: AudioUnitParameterID,
    scope: sys::AudioUnitScope,
    element: sys::AudioUnitElement,
    value: sys::AudioUnitParameterValue,
    buffer_offset_in_frames: sys::UInt32,
) -> sys::OSStatus {
    unsafe {
        sys::AudioUnitSetParameter(
            *unit,
            id,
            scope,
            element,
            value,
            buffer_offset_in_frames
        )
    }
}

pub fn audio_unit_render(
    inUnit: sys::AudioUnit,
    ioActionFlags: *mut sys::AudioUnitRenderActionFlags,
    inTimeStamp: *const sys::AudioTimeStamp,
    inOutputBusNumber: u32,
    inNumberFrames: u32,
    ioData: *mut sys::AudioBufferList
) -> sys::OSStatus {
    unsafe {
        sys::AudioUnitRender(
            inUnit,
            ioActionFlags,
            inTimeStamp,
            inOutputBusNumber,
            inNumberFrames,
            ioData
        )
    }
}

pub fn audio_unit_initialize(
    unit: &sys::AudioUnit,
) -> sys::OSStatus {
    unsafe {
        sys::AudioUnitInitialize(*unit)
    }
}

// TODO: Maybe we can merge the following two functions into something like
//       `destroy_audio_unit(unit: &sys::AudioUnit)` and call
//        `AudioUnitUninitialize`, `AudioComponentInstanceDispose` in this
//        function.
pub fn audio_unit_uninitialize(
    unit: &sys::AudioUnit,
) -> sys::OSStatus {
    unsafe {
        sys::AudioUnitUninitialize(*unit)
    }
}

pub fn dispose_audio_unit(
    unit: &sys::AudioUnit,
) -> sys::OSStatus {
    unsafe {
        sys::AudioComponentInstanceDispose(*unit)
    }
}

pub fn audio_output_unit_start(
    unit: sys::AudioUnit,
) -> sys::OSStatus {
    unsafe {
       sys::AudioOutputUnitStart(unit)
    }
}

pub fn audio_output_unit_stop(
    unit: sys::AudioUnit,
) -> sys::OSStatus {
    unsafe {
       sys::AudioOutputUnitStop(unit)
    }
}

pub fn show_callback_info(
    id: sys::AudioObjectID,
    number_of_addresses: u32,
    addresses: *const sys::AudioObjectPropertyAddress,
    data: *mut c_void) {
    use std::slice;

    println!("\n\n---------------------\ndevice: {}, data @ {:p}", id, data);
    let addrs = unsafe {
        slice::from_raw_parts(addresses, number_of_addresses as usize)
    };
    for (i, addr) in addrs.iter().enumerate() {
        println!("address {}\n\tselector {}\n\tscope {}\n\telement {}",
                 i, addr.mSelector, addr.mScope, addr.mElement);
    }
    println!("---------------------\n\n");
}

#[test]
fn test_create_static_cfstring_ref() {
    use super::*;

    let cfstrref = cfstringref_from_static_string(PRIVATE_AGGREGATE_DEVICE_NAME);
    let cstring = audiounit_strref_to_cstr_utf8(cfstrref);
    unsafe {
        CFRelease(cfstrref as *const c_void);
    }

    assert_eq!(
        PRIVATE_AGGREGATE_DEVICE_NAME,
        cstring.into_string().unwrap()
    );

    // TODO: Find a way to check the string's inner pointer is same.
}

// #[test]
// fn test_create_cfstring_ref() {
//     use super::*;

//     let test_string = "Rustaceans 🦀";
//     let cfstrref = cfstringref_from_string(test_string);
//     let cstring = audiounit_strref_to_cstr_utf8(cfstrref);
//     unsafe {
//         CFRelease(cfstrref as *const c_void);
//     }

//     assert_eq!(
//         test_string,
//         cstring.to_string_lossy()
//     );

//     // TODO: Find a way to check the string's inner pointer is different.
// }

#[test]
fn test_audio_object_add_property_listener_for_unknown_device() {
    use super::DEVICES_PROPERTY_ADDRESS;

    extern fn listener(
        id: sys::AudioObjectID,
        number_of_addresses: u32,
        addresses: *const sys::AudioObjectPropertyAddress,
        data: *mut c_void
    ) -> sys::OSStatus {
        assert!(false, "Should not be called.");
        sys::kAudioHardwareUnspecifiedError as sys::OSStatus
    }

    assert_eq!(
        audio_object_add_property_listener(
            sys::kAudioObjectUnknown,
            &DEVICES_PROPERTY_ADDRESS,
            listener,
            ptr::null_mut(),
        ),
        sys::kAudioHardwareBadObjectError as sys::OSStatus
    );
}

#[test]
fn test_audio_object_remove_property_listener_for_unknown_device() {
    use super::DEVICES_PROPERTY_ADDRESS;

    extern fn listener(
        _: sys::AudioObjectID,
        _: u32,
        _: *const sys::AudioObjectPropertyAddress,
        _: *mut c_void
    ) -> sys::OSStatus {
        assert!(false, "Should not be called.");
        sys::kAudioHardwareUnspecifiedError as sys::OSStatus
    }

    assert_eq!(
        audio_object_remove_property_listener(
            sys::kAudioObjectUnknown,
            &DEVICES_PROPERTY_ADDRESS,
            listener,
            ptr::null_mut(),
        ),
        sys::kAudioHardwareBadObjectError as sys::OSStatus
    );
}

#[test]
fn test_audio_object_remove_property_listener_without_adding_any_listener() {
    use super::DEVICES_PROPERTY_ADDRESS;

    extern fn listener(
        _: sys::AudioObjectID,
        _: u32,
        _: *const sys::AudioObjectPropertyAddress,
        _: *mut c_void
    ) -> sys::OSStatus {
        assert!(false, "Should not be called.");
        sys::kAudioHardwareUnspecifiedError as sys::OSStatus
    }

    // It's ok to remove listener that is never registered for the system device.
    assert_eq!(
        audio_object_remove_property_listener(
            sys::kAudioObjectSystemObject,
            &DEVICES_PROPERTY_ADDRESS,
            listener,
            ptr::null_mut(),
        ),
        0
    )
}

#[test]
fn test_audio_object_add_then_remove_property_listener() {
    use super::DEVICES_PROPERTY_ADDRESS;

    extern fn listener(
        _: sys::AudioObjectID,
        _: u32,
        _: *const sys::AudioObjectPropertyAddress,
        _: *mut c_void
    ) -> sys::OSStatus {
        assert!(false, "Should not be called.");
        sys::kAudioHardwareUnspecifiedError as sys::OSStatus
    }

    assert_eq!(
        audio_object_add_property_listener(
            sys::kAudioObjectSystemObject,
            &DEVICES_PROPERTY_ADDRESS,
            listener,
            ptr::null_mut(),
        ),
        0
    );

    assert_eq!(
        audio_object_remove_property_listener(
            sys::kAudioObjectSystemObject,
            &DEVICES_PROPERTY_ADDRESS,
            listener,
            ptr::null_mut(),
        ),
        0
    );
}

#[test]
#[ignore]
fn test_manual_audio_object_add_property_listener() {
    use super::DEVICES_PROPERTY_ADDRESS;

    let mut called: u32 = 0;

    extern fn listener(
        id: sys::AudioObjectID,
        number_of_addresses: u32,
        addresses: *const sys::AudioObjectPropertyAddress,
        data: *mut c_void
    ) -> sys::OSStatus {
        show_callback_info(id, number_of_addresses, addresses, data);
        let called = unsafe {
            &mut (*(data as *mut u32))
        };
        *called += 1;

        0 // noErr.
    }

    let r = audio_object_add_property_listener(
        sys::kAudioObjectSystemObject,
        &DEVICES_PROPERTY_ADDRESS,
        listener,
        &mut called as *mut u32 as *mut c_void,
    );
    assert_eq!(r, 0);

    while called == 0 {};

    let r = audio_object_remove_property_listener(
        sys::kAudioObjectSystemObject,
        &DEVICES_PROPERTY_ADDRESS,
        listener,
        &mut called as *mut u32 as *mut c_void,
    );
    assert_eq!(r, 0);

    // Since this function never ends, we can make sure `called` exists
    // when listener is called!
}

#[test]
fn test_audio_unit_add_property_listener_for_null_unit() {
    extern fn listener(
        _: *mut c_void,
        _: sys::AudioUnit,
        _: sys::AudioUnitPropertyID,
        _: sys::AudioUnitScope,
        _: sys::AudioUnitElement
    ) {
        assert!(false, "Should not be called.");
    }

    let unit = ptr::null_mut();
    assert_eq!(
        audio_unit_add_property_listener(
            &unit,
            sys::kAudioDevicePropertyBufferFrameSize,
            listener,
            ptr::null_mut()
        ),
        sys::kAudio_ParamError
    );
}


#[test]
fn test_audio_unit_remove_property_listener_with_user_data_for_null_unit() {
    extern fn listener(
        _: *mut c_void,
        _: sys::AudioUnit,
        _: sys::AudioUnitPropertyID,
        _: sys::AudioUnitScope,
        _: sys::AudioUnitElement
    ) {
        assert!(false, "Should not be called.");
    }

    let unit = ptr::null_mut();
    assert_eq!(
        audio_unit_remove_property_listener_with_user_data(
            &unit,
            sys::kAudioDevicePropertyBufferFrameSize,
            listener,
            ptr::null_mut()
        ),
        sys::kAudio_ParamError
    );
}

#[test]
fn test_audio_unit_remove_property_listener_with_user_data_without_adding_any() {
    extern fn listener(
        _: *mut c_void,
        _: sys::AudioUnit,
        _: sys::AudioUnitPropertyID,
        _: sys::AudioUnitScope,
        _: sys::AudioUnitElement
    ) {
        assert!(false, "Should not be called.");
    }

    let default_device = get_default_input_or_output_device();
    let mut unit = ptr::null_mut();
    super::audiounit_create_unit(&mut unit, &default_device).unwrap();
    assert!(!unit.is_null());

    assert_eq!(
        audio_unit_remove_property_listener_with_user_data(
            &unit,
            sys::kAudioDevicePropertyBufferFrameSize,
            listener,
            ptr::null_mut()
        ),
        0
    );
}

#[test]
fn test_audio_unit_add_then_remove_property_listener() {
    extern fn listener(
        _: *mut c_void,
        _: sys::AudioUnit,
        _: sys::AudioUnitPropertyID,
        _: sys::AudioUnitScope,
        _: sys::AudioUnitElement
    ) {
        assert!(false, "Should not be called.");
    }

    let default_device = get_default_input_or_output_device();
    let mut unit = ptr::null_mut();
    super::audiounit_create_unit(&mut unit, &default_device).unwrap();
    assert!(!unit.is_null());

    assert_eq!(
        audio_unit_add_property_listener(
            &unit,
            sys::kAudioDevicePropertyBufferFrameSize,
            listener,
            ptr::null_mut()
        ),
        0
    );

    assert_eq!(
        audio_unit_remove_property_listener_with_user_data(
            &unit,
            sys::kAudioDevicePropertyBufferFrameSize,
            listener,
            ptr::null_mut()
        ),
        0
    );
}

#[test]
fn test_audio_unit_add_then_fire_then_remove_property_listener() {
    // Uncomment the following println to show the logs.
    macro_rules! debug_println {
        ($( $args:expr ),*) => {
            // println!( $( $args ),* );
        }
    }

    use super::device_flags;
    const GLOBAL_ELEMENT: sys::AudioUnitElement = 0;
    const OUT_ELEMENT: sys::AudioUnitElement = 0;
    const IN_ELEMENT: sys::AudioUnitElement = 1;

    let mut called: u32 = 0;

    extern fn listener(
        data: *mut c_void,
        unit: sys::AudioUnit,
        id: sys::AudioUnitPropertyID,
        scope: sys::AudioUnitScope,
        element: sys::AudioUnitElement
    ) {
        // This callback will be fired twice. One for input or output scope,
        // the other is for global scope.
        debug_println!("listener > id: {}, unit: {:?}, scope: {}, element: {}, data @ {:p}",
            id, unit, scope, element, data);

        assert_eq!(
            id,
            sys::kAudioDevicePropertyBufferFrameSize
        );

        assert!((scope == sys::kAudioUnitScope_Output && element == IN_ELEMENT) ||
                (scope == sys::kAudioUnitScope_Input && element == OUT_ELEMENT) ||
                (scope == sys::kAudioUnitScope_Global && element == GLOBAL_ELEMENT));

        let mut buffer_frames: u32 = 0;
        let mut size = mem::size_of::<u32>();
        assert_eq!(
            audio_unit_get_property(
                &unit,
                id,
                scope,
                element,
                &mut buffer_frames,
                &mut size
            ),
            0
        );

        debug_println!("updated {} buffer frames: {}",
            if element == IN_ELEMENT { "input" } else { "output" }, buffer_frames);

        let called = unsafe {
            &mut (*(data as *mut u32))
        };
        *called += 1;

        // It's ok to remove listener here.
        // assert_eq!(
        //     audio_unit_remove_property_listener_with_user_data(
        //         &unit,
        //         id,
        //         listener,
        //         data
        //     ),
        //     0
        // );
    }

    let default_device = get_default_input_or_output_device();
    if default_device.id == sys::kAudioObjectUnknown ||
       default_device.flags == device_flags::DEV_UNKNOWN {
        return;
    }

    assert!(default_device.flags.intersects(device_flags::DEV_INPUT | device_flags::DEV_OUTPUT));
    assert!(!default_device.flags.contains(device_flags::DEV_INPUT | device_flags::DEV_OUTPUT));

    let is_input = if default_device.flags.contains(device_flags::DEV_INPUT) {
        true
    } else {
        false
    };

    let mut unit = ptr::null_mut();
    super::audiounit_create_unit(&mut unit, &default_device).unwrap();
    assert!(!unit.is_null());

    let mut buffer_frames: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            &unit,
            sys::kAudioDevicePropertyBufferFrameSize,
            if is_input { sys::kAudioUnitScope_Output } else { sys::kAudioUnitScope_Input },
            if is_input { IN_ELEMENT } else { OUT_ELEMENT },
            &mut buffer_frames,
            &mut size
        ),
        0
    );

    debug_println!("current {} buffer frames: {}",
        if is_input { "input" } else { "output" }, buffer_frames);

    assert_eq!(
        audio_unit_add_property_listener(
            &unit,
            sys::kAudioDevicePropertyBufferFrameSize,
            listener,
            &mut called as *mut u32 as *mut c_void
        ),
        0
    );

    // Make sure buffer_frames will be set to a new value.
    assert_ne!(buffer_frames, 0);
    buffer_frames *= 2;
    debug_println!("target {} buffer frames: {}",
        if is_input { "input" } else { "output" }, buffer_frames);

    assert_eq!(
        audio_unit_set_property(
            &unit,
            sys::kAudioDevicePropertyBufferFrameSize,
            if is_input { sys::kAudioUnitScope_Output } else { sys::kAudioUnitScope_Input },
            if is_input { IN_ELEMENT } else { OUT_ELEMENT },
            &buffer_frames,
            size
        ),
        0
    );

    while called < 2 {};

    assert_eq!(
        audio_unit_remove_property_listener_with_user_data(
            &unit,
            sys::kAudioDevicePropertyBufferFrameSize,
            listener,
            &mut called as *mut u32 as *mut c_void
        ),
        0
    );

    assert_eq!(called, 2);
}

fn get_default_input_or_output_device() -> super::device_info {
    use super::{
        audiounit_get_default_device_id,
        device_flags,
        device_info,
        DeviceType,
    };

    let mut device = device_info::new();
    assert_eq!(
        device.id,
        sys::kAudioObjectUnknown
    );
    assert_eq!(
        device.flags,
        device_flags::DEV_UNKNOWN
    );

    // let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    let default_output_id = sys::kAudioObjectUnknown;
    let default_input_id = audiounit_get_default_device_id(DeviceType::INPUT);

    if default_output_id != sys::kAudioObjectUnknown {
        device.flags |= device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT;
        device.id = default_output_id;
    } else if default_input_id != sys::kAudioObjectUnknown {
        device.flags |= device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT;
        device.id = default_input_id;
    }

    device
}
