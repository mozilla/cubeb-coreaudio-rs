use super::utils::{
    test_get_all_devices, test_get_all_onwed_devices, test_get_default_device,
    test_get_master_device, Scope,
};
use super::*;

// These tests that calling `AggregateDevice::create_blank_device` are marked `ignore` by
// default since they cannot run with those tests calling `audiounit_add_device_listener`
// directly or indirectly (via `register_device_collection_changed`) at the same time.
//
// The `audiounit_collection_changed_callback` will be fired upon
// `AggregateDevice::create_blank_device` is called.
// In `audiounit_collection_changed_callback`, it will register an asynchronous function to notify
// the device-collection is changed.
//
// In current implementation, those asynchronous functions might cause the following errors:
//
// 1. If those tests calling `audiounit_add_device_listener` is finished before those
//    asynchronous functions fired by `audiounit_collection_changed_callback` start executing,
//    without unregistering the callback by `audiounit_remove_device_listener`, when those
//    asynchronous functions are executed, their pointers to those contexts declared in the tests
//    are already destroyed. So we will get a EXC_BAD_ACCESS error when we try dereferencing the
//    pointers pointing to a destroyed context.
//
//    One example is to run `test_context_register_device_collection_changed_twice` at the same
//    time with other tests that initialize a stream for both input and output(this will create an
//    aggregate device and fire `audiounit_collection_changed_callback` indirectly, see the comment
//    in `AggregateDevice::create_blank_device` and `test_stream_set_panning`).
//
//    A simple way to verify this is to add a log at the beginning
//    `audiounit_collection_changed_callback` and a log in `AudioUnitContext::drop`. You will get
//    this error when `audiounit_collection_changed_callback` is called after the AudioUnitContext
//    is dropped.
//
// 2. If those tests calling `audiounit_add_device_listener` is finished between the time after
//    those asynchronous functions are executed but before those asynchronous functions are
//    finished, those tests will try destroying the contexts that are currently locked by those
//    asynchronous functions. Thus, we will get panics in `OwnedCriticalSection::drop/destroy`
//    since `pthread_mutex_destroy` returns `EBUSY(16)` rather than 0.
//
//    Theoretically, this could happen when the operations are executed in the following order:
//    1. Create an AudioUnitContext `ctx`
//    2. Register device-collection changed for `ctx`
//    3. Initialize an AudioUnitStream `stm` within `ctx` for both input and output. It will
//       create an aggregate device and fire the `audiounit_collection_changed_callback`
//       indirectly. In the `audiounit_collection_changed_callback`, it will dispatch an
//       asynchronous task that will lock the `ctx`
//    4. The asynchronous task starts runnning and lock the `ctx`
//    5. `ctx` is destroyed while the asynchronous task is running, before the asynchronous task
//       is finished, we will get a fail for destroying a locked `ctx`
//
//    A simple way to verify this is to add two logs at the beginning and the end of
//    `async_dispatch` in `audiounit_collection_changed_callback` and two logs at the beginning
//    and the end of the tests calling `audiounit_add_device_listener`. You will find those tests
//    fail when the tests are ended while those asynchronous functions are still running.
//
// The tests that call `AggregateDevice::create_blank_device` are ignored by default:
// - test_aggregate_get_sub_devices_for_blank_aggregate_devices
// - test_create_blank_device
// - test_aggregate_set_sub_devices_for_unknown_input_output_devices
// - test_aggregate_set_sub_devices
// - test_aggregate_set_master_device_for_a_blank_aggregate_device
// - test_aggregate_set_master_device
// - test_aggregate_activate_clock_drift_compensation_for_an_aggregate_device_without_master_device
// - test_aggregate_activate_clock_drift_compensation
//
// The above tests are added a prefix `test_aggregate` so we can run these ignored tests easily on
// an indivisual test command, rather than run these tests with others together.

// AggregateDevice::create_blank_device_sync
// ------------------------------------
#[test]
#[ignore]
fn test_aggregate_create_blank_device() {
    // TODO: Test this when there is no available devices.
    let plugin = AggregateDevice::get_system_plugin_id().unwrap();
    let device = AggregateDevice::create_blank_device_sync(plugin).unwrap();
    let devices = test_get_all_devices();
    let device = devices.into_iter().find(|dev| dev == &device).unwrap();
    let uid = get_device_global_uid(device).unwrap().into_string();
    assert!(uid.contains(PRIVATE_AGGREGATE_DEVICE_NAME));
}

