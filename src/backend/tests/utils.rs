use super::*;

// Common Utils
// ------------------------------------------------------------------------------------------------
#[derive(PartialEq)]
pub enum Scope {
    Input,
    Output,
}

pub fn test_get_default_device_id(scope: Scope) -> Option<AudioObjectID> {
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

// Test Templates
// ------------------------------------------------------------------------------------------------
pub fn test_ops_context_operation<F>(name: &'static str, operation: F)
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

// Note: The in-out stream initializeed with different device will create an aggregate_device and
//       result in firing device-collection-changed callbacks. Run in-out streams with tests
//       capturing device-collection-changed callbacks may cause troubles. See more details in the
//       comments for test_create_blank_aggregate_device.
pub fn test_ops_stream_operation<F>(
    name: &'static str,
    input_device: ffi::cubeb_devid,
    input_stream_params: *mut ffi::cubeb_stream_params,
    output_device: ffi::cubeb_devid,
    output_stream_params: *mut ffi::cubeb_stream_params,
    latency_frames: u32,
    data_callback: ffi::cubeb_data_callback,
    state_callback: ffi::cubeb_state_callback,
    user_ptr: *mut c_void,
    operation: F,
) where
    F: FnOnce(*mut ffi::cubeb_stream),
{
    test_ops_context_operation("context: stream operation", |context_ptr| {
        let mut stream: *mut ffi::cubeb_stream = ptr::null_mut();
        let stream_name = CString::new(name).expect("Failed to create stream name");
        // TODO: stream_init fails when there is no input/output device when the stream parameter
        //       for input/output is given.
        assert_eq!(
            unsafe {
                OPS.stream_init.unwrap()(
                    context_ptr,
                    &mut stream,
                    stream_name.as_ptr(),
                    input_device,
                    input_stream_params,
                    output_device,
                    output_stream_params,
                    latency_frames,
                    data_callback,
                    state_callback,
                    user_ptr,
                )
            },
            ffi::CUBEB_OK
        );
        assert!(!stream.is_null());
        operation(stream);
        unsafe {
            OPS.stream_destroy.unwrap()(stream);
        }
    });
}
