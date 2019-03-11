use super::*;

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
