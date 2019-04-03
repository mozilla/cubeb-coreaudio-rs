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

// Private APIs
// ============================================================================
// get_sub_devices
// ------------------------------------
// Ignore this by default. The reason is same as below.
#[test]
#[ignore]
fn test_aggregate_get_sub_devices_for_blank_aggregate_devices() {
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
// - test_aggregate_get_sub_devices_for_blank_aggregate_devices
// - test_create_blank_aggregate_device
// - test_aggregate_set_aggregate_sub_device_list_for_unknown_input_output_devices
// - test_aggregate_set_aggregate_sub_device_list
// - test_aggregate_set_master_aggregate_device_for_a_blank_aggregate_device
// - test_aggregate_set_master_aggregate_device
// - test_aggregate_activate_clock_drift_compensation_for_an_aggregate_device_without_master_device
// - test_aggregate_activate_clock_drift_compensation
//
// The above tests are added a prefix `test_aggregate` so we can run these ignored tests easily on
// an indivisual test command, rather than run these tests with others together.
//
// TODO: Find out why `test_create_blank_aggregate_device` cannot be run with others.
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
fn test_aggregate_set_aggregate_sub_device_list_for_unknown_input_output_devices() {
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
fn test_aggregate_set_aggregate_sub_device_list() {
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
fn test_aggregate_set_master_aggregate_device_for_a_blank_aggregate_device() {
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
fn test_aggregate_set_master_aggregate_device() {
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
fn test_aggregate_activate_clock_drift_compensation_for_a_blank_aggregate_device() {
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
fn test_aggregate_activate_clock_drift_compensation_for_an_aggregate_device_without_master_device() {
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
fn test_aggregate_activate_clock_drift_compensation() {
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
fn test_aggregate_destroy_aggregate_device_for_a_unknown_plugin_device() {
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
fn test_aggregate_destroy_aggregate_device_for_a_unknown_aggregate_device() {
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

// Utils
// ------------------------------------
fn valid_id(id: AudioObjectID) -> bool {
    id != kAudioObjectUnknown
}

fn is_output(id: AudioObjectID) -> bool {
    audiounit_get_channel_count(id, kAudioDevicePropertyScopeOutput) > 0
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
