use super::*;

// Utils
// ------------------------------------------------------------------------------------------------
#[derive(PartialEq)]
enum Scope {
    Input,
    Output,
}

fn get_default_device_id(scope: Scope) -> Option<AudioObjectID> {
    let address = AudioObjectPropertyAddress {
        mSelector: if scope == Scope::Input {
            kAudioHardwarePropertyDefaultInputDevice
        } else {
            kAudioHardwarePropertyDefaultOutputDevice
        },
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

    let mut devid: AudioDeviceID = kAudioObjectUnknown;
    let mut size = mem::size_of::<AudioDeviceID>();
    let status = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &address,
            0,
            ptr::null(),
            &mut size as *mut usize as *mut UInt32,
            &mut devid as *mut AudioDeviceID as *mut c_void,
        )
    };
    if status != NO_ERR || devid == kAudioObjectUnknown {
        return None;
    }
    Some(devid)
}

mod interfaces;
