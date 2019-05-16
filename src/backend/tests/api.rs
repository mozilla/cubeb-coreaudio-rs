use super::utils::{
    test_audiounit_get_buffer_frame_size, test_audiounit_scope_is_enabled, test_create_audiounit,
    test_device_channels_in_scope, test_device_in_scope, test_get_all_devices,
    test_get_default_audiounit, test_get_default_device, test_get_default_raw_stream,
    test_get_default_source_data, test_get_default_source_name, test_get_raw_context,
    ComponentSubType, PropertyScope, Scope,
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
    let input = io_side::INPUT;
    assert_eq!(input.to_string(), "input");
    let output = io_side::OUTPUT;
    assert_eq!(output.to_string(), "output");
}

// has_input
// ------------------------------------
// TODO

// has_output
// ------------------------------------
// TODO

// channel_label_to_cubeb_channel
// ------------------------------------
// Convert a CAChannelLabel into a ChannelLayout
#[test]
fn test_channel_label_to_cubeb_channel_layout() {
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
        let channel_label = CAChannelLabel(*label);
        let layout: ChannelLayout = channel_label.into();
        assert_eq!(layout, *channel);
    }
}

// cubeb_channel_to_channel_label
// ------------------------------------
// Convert a ChannelLayout into a CAChannelLabel
#[test]
fn test_cubeb_channel_layout_to_channel_label() {
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
        let channel_label = CAChannelLabel(*label);
        assert_eq!(CAChannelLabel::from(*channel), channel_label);
    }
}

#[test]
#[should_panic]
fn test_cubeb_channel_layout_to_channel_label_with_invalid_channel() {
    let _label = CAChannelLabel::from(ChannelLayout::_3F4_LFE);
}

#[test]
#[should_panic]
fn test_cubeb_channel_layout_to_channel_label_with_unknown_channel() {
    assert_eq!(
        ChannelLayout::from(ffi::CHANNEL_UNKNOWN),
        ChannelLayout::UNDEFINED
    );
    let _label = CAChannelLabel::from(ChannelLayout::UNDEFINED);
}

// active_streams
// update_latency_by_adding_stream
// update_latency_by_removing_stream
// ------------------------------------
#[test]
fn test_increase_and_decrease_context_streams() {
    use std::thread;
    const STREAMS: u32 = 10;

    let context = AudioUnitContext::new();
    let context_ptr_value = &context as *const AudioUnitContext as usize;

    let mut join_handles = vec![];
    for i in 0..STREAMS {
        join_handles.push(thread::spawn(move || {
            let context = unsafe { &*(context_ptr_value as *const AudioUnitContext) };
            let global_latency = context.update_latency_by_adding_stream(i);
            global_latency
        }));
    }
    let mut latencies = vec![];
    for handle in join_handles {
        latencies.push(handle.join().unwrap());
    }
    assert_eq!(context.active_streams(), STREAMS);
    check_streams(&context, STREAMS);

    check_latency(&context, latencies[0]);
    for i in 0..latencies.len() - 1 {
        assert_eq!(latencies[i], latencies[i + 1]);
    }

    let mut join_handles = vec![];
    for _ in 0..STREAMS {
        join_handles.push(thread::spawn(move || {
            let context = unsafe { &*(context_ptr_value as *const AudioUnitContext) };
            context.update_latency_by_removing_stream();
        }));
    }
    for handle in join_handles {
        let _ = handle.join();
    }
    check_streams(&context, 0);

    check_latency(&context, None);
}

fn check_streams(context: &AudioUnitContext, number: u32) {
    let guard = context.latency_controller.lock().unwrap();
    assert_eq!(guard.streams, number);
}

fn check_latency(context: &AudioUnitContext, latency: Option<u32>) {
    let guard = context.latency_controller.lock().unwrap();
    assert_eq!(guard.latency, latency);
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
    test_get_default_raw_stream(|stream| {
        // Set input and output rates to 48000 and 44100 respectively.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (48000_f64, 44100_f64));
        let frames: i64 = 100;
        let times = stream.input_hw_rate / f64::from(stream.output_stream_params.rate());
        let expected = (frames as f64 * times).ceil() as i64;
        assert_eq!(stream.minimum_resampling_input_frames(frames), expected);
    });
}

#[test]
#[should_panic]
fn test_minimum_resampling_input_frames_zero_input_rate() {
    test_get_default_raw_stream(|stream| {
        // Set input and output rates to 0 and 44100 respectively.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (0_f64, 44100_f64));
        let frames: i64 = 100;
        assert_eq!(stream.minimum_resampling_input_frames(frames), 0);
    });
}

#[test]
#[should_panic]
fn test_minimum_resampling_input_frames_zero_output_rate() {
    test_get_default_raw_stream(|stream| {
        // Set input and output rates to 48000 and 0 respectively.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (48000_f64, 0_f64));
        let frames: i64 = 100;
        assert_eq!(stream.minimum_resampling_input_frames(frames), 0);
    });
}

