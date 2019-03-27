use super::utils::{
    test_audiounit_scope_is_enabled, test_create_audiounit, test_device_channels_in_scope,
    test_device_in_scope, test_get_all_devices, test_get_default_audiounit,
    test_get_default_device, test_get_default_source_data, test_get_default_source_name,
    test_get_empty_stream, test_get_locked_context, ComponentSubType, Scope,
};
use super::*;
use std::any::Any;
use std::fmt::Debug;

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
        println!("No output device.");
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
        println!("No output audiounit or device source name found.");
        return;
    }

    let source = source.unwrap();
    let unit = unit.unwrap();
    if let Some(layout) = devices_layouts.get(source.as_str()) {
        assert_eq!(
            audiounit_get_preferred_channel_layout(unit.get_inner()),
            *layout
        );
    } else {
        println!("Device {} is not in the whitelist.", source);
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
        println!("No output audiounit or device source name found.");
        return;
    }

    let source = source.unwrap();
    let unit = unit.unwrap();
    if let Some(layout) = devices_layouts.get(source.as_str()) {
        assert_eq!(
            audiounit_get_current_channel_layout(unit.get_inner()),
            *layout
        );
    } else {
        println!("Device {} is not in the whitelist.", source);
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
        println!("No output audiounit or device source name found.");
        return;
    }

    let source = source.unwrap();
    let unit = unit.unwrap();
    if let Some(layout) = devices_layouts.get(source.as_str()) {
        assert!(audiounit_set_channel_layout(unit.get_inner(), io_side::OUTPUT, *layout).is_ok());
        assert_eq!(
            audiounit_get_current_channel_layout(unit.get_inner()),
            *layout
        );
    } else {
        println!("Device {} is not in the whitelist.", source);
    }
}

#[test]
fn test_set_channel_layout_output_undefind() {
    if let Some(unit) = test_get_default_audiounit(Scope::Output) {
        // Get original layout.
        let original_layout = audiounit_get_current_channel_layout(unit.get_inner());
        // Leave layout as it is.
        assert!(audiounit_set_channel_layout(
            unit.get_inner(),
            io_side::OUTPUT,
            ChannelLayout::UNDEFINED
        )
        .is_ok());
        // Check the layout is same as the original one.
        assert_eq!(
            audiounit_get_current_channel_layout(unit.get_inner()),
            original_layout
        );
    }
}

