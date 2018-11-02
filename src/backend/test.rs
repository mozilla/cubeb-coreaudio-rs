// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

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

#[test]
fn test_ops_context_register_device_collection_changed_unknown() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    assert_eq!(
        unsafe {
            OPS.register_device_collection_changed.unwrap()(
                c,
                ffi::CUBEB_DEVICE_TYPE_UNKNOWN,
                None,
                ptr::null_mut()
            )
        },
        ffi::CUBEB_ERROR_INVALID_PARAMETER
    );
}

fn test_ops_context_register_device_collection_changed_twice(devtype: u32) {
    // Init cubeb context.
    let mut c: *mut ffi::cubeb = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.init.unwrap()(&mut c, ptr::null()) },
        ffi::CUBEB_OK
    );

    extern "C" fn callback(_: *mut ffi::cubeb, _: *mut c_void) {
    }

    // Register a callback within the defined scope.
    assert_eq!(
        unsafe {
            OPS.register_device_collection_changed.unwrap()(
                c,
                devtype,
                Some(callback),
                ptr::null_mut()
            )
        },
        ffi::CUBEB_OK
    );

    // Hit an assertion when registering two callbacks within the same scope.
    unsafe {
        OPS.register_device_collection_changed.unwrap()(
            c,
            devtype,
            Some(callback),
            ptr::null_mut()
        );
    }

    // Destroy cubeb context.
    unsafe { OPS.destroy.unwrap()(c) }
}

#[test]
#[should_panic]
fn test_ops_context_register_device_collection_changed_twice_input() {
    test_ops_context_register_device_collection_changed_twice(ffi::CUBEB_DEVICE_TYPE_INPUT);
}

#[test]
#[should_panic]
fn test_ops_context_register_device_collection_changed_twice_output() {
    test_ops_context_register_device_collection_changed_twice(ffi::CUBEB_DEVICE_TYPE_OUTPUT);
}

#[test]
#[should_panic]
fn test_ops_context_register_device_collection_changed_twice_inout() {
    test_ops_context_register_device_collection_changed_twice(ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT);
}

#[test]
fn test_ops_context_register_device_collection_changed() {
    // Init cubeb context.
    let mut c: *mut ffi::cubeb = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.init.unwrap()(&mut c, ptr::null()) },
        ffi::CUBEB_OK
    );

    let devtypes: [ffi::cubeb_device_type; 3] = [
        ffi::CUBEB_DEVICE_TYPE_INPUT,
        ffi::CUBEB_DEVICE_TYPE_OUTPUT,
        ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_INPUT
    ];

    extern "C" fn callback(context: *mut ffi::cubeb, user: *mut c_void) {
    }

    for devtype in &devtypes {
        // Register a callback in the defined scoped.
        assert_eq!(
            unsafe {
                OPS.register_device_collection_changed.unwrap()(
                    c,
                    *devtype,
                    Some(callback),
                    ptr::null_mut()
                )
            },
            ffi::CUBEB_OK
        );

        // Unregister all callbacks regardless of the scope.
        assert_eq!(
            unsafe {
                OPS.register_device_collection_changed.unwrap()(
                    c,
                    ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT,
                    None,
                    ptr::null_mut()
                )
            },
            ffi::CUBEB_OK
        );

        // Register callback in the defined scoped again.
        assert_eq!(
            unsafe {
                OPS.register_device_collection_changed.unwrap()(
                    c,
                    *devtype,
                    Some(callback),
                    ptr::null_mut()
                )
            },
            ffi::CUBEB_OK
        );

        // Unregister callback within the defined scope.
        assert_eq!(
            unsafe {
                OPS.register_device_collection_changed.unwrap()(
                    c,
                    *devtype,
                    None,
                    ptr::null_mut()
                )
            },
            ffi::CUBEB_OK
        );
    }

    // Destroy cubeb context.
    unsafe { OPS.destroy.unwrap()(c) }
}

