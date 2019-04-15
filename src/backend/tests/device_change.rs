use super::utils::{
    test_change_default_device, test_create_device_change_listener, test_get_devices_in_scope,
    test_ops_stream_operation, Scope,
};
use super::*;

#[ignore]
#[test]
fn test_switch_device() {
    test_switch_device_in_scope(Scope::Input);
    test_switch_device_in_scope(Scope::Output);
}

fn test_switch_device_in_scope(scope: Scope) {
    use std::thread;
    use std::time::Duration;

    // Do nothing if there is no 2 available output devices at least.
    let devices = test_get_devices_in_scope(scope.clone());
    if devices.len() < 2 {
        println!("Need 2 devices for {:?} at least.", scope);
        return;
    }

    println!(
        "Switch default device for {:?} while a stream is working.",
        scope
    );

    let count = Arc::new(Mutex::new(0));
    let also_count = Arc::clone(&count);
    let listener = test_create_device_change_listener(scope.clone(), move |_addresses| {
        let mut cnt = also_count.lock().unwrap();
        *cnt += 1;
        NO_ERR
    });
    listener.start();

    test_get_started_stream_in_scope(scope.clone(), move |_stream| loop {
        thread::sleep(Duration::from_millis(500));
        if !less_than(&count, devices.len()) {
            break;
        }
        test_change_default_device(scope.clone());
    });

    fn less_than(count: &Arc<Mutex<usize>>, limit: usize) -> bool {
        let cnt = count.lock().unwrap();
        *cnt < limit
    }
}

fn test_get_started_stream_in_scope<F>(scope: Scope, operation: F)
where
    F: FnOnce(*mut ffi::cubeb_stream),
{
    use std::f32::consts::PI;
    const SAMPLE_FREQUENCY: u32 = 48_000;

    // Make sure the parameters meet the requirements of AudioUnitContext::stream_init
    // (in the comments).
    let mut stream_params = ffi::cubeb_stream_params::default();
    stream_params.format = ffi::CUBEB_SAMPLE_S16NE;
    stream_params.rate = SAMPLE_FREQUENCY;
    stream_params.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream_params.channels = 1;
    stream_params.layout = ffi::CUBEB_LAYOUT_MONO;

    let (input_params, output_params) = match scope {
        Scope::Input => (
            &mut stream_params as *mut ffi::cubeb_stream_params,
            ptr::null_mut(),
        ),
        Scope::Output => (
            ptr::null_mut(),
            &mut stream_params as *mut ffi::cubeb_stream_params,
        ),
    };

    extern "C" fn state_callback(
        stream: *mut ffi::cubeb_stream,
        user_ptr: *mut c_void,
        state: ffi::cubeb_state,
    ) {
        assert!(!stream.is_null());
        assert!(!user_ptr.is_null());
        assert_ne!(state, ffi::CUBEB_STATE_ERROR);
    }

    extern "C" fn input_data_callback(
        stream: *mut ffi::cubeb_stream,
        user_ptr: *mut c_void,
        input_buffer: *const c_void,
        output_buffer: *mut c_void,
        nframes: i64,
    ) -> i64 {
        assert!(!stream.is_null());
        assert!(!user_ptr.is_null());
        assert!(!input_buffer.is_null());
        assert!(output_buffer.is_null());
        nframes
    }

    let mut position: i64 = 0; // TODO: Use Atomic instead.

    fn f32_to_i16_sample(x: f32) -> i16 {
        (x * f32::from(i16::max_value())) as i16
    }

    extern "C" fn output_data_callback(
        stream: *mut ffi::cubeb_stream,
        user_ptr: *mut c_void,
        input_buffer: *const c_void,
        output_buffer: *mut c_void,
        nframes: i64,
    ) -> i64 {
        assert!(!stream.is_null());
        assert!(!user_ptr.is_null());
        assert!(input_buffer.is_null());
        assert!(!output_buffer.is_null());

        let buffer = unsafe {
            let ptr = output_buffer as *mut i16;
            let len = nframes as usize;
            slice::from_raw_parts_mut(ptr, len)
        };

        let position = unsafe { &mut *(user_ptr as *mut i64) };

        // Generate tone on the fly.
        for data in buffer.iter_mut() {
            let t1 = (2.0 * PI * 350.0 * (*position) as f32 / SAMPLE_FREQUENCY as f32).sin();
            let t2 = (2.0 * PI * 440.0 * (*position) as f32 / SAMPLE_FREQUENCY as f32).sin();
            *data = f32_to_i16_sample(0.5 * (t1 + t2));
            *position += 1;
        }

        nframes
    }

    test_ops_stream_operation(
        "stream",
        ptr::null_mut(), // Use default input device.
        input_params,
        ptr::null_mut(), // Use default output device.
        output_params,
        4096, // TODO: Get latency by get_min_latency instead ?
        match scope {
            Scope::Input => Some(input_data_callback),
            Scope::Output => Some(output_data_callback),
        },
        Some(state_callback),
        &mut position as *mut i64 as *mut c_void,
        |stream| {
            assert_eq!(unsafe { OPS.stream_start.unwrap()(stream) }, ffi::CUBEB_OK);
            operation(stream);
            assert_eq!(unsafe { OPS.stream_stop.unwrap()(stream) }, ffi::CUBEB_OK);
        },
    );
}
