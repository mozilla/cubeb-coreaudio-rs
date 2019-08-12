use super::utils::{test_get_default_device, Scope};
use super::*;

// get_device_global_uid
// ------------------------------------
#[test]
fn test_get_device_global_uid() {
    // Input device.
    if let Some(input) = test_get_default_device(Scope::Input) {
        let uid = get_device_global_uid(input).unwrap();
        let uid = uid.into_string();
        assert!(!uid.is_empty());
    }

    // Output device.
    if let Some(output) = test_get_default_device(Scope::Output) {
        let uid = get_device_global_uid(output).unwrap();
        let uid = uid.into_string();
        assert!(!uid.is_empty());
    }
}

#[test]
#[should_panic]
fn test_get_device_global_uid_by_unknwon_device() {
    // Unknown device.
    assert!(get_device_global_uid(kAudioObjectUnknown).is_err());
}

// get_device_uid
// ------------------------------------
#[test]
fn test_get_device_uid() {
    // Input device.
    if let Some(input) = test_get_default_device(Scope::Input) {
        let uid = get_device_uid(input, DeviceType::INPUT).unwrap();
        let uid = uid.into_string();
        assert!(!uid.is_empty());
    }

    // Output device.
    if let Some(output) = test_get_default_device(Scope::Output) {
        let uid = get_device_uid(output, DeviceType::OUTPUT).unwrap();
        let uid = uid.into_string();
        assert!(!uid.is_empty());
    }
}

#[test]
#[should_panic]
fn test_get_device_uid_by_unknwon_device() {
    // Unknown device.
    assert!(get_device_uid(kAudioObjectUnknown, DeviceType::INPUT).is_err());
}

// get_device_source
// ------------------------------------
// Some USB headsets (e.g., Plantronic .Audio 628) fails to get data source.
#[test]
fn test_get_device_source() {
    if let Some(device) = test_get_default_device(Scope::Input) {
        if let Ok(source) = get_device_source(device, DeviceType::INPUT) {
            println!(
                "input: {:X}, {:?}",
                source,
                convert_uint32_into_string(source)
            );
        } else {
            println!("No input data source.");
        }
    } else {
        println!("No input device.");
    }

    if let Some(device) = test_get_default_device(Scope::Output) {
        if let Ok(source) = get_device_source(device, DeviceType::OUTPUT) {
            println!(
                "output: {:X}, {:?}",
                source,
                convert_uint32_into_string(source)
            );
        } else {
            println!("No output data source.");
        }
    } else {
        println!("No output device.");
    }
}

#[test]
#[should_panic]
fn test_get_device_source_by_unknown_device() {
    assert!(get_device_source(kAudioObjectUnknown, DeviceType::INPUT).is_err());
}

// get_device_source_name
// ------------------------------------
#[test]
fn test_get_device_source_name() {
    if let Some(device) = test_get_default_device(Scope::Input) {
        if let Ok(name) = get_device_source_name(device, DeviceType::INPUT) {
            println!("input: {}", name.into_string());
        } else {
            println!("No input data source name.");
        }
    } else {
        println!("No input device.");
    }

    if let Some(device) = test_get_default_device(Scope::Output) {
        if let Ok(name) = get_device_source_name(device, DeviceType::OUTPUT) {
            println!("output: {}", name.into_string());
        } else {
            println!("No output data source name.");
        }
    } else {
        println!("No output device.");
    }
}

#[test]
#[should_panic]
fn test_get_device_source_name_by_unknown_device() {
    assert!(get_device_source_name(kAudioObjectUnknown, DeviceType::INPUT).is_err());
}

// get_device_name
// ------------------------------------
#[test]
fn test_get_device_name() {
    if let Some(device) = test_get_default_device(Scope::Input) {
        let name = get_device_name(device, DeviceType::INPUT).unwrap();
        println!("input device name: {}", name.into_string());
    } else {
        println!("No input device.");
    }

    if let Some(device) = test_get_default_device(Scope::Output) {
        let name = get_device_name(device, DeviceType::OUTPUT).unwrap();
        println!("output device name: {}", name.into_string());
    } else {
        println!("No output device.");
    }
}

#[test]
#[should_panic]
fn test_get_device_name_by_unknown_device() {
    assert!(get_device_name(kAudioObjectUnknown, DeviceType::INPUT).is_err());
}

// get_device_label
// ------------------------------------
#[test]
fn test_get_device_label() {
    if let Some(device) = test_get_default_device(Scope::Input) {
        let name = get_device_label(device, DeviceType::INPUT).unwrap();
        println!("input device label: {}", name.into_string());
    } else {
        println!("No input device.");
    }

    if let Some(device) = test_get_default_device(Scope::Output) {
        let name = get_device_label(device, DeviceType::OUTPUT).unwrap();
        println!("output device label: {}", name.into_string());
    } else {
        println!("No output device.");
    }
}

#[test]
#[should_panic]
fn test_get_device_label_by_unknown_device() {
    assert!(get_device_label(kAudioObjectUnknown, DeviceType::INPUT).is_err());
}

// get_device_manufacturer
// ------------------------------------
#[test]
fn test_get_device_manufacturer() {
    if let Some(device) = test_get_default_device(Scope::Input) {
        // Some devices like AirPods cannot get the vendor info so we print the error directly.
        // TODO: Replace `map` and `unwrap_or_else` by `map_or_else`
        let name = get_device_manufacturer(device, DeviceType::INPUT)
            .map(|name| name.into_string())
            .unwrap_or_else(|e| format!("Error: {}", e));
        println!("input device vendor: {}", name);
    } else {
        println!("No input device.");
    }

    if let Some(device) = test_get_default_device(Scope::Output) {
        // Some devices like AirPods cannot get the vendor info so we print the error directly.
        // TODO: Replace `map` and `unwrap_or_else` by `map_or_else`
        let name = get_device_manufacturer(device, DeviceType::OUTPUT)
            .map(|name| name.into_string())
            .unwrap_or_else(|e| format!("Error: {}", e));
        println!("output device vendor: {}", name);
    } else {
        println!("No output device.");
    }
}

#[test]
#[should_panic]
fn test_get_device_manufacturer_by_unknown_device() {
    assert!(get_device_manufacturer(kAudioObjectUnknown, DeviceType::INPUT).is_err());
}

// get_device_buffer_frame_size_range
// ------------------------------------
#[test]
fn test_get_device_buffer_frame_size_range() {
    if let Some(device) = test_get_default_device(Scope::Input) {
        let (min, max) = get_device_buffer_frame_size_range(device, DeviceType::INPUT).unwrap();
        println!("range of input buffer frame size: {}-{}", min, max);
    } else {
        println!("No input device.");
    }

    if let Some(device) = test_get_default_device(Scope::Output) {
        let (min, max) = get_device_buffer_frame_size_range(device, DeviceType::OUTPUT).unwrap();
        println!("range of output buffer frame size: {}-{}", min, max);
    } else {
        println!("No output device.");
    }
}

#[test]
#[should_panic]
fn test_get_device_buffer_frame_size_range_by_unknown_device() {
    assert!(get_device_buffer_frame_size_range(kAudioObjectUnknown, DeviceType::INPUT).is_err());
}
