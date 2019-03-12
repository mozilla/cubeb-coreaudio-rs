use super::*;

// Test Templates
// ------------------------------------------------------------------------------------------------
fn test_context_operation<F>(name: &'static str, operation: F)
where
    F: FnOnce(*mut ffi::cubeb),
{
    use std::ffi::CString;
    let name_c_string = CString::new(name).expect("Failed to create context name");
    let mut context = ptr::null_mut::<ffi::cubeb>();
    assert_eq!(
        unsafe { OPS.init.unwrap()(&mut context, name_c_string.as_ptr()) },
        ffi::CUBEB_OK
    );
    assert!(!context.is_null());
    operation(context);
    unsafe { OPS.destroy.unwrap()(context) }
}

// Note: The in-out stream initializeed with different device will create an aggregate_device and
//       result in firing device-collection-changed callbacks. Run in-out streams with tests
//       capturing device-collection-changed callbacks may cause troubles. See more details in the
//       comments for test_create_blank_aggregate_device.
fn test_stream_operation<F>(
    name: &'static str,
    input_stream_params: *mut ffi::cubeb_stream_params,
    output_stream_params: *mut ffi::cubeb_stream_params,
    data_callback: ffi::cubeb_data_callback,
    state_callback: ffi::cubeb_state_callback,
    user_ptr: *mut c_void,
    operation: F,
) where
    F: FnOnce(*mut ffi::cubeb_stream),
{
    test_context_operation("context: stream operation", |context_ptr| {
        let mut stream: *mut ffi::cubeb_stream = ptr::null_mut();
        let stream_name = CString::new(name).expect("Failed to create stream name");
        // TODO: stream_init fails when there is no input/output device when the stream parameter
        //       for input/output is given.
        assert_eq!(
            unsafe {
                OPS.stream_init.unwrap()(
                    context_ptr,
                    &mut stream,
                    stream_name.as_ptr(),
                    ptr::null(), // Use default input device
                    input_stream_params,
                    ptr::null(), // Use default output device
                    output_stream_params,
                    4096, // TODO: Get latency by get_min_latency instead ?
                    data_callback,
                    state_callback,
                    user_ptr,
                )
            },
            ffi::CUBEB_OK
        );
        assert!(!stream.is_null());
        operation(stream);
        unsafe {
            OPS.stream_destroy.unwrap()(stream);
        }
    });
}

fn test_default_output_stream_operation<F>(name: &'static str, operation: F)
where
    F: FnOnce(*mut ffi::cubeb_stream),
{
    // Make sure the parameters meet the requirements of AudioUnitContext::stream_init
    // (in the comments).
    let mut output_params = ffi::cubeb_stream_params::default();
    output_params.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    output_params.rate = 44100;
    output_params.channels = 2;
    output_params.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
    output_params.prefs = ffi::CUBEB_STREAM_PREF_NONE;

    // TODO: test_stream_operation fails and hit an assertion when there is no device,
    test_stream_operation(
        name,
        ptr::null_mut(),
        &mut output_params,
        None,
        None,
        ptr::null_mut(),
        operation,
    );
}

// Context Operations
// ------------------------------------------------------------------------------------------------
#[test]
fn test_ops_context_init_and_destroy() {
    test_context_operation("context: init and destroy", |_context_ptr| {});
}

#[test]
fn test_ops_context_backend_id() {
    test_context_operation("context: backend id", |context_ptr| {
        let backend = unsafe {
            let ptr = OPS.get_backend_id.unwrap()(context_ptr);
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        };
        assert_eq!(backend, "audiounit-rust");
    });
}

#[test]
fn test_ops_context_max_channel_count() {
    test_context_operation("context: max channel count", |context_ptr| {
        let having_output = get_default_device_id(Scope::Output).is_some();
        let mut max_channel_count = 0;
        let r = unsafe { OPS.get_max_channel_count.unwrap()(context_ptr, &mut max_channel_count) };
        if having_output {
            assert_eq!(r, ffi::CUBEB_OK);
            assert_ne!(max_channel_count, 0);
        } else {
            assert_eq!(r, ffi::CUBEB_ERROR);
            assert_eq!(max_channel_count, 0);
        }
    });
}

#[test]
fn test_ops_context_min_latency() {
    test_context_operation("context: min latency", |context_ptr| {
        let having_output = get_default_device_id(Scope::Output).is_some();
        let params = ffi::cubeb_stream_params::default();
        let mut latency = u32::max_value();
        let r = unsafe { OPS.get_min_latency.unwrap()(context_ptr, params, &mut latency) };
        if having_output {
            assert_eq!(r, ffi::CUBEB_OK);
            assert!(latency >= SAFE_MIN_LATENCY_FRAMES);
            assert!(SAFE_MAX_LATENCY_FRAMES >= latency);
        } else {
            assert_eq!(r, ffi::CUBEB_ERROR);
            assert_eq!(latency, u32::max_value());
        }
    });
}

#[test]
fn test_ops_context_preferred_sample_rate() {
    test_context_operation("context: preferred sample rate", |context_ptr| {
        let having_output = get_default_device_id(Scope::Output).is_some();
        let mut rate = u32::max_value();
        let r = unsafe { OPS.get_preferred_sample_rate.unwrap()(context_ptr, &mut rate) };
        if having_output {
            assert_eq!(r, ffi::CUBEB_OK);
            assert_ne!(rate, u32::max_value());
            assert_ne!(rate, 0);
        } else {
            assert_eq!(r, ffi::CUBEB_ERROR);
            assert_eq!(rate, u32::max_value());
        }
    });
}

#[test]
fn test_ops_context_enumerate_devices_unknown() {
    test_context_operation("context: enumerate devices (unknown)", |context_ptr| {
        let mut coll = ffi::cubeb_device_collection {
            device: ptr::null_mut(),
            count: 0,
        };
        assert_eq!(
            unsafe {
                OPS.enumerate_devices.unwrap()(
                    context_ptr,
                    ffi::CUBEB_DEVICE_TYPE_UNKNOWN,
                    &mut coll,
                )
            },
            ffi::CUBEB_OK
        );
        assert_eq!(coll.count, 0);
        assert_eq!(coll.device, ptr::null_mut());
        assert_eq!(
            unsafe { OPS.device_collection_destroy.unwrap()(context_ptr, &mut coll) },
            ffi::CUBEB_OK
        );
        assert_eq!(coll.count, 0);
        assert_eq!(coll.device, ptr::null_mut());
    });
}

#[test]
fn test_ops_context_enumerate_devices_input() {
    test_context_operation("context: enumerate devices (input)", |context_ptr| {
        let having_input = get_default_device_id(Scope::Input).is_some();
        let mut coll = ffi::cubeb_device_collection {
            device: ptr::null_mut(),
            count: 0,
        };
        assert_eq!(
            unsafe {
                OPS.enumerate_devices.unwrap()(context_ptr, ffi::CUBEB_DEVICE_TYPE_INPUT, &mut coll)
            },
            ffi::CUBEB_OK
        );
        if having_input {
            assert_ne!(coll.count, 0);
            assert_ne!(coll.device, ptr::null_mut());
        } else {
            assert_eq!(coll.count, 0);
            assert_eq!(coll.device, ptr::null_mut());
        }
        assert_eq!(
            unsafe { OPS.device_collection_destroy.unwrap()(context_ptr, &mut coll) },
            ffi::CUBEB_OK
        );
        assert_eq!(coll.count, 0);
        assert_eq!(coll.device, ptr::null_mut());
    });
}

#[test]
fn test_ops_context_enumerate_devices_output() {
    test_context_operation("context: enumerate devices (output)", |context_ptr| {
        let having_output = get_default_device_id(Scope::Output).is_some();
        let mut coll = ffi::cubeb_device_collection {
            device: ptr::null_mut(),
            count: 0,
        };
        assert_eq!(
            unsafe {
                OPS.enumerate_devices.unwrap()(
                    context_ptr,
                    ffi::CUBEB_DEVICE_TYPE_OUTPUT,
                    &mut coll,
                )
            },
            ffi::CUBEB_OK
        );
        if having_output {
            assert_ne!(coll.count, 0);
            assert_ne!(coll.device, ptr::null_mut());
        } else {
            assert_eq!(coll.count, 0);
            assert_eq!(coll.device, ptr::null_mut());
        }
        assert_eq!(
            unsafe { OPS.device_collection_destroy.unwrap()(context_ptr, &mut coll) },
            ffi::CUBEB_OK
        );
        assert_eq!(coll.count, 0);
        assert_eq!(coll.device, ptr::null_mut());
    });
}

#[test]
fn test_ops_context_device_collection_destroy() {
    // Destroy a dummy device collection, without calling enumerate_devices to allocate memory for the device collection
    test_context_operation("context: device collection destroy", |context_ptr| {
        let mut coll = ffi::cubeb_device_collection {
            device: ptr::null_mut(),
            count: 0,
        };
        assert_eq!(
            unsafe { OPS.device_collection_destroy.unwrap()(context_ptr, &mut coll) },
            ffi::CUBEB_OK
        );
        assert_eq!(coll.device, ptr::null_mut());
        assert_eq!(coll.count, 0);
    });
}

#[test]
fn test_ops_context_register_device_collection_changed_unknown() {
    test_context_operation(
        "context: register device collection changed (unknown)",
        |context_ptr| {
            assert_eq!(
                unsafe {
                    OPS.register_device_collection_changed.unwrap()(
                        context_ptr,
                        ffi::CUBEB_DEVICE_TYPE_UNKNOWN,
                        None,
                        ptr::null_mut(),
                    )
                },
                ffi::CUBEB_ERROR_INVALID_PARAMETER
            );
        },
    );
}

fn test_ops_context_register_device_collection_changed_twice(devtype: u32) {
    extern "C" fn callback(_: *mut ffi::cubeb, _: *mut c_void) {}
    let label_input: &'static str = "context: register device collection changed twice (input)";
    let label_output: &'static str = "context: register device collection changed twice (output)";
    let label_inout: &'static str = "context: register device collection changed twice (inout)";
    let label = if devtype == ffi::CUBEB_DEVICE_TYPE_INPUT {
        label_input
    } else if devtype == ffi::CUBEB_DEVICE_TYPE_OUTPUT {
        label_output
    } else if devtype == ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT {
        label_inout
    } else {
        return;
    };

    test_context_operation(label, |context_ptr| {
        // Register a callback within the defined scope.
        assert_eq!(
            unsafe {
                OPS.register_device_collection_changed.unwrap()(
                    context_ptr,
                    devtype,
                    Some(callback),
                    ptr::null_mut(),
                )
            },
            ffi::CUBEB_OK
        );

        // Hit an assertion when registering two callbacks within the same scope.
        assert_eq!(
            unsafe {
                OPS.register_device_collection_changed.unwrap()(
                    context_ptr,
                    devtype,
                    Some(callback),
                    ptr::null_mut(),
                )
            },
            ffi::CUBEB_ERROR
        );
    });
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
    test_ops_context_register_device_collection_changed_twice(
        ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT,
    );
}

#[test]
fn test_ops_context_register_device_collection_changed() {
    extern "C" fn callback(_: *mut ffi::cubeb, _: *mut c_void) {}
    test_context_operation(
        "context: register device collection changed",
        |context_ptr| {
            let devtypes: [ffi::cubeb_device_type; 3] = [
                ffi::CUBEB_DEVICE_TYPE_INPUT,
                ffi::CUBEB_DEVICE_TYPE_OUTPUT,
                ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT,
            ];

            for devtype in &devtypes {
                // Register a callback in the defined scoped.
                assert_eq!(
                    unsafe {
                        OPS.register_device_collection_changed.unwrap()(
                            context_ptr,
                            *devtype,
                            Some(callback),
                            ptr::null_mut(),
                        )
                    },
                    ffi::CUBEB_OK
                );

                // Unregister all callbacks regardless of the scope.
                assert_eq!(
                    unsafe {
                        OPS.register_device_collection_changed.unwrap()(
                            context_ptr,
                            ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT,
                            None,
                            ptr::null_mut(),
                        )
                    },
                    ffi::CUBEB_OK
                );

                // Register callback in the defined scoped again.
                assert_eq!(
                    unsafe {
                        OPS.register_device_collection_changed.unwrap()(
                            context_ptr,
                            *devtype,
                            Some(callback),
                            ptr::null_mut(),
                        )
                    },
                    ffi::CUBEB_OK
                );

                // Unregister callback within the defined scope.
                assert_eq!(
                    unsafe {
                        OPS.register_device_collection_changed.unwrap()(
                            context_ptr,
                            *devtype,
                            None,
                            ptr::null_mut(),
                        )
                    },
                    ffi::CUBEB_OK
                );
            }
        },
    );
}

#[test]
#[ignore]
fn test_ops_context_register_device_collection_changed_manual() {
    test_context_operation(
        "(manual) context: register device collection changed",
        |context_ptr| {
            println!("context @ {:p}", context_ptr);

            struct Data {
                context: *mut ffi::cubeb,
                touched: u32, // TODO: Use AtomicU32 instead
            }

            extern "C" fn input_callback(context: *mut ffi::cubeb, user: *mut c_void) {
                println!("input > context @ {:p}", context);
                let data = unsafe { &mut (*(user as *mut Data)) };
                assert_eq!(context, data.context);
                data.touched += 1;
            }

            extern "C" fn output_callback(context: *mut ffi::cubeb, user: *mut c_void) {
                println!("output > context @ {:p}", context);
                let data = unsafe { &mut (*(user as *mut Data)) };
                assert_eq!(context, data.context);
                data.touched += 1;
            }

            let mut data = Data {
                context: context_ptr,
                touched: 0,
            };

            // Register a callback for input scope.
            assert_eq!(
                unsafe {
                    OPS.register_device_collection_changed.unwrap()(
                        context_ptr,
                        ffi::CUBEB_DEVICE_TYPE_INPUT,
                        Some(input_callback),
                        &mut data as *mut Data as *mut c_void,
                    )
                },
                ffi::CUBEB_OK
            );

            // Register a callback for output scope.
            assert_eq!(
                unsafe {
                    OPS.register_device_collection_changed.unwrap()(
                        context_ptr,
                        ffi::CUBEB_DEVICE_TYPE_OUTPUT,
                        Some(output_callback),
                        &mut data as *mut Data as *mut c_void,
                    )
                },
                ffi::CUBEB_OK
            );

            while data.touched < 2 {}
        },
    );
}

// Stream Operations
// ------------------------------------------------------------------------------------------------
#[test]
fn test_ops_stream_init_and_destroy() {
    test_default_output_stream_operation("stream: init and destroy", |_stream| {});
}

#[test]
fn test_ops_stream_start() {
    test_default_output_stream_operation("stream: start", |stream| {
        assert_eq!(unsafe { OPS.stream_start.unwrap()(stream) }, ffi::CUBEB_OK);
    });
}

#[test]
fn test_ops_stream_stop() {
    test_default_output_stream_operation("stream: stop", |stream| {
        assert_eq!(unsafe { OPS.stream_stop.unwrap()(stream) }, ffi::CUBEB_OK);
    });
}

#[test]
fn test_ops_stream_reset_default_device() {
    test_default_output_stream_operation("stream: reset default device", |stream| {
        assert_eq!(
            unsafe { OPS.stream_reset_default_device.unwrap()(stream) },
            ffi::CUBEB_ERROR_NOT_SUPPORTED
        );
    });
}

#[test]
fn test_ops_stream_position() {
    test_default_output_stream_operation("stream: position", |stream| {
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
    test_default_output_stream_operation("stream: latency", |stream| {
        let mut latency = u32::max_value();
        assert_eq!(
            unsafe { OPS.stream_get_latency.unwrap()(stream, &mut latency) },
            ffi::CUBEB_OK
        );
        assert_ne!(latency, u32::max_value());
        assert_ne!(latency, 0);
    });
}

#[test]
fn test_ops_stream_set_volume() {
    test_default_output_stream_operation("stream: set volume", |stream| {
        assert_eq!(
            unsafe { OPS.stream_set_volume.unwrap()(stream, 0.5) },
            ffi::CUBEB_OK
        );
    });
}

#[test]
fn test_ops_stream_set_panning() {
    test_default_output_stream_operation("stream: set panning", |stream| {
        assert_eq!(
            unsafe { OPS.stream_set_panning.unwrap()(stream, 0.5) },
            ffi::CUBEB_OK
        );
    });
}

#[test]
fn test_ops_stream_current_device() {
    test_default_output_stream_operation("stream: get current device and destroy it", |stream| {
        let mut device: *mut ffi::cubeb_device = ptr::null_mut();
        // TODO: stream_get_current_device only returns OK when the machine has both input and
        //       output devices.
        assert_eq!(
            unsafe { OPS.stream_get_current_device.unwrap()(stream, &mut device) },
            ffi::CUBEB_OK
        );
        assert!(!device.is_null());
        // Uncomment the below to print out the results.
        // let deviceref = unsafe { DeviceRef::from_ptr(device) };
        // println!("output: {}", deviceref.output_name().unwrap_or("(no device name)"));
        // println!("input: {}", deviceref.input_name().unwrap_or("(no device name)"));
        assert_eq!(
            unsafe { OPS.stream_device_destroy.unwrap()(stream, device) },
            ffi::CUBEB_OK
        );
    });
}

#[test]
fn test_ops_stream_device_destroy() {
    test_default_output_stream_operation("stream: destroy null device", |stream| {
        assert_eq!(
            unsafe {
                OPS.stream_device_destroy.unwrap()(stream, ptr::null_mut())
            },
            ffi::CUBEB_OK // It returns OK anyway.
        );
    });
}

// Enable this after cubeb-rs version is updated to one that implements
// stream_register_device_changed_callback operation.
// #[test]
// fn test_ops_stream_register_device_changed_callback() {
//     extern "C" fn callback(_: *mut c_void) {}

//     test_default_output_stream_operation("stream: register device changed callback", |stream| {
//         assert_eq!(
//             unsafe {
//                 OPS.stream_register_device_changed_callback.unwrap()(
//                     stream,
//                     Some(callback)
//                 )
//             },
//             ffi::CUBEB_OK
//         );
//     });
// }

// TODO: Add a manual test for stream_register_device_changed_callback operation.
