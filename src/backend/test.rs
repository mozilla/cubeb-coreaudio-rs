// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

use super::*;

// Note / Template
// ============================================================================
#[test]
fn test_stream_drop_mutex_incorrect() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
    let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;

    // The scope of `_lock` is a critical section.
    let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });

    // Add one stream to the context in advance to avoid the borrowing-twice
    // issue of ctx.
    audiounit_increment_active_streams(&mut ctx);

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    // The resampler will be initialized in `audiounit_setup_stream` (or via
    // `stream_init`), and it only accepts the formats with FLOAT32NE or S16NE.
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 96_000;
    raw.channels = 32;
    raw.layout = ffi::CUBEB_LAYOUT_3F1_LFE;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // It's crucial to call to audiounit_set_device_info to set
    // stream.output_device to output device type, or we will hit the
    // assertion in audiounit_create_unit.

    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    // Return an error if there is no available device.
    if !valid_id(default_output_id) {
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::OUTPUT
        ).is_ok()
    );

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );

    {
        let stm_mutex_ptr = &mut stream.mutex as *mut OwnedCriticalSection;
        let _stm_lock = AutoLock::new(unsafe { &mut (*stm_mutex_ptr) });
        assert!(audiounit_setup_stream(&mut stream).is_ok());
    }

    assert!(!stream.output_unit.is_null());

    // If the following `drop` is commented, the AudioUnitStream::drop()
    // will lock the AudioUnitStream.context.mutex without releasing the
    // AudioUnitStream.context.mutex in use (`ctx_lock` here) first and
    // cause a deadlock, when hitting the `assert!(false)` at the end of
    // this test.
    // The `ctx_lock` is created before `stream`
    // (whose type is AudioUnitStream), so `stream.drop()` will be called
    // before `ctx_lock.drop()`

    // Force to drop the context lock before stream is dropped, since
    // AudioUnitStream::Drop() will lock the context mutex.
    drop(ctx_lock);
}

#[test]
fn test_stream_drop_mutex_correct() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
    let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;

    // Add one stream to the context in advance to avoid the borrowing-twice
    // issue of ctx.
    // `AudioUnitStream::drop()` will check the context has at least one stream.
    {
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr ) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    // The scope of `ctx_lock` is a critical section.
    // When `AudioUnitStream::drop()` is called, `AudioUnitContext.mutex`
    // needs to be unlocked. That's why `_lock` needs to be declared after
    // `stream` so it will be dropped and unlocked before dropping `stream`.
    let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });

    // The resampler will be initialized in `audiounit_setup_stream` (or via
    // `stream_init`), and it only accepts the formats with FLOAT32NE or S16NE.
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 96_000;
    raw.channels = 32;
    raw.layout = ffi::CUBEB_LAYOUT_3F1_LFE;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // It's crucial to call to audiounit_set_device_info to set
    // stream.output_device to output device type, or we will hit the
    // assertion in audiounit_create_unit.

    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    // Return an error if there is no available device.
    if !valid_id(default_output_id) {
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::OUTPUT
        ).is_ok()
    );

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );

    {
        let stm_mutex_ptr = &mut stream.mutex as *mut OwnedCriticalSection;
        let _stm_lock = AutoLock::new(unsafe { &mut (*stm_mutex_ptr) });
        assert!(audiounit_setup_stream(&mut stream).is_ok());
    }

    assert!(!stream.output_unit.is_null());

    // Do some stream operations here ...
}

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
        if valid_id(audiounit_get_default_device_id(DeviceType::OUTPUT)) {
            ffi::CUBEB_OK
        } else {
            ffi::CUBEB_ERROR
        }
    );
    assert!(max_channel_count > 0);
}

#[test]
fn test_ops_context_min_latency() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let params: ffi::cubeb_stream_params = unsafe { ::std::mem::zeroed() };
    let mut latency = u32::max_value();
    if valid_id(audiounit_get_default_device_id(DeviceType::OUTPUT)) {
        assert_eq!(
            unsafe { OPS.get_min_latency.unwrap()(c, params, &mut latency) },
            ffi::CUBEB_OK
        );
        assert!(latency >= SAFE_MIN_LATENCY_FRAMES);
        assert!(SAFE_MAX_LATENCY_FRAMES >= latency);
    } else {
        assert_eq!(
            unsafe { OPS.get_min_latency.unwrap()(c, params, &mut latency) },
            ffi::CUBEB_ERROR
        );
        assert_eq!(latency, u32::max_value());
    }
}

#[test]
fn test_ops_context_preferred_sample_rate() {
    let c: *mut ffi::cubeb = ptr::null_mut();
    let mut rate = u32::max_value();
    assert_eq!(
        unsafe { OPS.get_preferred_sample_rate.unwrap()(c, &mut rate) },
        if valid_id(audiounit_get_default_device_id(DeviceType::OUTPUT)) {
            ffi::CUBEB_OK
        } else {
            ffi::CUBEB_ERROR
        }
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

// A panic in `capi_register_device_collection_changed` causes
// `EXC_BAD_INSTRUCTION` on my MacBook Air but it's fine on my MacBook Pro.
// It'w weird that it works fine if replacing
// `register_device_collection_changed: Option<unsafe extern "C" fn(..,) -> c_int>`
// to `register_device_collection_changed: unsafe extern "C" fn(..,) -> c_int`
// Test them in `AudioUnitContext` directly instead of calling them via `OPS` for now.
#[test]
#[ignore]
#[should_panic]
fn test_ops_context_register_device_collection_changed_twice_input() {
    test_ops_context_register_device_collection_changed_twice(ffi::CUBEB_DEVICE_TYPE_INPUT);
}

#[test]
#[ignore]
#[should_panic]
fn test_ops_context_register_device_collection_changed_twice_output() {
    test_ops_context_register_device_collection_changed_twice(ffi::CUBEB_DEVICE_TYPE_OUTPUT);
}

#[test]
#[ignore]
#[should_panic]
fn test_ops_context_register_device_collection_changed_twice_inout() {
    test_ops_context_register_device_collection_changed_twice(ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT);
}

fn test_context_register_device_collection_changed_twice(devtype: DeviceType) {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext, since those OwnedCriticalSection
    // will be used when register_device_collection_changed is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    extern "C" fn callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    assert!(
        ctx.register_device_collection_changed(
            DeviceType::INPUT,
            Some(callback),
            ptr::null_mut()
        ).is_ok();
    );

    assert!(
        ctx.register_device_collection_changed(
            DeviceType::INPUT,
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
        ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT
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
#[test]
fn test_stream_set_volume() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    let name = CString::new("test set valume").expect("CString::new failed");

    // The resampler will be initialized in `audiounit_setup_stream` (or via
    // `stream_init`), and it only accepts the formats with FLOAT32NE or S16NE.
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 44100;
    raw.channels = 2;
    raw.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;

    let params = StreamParams::from(raw);
    let stream = ctx.stream_init(
        Some(&name),
        ptr::null(),
        None,
        ptr::null(),
        Some(&&params),
        4096,
        None,
        None,
        ptr::null_mut()
    ).unwrap();

    assert!(stream.set_volume(0.5).is_ok());

    // stream should be dropped autmatically.
    // See the implementation of the ffi_type_heap macro.
}

// #[test]
// fn test_ops_stream_set_panning() {
//     let s: *mut ffi::cubeb_stream = ptr::null_mut();
//     unsafe {
//         OPS.stream_set_panning.unwrap()(s, 0.5);
//     }
// }

#[test]
fn test_stream_set_panning() {
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    let name = CString::new("test set panning").expect("CString::new failed");

    // TODO: `test_context_register_device_collection_changed_twice`
    //       cannot be executed if the stream is initializeed with input
    //       parameter and out parameter at the same time. If a stream
    //       works for both input and output scope, we will create an
    //       aggregate device and fire the
    //       `audiounit_collection_changed_callback` indirectly (see the NOTE
    //       in `audiounit_create_blank_aggregate_device`) and then cause a
    //       EXC_BAD_ACCESS error for the same reason as the case 1 commented
    //       in the comment above `test_create_blank_aggregate_device`.

    // let mut raw_in = ffi::cubeb_stream_params::default();
    // raw_in.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    // raw_in.rate = 48_000;
    // raw_in.channels = 1;
    // raw_in.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
    // raw_in.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    // let params_in = StreamParams::from(raw_in);

    // The resampler will be initialized in `audiounit_setup_stream` (or via
    // `stream_init`), and it only accepts the formats with FLOAT32NE or S16NE.
    let mut raw_out = ffi::cubeb_stream_params::default();
    raw_out.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw_out.rate = 44100;
    raw_out.channels = 2;
    raw_out.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
    raw_out.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    let params_out = StreamParams::from(raw_out);

    // TODO: What if `raw_in.format` and `raw_out.format` are different ?

    let stream = ctx.stream_init(
        Some(&name),
        ptr::null(),
        None, // Some(&&params_in),
        ptr::null(),
        Some(&&params_out),
        4096,
        None,
        None,
        ptr::null_mut()
    ).unwrap();

    if ctx.max_channel_count().unwrap() > 2 {
        assert_eq!(
            stream.set_panning(0.5).unwrap_err(),
            Error::invalid_format()
        );
    } else {
        assert!(stream.set_panning(0.5).is_ok());
    }

    // stream should be dropped autmatically.
    // See the implementation of the ffi_type_heap macro.
}

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

// #[test]
// fn test_ops_register_device_changed_callback() {
// }

#[test]
#[ignore]
fn test_manual_stream_register_device_changed_callback() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    const USER_PTR: *mut c_void = 0xDEAD_BEEF as *mut c_void;
    let mut stream = AudioUnitStream::new(
        &mut ctx,
        USER_PTR,
        None,
        None,
        0
    );
    stream.init();

    // The resampler will be initialized in `audiounit_setup_stream` (or via
    // `stream_init`), and it only accepts the formats with FLOAT32NE or S16NE.
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 96_000;
    raw.channels = 4;
    raw.layout = ffi::CUBEB_LAYOUT_3F1_LFE;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);
    // TODO: Return an error if input has specific layout ?
    //       Should input layout always be defined as `undefined` ?
    stream.input_stream_params = StreamParams::from(raw);

    // It's crucial to call to audiounit_set_device_info to set
    // stream.input_device stream.output_device to input and output device
    // type, or we will hit the assertion in audiounit_create_unit.

    let default_input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    // Return an error if there is no available device.
    if  !valid_id(default_input_id) || !valid_id(default_output_id) {
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::OUTPUT
        ).is_ok()
    );

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::INPUT
        ).is_ok()
    );

    assert_eq!(stream.input_device.id, default_input_id);
    assert_eq!(
        stream.input_device.flags,
        device_flags::DEV_INPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );


    {
        let ctx_mutex_ptr = &mut stream.context.mutex as *mut OwnedCriticalSection;
        let _ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        let stm_mutex_ptr = &mut stream.mutex as *mut OwnedCriticalSection;
        let _stm_lock = AutoLock::new(unsafe { &mut (*stm_mutex_ptr) });
        assert!(audiounit_setup_stream(&mut stream).is_ok());
    }

    extern "C" fn on_device_changed(user: *mut c_void) {
        assert_eq!(user, USER_PTR);
        println!("on_device_changed: user_ptr = {:p}", user);
    }

    assert!(stream.register_device_changed_callback(Some(on_device_changed)).is_ok());

    loop {}
}

#[test]
#[ignore]
fn test_manual_ctx_stream_register_device_changed_callback() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    const USER_PTR: *mut c_void = 0xDEAD_BEEF as *mut c_void;
    let name = CString::new("test register device changed callback").expect("CString::new failed");

    // The resampler will be initialized in `audiounit_setup_stream` (or via
    // `stream_init`), and it only accepts the formats with FLOAT32NE or S16NE.
    let mut raw_in = ffi::cubeb_stream_params::default();
    raw_in.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw_in.rate = 48_000;
    raw_in.channels = 1;
    raw_in.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
    raw_in.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    let params_in = StreamParams::from(raw_in);

    let mut raw_out = ffi::cubeb_stream_params::default();
    raw_out.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw_out.rate = 44100;
    raw_out.channels = 2;
    raw_out.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
    raw_out.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    let params_out = StreamParams::from(raw_out);

    // TODO: What if `raw_in.format` and `raw_out.format` are different ?

    let stream = ctx.stream_init(
        Some(&name),
        ptr::null(),
        Some(&&params_in),
        ptr::null(),
        Some(&&params_out),
        4096,
        None,
        None,
        USER_PTR
    ).unwrap();

    extern "C" fn on_device_changed(user: *mut c_void) {
        assert_eq!(user, USER_PTR);
        println!("on_device_changed: user_ptr = {:p}", user);
    }

    assert!(stream.register_device_changed_callback(Some(on_device_changed)).is_ok());

    loop {}

    // stream should be dropped autmatically.
    // See the implementation of the ffi_type_heap macro.
}

// Private APIs
// ============================================================================
// make_sized_audio_channel_layout
// ------------------------------------
#[test]
#[should_panic]
fn test_make_sized_audio_channel_layout_with_wrong_size() {
    // let _ = make_sized_audio_channel_layout(0);
    let one_channel_size = mem::size_of::<AudioChannelLayout>();
    let padding_size = 10;
    assert_ne!(mem::size_of::<AudioChannelDescription>(), padding_size);
    let wrong_size = one_channel_size + padding_size;
    let _ = make_sized_audio_channel_layout(wrong_size);
}

