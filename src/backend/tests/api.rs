use super::utils::{
    test_get_all_devices, test_get_default_audiounit, test_get_default_device,
    test_get_default_source_name, test_get_empty_stream, test_get_locked_context, Scope,
};
use super::*;

// make_sized_audio_channel_layout
// ------------------------------------
#[test]
fn test_make_sized_audio_channel_layout() {
    for channels in 1..10 {
        let size = mem::size_of::<AudioChannelLayout>()
            + (channels - 1) * mem::size_of::<AudioChannelDescription>();
        let _ = make_sized_audio_channel_layout(size);
    }
}

#[test]
#[should_panic]
fn test_make_sized_audio_channel_layout_with_wrong_size() {
    // let _ = make_sized_audio_channel_layout(0);
    let one_channel_size = mem::size_of::<AudioChannelLayout>();
    let padding_size = 10;
    assert_ne!(mem::size_of::<AudioChannelDescription>(), padding_size);
    let wrong_size = one_channel_size + padding_size;
    let _ = make_sized_audio_channel_layout(wrong_size);
}

// to_string
// ------------------------------------
#[test]
fn test_to_string() {
    assert_eq!(to_string(&io_side::INPUT), "input");
    assert_eq!(to_string(&io_side::OUTPUT), "output");
}

// has_input
// ------------------------------------
// TODO

// has_output
// ------------------------------------
// TODO

// channel_label_to_cubeb_channel
// ------------------------------------
#[test]
fn test_channel_label_to_cubeb_channel() {
    let pairs = [
        (kAudioChannelLabel_Left, ChannelLayout::FRONT_LEFT),
        (kAudioChannelLabel_Right, ChannelLayout::FRONT_RIGHT),
        (kAudioChannelLabel_Center, ChannelLayout::FRONT_CENTER),
        (kAudioChannelLabel_LFEScreen, ChannelLayout::LOW_FREQUENCY),
        (kAudioChannelLabel_LeftSurround, ChannelLayout::BACK_LEFT),
        (kAudioChannelLabel_RightSurround, ChannelLayout::BACK_RIGHT),
        (
            kAudioChannelLabel_LeftCenter,
            ChannelLayout::FRONT_LEFT_OF_CENTER,
        ),
        (
            kAudioChannelLabel_RightCenter,
            ChannelLayout::FRONT_RIGHT_OF_CENTER,
        ),
        (
            kAudioChannelLabel_CenterSurround,
            ChannelLayout::BACK_CENTER,
        ),
        (
            kAudioChannelLabel_LeftSurroundDirect,
            ChannelLayout::SIDE_LEFT,
        ),
        (
            kAudioChannelLabel_RightSurroundDirect,
            ChannelLayout::SIDE_RIGHT,
        ),
        (
            kAudioChannelLabel_TopCenterSurround,
            ChannelLayout::TOP_CENTER,
        ),
        (
            kAudioChannelLabel_VerticalHeightLeft,
            ChannelLayout::TOP_FRONT_LEFT,
        ),
        (
            kAudioChannelLabel_VerticalHeightCenter,
            ChannelLayout::TOP_FRONT_CENTER,
        ),
        (
            kAudioChannelLabel_VerticalHeightRight,
            ChannelLayout::TOP_FRONT_RIGHT,
        ),
        (kAudioChannelLabel_TopBackLeft, ChannelLayout::TOP_BACK_LEFT),
        (
            kAudioChannelLabel_TopBackCenter,
            ChannelLayout::TOP_BACK_CENTER,
        ),
        (
            kAudioChannelLabel_TopBackRight,
            ChannelLayout::TOP_BACK_RIGHT,
        ),
        (kAudioChannelLabel_Unknown, ChannelLayout::UNDEFINED),
    ];

    for (label, channel) in pairs.iter() {
        assert_eq!(channel_label_to_cubeb_channel(*label), *channel);
    }
}

