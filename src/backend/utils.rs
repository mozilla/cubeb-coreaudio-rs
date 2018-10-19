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
pub fn create_static_cfstring_ref(string: &'static str) -> sys::CFStringRef {
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
            address, // as `*const AudioObjectPropertyAddress` automatically.
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
            address, // as `*const AudioObjectPropertyAddress` automatically.
            0,
            ptr::null(),
            size as *mut sys::UInt32, // Cast raw usize pointer to raw u32 pointer.
            data as *mut c_void, // Cast raw T pointer to void pointer.
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
            address, // as `*const AudioObjectPropertyAddress` automatically.
            0,
            ptr::null(),
            size as *mut sys::UInt32, // Cast raw usize pointer to raw u32 pointer.
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
            address, // as `*const AudioObjectPropertyAddress` automatically.
            0,
            ptr::null(),
            size as sys::UInt32, // Cast usize variable to raw u32 variable.
            data as *const c_void, // Cast raw T pointer to void pointer.
        )
    }
}

#[test]
fn test_create_static_cfstring_ref() {
    use super::*;

    let cfstrref = create_static_cfstring_ref(PRIVATE_AGGREGATE_DEVICE_NAME);
    let cstring = audiounit_strref_to_cstr_utf8(cfstrref);

    assert_eq!(
        PRIVATE_AGGREGATE_DEVICE_NAME,
        cstring.into_string().unwrap()
    );

    // TODO: Find a way to check the string's inner pointer is same.
}