#[test]
fn test_make_sized_audio_channel_layout() {
    for channels in 1..10 {
        let size = mem::size_of::<AudioChannelLayout>()
            + (channels - 1) * mem::size_of::<AudioChannelDescription>();
        let _ = make_sized_audio_channel_layout(size);
    }
}

// to_string
// ------------------------------------
#[test]
fn test_to_string() {
    assert_eq!(
        to_string(&io_side::INPUT),
        "input"
    );
    assert_eq!(
        to_string(&io_side::OUTPUT),
        "output"
    );
}

// has_input
// ------------------------------------
// TODO

// has_output
// ------------------------------------
// TODO

// channel_label_to_cubeb_channel
// ------------------------------------
#[test]
fn test_channel_label_to_cubeb_channel() {
    let pairs = [
        (kAudioChannelLabel_Left, ChannelLayout::FRONT_LEFT),
        (kAudioChannelLabel_Right, ChannelLayout::FRONT_RIGHT),
        (kAudioChannelLabel_Center, ChannelLayout::FRONT_CENTER),
        (kAudioChannelLabel_LFEScreen, ChannelLayout::LOW_FREQUENCY),
        (kAudioChannelLabel_LeftSurround, ChannelLayout::BACK_LEFT),
        (kAudioChannelLabel_RightSurround, ChannelLayout::BACK_RIGHT),
        (kAudioChannelLabel_LeftCenter, ChannelLayout::FRONT_LEFT_OF_CENTER),
        (kAudioChannelLabel_RightCenter, ChannelLayout::FRONT_RIGHT_OF_CENTER),
        (kAudioChannelLabel_CenterSurround, ChannelLayout::BACK_CENTER),
        (kAudioChannelLabel_LeftSurroundDirect, ChannelLayout::SIDE_LEFT),
        (kAudioChannelLabel_RightSurroundDirect, ChannelLayout::SIDE_RIGHT),
        (kAudioChannelLabel_TopCenterSurround, ChannelLayout::TOP_CENTER),
        (kAudioChannelLabel_VerticalHeightLeft, ChannelLayout::TOP_FRONT_LEFT),
        (kAudioChannelLabel_VerticalHeightCenter, ChannelLayout::TOP_FRONT_CENTER),
        (kAudioChannelLabel_VerticalHeightRight, ChannelLayout::TOP_FRONT_RIGHT),
        (kAudioChannelLabel_TopBackLeft, ChannelLayout::TOP_BACK_LEFT),
        (kAudioChannelLabel_TopBackCenter, ChannelLayout::TOP_BACK_CENTER),
        (kAudioChannelLabel_TopBackRight, ChannelLayout::TOP_BACK_RIGHT),
        (kAudioChannelLabel_Unknown, ChannelLayout::UNDEFINED),
    ];

    for (label, channel) in pairs.iter() {
        assert_eq!(
            channel_label_to_cubeb_channel(*label),
            *channel
        );
    }
}

// cubeb_channel_to_channel_label
// ------------------------------------
#[test]
#[should_panic]
fn test_cubeb_channel_to_channel_label_with_invalid_channel() {
    assert_eq!(
        cubeb_channel_to_channel_label(ChannelLayout::_3F4_LFE),
        kAudioChannelLabel_Unknown
    );
}

#[test]
#[should_panic]
fn test_cubeb_channel_to_channel_label_with_unknown_channel() {
    assert_eq!(ChannelLayout::from(ffi::CHANNEL_UNKNOWN), ChannelLayout::UNDEFINED);
    assert_eq!(
        cubeb_channel_to_channel_label(ChannelLayout::UNDEFINED),
        kAudioChannelLabel_Unknown
    );
}

#[test]
fn test_cubeb_channel_to_channel_label() {
    let pairs = [
        (ChannelLayout::FRONT_LEFT, kAudioChannelLabel_Left),
        (ChannelLayout::FRONT_RIGHT, kAudioChannelLabel_Right),
        (ChannelLayout::FRONT_CENTER, kAudioChannelLabel_Center),
        (ChannelLayout::LOW_FREQUENCY, kAudioChannelLabel_LFEScreen),
        (ChannelLayout::BACK_LEFT, kAudioChannelLabel_LeftSurround),
        (ChannelLayout::BACK_RIGHT, kAudioChannelLabel_RightSurround),
        (ChannelLayout::FRONT_LEFT_OF_CENTER, kAudioChannelLabel_LeftCenter),
        (ChannelLayout::FRONT_RIGHT_OF_CENTER, kAudioChannelLabel_RightCenter),
        (ChannelLayout::BACK_CENTER, kAudioChannelLabel_CenterSurround),
        (ChannelLayout::SIDE_LEFT, kAudioChannelLabel_LeftSurroundDirect),
        (ChannelLayout::SIDE_RIGHT, kAudioChannelLabel_RightSurroundDirect),
        (ChannelLayout::TOP_CENTER, kAudioChannelLabel_TopCenterSurround),
        (ChannelLayout::TOP_FRONT_LEFT, kAudioChannelLabel_VerticalHeightLeft),
        (ChannelLayout::TOP_FRONT_CENTER, kAudioChannelLabel_VerticalHeightCenter),
        (ChannelLayout::TOP_FRONT_RIGHT, kAudioChannelLabel_VerticalHeightRight),
        (ChannelLayout::TOP_BACK_LEFT, kAudioChannelLabel_TopBackLeft),
        (ChannelLayout::TOP_BACK_CENTER, kAudioChannelLabel_TopBackCenter),
        (ChannelLayout::TOP_BACK_RIGHT, kAudioChannelLabel_TopBackRight),
    ];

    for (channel, label) in pairs.iter() {
        assert_eq!(
            cubeb_channel_to_channel_label(*channel),
            *label
        );
    }
}

// increment_active_streams
// decrement_active_streams
// active_streams
// ------------------------------------
#[test]
fn test_increase_and_decrease_active_streams() {
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
    let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
    // The scope of `_lock` is a critical section.
    let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });

    assert_eq!(ctx.active_streams, 0);
    for i in 1..10 {
        audiounit_increment_active_streams(&mut ctx);
        assert_eq!(ctx.active_streams, i);
        assert_eq!(audiounit_active_streams(&mut ctx), i);
    }

    for i in (0..9).rev() {
        audiounit_decrement_active_streams(&mut ctx);
        assert_eq!(ctx.active_streams, i);
        assert_eq!(audiounit_active_streams(&mut ctx), i);
    }
}

// set_global_latency
// ------------------------------------
fn test_set_global_latency() {
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
    let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
    // The scope of `_lock` is a critical section.
    let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });

    assert_eq!(ctx.active_streams, 0);
    audiounit_increment_active_streams(&mut ctx);
    assert_eq!(ctx.active_streams, 1);

    for i in 0..10 {
        audiounit_set_global_latency(&mut ctx, i);
        assert_eq!(ctx.global_latency_frames, i);
    }
}

// make_silent
// ------------------------------------
#[test]
fn test_make_silent() {
    let mut array = allocate_array::<u32>(10);
    for data in array.iter_mut() {
        *data = 0xFFFF;
    }

    let mut buffer = AudioBuffer::default();
    buffer.mData = array.as_mut_ptr() as *mut c_void;
    buffer.mDataByteSize = (array.len() * mem::size_of::<u32>()) as u32;
    buffer.mNumberChannels = 1;

    audiounit_make_silent(&mut buffer);
    for data in array {
        assert_eq!(data, 0);
    }
}

// render_input
// ------------------------------------
// TODO

// input_callback
// ------------------------------------
// TODO

// mix_output_buffer
// ------------------------------------
// TODO

// minimum_resampling_input_frames
// ------------------------------------
#[test]
#[should_panic]
fn test_minimum_resampling_input_frames_zero_input_rate() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    // Set output rate to 44100
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 44100;
    raw.channels = 2;
    raw.layout = ffi::CUBEB_LAYOUT_STEREO;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // Set input rate to 0
    stream.input_hw_rate = 0_f64;

    // Set frames to 100
    let frames = 100;

    assert_eq!(
        minimum_resampling_input_frames(&stream, frames),
        0
    );
}

#[test]
#[should_panic]
fn test_minimum_resampling_input_frames_zero_output_rate() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    // Set output rate to 0
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 0;
    raw.channels = 2;
    raw.layout = ffi::CUBEB_LAYOUT_STEREO;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // Set input rate to 48000
    stream.input_hw_rate = 48000_f64;

    // Set frames to 100
    let frames = 100;

    assert_eq!(
        minimum_resampling_input_frames(&stream, frames),
        i64::min_value()
    );
}

#[test]
fn test_minimum_resampling_input_frames_equal_input_output_rate() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    // Set output rate to 44100
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 44100;
    raw.channels = 2;
    raw.layout = ffi::CUBEB_LAYOUT_STEREO;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // Set input rate to 44100
    stream.input_hw_rate = 44100_f64;

    // Set frames to 100
    let frames = 100;

    assert_eq!(
        minimum_resampling_input_frames(&stream, frames),
        frames
    );
}

#[test]
fn test_minimum_resampling_input_frames() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    // Set output rate to 44100
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 44100;
    raw.channels = 2;
    raw.layout = ffi::CUBEB_LAYOUT_STEREO;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // Set input rate to 48000
    stream.input_hw_rate = 48000_f64;

    // Set frames to 100
    let frames = 100;

    assert_eq!(
        minimum_resampling_input_frames(&stream, frames),
        (stream.input_hw_rate * frames as f64 / raw.rate as f64).ceil() as i64
    );
}

// output_callback
// ------------------------------------
// TODO

// set_device_info
// ------------------------------------
#[test]
fn test_set_device_info_for_unknown_input_device() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert_eq!(stream.input_device.id, kAudioObjectUnknown);
    assert_eq!(stream.input_device.flags, device_flags::DEV_UNKNOWN);

    let default_input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    // Return an error if there is no available device.
    if !valid_id(default_input_id) {
        assert_eq!(
            audiounit_set_device_info(
                &mut stream,
                kAudioObjectUnknown,
                io_side::INPUT
            ).unwrap_err(),
            Error::error()
        );
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::INPUT
        ).is_ok()
    );

    assert_eq!(stream.input_device.id, default_input_id);
    assert_eq!(
        stream.input_device.flags,
        device_flags::DEV_INPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );
}

#[test]
fn test_set_device_info_for_unknown_output_device() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert_eq!(stream.output_device.id, kAudioObjectUnknown);
    assert_eq!(stream.output_device.flags, device_flags::DEV_UNKNOWN);

    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    // Return an error if there is no available device.
    if !valid_id(default_output_id) {
        assert_eq!(
            audiounit_set_device_info(
                &mut stream,
                kAudioObjectUnknown,
                io_side::OUTPUT
            ).unwrap_err(),
            Error::error()
        );
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::OUTPUT
        ).is_ok()
    );

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );
}

// FIXIT: Should we set {input, output}_device as the default one
//        if user pass `kAudioObjectSystemObject` as device id ?
#[test]
#[ignore]
fn test_set_device_info_for_system_input_device() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert_eq!(stream.input_device.id, kAudioObjectUnknown);
    assert_eq!(stream.input_device.flags, device_flags::DEV_UNKNOWN);

    let default_input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    // Return an error if there is no available device.
    if !valid_id(default_input_id) {
        assert_eq!(
            audiounit_set_device_info(
                &mut stream,
                kAudioObjectSystemObject,
                io_side::INPUT
            ).unwrap_err(),
            Error::error()
        );
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectSystemObject,
            io_side::INPUT
        ).is_ok()
    );

    assert_eq!(stream.input_device.id, default_input_id);
    assert_eq!(
        stream.input_device.flags,
        device_flags::DEV_INPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );
}

// FIXIT: Should we set {input, output}_device as the default one
//        if user pass `kAudioObjectSystemObject` as device id ?
#[test]
#[ignore]
fn test_set_device_info_for_system_output_device() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert_eq!(stream.output_device.id, kAudioObjectUnknown);
    assert_eq!(stream.output_device.flags, device_flags::DEV_UNKNOWN);

    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    // Return an error if there is no available device.
    if !valid_id(default_output_id) {
        assert_eq!(
            audiounit_set_device_info(
                &mut stream,
                kAudioObjectSystemObject,
                io_side::OUTPUT
            ).unwrap_err(),
            Error::error()
        );
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectSystemObject,
            io_side::OUTPUT
        ).is_ok()
    );

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );
}

// FIXIT: We should prevent the device from being assigned to a nonexistent
//        device.
#[test]
#[ignore]
fn test_set_device_info_for_nonexistent_input_device() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert_eq!(stream.input_device.id, kAudioObjectUnknown);
    assert_eq!(stream.input_device.flags, device_flags::DEV_UNKNOWN);

    let input_devices = audiounit_get_devices_of_type(DeviceType::INPUT);
    if input_devices.is_empty() {
        return;
    }

    // Find a nonexistent device. Start from 2, since 0 is kAudioObjectUnknown and
    // 1 is kAudioObjectSystemObject.
    let mut id: AudioDeviceID = 2;
    while input_devices.contains(&id) {
        id += 1;
    }

    assert_eq!(
        audiounit_set_device_info(
            &mut stream,
            id,
            io_side::INPUT
        ).unwrap_err(),
        Error::error()
    );
}

