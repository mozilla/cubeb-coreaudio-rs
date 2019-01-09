// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

use super::*;

// The following tests are sorted by the order of Ops in cubeb-backend.
#[test]
fn test_ops_context_init() {
    let mut c: *mut ffi::cubeb = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.init.unwrap()(&mut c, ptr::null()) },
        ffi::CUBEB_OK
    );
    unsafe {
        OPS.destroy.unwrap()(c);
    }
}

#[test]
fn test_ops_context_backend_id() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let backend = unsafe {
        let ptr = OPS.get_backend_id.unwrap()(c);
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    };
    assert_eq!(backend, "audiounit-rust");
}

#[test]
fn test_ops_context_max_channel_count() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut max_channel_count = u32::max_value();
    assert_eq!(
        unsafe { OPS.get_max_channel_count.unwrap()(c, &mut max_channel_count) },
        ffi::CUBEB_OK
    );
    assert_eq!(max_channel_count, 256);
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
    assert_eq!(latency, 256);
}

#[test]
fn test_ops_context_preferred_sample_rate() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut rate = u32::max_value();
    assert_eq!(
        unsafe { OPS.get_preferred_sample_rate.unwrap()(c, &mut rate) },
        ffi::CUBEB_OK
    );
    assert_eq!(rate, 48000);
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
        ffi::CUBEB_ERROR_NOT_SUPPORTED
    );
    assert_eq!(coll.device, 0xDEAD_BEEF as *mut ffi::cubeb_device_info);
    assert_eq!(coll.count, usize::max_value());
}

#[test]
fn test_ops_context_device_collection_destroy() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut coll = ffi::cubeb_device_collection {
        device: 0xDEAD_BEEF as *mut ffi::cubeb_device_info,
        count: usize::max_value(),
    };
    // capi_device_collection_destroy will return ffi::CUBEB_OK anyway
    // no matter what device_collection_destroy returns (we throw a
    // not_supported error in our implementation). Change CUBEB_OK
    // to CUBEB_ERROR_NOT_SUPPORTED after cubeb-rs is updated to the
    // newest version that fixes this problem.
    // https://github.com/djg/cubeb-rs/pull/37
    assert_eq!(
        unsafe { OPS.device_collection_destroy.unwrap()(c, &mut coll) },
        ffi::CUBEB_OK
    );
    assert_eq!(coll.device, ptr::null_mut());
    assert_eq!(coll.count, 0);
}

#[test]
fn test_ops_context_stream_init() {
    use std::ffi::CString;

    let mut c: *mut ffi::cubeb = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.init.unwrap()(&mut c, ptr::null()) },
        ffi::CUBEB_OK
    );

    let mut stream: *mut ffi::cubeb_stream = ptr::null_mut();
    let name = CString::new("test").unwrap().as_ptr();
    assert_eq!(
        unsafe {
            OPS.stream_init.unwrap()(
                c,
                &mut stream,
                name,
                ptr::null(),
                ptr::null_mut(),
                ptr::null(),
                ptr::null_mut(),
                4096,
                None,
                None,
                ptr::null_mut(),
            )
        },
        ffi::CUBEB_ERROR_NOT_SUPPORTED
    );

    unsafe {
        OPS.destroy.unwrap()(c);
    };
}

// The stream must be boxed since capi_stream_destroy releases the stream
// by Box::from_raw.
// stream_destroy: Some($crate::capi::capi_stream_destroy::<$stm>),

#[test]
fn test_ops_stream_start() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.stream_start.unwrap()(s) },
        ffi::CUBEB_ERROR_NOT_SUPPORTED
    );
}

#[test]
fn test_ops_stream_stop() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.stream_stop.unwrap()(s) },
        ffi::CUBEB_ERROR_NOT_SUPPORTED
    );
}

#[test]
fn test_ops_stream_reset_default_device() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.stream_reset_default_device.unwrap()(s) },
        ffi::CUBEB_ERROR_NOT_SUPPORTED
    );
}

#[test]
fn test_ops_stream_position() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    let mut position = u64::max_value();
    assert_eq!(
        unsafe { OPS.stream_get_position.unwrap()(s, &mut position) },
        ffi::CUBEB_OK
    );
    assert_eq!(position, 0);
}

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
    assert_eq!(
        unsafe { OPS.stream_set_volume.unwrap()(s, 0.5) },
        ffi::CUBEB_ERROR_NOT_SUPPORTED
    );
}

#[test]
fn test_ops_stream_set_panning() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.stream_set_panning.unwrap()(s, 0.5) },
        ffi::CUBEB_ERROR_NOT_SUPPORTED
    );
}

#[test]
fn test_ops_stream_current_device() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    let mut device: *mut ffi::cubeb_device = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.stream_get_current_device.unwrap()(s, &mut device) },
        ffi::CUBEB_OK
    );
    assert_eq!(device, 0xDEAD_BEEF as *mut ffi::cubeb_device);
}

#[test]
fn test_ops_stream_device_destroy() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    unsafe {
        OPS.stream_device_destroy.unwrap()(s, 0xDEAD_BEEF as *mut ffi::cubeb_device);
    }
}

// Enable this after cubeb-rs is updated to the newest version that
// implements stream_register_device_changed_callback operation.
// https://github.com/djg/cubeb-rs/pull/36
// #[test]
// fn test_ops_stream_register_device_changed_callback() {
//     let s: *mut ffi::cubeb_stream = ptr::null_mut();
//     extern "C" fn callback(_: *mut c_void) {}
//     assert_eq!(
//         unsafe {
//             OPS.stream_register_device_changed_callback.unwrap()(
//                 s,
//                 Some(callback)
//             )
//         },
//         ffi::CUBEB_ERROR_NOT_SUPPORTED
//     );
// }

#[test]
fn test_ops_context_register_device_collection_changed() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    extern "C" fn callback(_: *mut ffi::cubeb, _: *mut c_void) {}
    assert_eq!(
        unsafe {
            OPS.register_device_collection_changed.unwrap()(
                c,
                ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT,
                Some(callback),
                0xDEAD_BEEF as *mut c_void,
            )
        },
        ffi::CUBEB_ERROR_NOT_SUPPORTED
    );
}