// cubeb_channel_to_channel_label
// ------------------------------------
#[test]
fn test_cubeb_channel_to_channel_label() {
    let pairs = [
        (ChannelLayout::FRONT_LEFT, kAudioChannelLabel_Left),
        (ChannelLayout::FRONT_RIGHT, kAudioChannelLabel_Right),
        (ChannelLayout::FRONT_CENTER, kAudioChannelLabel_Center),
        (ChannelLayout::LOW_FREQUENCY, kAudioChannelLabel_LFEScreen),
        (ChannelLayout::BACK_LEFT, kAudioChannelLabel_LeftSurround),
        (ChannelLayout::BACK_RIGHT, kAudioChannelLabel_RightSurround),
        (
            ChannelLayout::FRONT_LEFT_OF_CENTER,
            kAudioChannelLabel_LeftCenter,
        ),
        (
            ChannelLayout::FRONT_RIGHT_OF_CENTER,
            kAudioChannelLabel_RightCenter,
        ),
        (
            ChannelLayout::BACK_CENTER,
            kAudioChannelLabel_CenterSurround,
        ),
        (
            ChannelLayout::SIDE_LEFT,
            kAudioChannelLabel_LeftSurroundDirect,
        ),
        (
            ChannelLayout::SIDE_RIGHT,
            kAudioChannelLabel_RightSurroundDirect,
        ),
        (
            ChannelLayout::TOP_CENTER,
            kAudioChannelLabel_TopCenterSurround,
        ),
        (
            ChannelLayout::TOP_FRONT_LEFT,
            kAudioChannelLabel_VerticalHeightLeft,
        ),
        (
            ChannelLayout::TOP_FRONT_CENTER,
            kAudioChannelLabel_VerticalHeightCenter,
        ),
        (
            ChannelLayout::TOP_FRONT_RIGHT,
            kAudioChannelLabel_VerticalHeightRight,
        ),
        (ChannelLayout::TOP_BACK_LEFT, kAudioChannelLabel_TopBackLeft),
        (
            ChannelLayout::TOP_BACK_CENTER,
            kAudioChannelLabel_TopBackCenter,
        ),
        (
            ChannelLayout::TOP_BACK_RIGHT,
            kAudioChannelLabel_TopBackRight,
        ),
    ];

    for (channel, label) in pairs.iter() {
        assert_eq!(cubeb_channel_to_channel_label(*channel), *label);
    }
}

#[test]
#[should_panic]
fn test_cubeb_channel_to_channel_label_with_invalid_channel() {
    assert_eq!(
        cubeb_channel_to_channel_label(ChannelLayout::_3F4_LFE),
        kAudioChannelLabel_Unknown
    );
}

#[test]
#[should_panic]
fn test_cubeb_channel_to_channel_label_with_unknown_channel() {
    assert_eq!(
        ChannelLayout::from(ffi::CHANNEL_UNKNOWN),
        ChannelLayout::UNDEFINED
    );
    assert_eq!(
        cubeb_channel_to_channel_label(ChannelLayout::UNDEFINED),
        kAudioChannelLabel_Unknown
    );
}

// increment_active_streams
// decrement_active_streams
// active_streams
// ------------------------------------
#[test]
fn test_increase_and_decrease_active_streams() {
    test_get_locked_context(|context| {
        assert_eq!(context.active_streams, 0);

        for i in 1..10 {
            audiounit_increment_active_streams(context);
            assert_eq!(context.active_streams, i);
            assert_eq!(audiounit_active_streams(context), i);
        }

        for i in (0..9).rev() {
            audiounit_decrement_active_streams(context);
            assert_eq!(context.active_streams, i);
            assert_eq!(audiounit_active_streams(context), i);
        }
    });
}

// set_global_latency
// ------------------------------------
#[test]
fn test_set_global_latency() {
    test_get_locked_context(|context| {
        assert_eq!(context.active_streams, 0);
        audiounit_increment_active_streams(context);
        assert_eq!(context.active_streams, 1);

        for i in 0..10 {
            audiounit_set_global_latency(context, i);
            assert_eq!(context.global_latency_frames, i);
        }
    });
}