// AggregateDevice::get_sub_devices
// ------------------------------------
#[test]
#[ignore]
#[should_panic]
fn test_aggregate_get_sub_devices_for_blank_aggregate_devices() {
    // TODO: Test this when there is no available devices.
    let plugin = AggregateDevice::get_system_plugin_id().unwrap();
    let device = AggregateDevice::create_blank_device_sync(plugin).unwrap();
    // There is no sub device in a blank aggregate device!
    // AggregateDevice::get_sub_devices guarantees returning a non-empty devices vector, so
    // the following call will panic!
    let sub_devices = AggregateDevice::get_sub_devices(device).unwrap();
    assert!(sub_devices.is_empty());
    assert!(AggregateDevice::destroy_device(plugin, device).is_ok());
}

// AggregateDevice::set_sub_devices_sync
// ------------------------------------
#[test]
#[ignore]
fn test_aggregate_set_sub_devices() {
    let input_device = test_get_default_device(Scope::Input);
    let output_device = test_get_default_device(Scope::Output);
    if input_device.is_none() || output_device.is_none() || input_device == output_device {
        println!("No input or output device to create an aggregate device.");
        return;
    }

    let input_device = input_device.unwrap();
    let output_device = output_device.unwrap();

    let plugin = AggregateDevice::get_system_plugin_id().unwrap();
    let device = AggregateDevice::create_blank_device_sync(plugin).unwrap();
    assert!(AggregateDevice::set_sub_devices_sync(device, input_device, output_device).is_ok());

    let sub_devices = AggregateDevice::get_sub_devices(device).unwrap();
    let input_sub_devices = AggregateDevice::get_sub_devices(input_device).unwrap();
    let output_sub_devices = AggregateDevice::get_sub_devices(output_device).unwrap();

    // TODO: There may be overlapping devices between input_sub_devices and output_sub_devices,
    //       but now AggregateDevice::set_sub_devices will add them directly.
    assert_eq!(
        sub_devices.len(),
        input_sub_devices.len() + output_sub_devices.len()
    );
    for dev in &input_sub_devices {
        assert!(sub_devices.contains(dev));
    }
    for dev in &output_sub_devices {
        assert!(sub_devices.contains(dev));
    }

    let onwed_devices = test_get_all_onwed_devices(device);
    let onwed_device_uids = get_device_uids(&onwed_devices);
    let input_sub_device_uids = get_device_uids(&input_sub_devices);
    let output_sub_device_uids = get_device_uids(&output_sub_devices);
    for uid in &input_sub_device_uids {
        assert!(onwed_device_uids.contains(uid));
    }
    for uid in &output_sub_device_uids {
        assert!(onwed_device_uids.contains(uid));
    }

    assert!(AggregateDevice::destroy_device(plugin, device).is_ok());
}

#[test]
#[ignore]
#[should_panic]
fn test_aggregate_set_sub_devices_for_unknown_input_devices() {
    let output_device = test_get_default_device(Scope::Output);
    if output_device.is_none() {
        panic!("Need a output device for the test!");
    }
    let output_device = output_device.unwrap();

    let plugin = AggregateDevice::get_system_plugin_id().unwrap();
    let device = AggregateDevice::create_blank_device_sync(plugin).unwrap();

    assert!(AggregateDevice::set_sub_devices(device, kAudioObjectUnknown, output_device).is_err());

    assert!(AggregateDevice::destroy_device(plugin, device).is_ok());
}

#[test]
#[ignore]
#[should_panic]
fn test_aggregate_set_sub_devices_for_unknown_output_devices() {
    let input_device = test_get_default_device(Scope::Input);
    if input_device.is_none() {
        panic!("Need a input device for the test!");
    }
    let input_device = input_device.unwrap();

    let plugin = AggregateDevice::get_system_plugin_id().unwrap();
    let device = AggregateDevice::create_blank_device_sync(plugin).unwrap();

    assert!(AggregateDevice::set_sub_devices(device, input_device, kAudioObjectUnknown).is_err());

    assert!(AggregateDevice::destroy_device(plugin, device).is_ok());
}

fn get_device_uids(devices: &Vec<AudioObjectID>) -> Vec<String> {
    devices
        .iter()
        .map(|device| get_device_global_uid(*device).unwrap().into_string())
        .collect()
}

