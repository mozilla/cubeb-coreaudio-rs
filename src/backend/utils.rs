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

#[test]
fn test_audio_object_add_property_listener_for_unknown_device() {
    use super::DEVICES_PROPERTY_ADDRESS;

    extern fn listener(
        id: sys::AudioObjectID,
        number_of_addresses :u32,
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
    use std::slice;

    let mut called: u32 = 0;

    extern fn listener(
        id: sys::AudioObjectID,
        number_of_addresses: u32,
        addresses: *const sys::AudioObjectPropertyAddress,
        data: *mut c_void
    ) -> sys::OSStatus {
        let addrs = unsafe {
            slice::from_raw_parts(addresses, number_of_addresses as usize)
        };
        // TODO: Find a way to test the case for number_of_addresses > 1.
        for (i, addr) in addrs.iter().enumerate() {
            println!("device {} > address {}: selector {}, scope {}, element {}",
                      id, i, addr.mSelector, addr.mScope, addr.mElement);
        }
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
