use super::*;

// Common Utils
// ------------------------------------------------------------------------------------------------
#[derive(Clone, PartialEq)]
pub enum Scope {
    Input,
    Output,
}

impl From<Scope> for io_side {
    fn from(scope: Scope) -> Self {
        match scope {
            Scope::Input => io_side::INPUT,
            Scope::Output => io_side::OUTPUT,
        }
    }
}

pub fn test_get_default_device(scope: Scope) -> Option<AudioObjectID> {
    let address = AudioObjectPropertyAddress {
        mSelector: if scope == Scope::Input {
            kAudioHardwarePropertyDefaultInputDevice
        } else {
            kAudioHardwarePropertyDefaultOutputDevice
        },
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

    let mut devid: AudioObjectID = kAudioObjectUnknown;
    let mut size = mem::size_of::<AudioObjectID>();
    let status = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &address,
            0,
            ptr::null(),
            &mut size as *mut usize as *mut UInt32,
            &mut devid as *mut AudioObjectID as *mut c_void,
        )
    };
    if status != NO_ERR || devid == kAudioObjectUnknown {
        return None;
    }
    Some(devid)
}

// TODO: 1. Return Result with custom errors.
//       2. Allow to create a in-out unit.
pub fn test_get_default_audiounit(scope: Scope) -> Option<AudioUnit> {
    let device = test_get_default_device(scope.clone());
    let unit = test_create_haloutput_audiounit();
    if device.is_none() || unit.is_none() {
        return None;
    }
    let unit = unit.unwrap();
    let device = device.unwrap();
    match scope {
        Scope::Input => {
            if test_enable_audiounit_in_scope(unit, Scope::Input, true).is_err()
                || test_enable_audiounit_in_scope(unit, Scope::Output, false).is_err()
            {
                return None;
            }
        }
        Scope::Output => {
            if test_enable_audiounit_in_scope(unit, Scope::Input, false).is_err()
                || test_enable_audiounit_in_scope(unit, Scope::Output, true).is_err()
            {
                return None;
            }
        }
    }

    let status = unsafe {
        AudioUnitSetProperty(
            unit,
            kAudioOutputUnitProperty_CurrentDevice,
            kAudioUnitScope_Global,
            0, // Global bus
            &device as *const AudioObjectID as *const c_void,
            mem::size_of::<AudioObjectID>() as u32,
        )
    };
    if status == NO_ERR {
        Some(unit)
    } else {
        None
    }
}

// TODO: Return Result with custom errors.
fn test_create_haloutput_audiounit() -> Option<AudioUnit> {
    let desc = AudioComponentDescription {
        componentType: kAudioUnitType_Output,
        componentSubType: kAudioUnitSubType_HALOutput,
        componentManufacturer: kAudioUnitManufacturer_Apple,
        componentFlags: 0,
        componentFlagsMask: 0,
    };
    let comp = unsafe { AudioComponentFindNext(ptr::null_mut(), &desc) };
    if comp.is_null() {
        return None;
    }
    let mut unit: AudioUnit = ptr::null_mut();
    let status = unsafe { AudioComponentInstanceNew(comp, &mut unit) };
    // TODO: Is unit possible to be null when no error returns ?
    if status != NO_ERR || unit.is_null() {
        None
    } else {
        Some(unit)
    }
}

fn test_enable_audiounit_in_scope(
    unit: AudioUnit,
    scope: Scope,
    enable: bool,
) -> std::result::Result<(), OSStatus> {
    assert!(!unit.is_null());
    let (scope, element) = match scope {
        Scope::Input => (kAudioUnitScope_Input, AU_IN_BUS),
        Scope::Output => (kAudioUnitScope_Output, AU_OUT_BUS),
    };
    let on_off: u32 = if enable { 1 } else { 0 };
    let status = unsafe {
        AudioUnitSetProperty(
            unit,
            kAudioOutputUnitProperty_EnableIO,
            scope,
            element,
            &on_off as *const u32 as *const c_void,
            mem::size_of::<u32>() as u32,
        )
    };
    if status == NO_ERR {
        Ok(())
    } else {
        Err(status)
    }
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

pub fn test_get_locked_context<F>(operation: F)
where
    F: FnOnce(&mut AudioUnitContext),
{
    // Initialize the the mutex (whose type is OwnedCriticalSection) within AudioUnitContext,
    // by AudioUnitContext::Init, to make the mutex work.
    let mut context = AudioUnitContext::new();
    context.init();

    // Create a `mutext_ptr` here to avoid the borrowing-twice issue.
    let mutex_ptr = &mut context.mutex as *mut OwnedCriticalSection;
    // The scope of `_lock` is a critical section.
    let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });

    operation(&mut context);
}

pub fn test_get_stream<F>(
    user_ptr: *mut c_void,
    data_callback: ffi::cubeb_data_callback,
    state_callback: ffi::cubeb_state_callback,
    latency_frames: u32,
    operation: F,
) where
    F: FnOnce(&mut AudioUnitStream),
{
    // Initialize the the mutex (whose type is OwnedCriticalSection) within AudioUnitContext,
    // by AudioUnitContext::Init, to make the mutex work.
    let mut context = AudioUnitContext::new();
    context.init();

    // Add a stream to the context since we are about to create one.
    // AudioUnitStream::drop() will check the context has at least one stream.
    {
        // Create a `mutext_ptr` here to avoid the borrowing-twice issue.
        let mutex_ptr = &mut context.mutex as *mut OwnedCriticalSection;
        // The scope of `_lock` is a critical section.
        let _lock = AutoLock::new(unsafe { &mut (*mutex_ptr) });
        audiounit_increment_active_streams(&mut context);
    }

    let mut stream = AudioUnitStream::new(
        &mut context,
        user_ptr,
        data_callback,
        state_callback,
        latency_frames,
    );
    stream.init();

    operation(&mut stream);
}

pub fn test_get_empty_stream<F>(operation: F)
where
    F: FnOnce(&mut AudioUnitStream),
{
    test_get_stream(ptr::null_mut(), None, None, 0, operation);
}
