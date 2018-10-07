use super::*;

// Interface
// ============================================================================
#[test]
fn test_ops_context_init() {
    let mut c: *mut ffi::cubeb = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.init.unwrap()(&mut c, ptr::null()) },
        ffi::CUBEB_OK
    );
    unsafe { OPS.destroy.unwrap()(c) }
}

#[test]
fn test_ops_context_max_channel_count() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut max_channel_count = u32::max_value();
    assert_eq!(
        unsafe { OPS.get_max_channel_count.unwrap()(c, &mut max_channel_count) },
        ffi::CUBEB_OK
    );
    assert_eq!(max_channel_count, 0);
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
    assert_eq!(latency, 0);
}

#[test]
fn test_ops_context_preferred_sample_rate() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut rate = u32::max_value();
    assert_eq!(
        unsafe { OPS.get_preferred_sample_rate.unwrap()(c, &mut rate) },
        ffi::CUBEB_OK
    );
    assert_eq!(rate, 0);
}

#[test]
fn test_ops_context_enumerate_devices_unknown() {
    let ctx: *mut ffi::cubeb = ptr::null_mut();
    let mut coll = ffi::cubeb_device_collection {
        device: ptr::null_mut(),
        count: 0,
    };
    assert_eq!(
        unsafe {
            OPS.enumerate_devices.unwrap()(
                ctx,
                ffi::CUBEB_DEVICE_TYPE_UNKNOWN,
                &mut coll
            )
        },
        ffi::CUBEB_OK
    );
    assert_eq!(coll.count, 0);
    assert_eq!(coll.device, ptr::null_mut());
    assert_eq!(
        unsafe { OPS.device_collection_destroy.unwrap()(ctx, &mut coll) },
        ffi::CUBEB_OK
    );
}

#[test]
fn test_ops_context_enumerate_devices_input() {
    let ctx: *mut ffi::cubeb = ptr::null_mut();
    let mut coll = ffi::cubeb_device_collection {
        device: ptr::null_mut(),
        count: 0,
    };
    assert_eq!(
        unsafe {
            OPS.enumerate_devices.unwrap()(
                ctx,
                ffi::CUBEB_DEVICE_TYPE_INPUT,
                &mut coll
            )
        },
        ffi::CUBEB_OK
    );
    if coll.count > 0 {
        assert_ne!(coll.device, ptr::null_mut());
    } else {
        assert_eq!(coll.device, ptr::null_mut());
    }
    assert_eq!(
        unsafe { OPS.device_collection_destroy.unwrap()(ctx, &mut coll) },
        ffi::CUBEB_OK
    );
}

#[test]
fn test_ops_context_enumerate_devices_output() {
    let ctx: *mut ffi::cubeb = ptr::null_mut();
    let mut coll = ffi::cubeb_device_collection {
        device: ptr::null_mut(),
        count: 0,
    };
    assert_eq!(
        unsafe {
            OPS.enumerate_devices.unwrap()(
                ctx,
                ffi::CUBEB_DEVICE_TYPE_OUTPUT,
                &mut coll
            )
        },
        ffi::CUBEB_OK
    );
    if coll.count > 0 {
        assert_ne!(coll.device, ptr::null_mut());
    } else {
        assert_eq!(coll.device, ptr::null_mut());
    }
    assert_eq!(
        unsafe { OPS.device_collection_destroy.unwrap()(ctx, &mut coll) },
        ffi::CUBEB_OK
    );
}

#[test]
fn test_ops_context_device_collection_destroy() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut coll = ffi::cubeb_device_collection {
        device: ptr::null_mut(),
        count: 0,
    };
    assert_eq!(
        unsafe { OPS.device_collection_destroy.unwrap()(c, &mut coll) },
        ffi::CUBEB_OK
    );
    assert_eq!(coll.device, ptr::null_mut());
    assert_eq!(coll.count, 0);
}