// FIXIT: We should prevent the device from being assigned to a nonexistent
//        device.
#[test]
#[ignore]
fn test_set_device_info_for_nonexistent_output_device() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert_eq!(stream.output_device.id, kAudioObjectUnknown);
    assert_eq!(stream.output_device.flags, device_flags::DEV_UNKNOWN);

    let output_devices = audiounit_get_devices_of_type(DeviceType::OUTPUT);
    if output_devices.is_empty() {
        return;
    }

    // Find a nonexistent device. Start from 2, since 0 is kAudioObjectUnknown and
    // 1 is kAudioObjectSystemObject.
    let mut id: AudioDeviceID = 2;
    while output_devices.contains(&id) {
        id += 1;
    }

    assert_eq!(
        audiounit_set_device_info(
            &mut stream,
            id,
            io_side::OUTPUT
        ).unwrap_err(),
        Error::error()
    );
}

// reinit_stream
// ------------------------------------
// TODO

// reinit_stream_async
// ------------------------------------
// TODO

// event_addr_to_string
// ------------------------------------
// TODO

// property_listener_callback
// ------------------------------------
// TODO

// add_listener
// ------------------------------------
#[test]
fn test_add_listener_for_unknown_device() {
    extern fn listener(
        _: AudioObjectID,
        _: u32,
        _: *const AudioObjectPropertyAddress,
        _: *mut c_void
    ) -> OSStatus {
        assert!(false, "Should not be called.");
        kAudioHardwareUnspecifiedError as OSStatus
    }

    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    let mut listener = property_listener::new(
        kAudioObjectUnknown,
        &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
        listener,
        &mut stream
    );

    assert_eq!(
        audiounit_add_listener(&mut listener),
        kAudioHardwareBadObjectError as OSStatus
    );
}

// remove_listener
// ------------------------------------
#[test]
fn test_remove_listener_for_unknown_device() {
    extern fn listener(
        _: AudioObjectID,
        _: u32,
        _: *const AudioObjectPropertyAddress,
        _: *mut c_void
    ) -> OSStatus {
        assert!(false, "Should not be called.");
        kAudioHardwareUnspecifiedError as OSStatus
    }

    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    let mut listener = property_listener::new(
        kAudioObjectUnknown,
        &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
        listener,
        &mut stream
    );

    assert_eq!(
        audiounit_remove_listener(&mut listener),
        kAudioHardwareBadObjectError as OSStatus
    );
}

#[test]
fn test_remove_listener_without_adding_any_listener() {
    extern fn listener(
        _: AudioObjectID,
        _: u32,
        _: *const AudioObjectPropertyAddress,
        _: *mut c_void
    ) -> OSStatus {
        assert!(false, "Should not be called.");
        kAudioHardwareUnspecifiedError as OSStatus
    }

    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    let mut listener = property_listener::new(
        kAudioObjectSystemObject,
        &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
        listener,
        &mut stream
    );

    assert_eq!(
        audiounit_remove_listener(&mut listener),
        0
    );
}

#[test]
fn test_add_then_remove_listener() {
    extern fn listener(
        _: AudioObjectID,
        _: u32,
        _: *const AudioObjectPropertyAddress,
        _: *mut c_void
    ) -> OSStatus {
        assert!(false, "Should not be called.");
        kAudioHardwareUnspecifiedError as OSStatus
    }

    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    let mut listener = property_listener::new(
        kAudioObjectSystemObject,
        &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
        listener,
        &mut stream
    );

    assert_eq!(
        audiounit_add_listener(&mut listener),
        0
    );

    assert_eq!(
        audiounit_remove_listener(&mut listener),
        0
    );
}

// install_system_changed_callback
// ------------------------------------
// TODO

// uninstall_system_changed_callback
// ------------------------------------
// TODO

// get_acceptable_latency_range
// ------------------------------------
#[test]
fn test_get_acceptable_latency_range() {
    let mut latency_range = AudioValueRange::default();

    // Get an error if there is no avaiable output device.
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(output_id) {
        assert_eq!(
            audiounit_get_acceptable_latency_range(
                &mut latency_range
            ).unwrap_err(),
            Error::error()
        );
        return;
    }

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

// convert_channel_layout
// ------------------------------------
#[test]
fn test_convert_channel_layout() {
    let pairs = [
        // The single channel is mapped to mono now.
        (vec![kAudioObjectUnknown], ChannelLayout::MONO),
        (vec![kAudioChannelLabel_Mono], ChannelLayout::MONO),
        // The dual channels are mapped to stereo now.
        (
            vec![kAudioChannelLabel_Mono, kAudioChannelLabel_LFEScreen],
            ChannelLayout::STEREO,
        ),
        (
            vec![kAudioChannelLabel_Left, kAudioChannelLabel_Right],
            ChannelLayout::STEREO,
        ),
        // The Layouts containing any unknonwn channel will be mapped to UNDEFINED.
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Unknown,
            ],
            ChannelLayout::UNDEFINED,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Unused,
            ],
            ChannelLayout::UNDEFINED,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_ForeignLanguage,
            ],
            ChannelLayout::UNDEFINED,
        ),
        // The SMPTE layouts.
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::STEREO_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
            ],
            ChannelLayout::_3F,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_3F_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_CenterSurround,
            ],
            ChannelLayout::_2F1,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_CenterSurround,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_2F1_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_CenterSurround,
            ],
            ChannelLayout::_3F1,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_CenterSurround,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_3F1_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
            ],
            ChannelLayout::_2F2,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_2F2_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurround,
                kAudioChannelLabel_RightSurround,
            ],
            ChannelLayout::QUAD,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurround,
                kAudioChannelLabel_RightSurround,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::QUAD_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
            ],
            ChannelLayout::_3F2,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_3F2_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurround,
                kAudioChannelLabel_RightSurround,
                kAudioChannelLabel_Center,
            ],
            ChannelLayout::_3F2_BACK,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurround,
                kAudioChannelLabel_RightSurround,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_3F2_LFE_BACK,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LFEScreen,
                kAudioChannelLabel_CenterSurround,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
            ],
            ChannelLayout::_3F3R_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LFEScreen,
                kAudioChannelLabel_LeftSurround,
                kAudioChannelLabel_RightSurround,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
            ],
            ChannelLayout::_3F4_LFE,
        ),
    ];

    const MAX_CHANNELS: usize = 10;
    // A Rust mapping structure of the AudioChannelLayout with MAX_CHANNELS channels
    // https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.13.sdk/System/Library/Frameworks/CoreAudio.framework/Versions/A/Headers/CoreAudioTypes.h#L1332
    #[repr(C)]
    struct TestLayout {
        tag: AudioChannelLayoutTag,
        map: AudioChannelBitmap,
        number_channel_descriptions: UInt32,
        channel_descriptions: [AudioChannelDescription; MAX_CHANNELS],
    }

    impl Default for TestLayout {
        fn default() -> Self {
            Self {
                tag: AudioChannelLayoutTag::default(),
                map: AudioChannelBitmap::default(),
                number_channel_descriptions: UInt32::default(),
                channel_descriptions: [AudioChannelDescription::default(); MAX_CHANNELS],
            }
        }
    }

    let mut layout = TestLayout::default();
    layout.tag = kAudioChannelLayoutTag_UseChannelDescriptions;

    for (labels, expected_layout) in pairs.iter() {
        assert!(labels.len() <= MAX_CHANNELS);
        layout.number_channel_descriptions = labels.len() as u32;
        for (idx, label) in labels.iter().enumerate() {
            layout.channel_descriptions[idx].mChannelLabel = *label;
        }
        let layout_ref = unsafe { &(*(&layout as *const TestLayout as *const AudioChannelLayout)) };
        assert_eq!(
            audiounit_convert_channel_layout(layout_ref),
            *expected_layout
        );
    }
}

// get_preferred_channel_layout
// ------------------------------------
// TODO: Should it be prevented ? The AudioUnitElement is for output only.
//       It should be called for the input side.
// #[test]
// fn test_get_preferred_channel_layout_input() {
// }

#[test]
fn test_get_preferred_channel_layout_output() {
    // Initialize the unit to default output device.
    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(default_output_id) {
        return;
    }
    let mut unit: AudioUnit = ptr::null_mut();
    let device = device_info {
        id: default_output_id,
        flags: device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT
    };
    assert!(audiounit_create_unit(&mut unit, &device).is_ok());
    assert!(!unit.is_null());

    // TODO: The preferred layout might be undefined for some devices ?
    assert_ne!(audiounit_get_preferred_channel_layout(unit), ChannelLayout::UNDEFINED);
}

// get_current_channel_layout
// ------------------------------------
// TODO: Should it be prevented ? The AudioUnitElement is for output only.
//       It should be called for the input side.
// #[test]
// fn test_get_current_channel_layout_input() {
//     // Initialize the unit to the default input device.
//     let default_input_id = audiounit_get_default_device_id(DeviceType::INPUT);
//     if !valid_id(default_input_id) {
//         return;
//     }
//     let mut unit: AudioUnit = ptr::null_mut();
//     let device = device_info {
//         id: default_input_id,
//         flags: device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT
//     };
//     assert!(audiounit_create_unit(&mut unit, &device).is_ok());
//     assert!(!unit.is_null());

//     assert_eq!(audiounit_get_current_channel_layout(unit), ChannelLayout::UNDEFINED);
// }

#[test]
fn test_get_current_channel_layout_output() {
    // Initialize the unit to default output device.
    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(default_output_id) {
        return;
    }
    let mut unit: AudioUnit = ptr::null_mut();
    let device = device_info {
        id: default_output_id,
        flags: device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT
    };
    assert!(audiounit_create_unit(&mut unit, &device).is_ok());
    assert!(!unit.is_null());

    // TODO: The current layout might be undefined for some devices ?
    assert_ne!(audiounit_get_current_channel_layout(unit), ChannelLayout::UNDEFINED);
}

// audio_stream_desc_init
// ------------------------------------
#[test]
fn test_audio_stream_desc_init() {
    let mut channels = 0;
    for (bits, format, flags) in [
        (16_u32, ffi::CUBEB_SAMPLE_S16LE, kAudioFormatFlagIsSignedInteger),
        (16_u32, ffi::CUBEB_SAMPLE_S16BE, kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsBigEndian),
        (32_u32, ffi::CUBEB_SAMPLE_FLOAT32LE, kAudioFormatFlagIsFloat),
        (32_u32, ffi::CUBEB_SAMPLE_FLOAT32BE, kAudioFormatFlagIsFloat | kAudioFormatFlagIsBigEndian),
    ].iter() {
        channels += 1;

        let mut raw = ffi::cubeb_stream_params::default();
        raw.format = *format;
        raw.rate = 48_000;
        raw.channels = channels;
        raw.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
        raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;

        let params = StreamParams::from(raw);

        let mut stream_description = AudioStreamBasicDescription::default();

        assert!(
            audio_stream_desc_init(
                &mut stream_description,
                &params
            ).is_ok()
        );

        assert_eq!(stream_description.mFormatID, kAudioFormatLinearPCM);
        assert_eq!(stream_description.mFormatFlags, flags | kLinearPCMFormatFlagIsPacked);
        assert_eq!(stream_description.mSampleRate as u32, raw.rate);
        assert_eq!(stream_description.mChannelsPerFrame, raw.channels);
        assert_eq!(stream_description.mBytesPerFrame, (bits / 8) * raw.channels);
        assert_eq!(stream_description.mFramesPerPacket, 1);
        assert_eq!(stream_description.mBytesPerPacket, (bits / 8) * raw.channels);
        assert_eq!(stream_description.mReserved, 0);
    }
}

// init_mixer
// ------------------------------------
#[test]
fn test_init_mixer() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
    let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;

    // Add one stream to the context in advance to avoid the borrowing-twice
    // issue of ctx.
    // `AudioUnitStream::drop()` will check the context has at least one stream.
    {
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr ) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    audiounit_init_mixer(&mut stream);
    assert!(!stream.mixer.as_mut_ptr().is_null());
}

// set_channel_layout
// ------------------------------------
#[test]
#[should_panic]
fn test_set_channel_layout_with_null_unit() {
    assert!(audiounit_set_channel_layout(ptr::null_mut(), io_side::OUTPUT, ChannelLayout::UNDEFINED).is_err());
}

#[test]
fn test_set_channel_layout_undefind_layout() {
    // Initialize the unit to default output device.
    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(default_output_id) {
        return;
    }
    let mut unit: AudioUnit = ptr::null_mut();
    let device = device_info {
        id: default_output_id,
        flags: device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT
    };
    assert!(audiounit_create_unit(&mut unit, &device).is_ok());
    assert!(!unit.is_null());

    // Get original layout.
    let original_layout = audiounit_get_current_channel_layout(unit);

    // Leave layout as it is.
    assert!(audiounit_set_channel_layout(unit, io_side::OUTPUT, ChannelLayout::UNDEFINED).is_ok());

    // Check the layout is same as the original one.
    assert_eq!(
        audiounit_get_current_channel_layout(unit),
        original_layout
    );
}

#[test]
fn test_set_channel_layout_input() {
    // Initialize the unit to the default input device.
    let default_input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    if !valid_id(default_input_id) {
        return;
    }
    let mut unit: AudioUnit = ptr::null_mut();
    let device = device_info {
        id: default_input_id,
        flags: device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT
    };
    assert!(audiounit_create_unit(&mut unit, &device).is_ok());
    assert!(!unit.is_null());

    assert_eq!(
        audiounit_set_channel_layout(unit, io_side::INPUT, ChannelLayout::UNDEFINED).unwrap_err(),
        Error::error()
    );
}