// make_silent
// ------------------------------------
#[test]
fn test_make_silent() {
    let mut array = allocate_array::<u32>(10);
    for data in array.iter_mut() {
        *data = 0xFFFF;
    }

    let mut buffer = AudioBuffer::default();
    buffer.mData = array.as_mut_ptr() as *mut c_void;
    buffer.mDataByteSize = (array.len() * mem::size_of::<u32>()) as u32;
    buffer.mNumberChannels = 1;

    audiounit_make_silent(&mut buffer);
    for data in array {
        assert_eq!(data, 0);
    }
}

// render_input
// ------------------------------------
// TODO

// input_callback
// ------------------------------------
// TODO

// mix_output_buffer
// ------------------------------------
// TODO

// minimum_resampling_input_frames
// ------------------------------------
#[test]
fn test_minimum_resampling_input_frames() {
    test_get_empty_stream(|stream| {
        // Set input and output rates to 48000 and 44100 respectively.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (48000_f64, 44100_f64));
        let frames: i64 = 100;
        let times = stream.input_hw_rate / f64::from(stream.output_stream_params.rate());
        let expected = (frames as f64 * times).ceil() as i64;
        assert_eq!(minimum_resampling_input_frames(&stream, frames), expected);
    });
}

#[test]
#[should_panic]
fn test_minimum_resampling_input_frames_zero_input_rate() {
    test_get_empty_stream(|stream| {
        // Set input and output rates to 0 and 44100 respectively.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (0_f64, 44100_f64));
        let frames: i64 = 100;
        assert_eq!(minimum_resampling_input_frames(&stream, frames), 0);
    });
}

#[test]
#[should_panic]
fn test_minimum_resampling_input_frames_zero_output_rate() {
    test_get_empty_stream(|stream| {
        // Set input and output rates to 48000 and 0 respectively.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (48000_f64, 0_f64));
        let frames: i64 = 100;
        assert_eq!(minimum_resampling_input_frames(&stream, frames), 0);
    });
}

#[test]
fn test_minimum_resampling_input_frames_equal_input_output_rate() {
    test_get_empty_stream(|stream| {
        // Set both input and output rates to 44100.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (44100_f64, 44100_f64));
        let frames: i64 = 100;
        assert_eq!(minimum_resampling_input_frames(&stream, frames), frames);
    });
}

fn test_minimum_resampling_input_frames_set_stream_rates(
    stream: &mut AudioUnitStream,
    rates: (f64, f64),
) {
    let (input_rate, output_rate) = rates;

    // Set stream output rate
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = output_rate as u32;
    raw.channels = 2;
    raw.layout = ffi::CUBEB_LAYOUT_STEREO;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // Set stream input rate
    stream.input_hw_rate = input_rate;
}

// output_callback
// ------------------------------------
// TODO

// set_device_info
// ------------------------------------
#[test]
fn test_set_device_info_to_unknown_input_device() {
    test_get_empty_stream(|stream| {
        // The input device info of the stream will be set to the system default if the predefined
        // input device is unknown.
        if let Ok(default_input) =
            test_set_device_info_and_get_default_device(stream, Scope::Input, kAudioObjectUnknown)
        {
            assert_eq!(stream.input_device.id, default_input);
            assert_eq!(
                stream.input_device.flags,
                device_flags::DEV_INPUT
                    | device_flags::DEV_SELECTED_DEFAULT
                    | device_flags::DEV_SYSTEM_DEFAULT
            );
        }
    });
}

#[test]
fn test_set_device_info_to_unknown_output_device() {
    test_get_empty_stream(|stream| {
        // The output device info of the stream will be set to the system default if the predefined
        // output device is unknown.
        if let Ok(default_output) =
            test_set_device_info_and_get_default_device(stream, Scope::Output, kAudioObjectUnknown)
        {
            assert_eq!(stream.output_device.id, default_output);
            assert_eq!(
                stream.output_device.flags,
                device_flags::DEV_OUTPUT
                    | device_flags::DEV_SELECTED_DEFAULT
                    | device_flags::DEV_SYSTEM_DEFAULT
            );
        }
    });
}