// stream_init: Some($crate::capi::capi_stream_init::<$ctx>),
// stream_destroy: Some($crate::capi::capi_stream_destroy::<$stm>),
// stream_start: Some($crate::capi::capi_stream_start::<$stm>),
// stream_stop: Some($crate::capi::capi_stream_stop::<$stm>),
// stream_get_position: Some($crate::capi::capi_stream_get_position::<$stm>),

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
    unsafe {
        OPS.stream_set_volume.unwrap()(s, 0.5);
    }
}

#[test]
fn test_ops_stream_set_panning() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    unsafe {
        OPS.stream_set_panning.unwrap()(s, 0.5);
    }
}

#[test]
fn test_ops_stream_current_device() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    let mut device: *mut ffi::cubeb_device = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.stream_get_current_device.unwrap()(s, &mut device) },
        ffi::CUBEB_OK
    );
    assert_eq!(device, 0xDEAD_BEEF as *mut _);
}

#[test]
fn test_ops_stream_device_destroy() {
    let s: *mut ffi::cubeb_stream = ptr::null_mut();
    unsafe {
        OPS.stream_device_destroy.unwrap()(s, 0xDEAD_BEEF as *mut _);
    }
}

// Private APIs
// ============================================================================
// get_default_device_id
// ------------------------------------
#[test]
fn test_get_default_device_id() {
    // Invalid types:
    assert_eq!(
        audiounit_get_default_device_id(DeviceType::UNKNOWN),
        kAudioObjectUnknown,
    );
    assert_eq!(
        audiounit_get_default_device_id(DeviceType::INPUT | DeviceType::OUTPUT),
        kAudioObjectUnknown,
    );
    // The following types work since DeviceType::UNKNOWN is 0.
    // TODO: Is that a bug?
    // assert_eq!(
    //     audiounit_get_default_device_id(DeviceType::UNKNOWN | DeviceType::INPUT),
    //     kAudioObjectUnknown,
    // );
    // assert_eq!(
    //     audiounit_get_default_device_id(DeviceType::UNKNOWN | DeviceType::OUTPUT),
    //     kAudioObjectUnknown,
    // );

    // Valid types:
    // P.S. Works only when there is available default input and output.
    assert_ne!(
        audiounit_get_default_device_id(DeviceType::INPUT),
        kAudioObjectUnknown,
    );
    assert_ne!(
        audiounit_get_default_device_id(DeviceType::OUTPUT),
        kAudioObjectUnknown,
    )
}

// get_channel_count
// ------------------------------------
#[test]
fn test_get_channel_count() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    if valid_id(input_id) {
        assert!(audiounit_get_channel_count(input_id, kAudioDevicePropertyScopeInput) > 0);
    }

    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if valid_id(output_id) {
        assert!(audiounit_get_channel_count(output_id, kAudioDevicePropertyScopeOutput) > 0);
    }
}

// get_devices_of_type
// ------------------------------------
#[test]
fn test_get_devices_of_type() {
    // FIXIT: Open this assertion after C version is updated.
    // let no_devs = audiounit_get_devices_of_type(DeviceType::UNKNOWN);
    // assert!(no_devs.is_empty());

    let all_devs = audiounit_get_devices_of_type(DeviceType::INPUT | DeviceType::OUTPUT);
    let in_devs = audiounit_get_devices_of_type(DeviceType::INPUT);
    let out_devs = audiounit_get_devices_of_type(DeviceType::OUTPUT);

    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    if valid_id(input_id) {
        assert!(!all_devs.is_empty());
        assert!(!in_devs.is_empty());
        assert!(all_devs.contains(&input_id));
        assert!(in_devs.contains(&input_id));
    }

    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if valid_id(output_id) {
        assert!(!all_devs.is_empty());
        assert!(!out_devs.is_empty());
        assert!(all_devs.contains(&input_id));
        assert!(out_devs.contains(&output_id));
    }
}

// Utils
// ------------------------------------
fn valid_id(id: AudioObjectID) -> bool {
    id != kAudioObjectUnknown
}