#[test]
fn test_set_channel_layout_output() {
    // TODO: Add more devices and its available layouts.
    use std::collections::HashMap;
    let devices_layouts: HashMap<&'static str, Vec<ChannelLayout>> = [
        ("hdpn", vec![ChannelLayout::STEREO]),
        ("ispk", vec![ChannelLayout::STEREO]),
    ].into_iter().cloned().collect();

    // Initialize the unit to default output device.
    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(default_output_id) {
        return;
    }
    let mut unit: AudioUnit = ptr::null_mut();
    let device = device_info {
        id: default_output_id,
        flags: device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT
    };
    assert!(audiounit_create_unit(&mut unit, &device).is_ok());
    assert!(!unit.is_null());

    let mut device = ffi::cubeb_device::default();
    assert!(
        audiounit_get_default_device_name(
            unsafe { &*(ptr::null() as *const AudioUnitStream) },
            &mut device,
            DeviceType::OUTPUT
        ).is_ok()
    );

    let device_name = unsafe {
        CStr::from_ptr(device.output_name)
            .to_string_lossy()
            .into_owned()
    };

    if let Some(layouts) = devices_layouts.get(device_name.as_str()) {
        for layout in layouts.iter() {
            assert!(audiounit_set_channel_layout(unit, io_side::OUTPUT, *layout).is_ok());
            assert_eq!(
                audiounit_get_current_channel_layout(unit),
                *layout
            );
        }
    }
}

// layout_init
// ------------------------------------
#[test]
fn test_layout_init() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
    let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;

    // Add one stream to the context in advance to avoid the borrowing-twice
    // issue of ctx.
    // `AudioUnitStream::drop()` will check the context has at least one stream.
    {
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr ) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    // The scope of `ctx_lock` is a critical section.
    // When `AudioUnitStream::drop()` is called, `AudioUnitContext.mutex`
    // needs to be unlocked. That's why `_lock` needs to be declared after
    // `stream` so it will be dropped and unlocked before dropping `stream`.
    let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });

    // The resampler will be initialized in `audiounit_setup_stream` (or via
    // `stream_init`), and it only accepts the formats with FLOAT32NE or S16NE.
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 96_000;
    raw.channels = 32;
    raw.layout = ffi::CUBEB_LAYOUT_3F1_LFE;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // It's crucial to call to audiounit_set_device_info to set
    // stream.output_device to output device type, or we will hit the
    // assertion in audiounit_create_unit.

    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    // Return an error if there is no available device.
    if !valid_id(default_output_id) {
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::OUTPUT
        ).is_ok()
    );

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(audiounit_create_unit(&mut stream.output_unit, &stream.output_device).is_ok());
    assert!(!stream.output_unit.is_null());
    assert_eq!(
        stream.context.layout.load(atomic::Ordering::SeqCst),
        ChannelLayout::UNDEFINED
    );

    let layout = audiounit_get_current_channel_layout(stream.output_unit);
    audiounit_layout_init(&mut stream, io_side::OUTPUT);
    assert_eq!(
        stream.context.layout.load(atomic::Ordering::SeqCst),
        layout
    );
}

// get_sub_devices
// ------------------------------------
// FIXIT: It doesn't make any sense to return the sub devices for an unknown
//        device! It should either get a panic or return an empty list!
#[test]
// #[should_panic]
#[ignore]
fn test_get_sub_devices_for_a_unknown_device() {
    let devices = audiounit_get_sub_devices(kAudioObjectUnknown);
    assert!(devices.is_empty());
}

// You can check this by creating an aggregate device in `Audio MIDI Setup`
// application and print out the sub devices of them!
#[test]
fn test_get_sub_devices() {
    let devices = audiounit_get_devices_of_type(DeviceType::INPUT | DeviceType::OUTPUT);
    for device in devices {
        assert!(valid_id(device));
        // `audiounit_get_sub_devices(device)` will return a one-element vector
        //  containing `device` itself if it's not an aggregate device.
        let sub_devices = audiounit_get_sub_devices(device);
        // TODO: If device is a blank aggregate device, then the assertion fails!
        assert!(!sub_devices.is_empty());
    }
}