// FIXIT: Is it ok to set input device to kAudioObjectSystemObject ?
//        The flags will be DEV_INPUT if we do so,
//        but shouldn't it be DEV_INPUT | DEV_SYSTEM_DEFAULT at least ?
#[ignore]
#[test]
fn test_set_device_info_to_system_input_device() {
    test_get_empty_stream(|stream| {
        // Will the input device info of the stream be set to the system default if the predefined
        // input device is system device ?
        if let Ok(default_input) = test_set_device_info_and_get_default_device(
            stream,
            Scope::Input,
            kAudioObjectSystemObject,
        ) {
            assert_eq!(
                stream.input_device.id,
                default_input /* or kAudioObjectSystemObject ? */
            );
            assert_eq!(
                stream.input_device.flags,
                device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT
            );
        }
    });
}

// FIXIT: Is it ok to set output device to kAudioObjectSystemObject ?
//        The flags will be DEV_OUTPUT if we do so,
//        but shouldn't it be DEV_OUTPUT | DEV_SYSTEM_DEFAULT at least ?
#[ignore]
#[test]
fn test_set_device_info_to_system_output_device() {
    test_get_empty_stream(|stream| {
        // Will the output device info of the stream be set to the system default if the predefined
        // input device is system device ?
        if let Ok(default_output) = test_set_device_info_and_get_default_device(
            stream,
            Scope::Output,
            kAudioObjectSystemObject,
        ) {
            assert_eq!(
                stream.output_device.id,
                default_output /* or kAudioObjectSystemObject ? */
            );
            assert_eq!(
                stream.output_device.flags,
                device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT
            );
        }
    });
}

// FIXIT: Is it ok to set input device to a nonexistent device ?
#[ignore]
#[test]
fn test_set_device_info_to_nonexistent_input_device() {
    test_get_empty_stream(|stream| {
        if let Ok(_default_input) = test_set_device_info_and_get_default_device(
            stream,
            Scope::Input,
            std::u32::MAX, // TODO: Create an API to get nonexistent device.
        ) {
            assert!(false);
        }
    });
}

// FIXIT: Is it ok to set output device to a nonexistent device ?
#[ignore]
#[test]
fn test_set_device_info_to_nonexistent_output_device() {
    test_get_empty_stream(|stream| {
        if let Ok(_default_output) = test_set_device_info_and_get_default_device(
            stream,
            Scope::Output,
            std::u32::MAX, // TODO: Create an API to get nonexistent device.
        ) {
            assert!(false);
        }
    });
}

fn test_set_device_info_and_get_default_device(
    stream: &mut AudioUnitStream,
    scope: Scope,
    predefined_device: AudioObjectID,
) -> std::result::Result<AudioObjectID, ()> {
    assert_eq!(stream.input_device.id, kAudioObjectUnknown);
    assert_eq!(stream.input_device.flags, device_flags::DEV_UNKNOWN);
    assert_eq!(stream.output_device.id, kAudioObjectUnknown);
    assert_eq!(stream.output_device.flags, device_flags::DEV_UNKNOWN);

    let default_device = test_get_default_device(scope.clone().into());
    // Fail to call audiounit_set_device_info when there is no available device.
    if default_device.is_none() {
        assert_eq!(
            audiounit_set_device_info(stream, predefined_device, scope.into()).unwrap_err(),
            Error::error()
        );
        return Err(());
    }

    // Set the device info to the predefined device
    assert!(audiounit_set_device_info(stream, predefined_device, scope.into()).is_ok());
    Ok(default_device.unwrap())
}

// reinit_stream
// ------------------------------------
// TODO

// reinit_stream_async
// ------------------------------------
// TODO

// event_addr_to_string
// ------------------------------------
// TODO

// property_listener_callback
// ------------------------------------
// TODO

