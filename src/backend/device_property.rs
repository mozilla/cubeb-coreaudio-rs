use super::*;

pub fn get_device_uid(id: AudioDeviceID) -> Result<CFStringRef> {
    assert_ne!(id, kAudioObjectUnknown);

    let mut size = mem::size_of::<CFStringRef>();
    let mut uid: CFStringRef = ptr::null();
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyDeviceUID,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };
    let err = audio_object_get_property_data(id, &address, &mut size, &mut uid);
    if err == NO_ERR {
        Ok(uid)
    } else {
        Err(Error::error())
    }
}