#[test]
#[ignore]
fn test_manual_ops_context_register_device_collection_changed() {
    // Init cubeb context.
    let mut c: *mut ffi::cubeb = ptr::null_mut();
    assert_eq!(
        unsafe { OPS.init.unwrap()(&mut c, ptr::null()) },
        ffi::CUBEB_OK
    );
    println!("context @ {:p}", c);

    extern "C" fn input_callback(context: *mut ffi::cubeb, user: *mut c_void) {
        assert_eq!(user, 0xDEAD_BEEF as *mut c_void);
        println!("input > context @ {:p}", context);
    }

    extern "C" fn output_callback(context: *mut ffi::cubeb, user: *mut c_void) {
        assert_eq!(user, 0xDEAD_BEEF as *mut c_void);
        println!("output > context @ {:p}", context);
    }

    // Register a callback for input scope.
    assert_eq!(
        unsafe {
            OPS.register_device_collection_changed.unwrap()(
                c,
                ffi::CUBEB_DEVICE_TYPE_INPUT,
                Some(input_callback),
                0xDEAD_BEEF as *mut c_void
            )
        },
        ffi::CUBEB_OK
    );

    // Register a callback for output scope.
    assert_eq!(
        unsafe {
            OPS.register_device_collection_changed.unwrap()(
                c,
                ffi::CUBEB_DEVICE_TYPE_OUTPUT,
                Some(output_callback),
                0xDEAD_BEEF as *mut c_void
            )
        },
        ffi::CUBEB_OK
    );

    loop {}

    // Destroy cubeb context.
    unsafe { OPS.destroy.unwrap()(c) }
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
// increment_active_streams
// ------------------------------------
#[test]
fn test_increment_active_streams() {
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    let mutex_ptr: *mut OwnedCriticalSection;
    {
        mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
    }
    // The scope of `_lock` is a critical section.
    let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });

    assert_eq!(ctx.active_streams, 0);
    for i in 1..10 {
        audiounit_increment_active_streams(&mut ctx);
        assert_eq!(ctx.active_streams, i);
    }
}