// add_listener (for default output device)
// ------------------------------------
#[test]
fn test_add_listener_unknown_device() {
    extern "C" fn callback(
        _id: AudioObjectID,
        _number_of_addresses: u32,
        _addresses: *const AudioObjectPropertyAddress,
        _data: *mut c_void,
    ) -> OSStatus {
        assert!(false, "Should not be called.");
        kAudioHardwareUnspecifiedError as OSStatus
    }

    test_get_empty_stream(|stream| {
        let mut listener = property_listener::new(
            kAudioObjectUnknown,
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
            callback,
            stream,
        );
        assert_eq!(
            audiounit_add_listener(&mut listener),
            kAudioHardwareBadObjectError as OSStatus
        );
    });
}

// remove_listener (for default output device)
// ------------------------------------
#[test]
fn test_add_listener_then_remove_system_device() {
    extern "C" fn callback(
        _id: AudioObjectID,
        _number_of_addresses: u32,
        _addresses: *const AudioObjectPropertyAddress,
        _data: *mut c_void,
    ) -> OSStatus {
        assert!(false, "Should not be called.");
        kAudioHardwareUnspecifiedError as OSStatus
    }

    test_get_empty_stream(|stream| {
        let mut listener = property_listener::new(
            kAudioObjectSystemObject,
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
            callback,
            stream,
        );
        assert_eq!(audiounit_add_listener(&mut listener), NO_ERR);
        assert_eq!(audiounit_remove_listener(&mut listener), NO_ERR);
    });
}

#[test]
fn test_remove_listener_without_adding_any_listener_before_system_device() {
    extern "C" fn callback(
        _id: AudioObjectID,
        _number_of_addresses: u32,
        _addresses: *const AudioObjectPropertyAddress,
        _data: *mut c_void,
    ) -> OSStatus {
        assert!(false, "Should not be called.");
        kAudioHardwareUnspecifiedError as OSStatus
    }

    test_get_empty_stream(|stream| {
        let mut listener = property_listener::new(
            kAudioObjectSystemObject,
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
            callback,
            stream,
        );
        assert_eq!(audiounit_remove_listener(&mut listener), NO_ERR);
    });
}

#[test]
fn test_remove_listener_unknown_device() {
    extern "C" fn callback(
        _id: AudioObjectID,
        _number_of_addresses: u32,
        _addresses: *const AudioObjectPropertyAddress,
        _data: *mut c_void,
    ) -> OSStatus {
        assert!(false, "Should not be called.");
        kAudioHardwareUnspecifiedError as OSStatus
    }

    test_get_empty_stream(|stream| {
        let mut listener = property_listener::new(
            kAudioObjectUnknown,
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
            callback,
            stream,
        );
        assert_eq!(
            audiounit_remove_listener(&mut listener),
            kAudioHardwareBadObjectError as OSStatus
        );
    });
}

// install_system_changed_callback
// ------------------------------------
// TODO

// uninstall_system_changed_callback
// ------------------------------------
// TODO

// get_acceptable_latency_range
// ------------------------------------
#[test]
fn test_get_acceptable_latency_range() {
    let mut latency_range = AudioValueRange::default();

    let default_output = test_get_default_device(Scope::Output);
    if default_output.is_none() {
        assert_eq!(
            audiounit_get_acceptable_latency_range(&mut latency_range).unwrap_err(),
            Error::error()
        );
        return;
    }

    assert!(audiounit_get_acceptable_latency_range(&mut latency_range).is_ok());
    assert!(latency_range.mMinimum > 0.0);
    assert!(latency_range.mMaximum > 0.0);
    assert!(latency_range.mMaximum > latency_range.mMinimum);
}

// get_default_device_id
// ------------------------------------
#[test]
fn test_get_default_device_id() {
    // Invalid types:
    assert_eq!(
        audiounit_get_default_device_id(DeviceType::UNKNOWN),
        kAudioObjectUnknown,
    );
    assert_eq!(
        audiounit_get_default_device_id(DeviceType::INPUT | DeviceType::OUTPUT),
        kAudioObjectUnknown,
    );

    if test_get_default_device(Scope::Input).is_some() {
        assert_ne!(
            audiounit_get_default_device_id(DeviceType::INPUT),
            kAudioObjectUnknown,
        );
    }

    if test_get_default_device(Scope::Output).is_some() {
        assert_ne!(
            audiounit_get_default_device_id(DeviceType::OUTPUT),
            kAudioObjectUnknown,
        );
    }
}