// AggregateDevice::set_master_device
// ------------------------------------
#[test]
#[ignore]
fn test_aggregate_set_master_device() {
    let input_device = test_get_default_device(Scope::Input);
    let output_device = test_get_default_device(Scope::Output);
    if input_device.is_none() || output_device.is_none() || input_device == output_device {
        println!("No input or output device to create an aggregate device.");
        return;
    }

    let input_device = input_device.unwrap();
    let output_device = output_device.unwrap();

    let plugin = AggregateDevice::get_system_plugin_id().unwrap();
    let device = AggregateDevice::create_blank_device_sync(plugin).unwrap();
    assert!(AggregateDevice::set_sub_devices_sync(device, input_device, output_device).is_ok());
    assert!(AggregateDevice::set_master_device(device).is_ok());

    // Check if master is set to the first sub device of the default output device.
    // TODO: What if the output device in the aggregate device is not the default output device?
    let first_output_sub_device_uid =
        get_device_uid(AggregateDevice::get_sub_devices(device).unwrap()[0]);
    let master_device_uid = test_get_master_device(device);
    assert_eq!(first_output_sub_device_uid, master_device_uid);
}

#[test]
#[ignore]
fn test_aggregate_set_master_device_for_a_blank_aggregate_device() {
    let output_device = test_get_default_device(Scope::Output);
    if output_device.is_none() {
        println!("No output device to test.");
        return;
    }

    let plugin = AggregateDevice::get_system_plugin_id().unwrap();
    let device = AggregateDevice::create_blank_device_sync(plugin).unwrap();
    assert!(AggregateDevice::set_master_device(device).is_ok());

    // TODO: it's really weird the aggregate device actually own nothing
    //       but its master device can be set successfully!
    // The sub devices of this blank aggregate device (by `AggregateDevice::get_sub_devices`)
    // and the own devices (by `test_get_all_onwed_devices`) is empty since the size returned
    // from `audio_object_get_property_data_size` is 0.
    // The CFStringRef of the master device returned from `test_get_master_device` is actually
    // non-null.

    assert!(AggregateDevice::destroy_device(plugin, device).is_ok());
}

fn get_device_uid(id: AudioObjectID) -> String {
    get_device_global_uid(id).unwrap().into_string()
}

// activate_clock_drift_compensation
// ------------------------------------
#[test]
#[should_panic]
#[ignore]
fn test_aggregate_activate_clock_drift_compensation_for_a_blank_aggregate_device() {
    // Create a blank aggregate device.
    let plugin_id = AggregateDevice::get_system_plugin_id().unwrap();
    assert_ne!(plugin_id, kAudioObjectUnknown);
    let aggregate_device_id = AggregateDevice::create_blank_device_sync(plugin_id).unwrap();
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Get owned sub devices.
    let devices = get_onwed_devices(aggregate_device_id);
    assert!(devices.is_empty());

    // Get a panic since no sub devices to be set compensation.
    assert!(AggregateDevice::activate_clock_drift_compensation(aggregate_device_id).is_err());

    // Destroy the aggregate device. (The program cannot reach here.)
    assert!(AggregateDevice::destroy_device(plugin_id, aggregate_device_id).is_ok());
}

#[test]
#[ignore]
fn test_aggregate_activate_clock_drift_compensation_for_an_aggregate_device_without_master_device()
{
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(input_id) || !valid_id(output_id) || input_id == output_id {
        return;
    }

    // Create a blank aggregate device.
    let plugin_id = AggregateDevice::get_system_plugin_id().unwrap();
    assert_ne!(plugin_id, kAudioObjectUnknown);
    let aggregate_device_id = AggregateDevice::create_blank_device_sync(plugin_id).unwrap();
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Set the sub devices into the created aggregate device.
    assert!(AggregateDevice::set_sub_devices(aggregate_device_id, input_id, output_id).is_ok());

    // TODO: Is the master device the first output sub device by default if we
    //       don't set that ? Is it because we add the output sub device list
    //       before the input's one ? (See implementation of
    //       AggregateDevice::set_sub_devices).
    // TODO: Does this check work if output_id is an aggregate device ?
    assert_eq!(
        get_master_device(aggregate_device_id),
        to_device_name(output_id).unwrap()
    );

    // Set clock drift compensation.
    assert!(AggregateDevice::activate_clock_drift_compensation(aggregate_device_id).is_ok());

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
    assert!(AggregateDevice::destroy_device(plugin_id, aggregate_device_id).is_ok());
}

