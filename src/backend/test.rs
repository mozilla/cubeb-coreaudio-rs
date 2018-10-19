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
    assert!(max_channel_count > 0);
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
    assert!(latency >= SAFE_MIN_LATENCY_FRAMES);
    assert!(SAFE_MAX_LATENCY_FRAMES >= latency);
}

#[test]
fn test_ops_context_preferred_sample_rate() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut rate = u32::max_value();
    assert_eq!(
        unsafe { OPS.get_preferred_sample_rate.unwrap()(c, &mut rate) },
        ffi::CUBEB_OK
    );
    assert!(rate > 0);
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

// #[test]
// fn test_ops_stream_latency() {
//     let s: *mut ffi::cubeb_stream = ptr::null_mut();
//     let mut latency = u32::max_value();
//     assert_eq!(
//         unsafe { OPS.stream_get_latency.unwrap()(s, &mut latency) },
//         ffi::CUBEB_OK
//     );
//     assert_eq!(latency, 0);
// }

// #[test]
// fn test_ops_stream_set_volume() {
//     let s: *mut ffi::cubeb_stream = ptr::null_mut();
//     unsafe {
//         OPS.stream_set_volume.unwrap()(s, 0.5);
//     }
// }

// #[test]
// fn test_ops_stream_set_panning() {
//     let s: *mut ffi::cubeb_stream = ptr::null_mut();
//     unsafe {
//         OPS.stream_set_panning.unwrap()(s, 0.5);
//     }
// }

// #[test]
// fn test_ops_stream_current_device() {
//     let s: *mut ffi::cubeb_stream = ptr::null_mut();
//     let mut device: *mut ffi::cubeb_device = ptr::null_mut();
//     assert_eq!(
//         unsafe { OPS.stream_get_current_device.unwrap()(s, &mut device) },
//         ffi::CUBEB_OK
//     );
//     assert_eq!(device, 0xDEAD_BEEF as *mut _);
// }

// #[test]
// fn test_ops_stream_device_destroy() {
//     let s: *mut ffi::cubeb_stream = ptr::null_mut();
//     unsafe {
//         OPS.stream_device_destroy.unwrap()(s, 0xDEAD_BEEF as *mut _);
//     }
// }

// Private APIs
// ============================================================================
// get_acceptable_latency_range
// ------------------------------------
#[test]
fn test_get_acceptable_latency_range() {
    let mut latency_range = AudioValueRange::default();
    assert!(
        audiounit_get_acceptable_latency_range(
            &mut latency_range
        ).is_ok()
    );
    assert!(latency_range.mMinimum > 0.0);
    assert!(latency_range.mMaximum > 0.0);
    assert!(latency_range.mMaximum > latency_range.mMinimum);
}

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

// get_device_name
// ------------------------------------
#[test]
fn test_get_device_name() {
    // Unknown device:
    assert_eq!(
        get_device_name(kAudioObjectUnknown),
        ptr::null()
    );

    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    if valid_id(input_id) {
        assert_ne!(
            get_device_name(input_id),
            ptr::null()
        );
    }

    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if valid_id(output_id) {
        assert_ne!(
            get_device_name(output_id),
            ptr::null()
        );
    }
}


// convert_uint32_into_string
// ------------------------------------
#[test]
fn test_convert_uint32_into_string() {
    let empty = convert_uint32_into_string(0);
    assert_eq!(empty, CString::default());

    let data: u32 = ('R' as u32) << 24 |
                    ('U' as u32) << 16 |
                    ('S' as u32) << 8 |
                    'T' as u32;
    let data_string = convert_uint32_into_string(data);
    assert_eq!(data_string, CString::new("RUST").unwrap());
}


// audiounit_get_default_device_datasource
// ------------------------------------
#[test]
fn test_get_default_device_datasource() {
    let mut data = 0;

    // unknown type:
    assert_eq!(
        audiounit_get_default_device_datasource(
            DeviceType::UNKNOWN,
            &mut data
        ).unwrap_err(),
        Error::error()
    );

    // TODO: The following fail with some USB headsets (e.g., Plantronic .Audio 628).
    //       Find a reliable way to test the input/output scope.

    // input:
    data = 0;
    assert!(
        audiounit_get_default_device_datasource(
            DeviceType::INPUT,
            &mut data
        ).is_ok()
    );
    assert_ne!(data, 0);

    // output:
    data = 0;
    assert!(
        audiounit_get_default_device_datasource(
            DeviceType::OUTPUT,
            &mut data
        ).is_ok()
    );
    assert_ne!(data, 0);

    // in-out:
    assert_eq!(
        audiounit_get_default_device_datasource(
            DeviceType::INPUT | DeviceType::OUTPUT,
            &mut data
        ).unwrap_err(),
        Error::error()
    );
}