// Ignore this by default. The reason is same as below.
#[test]
#[ignore]
fn test_get_sub_devices_for_blank_aggregate_devices() {
    // TODO: Test this when there is no available devices.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(
            &mut plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
    assert_ne!(
        plugin_id,
        kAudioObjectUnknown
    );
    assert_ne!(
        aggregate_device_id,
        kAudioObjectUnknown
    );
    // There is no sub devices for a blank aggregate device!
    let devices = audiounit_get_sub_devices(aggregate_device_id);
    assert!(devices.is_empty());

    assert!(
        audiounit_destroy_aggregate_device(
            plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
}

// create_blank_aggregate_device
// ------------------------------------
// This is marked as `ignore` by default since it cannot run with those
// tests calling `audiounit_add_device_listener` directly or indirectly
// (via `register_device_collection_changed`) at the same time.
//
// The `audiounit_collection_changed_callback` will be fired upon
// `audiounit_create_blank_aggregate_device` is called.
// In `audiounit_collection_changed_callback`, it will register an asynchronous
// function to notify the device-collection is changed. In current
// implementation, those asynchronous functions might cause the following
// errors:
//
// 1. If those tests calling `audiounit_add_device_listener` is finished
//    before those asynchronous functions fired by
//    `audiounit_collection_changed_callback` start executing,
//    without unregistering the callback by `audiounit_remove_device_listener`,
//    when those asynchronous functions are executed, their pointers to those
//    contexts declared in the tests are already destroyed. So we will get a
//    EXC_BAD_ACCESS error when we try dereferencing the destroyed pointers
//    that should be pointed to the alive contexts. Thus, it's critical to make
//    sure the device-collection callback is unregistered for the context about
//    to be destroyed!
//
//    One example is to run `test_context_register_device_collection_changed_twice`
//    at the same time with other tests that initialize a stream for both input
//    and output(this will create an aggregate device and fire
//    `audiounit_collection_changed_callback` indirectly, see the comment in
//    `audiounit_create_blank_aggregate_device` and `test_stream_set_panning`).
//
//    A simple way to verify this is to add a log at the beginning
//    `audiounit_collection_changed_callback` and a log in
//    `AudioUnitContext::drop`. You will get this error when
//    `audiounit_collection_changed_callback` is called after the
//    AudioUnitContext is dropped.
//
// 2. If those tests calling `audiounit_add_device_listener` is finished
//    between the time after those asynchronous functions are executed but
//    before those asynchronous functions are finished, those tests will try
//    destroying the contexts that are currently locked by those asynchronous
//    functions. Thus, we will get panics in
//    `OwnedCriticalSection::drop/destroy` since `pthread_mutex_destroy`
//    returns `EBUSY(16)` rather than 0.
//
//    Theoretically, this could happen when the operations are executed in the
//    following order:
//    1. Create an AudioUnitContext `ctx`
//    2. Register device-collection changed for `ctx`
//    3. Initialize an AudioUnitStream `stm` within `ctx` for both input and
//       output. It will create an aggregate device and fire the
//       `audiounit_collection_changed_callback` indirectly.
//       In the `audiounit_collection_changed_callback`, it will dispatch an
//       asynchronous task that will lock the `ctx`
//    4. The asynchronous task starts runnning and lock the `ctx`
//    5. `ctx` is destroyed while the asynchronous task is running, before the
//       asynchronous task is finished, we will get a fail for destroying a
//       locked `ctx`
//
//    A simple way to verify this is to add two logs at the beginning and the
//    end of `async_dispatch` in `audiounit_collection_changed_callback` and
//    two logs at the beginning and the end of the tests calling
//    `audiounit_add_device_listener`. You will find those tests fail when the
//    tests are ended while those asynchronous functions are still running.
#[test]
#[ignore]
fn test_create_blank_aggregate_device() {
    // TODO: Test this when there is no available devices.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(
            &mut plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
    assert_ne!(
        plugin_id,
        kAudioObjectUnknown
    );
    assert_ne!(
        aggregate_device_id,
        kAudioObjectUnknown
    );

    let all_devices = get_all_devices();
    assert!(!all_devices.is_empty());
    assert!(all_devices.contains(&aggregate_device_id));

    let all_devices_names = to_devices_names(&all_devices);
    assert!(!all_devices_names.is_empty());
    let mut aggregate_device_found = false;
    for name_opt in all_devices_names {
        if let Some(name) = name_opt {
            if name.contains(PRIVATE_AGGREGATE_DEVICE_NAME) {
                aggregate_device_found = true;
                break;
            }
        }
    }
    assert!(aggregate_device_found);

    assert!(
        audiounit_destroy_aggregate_device(
            plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );

    fn get_all_devices() -> Vec<AudioObjectID> {
        let mut size: usize = 0;
        let mut ret = audio_object_get_property_data_size(
            kAudioObjectSystemObject,
            &DEVICES_PROPERTY_ADDRESS,
            &mut size
        );
        if ret != NO_ERR {
            return Vec::new();
        }
        /* Total number of input and output devices. */
        let mut devices: Vec<AudioObjectID> = allocate_array_by_size(size);
        ret = audio_object_get_property_data(
            kAudioObjectSystemObject,
            &DEVICES_PROPERTY_ADDRESS,
            &mut size,
            devices.as_mut_ptr()
        );
        if ret != NO_ERR {
            return Vec::new();
        }
        devices.sort();
        devices
    }
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

// set_aggregate_sub_device_list
// ------------------------------------
#[test]
fn test_set_aggregate_sub_device_list_for_a_unknown_aggregate_device() {
    // If aggregate device id is kAudioObjectUnknown, we won't be able to
    // set device list.
    assert_eq!(
        audiounit_set_aggregate_sub_device_list(
            kAudioObjectUnknown,
            kAudioObjectUnknown,
            kAudioObjectUnknown
        ).unwrap_err(),
        Error::error()
    );

    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);

    if !valid_id(input_id) || !valid_id(output_id) /* || input_id == output_id */ {
        return;
    }

    assert_eq!(
        audiounit_set_aggregate_sub_device_list(
            kAudioObjectUnknown,
            input_id,
            output_id
        ).unwrap_err(),
        Error::error()
    );
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_set_aggregate_sub_device_list_for_unknown_input_output_devices() {
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(
            &mut plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // NOTE: We will get errors and pass the test here since get_device_name()
    //       return a NULL CFStringRef for a unknown devicie. Instead of
    //       replying on get_device_name(). We should check this in the
    //       beginning of the audiounit_set_aggregate_sub_device_list().

    // Both input and output are unknown.
    assert_eq!(
        audiounit_set_aggregate_sub_device_list(
            aggregate_device_id,
            kAudioObjectUnknown,
            kAudioObjectUnknown
        ).unwrap_err(),
        Error::error()
    );

    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);

    // Only input is unknown.
    if valid_id(output_id) {
        assert_eq!(
            audiounit_set_aggregate_sub_device_list(
                aggregate_device_id,
                kAudioObjectUnknown,
                output_id
            ).unwrap_err(),
            Error::error()
        );
    }

    // Only output is unknown.
    if valid_id(input_id) {
        assert_eq!(
            audiounit_set_aggregate_sub_device_list(
                aggregate_device_id,
                input_id,
                kAudioObjectUnknown
            ).unwrap_err(),
            Error::error()
        );
    }

    assert!(
        audiounit_destroy_aggregate_device(
            plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_set_aggregate_sub_device_list() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(input_id) || !valid_id(output_id) /* || input_id == output_id */ {
        return;
    }

    let input_sub_devices = audiounit_get_sub_devices(input_id);
    let output_sub_devices = audiounit_get_sub_devices(output_id);

    // Create a blank aggregate device.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(
            &mut plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Set sub devices for the created aggregate device.
    assert!(
        audiounit_set_aggregate_sub_device_list(
            aggregate_device_id,
            input_id,
            output_id
        ).is_ok()
    );
    let sub_devices = audiounit_get_sub_devices(aggregate_device_id);

    assert!(sub_devices.len() <= input_sub_devices.len() + output_sub_devices.len());

    // Make sure all the sub devices of the default input and output devices
    // are also the sub devices of the aggregate device.
    for device in &input_sub_devices {
        assert!(sub_devices.contains(device));
    }

    for device in &output_sub_devices {
        assert!(sub_devices.contains(device));
    }

    let onwed_devices = get_onwed_devices(aggregate_device_id);
    assert!(!onwed_devices.is_empty());
    let owned_devices_names = to_devices_names(&onwed_devices);
    show_devices_names("aggregate owning devices", &owned_devices_names);

    let input_sub_devices_names = to_devices_names(&input_sub_devices);
    show_devices_names("input sub devices", &owned_devices_names);

    let output_sub_devices_names = to_devices_names(&output_sub_devices);
    show_devices_names("output sub devices", &owned_devices_names);

    for name_opt in &input_sub_devices_names {
        assert!(owned_devices_names.contains(name_opt));
    }

    for name_opt in &output_sub_devices_names {
        assert!(owned_devices_names.contains(name_opt));
    }

    assert!(
        audiounit_destroy_aggregate_device(
            plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );

    fn show_devices_names(title: &'static str, names: &Vec<Option<String>>) {
        println!("\n{}\n-----------", title);
        for name_opt in names {
            if let Some(name) = name_opt {
                println!("{}", name);
            }
        }
        println!();
    }
}

// set_master_aggregate_device
// ------------------------------------
#[test]
#[should_panic]
fn test_set_master_aggregate_device_for_a_unknown_aggregate_device() {
    assert_eq!(
        audiounit_set_master_aggregate_device(
            kAudioObjectUnknown
        ).unwrap_err(),
        Error::error()
    );
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_set_master_aggregate_device_for_a_blank_aggregate_device() {
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(
            &mut plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // TODO: If there is no available device, we will set master device
    //       to a device whose name is a NULL CFStringRef (see implementation)
    //       but surprisingly it's ok! On the other hand, it's also ok to set
    //       the default ouput device(if any) for a blank aggregate device.
    //       That is, it's ok to set the default ouput device to an aggregate
    //       device whose sub devices list doesn't include default ouput device!
    //       This is weird to me. Maybe we should return errors when above
    //       conditions are met.
    assert!(
        audiounit_set_master_aggregate_device(
            aggregate_device_id
        ).is_ok()
    );

    // Make sure this blank aggregate device owns nothing.
    // TODO: it's really weird it actually own nothing but
    //       it can set master device successfully!
    let owned_sub_devices = get_onwed_devices(aggregate_device_id);
    assert!(owned_sub_devices.is_empty());

    // Check if master is nothing.
    let master_device = get_master_device(aggregate_device_id);
    assert!(master_device.is_empty());

    assert!(
        audiounit_destroy_aggregate_device(
            plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_set_master_aggregate_device() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(input_id) || !valid_id(output_id) /* || input_id == output_id */ {
        return;
    }

    let output_sub_devices = audiounit_get_sub_devices(output_id);
    if output_sub_devices.is_empty() {
        return;
    }

    // Create a blank aggregate device.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(
            &mut plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Set the sub devices into the created aggregate device.
    assert!(
        audiounit_set_aggregate_sub_device_list(
            aggregate_device_id,
            input_id,
            output_id
        ).is_ok()
    );

    // Set the master device.
    assert!(
        audiounit_set_master_aggregate_device(
            aggregate_device_id
        ).is_ok()
    );

    // Check if master is set to default output device.
    let master_device = get_master_device(aggregate_device_id);
    let default_output_device = to_device_name(output_id).unwrap();
    assert_eq!(
        master_device,
        default_output_device
    );

    // Check the first owning device is the default output device.
    let onwed_devices = get_onwed_devices(aggregate_device_id);
    assert!(!onwed_devices.is_empty());
    let mut first_output_device = None;
    for device in &onwed_devices {
        if is_output(*device) {
            first_output_device = Some(*device);
        }
    }
    assert!(first_output_device.is_some());
    // TODO: Does this check work if output_id is an aggregate device ?
    assert_eq!(
        to_device_name(first_output_device.unwrap()),
        to_device_name(output_id)
    );

    // Destroy the aggregate device.
    assert!(
        audiounit_destroy_aggregate_device(
            plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
}

fn get_master_device(aggregate_device_id: AudioObjectID) -> String {
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    let master_aggregate_sub_device = AudioObjectPropertyAddress {
        mSelector: kAudioAggregateDevicePropertyMasterSubDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster
    };

    let mut master_sub_device: CFStringRef = ptr::null_mut();
    let mut size = mem::size_of::<CFStringRef>();
    assert_eq!(
        audio_object_get_property_data(
            aggregate_device_id,
            &master_aggregate_sub_device,
            &mut size,
            &mut master_sub_device
        ),
        NO_ERR
    );
    assert!(!master_sub_device.is_null());

    let master_device = strref_to_string(master_sub_device);

    unsafe {
        CFRelease(master_sub_device as *const c_void);
    }

    master_device
}

// activate_clock_drift_compensation
// ------------------------------------
#[test]
#[should_panic]
fn test_activate_clock_drift_compensation_for_a_unknown_aggregate_device() {
    assert_eq!(
        audiounit_activate_clock_drift_compensation(
            kAudioObjectUnknown
        ).unwrap_err(),
        Error::error()
    );
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[should_panic]
#[ignore]
fn test_activate_clock_drift_compensation_for_a_blank_aggregate_device() {
    // Create a blank aggregate device.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(
            &mut plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Get owned sub devices.
    let devices = get_onwed_devices(aggregate_device_id);
    assert!(devices.is_empty());

    // Get a panic since no sub devices to be set compensation.
    assert_eq!(
        audiounit_activate_clock_drift_compensation(
            aggregate_device_id
        ).unwrap_err(),
        Error::error()
    );

    // Destroy the aggregate device. (The program cannot reach here.)
    assert!(
        audiounit_destroy_aggregate_device(
            plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_activate_clock_drift_compensation_for_an_aggregate_device_without_master_device() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(input_id) || !valid_id(output_id) /* || input_id == output_id */ {
        return;
    }

    // Create a blank aggregate device.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(
            &mut plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Set the sub devices into the created aggregate device.
    assert!(
        audiounit_set_aggregate_sub_device_list(
            aggregate_device_id,
            input_id,
            output_id
        ).is_ok()
    );

    // TODO: Is the master device the first output sub device by default if we
    //       don't set that ? Is it because we add the output sub device list
    //       before the input's one ? (See implementation of
    //       audiounit_set_aggregate_sub_device_list).
    // TODO: Does this check work if output_id is an aggregate device ?
    assert_eq!(
        get_master_device(aggregate_device_id),
        to_device_name(output_id).unwrap()
    );

    // Set clock drift compensation.
    assert!(
        audiounit_activate_clock_drift_compensation(
            aggregate_device_id
        ).is_ok()
    );

    // Check the compensations.
    let devices = get_onwed_devices(aggregate_device_id);
    assert!(!devices.is_empty());
    let compensations = get_drift_compensations(&devices);
    assert!(!compensations.is_empty());
    assert_eq!(
        devices.len(),
        compensations.len()
    );

    for (i, compensation) in compensations.iter().enumerate() {
        assert_eq!(
            *compensation,
            if i == 0 {
                0
            } else {
                1
            }
        );
    }

    // Destroy the aggregate device.
    assert!(
        audiounit_destroy_aggregate_device(
            plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_activate_clock_drift_compensation() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(input_id) || !valid_id(output_id) /* || input_id == output_id */ {
        return;
    }

    let output_sub_devices = audiounit_get_sub_devices(output_id);
    if output_sub_devices.is_empty() {
        return;
    }

    // Create a blank aggregate device.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(
            &mut plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Set the sub devices into the created aggregate device.
    assert!(
        audiounit_set_aggregate_sub_device_list(
            aggregate_device_id,
            input_id,
            output_id
        ).is_ok()
    );

    // Set the master device.
    assert!(
        audiounit_set_master_aggregate_device(
            aggregate_device_id
        ).is_ok()
    );

    // Set clock drift compensation.
    assert!(
        audiounit_activate_clock_drift_compensation(
            aggregate_device_id
        ).is_ok()
    );

    // Check the compensations.
    let devices = get_onwed_devices(aggregate_device_id);
    assert!(!devices.is_empty());
    let compensations = get_drift_compensations(&devices);
    assert!(!compensations.is_empty());
    assert_eq!(
        devices.len(),
        compensations.len()
    );

    for (i, compensation) in compensations.iter().enumerate() {
        assert_eq!(
            *compensation,
            if i == 0 {
                0
            } else {
                1
            }
        );
    }

    // Destroy the aggregate device.
    assert!(
        audiounit_destroy_aggregate_device(
            plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
}

fn get_onwed_devices(
    aggregate_device_id: AudioDeviceID
) -> Vec<AudioObjectID> {
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    let address_owned = AudioObjectPropertyAddress {
        mSelector: kAudioObjectPropertyOwnedObjects,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster
    };

    let qualifier_data_size = mem::size_of::<AudioObjectID>();
    let class_id: AudioClassID = kAudioSubDeviceClassID;
    let qualifier_data = &class_id;
    let mut size: usize = 0;

    unsafe {
        assert_eq!(
            AudioObjectGetPropertyDataSize(
                aggregate_device_id,
                &address_owned,
                qualifier_data_size as u32,
                qualifier_data as *const u32 as *const c_void,
                &mut size as *mut usize as *mut u32
            ),
            NO_ERR
        );
    }

    // assert_ne!(size, 0);
    if size == 0 {
        return Vec::new();
    }

    let elements = size / mem::size_of::<AudioObjectID>();
    let mut devices: Vec<AudioObjectID> = allocate_array(elements);

    unsafe {
        assert_eq!(
            AudioObjectGetPropertyData(
                aggregate_device_id,
                &address_owned,
                qualifier_data_size as u32,
                qualifier_data as *const u32 as *const c_void,
                &mut size as *mut usize as *mut u32,
                devices.as_mut_ptr() as *mut c_void
            ),
            NO_ERR
        );
    }

    devices
}

fn get_drift_compensations(
    devices: &Vec<AudioObjectID>
) -> Vec<u32> {
    assert!(!devices.is_empty());

    let address_drift = AudioObjectPropertyAddress {
        mSelector: kAudioSubDevicePropertyDriftCompensation,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster
    };

    let mut compensations = Vec::new();

    for device in devices {
        assert_ne!(*device, kAudioObjectUnknown);

        let mut size = mem::size_of::<u32>();
        let mut compensation = u32::max_value();

        assert_eq!(
            audio_object_get_property_data(
                *device,
                &address_drift,
                &mut size,
                &mut compensation
            ),
            NO_ERR
        );

        compensations.push(compensation);
    }

    compensations
}

// workaround_for_airpod
// ------------------------------------
// TODO

// create_aggregate_device
// ------------------------------------
// TODO

// destroy_aggregate_device
// ------------------------------------
#[test]
#[should_panic]
fn test_destroy_aggregate_device_for_unknown_plugin_and_aggregate_devices() {
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert_eq!(
        audiounit_destroy_aggregate_device(
            kAudioObjectUnknown,
            &mut aggregate_device_id
        ).unwrap_err(),
        Error::error()
    )
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
#[should_panic]
fn test_destroy_aggregate_device_for_a_unknown_plugin_device() {
    // TODO: Test this when there is no available devices.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(
            &mut plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
    assert_ne!(
        plugin_id,
        kAudioObjectUnknown
    );
    assert_ne!(
        aggregate_device_id,
        kAudioObjectUnknown
    );

    assert_eq!(
        audiounit_destroy_aggregate_device(
            kAudioObjectUnknown,
            &mut aggregate_device_id
        ).unwrap_err(),
        Error::error()
    );
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
#[should_panic]
fn test_destroy_aggregate_device_for_a_unknown_aggregate_device() {
    // TODO: Test this when there is no available devices.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(
            &mut plugin_id,
            &mut aggregate_device_id
        ).is_ok()
    );
    assert_ne!(
        plugin_id,
        kAudioObjectUnknown
    );
    assert_ne!(
        aggregate_device_id,
        kAudioObjectUnknown
    );

    aggregate_device_id = kAudioObjectUnknown;

    assert_eq!(
        audiounit_destroy_aggregate_device(
            plugin_id,
            &mut aggregate_device_id
        ).unwrap_err(),
        Error::error()
    );
}

// #[test]
// fn test_destroy_aggregate_device() {
// }

// Other tests for audiounit_destroy_aggregate_devic are combined with
// other tests that call audiounit_create_blank_aggregate_device:
// - test_get_sub_devices_for_blank_aggregate_devices
// - test_create_blank_aggregate_device
// - test_set_aggregate_sub_device_list_for_unknown_input_output_devices
// - test_set_aggregate_sub_device_list
// - test_set_master_aggregate_device_for_a_blank_aggregate_device
// - test_set_master_aggregate_device
// - test_activate_clock_drift_compensation_for_an_aggregate_device_without_master_device
// - test_activate_clock_drift_compensation

// new_unit_instance
// ------------------------------------
#[test]
fn test_new_unit_instance() {
    let flags_list = [
        device_flags::DEV_UNKNOWN,
        device_flags::DEV_INPUT,
        device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
    ];

    for flags in flags_list.iter() {
        let device = device_info {
            id: kAudioObjectUnknown,
            flags: *flags
        };
        let mut unit: AudioUnit = ptr::null_mut();
        assert!(audiounit_new_unit_instance(&mut unit, &device).is_ok());
        assert!(!unit.is_null());
        // Destroy the AudioUnits
        unsafe {
            AudioUnitUninitialize(unit);
            AudioComponentInstanceDispose(unit);
        }
    }
}

#[test]
#[should_panic]
fn test_new_unit_instance_twice() {
    let device = device_info::new();
    let mut unit: AudioUnit = ptr::null_mut();
    assert!(audiounit_new_unit_instance(&mut unit, &device).is_ok());
    assert!(!unit.is_null());

    // audiounit_new_unit_instance will get a panic immediately
    // when it's called, so the `assert_eq` and the code after
    // that won't be executed.
    assert_eq!(
        audiounit_new_unit_instance(&mut unit, &device).unwrap_err(),
        Error::error()
    );

    // Destroy the AudioUnits
    unsafe {
        AudioUnitUninitialize(unit);
        AudioComponentInstanceDispose(unit);
    }
}

// enable_unit_scope
// ------------------------------------
#[test]
#[should_panic]
fn test_enable_unit_scope_with_null_unit() {
    let unit: AudioUnit = ptr::null_mut();

    // audiounit_enable_unit_scope will get a panic immediately
    // when it's called, so the `assert_eq` and the code after
    // that won't be executed.
    assert_eq!(
        audiounit_enable_unit_scope(
            &unit,
            io_side::INPUT,
            enable_state::DISABLE
        ).unwrap_err(),
        Error::error()
    );

    assert_eq!(
        audiounit_enable_unit_scope(
            &unit,
            io_side::INPUT,
            enable_state::ENABLE
        ).unwrap_err(),
        Error::error()
    );

    assert_eq!(
        audiounit_enable_unit_scope(
            &unit,
            io_side::OUTPUT,
            enable_state::DISABLE
        ).unwrap_err(),
        Error::error()
    );

    assert_eq!(
        audiounit_enable_unit_scope(
            &unit,
            io_side::OUTPUT,
            enable_state::ENABLE
        ).unwrap_err(),
        Error::error()
    );
}

#[test]
fn test_enable_unit_output_scope_for_default_output_unit() {
    // For those units whose subtype is kAudioUnitSubType_DefaultOutput,
    // their input or output scopes cannot be enabled or disabled.

    let devices = [
        device_info {
            id: kAudioObjectUnknown,
            flags: device_flags::DEV_OUTPUT |
                   device_flags::DEV_SYSTEM_DEFAULT
        },
        device_info {
            id: kAudioObjectUnknown,
            flags: device_flags::DEV_INPUT |
                   device_flags::DEV_OUTPUT |
                   device_flags::DEV_SYSTEM_DEFAULT
        },
    ];

    for device in devices.iter() {
        let mut unit: AudioUnit = ptr::null_mut();
        assert!(audiounit_new_unit_instance(&mut unit, &device).is_ok());
        assert!(!unit.is_null());

        let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
        if valid_id(output_id) {
            // Check if the output scope is enabled.
            assert!(unit_scope_is_enabled(unit, false));

            // The input scope is enabled if it's also a input device.
            // Otherwise, it's disabled.
            if is_input(output_id) {
                assert!(unit_scope_is_enabled(unit, true));
            } else {
                assert!(!unit_scope_is_enabled(unit, true));
            }
        }

        assert_eq!(
            audiounit_enable_unit_scope(
                &unit,
                io_side::OUTPUT,
                enable_state::ENABLE
            ).unwrap_err(),
            Error::error()
        );

        assert_eq!(
            audiounit_enable_unit_scope(
                &unit,
                io_side::OUTPUT,
                enable_state::DISABLE
            ).unwrap_err(),
            Error::error()
        );

        assert_eq!(
            audiounit_enable_unit_scope(
                &unit,
                io_side::INPUT,
                enable_state::ENABLE
            ).unwrap_err(),
            Error::error()
        );

        assert_eq!(
            audiounit_enable_unit_scope(
                &unit,
                io_side::INPUT,
                enable_state::DISABLE
            ).unwrap_err(),
            Error::error()
        );

        // Destroy the AudioUnits
        unsafe {
            AudioUnitUninitialize(unit);
            AudioComponentInstanceDispose(unit);
        }
    }
}

#[test]
fn test_enable_unit_scope() {
    // It's ok to enable and disable the scopes of input or output
    // for those units whose subtype are kAudioUnitSubType_HALOutput
    // even when there is no available input or output devices.

    let flags_list = [
        device_flags::DEV_UNKNOWN,
        device_flags::DEV_INPUT,
        device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT,
    ];

    for flags in flags_list.iter() {
        let device = device_info {
            id: kAudioObjectUnknown,
            flags: *flags
        };
        let mut unit: AudioUnit = ptr::null_mut();
        assert!(audiounit_new_unit_instance(&mut unit, &device).is_ok());
        assert!(!unit.is_null());

        assert!(
            audiounit_enable_unit_scope(
                &unit,
                io_side::OUTPUT,
                enable_state::ENABLE
            ).is_ok()
        );

        assert!(
            audiounit_enable_unit_scope(
                &unit,
                io_side::OUTPUT,
                enable_state::DISABLE
            ).is_ok()
        );

        assert!(
            audiounit_enable_unit_scope(
                &unit,
                io_side::INPUT,
                enable_state::ENABLE
            ).is_ok()
        );

        assert!(
            audiounit_enable_unit_scope(
                &unit,
                io_side::INPUT,
                enable_state::DISABLE
            ).is_ok()
        );

        // Destroy the AudioUnits
        unsafe {
            AudioUnitUninitialize(unit);
            AudioComponentInstanceDispose(unit);
        }
    }
}

// create_unit
// ------------------------------------
#[test]
#[should_panic]
fn test_create_unit_with_unknown_scope() {
    let device = device_info::new();
    let mut unit: AudioUnit = ptr::null_mut();
    assert!(audiounit_create_unit(&mut unit, &device).is_ok());
    assert!(!unit.is_null());
}

#[test]
#[should_panic]
fn test_create_unit_twice() {
    let flags_list = [
        device_flags::DEV_INPUT,
        device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
    ];

    // The first audiounit_create_unit calling will get a panic immediately
    // so the loop is executed once.
    for flags in flags_list.iter() {
        let mut device = device_info::new();
        device.flags |= *flags;
        let mut unit: AudioUnit = ptr::null_mut();
        assert!(audiounit_create_unit(&mut unit, &device).is_ok());
        assert!(!unit.is_null());
        assert_eq!(
            audiounit_create_unit(&mut unit, &device).unwrap_err(),
            Error::error()
        );
    }
}

#[test]
fn test_create_unit() {
    let flags_list = [
        device_flags::DEV_INPUT,
        device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
    ];

    // The first audiounit_create_unit calling will get a panic immediately
    // so the loop is executed once.
    for flags in flags_list.iter() {
        let mut device = device_info::new();
        device.flags |= *flags;

        // Check the output scope is enabled.
        if device.flags.contains(device_flags::DEV_OUTPUT) {
            let device_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
            if valid_id(device_id) {
                device.id = device_id;
                let mut unit: AudioUnit = ptr::null_mut();
                assert!(audiounit_create_unit(&mut unit, &device).is_ok());
                assert!(!unit.is_null());
                assert!(unit_scope_is_enabled(unit, false));

                // For default output device, the input scope is enabled
                // if it's also a input device. Otherwise, it's disabled.
                if device.flags.contains(device_flags::DEV_INPUT |
                                         device_flags::DEV_SYSTEM_DEFAULT) {
                    if is_input(device_id) {
                        assert!(unit_scope_is_enabled(unit, true));
                    } else {
                        assert!(!unit_scope_is_enabled(unit, true));
                    }

                    // Destroy the audioUnit.
                    unsafe {
                        AudioUnitUninitialize(unit);
                        AudioComponentInstanceDispose(unit);
                    }
                    continue;
                }

                // Destroy the audioUnit.
                unsafe {
                    AudioUnitUninitialize(unit);
                    AudioComponentInstanceDispose(unit);
                }
            }
        }

        // Check the input scope is enabled.
        if device.flags.contains(device_flags::DEV_INPUT) {
            let device_id = audiounit_get_default_device_id(DeviceType::INPUT);
            if valid_id(device_id) {
                device.id = device_id;
                let mut unit: AudioUnit = ptr::null_mut();
                assert!(audiounit_create_unit(&mut unit, &device).is_ok());
                assert!(!unit.is_null());
                assert!(unit_scope_is_enabled(unit, true));
                // Destroy the audioUnit.
                unsafe {
                    AudioUnitUninitialize(unit);
                    AudioComponentInstanceDispose(unit);
                }
            }
        }
    }
}

// init_input_linear_buffer
// ------------------------------------
// FIXIT: We should get a panic! The type should be unknown before the audio
//        description is set!
#[test]
#[should_panic]
#[ignore]
fn test_init_input_linear_buffer_without_valid_audiodescription() {
    // Create a stream.
    // ------------------------------------------------------------------------
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert!(stream.input_linear_buffer.is_none());

    assert!(audiounit_init_input_linear_buffer(&mut stream, 0).is_ok());
}

// TODO: Should we get a panic ?
#[test]
fn test_init_input_linear_buffer_without_setting_latency() {
    // Create a stream.
    // ------------------------------------------------------------------------
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert!(stream.input_linear_buffer.is_none());

    // // Set latency
    // // ------------------------------------------------------------------------
    // {
    //     // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
    //     let ctx_mutex_ptr = &mut stream.context.mutex as *mut OwnedCriticalSection;
    //     // The scope of `ctx_lock` is a critical section.
    //     let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
    //     assert_eq!(stream.latency_frames, 0);
    //     stream.latency_frames = audiounit_clamp_latency(&mut stream, 0);
    //     assert_ne!(stream.latency_frames, 0);
    // }

    // Set audio input description according to the input params.
    // ------------------------------------------------------------------------
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 48_000;
    raw.channels = 2;
    raw.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;

    let params = StreamParams::from(raw);

    assert_eq!(stream.input_desc.mFormatFlags, 0);
    assert_eq!(stream.input_desc.mChannelsPerFrame, 0);

    assert!(
        audio_stream_desc_init(
            &mut stream.input_desc,
            &params
        ).is_ok()
    );

    assert_ne!(stream.input_desc.mFormatFlags & kAudioFormatFlagIsFloat, 0);
    assert_eq!(stream.input_desc.mChannelsPerFrame, 2);

    // Set input_linear_buffer
    // ------------------------------------------------------------------------
    assert!(audiounit_init_input_linear_buffer(&mut stream, 1).is_ok());
    assert!(stream.input_linear_buffer.is_some());

    let buf_f32 = [1.0_f32, 2.1, 3.2];
    stream.input_linear_buffer.as_mut().unwrap().push(
        buf_f32.as_ptr() as *const c_void,
        buf_f32.len()
    );
}

fn test_init_input_linear_buffer_impl<T: std::any::Any>(array: &[T]) {
    const CHANNEL: u32 = 2;
    const BUF_CAPACITY: u32 = 1;

    // Get format parameters for the type.
    // ------------------------------------------------------------------------
    let type_id = std::any::TypeId::of::<T>();
    let (format, format_flag) = if type_id == std::any::TypeId::of::<f32>() {
        (ffi::CUBEB_SAMPLE_FLOAT32NE, kAudioFormatFlagIsFloat)
    } else if type_id == std::any::TypeId::of::<i16>() {
        (ffi::CUBEB_SAMPLE_S16NE, kAudioFormatFlagIsSignedInteger)
    } else {
        panic!("Unsupported type!");
    };

    // Create a stream.
    // ------------------------------------------------------------------------
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert!(stream.input_linear_buffer.is_none());

    // Set latency.
    // ------------------------------------------------------------------------
    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut stream.context.mutex as *mut OwnedCriticalSection;
        // The scope of `ctx_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        assert_eq!(stream.latency_frames, 0);
        stream.latency_frames = audiounit_clamp_latency(&mut stream, 0);
        assert_ne!(stream.latency_frames, 0);
    }

    // Set audio input description according to the input params.
    // ------------------------------------------------------------------------
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = format;
    raw.rate = 48_000;
    raw.channels = CHANNEL;
    raw.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;

    let params = StreamParams::from(raw);

    assert_eq!(stream.input_desc.mFormatFlags, 0);
    assert_eq!(stream.input_desc.mChannelsPerFrame, 0);

    assert!(
        audio_stream_desc_init(
            &mut stream.input_desc,
            &params
        ).is_ok()
    );

    assert_ne!(stream.input_desc.mFormatFlags & format_flag, 0);
    assert_eq!(stream.input_desc.mChannelsPerFrame, CHANNEL);

    // Set input_linear_buffer
    // ------------------------------------------------------------------------
    assert!(audiounit_init_input_linear_buffer(&mut stream, BUF_CAPACITY).is_ok());
    assert!(stream.input_linear_buffer.is_some());

    stream.input_linear_buffer.as_mut().unwrap().push(
        array.as_ptr() as *const c_void,
        array.len()
    );
}

#[test]
fn test_init_input_linear_buffer() {
    test_init_input_linear_buffer_impl(&[3.1_f32, 4.1, 5.9, 2.6, 5.35]);
    test_init_input_linear_buffer_impl(&[13_i16, 21, 34, 55, 89, 144]);
}

// clamp_latency
// ------------------------------------
// TODO: Add a test to test the behavior of clamp_latency without any
//       active stream.
//       We are unable to test it right now. If we add a test that should get
//       a panic when hitting the assertion in audiounit_clamp_latency since
//       there is no active stream, then we will get another panic when
//       AudioUnitStream::drop/destroy is called. AudioUnitStream::drop/destroy
//       will check we have at least one active stream when destroying
//       AudioUnitStream. Maybe we can add this test after refactoring.
//       Simply add a note here for now.

#[test]
fn test_clamp_latency_with_one_active_stream() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
    let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    // The scope of `_lock` is a critical section.
    // When `AudioUnitStream::drop()` is called, `AudioUnitContext.mutex`
    // needs to be unlocked. That's why `_lock` needs to be declared after
    // `stream` so it will be dropped and unlocked before dropping `stream`.
    let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });

    // TODO: It works even when there is no output unit(AudioUnit).
    //       Should we throw an error or panic in this case ?

    let range = 0..2 * SAFE_MAX_LATENCY_FRAMES;
    assert!(range.start < SAFE_MIN_LATENCY_FRAMES);
    // assert!(range.end < SAFE_MAX_LATENCY_FRAMES);
    for latency in range {
        let clamp = audiounit_clamp_latency(&mut stream, latency);
        assert_eq!(
            clamp,
            if latency < SAFE_MIN_LATENCY_FRAMES {
                SAFE_MIN_LATENCY_FRAMES
            } else if latency > SAFE_MAX_LATENCY_FRAMES {
                SAFE_MAX_LATENCY_FRAMES
            } else {
                latency
            }
        );
    }
}

#[test]
#[should_panic]
fn test_clamp_latency_with_more_than_one_active_streams_without_output_unit() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
    let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;

    // Add two streams to the context.
    // `AudioUnitStream::drop()` will check the context has at least one stream.
    {
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    // The scope of `_lock` is a critical section.
    // When `AudioUnitStream::drop()` is called, `AudioUnitContext.mutex`
    // needs to be unlocked. That's why `_lock` needs to be declared after
    // `stream` so it will be dropped and unlocked before dropping `stream`.
    let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });

    // TODO: We only check this when we have more than one streams.
    //       Should we also check this when we have only one stream ?
    // Get a panic since we don't have valid output AudioUnit.
    let _ = audiounit_clamp_latency(&mut stream, 0);
}

#[test]
fn test_clamp_latency_with_more_than_one_active_streams() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
    let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;

    // Add two streams to the context.
    // `AudioUnitStream::drop()` will check the context has at least one stream.
    {
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    // The scope of `_lock` is a critical section.
    // When `AudioUnitStream::drop()` is called, `AudioUnitContext.mutex`
    // needs to be unlocked. That's why `_lock` needs to be declared after
    // `stream` so it will be dropped and unlocked before dropping `stream`.
    let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });

    // Initialize the output unit to default output device.
    let device = device_info {
        id: kAudioObjectUnknown,
        flags: device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT
    };
    assert!(audiounit_create_unit(&mut stream.output_unit, &device).is_ok());
    assert!(!stream.output_unit.is_null());
    let maybe_buffer_size = {
        let mut buffer_size: u32 = 0;
        if audio_unit_get_property(
            stream.output_unit,
            kAudioDevicePropertyBufferFrameSize,
            kAudioUnitScope_Output,
            AU_OUT_BUS,
            &mut buffer_size,
            &mut mem::size_of_val(&buffer_size)
        ) == 0 {
            Some(buffer_size)
        } else {
            None
        }
    };

    let range = 0..2 * SAFE_MAX_LATENCY_FRAMES;
    assert!(range.start < SAFE_MIN_LATENCY_FRAMES);
    // assert!(range.end < SAFE_MAX_LATENCY_FRAMES);
    for latency in range {
        let clamp = audiounit_clamp_latency(&mut stream, latency);
        assert_eq!(
            clamp,
            clamp_values(
                if let Some(buffer_size) = maybe_buffer_size {
                    cmp::min(buffer_size, latency)
                } else {
                    latency
                }
            )
        );
    }

    fn clamp_values(value: u32) -> u32 {
        cmp::max(cmp::min(value, SAFE_MAX_LATENCY_FRAMES),
                 SAFE_MIN_LATENCY_FRAMES)
    }
}

// set_buffer_size
// ------------------------------------
#[test]
#[should_panic]
fn test_set_buffer_size_for_input_with_null_input_unit()
{
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert!(stream.input_unit.is_null());

    assert_eq!(
        audiounit_set_buffer_size(
            &mut stream,
            2048,
            io_side::INPUT
        ).unwrap_err(),
        Error::error()
    );
}

#[test]
fn test_set_buffer_size_for_input()
{
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 48_000;
    raw.channels = 1;
    raw.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.input_stream_params = StreamParams::from(raw);

    // It's crucial to call to audiounit_set_device_info to set
    // stream.input_device, or we will hit the
    // assertion in audiounit_create_unit.

    let default_input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    if !valid_id(default_input_id) {
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::INPUT
        ).is_ok()
    );

    assert_eq!(stream.input_device.id, default_input_id);
    assert_eq!(
        stream.input_device.flags,
        device_flags::DEV_INPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(
        audiounit_create_unit(
            &mut stream.input_unit,
            &stream.input_device
        ).is_ok()
    );

    assert!(!stream.input_unit.is_null());

    let mut buffer_frames: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            stream.input_unit,
            kAudioDevicePropertyBufferFrameSize,
            kAudioUnitScope_Output,
            AU_IN_BUS,
            &mut buffer_frames,
            &mut size
        ),
        0
    );

    assert_ne!(buffer_frames, 0);
    buffer_frames *= 2;
    assert!(
        audiounit_set_buffer_size(
            &mut stream,
            buffer_frames,
            io_side::INPUT
        ).is_ok()
    );
}

#[test]
#[should_panic]
fn test_set_buffer_size_for_output_with_null_output_unit()
{
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert!(stream.output_unit.is_null());

    assert_eq!(
        audiounit_set_buffer_size(
            &mut stream,
            2048,
            io_side::OUTPUT
        ).unwrap_err(),
        Error::error()
    );
}

#[test]
fn test_set_buffer_size_for_output()
{
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 44_100;
    raw.channels = 2;
    raw.layout = ffi::CUBEB_LAYOUT_STEREO;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // It's crucial to call to audiounit_set_device_info to set
    // stream.input_device, or we will hit the
    // assertion in audiounit_create_unit.

    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(default_output_id) {
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::OUTPUT
        ).is_ok()
    );

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(
        audiounit_create_unit(
            &mut stream.output_unit,
            &stream.output_device
        ).is_ok()
    );

    assert!(!stream.output_unit.is_null());

    let mut buffer_frames: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            stream.output_unit,
            kAudioDevicePropertyBufferFrameSize,
            kAudioUnitScope_Input,
            AU_OUT_BUS,
            &mut buffer_frames,
            &mut size
        ),
        0
    );

    assert_ne!(buffer_frames, 0);
    buffer_frames *= 2;
    assert!(
        audiounit_set_buffer_size(
            &mut stream,
            buffer_frames,
            io_side::OUTPUT
        ).is_ok()
    );
}

// configure_input
// ------------------------------------
#[test]
#[should_panic]
fn test_configure_input_with_null_unit() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert!(stream.input_unit.is_null());
    assert!{
        audiounit_configure_input(
            &mut stream
        ).is_err()
    }
}

// Ignore the test by default to avoid overwritting the buffer frame size
// within the same input device that is used in test_configure_input.
#[test]
#[ignore]
fn test_configure_input_with_zero_latency_frames() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 48_000;
    raw.channels = 1;
    raw.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.input_stream_params = StreamParams::from(raw);

    // It's crucial to call to audiounit_set_device_info to set
    // stream.input_device, or we will hit the
    // assertion in audiounit_create_unit.

    let default_input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    if !valid_id(default_input_id) {
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::INPUT
        ).is_ok()
    );

    assert_eq!(stream.input_device.id, default_input_id);
    assert_eq!(
        stream.input_device.flags,
        device_flags::DEV_INPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(
        audiounit_create_unit(
            &mut stream.input_unit,
            &stream.input_device
        ).is_ok()
    );

    assert!(!stream.input_unit.is_null());

    assert_eq!(stream.latency_frames, 0);

    assert!(
        audiounit_configure_input(
            &mut stream
        ).is_ok()
    );

    assert_ne!(
        stream.input_hw_rate,
        0_f64
    );

    let mut description = AudioStreamBasicDescription::default();
    let mut size = mem::size_of::<AudioStreamBasicDescription>();
    assert_eq!(
        audio_unit_get_property(
            stream.input_unit,
            kAudioUnitProperty_StreamFormat,
            kAudioUnitScope_Output,
            AU_IN_BUS,
            &mut description,
            &mut size
        ),
        0
    );
    assert_eq!(
        description.mSampleRate,
        stream.input_hw_rate
    );

    let mut buffer_frames: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            stream.input_unit,
            kAudioDevicePropertyBufferFrameSize,
            kAudioUnitScope_Output,
            AU_IN_BUS,
            &mut buffer_frames,
            &mut size
        ),
        0
    );
    // TODO: buffer frames size won't be 0 even it's ok to set that!
    assert_ne!(
        stream.latency_frames,
        buffer_frames
    );

    let mut frames_per_slice: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            stream.input_unit,
            kAudioUnitProperty_MaximumFramesPerSlice,
            kAudioUnitScope_Global,
            0,
            &mut frames_per_slice,
            &mut size
        ),
        0
    );
    // TODO: frames per slice won't be 0 even it's ok to set that!
    assert_ne!(
        stream.latency_frames,
        frames_per_slice
    );
}

fn test_configure_input_impl<T: std::any::Any>(array: &[T]) {
    // Get format parameters for the type.
    let type_id = std::any::TypeId::of::<T>();
    let format = if type_id == std::any::TypeId::of::<f32>() {
        ffi::CUBEB_SAMPLE_FLOAT32NE
    } else if type_id == std::any::TypeId::of::<i16>() {
        ffi::CUBEB_SAMPLE_S16NE
    } else {
        panic!("Unsupported type!");
    };

    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();
    assert!(stream.input_linear_buffer.is_none());

    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = format;
    raw.rate = 48_000;
    raw.channels = 1;
    raw.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.input_stream_params = StreamParams::from(raw);

    // It's crucial to call to audiounit_set_device_info to set
    // stream.input_device, or we will hit the
    // assertion in audiounit_create_unit.

    let default_input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    if !valid_id(default_input_id) {
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::INPUT
        ).is_ok()
    );

    assert_eq!(stream.input_device.id, default_input_id);
    assert_eq!(
        stream.input_device.flags,
        device_flags::DEV_INPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(
        audiounit_create_unit(
            &mut stream.input_unit,
            &stream.input_device
        ).is_ok()
    );

    assert!(!stream.input_unit.is_null());

    // Set the latency_frames to a valid value so `buffer frames size` and
    // `frames per slice` can be set correctly! Comparing the checks for
    // these two with `test_configure_input_with_zero_latency_frames` to
    // know why latency_frames should be set to a correct value.
    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut stream.context.mutex as *mut OwnedCriticalSection;
        // The scope of `ctx_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        assert_eq!(stream.latency_frames, 0);
        stream.latency_frames = audiounit_clamp_latency(&mut stream, 0);
        assert_ne!(stream.latency_frames, 0);
    }

    assert!(
        audiounit_configure_input(
            &mut stream
        ).is_ok()
    );

    assert_ne!(
        stream.input_hw_rate,
        0_f64
    );

    let mut description = AudioStreamBasicDescription::default();
    let mut size = mem::size_of::<AudioStreamBasicDescription>();
    assert_eq!(
        audio_unit_get_property(
            stream.input_unit,
            kAudioUnitProperty_StreamFormat,
            kAudioUnitScope_Output,
            AU_IN_BUS,
            &mut description,
            &mut size
        ),
        0
    );
    assert_eq!(
        description.mSampleRate,
        stream.input_hw_rate
    );

    let mut buffer_frames: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            stream.input_unit,
            kAudioDevicePropertyBufferFrameSize,
            kAudioUnitScope_Output,
            AU_IN_BUS,
            &mut buffer_frames,
            &mut size
        ),
        0
    );
    assert_eq!(
        stream.latency_frames,
        buffer_frames
    );

    let mut frames_per_slice: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            stream.input_unit,
            kAudioUnitProperty_MaximumFramesPerSlice,
            kAudioUnitScope_Global,
            0,
            &mut frames_per_slice,
            &mut size
        ),
        0
    );
    assert_eq!(
        stream.latency_frames,
        frames_per_slice
    );

    assert!(stream.input_linear_buffer.is_some());
    stream.input_linear_buffer.as_mut().unwrap().push(
        array.as_ptr() as *const c_void,
        array.len()
    );

    // TODO: Check input callback ...
    // struct Data {
    //     stream: *mut ffi::cubeb_stream,
    //     called: usize,
    //     states: [ffi::cubeb_state; 2]
    // }

    // let mut data = Data {
    //     stream: &mut stream as *mut AudioUnitStream as *mut ffi::cubeb_stream,
    //     called: 0,
    //     states: [ffi::CUBEB_STATE_STARTED, ffi::CUBEB_STATE_STOPPED]
    // };

    // extern fn state_callback(
    //     stm: *mut ffi::cubeb_stream,
    //     user_ptr: *mut c_void,
    //     state: ffi::cubeb_state
    // ) {
    //     let data = unsafe { &mut *(user_ptr as *mut Data) };
    //     assert_eq!(stm, data.stream);
    //     assert_eq!(state, data.states[data.called]);
    //     data.called += 1;
    // }
    // stream.user_ptr = &mut data as *mut Data as *mut c_void;
    // stream.state_callback = Some(state_callback);
    // audio_unit_initialize(stream.input_unit);
    // assert!(stream.start().is_ok());
    // for i in 0..10000000 {}
    // assert!(stream.stop().is_ok());
}

