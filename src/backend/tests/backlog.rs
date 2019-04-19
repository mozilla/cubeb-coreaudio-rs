// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.
use super::utils::test_get_empty_stream;
use super::*;

// Interface
// ============================================================================
// A panic in `capi_register_device_collection_changed` causes
// `EXC_BAD_INSTRUCTION` on my MacBook Air but it's fine on my MacBook Pro.
// It'w weird that it works fine if replacing
// `register_device_collection_changed: Option<unsafe extern "C" fn(..,) -> c_int>`
// to `register_device_collection_changed: unsafe extern "C" fn(..,) -> c_int`
// Test them in `AudioUnitContext` directly instead of calling them via `OPS` for now.
fn test_context_register_device_collection_changed_twice(devtype: DeviceType) {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext, since those OwnedCriticalSection
    // will be used when register_device_collection_changed is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    extern "C" fn callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    assert!(
        ctx.register_device_collection_changed(
            devtype,
            Some(callback),
            ptr::null_mut()
        ).is_ok();
    );

    assert!(
        ctx.register_device_collection_changed(
            devtype,
            Some(callback),
            ptr::null_mut()
        ).is_err();
    );
}

#[test]
#[should_panic]
fn test_context_register_device_collection_changed_twice_input() {
    test_context_register_device_collection_changed_twice(DeviceType::INPUT);
}

#[test]
#[should_panic]
fn test_context_register_device_collection_changed_twice_output() {
    test_context_register_device_collection_changed_twice(DeviceType::OUTPUT);
}

#[test]
#[should_panic]
fn test_context_register_device_collection_changed_twice_inout() {
    test_context_register_device_collection_changed_twice(DeviceType::INPUT | DeviceType::OUTPUT);
}

#[test]
fn test_stream_register_device_changed_callback() {
    extern "C" fn callback(_: *mut c_void) {}

    test_get_empty_stream(|stream| {
        assert!(stream
            .register_device_changed_callback(Some(callback))
            .is_ok());
        assert!(stream.register_device_changed_callback(None).is_ok());
    });
}

#[test]
#[should_panic]
fn test_stream_register_device_changed_callback_twice() {
    extern "C" fn callback1(_: *mut c_void) {}
    extern "C" fn callback2(_: *mut c_void) {}

    test_get_empty_stream(|stream| {
        assert!(stream
            .register_device_changed_callback(Some(callback1))
            .is_ok());
        assert!(stream
            .register_device_changed_callback(Some(callback2))
            .is_err());
    });
}