// audiounit_get_default_device_name
// ------------------------------------
#[test]
fn test_get_default_device_name() {
    let ctx = AudioUnitContext { ops: ptr::null() };
    let stream = AudioUnitStream::new(&ctx).unwrap();
    let mut device = ffi::cubeb_device::default();

    // unknown type:
    assert_eq!(
        audiounit_get_default_device_name(
            stream.as_ref(),
            &mut device,
            DeviceType::UNKNOWN
        ).unwrap_err(),
        Error::error()
    );

    // TODO: The following fail with some USB headsets (e.g., Plantronic .Audio 628).
    //       Find a reliable way to test the input/output scope.

    // input:
    device = ffi::cubeb_device::default();
    assert!(
        audiounit_get_default_device_name(
            stream.as_ref(),
            &mut device,
            DeviceType::INPUT
        ).is_ok()
    );
    assert_ne!(device.input_name, ptr::null_mut());
    assert_eq!(device.output_name, ptr::null_mut());

    // output:
    device = ffi::cubeb_device::default();
    assert!(
        audiounit_get_default_device_name(
            stream.as_ref(),
            &mut device,
            DeviceType::OUTPUT
        ).is_ok()
    );
    assert_eq!(device.input_name, ptr::null_mut());
    assert_ne!(device.output_name, ptr::null_mut());

    // in-out:
    device = ffi::cubeb_device::default();
    assert_eq!(
        audiounit_get_default_device_name(
            stream.as_ref(),
            &mut device,
            DeviceType::INPUT | DeviceType::OUTPUT
        ).unwrap_err(),
        Error::error()
    );
    assert_eq!(device.input_name, ptr::null_mut());
    assert_eq!(device.output_name, ptr::null_mut());

}

// strref_to_cstr_utf8
// ------------------------------------
// TODO

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

// get_available_samplerate
// ------------------------------------
#[test]
fn test_get_available_samplerate_unknown() {
    let mut defualt = 0;
    let mut min = 0;
    let mut max = 0;

    // global scope:
    audiounit_get_available_samplerate(
        kAudioObjectUnknown,
        kAudioObjectPropertyScopeGlobal,
        &mut min,
        &mut max,
        &mut defualt
    );
    assert_eq!(defualt, 0);
    assert_eq!(min, 0);
    assert_eq!(max, 0);

    // input scope:
    audiounit_get_available_samplerate(
        kAudioObjectUnknown,
        kAudioDevicePropertyScopeInput,
        &mut min,
        &mut max,
        &mut defualt
    );
    assert_eq!(defualt, 0);
    assert_eq!(min, 0);
    assert_eq!(max, 0);

    // output scope:
    audiounit_get_available_samplerate(
        kAudioObjectUnknown,
        kAudioDevicePropertyScopeOutput,
        &mut min,
        &mut max,
        &mut defualt
    );
    assert_eq!(defualt, 0);
    assert_eq!(min, 0);
    assert_eq!(max, 0);
}

#[test]
fn test_get_available_samplerate_input() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    if !valid_id(input_id) {
        return;
    }

    let mut defualt = 0;
    let mut min = 0;
    let mut max = 0;

    // global scope:
    audiounit_get_available_samplerate(
        input_id,
        kAudioObjectPropertyScopeGlobal,
        &mut min,
        &mut max,
        &mut defualt
    );
    // println!("[samplerate_input] <global> default: {}, min: {}, max: {}", defualt, min, max);
    assert!(defualt > 0);
    assert!(min > 0);
    assert!(max > 0);
    assert!(min <= max);
    assert!(min <= defualt);
    assert!(defualt <= max);

    // input scope:
    defualt = 0;
    min = 0;
    max = 0;
    audiounit_get_available_samplerate(
        input_id,
        kAudioDevicePropertyScopeInput,
        &mut min,
        &mut max,
        &mut defualt
    );
    // println!("[samplerate_input] <input> default: {}, min: {}, max: {}", defualt, min, max);
    assert!(defualt > 0);
    assert!(min > 0);
    assert!(max > 0);
    assert!(min <= max);
    assert!(min <= defualt);
    assert!(defualt <= max);

    // output scope:
    defualt = 0;
    min = 0;
    max = 0;
    audiounit_get_available_samplerate(
        input_id,
        kAudioDevicePropertyScopeOutput,
        &mut min,
        &mut max,
        &mut defualt
    );
    // println!("[samplerate_input] <output> default: {}, min: {}, max: {}", defualt, min, max);
    if is_output(input_id) {
        assert!(defualt > 0);
        assert!(min > 0);
        assert!(max > 0);
        assert!(min <= max);
        assert!(min <= defualt);
        assert!(defualt <= max);
    } else {
        // assert_eq!(defualt, 0);
        // assert_eq!(min, 0);
        // assert_eq!(max, 0);

        // Surprisingly it works!
        assert!(defualt > 0);
        assert!(min > 0);
        assert!(max > 0);
        assert!(min <= max);
        assert!(min <= defualt);
        assert!(defualt <= max);
    }
}