#[test]
fn test_configure_input() {
    test_configure_input_impl(&[1.1_f32, 2.2, 3.3, 4.4]);
    test_configure_input_impl(&[1_i16, 2, 3, 4, 5, 6, 7]);
}

// configure_output
// ------------------------------------
#[test]
#[should_panic]
fn test_configure_output_with_null_unit() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    assert!(stream.output_unit.is_null());
    assert!{
        audiounit_configure_output(
            &mut stream
        ).is_err()
    }
}

// Ignore the test by default to avoid overwritting the buffer frame size
// within the same output device that is used in test_configure_output.
#[test]
#[ignore]
fn test_configure_output_with_zero_latency_frames() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 44_100;
    raw.channels = 2;
    raw.layout = ffi::CUBEB_LAYOUT_STEREO;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // It's crucial to call to audiounit_set_device_info to set
    // stream.input_device, or we will hit the
    // assertion in audiounit_create_unit.

    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(default_output_id) {
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::OUTPUT
        ).is_ok()
    );

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(
        audiounit_create_unit(
            &mut stream.output_unit,
            &stream.output_device
        ).is_ok()
    );

    assert!(!stream.output_unit.is_null());

    assert_eq!(stream.latency_frames, 0);

    assert!(
        audiounit_configure_output(
            &mut stream
        ).is_ok()
    );

    assert_ne!(
        stream.output_hw_rate,
        0_f64
    );

    let mut description = AudioStreamBasicDescription::default();
    let mut size = mem::size_of::<AudioStreamBasicDescription>();
    assert_eq!(
        audio_unit_get_property(
            stream.output_unit,
            kAudioUnitProperty_StreamFormat,
            kAudioUnitScope_Output,
            AU_OUT_BUS,
            &mut description,
            &mut size
        ),
        0
    );
    assert_eq!(
        description.mSampleRate,
        stream.output_hw_rate
    );

    let mut buffer_frames: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            stream.output_unit,
            kAudioDevicePropertyBufferFrameSize,
            kAudioUnitScope_Input,
            AU_OUT_BUS,
            &mut buffer_frames,
            &mut size
        ),
        0
    );
    // TODO: buffer frames size won't be 0 even it's ok to set that!
    assert_ne!(
        stream.latency_frames,
        buffer_frames
    );

    let mut frames_per_slice: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            stream.output_unit,
            kAudioUnitProperty_MaximumFramesPerSlice,
            kAudioUnitScope_Global,
            0,
            &mut frames_per_slice,
            &mut size
        ),
        0
    );
    // TODO: frames per slice won't be 0 even it's ok to set that!
    assert_ne!(
        stream.latency_frames,
        frames_per_slice
    );
}

