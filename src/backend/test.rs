use super::*;

// Implementation
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