// convert_channel_layout
// ------------------------------------
#[test]
fn test_convert_channel_layout() {
    let pairs = [
        // The single channel is mapped to mono now.
        (vec![kAudioObjectUnknown], ChannelLayout::MONO),
        (vec![kAudioChannelLabel_Mono], ChannelLayout::MONO),
        // The dual channels are mapped to stereo now.
        (
            vec![kAudioChannelLabel_Mono, kAudioChannelLabel_LFEScreen],
            ChannelLayout::STEREO,
        ),
        (
            vec![kAudioChannelLabel_Left, kAudioChannelLabel_Right],
            ChannelLayout::STEREO,
        ),
        // The Layouts containing any unknonwn channel will be mapped to UNDEFINED.
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Unknown,
            ],
            ChannelLayout::UNDEFINED,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Unused,
            ],
            ChannelLayout::UNDEFINED,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_ForeignLanguage,
            ],
            ChannelLayout::UNDEFINED,
        ),
        // The SMPTE layouts.
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::STEREO_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
            ],
            ChannelLayout::_3F,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_3F_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_CenterSurround,
            ],
            ChannelLayout::_2F1,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_CenterSurround,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_2F1_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_CenterSurround,
            ],
            ChannelLayout::_3F1,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_CenterSurround,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_3F1_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
            ],
            ChannelLayout::_2F2,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_2F2_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurround,
                kAudioChannelLabel_RightSurround,
            ],
            ChannelLayout::QUAD,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurround,
                kAudioChannelLabel_RightSurround,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::QUAD_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
            ],
            ChannelLayout::_3F2,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_3F2_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurround,
                kAudioChannelLabel_RightSurround,
                kAudioChannelLabel_Center,
            ],
            ChannelLayout::_3F2_BACK,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_LeftSurround,
                kAudioChannelLabel_RightSurround,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LFEScreen,
            ],
            ChannelLayout::_3F2_LFE_BACK,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LFEScreen,
                kAudioChannelLabel_CenterSurround,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
            ],
            ChannelLayout::_3F3R_LFE,
        ),
        (
            vec![
                kAudioChannelLabel_Left,
                kAudioChannelLabel_Right,
                kAudioChannelLabel_Center,
                kAudioChannelLabel_LFEScreen,
                kAudioChannelLabel_LeftSurround,
                kAudioChannelLabel_RightSurround,
                kAudioChannelLabel_LeftSurroundDirect,
                kAudioChannelLabel_RightSurroundDirect,
            ],
            ChannelLayout::_3F4_LFE,
        ),
    ];

    const MAX_CHANNELS: usize = 10;
    // A Rust mapping structure of the AudioChannelLayout with MAX_CHANNELS channels
    // https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.13.sdk/System/Library/Frameworks/CoreAudio.framework/Versions/A/Headers/CoreAudioTypes.h#L1332
    #[repr(C)]
    struct TestLayout {
        tag: AudioChannelLayoutTag,
        map: AudioChannelBitmap,
        number_channel_descriptions: UInt32,
        channel_descriptions: [AudioChannelDescription; MAX_CHANNELS],
    }

    impl Default for TestLayout {
        fn default() -> Self {
            Self {
                tag: AudioChannelLayoutTag::default(),
                map: AudioChannelBitmap::default(),
                number_channel_descriptions: UInt32::default(),
                channel_descriptions: [AudioChannelDescription::default(); MAX_CHANNELS],
            }
        }
    }

    let mut layout = TestLayout::default();
    layout.tag = kAudioChannelLayoutTag_UseChannelDescriptions;

    for (labels, expected_layout) in pairs.iter() {
        assert!(labels.len() <= MAX_CHANNELS);
        layout.number_channel_descriptions = labels.len() as u32;
        for (idx, label) in labels.iter().enumerate() {
            layout.channel_descriptions[idx].mChannelLabel = *label;
        }
        let layout_ref = unsafe { &(*(&layout as *const TestLayout as *const AudioChannelLayout)) };
        assert_eq!(
            audiounit_convert_channel_layout(layout_ref),
            *expected_layout
        );
    }
}