#[test]
fn test_configure_output() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 44_100;
    raw.channels = 2;
    raw.layout = ffi::CUBEB_LAYOUT_STEREO;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // It's crucial to call to audiounit_set_device_info to set
    // stream.input_device, or we will hit the
    // assertion in audiounit_create_unit.

    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(default_output_id) {
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::OUTPUT
        ).is_ok()
    );

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(
        audiounit_create_unit(
            &mut stream.output_unit,
            &stream.output_device
        ).is_ok()
    );

    assert!(!stream.output_unit.is_null());

    // Set the latency_frames to a valid value so `buffer frames size` and
    // `frames per slice` can be set correctly! Comparing the checks for
    // these two with `test_configure_output_with_zero_latency_frames` to
    // know why latency_frames should be set to a correct value.
    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut stream.context.mutex as *mut OwnedCriticalSection;
        // The scope of `ctx_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        assert_eq!(stream.latency_frames, 0);
        stream.latency_frames = audiounit_clamp_latency(&mut stream, 0);
        assert_ne!(stream.latency_frames, 0);
    }

    assert!(
        audiounit_configure_output(
            &mut stream
        ).is_ok()
    );

    assert_ne!(
        stream.output_hw_rate,
        0_f64
    );

    let mut description = AudioStreamBasicDescription::default();
    let mut size = mem::size_of::<AudioStreamBasicDescription>();
    assert_eq!(
        audio_unit_get_property(
            stream.output_unit,
            kAudioUnitProperty_StreamFormat,
            kAudioUnitScope_Output,
            AU_OUT_BUS,
            &mut description,
            &mut size
        ),
        0
    );
    assert_eq!(
        description.mSampleRate,
        stream.output_hw_rate
    );

    let mut buffer_frames: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            stream.output_unit,
            kAudioDevicePropertyBufferFrameSize,
            kAudioUnitScope_Input,
            AU_OUT_BUS,
            &mut buffer_frames,
            &mut size
        ),
        0
    );
    assert_eq!(
        stream.latency_frames,
        buffer_frames
    );

    let mut frames_per_slice: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            stream.output_unit,
            kAudioUnitProperty_MaximumFramesPerSlice,
            kAudioUnitScope_Global,
            0,
            &mut frames_per_slice,
            &mut size
        ),
        0
    );
    assert_eq!(
        stream.latency_frames,
        frames_per_slice
    );

    // TODO: check layout, output callback, ....
    // struct Data {
    //     stream: *mut ffi::cubeb_stream,
    //     called: usize,
    //     states: [ffi::cubeb_state; 2]
    // }

    // let mut data = Data {
    //     stream: &mut stream as *mut AudioUnitStream as *mut ffi::cubeb_stream,
    //     called: 0,
    //     states: [ffi::CUBEB_STATE_STARTED, ffi::CUBEB_STATE_STOPPED]
    // };

    // extern fn state_callback(
    //     stm: *mut ffi::cubeb_stream,
    //     user_ptr: *mut c_void,
    //     state: ffi::cubeb_state
    // ) {
    //     println!("state: {}", state);
    //     let data = unsafe { &mut *(user_ptr as *mut Data) };
    //     assert_eq!(stm, data.stream);
    //     assert_eq!(state, data.states[data.called]);
    //     data.called += 1;
    // }
    // stream.user_ptr = &mut data as *mut Data as *mut c_void;
    // stream.state_callback = Some(state_callback);
    // audio_unit_initialize(stream.output_unit);
    // assert!(stream.start().is_ok());
    // for i in 0..10000000 {}
    // assert!(stream.stop().is_ok());
}

