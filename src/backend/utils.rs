extern crate coreaudio_sys as sys;

use std::mem;
use std::os::raw::{c_char, c_void};
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


// References:
// http://rustaudio.github.io/coreaudio-rs/coreaudio_sys/audio_unit/index.html
// https://gist.github.com/ChunMinChang/8d13946ebc6c95b2622466c89a0c9bcc
#[test]
fn test_dispatch_async_f() {
    use std::{thread, time};

    let label = "Run with native dispatch apis";

    // https://github.com/phracker/MacOSX-SDKs/blob/9fc3ed0ad0345950ac25c28695b0427846eea966/MacOSX10.13.sdk/usr/include/dispatch/queue.h#L472
    const DISPATCH_QUEUE_SERIAL: sys::dispatch_queue_attr_t = 0 as sys::dispatch_queue_attr_t;

    // http://rustaudio.github.io/coreaudio-rs/coreaudio_sys/audio_unit/fn.dispatch_queue_create.html
    let queue = unsafe {
        sys::dispatch_queue_create(
            label.as_ptr() as *const c_char,
            DISPATCH_QUEUE_SERIAL
        )
    };

    // Allocate the `context` on heap, otherwise the `context` will be
    // freed before `work` is fired and after program goes out of the
    // scope of the unsafe block.
    let context: Box<i32> = Box::new(123);

    extern fn work(leaked_ptr: *mut c_void) {
        let leaked_context = leaked_ptr as *mut i32;

        // Retake the leaked `context`.
        let context = unsafe { Box::from_raw(leaked_context) };
        // println!("context: {}", context);
        assert_eq!(context.as_ref(), &123);

        // `context` is released after finishing this function call.
    }

    // http://rustaudio.github.io/coreaudio-rs/coreaudio_sys/audio_unit/fn.dispatch_async_f.html
    unsafe {
        sys::dispatch_async_f(
            queue,
            Box::into_raw(context) as *mut c_void, // Leak the `context`.
            Some(work)
        );
    }

    // // Wait 100 milliseconds to show the results.
    // let duration = time::Duration::from_millis(100);
    // thread::sleep(duration);
}