// get_preferred_channel_layout
// ------------------------------------
#[test]
fn test_get_preferred_channel_layout_output() {
    // Predefined whitelist
    use std::collections::HashMap;
    let devices_layouts: HashMap<&'static str, ChannelLayout> = [
        ("hdpn", ChannelLayout::STEREO),
        ("ispk", ChannelLayout::STEREO),
        ("FApd", ChannelLayout::STEREO),
    ]
    .into_iter()
    .cloned()
    .collect();

    let source = test_get_default_source_name(Scope::Output);
    let unit = test_get_default_audiounit(Scope::Output);
    if source.is_none() || unit.is_none() {
        return;
    }

    let source = source.unwrap();
    let unit = unit.unwrap();
    if let Some(layout) = devices_layouts.get(source.as_str()) {
        assert_eq!(audiounit_get_preferred_channel_layout(unit), *layout);
    }
}

// TODO: Should it be banned ? It only works with output audiounit for now.
// #[test]
// fn test_get_preferred_channel_layout_input() {
// }

// get_current_channel_layout
// ------------------------------------
#[test]
fn test_get_current_channel_layout_output() {
    // Predefined whitelist
    use std::collections::HashMap;
    let devices_layouts: HashMap<&'static str, ChannelLayout> = [
        ("hdpn", ChannelLayout::STEREO),
        ("ispk", ChannelLayout::STEREO),
        ("FApd", ChannelLayout::STEREO),
    ]
    .into_iter()
    .cloned()
    .collect();

    let source = test_get_default_source_name(Scope::Output);
    let unit = test_get_default_audiounit(Scope::Output);
    if source.is_none() || unit.is_none() {
        return;
    }

    let source = source.unwrap();
    let unit = unit.unwrap();
    if let Some(layout) = devices_layouts.get(source.as_str()) {
        assert_eq!(audiounit_get_current_channel_layout(unit), *layout);
    }
}

// TODO: Should it be banned ? It only works with output audiounit for now.
// #[test]
// fn test_get_current_channel_layout_input() {
// }

// audio_stream_desc_init
// ------------------------------------
#[test]
fn test_audio_stream_desc_init() {
    let mut channels = 0;
    for (bits, format, flags) in [
        (
            16_u32,
            ffi::CUBEB_SAMPLE_S16LE,
            kAudioFormatFlagIsSignedInteger,
        ),
        (
            16_u32,
            ffi::CUBEB_SAMPLE_S16BE,
            kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsBigEndian,
        ),
        (32_u32, ffi::CUBEB_SAMPLE_FLOAT32LE, kAudioFormatFlagIsFloat),
        (
            32_u32,
            ffi::CUBEB_SAMPLE_FLOAT32BE,
            kAudioFormatFlagIsFloat | kAudioFormatFlagIsBigEndian,
        ),
    ]
    .iter()
    {
        let bytes = bits / 8;
        channels += 1;

        let mut raw = ffi::cubeb_stream_params::default();
        raw.format = *format;
        raw.rate = 48_000;
        raw.channels = channels;
        raw.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
        raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;

        let params = StreamParams::from(raw);

        let mut stream_description = AudioStreamBasicDescription::default();

        assert!(audio_stream_desc_init(&mut stream_description, &params).is_ok());

        assert_eq!(stream_description.mFormatID, kAudioFormatLinearPCM);
        assert_eq!(
            stream_description.mFormatFlags,
            flags | kLinearPCMFormatFlagIsPacked
        );
        assert_eq!(stream_description.mSampleRate as u32, raw.rate);
        assert_eq!(stream_description.mChannelsPerFrame, raw.channels);
        assert_eq!(stream_description.mBytesPerFrame, bytes * raw.channels);
        assert_eq!(stream_description.mFramesPerPacket, 1);
        assert_eq!(stream_description.mBytesPerPacket, bytes * raw.channels);
        assert_eq!(stream_description.mReserved, 0);
    }
}