#[test]
fn test_get_available_samplerate_output() {
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(output_id) {
        return;
    }

    let mut defualt = 0;
    let mut min = 0;
    let mut max = 0;

    // global scope:
    audiounit_get_available_samplerate(
        output_id,
        kAudioObjectPropertyScopeGlobal,
        &mut min,
        &mut max,
        &mut defualt
    );
    // println!("[samplerate_output] <global> default: {}, min: {}, max: {}", defualt, min, max);
    assert!(defualt > 0);
    assert!(min > 0);
    assert!(max > 0);
    assert!(min <= max);
    assert!(min <= defualt);
    assert!(defualt <= max);

    // input scope:
    defualt = 0;
    min = 0;
    max = 0;
    audiounit_get_available_samplerate(
        output_id,
        kAudioDevicePropertyScopeInput,
        &mut min,
        &mut max,
        &mut defualt
    );
    // println!("[samplerate_output] <input> default: {}, min: {}, max: {}", defualt, min, max);
    if is_input(output_id) {
        assert!(defualt > 0);
        assert!(min > 0);
        assert!(max > 0);
        assert!(min <= max);
        assert!(min <= defualt);
        assert!(defualt <= max);
    } else {
        // assert_eq!(defualt, 0);
        // assert_eq!(min, 0);
        // assert_eq!(max, 0);

        // Surprisingly it works!
        assert!(defualt > 0);
        assert!(min > 0);
        assert!(max > 0);
        assert!(min <= max);
        assert!(min <= defualt);
        assert!(defualt <= max);
    }

    // output scope:
    defualt = 0;
    min = 0;
    max = 0;
    audiounit_get_available_samplerate(
        output_id,
        kAudioDevicePropertyScopeOutput,
        &mut min,
        &mut max,
        &mut defualt
    );
    // println!("[samplerate_output] <output> default: {}, min: {}, max: {}", defualt, min, max);
    assert!(defualt > 0);
    assert!(min > 0);
    assert!(max > 0);
    assert!(min <= max);
    assert!(min <= defualt);
    assert!(defualt <= max);
}

// get_device_presentation_latency
// ------------------------------------
#[test]
fn test_get_device_presentation_latency_unknown() {
    let mut latency = 0;

    // global scope:
    latency = audiounit_get_device_presentation_latency(
        kAudioObjectUnknown,
        kAudioObjectPropertyScopeGlobal,
    );
    assert_eq!(latency, 0);

    // input scope:
    latency = audiounit_get_device_presentation_latency(
        kAudioObjectUnknown,
        kAudioDevicePropertyScopeInput,
    );
    assert_eq!(latency, 0);

    // output scope:
    latency = audiounit_get_device_presentation_latency(
        kAudioObjectUnknown,
        kAudioDevicePropertyScopeOutput,
    );
    assert_eq!(latency, 0);
}

#[test]
fn test_get_device_presentation_latency_input() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    if !valid_id(input_id) {
        return;
    }

    let mut latency = 0;

    // global scope:
    latency = audiounit_get_device_presentation_latency(
        input_id,
        kAudioObjectPropertyScopeGlobal,
    );
    assert_eq!(latency, 0);

    // input scope:
    latency = audiounit_get_device_presentation_latency(
        input_id,
        kAudioDevicePropertyScopeInput,
    );
    assert!(latency > 0);

    // output scope:
    latency = audiounit_get_device_presentation_latency(
        input_id,
        kAudioDevicePropertyScopeOutput,
    );
    if is_output(input_id) {
        assert!(latency > 0);
    } else {
        assert_eq!(latency, 0);
    }
}