// set_device_info
// ------------------------------------
// TODO: Add test after C version is updated.

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
        let name_str = get_device_name(input_id);
        assert_ne!(
            name_str,
            ptr::null()
        );
        unsafe {
            CFRelease(name_str as *const c_void);
        }
    }

    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if valid_id(output_id) {
        let name_str = get_device_name(output_id);
        assert_ne!(
            name_str,
            ptr::null()
        );
        unsafe {
            CFRelease(name_str as *const c_void);
        }
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
    let ctx = AudioUnitContext::new();
    // We don't use mutex here, so there is no need to call `ctx.mutex.init()`
    // or `ctx.init()`.
    let stream = AudioUnitStream::new(
        &ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    let mut device = ffi::cubeb_device::default();

    // unknown type:
    assert_eq!(
        audiounit_get_default_device_name(
            &stream,
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
            &stream,
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
            &stream,
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
            &stream,
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

    // TODO: latency on some devices are 0 so the test fails!

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

    // TODO: latency on some devices are 0 so the test fails!

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
        let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
        assert_eq!(
            info.preferred,
            if input_id == default_output_id {
                ffi::CUBEB_DEVICE_PREF_ALL
            } else {
                ffi::CUBEB_DEVICE_PREF_NONE
            }
        );
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
        let default_input_id = audiounit_get_default_device_id(DeviceType::INPUT);
        assert_eq!(
            info.preferred,
            if output_id == default_input_id {
                ffi::CUBEB_DEVICE_PREF_ALL
            } else {
                ffi::CUBEB_DEVICE_PREF_NONE
            }
        );
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

// add_device_listener
// ------------------------------------
#[test]
// #[should_panic]
#[ignore]
fn test_add_device_listener_with_none_callback() {
    let mut ctx = AudioUnitContext::new();
    ctx.init();
    let ctx_ptr = &mut ctx as *mut AudioUnitContext;
    // The scope of `lock` is a critical section.
    let _lock = AutoLock::new(&mut ctx.mutex);

    // If it's ok to register `none` as callback, we should pass the following
    // test. Otherwise, we should get a panic or error!
    // See implementation in audiounit_add_device_listener for more detail.
    // TODO: Update this test after C version is updated!

    // The test will fail since we will register
    // `audiounit_collection_changed_callback` twice
    // as the callback for `audio_object_add_property_listener`, since we pass
    // None as `collection_changed_callback`.
    // The `audio_object_add_property_listener` will return a 'nope' error
    // (kAudioHardwareIllegalOperationError).
    for devtype in &[DeviceType::INPUT, DeviceType::OUTPUT] {
        assert_eq!(
            audiounit_add_device_listener(
                ctx_ptr,
                *devtype,
                None,
                ptr::null_mut()
            ),
            0
        );
    }

    assert_eq!(
        ctx.input_collection_changed_callback,
        None
    );

    assert_eq!(
        ctx.output_collection_changed_callback,
        None
    );

    // If it's not ok to register `none` as callback, we should pass the following test.
    // for devtype in &[DeviceType::INPUT, DeviceType::OUTPUT] {
    //     assert_ne!(
    //         audiounit_add_device_listener(
    //             ctx_ptr,
    //             *devtype,
    //             None,
    //             ptr::null_mut()
    //         ),
    //         0
    //     );
    // }
}

#[test]
#[should_panic]
fn test_add_device_listener_within_unknown_scope() {
    extern "C" fn callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    let mut ctx = AudioUnitContext::new();
    ctx.init();
    let ctx_ptr = &mut ctx as *mut AudioUnitContext;
    // The scope of `lock` is a critical section.
    let _lock = AutoLock::new(&mut ctx.mutex);

    // let _ = audiounit_add_device_listener(
    //     ctx_ptr,
    //     DeviceType::UNKNOWN,
    //     None,
    //     ptr::null_mut()
    // );

    let _ = audiounit_add_device_listener(
        ctx_ptr,
        DeviceType::UNKNOWN,
        Some(callback),
        ptr::null_mut()
    );
}

#[test]
fn test_add_device_listeners_dont_affect_other_scopes_with_same_callback() {
    extern "C" fn callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    let mut ctx = AudioUnitContext::new();
    ctx.init();
    let ctx_ptr = &mut ctx as *mut AudioUnitContext;
    // The scope of `lock` is a critical section.
    let _lock = AutoLock::new(&mut ctx.mutex);

    for devtype in [
        DeviceType::INPUT,
        DeviceType::OUTPUT,
        DeviceType::INPUT | DeviceType::OUTPUT
    ].iter() {
        assert!(ctx.input_collection_changed_callback.is_none());
        assert!(ctx.output_collection_changed_callback.is_none());

        // Register a callback within a specific scope.
        assert_eq!(
            audiounit_add_device_listener(
                ctx_ptr,
                *devtype,
                Some(callback),
                ptr::null_mut()
            ),
            0
        );

        // TODO: It doesn't work, but the return value is ok.
        assert_eq!(
            audiounit_remove_device_listener(
                ctx_ptr,
                DeviceType::UNKNOWN
            ),
            0
        );

        if devtype.contains(DeviceType::INPUT) {
            assert!(ctx.input_collection_changed_callback.is_some());
            assert!(ctx.input_collection_changed_callback.unwrap() == callback);
        } else {
            assert!(ctx.input_collection_changed_callback.is_none());
        }

        if devtype.contains(DeviceType::OUTPUT) {
            assert!(ctx.output_collection_changed_callback.is_some());
            assert!(ctx.output_collection_changed_callback.unwrap() == callback);
        } else {
            assert!(ctx.output_collection_changed_callback.is_none());
        }

        // Unregister the callbacks within all scopes.
        assert_eq!(
            audiounit_remove_device_listener(
                ctx_ptr,
                DeviceType::INPUT | DeviceType::OUTPUT,
            ),
            0
        );
    }
}

#[test]
fn test_add_device_listeners_dont_affect_other_scopes_with_different_callbacks() {
    use std::collections::HashMap;

    extern "C" fn inout_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    extern "C" fn in_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    extern "C" fn out_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    let mut map: HashMap<DeviceType, extern fn(*mut ffi::cubeb, *mut c_void)> = HashMap::new();
    map.insert(DeviceType::INPUT, in_callback);
    map.insert(DeviceType::OUTPUT, out_callback);
    map.insert(DeviceType::INPUT | DeviceType::OUTPUT, inout_callback);

    let mut ctx = AudioUnitContext::new();
    ctx.init();
    let ctx_ptr = &mut ctx as *mut AudioUnitContext;
    // The scope of `lock` is a critical section.
    let _lock = AutoLock::new(&mut ctx.mutex);

    for (devtype, callback) in map.iter() {
        assert!(ctx.input_collection_changed_callback.is_none());
        assert!(ctx.output_collection_changed_callback.is_none());

        // Register a callback within a specific scope.
        assert_eq!(
            audiounit_add_device_listener(
                ctx_ptr,
                *devtype,
                Some(*callback),
                ptr::null_mut()
            ),
            0
        );

        // TODO: It doesn't work, but the return value is ok.
        assert_eq!(
            audiounit_remove_device_listener(
                ctx_ptr,
                DeviceType::UNKNOWN
            ),
            0
        );

        if devtype.contains(DeviceType::INPUT) {
            assert!(ctx.input_collection_changed_callback.is_some());
            assert_eq!(ctx.input_collection_changed_callback.unwrap(), *callback);
        } else {
            assert!(ctx.input_collection_changed_callback.is_none());
        }

        if devtype.contains(DeviceType::OUTPUT) {
            assert!(ctx.output_collection_changed_callback.is_some());
            assert_eq!(ctx.output_collection_changed_callback.unwrap(), *callback);
        } else {
            assert!(ctx.output_collection_changed_callback.is_none());
        }

        // Unregister the callbacks within all scopes.
        assert_eq!(
            audiounit_remove_device_listener(
                ctx_ptr,
                DeviceType::INPUT | DeviceType::OUTPUT
            ),
            0
        );

        assert!(ctx.input_collection_changed_callback.is_none());
        assert!(ctx.output_collection_changed_callback.is_none());
    }
}

// remove_device_listener
// ------------------------------------
#[test]
fn test_remove_device_listener_without_adding_listeners() {
    let mut ctx = AudioUnitContext::new();
    ctx.init();
    let ctx_ptr = &mut ctx as *mut AudioUnitContext;
    // The scope of `lock` is a critical section.
    let _lock = AutoLock::new(&mut ctx.mutex);

    for devtype in &[
        DeviceType::UNKNOWN,
        DeviceType::INPUT,
        DeviceType::OUTPUT,
        DeviceType::INPUT | DeviceType::OUTPUT,
    ] {
        assert_eq!(
            audiounit_remove_device_listener(
                ctx_ptr,
                *devtype
            ),
            0
        );
    }
}

#[test]
fn test_remove_device_listeners_within_all_scopes() {
    use std::collections::HashMap;

    extern "C" fn inout_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    extern "C" fn in_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    extern "C" fn out_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    let mut map: HashMap<DeviceType, extern fn(*mut ffi::cubeb, *mut c_void)> = HashMap::new();
    map.insert(DeviceType::INPUT, in_callback);
    map.insert(DeviceType::OUTPUT, out_callback);
    map.insert(DeviceType::INPUT | DeviceType::OUTPUT, inout_callback);

    let mut ctx = AudioUnitContext::new();

    assert!(ctx.input_collection_changed_callback.is_none());
    assert!(ctx.output_collection_changed_callback.is_none());

    ctx.init();

    let ctx_ptr = &mut ctx as *mut AudioUnitContext;

    // The scope of `lock` is a critical section.
    let _lock = AutoLock::new(&mut ctx.mutex);

    for (devtype, callback) in map.iter() {
        assert_eq!(
            audiounit_add_device_listener(
                ctx_ptr,
                *devtype,
                Some(*callback),
                ptr::null_mut()
            ),
            0
        );

        // TODO: It doesn't work, but the return value is ok.
        assert_eq!(
            audiounit_remove_device_listener(
                ctx_ptr,
                DeviceType::UNKNOWN
            ),
            0
        );

        assert_eq!(
            audiounit_remove_device_listener(
                ctx_ptr,
                DeviceType::INPUT | DeviceType::OUTPUT
            ),
            0
        );

        assert!(ctx.input_collection_changed_callback.is_none());
        assert!(ctx.output_collection_changed_callback.is_none());
    }
}

#[test]
fn test_remove_device_listeners_dont_affect_other_scopes_with_same_callback() {
    extern "C" fn callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    let mut ctx = AudioUnitContext::new();
    ctx.init();
    let ctx_ptr = &mut ctx as *mut AudioUnitContext;
    // The scope of `lock` is a critical section.
    let _lock = AutoLock::new(&mut ctx.mutex);

    for devtype in [
        DeviceType::INPUT,
        DeviceType::OUTPUT,
        DeviceType::INPUT | DeviceType::OUTPUT
    ].iter() {
        assert!(ctx.input_collection_changed_callback.is_none());
        assert!(ctx.output_collection_changed_callback.is_none());

        // Register a callback within all scopes.
        assert_eq!(
            audiounit_add_device_listener(
                ctx_ptr,
                DeviceType::INPUT | DeviceType::OUTPUT,
                Some(callback),
                ptr::null_mut()
            ),
            0
        );

        assert!(ctx.input_collection_changed_callback.is_some());
        assert!(ctx.input_collection_changed_callback.unwrap() == callback);
        assert!(ctx.output_collection_changed_callback.is_some());
        assert!(ctx.output_collection_changed_callback.unwrap() == callback);

        // Unregister the callbacks within one specific scopes.
        assert_eq!(
            audiounit_remove_device_listener(
                ctx_ptr,
                *devtype,
            ),
            0
        );

        if devtype.contains(DeviceType::INPUT) {
            assert!(ctx.input_collection_changed_callback.is_none());
        } else {
            assert!(ctx.input_collection_changed_callback.is_some());
            assert!(ctx.input_collection_changed_callback.unwrap() == callback);
        }

        if devtype.contains(DeviceType::OUTPUT) {
            assert!(ctx.output_collection_changed_callback.is_none());
        } else {
            assert!(ctx.output_collection_changed_callback.is_some());
            assert!(ctx.output_collection_changed_callback.unwrap() == callback);
        }

        // Unregister the callbacks within all scopes.
        assert_eq!(
            audiounit_remove_device_listener(
                ctx_ptr,
                DeviceType::INPUT | DeviceType::OUTPUT,
            ),
            0
        );
    }
}

#[test]
fn test_remove_device_listeners_dont_affect_other_scopes_with_different_callbacks() {
    use std::collections::HashMap;

    extern "C" fn in_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    extern "C" fn out_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    let mut map: HashMap<DeviceType, extern fn(*mut ffi::cubeb, *mut c_void)> = HashMap::new();
    map.insert(DeviceType::INPUT, in_callback);
    map.insert(DeviceType::OUTPUT, out_callback);

    let mut ctx = AudioUnitContext::new();

    assert!(ctx.input_collection_changed_callback.is_none());
    assert!(ctx.output_collection_changed_callback.is_none());

    ctx.init();

    let ctx_ptr = &mut ctx as *mut AudioUnitContext;

    // The scope of `lock` is a critical section.
    let _lock = AutoLock::new(&mut ctx.mutex);

    for (devtype, _) in map.iter() {
        assert!(ctx.input_collection_changed_callback.is_none());
        assert!(ctx.output_collection_changed_callback.is_none());

        // Register callbacks within all scopes.
        for (scope, listener) in map.iter() {
            assert_eq!(
                audiounit_add_device_listener(
                    ctx_ptr,
                    *scope,
                    Some(*listener),
                    ptr::null_mut()
                ),
                0
            );
        }

        assert!(ctx.input_collection_changed_callback.is_some());
        assert_eq!(
            ctx.input_collection_changed_callback.unwrap(),
            *(map.get(&DeviceType::INPUT).unwrap())
        );
        assert!(ctx.output_collection_changed_callback.is_some());
        assert_eq!(
            ctx.output_collection_changed_callback.unwrap(),
            *(map.get(&DeviceType::OUTPUT).unwrap())
        );

        // Unregister the callbacks within one specific scopes.
        assert_eq!(
            audiounit_remove_device_listener(
                ctx_ptr,
                *devtype,
            ),
            0
        );

        if devtype == &DeviceType::INPUT {
            assert!(ctx.input_collection_changed_callback.is_none());

            assert!(ctx.output_collection_changed_callback.is_some());
            assert_eq!(
                ctx.output_collection_changed_callback.unwrap(),
                *(map.get(&DeviceType::OUTPUT).unwrap())
            );
        } else {
            assert_eq!(devtype, &DeviceType::OUTPUT);

            assert!(ctx.output_collection_changed_callback.is_none());

            assert!(ctx.input_collection_changed_callback.is_some());
            assert_eq!(
                ctx.input_collection_changed_callback.unwrap(),
                *(map.get(&DeviceType::INPUT).unwrap())
            );
        }

        // Unregister the callbacks within all scopes.
        assert_eq!(
            audiounit_remove_device_listener(
                ctx_ptr,
                DeviceType::INPUT | DeviceType::OUTPUT,
            ),
            0
        );
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
