use super::*;

pub fn get_device_global_uid(id: AudioDeviceID) -> Result<CFStringRef> {
    get_device_uid(id, DeviceType::INPUT | DeviceType::OUTPUT)
}

pub fn get_device_uid(id: AudioDeviceID, devtype: DeviceType) -> Result<CFStringRef> {
    assert_ne!(id, kAudioObjectUnknown);

    let mut size = mem::size_of::<CFStringRef>();
    let mut uid: CFStringRef = ptr::null();

    const GLOBAL: ffi::cubeb_device_type =
        ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT;
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyDeviceUID,
        mScope: match devtype.bits() {
            ffi::CUBEB_DEVICE_TYPE_INPUT => kAudioDevicePropertyScopeInput,
            ffi::CUBEB_DEVICE_TYPE_OUTPUT => kAudioDevicePropertyScopeOutput,
            GLOBAL => kAudioObjectPropertyScopeGlobal,
            _ => panic!("Invalid type"),
        },
        mElement: kAudioObjectPropertyElementMaster,
    };
    let err = audio_object_get_property_data(id, &address, &mut size, &mut uid);
    if err == NO_ERR {
        Ok(uid)
    } else {
        Err(Error::error())
    }
}

pub fn get_device_source(id: AudioDeviceID, devtype: DeviceType) -> Result<u32> {
    assert_ne!(id, kAudioObjectUnknown);

    let mut size = mem::size_of::<u32>();
    let mut source: u32 = 0;

    const GLOBAL: ffi::cubeb_device_type =
        ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT;
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyDataSource,
        mScope: match devtype.bits() {
            ffi::CUBEB_DEVICE_TYPE_INPUT => kAudioDevicePropertyScopeInput,
            ffi::CUBEB_DEVICE_TYPE_OUTPUT => kAudioDevicePropertyScopeOutput,
            GLOBAL => kAudioObjectPropertyScopeGlobal,
            _ => panic!("Invalid type"),
        },
        mElement: kAudioObjectPropertyElementMaster,
    };
    let err = audio_object_get_property_data(id, &address, &mut size, &mut source);
    if err == NO_ERR {
        Ok(source)
    } else {
        Err(Error::error())
    }
}

pub fn get_device_source_name(id: AudioDeviceID, devtype: DeviceType) -> Result<CFStringRef> {
    assert_ne!(id, kAudioObjectUnknown);

    let mut source: u32 = get_device_source(id, devtype)?;
    let mut size = mem::size_of::<AudioValueTranslation>();
    let mut name: CFStringRef = ptr::null();
    let mut trl = AudioValueTranslation {
        mInputData: &mut source as *mut u32 as *mut c_void,
        mInputDataSize: mem::size_of::<u32>() as u32,
        mOutputData: &mut name as *mut CFStringRef as *mut c_void,
        mOutputDataSize: mem::size_of::<CFStringRef>() as u32,
    };

    const GLOBAL: ffi::cubeb_device_type =
        ffi::CUBEB_DEVICE_TYPE_INPUT | ffi::CUBEB_DEVICE_TYPE_OUTPUT;
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyDataSourceNameForIDCFString,
        mScope: match devtype.bits() {
            ffi::CUBEB_DEVICE_TYPE_INPUT => kAudioDevicePropertyScopeInput,
            ffi::CUBEB_DEVICE_TYPE_OUTPUT => kAudioDevicePropertyScopeOutput,
            GLOBAL => kAudioObjectPropertyScopeGlobal,
            _ => panic!("Invalid type"),
        },
        mElement: kAudioObjectPropertyElementMaster,
    };

    let err = audio_object_get_property_data(id, &address, &mut size, &mut trl);
    if err == NO_ERR {
        Ok(name)
    } else {
        Err(Error::error())
    }
}