#[test]
#[ignore]
fn test_aggregate_activate_clock_drift_compensation() {
    let input_id = audiounit_get_default_device_id(DeviceType::INPUT);
    let output_id = audiounit_get_default_device_id(DeviceType::OUTPUT);
    if !valid_id(input_id) || !valid_id(output_id) || input_id == output_id {
        return;
    }

    let output_sub_devices = AggregateDevice::get_sub_devices(output_id).unwrap();
    if output_sub_devices.is_empty() {
        return;
    }

    // Create a blank aggregate device.
    let plugin_id = AggregateDevice::get_system_plugin_id().unwrap();
    assert_ne!(plugin_id, kAudioObjectUnknown);
    let aggregate_device_id = AggregateDevice::create_blank_device_sync(plugin_id).unwrap();
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    // Set the sub devices into the created aggregate device.
    assert!(AggregateDevice::set_sub_devices(aggregate_device_id, input_id, output_id).is_ok());

    // Set the master device.
    assert!(AggregateDevice::set_master_device(aggregate_device_id).is_ok());

    // Set clock drift compensation.
    assert!(AggregateDevice::activate_clock_drift_compensation(aggregate_device_id).is_ok());

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
    assert!(AggregateDevice::destroy_device(plugin_id, aggregate_device_id).is_ok());
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
#[test]
#[ignore]
#[should_panic]
fn test_aggregate_destroy_aggregate_device_for_a_unknown_plugin_device() {
    // TODO: Test this when there is no available devices.
    let plugin_id = AggregateDevice::get_system_plugin_id().unwrap();
    assert_ne!(plugin_id, kAudioObjectUnknown);
    let aggregate_device_id = AggregateDevice::create_blank_device_sync(plugin_id).unwrap();
    assert_ne!(aggregate_device_id, kAudioObjectUnknown);

    assert!(AggregateDevice::destroy_device(kAudioObjectUnknown, aggregate_device_id).is_err());
}

#[test]
#[ignore]
#[should_panic]
fn test_aggregate_destroy_aggregate_device_for_a_unknown_aggregate_device() {
    // TODO: Test this when there is no available devices.
    let plugin_id = AggregateDevice::get_system_plugin_id().unwrap();
    assert_ne!(plugin_id, kAudioObjectUnknown);
    let aggregate_device_id = kAudioObjectUnknown;
    assert!(AggregateDevice::destroy_device(plugin_id, aggregate_device_id).is_err());
}

// Utils
// ------------------------------------
fn valid_id(id: AudioObjectID) -> bool {
    id != kAudioObjectUnknown
}

fn to_device_name(id: AudioObjectID) -> Option<String> {
    let uid = get_device_global_uid(id).unwrap();
    Some(uid.into_string())
}

fn strref_to_string(strref: CFStringRef) -> String {
    let cstring = cfstringref_to_cstring(strref);
    cstring.into_string().unwrap()
}

fn cfstringref_to_cstring(strref: CFStringRef) -> CString {
    use std::os::raw::c_char;

    let empty = CString::default();
    if strref.is_null() {
        return empty;
    }

    let len = unsafe { CFStringGetLength(strref) };
    // Add 1 to size to allow for '\0' termination character.
    let size = unsafe { CFStringGetMaximumSizeForEncoding(len, kCFStringEncodingUTF8) + 1 };
    let mut buffer = vec![b'\x00'; size as usize];

    let success = unsafe {
        CFStringGetCString(
            strref,
            buffer.as_mut_ptr() as *mut c_char,
            size,
            kCFStringEncodingUTF8,
        ) != 0
    };
    if !success {
        buffer.clear();
        return empty;
    }

    // CString::new() will consume the input bytes vec and add a '\0' at the
    // end of the bytes. We need to remove the '\0' from the bytes data
    // returned from CFStringGetCString by ourselves to avoid memory leaks.
    // The size returned from CFStringGetMaximumSizeForEncoding is always
    // greater than or equal to the string length, where the string length
    // is the number of characters from the beginning to nul-terminator('\0'),
    // so we should shrink the string vector to fit that size.
    let str_len = unsafe { libc::strlen(buffer.as_ptr() as *mut c_char) };
    buffer.truncate(str_len); // Drop the elements from '\0'(including '\0').

    CString::new(buffer).unwrap_or(empty)
}
