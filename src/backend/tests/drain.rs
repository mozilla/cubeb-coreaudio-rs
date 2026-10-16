use super::*;

#[derive(Default)]
struct CallbackData {
    calls: usize,
    short: bool,
    zero: bool,
    error: bool,
    states: Vec<ffi::cubeb_state>,
}

extern "C" fn data_callback(
    _: *mut ffi::cubeb_stream,
    user: *mut c_void,
    _: *const c_void,
    output: *mut c_void,
    frames: i64,
) -> i64 {
    let data = unsafe { &mut *(user as *mut CallbackData) };
    data.calls += 1;
    if data.error {
        return -1;
    }
    if !output.is_null() {
        unsafe { ptr::write_bytes(output as *mut f32, 0, frames as usize) };
    }
    if data.short {
        if data.zero {
            0
        } else {
            frames - 1
        }
    } else {
        frames
    }
}

extern "C" fn state_callback(
    _: *mut ffi::cubeb_stream,
    user: *mut c_void,
    state: ffi::cubeb_state,
) {
    unsafe { &mut *(user as *mut CallbackData) }
        .states
        .push(state);
}

// Set up a real resampler and stream queue without opening any AudioUnits.
fn with_stream(
    rate: u32,
    input: bool,
    output: bool,
    test: impl FnOnce(&mut AudioUnitStream, &mut CallbackData),
) {
    let mut context = AudioUnitContext::new();
    context.update_latency_by_adding_stream(128);
    let mut data = CallbackData::default();
    let mut stream = Box::new(AudioUnitStream::new(
        &mut context,
        &mut data as *mut _ as *mut c_void,
        Some(data_callback),
        Some(state_callback),
        128,
    ));
    stream.core_stream_data.stm_ptr = &*stream;
    let params = ffi::cubeb_stream_params {
        format: ffi::CUBEB_SAMPLE_FLOAT32NE,
        rate,
        channels: 1,
        layout: ffi::CUBEB_LAYOUT_MONO,
        prefs: ffi::CUBEB_STREAM_PREF_NONE,
        input_params: ffi::CUBEB_INPUT_PROCESSING_PARAM_NONE,
    };
    stream.core_stream_data.resampler = Resampler::new(
        &mut *stream as *mut _ as *mut ffi::cubeb_stream,
        if input { Some(params) } else { None },
        if output { Some(params) } else { None },
        48000,
        Some(audiounit_data_callback),
        &mut data as *mut _ as *mut c_void,
        ffi::CUBEB_RESAMPLER_QUALITY_DESKTOP,
        ffi::CUBEB_RESAMPLER_RECLOCK_NONE,
    );
    stream
        .queue
        .clone()
        .run_sync(|| test(&mut stream, &mut data));
}

fn fill(stream: &mut AudioUnitStream, input: bool, output: bool) -> i64 {
    let mut in_buffer = [0.0f32; 128];
    let mut out_buffer = [0.0f32; 128];
    let mut frames = 128;
    stream.core_stream_data.resampler.fill(
        if input {
            in_buffer.as_mut_ptr().cast()
        } else {
            ptr::null_mut()
        },
        if input { &mut frames } else { ptr::null_mut() },
        if output {
            out_buffer.as_mut_ptr().cast()
        } else {
            ptr::null_mut()
        },
        if output { 128 } else { 0 },
    )
}

#[test]
fn drain_survives_resampler_replacement_and_explicit_restart() {
    for rate in [48000, 44100] {
        for (input, output) in [(true, false), (false, true), (true, true)] {
            for zero in [false, true] {
                with_stream(rate, input, output, |stream, data| {
                    assert_eq!(fill(stream, input, output), 128);
                    data.short = true;
                    data.zero = zero;
                    let result = fill(stream, input, output);
                    assert_eq!(data.calls, 2);
                    if output && rate == 44100 && !zero {
                        assert_eq!(result, 128, "resampler hides the short return");
                    }
                    stream.core_stream_data.resampler.reset();
                    for _ in 0..4 {
                        assert_eq!(fill(stream, input, output), 0);
                    }
                    assert_eq!(data.calls, 2);

                    stream.prepare_start();
                    data.short = false;
                    assert_eq!(fill(stream, input, output), 128);
                    assert_eq!(data.calls, 3);
                });
            }
        }
    }
}

#[test]
fn reinit_completes_pending_drain_once() {
    for zero in [false, true] {
        with_stream(44100, false, true, |stream, data| {
            stream.stopped.store(false, Ordering::SeqCst);
            assert_eq!(fill(stream, false, true), 128);
            data.short = true;
            data.zero = zero;
            fill(stream, false, true);
            stream.reinit().unwrap();
            assert!(stream.stopped.load(Ordering::SeqCst));
            assert!(stream.delayed_reinit);
            assert_eq!(data.states, [ffi::CUBEB_STATE_DRAINED]);
            stream.reinit().unwrap();
            assert_eq!(data.states, [ffi::CUBEB_STATE_DRAINED]);
            assert_eq!(data.calls, 2);
        });
    }
}

#[test]
fn callback_error_survives_replacement_and_reinit() {
    with_stream(44100, false, true, |stream, data| {
        stream.stopped.store(false, Ordering::SeqCst);
        data.error = true;
        assert_eq!(fill(stream, false, true), -1);
        stream.core_stream_data.resampler.reset();
        assert_eq!(fill(stream, false, true), -1);
        assert_eq!(data.calls, 1);
        stream.reinit().unwrap();
        stream.reinit().unwrap();
        assert_eq!(data.states, [ffi::CUBEB_STATE_ERROR]);
    });
}