#[test]
fn test_set_channel_layout_input() {
    if let Some(unit) = test_get_default_audiounit(Scope::Input) {
        assert_eq!(
            audiounit_set_channel_layout(
                unit.get_inner(),
                io_side::INPUT,
                ChannelLayout::UNDEFINED
            )
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
            stream.output_unit = unit.get_inner();

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

// get_device_name
// ------------------------------------
#[test]
fn test_get_device_name() {
    // Unknown device.
    assert!(get_device_name(kAudioObjectUnknown).is_null());

    // Input device.
    if let Some(input) = test_get_default_device(Scope::Input) {
        let name = get_device_name(input);
        assert!(!name.is_null());
        unsafe {
            CFRelease(name as *const c_void);
        }
    }

    // Output device.
    if let Some(output) = test_get_default_device(Scope::Output) {
        let name = get_device_name(output);
        assert!(!name.is_null());
        unsafe {
            CFRelease(name as *const c_void);
        }
    }
}

// set_aggregate_sub_device_list
// ------------------------------------
#[test]
fn test_set_aggregate_sub_device_list_for_an_unknown_aggregate_device() {
    // If aggregate device id is kAudioObjectUnknown, we won't be able to
    // set device list.
    assert_eq!(
        audiounit_set_aggregate_sub_device_list(
            kAudioObjectUnknown,
            kAudioObjectUnknown,
            kAudioObjectUnknown
        )
        .unwrap_err(),
        Error::error()
    );

    let default_input = test_get_default_device(Scope::Input);
    let default_output = test_get_default_device(Scope::Output);
    if default_input.is_none() || default_output.is_none() {
        println!("No input or output device.");
        return;
    }

    let default_input = default_input.unwrap();
    let default_output = default_output.unwrap();
    assert_eq!(
        audiounit_set_aggregate_sub_device_list(kAudioObjectUnknown, default_input, default_output)
            .unwrap_err(),
        Error::error()
    );
}

// set_master_aggregate_device
// ------------------------------------
#[test]
#[should_panic]
fn test_set_master_aggregate_device_for_an_unknown_aggregate_device() {
    assert_eq!(
        audiounit_set_master_aggregate_device(kAudioObjectUnknown).unwrap_err(),
        Error::error()
    );
}

// activate_clock_drift_compensation
// ------------------------------------
#[test]
#[should_panic]
fn test_activate_clock_drift_compensation_for_an_unknown_aggregate_device() {
    assert_eq!(
        audiounit_activate_clock_drift_compensation(kAudioObjectUnknown).unwrap_err(),
        Error::error()
    );
}

// workaround_for_airpod
// ------------------------------------
// TODO

// create_aggregate_device
// ------------------------------------
// TODO

// destroy_aggregate_device
// ------------------------------------
#[test]
#[should_panic]
fn test_destroy_aggregate_device_for_unknown_plugin_and_aggregate_devices() {
    let mut aggregate_device_id = kAudioObjectUnknown;
    assert_eq!(
        audiounit_destroy_aggregate_device(kAudioObjectUnknown, &mut aggregate_device_id)
            .unwrap_err(),
        Error::error()
    )
}

// new_unit_instance
// ------------------------------------
#[test]
fn test_new_unit_instance() {
    let flags_list = [
        device_flags::DEV_UNKNOWN,
        device_flags::DEV_INPUT,
        device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
    ];

    for flags in flags_list.iter() {
        let device = device_info {
            id: kAudioObjectUnknown,
            flags: *flags,
        };
        let mut unit: AudioUnit = ptr::null_mut();
        assert!(audiounit_new_unit_instance(&mut unit, &device).is_ok());
        assert!(!unit.is_null());
        // Destroy the AudioUnits
        unsafe {
            AudioUnitUninitialize(unit);
            AudioComponentInstanceDispose(unit);
        }
    }
}

#[test]
#[should_panic]
fn test_new_unit_instance_twice() {
    let device = device_info::new();
    let mut unit: AudioUnit = ptr::null_mut();
    assert!(audiounit_new_unit_instance(&mut unit, &device).is_ok());
    assert!(!unit.is_null());

    // audiounit_new_unit_instance will get a panic immediately
    // when it's called, so the `assert_eq` and the code after
    // that won't be executed.
    assert_eq!(
        audiounit_new_unit_instance(&mut unit, &device).unwrap_err(),
        Error::error()
    );

    // Destroy the AudioUnits
    unsafe {
        AudioUnitUninitialize(unit);
        AudioComponentInstanceDispose(unit);
    }
}

// enable_unit_scope
// ------------------------------------
#[test]
fn test_enable_unit_scope() {
    // It's ok to enable and disable the scopes of input or output
    // for the unit whose subtype is kAudioUnitSubType_HALOutput
    // even when there is no available input or output devices.
    if let Some(unit) = test_create_audiounit(ComponentSubType::HALOutput) {
        assert!(audiounit_enable_unit_scope(
            &(unit.get_inner()),
            io_side::OUTPUT,
            enable_state::ENABLE
        )
        .is_ok());
        assert!(audiounit_enable_unit_scope(
            &(unit.get_inner()),
            io_side::OUTPUT,
            enable_state::DISABLE
        )
        .is_ok());
        assert!(audiounit_enable_unit_scope(
            &(unit.get_inner()),
            io_side::INPUT,
            enable_state::ENABLE
        )
        .is_ok());
        assert!(audiounit_enable_unit_scope(
            &(unit.get_inner()),
            io_side::INPUT,
            enable_state::DISABLE
        )
        .is_ok());
    }
}

#[test]
fn test_enable_unit_output_scope_for_default_output_unit() {
    if let Some(unit) = test_create_audiounit(ComponentSubType::DefaultOutput) {
        assert_eq!(
            audiounit_enable_unit_scope(&(unit.get_inner()), io_side::OUTPUT, enable_state::ENABLE)
                .unwrap_err(),
            Error::error()
        );
        assert_eq!(
            audiounit_enable_unit_scope(
                &(unit.get_inner()),
                io_side::OUTPUT,
                enable_state::DISABLE
            )
            .unwrap_err(),
            Error::error()
        );
        assert_eq!(
            audiounit_enable_unit_scope(&(unit.get_inner()), io_side::INPUT, enable_state::ENABLE)
                .unwrap_err(),
            Error::error()
        );
        assert_eq!(
            audiounit_enable_unit_scope(&(unit.get_inner()), io_side::INPUT, enable_state::DISABLE)
                .unwrap_err(),
            Error::error()
        );
    }
}

#[test]
#[should_panic]
fn test_enable_unit_scope_with_null_unit() {
    let unit: AudioUnit = ptr::null_mut();
    assert_eq!(
        audiounit_enable_unit_scope(&unit, io_side::INPUT, enable_state::DISABLE).unwrap_err(),
        Error::error()
    );
}

// create_unit
// ------------------------------------
#[test]
fn test_create_unit() {
    let flags_list = [
        device_flags::DEV_INPUT,
        device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
    ];

    let default_input = test_get_default_device(Scope::Input);
    let default_output = test_get_default_device(Scope::Output);

    for flags in flags_list.iter() {
        let mut device = device_info::new();
        device.flags |= *flags;

        // Check the output scope is enabled.
        if device.flags.contains(device_flags::DEV_OUTPUT) && default_output.is_some() {
            let device_id = default_output.unwrap();
            device.id = device_id;
            let mut unit: AudioUnit = ptr::null_mut();
            assert!(audiounit_create_unit(&mut unit, &device).is_ok());
            assert!(!unit.is_null());
            assert!(test_audiounit_scope_is_enabled(unit, Scope::Output));

            // For default output device, the input scope is enabled
            // if it's also a input device. Otherwise, it's disabled.
            if device
                .flags
                .contains(device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT)
            {
                assert_eq!(
                    test_device_in_scope(device_id, Scope::Input),
                    test_audiounit_scope_is_enabled(unit, Scope::Input)
                );

                // Destroy the audioUnit.
                unsafe {
                    AudioUnitUninitialize(unit);
                    AudioComponentInstanceDispose(unit);
                }
                continue;
            }

            // Destroy the audioUnit.
            unsafe {
                AudioUnitUninitialize(unit);
                AudioComponentInstanceDispose(unit);
            }
        }

        // Check the input scope is enabled.
        if device.flags.contains(device_flags::DEV_INPUT) && default_input.is_some() {
            let device_id = default_input.unwrap();
            device.id = device_id;
            let mut unit: AudioUnit = ptr::null_mut();
            assert!(audiounit_create_unit(&mut unit, &device).is_ok());
            assert!(!unit.is_null());
            assert!(test_audiounit_scope_is_enabled(unit, Scope::Input));
            // Destroy the audioUnit.
            unsafe {
                AudioUnitUninitialize(unit);
                AudioComponentInstanceDispose(unit);
            }
        }
    }
}

#[test]
#[should_panic]
fn test_create_unit_with_unknown_scope() {
    let device = device_info::new();
    let mut unit: AudioUnit = ptr::null_mut();
    assert!(audiounit_create_unit(&mut unit, &device).is_ok());
    assert!(!unit.is_null());
}

#[test]
#[should_panic]
fn test_create_unit_twice() {
    let flags_list = [
        device_flags::DEV_INPUT,
        device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_INPUT | device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
    ];

    // The first audiounit_create_unit calling will get a panic immediately
    // so the loop is executed once.
    for flags in flags_list.iter() {
        let mut device = device_info::new();
        device.flags |= *flags;
        let mut unit: AudioUnit = ptr::null_mut();
        assert!(audiounit_create_unit(&mut unit, &device).is_ok());
        assert!(!unit.is_null());
        assert_eq!(
            audiounit_create_unit(&mut unit, &device).unwrap_err(),
            Error::error()
        );
    }
}

// init_input_linear_buffer
// ------------------------------------
#[test]
fn test_init_input_linear_buffer() {
    // Test if the stream latency frame is 4096
    test_init_input_linear_buffer_impl(&[3.1_f32, 4.1, 5.9, 2.6, 5.35], 4096);
    test_init_input_linear_buffer_impl(&[13_i16, 21, 34, 55, 89, 144], 4096);

    // TODO: Is it ok without setting latency ?
    test_init_input_linear_buffer_impl(&[3.1_f32, 4.1, 5.9, 2.6, 5.35], 0);
    test_init_input_linear_buffer_impl(&[13_i16, 21, 34, 55, 89, 144], 0);

    fn test_init_input_linear_buffer_impl<T: Any + Debug + PartialEq>(array: &[T], latency: u32) {
        const CHANNEL: u32 = 2;
        const BUF_CAPACITY: u32 = 1;

        let type_id = std::any::TypeId::of::<T>();
        let format = if type_id == std::any::TypeId::of::<f32>() {
            kAudioFormatFlagIsFloat
        } else if type_id == std::any::TypeId::of::<i16>() {
            kAudioFormatFlagIsSignedInteger
        } else {
            panic!("Unsupported type!");
        };

        test_get_empty_stream(|stream| {
            stream.latency_frames = latency;
            stream.input_desc.mFormatFlags |= format;
            stream.input_desc.mChannelsPerFrame = CHANNEL;

            assert!(stream.input_linear_buffer.is_none());
            assert!(audiounit_init_input_linear_buffer(stream, BUF_CAPACITY).is_ok());
            assert!(stream.input_linear_buffer.is_some());

            let buffer_ref = stream.input_linear_buffer.as_mut().unwrap();
            buffer_ref.push(array.as_ptr() as *const c_void, array.len());

            assert_eq!(buffer_ref.elements(), array.len());
            let data = buffer_ref.as_ptr() as *const T;
            for (idx, item) in array.iter().enumerate() {
                unsafe {
                    assert_eq!(*data.add(idx), *item);
                }
            }
        });
    }
}

// FIXIT: We should get a panic! The type is unknown before the audio description is set!
#[ignore]
#[test]
#[should_panic]
fn test_init_input_linear_buffer_without_valid_audiodescription() {
    test_get_empty_stream(|stream| {
        stream.latency_frames = 4096;
        assert!(stream.input_linear_buffer.is_none());
        assert!(audiounit_init_input_linear_buffer(stream, 1).is_err());
    });
}

// set_buffer_size
// ------------------------------------
#[test]
fn test_set_buffer_size() {
    test_set_buffer_size_by_scope(Scope::Input);
    test_set_buffer_size_by_scope(Scope::Output);

    fn test_set_buffer_size_by_scope(scope: Scope) {
        test_get_empty_stream(|stream| {
            let default_unit = test_get_default_audiounit(scope.clone());
            if default_unit.is_none() {
                println!("No audiounit for {:?}.", scope);
                return;
            }
            let default_unit = default_unit.unwrap();

            let (unit, unit_scope, element) = match scope {
                Scope::Input => {
                    stream.input_unit = default_unit.get_inner();
                    (stream.input_unit, kAudioUnitScope_Output, AU_IN_BUS)
                }
                Scope::Output => {
                    stream.output_unit = default_unit.get_inner();
                    (stream.output_unit, kAudioUnitScope_Input, AU_OUT_BUS)
                }
            };

            let mut buffer_frames: u32 = 0;
            let mut size = mem::size_of::<u32>();
            assert_eq!(
                audio_unit_get_property(
                    unit,
                    kAudioDevicePropertyBufferFrameSize,
                    unit_scope,
                    element,
                    &mut buffer_frames,
                    &mut size
                ),
                NO_ERR
            );
            assert_ne!(buffer_frames, 0);
            buffer_frames *= 2;
            assert!(audiounit_set_buffer_size(stream, buffer_frames, scope.into()).is_ok());
        });
    }
}

#[test]
#[should_panic]
fn test_set_buffer_size_for_input_with_null_input_unit() {
    test_set_buffer_size_by_scope_with_null_unit(Scope::Input);
}

#[test]
#[should_panic]
fn test_set_buffer_size_for_output_with_null_output_unit() {
    test_set_buffer_size_by_scope_with_null_unit(Scope::Output);
}

fn test_set_buffer_size_by_scope_with_null_unit(scope: Scope) {
    test_get_empty_stream(|stream| {
        let unit = match scope {
            Scope::Input => stream.input_unit,
            Scope::Output => stream.output_unit,
        };
        assert!(unit.is_null());
        assert_eq!(
            audiounit_set_buffer_size(stream, 2048, scope.into()).unwrap_err(),
            Error::error()
        );
    });
}

// setup_stream
// ------------------------------------
// TODO

// stream_destroy_internal
// ------------------------------------
// TODO

// stream_destroy
// ------------------------------------
// TODO

// stream_start_internal
// ------------------------------------
// TODO

// stream_start
// ------------------------------------
// TODO

// stream_stop_internal
// ------------------------------------
// TODO

// stream_get_volume
// ------------------------------------
#[test]
fn test_stream_get_volume() {
    if let Some(unit) = test_get_default_audiounit(Scope::Output) {
        test_get_empty_stream(|stream| {
            stream.output_unit = unit.get_inner();

            let expected_volume: f32 = 0.5;
            stream.set_volume(expected_volume);

            let mut actual_volume: f32 = 0.0;
            assert!(audiounit_stream_get_volume(stream, &mut actual_volume).is_ok());

            assert_eq!(expected_volume, actual_volume);
        });
    } else {
        println!("No output audiounit.");
    }
}

// convert_uint32_into_string
// ------------------------------------
#[test]
fn test_convert_uint32_into_string() {
    let empty = convert_uint32_into_string(0);
    assert_eq!(empty, CString::default());

    let data: u32 = ('R' as u32) << 24 | ('U' as u32) << 16 | ('S' as u32) << 8 | 'T' as u32;
    let data_string = convert_uint32_into_string(data);
    assert_eq!(data_string, CString::new("RUST").unwrap());
}

// get_default_device_datasource
// ------------------------------------
#[test]
fn test_get_default_device_datasource() {
    // Input type.
    test_get_default_datasource_in_scope(Scope::Input);

    // Output type.
    test_get_default_datasource_in_scope(Scope::Output);

    // Unknown type.
    let mut data = 0;
    assert_eq!(
        audiounit_get_default_device_datasource(DeviceType::UNKNOWN, &mut data).unwrap_err(),
        Error::error()
    );

    // In-out type.
    let mut data = 0;
    assert_eq!(
        audiounit_get_default_device_datasource(DeviceType::INPUT | DeviceType::OUTPUT, &mut data)
            .unwrap_err(),
        Error::error()
    );

    fn test_get_default_datasource_in_scope(scope: Scope) {
        if let Some(source) = test_get_default_source_data(scope.clone()) {
            let mut data = 0;
            assert!(audiounit_get_default_device_datasource(scope.into(), &mut data).is_ok());
            assert_ne!(data, 0);
            assert_eq!(data, source);
        } else {
            println!("No source data for {:?}.", scope);
        }
    }
}

// get_default_device_name
// ------------------------------------
#[test]
fn test_get_default_device_name() {
    test_get_empty_stream(|stream| {
        // Input type.
        test_get_default_device_name_in_scope(stream, Scope::Input);

        // Output type.
        test_get_default_device_name_in_scope(stream, Scope::Output);

        // Unknown type.
        let mut device = ffi::cubeb_device::default();
        assert_eq!(
            audiounit_get_default_device_name(stream, &mut device, DeviceType::UNKNOWN)
                .unwrap_err(),
            Error::error()
        );
        // No need to release the memory since device.{input, output}_name are nulls.

        // FIXIT: Shouldn't we get both input and output name ?
        // In-out type.
        let mut device = ffi::cubeb_device::default();
        assert_eq!(
            audiounit_get_default_device_name(
                &stream,
                &mut device,
                DeviceType::INPUT | DeviceType::OUTPUT
            )
            .unwrap_err(),
            Error::error()
        );
        // No need to release the memory since device.{input, output}_name are nulls.

        fn test_get_default_device_name_in_scope(stream: &mut AudioUnitStream, scope: Scope) {
            if let Some(name) = test_get_default_source_name(scope.clone()) {
                let mut device: Box<ffi::cubeb_device> = Box::new(ffi::cubeb_device::default());
                assert!(audiounit_get_default_device_name(
                    stream,
                    device.as_mut(),
                    scope.clone().into()
                )
                .is_ok());
                let (input_name_is_null, output_name_is_null) = match scope {
                    Scope::Input => (false, true),
                    Scope::Output => (true, false),
                };
                assert_eq!(device.input_name.is_null(), input_name_is_null);
                assert_eq!(device.output_name.is_null(), output_name_is_null);
                // Release the memory of device.output_name.
                let device_ref = unsafe { DeviceRef::from_ptr(Box::into_raw(device)) };
                stream.device_destroy(device_ref);
            } else {
                println!("No source name for {:?}", scope);
            }
        }
    });
}

// strref_to_cstr_utf8
// ------------------------------------
// TODO

// get_channel_count
// ------------------------------------
#[test]
fn test_get_channel_count() {
    test_channel_count(Scope::Input);
    test_channel_count(Scope::Output);

    fn test_channel_count(scope: Scope) {
        let property_scope = match scope {
            Scope::Input => kAudioDevicePropertyScopeInput,
            Scope::Output => kAudioDevicePropertyScopeOutput,
        };
        if let Some(device) = test_get_default_device(scope.clone()) {
            let channels = audiounit_get_channel_count(device, property_scope);
            assert!(channels > 0);
            assert_eq!(
                channels,
                test_device_channels_in_scope(device, scope).unwrap()
            );
        } else {
            println!("No device for {:?}.", scope);
        }
    }
}

// get_available_samplerate
// ------------------------------------
#[test]
fn test_get_available_samplerate() {
    let samplerates = test_get_available_samplerate_of_device(kAudioObjectUnknown);
    for rates in samplerates {
        check_samplerates_are_zeros(rates);
    }

    test_get_available_samplerate_in_scope(Scope::Input);
    test_get_available_samplerate_in_scope(Scope::Output);

    fn test_get_available_samplerate_in_scope(scope: Scope) {
        if let Some(device) = test_get_default_device(scope.clone()) {
            let samplerates = test_get_available_samplerate_of_device(device);
            for rates in samplerates {
                // Surprisingly, we can get the input/output samplerates from a non-input/non-output device.
                check_samplerates(rates);
            }
        } else {
            println!("No device for {:?}.", scope);
        }
    }

    fn test_get_available_samplerate_of_device(id: AudioObjectID) -> Vec<(u32, u32, u32)> {
        let scopes = [
            kAudioObjectPropertyScopeGlobal,
            kAudioDevicePropertyScopeInput,
            kAudioDevicePropertyScopeOutput,
        ];
        let mut samplerates = Vec::new();
        for scope in scopes.iter() {
            samplerates.push(test_get_available_samplerate_of_device_in_scope(id, *scope));
        }
        samplerates
    }

    fn test_get_available_samplerate_of_device_in_scope(
        id: AudioObjectID,
        scope: AudioObjectPropertyScope,
    ) -> (u32, u32, u32) {
        let mut default = 0;
        let mut min = 0;
        let mut max = 0;
        audiounit_get_available_samplerate(id, scope, &mut min, &mut max, &mut default);
        (min, max, default)
    }

    fn check_samplerates((min, max, default): (u32, u32, u32)) {
        assert!(default > 0);
        assert!(min > 0);
        assert!(max > 0);
        assert!(min <= max);
        assert!(min <= default);
        assert!(default <= max);
    }

    fn check_samplerates_are_zeros((min, max, default): (u32, u32, u32)) {
        assert_eq!(min, 0);
        assert_eq!(max, 0);
        assert_eq!(default, 0);
    }
}

// get_device_presentation_latency
// ------------------------------------
#[test]
fn test_get_device_presentation_latency() {
    let latencies = test_get_device_presentation_latencies_of_device(kAudioObjectUnknown);
    for latency in latencies {
        // Hit the kAudioHardwareBadObjectError actually.
        assert_eq!(latency, 0);
    }

    test_get_device_presentation_latencies_in_scope(Scope::Input);
    test_get_device_presentation_latencies_in_scope(Scope::Output);

    fn test_get_device_presentation_latencies_in_scope(scope: Scope) {
        if let Some(device) = test_get_default_device(scope.clone()) {
            let latencies = test_get_device_presentation_latencies_of_device(device);
        // TODO: The latencies very from devices to devices. Check nothing here.
        } else {
            println!("No device for {:?}.", scope);
        }
    }

    fn test_get_device_presentation_latencies_of_device(id: AudioObjectID) -> Vec<u32> {
        let scopes = [
            kAudioObjectPropertyScopeGlobal,
            kAudioDevicePropertyScopeInput,
            kAudioDevicePropertyScopeOutput,
        ];
        let mut latencies = Vec::new();
        for scope in scopes.iter() {
            latencies.push(audiounit_get_device_presentation_latency(id, *scope));
        }
        latencies
    }
}

// is_aggregate_device
// ------------------------------------
#[test]
fn test_is_aggregate_device() {
    let mut aggregate_name = String::from(PRIVATE_AGGREGATE_DEVICE_NAME);
    aggregate_name.push_str("_something");
    let aggregate_name_cstring = CString::new(aggregate_name).unwrap();

    let mut info = ffi::cubeb_device_info::default();
    info.friendly_name = aggregate_name_cstring.as_ptr();
    assert!(is_aggregate_device(&info));

    let non_aggregate_name_cstring = CString::new("Hello World!").unwrap();
    info.friendly_name = non_aggregate_name_cstring.as_ptr();
    assert!(!is_aggregate_device(&info));
}

// device_destroy
// ------------------------------------
#[test]
fn test_device_destroy() {
    let mut device = ffi::cubeb_device_info::default();

    let device_id = CString::new("test: device id").unwrap();
    let friendly_name = CString::new("test: friendly name").unwrap();
    let vendor_name = CString::new("test: vendor name").unwrap();

    device.device_id = device_id.into_raw();
    // The group_id is a mirror to device_id in our implementation, so we could skip it.
    device.group_id = device.device_id;
    device.friendly_name = friendly_name.into_raw();
    device.vendor_name = vendor_name.into_raw();

    audiounit_device_destroy(&mut device);

    assert!(device.device_id.is_null());
    assert!(device.group_id.is_null());
    assert!(device.friendly_name.is_null());
    assert!(device.vendor_name.is_null());
}

#[test]
#[should_panic]
fn test_device_destroy_with_different_device_id_and_group_id() {
    let mut device = ffi::cubeb_device_info::default();

    let device_id = CString::new("test: device id").unwrap();
    let group_id = CString::new("test: group id").unwrap();
    let friendly_name = CString::new("test: friendly name").unwrap();
    let vendor_name = CString::new("test: vendor name").unwrap();

    device.device_id = device_id.into_raw();
    device.group_id = group_id.into_raw();
    device.friendly_name = friendly_name.into_raw();
    device.vendor_name = vendor_name.into_raw();

    audiounit_device_destroy(&mut device);
    // Hit the assertion above, so we will leak some memory allocated for the above cstring.

    assert!(device.device_id.is_null());
    assert!(device.group_id.is_null());
    assert!(device.friendly_name.is_null());
    assert!(device.vendor_name.is_null());
}

#[test]
fn test_device_destroy_empty_device() {
    let mut device = ffi::cubeb_device_info::default();

    assert!(device.device_id.is_null());
    assert!(device.group_id.is_null());
    assert!(device.friendly_name.is_null());
    assert!(device.vendor_name.is_null());

    audiounit_device_destroy(&mut device);

    assert!(device.device_id.is_null());
    assert!(device.group_id.is_null());
    assert!(device.friendly_name.is_null());
    assert!(device.vendor_name.is_null());
}

// get_devices_of_type
// ------------------------------------
#[test]
fn test_get_devices_of_type() {
    use std::collections::HashSet;

    // FIXIT: Open this assertion after C version is updated.
    // let no_devs = audiounit_get_devices_of_type(DeviceType::UNKNOWN);
    // assert!(no_devs.is_empty());
    let all_devices = audiounit_get_devices_of_type(DeviceType::INPUT | DeviceType::OUTPUT);
    let input_devices = audiounit_get_devices_of_type(DeviceType::INPUT);
    let output_devices = audiounit_get_devices_of_type(DeviceType::OUTPUT);

    let mut expected_all = test_get_all_devices();
    expected_all.sort();
    assert_eq!(all_devices, expected_all);
    for device in all_devices.iter() {
        if test_device_in_scope(*device, Scope::Input) {
            assert!(input_devices.contains(device));
        }
        if test_device_in_scope(*device, Scope::Output) {
            assert!(output_devices.contains(device));
        }
    }

    let input: HashSet<AudioObjectID> = input_devices.iter().cloned().collect();
    let output: HashSet<AudioObjectID> = output_devices.iter().cloned().collect();
    let union: HashSet<AudioObjectID> = input.union(&output).cloned().collect();
    let mut union_devices: Vec<AudioObjectID> = union.iter().cloned().collect();
    union_devices.sort();
    assert_eq!(all_devices, union_devices);
}
