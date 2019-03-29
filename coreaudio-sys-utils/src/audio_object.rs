use coreaudio_sys::*;
use std::os::raw::c_void;
use std::ptr;

pub fn audio_object_has_property(id: AudioObjectID, address: &AudioObjectPropertyAddress) -> bool {
    unsafe { AudioObjectHasProperty(id, address) != 0 }
}

pub fn audio_object_get_property_data<T>(
    id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
    size: *mut usize,
    data: *mut T,
) -> OSStatus {
    unsafe {
        AudioObjectGetPropertyData(
            id,
            address,
            0,
            ptr::null(),
            size as *mut UInt32,
            data as *mut c_void,
        )
    }
}

pub fn audio_object_get_property_data_size(
    id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
    size: *mut usize,
) -> OSStatus {
    unsafe { AudioObjectGetPropertyDataSize(id, address, 0, ptr::null(), size as *mut UInt32) }
}

pub fn audio_object_set_property_data<T>(
    id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
    size: usize,
    data: *const T,
) -> OSStatus {
    unsafe {
        AudioObjectSetPropertyData(
            id,
            address,
            0,
            ptr::null(),
            size as UInt32,
            data as *const c_void,
        )
    }
}

#[allow(non_camel_case_types)]
pub type audio_object_property_listener_proc =
    extern "C" fn(AudioObjectID, u32, *const AudioObjectPropertyAddress, *mut c_void) -> OSStatus;

pub fn audio_object_add_property_listener(
    id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
    listener: audio_object_property_listener_proc,
    data: *mut c_void,
) -> OSStatus {
    unsafe { AudioObjectAddPropertyListener(id, address, Some(listener), data) }
}

pub fn audio_object_remove_property_listener(
    id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
    listener: audio_object_property_listener_proc,
    data: *mut c_void,
) -> OSStatus {
    unsafe { AudioObjectRemovePropertyListener(id, address, Some(listener), data) }
}
