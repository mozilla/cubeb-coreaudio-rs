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

fn test_stream_operation<F>(name: &'static str, operation: F)
where
    F: FnOnce(*mut ffi::cubeb_stream),
{
    use std::ffi::CString;

    let mut c: *mut ffi::cubeb = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.init.unwrap()(&mut c, ptr::null()) },
        ffi::CUBEB_OK
    );

    let mut stream: *mut ffi::cubeb_stream = ptr::null_mut();
    let stream_name = CString::new(name).expect("Failed on creating stream name");
    assert_eq!(
        unsafe {
            OPS.stream_init.unwrap()(
                c,
                &mut stream,
                stream_name.as_ptr(),
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
        ffi::CUBEB_OK
    );

    operation(stream);

    unsafe {
        OPS.stream_destroy.unwrap()(stream);
        OPS.destroy.unwrap()(c);
    };
}

#[test]
fn test_ops_context_stream_init_and_destroy() {
    test_stream_operation("stream init and destroy", |_stream| {});
}

#[test]
fn test_ops_stream_start() {
    test_stream_operation("stream start", |stream| {
        assert_eq!(
            unsafe { OPS.stream_start.unwrap()(stream) },
            ffi::CUBEB_ERROR_NOT_SUPPORTED
        );
    });
}

#[test]
fn test_ops_stream_stop() {
    test_stream_operation("stream stop", |stream| {
        assert_eq!(
            unsafe { OPS.stream_stop.unwrap()(stream) },
            ffi::CUBEB_ERROR_NOT_SUPPORTED
        );
    });
}

#[test]
fn test_ops_stream_reset_default_device() {
    test_stream_operation("stream reset default device", |stream| {
        assert_eq!(
            unsafe { OPS.stream_reset_default_device.unwrap()(stream) },
            ffi::CUBEB_ERROR_NOT_SUPPORTED
        );
    });
}

#[test]
fn test_ops_stream_position() {
    test_stream_operation("stream position", |stream| {
        let mut position = u64::max_value();
        assert_eq!(
            unsafe { OPS.stream_get_position.unwrap()(stream, &mut position) },
            ffi::CUBEB_OK
        );
        assert_eq!(position, 0);
    });
}

#[test]
fn test_ops_stream_latency() {
    test_stream_operation("stream latency", |stream| {
        let mut latency = u32::max_value();
        assert_eq!(
            unsafe { OPS.stream_get_latency.unwrap()(stream, &mut latency) },
            ffi::CUBEB_OK
        );
        assert_eq!(latency, 0);
    });
}

#[test]
fn test_ops_stream_set_volume() {
    test_stream_operation("stream set volume", |stream| {
        let mut latency = u32::max_value();
        assert_eq!(
            unsafe { OPS.stream_set_volume.unwrap()(stream, 0.5) },
            ffi::CUBEB_ERROR_NOT_SUPPORTED
        );
    });
}

#[test]
fn test_ops_stream_set_panning() {
    test_stream_operation("stream set panning", |stream| {
        assert_eq!(
            unsafe { OPS.stream_set_panning.unwrap()(stream, 0.5) },
            ffi::CUBEB_ERROR_NOT_SUPPORTED
        );
    });
}

#[test]
fn test_ops_stream_current_device() {
    test_stream_operation("stream current device", |stream| {
        let mut device: *mut ffi::cubeb_device = ptr::null_mut();
        assert_eq!(
            unsafe { OPS.stream_get_current_device.unwrap()(stream, &mut device) },
            ffi::CUBEB_OK
        );
        assert_eq!(device, 0xDEAD_BEEF as *mut ffi::cubeb_device);
    });
}

#[test]
fn test_ops_stream_device_destroy() {
    test_stream_operation("stream current device", |stream| {
        assert_eq!(
            unsafe {
                OPS.stream_device_destroy.unwrap()(stream, 0xDEAD_BEEF as *mut ffi::cubeb_device)
            },
            ffi::CUBEB_OK
        );
    });
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
