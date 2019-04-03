use super::utils::{
    test_get_default_device, test_get_devices_in_scope, test_ops_stream_operation,
    test_set_default_device, Scope,
};
use super::*;

#[ignore]
#[test]
fn test_switch_output_device() {
    use std::f32::consts::PI;
    use std::io;

    const SAMPLE_FREQUENCY: u32 = 48_000;

    let mut position: i64 = 0; // TODO: Use Atomic instead.

    fn f32_to_i16_sample(x: f32) -> i16 {
        (x * f32::from(i16::max_value())) as i16
    }

    extern "C" fn state_callback(
        stream: *mut ffi::cubeb_stream,
        user_ptr: *mut c_void,
        state: ffi::cubeb_state,
    ) {
        assert!(!stream.is_null());
        assert!(!user_ptr.is_null());
        assert_ne!(state, ffi::CUBEB_STATE_ERROR);
    }

    extern "C" fn data_callback(
        stream: *mut ffi::cubeb_stream,
        user_ptr: *mut c_void,
        _input_buffer: *const c_void,
        output_buffer: *mut c_void,
        nframes: i64,
    ) -> i64 {
        assert!(!stream.is_null());
        assert!(!user_ptr.is_null());
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

    // Do nothing if there is no 2 available output devices at least.
    let devices = test_get_devices_in_scope(Scope::Output);
    if devices.len() < 2 {
        println!("Need 2 output devices at least.");
        return;
    }
    let current = test_get_default_device(Scope::Output).unwrap();
    let mut index = devices
        .iter()
        .position(|device| *device == current)
        .unwrap();
    println!("{:?}, {}, {}", devices, current, index);

    // Make sure the parameters meet the requirements of AudioUnitContext::stream_init
    // (in the comments).
    let mut output_params = ffi::cubeb_stream_params::default();
    output_params.format = ffi::CUBEB_SAMPLE_S16NE;
    output_params.rate = SAMPLE_FREQUENCY;
    output_params.channels = 1;
    output_params.layout = ffi::CUBEB_LAYOUT_MONO;
    output_params.prefs = ffi::CUBEB_STREAM_PREF_NONE;

    test_ops_stream_operation(
        "stream: North American dial tone",
        ptr::null_mut(), // Use default input device.
        ptr::null_mut(), // No input parameters.
        ptr::null_mut(), // Use default output device.
        &mut output_params,
        4096, // TODO: Get latency by get_min_latency instead ?
        Some(data_callback),
        Some(state_callback),
        &mut position as *mut i64 as *mut c_void,
        |stream| {
            assert_eq!(unsafe { OPS.stream_start.unwrap()(stream) }, ffi::CUBEB_OK);
            println!("Start playing! Enter 's' to switch device. Enter 'q' to quit.");
            loop {
                let mut input = String::new();
                let _ = io::stdin().read_line(&mut input);
                assert_eq!(input.pop().unwrap(), '\n');
                if input == "s" {
                    let original = devices[index];
                    index = (index + 1) % devices.len();
                    let new = devices[index];
                    assert!(test_set_default_device(new, Scope::Output).unwrap());
                    println!("Switch from {} to {}", original, new);
                }
                if input == "q" {
                    println!("Quit.");
                    break;
                }
            }
            assert_eq!(unsafe { OPS.stream_stop.unwrap()(stream) }, ffi::CUBEB_OK);
        },
    );
}