// init_mixer
// ------------------------------------
#[test]
fn test_init_mixer() {
    test_get_empty_stream(|stream| {
        audiounit_init_mixer(stream);
        assert!(!stream.mixer.as_mut_ptr().is_null());
        // stream.mixer will be deallocated when stream is destroyed.
    });
}

// set_channel_layout
// ------------------------------------
#[test]
fn test_set_channel_layout_output() {
    // Predefined whitelist
    use std::collections::HashMap;
    let devices_layouts: HashMap<&'static str, ChannelLayout> = [
        ("hdpn", ChannelLayout::STEREO),
        ("ispk", ChannelLayout::STEREO),
        ("FApd", ChannelLayout::STEREO),
    ]
    .into_iter()
    .cloned()
    .collect();

    let source = test_get_default_source_name(Scope::Output);
    let unit = test_get_default_audiounit(Scope::Output);
    if source.is_none() || unit.is_none() {
        return;
    }

    let source = source.unwrap();
    let unit = unit.unwrap();
    if let Some(layout) = devices_layouts.get(source.as_str()) {
        assert!(audiounit_set_channel_layout(unit, io_side::OUTPUT, *layout).is_ok());
        assert_eq!(audiounit_get_current_channel_layout(unit), *layout);
    }
}

#[test]
fn test_set_channel_layout_output_undefind() {
    if let Some(unit) = test_get_default_audiounit(Scope::Output) {
        // Get original layout.
        let original_layout = audiounit_get_current_channel_layout(unit);

        // Leave layout as it is.
        assert!(
            audiounit_set_channel_layout(unit, io_side::OUTPUT, ChannelLayout::UNDEFINED).is_ok()
        );

        // Check the layout is same as the original one.
        assert_eq!(audiounit_get_current_channel_layout(unit), original_layout);
    }
}

#[test]
fn test_set_channel_layout_input() {
    if let Some(unit) = test_get_default_audiounit(Scope::Input) {
        assert_eq!(
            audiounit_set_channel_layout(unit, io_side::INPUT, ChannelLayout::UNDEFINED)
                .unwrap_err(),
            Error::error()
        );
    }
}

#[test]
#[should_panic]
fn test_set_channel_layout_with_null_unit() {
    assert!(audiounit_set_channel_layout(
        ptr::null_mut(),
        io_side::OUTPUT,
        ChannelLayout::UNDEFINED
    )
    .is_err());
}

// layout_init
// ------------------------------------
#[test]
fn test_layout_init() {
    if let Some(unit) = test_get_default_audiounit(Scope::Output) {
        test_get_empty_stream(move |stream| {
            stream.output_unit = unit;

            assert_eq!(
                stream.context.layout.load(atomic::Ordering::SeqCst),
                ChannelLayout::UNDEFINED
            );

            let layout = audiounit_get_current_channel_layout(stream.output_unit);

            audiounit_layout_init(stream, io_side::OUTPUT);

            assert_eq!(stream.context.layout.load(atomic::Ordering::SeqCst), layout);
        });
    }
}

// get_sub_devices
// ------------------------------------
// You can check this by creating an aggregate device in `Audio MIDI Setup`
// application and print out the sub devices of them!
#[test]
fn test_get_sub_devices() {
    let devices = test_get_all_devices();
    for device in devices {
        println!("{}", device);
        assert_ne!(device, kAudioObjectUnknown);
        // `audiounit_get_sub_devices(device)` will return a single-element vector
        //  containing `device` itself if it's not an aggregate device.
        let sub_devices = audiounit_get_sub_devices(device);
        // TODO: If the device is a blank aggregate device, then the assertion fails!
        assert!(!sub_devices.is_empty());
    }
}

// FIXIT: It doesn't make any sense to return the sub devices for an unknown
//        device! It should either get a panic or return an empty list!
#[test]
#[should_panic]
#[ignore]
fn test_get_sub_devices_for_a_unknown_device() {
    let devices = audiounit_get_sub_devices(kAudioObjectUnknown);
    assert!(devices.is_empty());
}
