use super::utils::test_get_locked_context;
use super::*;

// make_sized_audio_channel_layout
// ------------------------------------
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

#[test]
fn test_make_sized_audio_channel_layout() {
    for channels in 1..10 {
        let size = mem::size_of::<AudioChannelLayout>()
            + (channels - 1) * mem::size_of::<AudioChannelDescription>();
        let _ = make_sized_audio_channel_layout(size);
    }
}

// to_string
// ------------------------------------
#[test]
fn test_to_string() {
    assert_eq!(
        to_string(&io_side::INPUT),
        "input"
    );
    assert_eq!(
        to_string(&io_side::OUTPUT),
        "output"
    );
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
        (kAudioChannelLabel_LeftCenter, ChannelLayout::FRONT_LEFT_OF_CENTER),
        (kAudioChannelLabel_RightCenter, ChannelLayout::FRONT_RIGHT_OF_CENTER),
        (kAudioChannelLabel_CenterSurround, ChannelLayout::BACK_CENTER),
        (kAudioChannelLabel_LeftSurroundDirect, ChannelLayout::SIDE_LEFT),
        (kAudioChannelLabel_RightSurroundDirect, ChannelLayout::SIDE_RIGHT),
        (kAudioChannelLabel_TopCenterSurround, ChannelLayout::TOP_CENTER),
        (kAudioChannelLabel_VerticalHeightLeft, ChannelLayout::TOP_FRONT_LEFT),
        (kAudioChannelLabel_VerticalHeightCenter, ChannelLayout::TOP_FRONT_CENTER),
        (kAudioChannelLabel_VerticalHeightRight, ChannelLayout::TOP_FRONT_RIGHT),
        (kAudioChannelLabel_TopBackLeft, ChannelLayout::TOP_BACK_LEFT),
        (kAudioChannelLabel_TopBackCenter, ChannelLayout::TOP_BACK_CENTER),
        (kAudioChannelLabel_TopBackRight, ChannelLayout::TOP_BACK_RIGHT),
        (kAudioChannelLabel_Unknown, ChannelLayout::UNDEFINED),
    ];

    for (label, channel) in pairs.iter() {
        assert_eq!(
            channel_label_to_cubeb_channel(*label),
            *channel
        );
    }
}

// cubeb_channel_to_channel_label
// ------------------------------------
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
    assert_eq!(ChannelLayout::from(ffi::CHANNEL_UNKNOWN), ChannelLayout::UNDEFINED);
    assert_eq!(
        cubeb_channel_to_channel_label(ChannelLayout::UNDEFINED),
        kAudioChannelLabel_Unknown
    );
}

#[test]
fn test_cubeb_channel_to_channel_label() {
    let pairs = [
        (ChannelLayout::FRONT_LEFT, kAudioChannelLabel_Left),
        (ChannelLayout::FRONT_RIGHT, kAudioChannelLabel_Right),
        (ChannelLayout::FRONT_CENTER, kAudioChannelLabel_Center),
        (ChannelLayout::LOW_FREQUENCY, kAudioChannelLabel_LFEScreen),
        (ChannelLayout::BACK_LEFT, kAudioChannelLabel_LeftSurround),
        (ChannelLayout::BACK_RIGHT, kAudioChannelLabel_RightSurround),
        (ChannelLayout::FRONT_LEFT_OF_CENTER, kAudioChannelLabel_LeftCenter),
        (ChannelLayout::FRONT_RIGHT_OF_CENTER, kAudioChannelLabel_RightCenter),
        (ChannelLayout::BACK_CENTER, kAudioChannelLabel_CenterSurround),
        (ChannelLayout::SIDE_LEFT, kAudioChannelLabel_LeftSurroundDirect),
        (ChannelLayout::SIDE_RIGHT, kAudioChannelLabel_RightSurroundDirect),
        (ChannelLayout::TOP_CENTER, kAudioChannelLabel_TopCenterSurround),
        (ChannelLayout::TOP_FRONT_LEFT, kAudioChannelLabel_VerticalHeightLeft),
        (ChannelLayout::TOP_FRONT_CENTER, kAudioChannelLabel_VerticalHeightCenter),
        (ChannelLayout::TOP_FRONT_RIGHT, kAudioChannelLabel_VerticalHeightRight),
        (ChannelLayout::TOP_BACK_LEFT, kAudioChannelLabel_TopBackLeft),
        (ChannelLayout::TOP_BACK_CENTER, kAudioChannelLabel_TopBackCenter),
        (ChannelLayout::TOP_BACK_RIGHT, kAudioChannelLabel_TopBackRight),
    ];

    for (channel, label) in pairs.iter() {
        assert_eq!(
            cubeb_channel_to_channel_label(*channel),
            *label
        );
    }
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