#[test]
fn test_minimum_resampling_input_frames_equal_input_output_rate() {
    test_get_default_raw_stream(|stream| {
        // Set both input and output rates to 44100.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (44100_f64, 44100_f64));
        let frames: i64 = 100;
        assert_eq!(stream.minimum_resampling_input_frames(frames), frames);
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

// create_device_info
// ------------------------------------
#[test]
fn test_create_device_info_from_unknown_input_device() {
    if let Some(default_device_id) = test_get_default_device(Scope::Input) {
        let default_device = create_device_info(kAudioObjectUnknown, DeviceType::INPUT).unwrap();
        assert_eq!(default_device.id, default_device_id);
        assert_eq!(
            default_device.flags,
            device_flags::DEV_INPUT
                | device_flags::DEV_SELECTED_DEFAULT
                | device_flags::DEV_SYSTEM_DEFAULT
        );
    } else {
        println!("No input device to perform test.");
    }
}

#[test]
fn test_create_device_info_from_unknown_output_device() {
    if let Some(default_device_id) = test_get_default_device(Scope::Output) {
        let default_device = create_device_info(kAudioObjectUnknown, DeviceType::OUTPUT).unwrap();
        assert_eq!(default_device.id, default_device_id);
        assert_eq!(
            default_device.flags,
            device_flags::DEV_OUTPUT
                | device_flags::DEV_SELECTED_DEFAULT
                | device_flags::DEV_SYSTEM_DEFAULT
        );
    } else {
        println!("No output device to perform test.");
    }
}

#[test]
#[should_panic]
fn test_set_device_info_to_system_input_device() {
    let _device = create_device_info(kAudioObjectSystemObject, DeviceType::INPUT);
}

#[test]
#[should_panic]
fn test_set_device_info_to_system_output_device() {
    let _device = create_device_info(kAudioObjectSystemObject, DeviceType::OUTPUT);
}

// FIXIT: Is it ok to set input device to a nonexistent device ?
#[ignore]
#[test]
#[should_panic]
fn test_set_device_info_to_nonexistent_input_device() {
    let nonexistent_id = std::u32::MAX;
    let _device = create_device_info(nonexistent_id, DeviceType::INPUT);
}

// FIXIT: Is it ok to set output device to a nonexistent device ?
#[ignore]
#[test]
#[should_panic]
fn test_set_device_info_to_nonexistent_output_device() {
    let nonexistent_id = std::u32::MAX;
    let _device = create_device_info(nonexistent_id, DeviceType::OUTPUT);
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

    test_get_default_raw_stream(|stream| {
        let listener = device_property_listener::new(
            kAudioObjectUnknown,
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
            callback,
        );
        assert_eq!(
            stream.add_device_listener(&listener),
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

    test_get_default_raw_stream(|stream| {
        let listener = device_property_listener::new(
            kAudioObjectSystemObject,
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
            callback,
        );
        assert_eq!(stream.add_device_listener(&listener), NO_ERR);
        assert_eq!(stream.remove_device_listener(&listener), NO_ERR);
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

    test_get_default_raw_stream(|stream| {
        let listener = device_property_listener::new(
            kAudioObjectSystemObject,
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
            callback,
        );
        assert_eq!(stream.remove_device_listener(&listener), NO_ERR);
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

    test_get_default_raw_stream(|stream| {
        let listener = device_property_listener::new(
            kAudioObjectUnknown,
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS,
            callback,
        );
        assert_eq!(
            stream.remove_device_listener(&listener),
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
    let default_output = test_get_default_device(Scope::Output);
    let range = audiounit_get_acceptable_latency_range();
    if default_output.is_none() {
        println!("No output device.");
        assert_eq!(range.unwrap_err(), Error::error());
        return;
    }

    let range = range.unwrap();
    assert!(range.mMinimum > 0.0);
    assert!(range.mMaximum > 0.0);
    assert!(range.mMaximum > range.mMinimum);
}

// get_default_device_id
// ------------------------------------
#[test]
fn test_get_default_device_id() {
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

#[test]
#[should_panic]
fn test_get_default_device_id_with_unknown_type() {
    assert_eq!(
        audiounit_get_default_device_id(DeviceType::UNKNOWN),
        kAudioObjectUnknown,
    );
}

#[test]
#[should_panic]
fn test_get_default_device_id_with_inout_type() {
    assert_eq!(
        audiounit_get_default_device_id(DeviceType::INPUT | DeviceType::OUTPUT),
        kAudioObjectUnknown,
    );
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

// create_stream_description
// ------------------------------------
#[test]
fn test_create_stream_description() {
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
        let description = create_stream_description(&params).unwrap();
        assert_eq!(description.mFormatID, kAudioFormatLinearPCM);
        assert_eq!(
            description.mFormatFlags,
            flags | kLinearPCMFormatFlagIsPacked
        );
        assert_eq!(description.mSampleRate as u32, raw.rate);
        assert_eq!(description.mChannelsPerFrame, raw.channels);
        assert_eq!(description.mBytesPerFrame, bytes * raw.channels);
        assert_eq!(description.mFramesPerPacket, 1);
        assert_eq!(description.mBytesPerPacket, bytes * raw.channels);
        assert_eq!(description.mReserved, 0);
    }
}

// init_mixer
// ------------------------------------
#[test]
fn test_init_mixer() {
    test_get_default_raw_stream(|stream| {
        stream.init_mixer();
        assert!(!stream.mixer.as_ptr().is_null());
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
    } else {
        println!("No output audiounit.");
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
    } else {
        println!("No input audiounit.");
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
        test_get_default_raw_stream(move |stream| {
            stream.output_unit = unit.get_inner();

            assert_eq!(
                stream.context.layout.load(atomic::Ordering::SeqCst),
                ChannelLayout::UNDEFINED
            );

            let layout = audiounit_get_current_channel_layout(stream.output_unit);

            stream.layout_init(io_side::OUTPUT);

            assert_eq!(stream.context.layout.load(atomic::Ordering::SeqCst), layout);
        });
    } else {
        println!("No output audiounit.");
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

#[test]
#[should_panic]
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
#[should_panic]
fn test_set_aggregate_sub_device_list_for_an_unknown_aggregate_device() {
    // If aggregate device id is kAudioObjectUnknown, we are unable to set device list.
    let default_input = test_get_default_device(Scope::Input);
    let default_output = test_get_default_device(Scope::Output);
    if default_input.is_none() || default_output.is_none() {
        panic!("No input or output device.");
    }

    let default_input = default_input.unwrap();
    let default_output = default_output.unwrap();
    assert_eq!(
        audiounit_set_aggregate_sub_device_list(kAudioObjectUnknown, default_input, default_output)
            .unwrap_err(),
        Error::error()
    );
}

#[test]
#[should_panic]
fn test_set_aggregate_sub_device_list_for_unknown_devices() {
    // If aggregate device id is kAudioObjectUnknown, we are unable to set device list.
    assert_eq!(
        audiounit_set_aggregate_sub_device_list(
            kAudioObjectUnknown,
            kAudioObjectUnknown,
            kAudioObjectUnknown
        )
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

// create_default_audiounit
// ------------------------------------
#[test]
fn test_create_default_audiounit() {
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
        let unit = create_default_audiounit(*flags).unwrap();
        assert!(!unit.is_null());
        // Destroy the AudioUnits
        unsafe {
            AudioUnitUninitialize(unit);
            AudioComponentInstanceDispose(unit);
        }
    }
}

// enable_audiounit_scope
// ------------------------------------
#[test]
fn test_enable_audiounit_scope() {
    // It's ok to enable and disable the scopes of input or output
    // for the unit whose subtype is kAudioUnitSubType_HALOutput
    // even when there is no available input or output devices.
    if let Some(unit) = test_create_audiounit(ComponentSubType::HALOutput) {
        assert!(enable_audiounit_scope(unit.get_inner(), io_side::OUTPUT, true).is_ok());
        assert!(enable_audiounit_scope(unit.get_inner(), io_side::OUTPUT, false).is_ok());
        assert!(enable_audiounit_scope(unit.get_inner(), io_side::INPUT, true).is_ok());
        assert!(enable_audiounit_scope(unit.get_inner(), io_side::INPUT, false).is_ok());
    } else {
        println!("No audiounit to perform test.");
    }
}

#[test]
fn test_enable_audiounit_scope_for_default_output_unit() {
    if let Some(unit) = test_create_audiounit(ComponentSubType::DefaultOutput) {
        assert_eq!(
            enable_audiounit_scope(unit.get_inner(), io_side::OUTPUT, true).unwrap_err(),
            kAudioUnitErr_InvalidProperty
        );
        assert_eq!(
            enable_audiounit_scope(unit.get_inner(), io_side::OUTPUT, false).unwrap_err(),
            kAudioUnitErr_InvalidProperty
        );
        assert_eq!(
            enable_audiounit_scope(unit.get_inner(), io_side::INPUT, true).unwrap_err(),
            kAudioUnitErr_InvalidProperty
        );
        assert_eq!(
            enable_audiounit_scope(unit.get_inner(), io_side::INPUT, false).unwrap_err(),
            kAudioUnitErr_InvalidProperty
        );
    }
}

#[test]
#[should_panic]
fn test_enable_audiounit_scope_with_null_unit() {
    let unit: AudioUnit = ptr::null_mut();
    assert!(enable_audiounit_scope(unit, io_side::INPUT, false).is_err());
}

// create_audiounit
// ------------------------------------
#[test]
fn test_for_create_audiounit() {
    let flags_list = [
        device_flags::DEV_INPUT,
        device_flags::DEV_OUTPUT,
        device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT,
        device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT,
    ];

    let default_input = test_get_default_device(Scope::Input);
    let default_output = test_get_default_device(Scope::Output);

    for flags in flags_list.iter() {
        let mut device = device_info::default();
        device.flags |= *flags;

        // Check the output scope is enabled.
        if device.flags.contains(device_flags::DEV_OUTPUT) && default_output.is_some() {
            let device_id = default_output.clone().unwrap();
            device.id = device_id;
            let unit = create_audiounit(&device).unwrap();
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
            let device_id = default_input.clone().unwrap();
            device.id = device_id;
            let unit = create_audiounit(&device).unwrap();
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
fn test_create_audiounit_with_unknown_scope() {
    let device = device_info::default();
    let _unit = create_audiounit(&device);
}

// create_auto_array
// ------------------------------------
#[test]
fn test_create_auto_array() {
    let buffer_f32 = [3.1_f32, 4.1, 5.9, 2.6, 5.35];
    let buffer_i16 = [13_i16, 21, 34, 55, 89, 144];

    // Test if the stream latency frame is 4096
    test_create_auto_array_impl(&buffer_f32, 4096);
    test_create_auto_array_impl(&buffer_i16, 4096);
}

#[test]
#[should_panic]
fn test_create_auto_array_with_zero_latency_f32() {
    let buffer_f32 = [3.1_f32, 4.1, 5.9, 2.6, 5.35];
    test_create_auto_array_impl(&buffer_f32, 0);
}

#[test]
#[should_panic]
fn test_create_auto_array_with_zero_latency_i16() {
    let buffer_i16 = [13_i16, 21, 34, 55, 89, 144];
    test_create_auto_array_impl(&buffer_i16, 0);
}

fn test_create_auto_array_impl<T: Any + Debug + PartialEq>(buffer: &[T], latency: u32) {
    const CHANNEL: u32 = 2;
    const BUF_CAPACITY: usize = 1;

    let type_id = std::any::TypeId::of::<T>();
    let format = if type_id == std::any::TypeId::of::<f32>() {
        kAudioFormatFlagIsFloat
    } else if type_id == std::any::TypeId::of::<i16>() {
        kAudioFormatFlagIsSignedInteger
    } else {
        panic!("Unsupported type!");
    };

    let mut desc = AudioStreamBasicDescription::default();
    desc.mFormatFlags |= format;
    desc.mChannelsPerFrame = CHANNEL;

    let mut array = create_auto_array(desc, latency, BUF_CAPACITY).unwrap();
    array.push(buffer.as_ptr() as *const c_void, buffer.len());
    assert_eq!(array.elements(), buffer.len());
    let data = array.as_ptr() as *const T;
    for (idx, item) in buffer.iter().enumerate() {
        unsafe {
            assert_eq!(*data.add(idx), *item);
        }
    }
}

#[test]
#[should_panic]
fn test_create_auto_array_with_empty_audiodescription() {
    let desc = AudioStreamBasicDescription::default();
    assert_eq!(
        create_auto_array(desc, 256, 1).unwrap_err(),
        Error::invalid_format()
    );
}

#[test]
fn test_create_auto_array_with_invalid_audiodescription() {
    let mut desc = AudioStreamBasicDescription::default();
    desc.mFormatFlags |= kAudioFormatFlagIsBigEndian;
    desc.mChannelsPerFrame = 100;
    assert_eq!(
        create_auto_array(desc, 256, 1).unwrap_err(),
        Error::invalid_format()
    );
}

// clamp_latency
// ------------------------------------
#[test]
fn test_clamp_latency() {
    let range = 0..2 * SAFE_MAX_LATENCY_FRAMES;
    assert!(range.start < SAFE_MIN_LATENCY_FRAMES);
    // assert!(range.end < SAFE_MAX_LATENCY_FRAMES);
    for latency_frames in range {
        let clamp = clamp_latency(latency_frames);
        assert!(clamp >= SAFE_MIN_LATENCY_FRAMES);
        assert!(clamp <= SAFE_MAX_LATENCY_FRAMES);
    }
}

// set_buffer_size_sync
// ------------------------------------
#[test]
fn test_set_buffer_size_sync() {
    test_set_buffer_size_by_scope(Scope::Input);
    test_set_buffer_size_by_scope(Scope::Output);
    fn test_set_buffer_size_by_scope(scope: Scope) {
        let unit = test_get_default_audiounit(scope.clone());
        if unit.is_none() {
            println!("No audiounit for {:?}.", scope);
            return;
        }
        let unit = unit.unwrap();
        let prop_scope = match scope {
            Scope::Input => PropertyScope::Output,
            Scope::Output => PropertyScope::Input,
        };
        let mut buffer_frames = test_audiounit_get_buffer_frame_size(
            unit.get_inner(),
            scope.clone(),
            prop_scope.clone(),
        )
        .unwrap();
        assert_ne!(buffer_frames, 0);
        buffer_frames *= 2;
        assert!(
            set_buffer_size_sync(unit.get_inner(), scope.clone().into(), buffer_frames).is_ok()
        );
        let new_buffer_frames =
            test_audiounit_get_buffer_frame_size(unit.get_inner(), scope.clone(), prop_scope)
                .unwrap();
        assert_eq!(buffer_frames, new_buffer_frames);
    }
}

#[test]
#[should_panic]
fn test_set_buffer_size_sync_for_input_with_null_input_unit() {
    test_set_buffer_size_sync_by_scope_with_null_unit(Scope::Input);
}

#[test]
#[should_panic]
fn test_set_buffer_size_sync_for_output_with_null_output_unit() {
    test_set_buffer_size_sync_by_scope_with_null_unit(Scope::Output);
}

fn test_set_buffer_size_sync_by_scope_with_null_unit(scope: Scope) {
    let unit: AudioUnit = ptr::null_mut();
    assert!(set_buffer_size_sync(unit, scope.into(), 2048).is_err());
}

// configure_input
// ------------------------------------
// Ignore the test by default to avoid overwritting the buffer frame size to the device that is
// probably operating in other tests in parallel.
#[ignore]
#[test]
fn test_configure_input() {
    let buffer_f32 = [1.1_f32, 2.2, 3.3, 4.4];
    let buffer_i16 = [1_i16, 2, 3, 4, 5, 6, 7];

    test_configure_input_impl(&buffer_f32);
    test_configure_input_impl(&buffer_i16);

    fn test_configure_input_impl<T: Any + Debug + PartialEq>(buffer: &[T]) {
        // Get format parameters for the type.
        let type_id = std::any::TypeId::of::<T>();
        let cubeb_format = if type_id == std::any::TypeId::of::<f32>() {
            ffi::CUBEB_SAMPLE_FLOAT32NE
        } else if type_id == std::any::TypeId::of::<i16>() {
            ffi::CUBEB_SAMPLE_S16NE
        } else {
            panic!("Unsupported type!");
        };

        let mut params = ffi::cubeb_stream_params::default();
        params.format = cubeb_format;
        params.rate = 48_000;
        params.channels = 1;
        params.layout = ffi::CUBEB_LAYOUT_UNDEFINED;
        params.prefs = ffi::CUBEB_STREAM_PREF_NONE;

        test_configure_scope(Scope::Input, StreamParams::from(params), |stream| {
            check_hw_rate(stream);
            check_buffer_frame_size(stream, Scope::Input);
            check_frames_per_slice(stream, Scope::Input);
            check_linear_buffer(stream, buffer);
        });

        fn check_hw_rate(stream: &mut AudioUnitStream) {
            assert_ne!(stream.input_hw_rate, 0_f64);
            let mut description = AudioStreamBasicDescription::default();
            let mut size = mem::size_of::<AudioStreamBasicDescription>();
            assert_eq!(
                audio_unit_get_property(
                    stream.input_unit,
                    kAudioUnitProperty_StreamFormat,
                    kAudioUnitScope_Output,
                    AU_IN_BUS,
                    &mut description,
                    &mut size
                ),
                0
            );
            assert_eq!(description.mSampleRate, stream.input_hw_rate);
        }

        fn check_linear_buffer<T: Any + Debug + PartialEq>(
            stream: &mut AudioUnitStream,
            array: &[T],
        ) {
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
        }
    }
}

#[test]
#[should_panic]
fn test_configure_input_with_null_unit() {
    test_get_default_raw_stream(|stream| {
        assert!(stream.input_unit.is_null());
        assert!(stream.configure_input().is_err());
    });
}

// Ignore the test by default to avoid overwritting the buffer frame size for the input or output
// device that is using in test_configure_input or test_configure_output.
#[ignore]
#[test]
#[should_panic]
fn test_configure_input_with_zero_latency_frames() {
    let mut params = ffi::cubeb_stream_params::default();
    params.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    params.rate = 48_000;
    params.channels = 1;
    params.layout = ffi::CUBEB_LAYOUT_MONO;
    params.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    test_configure_scope_with_zero_latency_frames(
        Scope::Input,
        StreamParams::from(params),
        |stream| {
            check_buffer_frame_size(stream, Scope::Input);
            check_frames_per_slice(stream, Scope::Input);
        },
    );
}

// configure_output
// ------------------------------------
// Ignore the test by default to avoid overwritting the buffer frame size to the device that is
// probably operating in other tests in parallel.
#[ignore]
#[test]
fn test_configure_output() {
    const SAMPLE_RATE: u32 = 48_000;
    let mut params = ffi::cubeb_stream_params::default();
    params.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    params.rate = SAMPLE_RATE;
    params.channels = 2;
    params.layout = ffi::CUBEB_LAYOUT_STEREO;
    params.prefs = ffi::CUBEB_STREAM_PREF_NONE;

    test_configure_scope(Scope::Output, StreamParams::from(params), |stream| {
        check_hw_rate(stream);
        check_buffer_frame_size(stream, Scope::Output);
        check_frames_per_slice(stream, Scope::Output);
    });

    fn check_hw_rate(stream: &mut AudioUnitStream) {
        let rate = f64::from(SAMPLE_RATE);
        assert_eq!(stream.output_desc.mSampleRate, rate);
        assert_ne!(stream.output_hw_rate, 0_f64);
        let mut description = AudioStreamBasicDescription::default();
        let mut size = mem::size_of::<AudioStreamBasicDescription>();
        assert_eq!(
            audio_unit_get_property(
                stream.output_unit,
                kAudioUnitProperty_StreamFormat,
                kAudioUnitScope_Input,
                AU_OUT_BUS,
                &mut description,
                &mut size
            ),
            0
        );
        assert_eq!(description.mSampleRate, rate);
    }
}

#[test]
#[should_panic]
fn test_configure_output_with_null_unit() {
    test_get_default_raw_stream(|stream| {
        assert!(stream.output_unit.is_null());
        assert!(stream.configure_output().is_err());
    });
}

// Ignore the test by default to avoid overwritting the buffer frame size for the input or output
// device that is using in test_configure_input or test_configure_output.
#[ignore]
#[test]
#[should_panic]
fn test_configure_output_with_zero_latency_frames() {
    let mut params = ffi::cubeb_stream_params::default();
    params.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    params.rate = 48_000;
    params.channels = 2;
    params.layout = ffi::CUBEB_LAYOUT_STEREO;
    params.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    test_configure_scope_with_zero_latency_frames(
        Scope::Output,
        StreamParams::from(params),
        |stream| {
            check_buffer_frame_size(stream, Scope::Output);
            check_frames_per_slice(stream, Scope::Output);
        },
    );
}

// Utils for configure_{input, output}
// ------------------------------------
fn test_configure_scope<F>(scope: Scope, params: StreamParams, callback: F)
where
    F: FnOnce(&mut AudioUnitStream),
{
    if let Some(unit) = test_get_default_audiounit(scope.clone()) {
        test_get_default_raw_stream(|stream| {
            match scope {
                Scope::Input => {
                    stream.input_unit = unit.get_inner();
                    stream.input_stream_params = params;
                }
                Scope::Output => {
                    stream.output_unit = unit.get_inner();
                    stream.output_stream_params = params;
                }
            }

            // Set the latency_frames to a valid value so `buffer frames size` and
            // `frames per slice` can be set correctly!
            stream.latency_frames = clamp_latency(0);

            let res = match scope {
                Scope::Input => stream.configure_input(),
                Scope::Output => stream.configure_output(),
            };
            assert!(res.is_ok());
            callback(stream);
        });
    } else {
        println!("No audiounit for {:?}.", scope);
    }
}

fn test_configure_scope_with_zero_latency_frames<F>(scope: Scope, params: StreamParams, callback: F)
where
    F: FnOnce(&mut AudioUnitStream),
{
    if let Some(unit) = test_get_default_audiounit(scope.clone()) {
        test_get_default_raw_stream(|stream| {
            match scope {
                Scope::Input => {
                    stream.input_unit = unit.get_inner();
                    stream.input_stream_params = params;
                }
                Scope::Output => {
                    stream.output_unit = unit.get_inner();
                    stream.output_stream_params = params;
                }
            }
            assert_eq!(stream.latency_frames, 0);
            let res = match scope {
                Scope::Input => stream.configure_input(),
                Scope::Output => stream.configure_output(),
            };
            assert!(res.is_ok());
            callback(stream);
        });
    } else {
        panic!("No audiounit for {:?}.", scope);
    }
}

fn check_buffer_frame_size(stream: &mut AudioUnitStream, scope: Scope) {
    let (unit, prop_scope) = match scope {
        Scope::Input => (stream.input_unit, PropertyScope::Output),
        Scope::Output => (stream.output_unit, PropertyScope::Input),
    };
    let buffer_frames = test_audiounit_get_buffer_frame_size(unit, scope, prop_scope).unwrap();
    // The buffer frames will be set to the same value of latency_frames.
    assert_eq!(buffer_frames, stream.latency_frames);
}

fn check_frames_per_slice(stream: &mut AudioUnitStream, scope: Scope) {
    let unit = match scope {
        Scope::Input => stream.input_unit,
        Scope::Output => stream.output_unit,
    };
    let mut frames_per_slice: u32 = 0;
    let mut size = mem::size_of::<u32>();
    assert_eq!(
        audio_unit_get_property(
            unit,
            kAudioUnitProperty_MaximumFramesPerSlice,
            kAudioUnitScope_Global,
            0, // Global Bus
            &mut frames_per_slice,
            &mut size
        ),
        NO_ERR
    );
    // The frames per slice will be set to the same value of latency_frames.
    assert_eq!(frames_per_slice, stream.latency_frames);
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

// get_volume, set_volume
// ------------------------------------
#[test]
fn test_stream_get_volume() {
    if let Some(unit) = test_get_default_audiounit(Scope::Output) {
        let expected_volume: f32 = 0.5;
        set_volume(unit.get_inner(), expected_volume);
        assert_eq!(expected_volume, get_volume(unit.get_inner()).unwrap());
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

// get_default_datasource
// ------------------------------------
#[test]
fn test_get_default_device_datasource() {
    test_get_default_datasource_in_scope(Scope::Input);
    test_get_default_datasource_in_scope(Scope::Output);

    fn test_get_default_datasource_in_scope(scope: Scope) {
        if let Some(source) = test_get_default_source_data(scope.clone()) {
            assert_eq!(
                audiounit_get_default_datasource(scope.into()).unwrap(),
                source
            );
        } else {
            println!("No source data for {:?}.", scope);
        }
    }
}

// get_default_datasource_string
// ------------------------------------
#[test]
fn test_get_default_device_name() {
    test_get_default_device_name_in_scope(Scope::Input);
    test_get_default_device_name_in_scope(Scope::Output);

    fn test_get_default_device_name_in_scope(scope: Scope) {
        if let Some(name) = test_get_default_source_name(scope.clone()) {
            let source = audiounit_get_default_datasource_string(scope.into())
                .unwrap()
                .into_string()
                .unwrap();
            assert_eq!(name, source);
        } else {
            println!("No source name for {:?}", scope);
        }
    }
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
            // TODO: The latencies very from devices to devices. Check nothing here.
            let _latencies = test_get_device_presentation_latencies_of_device(device);
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

// create_device_from_hwdev
// ------------------------------------
#[test]
fn test_create_device_from_hwdev() {
    use std::collections::VecDeque;

    let results = test_create_device_from_hwdev_by_device(kAudioObjectUnknown);
    for result in results {
        // Hit the kAudioHardwareBadObjectError actually.
        assert_eq!(result.unwrap_err(), Error::error());
    }

    test_create_device_from_hwdev_in_scope(Scope::Input);
    test_create_device_from_hwdev_in_scope(Scope::Output);

    fn test_create_device_from_hwdev_in_scope(scope: Scope) {
        if let Some(device) = test_get_default_device(scope.clone()) {
            let is_input = test_device_in_scope(device, Scope::Input);
            let is_output = test_device_in_scope(device, Scope::Output);
            let mut results = test_create_device_from_hwdev_by_device(device);
            assert_eq!(results.len(), 2);
            // Input device type:
            if is_input {
                check_device_info_by_device(
                    results.pop_front().unwrap().unwrap(),
                    device,
                    Scope::Input,
                );
            } else {
                assert_eq!(results.pop_front().unwrap().unwrap_err(), Error::error());
            }
            // Output device type:
            if is_output {
                check_device_info_by_device(
                    results.pop_front().unwrap().unwrap(),
                    device,
                    Scope::Output,
                );
            } else {
                assert_eq!(results.pop_front().unwrap().unwrap_err(), Error::error());
            }
        } else {
            println!("No device for {:?}.", scope);
        }
    }

    fn test_create_device_from_hwdev_by_device(
        id: AudioObjectID,
    ) -> VecDeque<std::result::Result<ffi::cubeb_device_info, Error>> {
        let dev_types = [DeviceType::INPUT, DeviceType::OUTPUT];
        let mut results = VecDeque::new();
        for dev_type in dev_types.iter() {
            let mut info = ffi::cubeb_device_info::default();
            let result = audiounit_create_device_from_hwdev(&mut info, id, *dev_type);
            results.push_back(if result.is_ok() {
                Ok(info)
            } else {
                Err(result.unwrap_err())
            });
        }
        results
    }

    fn check_device_info_by_device(info: ffi::cubeb_device_info, id: AudioObjectID, scope: Scope) {
        assert!(!info.devid.is_null());
        assert!(mem::size_of_val(&info.devid) >= mem::size_of::<AudioObjectID>());
        assert_eq!(info.devid as AudioObjectID, id);
        assert!(!info.device_id.is_null());
        assert!(!info.friendly_name.is_null());
        assert_eq!(info.group_id, info.device_id);
        // TODO: Hit a kAudioHardwareUnknownPropertyError for AirPods
        // assert!(!info.vendor_name.is_null());

        // FIXIT: The device is defined to input-only or output-only, but some device is in-out!
        assert_eq!(info.device_type, DeviceType::from(scope.clone()).bits());
        assert_eq!(info.state, ffi::CUBEB_DEVICE_STATE_ENABLED);
        // TODO: The preference is set when the device is default input/output device if the device
        //       info is created from input/output scope. Should the preference be set if the
        //       device is a default input/output device if the device info is created from
        //       output/input scope ? The device may be a in-out device!
        assert_eq!(info.preferred, get_cubeb_device_pref(id, scope));

        assert_eq!(info.format, ffi::CUBEB_DEVICE_FMT_ALL);
        assert_eq!(info.default_format, ffi::CUBEB_DEVICE_FMT_F32NE);
        assert!(info.max_channels > 0);
        assert!(info.min_rate <= info.max_rate);
        assert!(info.min_rate <= info.default_rate);
        assert!(info.default_rate <= info.max_rate);

        assert!(info.latency_lo > 0);
        assert!(info.latency_hi > 0);
        assert!(info.latency_lo <= info.latency_hi);

        fn get_cubeb_device_pref(id: AudioObjectID, scope: Scope) -> ffi::cubeb_device_pref {
            let default_device = test_get_default_device(scope);
            if default_device.is_some() && default_device.unwrap() == id {
                ffi::CUBEB_DEVICE_PREF_ALL
            } else {
                ffi::CUBEB_DEVICE_PREF_NONE
            }
        }
    }
}

#[test]
#[should_panic]
fn test_create_device_from_hwdev_unknown_type() {
    let mut info = ffi::cubeb_device_info::default();
    assert!(audiounit_create_device_from_hwdev(
        &mut info,
        kAudioObjectUnknown,
        DeviceType::UNKNOWN
    )
    .is_err());
}

#[test]
#[should_panic]
fn test_create_device_from_hwdev_inout_type() {
    let mut info = ffi::cubeb_device_info::default();
    assert!(audiounit_create_device_from_hwdev(
        &mut info,
        kAudioObjectUnknown,
        DeviceType::INPUT | DeviceType::OUTPUT
    )
    .is_err());
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

#[test]
#[should_panic]
fn test_get_devices_of_type_unknown() {
    let no_devs = audiounit_get_devices_of_type(DeviceType::UNKNOWN);
    assert!(no_devs.is_empty());
}

// add_devices_changed_listener
// ------------------------------------
#[test]
fn test_add_devices_changed_listener() {
    use std::collections::HashMap;

    extern "C" fn inout_callback(_: *mut ffi::cubeb, _: *mut c_void) {}
    extern "C" fn in_callback(_: *mut ffi::cubeb, _: *mut c_void) {}
    extern "C" fn out_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    let mut map: HashMap<DeviceType, extern "C" fn(*mut ffi::cubeb, *mut c_void)> = HashMap::new();
    map.insert(DeviceType::INPUT, in_callback);
    map.insert(DeviceType::OUTPUT, out_callback);
    map.insert(DeviceType::INPUT | DeviceType::OUTPUT, inout_callback);

    test_get_raw_context(|context| {
        for (devtype, callback) in map.iter() {
            assert!(get_devices_changed_callback(context, Scope::Input).is_none());
            assert!(get_devices_changed_callback(context, Scope::Output).is_none());

            // Register a callback within a specific scope.
            assert!(context
                .add_devices_changed_listener(*devtype, Some(*callback), ptr::null_mut())
                .is_ok());

            if devtype.contains(DeviceType::INPUT) {
                let cb = get_devices_changed_callback(context, Scope::Input);
                assert!(cb.is_some());
                assert_eq!(cb.unwrap(), *callback);
            } else {
                let cb = get_devices_changed_callback(context, Scope::Input);
                assert!(cb.is_none());
            }

            if devtype.contains(DeviceType::OUTPUT) {
                let cb = get_devices_changed_callback(context, Scope::Output);
                assert!(cb.is_some());
                assert_eq!(cb.unwrap(), *callback);
            } else {
                let cb = get_devices_changed_callback(context, Scope::Output);
                assert!(cb.is_none());
            }

            // Unregister the callbacks within all scopes.
            assert!(context
                .remove_devices_changed_listener(DeviceType::INPUT | DeviceType::OUTPUT)
                .is_ok());

            assert!(get_devices_changed_callback(context, Scope::Input).is_none());
            assert!(get_devices_changed_callback(context, Scope::Output).is_none());
        }
    });
}

#[test]
#[should_panic]
fn test_add_devices_changed_listener_in_unknown_scope() {
    extern "C" fn callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    test_get_raw_context(|context| {
        let _ = context.add_devices_changed_listener(
            DeviceType::UNKNOWN,
            Some(callback),
            ptr::null_mut(),
        );
    });
}

#[test]
#[should_panic]
fn test_add_devices_changed_listener_with_none_callback() {
    test_get_raw_context(|context| {
        for devtype in &[DeviceType::INPUT, DeviceType::OUTPUT] {
            assert!(context
                .add_devices_changed_listener(*devtype, None, ptr::null_mut())
                .is_ok());
        }
    });
}

// remove_devices_changed_listener
// ------------------------------------
#[test]
fn test_remove_devices_changed_listener() {
    use std::collections::HashMap;

    extern "C" fn in_callback(_: *mut ffi::cubeb, _: *mut c_void) {}
    extern "C" fn out_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    let mut map: HashMap<DeviceType, extern "C" fn(*mut ffi::cubeb, *mut c_void)> = HashMap::new();
    map.insert(DeviceType::INPUT, in_callback);
    map.insert(DeviceType::OUTPUT, out_callback);

    test_get_raw_context(|context| {
        for (devtype, _callback) in map.iter() {
            assert!(get_devices_changed_callback(context, Scope::Input).is_none());
            assert!(get_devices_changed_callback(context, Scope::Output).is_none());

            // Register callbacks within all scopes.
            for (scope, listener) in map.iter() {
                assert!(context
                    .add_devices_changed_listener(*scope, Some(*listener), ptr::null_mut())
                    .is_ok());
            }

            let input_callback = get_devices_changed_callback(context, Scope::Input);
            assert!(input_callback.is_some());
            assert_eq!(
                input_callback.unwrap(),
                *(map.get(&DeviceType::INPUT).unwrap())
            );
            let output_callback = get_devices_changed_callback(context, Scope::Output);
            assert!(output_callback.is_some());
            assert_eq!(
                output_callback.unwrap(),
                *(map.get(&DeviceType::OUTPUT).unwrap())
            );

            // Unregister the callbacks within one specific scopes.
            assert!(context.remove_devices_changed_listener(*devtype).is_ok());

            if devtype.contains(DeviceType::INPUT) {
                let cb = get_devices_changed_callback(context, Scope::Input);
                assert!(cb.is_none());
            } else {
                let cb = get_devices_changed_callback(context, Scope::Input);
                assert!(cb.is_some());
                assert_eq!(cb.unwrap(), *(map.get(&DeviceType::INPUT).unwrap()));
            }

            if devtype.contains(DeviceType::OUTPUT) {
                let cb = get_devices_changed_callback(context, Scope::Output);
                assert!(cb.is_none());
            } else {
                let cb = get_devices_changed_callback(context, Scope::Output);
                assert!(cb.is_some());
                assert_eq!(cb.unwrap(), *(map.get(&DeviceType::OUTPUT).unwrap()));
            }

            // Unregister the callbacks within all scopes.
            assert!(context
                .remove_devices_changed_listener(DeviceType::INPUT | DeviceType::OUTPUT)
                .is_ok());
        }
    });
}

#[test]
fn test_remove_devices_changed_listener_without_adding_listeners() {
    test_get_raw_context(|context| {
        for devtype in &[
            DeviceType::INPUT,
            DeviceType::OUTPUT,
            DeviceType::INPUT | DeviceType::OUTPUT,
        ] {
            assert!(context.remove_devices_changed_listener(*devtype).is_ok());
        }
    });
}

#[test]
fn test_remove_devices_changed_listener_within_all_scopes() {
    use std::collections::HashMap;

    extern "C" fn inout_callback(_: *mut ffi::cubeb, _: *mut c_void) {}
    extern "C" fn in_callback(_: *mut ffi::cubeb, _: *mut c_void) {}
    extern "C" fn out_callback(_: *mut ffi::cubeb, _: *mut c_void) {}

    let mut map: HashMap<DeviceType, extern "C" fn(*mut ffi::cubeb, *mut c_void)> = HashMap::new();
    map.insert(DeviceType::INPUT, in_callback);
    map.insert(DeviceType::OUTPUT, out_callback);
    map.insert(DeviceType::INPUT | DeviceType::OUTPUT, inout_callback);

    test_get_raw_context(|context| {
        for (devtype, callback) in map.iter() {
            assert!(get_devices_changed_callback(context, Scope::Input).is_none());
            assert!(get_devices_changed_callback(context, Scope::Output).is_none());

            assert!(context
                .add_devices_changed_listener(*devtype, Some(*callback), ptr::null_mut())
                .is_ok());

            if devtype.contains(DeviceType::INPUT) {
                let cb = get_devices_changed_callback(context, Scope::Input);
                assert!(cb.is_some());
                assert_eq!(cb.unwrap(), *callback);
            }

            if devtype.contains(DeviceType::OUTPUT) {
                let cb = get_devices_changed_callback(context, Scope::Output);
                assert!(cb.is_some());
                assert_eq!(cb.unwrap(), *callback);
            }

            assert!(context
                .remove_devices_changed_listener(DeviceType::INPUT | DeviceType::OUTPUT)
                .is_ok());

            assert!(get_devices_changed_callback(context, Scope::Input).is_none());
            assert!(get_devices_changed_callback(context, Scope::Output).is_none());
        }
    });
}

fn get_devices_changed_callback(
    context: &AudioUnitContext,
    scope: Scope,
) -> ffi::cubeb_device_collection_changed_callback {
    let devices_guard = context.devices.lock().unwrap();
    match scope {
        Scope::Input => devices_guard.input.changed_callback,
        Scope::Output => devices_guard.output.changed_callback,
    }
}