#[test]
fn test_get_device_presentation_latency_output() {
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(output_id) {
        return;
    }

    let mut latency = 0;

    // global scope:
    latency = audiounit_get_device_presentation_latency(
        output_id,
        kAudioObjectPropertyScopeGlobal,
    );
    assert_eq!(latency, 0);

    // input scope:
    latency = audiounit_get_device_presentation_latency(
        output_id,
        kAudioDevicePropertyScopeInput,
    );
    if is_input(output_id) {
        assert!(latency > 0);
    } else {
        assert_eq!(latency, 0);
    }

    // output scope:
    latency = audiounit_get_device_presentation_latency(
        output_id,
        kAudioDevicePropertyScopeOutput,
    );
    assert!(latency > 0);
}

// create_device_from_hwdev
// ------------------------------------
#[test]
fn test_create_device_from_hwdev_unknown() {
    let mut info = ffi::cubeb_device_info::default();

    // unknown
    assert_eq!(
        audiounit_create_device_from_hwdev(
            &mut info,
            kAudioObjectUnknown,
            DeviceType::UNKNOWN,
        ).unwrap_err(),
        Error::error()
    );

    // input
    assert_eq!(
        audiounit_create_device_from_hwdev(
            &mut info,
            kAudioObjectUnknown,
            DeviceType::INPUT,
        ).unwrap_err(),
        Error::error()
    );

    // output
    assert_eq!(
        audiounit_create_device_from_hwdev(
            &mut info,
            kAudioObjectUnknown,
            DeviceType::OUTPUT,
        ).unwrap_err(),
        Error::error()
    );

    // in-out
    assert_eq!(
        audiounit_create_device_from_hwdev(
            &mut info,
            kAudioObjectUnknown,
            DeviceType::INPUT | DeviceType::OUTPUT,
        ).unwrap_err(),
        Error::error()
    );
}

#[test]
fn test_create_device_from_hwdev_input() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    if !valid_id(input_id) {
        return;
    }

    let mut info = ffi::cubeb_device_info::default();

    // unknown
    assert_eq!(
        audiounit_create_device_from_hwdev(
            &mut info,
            input_id,
            DeviceType::UNKNOWN,
        ).unwrap_err(),
        Error::error()
    );

    // input
    info = ffi::cubeb_device_info::default();
    assert!(
        audiounit_create_device_from_hwdev(
            &mut info,
            input_id,
            DeviceType::INPUT,
        ).is_ok()
    );
    assert_ne!(info.devid, ptr::null_mut());
    assert_ne!(info.device_id, ptr::null_mut());
    assert_eq!(info.group_id, info.device_id);
    assert_ne!(info.friendly_name, ptr::null_mut());
    assert_ne!(info.vendor_name, ptr::null_mut());
    assert_eq!(info.device_type, ffi::CUBEB_DEVICE_TYPE_INPUT);
    assert_eq!(info.state, ffi::CUBEB_DEVICE_STATE_ENABLED);
    assert_eq!(info.preferred, ffi::CUBEB_DEVICE_PREF_ALL);
    assert!(info.max_channels > 0);
    assert_eq!(info.default_format, ffi::CUBEB_DEVICE_FMT_F32NE);
    assert!(info.min_rate <= info.max_rate);
    assert!(info.min_rate <= info.default_rate);
    assert!(info.default_rate <= info.max_rate);
    assert!(info.latency_lo > 0);
    assert!(info.latency_hi > 0);
    assert!(info.latency_lo <= info.latency_hi);

    // output
    info = ffi::cubeb_device_info::default();
    if is_output(input_id) {
        assert!(
            audiounit_create_device_from_hwdev(
                &mut info,
                input_id,
                DeviceType::OUTPUT,
            ).is_ok()
        );
        assert_ne!(info.devid, ptr::null_mut());
        assert_ne!(info.device_id, ptr::null_mut());
        assert_eq!(info.group_id, info.device_id);
        assert_ne!(info.friendly_name, ptr::null_mut());
        assert_ne!(info.vendor_name, ptr::null_mut());
        assert_eq!(info.device_type, ffi::CUBEB_DEVICE_TYPE_OUTPUT);
        assert_eq!(info.state, ffi::CUBEB_DEVICE_STATE_ENABLED);
        assert_eq!(info.preferred, ffi::CUBEB_DEVICE_PREF_ALL);
        assert!(info.max_channels > 0);
        assert_eq!(info.default_format, ffi::CUBEB_DEVICE_FMT_F32NE);
        assert!(info.min_rate <= info.max_rate);
        assert!(info.min_rate <= info.default_rate);
        assert!(info.default_rate <= info.max_rate);
        assert!(info.latency_lo > 0);
        assert!(info.latency_hi > 0);
        assert!(info.latency_lo <= info.latency_hi);
    } else {
        assert_eq!(
            audiounit_create_device_from_hwdev(
                &mut info,
                input_id,
                DeviceType::OUTPUT,
            ).unwrap_err(),
            Error::error()
        );
    }

    // in-out
    assert_eq!(
        audiounit_create_device_from_hwdev(
            &mut info,
            input_id,
            DeviceType::INPUT | DeviceType::OUTPUT,
        ).unwrap_err(),
        Error::error()
    );
}

