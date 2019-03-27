// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

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

// Private APIs
// ============================================================================
// get_sub_devices
// ------------------------------------
// Ignore this by default. The reason is same as below.
#[test]
#[ignore]
fn test_get_sub_devices_for_blank_aggregate_devices() {
    // TODO: Test this when there is no available devices.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(&mut plugin_id, &mut aggregate_device_id).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);
    // There is no sub devices for a blank aggregate device!
    let devices = audiounit_get_sub_devices(aggregate_device_id);
    assert!(devices.is_empty());

    assert!(audiounit_destroy_aggregate_device(plugin_id, &mut aggregate_device_id).is_ok());
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
//
// The tests that call audiounit_create_blank_aggregate_device are ignored by default:
// - test_get_sub_devices_for_blank_aggregate_devices
// - test_create_blank_aggregate_device
// - test_set_aggregate_sub_device_list_for_unknown_input_output_devices
// - test_set_aggregate_sub_device_list
// - test_set_master_aggregate_device_for_a_blank_aggregate_device
// - test_set_master_aggregate_device
// - test_activate_clock_drift_compensation_for_an_aggregate_device_without_master_device
// - test_activate_clock_drift_compensation
#[test]
#[ignore]
fn test_create_blank_aggregate_device() {
    // TODO: Test this when there is no available devices.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(&mut plugin_id, &mut aggregate_device_id).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

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

    assert!(audiounit_destroy_aggregate_device(plugin_id, &mut aggregate_device_id).is_ok());

    fn get_all_devices() -> Vec<AudioObjectID> {
        let mut size: usize = 0;
        let mut ret = audio_object_get_property_data_size(
            kAudioObjectSystemObject,
            &DEVICES_PROPERTY_ADDRESS,
            &mut size,
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
            devices.as_mut_ptr(),
        );
        if ret != NO_ERR {
            return Vec::new();
        }
        devices.sort();
        devices
    }
}

// set_aggregate_sub_device_list
// ------------------------------------
// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_set_aggregate_sub_device_list_for_unknown_input_output_devices() {
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(&mut plugin_id, &mut aggregate_device_id).is_ok()
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
        )
        .unwrap_err(),
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
            )
            .unwrap_err(),
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
            )
            .unwrap_err(),
            Error::error()
        );
    }

    assert!(audiounit_destroy_aggregate_device(plugin_id, &mut aggregate_device_id).is_ok());
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_set_aggregate_sub_device_list() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(input_id) || !valid_id(output_id)
    /* || input_id == output_id */
    {
        return;
    }

    let input_sub_devices = audiounit_get_sub_devices(input_id);
    let output_sub_devices = audiounit_get_sub_devices(output_id);

    // Create a blank aggregate device.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(&mut plugin_id, &mut aggregate_device_id).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Set sub devices for the created aggregate device.
    assert!(
        audiounit_set_aggregate_sub_device_list(aggregate_device_id, input_id, output_id).is_ok()
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

    assert!(audiounit_destroy_aggregate_device(plugin_id, &mut aggregate_device_id).is_ok());

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
// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_set_master_aggregate_device_for_a_blank_aggregate_device() {
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(&mut plugin_id, &mut aggregate_device_id).is_ok()
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
    assert!(audiounit_set_master_aggregate_device(aggregate_device_id).is_ok());

    // Make sure this blank aggregate device owns nothing.
    // TODO: it's really weird it actually own nothing but
    //       it can set master device successfully!
    let owned_sub_devices = get_onwed_devices(aggregate_device_id);
    assert!(owned_sub_devices.is_empty());

    // Check if master is nothing.
    let master_device = get_master_device(aggregate_device_id);
    assert!(master_device.is_empty());

    assert!(audiounit_destroy_aggregate_device(plugin_id, &mut aggregate_device_id).is_ok());
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_set_master_aggregate_device() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(input_id) || !valid_id(output_id)
    /* || input_id == output_id */
    {
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
        audiounit_create_blank_aggregate_device(&mut plugin_id, &mut aggregate_device_id).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Set the sub devices into the created aggregate device.
    assert!(
        audiounit_set_aggregate_sub_device_list(aggregate_device_id, input_id, output_id).is_ok()
    );

    // Set the master device.
    assert!(audiounit_set_master_aggregate_device(aggregate_device_id).is_ok());

    // Check if master is set to default output device.
    let master_device = get_master_device(aggregate_device_id);
    let default_output_device = to_device_name(output_id).unwrap();
    assert_eq!(master_device, default_output_device);

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
    assert!(audiounit_destroy_aggregate_device(plugin_id, &mut aggregate_device_id).is_ok());
}

fn get_master_device(aggregate_device_id: AudioObjectID) -> String {
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    let master_aggregate_sub_device = AudioObjectPropertyAddress {
        mSelector: kAudioAggregateDevicePropertyMasterSubDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
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
// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[should_panic]
#[ignore]
fn test_activate_clock_drift_compensation_for_a_blank_aggregate_device() {
    // Create a blank aggregate device.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(&mut plugin_id, &mut aggregate_device_id).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Get owned sub devices.
    let devices = get_onwed_devices(aggregate_device_id);
    assert!(devices.is_empty());

    // Get a panic since no sub devices to be set compensation.
    assert_eq!(
        audiounit_activate_clock_drift_compensation(aggregate_device_id).unwrap_err(),
        Error::error()
    );

    // Destroy the aggregate device. (The program cannot reach here.)
    assert!(audiounit_destroy_aggregate_device(plugin_id, &mut aggregate_device_id).is_ok());
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_activate_clock_drift_compensation_for_an_aggregate_device_without_master_device() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(input_id) || !valid_id(output_id)
    /* || input_id == output_id */
    {
        return;
    }

    // Create a blank aggregate device.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(&mut plugin_id, &mut aggregate_device_id).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Set the sub devices into the created aggregate device.
    assert!(
        audiounit_set_aggregate_sub_device_list(aggregate_device_id, input_id, output_id).is_ok()
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
    assert!(audiounit_activate_clock_drift_compensation(aggregate_device_id).is_ok());

    // Check the compensations.
    let devices = get_onwed_devices(aggregate_device_id);
    assert!(!devices.is_empty());
    let compensations = get_drift_compensations(&devices);
    assert!(!compensations.is_empty());
    assert_eq!(devices.len(), compensations.len());

    for (i, compensation) in compensations.iter().enumerate() {
        assert_eq!(*compensation, if i == 0 { 0 } else { 1 });
    }

    // Destroy the aggregate device.
    assert!(audiounit_destroy_aggregate_device(plugin_id, &mut aggregate_device_id).is_ok());
}

// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
fn test_activate_clock_drift_compensation() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(input_id) || !valid_id(output_id)
    /* || input_id == output_id */
    {
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
        audiounit_create_blank_aggregate_device(&mut plugin_id, &mut aggregate_device_id).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Set the sub devices into the created aggregate device.
    assert!(
        audiounit_set_aggregate_sub_device_list(aggregate_device_id, input_id, output_id).is_ok()
    );

    // Set the master device.
    assert!(audiounit_set_master_aggregate_device(aggregate_device_id).is_ok());

    // Set clock drift compensation.
    assert!(audiounit_activate_clock_drift_compensation(aggregate_device_id).is_ok());

    // Check the compensations.
    let devices = get_onwed_devices(aggregate_device_id);
    assert!(!devices.is_empty());
    let compensations = get_drift_compensations(&devices);
    assert!(!compensations.is_empty());
    assert_eq!(devices.len(), compensations.len());

    for (i, compensation) in compensations.iter().enumerate() {
        assert_eq!(*compensation, if i == 0 { 0 } else { 1 });
    }

    // Destroy the aggregate device.
    assert!(audiounit_destroy_aggregate_device(plugin_id, &mut aggregate_device_id).is_ok());
}

fn get_onwed_devices(aggregate_device_id: AudioDeviceID) -> Vec<AudioObjectID> {
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    let address_owned = AudioObjectPropertyAddress {
        mSelector: kAudioObjectPropertyOwnedObjects,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
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

fn get_drift_compensations(devices: &Vec<AudioObjectID>) -> Vec<u32> {
    assert!(!devices.is_empty());

    let address_drift = AudioObjectPropertyAddress {
        mSelector: kAudioSubDevicePropertyDriftCompensation,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

    let mut compensations = Vec::new();

    for device in devices {
        assert_ne!(*device, kAudioObjectUnknown);

        let mut size = mem::size_of::<u32>();
        let mut compensation = u32::max_value();

        assert_eq!(
            audio_object_get_property_data(*device, &address_drift, &mut size, &mut compensation),
            NO_ERR
        );

        compensations.push(compensation);
    }

    compensations
}

// destroy_aggregate_device
// ------------------------------------
// Ignore this by default. The reason is same as test_create_blank_aggregate_device.
#[test]
#[ignore]
#[should_panic]
fn test_destroy_aggregate_device_for_a_unknown_plugin_device() {
    // TODO: Test this when there is no available devices.
    let mut plugin_id = kAudioObjectUnknown;
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert!(
        audiounit_create_blank_aggregate_device(&mut plugin_id, &mut aggregate_device_id).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    assert_eq!(
        audiounit_destroy_aggregate_device(kAudioObjectUnknown, &mut aggregate_device_id)
            .unwrap_err(),
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
        audiounit_create_blank_aggregate_device(&mut plugin_id, &mut aggregate_device_id).is_ok()
    );
    assert_ne!(plugin_id, kAudioObjectUnknown);
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    aggregate_device_id = kAudioObjectUnknown;

    assert_eq!(
        audiounit_destroy_aggregate_device(plugin_id, &mut aggregate_device_id).unwrap_err(),
        Error::error()
    );
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

    let mut stream = AudioUnitStream::new(&mut ctx, ptr::null_mut(), None, None, 0);
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

    let mut stream = AudioUnitStream::new(&mut ctx, ptr::null_mut(), None, None, 0);
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

    let mut stream = AudioUnitStream::new(&mut ctx, ptr::null_mut(), None, None, 0);
    stream.init();

    // The scope of `_lock` is a critical section.
    // When `AudioUnitStream::drop()` is called, `AudioUnitContext.mutex`
    // needs to be unlocked. That's why `_lock` needs to be declared after
    // `stream` so it will be dropped and unlocked before dropping `stream`.
    let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });

    // Initialize the output unit to default output device.
    let device = device_info {
        id: kAudioObjectUnknown,
        flags: device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
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
            &mut mem::size_of_val(&buffer_size),
        ) == 0
        {
            Some(buffer_size)
        } else {
            None
        }
    };

    let default_output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);

    let range = 0..2 * SAFE_MAX_LATENCY_FRAMES;
    assert!(range.start < SAFE_MIN_LATENCY_FRAMES);
    // assert!(range.end < SAFE_MAX_LATENCY_FRAMES);
    for latency in range {
        let clamp = audiounit_clamp_latency(&mut stream, latency);
        assert_eq!(
            clamp,
            if valid_id(default_output_id) {
                clamp_values(if let Some(buffer_size) = maybe_buffer_size {
                    cmp::min(buffer_size, latency)
                } else {
                    latency
                })
            } else {
                0
            }
        );
    }

    fn clamp_values(value: u32) -> u32 {
        cmp::max(
            cmp::min(value, SAFE_MAX_LATENCY_FRAMES),
            SAFE_MIN_LATENCY_FRAMES,
        )
    }
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

    let mut stream = AudioUnitStream::new(&mut ctx, ptr::null_mut(), None, None, 0);
    stream.init();

    assert!(stream.input_unit.is_null());
    assert! {
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

    let mut stream = AudioUnitStream::new(&mut ctx, ptr::null_mut(), None, None, 0);
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

    assert!(audiounit_set_device_info(&mut stream, kAudioObjectUnknown, io_side::INPUT).is_ok());

    assert_eq!(stream.input_device.id, default_input_id);
    assert_eq!(
        stream.input_device.flags,
        device_flags::DEV_INPUT
            | device_flags::DEV_SELECTED_DEFAULT
            | device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(audiounit_create_unit(&mut stream.input_unit, &stream.input_device).is_ok());

    assert!(!stream.input_unit.is_null());

    assert_eq!(stream.latency_frames, 0);

    assert!(audiounit_configure_input(&mut stream).is_ok());

    assert_ne!(stream.input_hw_rate, 0_f64);

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
    assert_eq!(description.mSampleRate, stream.input_hw_rate);

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
    assert_ne!(stream.latency_frames, buffer_frames);

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
    assert_ne!(stream.latency_frames, frames_per_slice);
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

    let mut stream = AudioUnitStream::new(&mut ctx, ptr::null_mut(), None, None, 0);
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

    assert!(audiounit_set_device_info(&mut stream, kAudioObjectUnknown, io_side::INPUT).is_ok());

    assert_eq!(stream.input_device.id, default_input_id);
    assert_eq!(
        stream.input_device.flags,
        device_flags::DEV_INPUT
            | device_flags::DEV_SELECTED_DEFAULT
            | device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(audiounit_create_unit(&mut stream.input_unit, &stream.input_device).is_ok());

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

    assert!(audiounit_configure_input(&mut stream).is_ok());

    assert_ne!(stream.input_hw_rate, 0_f64);

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
    assert_eq!(description.mSampleRate, stream.input_hw_rate);

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
    assert_eq!(stream.latency_frames, buffer_frames);

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
    assert_eq!(stream.latency_frames, frames_per_slice);

    assert!(stream.input_linear_buffer.is_some());
    stream
        .input_linear_buffer
        .as_mut()
        .unwrap()
        .push(array.as_ptr() as *const c_void, array.len());

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

    let mut stream = AudioUnitStream::new(&mut ctx, ptr::null_mut(), None, None, 0);
    stream.init();

    assert!(stream.output_unit.is_null());
    assert! {
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

    let mut stream = AudioUnitStream::new(&mut ctx, ptr::null_mut(), None, None, 0);
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

    assert!(audiounit_set_device_info(&mut stream, kAudioObjectUnknown, io_side::OUTPUT).is_ok());

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT
            | device_flags::DEV_SELECTED_DEFAULT
            | device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(audiounit_create_unit(&mut stream.output_unit, &stream.output_device).is_ok());

    assert!(!stream.output_unit.is_null());

    assert_eq!(stream.latency_frames, 0);

    assert!(audiounit_configure_output(&mut stream).is_ok());

    assert_ne!(stream.output_hw_rate, 0_f64);

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
    assert_eq!(description.mSampleRate, stream.output_hw_rate);

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
    assert_ne!(stream.latency_frames, buffer_frames);

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
    assert_ne!(stream.latency_frames, frames_per_slice);
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

    let mut stream = AudioUnitStream::new(&mut ctx, ptr::null_mut(), None, None, 0);
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

    assert!(audiounit_set_device_info(&mut stream, kAudioObjectUnknown, io_side::OUTPUT).is_ok());

    assert_eq!(stream.output_device.id, default_output_id);
    assert_eq!(
        stream.output_device.flags,
        device_flags::DEV_OUTPUT
            | device_flags::DEV_SELECTED_DEFAULT
            | device_flags::DEV_SYSTEM_DEFAULT
    );

    assert!(audiounit_create_unit(&mut stream.output_unit, &stream.output_device).is_ok());

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

    assert!(audiounit_configure_output(&mut stream).is_ok());

    assert_ne!(stream.output_hw_rate, 0_f64);

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
    assert_eq!(description.mSampleRate, stream.output_hw_rate);

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
    assert_eq!(stream.latency_frames, buffer_frames);

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
    assert_eq!(stream.latency_frames, frames_per_slice);

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

// create_device_from_hwdev
// ------------------------------------
#[test]
fn test_create_device_from_hwdev_unknown() {
    let mut info = ffi::cubeb_device_info::default();

    // unknown
    assert_eq!(
        audiounit_create_device_from_hwdev(&mut info, kAudioObjectUnknown, DeviceType::UNKNOWN,)
            .unwrap_err(),
        Error::error()
    );

    // input
    assert_eq!(
        audiounit_create_device_from_hwdev(&mut info, kAudioObjectUnknown, DeviceType::INPUT,)
            .unwrap_err(),
        Error::error()
    );

    // output
    assert_eq!(
        audiounit_create_device_from_hwdev(&mut info, kAudioObjectUnknown, DeviceType::OUTPUT,)
            .unwrap_err(),
        Error::error()
    );

    // in-out
    assert_eq!(
        audiounit_create_device_from_hwdev(
            &mut info,
            kAudioObjectUnknown,
            DeviceType::INPUT | DeviceType::OUTPUT,
        )
        .unwrap_err(),
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
        audiounit_create_device_from_hwdev(&mut info, input_id, DeviceType::UNKNOWN,).unwrap_err(),
        Error::error()
    );

    // input
    info = ffi::cubeb_device_info::default();
    assert!(audiounit_create_device_from_hwdev(&mut info, input_id, DeviceType::INPUT,).is_ok());
    assert!(!info.devid.is_null());
    assert!(!info.device_id.is_null());
    assert_eq!(info.group_id, info.device_id);
    assert!(!info.friendly_name.is_null());
    // assert!(!info.vendor_name.is_null());
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
            audiounit_create_device_from_hwdev(&mut info, input_id, DeviceType::OUTPUT,).is_ok()
        );
        assert!(!info.devid.is_null());
        // assert!(info.device_id.is_null());
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
            audiounit_create_device_from_hwdev(&mut info, input_id, DeviceType::OUTPUT,)
                .unwrap_err(),
            Error::error()
        );
    }

    // in-out
    assert_eq!(
        audiounit_create_device_from_hwdev(
            &mut info,
            input_id,
            DeviceType::INPUT | DeviceType::OUTPUT,
        )
        .unwrap_err(),
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
        audiounit_create_device_from_hwdev(&mut info, output_id, DeviceType::UNKNOWN,).unwrap_err(),
        Error::error()
    );

    // input
    info = ffi::cubeb_device_info::default();
    if is_input(output_id) {
        assert!(
            audiounit_create_device_from_hwdev(&mut info, output_id, DeviceType::INPUT,).is_ok()
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
            audiounit_create_device_from_hwdev(&mut info, output_id, DeviceType::INPUT,)
                .unwrap_err(),
            Error::error()
        );
    }

    // output
    info = ffi::cubeb_device_info::default();
    assert!(audiounit_create_device_from_hwdev(&mut info, output_id, DeviceType::OUTPUT,).is_ok());
    assert!(!info.devid.is_null());
    assert!(!info.device_id.is_null());
    assert_eq!(info.group_id, info.device_id);
    assert!(!info.friendly_name.is_null());
    // assert!(!info.vendor_name.is_null());
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
        )
        .unwrap_err(),
        Error::error()
    );
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
            audiounit_add_device_listener(ctx_ptr, *devtype, None, ptr::null_mut()),
            0
        );
    }

    assert_eq!(ctx.input_collection_changed_callback, None);

    assert_eq!(ctx.output_collection_changed_callback, None);

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
        ptr::null_mut(),
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
        DeviceType::INPUT | DeviceType::OUTPUT,
    ]
    .iter()
    {
        assert!(ctx.input_collection_changed_callback.is_none());
        assert!(ctx.output_collection_changed_callback.is_none());

        // Register a callback within a specific scope.
        assert_eq!(
            audiounit_add_device_listener(ctx_ptr, *devtype, Some(callback), ptr::null_mut()),
            0
        );

        // TODO: It doesn't work, but the return value is ok.
        assert_eq!(
            audiounit_remove_device_listener(ctx_ptr, DeviceType::UNKNOWN),
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
            audiounit_remove_device_listener(ctx_ptr, DeviceType::INPUT | DeviceType::OUTPUT,),
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

    let mut map: HashMap<DeviceType, extern "C" fn(*mut ffi::cubeb, *mut c_void)> = HashMap::new();
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
            audiounit_add_device_listener(ctx_ptr, *devtype, Some(*callback), ptr::null_mut()),
            0
        );

        // TODO: It doesn't work, but the return value is ok.
        assert_eq!(
            audiounit_remove_device_listener(ctx_ptr, DeviceType::UNKNOWN),
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
            audiounit_remove_device_listener(ctx_ptr, DeviceType::INPUT | DeviceType::OUTPUT),
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
        assert_eq!(audiounit_remove_device_listener(ctx_ptr, *devtype), 0);
    }
}

#[test]
fn test_remove_device_listeners_within_all_scopes() {
    use std::collections::HashMap;

    extern "C" fn inout_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    extern "C" fn in_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    extern "C" fn out_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    let mut map: HashMap<DeviceType, extern "C" fn(*mut ffi::cubeb, *mut c_void)> = HashMap::new();
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
            audiounit_add_device_listener(ctx_ptr, *devtype, Some(*callback), ptr::null_mut()),
            0
        );

        // TODO: It doesn't work, but the return value is ok.
        assert_eq!(
            audiounit_remove_device_listener(ctx_ptr, DeviceType::UNKNOWN),
            0
        );

        assert_eq!(
            audiounit_remove_device_listener(ctx_ptr, DeviceType::INPUT | DeviceType::OUTPUT),
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
        DeviceType::INPUT | DeviceType::OUTPUT,
    ]
    .iter()
    {
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
        assert_eq!(audiounit_remove_device_listener(ctx_ptr, *devtype,), 0);

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
            audiounit_remove_device_listener(ctx_ptr, DeviceType::INPUT | DeviceType::OUTPUT,),
            0
        );
    }
}

#[test]
fn test_remove_device_listeners_dont_affect_other_scopes_with_different_callbacks() {
    use std::collections::HashMap;

    extern "C" fn in_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    extern "C" fn out_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    let mut map: HashMap<DeviceType, extern "C" fn(*mut ffi::cubeb, *mut c_void)> = HashMap::new();
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
                audiounit_add_device_listener(ctx_ptr, *scope, Some(*listener), ptr::null_mut()),
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
        assert_eq!(audiounit_remove_device_listener(ctx_ptr, *devtype,), 0);

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
            audiounit_remove_device_listener(ctx_ptr, DeviceType::INPUT | DeviceType::OUTPUT,),
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
            if is_input {
                kAudioUnitScope_Input
            } else {
                kAudioUnitScope_Output
            },
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
        names.push(to_device_name(*device));
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