// setup_stream
// ------------------------------------
// TODO

// stream_destroy_internal
// ------------------------------------
// TODO

// stream_destroy
// ------------------------------------
// TODO

// stream_start_internal
// ------------------------------------
// TODO

// stream_start
// ------------------------------------
// TODO

// stream_stop_internal
// ------------------------------------
// TODO

// stream_get_volume
// ------------------------------------
#[test]
fn test_stream_get_volume() {
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.

    {
        // Create a `ctx_mutext_ptr` here to avoid borrowing issues for `ctx`.
        let ctx_mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

    // The resampler will be initialized in `audiounit_setup_stream` (or via
    // `stream_init`), and it only accepts the formats with FLOAT32NE or S16NE.
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = 96_000;
    raw.channels = 32;
    raw.layout = ffi::CUBEB_LAYOUT_3F1_LFE;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // It's crucial to call to audiounit_set_device_info to set
    // stream.output_device to output device type, or we will hit the
    // assertion in audiounit_create_unit.

    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    // Return an error if there is no available device.
    if !valid_id(default_output_id) {
        return;
    }

    assert!(
        audiounit_set_device_info(
            &mut stream,
            kAudioObjectUnknown,
            io_side::OUTPUT
        ).is_ok()
    );

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT |
        device_flags::DEV_SELECTED_DEFAULT |
        device_flags::DEV_SYSTEM_DEFAULT
    );

    {
        let ctx_mutex_ptr = &mut stream.context.mutex as *mut OwnedCriticalSection;
        let _ctx_lock = AutoLock::new(unsafe { &mut (*ctx_mutex_ptr) });
        let stm_mutex_ptr = &mut stream.mutex as *mut OwnedCriticalSection;
        let _stm_lock = AutoLock::new(unsafe { &mut (*stm_mutex_ptr) });
        assert!(audiounit_setup_stream(&mut stream).is_ok());
    }

    let expected_volume: f32 = 0.5;
    stream.set_volume(expected_volume);

    let mut actual_volume: f32 = 0.0;
    assert!(audiounit_stream_get_volume(&stream, &mut actual_volume).is_ok());

    assert_eq!(expected_volume, actual_volume);
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
    // We need to initialize the members with type OwnedCriticalSection in
    // AudioUnitContext and AudioUnitStream, since those OwnedCriticalSection
    // will be used when AudioUnitStream::drop/destroy is called.
    let mut ctx = AudioUnitContext::new();
    ctx.init();

    // Add a stream to the context. `AudioUnitStream::drop()` will check
    // the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid borrowing issues for `ctx`.
        let mutex_ptr = &mut ctx.mutex as *mut OwnedCriticalSection;

        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut ctx);
    }

    let mut stream = AudioUnitStream::new(
        &mut ctx,
        ptr::null_mut(),
        None,
        None,
        0
    );
    stream.init();

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
    assert!(!device.input_name.is_null());
    assert!(device.output_name.is_null());

    // output:
    device = ffi::cubeb_device::default();
    assert!(
        audiounit_get_default_device_name(
            &stream,
            &mut device,
            DeviceType::OUTPUT
        ).is_ok()
    );
    assert!(device.input_name.is_null());
    assert!(!device.output_name.is_null());

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
    assert!(device.input_name.is_null());
    assert!(device.output_name.is_null());

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
    assert!(!info.devid.is_null());
    assert!(!info.device_id.is_null());
    assert_eq!(info.group_id, info.device_id);
    assert!(!info.friendly_name.is_null());
    assert!(!info.vendor_name.is_null());
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
        assert!(!info.devid.is_null());
        assert!(info.device_id.is_null());
        assert_eq!(info.group_id, info.device_id);
        assert!(!info.friendly_name.is_null());
        assert!(!info.vendor_name.is_null());
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
        assert!(!info.devid.is_null());
        assert!(!info.device_id.is_null());
        assert_eq!(info.group_id, info.device_id);
        assert!(!info.friendly_name.is_null());
        assert!(!info.vendor_name.is_null());
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
    assert!(!info.devid.is_null());
    assert!(!info.device_id.is_null());
    assert_eq!(info.group_id, info.device_id);
    assert!(!info.friendly_name.is_null());
    assert!(!info.vendor_name.is_null());
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
#[test]
fn test_is_aggregate_device() {
    let mut aggregate_name = String::from(PRIVATE_AGGREGATE_DEVICE_NAME);
    aggregate_name.push_str("_something");
    let aggregate_name_cstring = CString::new(aggregate_name).unwrap();

    let mut info = ffi::cubeb_device_info::default();
    info.friendly_name = aggregate_name_cstring.as_ptr();
    assert!(is_aggregate_device(&info));

    let non_aggregate_name_cstring = CString::new("Hello World!").unwrap();
    info.friendly_name = non_aggregate_name_cstring.as_ptr();
    assert!(!is_aggregate_device(&info));
}

// device_destroy
// ------------------------------------
#[test]
fn test_device_destroy_empty_device() {
    let mut device = ffi::cubeb_device_info::default();

    assert!(device.device_id.is_null());
    assert!(device.group_id.is_null());
    assert!(device.friendly_name.is_null());
    assert!(device.vendor_name.is_null());

    audiounit_device_destroy(&mut device);

    assert!(device.device_id.is_null());
    assert!(device.group_id.is_null());
    assert!(device.friendly_name.is_null());
    assert!(device.vendor_name.is_null());
}

#[test]
#[should_panic]
fn test_device_destroy_with_different_device_id_and_group_id() {
    let mut device = ffi::cubeb_device_info::default();

    device.device_id = CString::new("device id")
        .expect("Failed on creating device id")
        .into_raw();
    // The result should be same if the group_id is null by comment the
    // following line.
    device.group_id = CString::new("group id")
        .expect("Failed on creating device id")
        .into_raw();
    device.friendly_name = CString::new("friendly name")
        .expect("Failed on creating friendly name")
        .into_raw();
    device.vendor_name = CString::new("vendor name")
        .expect("Failed on creating vendor name")
        .into_raw();

    audiounit_device_destroy(&mut device);

    assert!(device.device_id.is_null());
    assert!(device.group_id.is_null());
    assert!(device.friendly_name.is_null());
    assert!(device.vendor_name.is_null());
}

#[test]
fn test_device_destroy() {
    let mut device = ffi::cubeb_device_info::default();

    device.device_id = CString::new("device id")
        .expect("Failed on creating device id")
        .into_raw();
    device.group_id = device.device_id;
    device.friendly_name = CString::new("friendly name")
        .expect("Failed on creating friendly name")
        .into_raw();
    device.vendor_name = CString::new("vendor name")
        .expect("Failed on creating vendor name")
        .into_raw();

    audiounit_device_destroy(&mut device);

    assert!(device.device_id.is_null());
    assert!(device.group_id.is_null());
    assert!(device.friendly_name.is_null());
    assert!(device.vendor_name.is_null());
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
        assert!(all_devs.contains(&output_id));
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

fn unit_scope_is_enabled(unit: AudioUnit, is_input: bool) -> bool {
    assert!(!unit.is_null());
    let mut has_io: UInt32 = 0;
    assert_eq!(
        audio_unit_get_property(
            unit,
            kAudioOutputUnitProperty_HasIO,
            if is_input { kAudioUnitScope_Input } else { kAudioUnitScope_Output },
            if is_input { AU_IN_BUS } else { AU_OUT_BUS },
            &mut has_io,
            &mut mem::size_of::<UInt32>()
        ),
        0
    );
    has_io != 0
}

fn to_devices_names(devices: &Vec<AudioObjectID>) -> Vec<Option<String>> {
    let mut names = Vec::new();
    for device in devices {
        names.push(
            to_device_name(*device)
        );
    }
    names
}

fn to_device_name(id: AudioObjectID) -> Option<String> {
    let name_ref = get_device_name(id);
    if name_ref.is_null() {
        return None;
    }

    let name = strref_to_string(name_ref);
    unsafe {
        CFRelease(name_ref as *const c_void);
    }
    Some(name)
}

fn strref_to_string(strref: CFStringRef) -> String {
    let cstring = audiounit_strref_to_cstr_utf8(strref);
    cstring.into_string().unwrap()
}