#[test]
fn test_create_device_from_hwdev_output() {
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(output_id) {
        return;
    }

    let mut info = ffi::cubeb_device_info::default();

    // unknown
    assert_eq!(
        audiounit_create_device_from_hwdev(
            &mut info,
            output_id,
            DeviceType::UNKNOWN,
        ).unwrap_err(),
        Error::error()
    );

    // input
    info = ffi::cubeb_device_info::default();
    if is_input(output_id) {
        assert!(
            audiounit_create_device_from_hwdev(
                &mut info,
                output_id,
                DeviceType::INPUT,
            ).is_ok()
        );
        assert_ne!(info.devid, ptr::null_mut());
        assert_ne!(info.device_id, ptr::null_mut());
        assert_eq!(info.group_id, info.device_id);
        assert_ne!(info.friendly_name, ptr::null_mut());
        assert_ne!(info.vendor_name, ptr::null_mut());
        assert_eq!(info.device_type, ffi::CUBEB_DEVICE_TYPE_INPUT);
        assert_eq!(info.state, ffi::CUBEB_DEVICE_STATE_ENABLED);
        assert_eq!(info.preferred, ffi::CUBEB_DEVICE_PREF_ALL);
        assert!(info.max_channels > 0);
        assert_eq!(info.default_format, ffi::CUBEB_DEVICE_FMT_F32NE);
        assert!(info.min_rate <= info.max_rate);
        assert!(info.min_rate <= info.default_rate);
        assert!(info.default_rate <= info.max_rate);
        assert!(info.latency_lo > 0);
        assert!(info.latency_hi > 0);
        assert!(info.latency_lo <= info.latency_hi);
    } else {
        assert_eq!(
            audiounit_create_device_from_hwdev(
                &mut info,
                output_id,
                DeviceType::INPUT,
            ).unwrap_err(),
            Error::error()
        );
    }

    // output
    info = ffi::cubeb_device_info::default();
    assert!(
        audiounit_create_device_from_hwdev(
            &mut info,
            output_id,
            DeviceType::OUTPUT,
        ).is_ok()
    );
    assert_ne!(info.devid, ptr::null_mut());
    assert_ne!(info.device_id, ptr::null_mut());
    assert_eq!(info.group_id, info.device_id);
    assert_ne!(info.friendly_name, ptr::null_mut());
    assert_ne!(info.vendor_name, ptr::null_mut());
    assert_eq!(info.device_type, ffi::CUBEB_DEVICE_TYPE_OUTPUT);
    assert_eq!(info.state, ffi::CUBEB_DEVICE_STATE_ENABLED);
    assert_eq!(info.preferred, ffi::CUBEB_DEVICE_PREF_ALL);
    assert!(info.max_channels > 0);
    assert_eq!(info.default_format, ffi::CUBEB_DEVICE_FMT_F32NE);
    assert!(info.min_rate <= info.max_rate);
    assert!(info.min_rate <= info.default_rate);
    assert!(info.default_rate <= info.max_rate);
    assert!(info.latency_lo > 0);
    assert!(info.latency_hi > 0);
    assert!(info.latency_lo <= info.latency_hi);

    // in-out
    assert_eq!(
        audiounit_create_device_from_hwdev(
            &mut info,
            output_id,
            DeviceType::INPUT | DeviceType::OUTPUT,
        ).unwrap_err(),
        Error::error()
    );
}

// is_aggregate_device
// ------------------------------------
// TODO

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

fn is_input(id: AudioObjectID) -> bool {
    audiounit_get_channel_count(id, kAudioDevicePropertyScopeInput) > 0
}

fn is_output(id: AudioObjectID) -> bool {
    audiounit_get_channel_count(id, kAudioDevicePropertyScopeOutput) > 0
}
