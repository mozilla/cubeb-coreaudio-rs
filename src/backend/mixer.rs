use cubeb_backend::{ChannelLayout, SampleFormat};
use std::mem;
use std::os::raw::{c_int, c_void};

extern crate mixer;
pub use self::mixer::Channel;

const CHANNEL_OERDER: [mixer::Channel; 19] = [
    mixer::Channel::FrontLeft,
    mixer::Channel::FrontRight,
    mixer::Channel::FrontCenter,
    mixer::Channel::LowFrequency,
    mixer::Channel::BackLeft,
    mixer::Channel::BackRight,
    mixer::Channel::FrontLeftOfCenter,
    mixer::Channel::FrontRightOfCenter,
    mixer::Channel::BackCenter,
    mixer::Channel::SideLeft,
    mixer::Channel::SideRight,
    mixer::Channel::TopCenter,
    mixer::Channel::TopFrontLeft,
    mixer::Channel::TopFrontCenter,
    mixer::Channel::TopFrontRight,
    mixer::Channel::TopBackLeft,
    mixer::Channel::TopBackCenter,
    mixer::Channel::TopBackRight,
    mixer::Channel::Silence,
];

pub fn get_channel_order(channel_layout: ChannelLayout) -> Vec<mixer::Channel> {
    let mut map = channel_layout.bits();
    let mut order = Vec::new();
    let mut channel_index: usize = 0;
    while map != 0 {
        if map & 1 == 1 {
            order.push(CHANNEL_OERDER[channel_index]);
        }
        map >>= 1;
        channel_index += 1;
    }
    order
}

fn get_default_channel_order(channel_count: usize) -> Vec<mixer::Channel> {
    assert_ne!(channel_count, 0);
    let mut channels = Vec::with_capacity(channel_count);
    for i in 0..channel_count {
        channels.push(if i < CHANNEL_OERDER.len() {
            CHANNEL_OERDER[i]
        } else {
            mixer::Channel::Silence
        });
    }
    channels
}

#[derive(Debug)]
enum MixerType {
    IntegerMixer(mixer::Mixer<i16>),
    FloatMixer(mixer::Mixer<f32>),
}

impl MixerType {
    fn new(
        format: SampleFormat,
        input_channels: Vec<mixer::Channel>,
        output_channels: Vec<mixer::Channel>,
    ) -> Self {
        match format {
            SampleFormat::S16LE | SampleFormat::S16BE | SampleFormat::S16NE => {
                cubeb_log!("Create an integer type(i16) mixer");
                Self::IntegerMixer(mixer::Mixer::<i16>::new(input_channels, output_channels))
            }
            SampleFormat::Float32LE | SampleFormat::Float32BE | SampleFormat::Float32NE => {
                cubeb_log!("Create an integer type(f32) mixer");
                Self::FloatMixer(mixer::Mixer::<f32>::new(input_channels, output_channels))
            }
        }
    }

    fn sample_size(&self) -> usize {
        match self {
            MixerType::IntegerMixer(_) => mem::size_of::<i16>(),
            MixerType::FloatMixer(_) => mem::size_of::<f32>(),
        }
    }

    fn mix(
        &self,
        input_channels: &[mixer::Channel],
        input_buffer_ptr: *const u8,
        input_buffer_size: usize,
        output_channels: &[mixer::Channel],
        output_buffer_ptr: *mut u8,
        output_buffer_size: usize,
        frames: usize,
    ) {
        use std::slice;

        // Check input buffer size.
        let size_needed = frames * input_channels.len() * self.sample_size();
        assert!(input_buffer_size >= size_needed);
        // Check output buffer size.
        let size_needed = frames * output_channels.len() * self.sample_size();
        assert!(output_buffer_size >= size_needed);

        match self {
            MixerType::IntegerMixer(m) => {
                let in_buf_ptr = input_buffer_ptr as *const i16;
                let out_buf_ptr = output_buffer_ptr as *mut i16;
                let input_buffer =
                    unsafe { slice::from_raw_parts(in_buf_ptr, frames * input_channels.len()) };
                let output_buffer = unsafe {
                    slice::from_raw_parts_mut(out_buf_ptr, frames * output_channels.len())
                };
                let mut in_buf = input_buffer.chunks(input_channels.len());
                let mut out_buf = output_buffer.chunks_mut(output_channels.len());
                for _ in 0..frames {
                    m.mix(in_buf.next().unwrap(), out_buf.next().unwrap());
                }
            }
            MixerType::FloatMixer(m) => {
                let in_buf_ptr = input_buffer_ptr as *const f32;
                let out_buf_ptr = output_buffer_ptr as *mut f32;
                let input_buffer =
                    unsafe { slice::from_raw_parts(in_buf_ptr, frames * input_channels.len()) };
                let output_buffer = unsafe {
                    slice::from_raw_parts_mut(out_buf_ptr, frames * output_channels.len())
                };
                let mut in_buf = input_buffer.chunks(input_channels.len());
                let mut out_buf = output_buffer.chunks_mut(output_channels.len());
                for _ in 0..frames {
                    m.mix(in_buf.next().unwrap(), out_buf.next().unwrap());
                }
            }
        };
    }
}

#[derive(Debug)]
pub struct Mixer {
    mixer: MixerType,
    input_channels: Vec<mixer::Channel>,
    output_channels: Vec<mixer::Channel>,
    // Only accessed from callback thread.
    buffer: Vec<u8>,
}

impl Mixer {
    pub fn new(
        format: SampleFormat,
        in_channel_count: usize,
        input_layout: ChannelLayout,
        out_channel_count: usize,
        mut output_channels: Vec<mixer::Channel>,
    ) -> Self {
        // The input channel layout is expected to be a standard SMPTR layout.
        assert_eq!(
            in_channel_count as u32,
            input_layout.bits().count_ones(),
            "Mismatch between input channels and layout"
        );
        let input_channels = get_channel_order(input_layout);

        // When having one or two channel, force mono or stereo. Some devices (namely,
        // Bose QC35, mark 1 and 2), expose a single channel mapped to the right for
        // some reason.
        // TODO: Only apply this setting when device is Bose QC35 (by device_property.rs).
        if out_channel_count == 1 {
            output_channels = vec![mixer::Channel::FrontCenter];
        } else if out_channel_count == 2 {
            output_channels = vec![mixer::Channel::FrontLeft, mixer::Channel::FrontRight];
        }

        let all_slience = vec![mixer::Channel::Silence; out_channel_count];
        if output_channels.len() == 0
            || out_channel_count != output_channels.len()
            || all_slience == output_channels
        {
            cubeb_log!("Mismatch between output channels and layout. Apply default layout instead");
            output_channels = get_default_channel_order(out_channel_count);
        }

        Self {
            // TODO: Get input and output channels from mixer instead of copying them.
            mixer: MixerType::new(format, input_channels.clone(), output_channels.clone()),
            input_channels,
            output_channels,
            buffer: Vec::new(),
        }
    }

    pub fn update_buffer_size(&mut self, frames: usize) -> bool {
        let size_needed = frames * self.input_channels.len() * self.mixer.sample_size();
        let elements_needed = size_needed / mem::size_of::<u8>();
        if self.buffer.len() < elements_needed {
            self.buffer.resize(elements_needed, 0);
            true
        } else {
            false
        }
    }

    pub fn get_buffer_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }

    // `update_buffer_size` must be called before this.
    pub fn mix(&self, frames: usize, dest_buffer: *mut c_void, dest_buffer_size: usize) -> c_int {
        let (src_buffer_ptr, src_buffer_size) = self.get_buffer_info();
        self.mixer.mix(
            &self.input_channels,
            src_buffer_ptr,
            src_buffer_size,
            &self.output_channels,
            dest_buffer as *mut u8,
            dest_buffer_size,
            frames,
        );
        0
    }

    fn get_buffer_info(&self) -> (*const u8, usize) {
        (
            self.buffer.as_ptr(),
            self.buffer.len() * mem::size_of::<u8>(),
        )
    }
}

#[test]
fn test_get_channel_order() {
    assert_eq!(
        get_channel_order(ChannelLayout::MONO),
        [Channel::FrontCenter]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::MONO_LFE),
        [Channel::FrontCenter, Channel::LowFrequency]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::STEREO),
        [Channel::FrontLeft, Channel::FrontRight]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::STEREO_LFE),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::LowFrequency
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_3F),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::FrontCenter
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_3F_LFE),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::FrontCenter,
            Channel::LowFrequency
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_2F1),
        [Channel::FrontLeft, Channel::FrontRight, Channel::BackCenter]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_2F1_LFE),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::LowFrequency,
            Channel::BackCenter
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_3F1),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::FrontCenter,
            Channel::BackCenter
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_3F1_LFE),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::FrontCenter,
            Channel::LowFrequency,
            Channel::BackCenter
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_2F2),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::SideLeft,
            Channel::SideRight
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_2F2_LFE),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::LowFrequency,
            Channel::SideLeft,
            Channel::SideRight
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::QUAD),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::BackLeft,
            Channel::BackRight
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::QUAD_LFE),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::LowFrequency,
            Channel::BackLeft,
            Channel::BackRight
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_3F2),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::FrontCenter,
            Channel::SideLeft,
            Channel::SideRight
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_3F2_LFE),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::FrontCenter,
            Channel::LowFrequency,
            Channel::SideLeft,
            Channel::SideRight
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_3F2_BACK),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::FrontCenter,
            Channel::BackLeft,
            Channel::BackRight
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_3F2_LFE_BACK),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::FrontCenter,
            Channel::LowFrequency,
            Channel::BackLeft,
            Channel::BackRight
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_3F3R_LFE),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::FrontCenter,
            Channel::LowFrequency,
            Channel::BackCenter,
            Channel::SideLeft,
            Channel::SideRight
        ]
    );
    assert_eq!(
        get_channel_order(ChannelLayout::_3F4_LFE),
        [
            Channel::FrontLeft,
            Channel::FrontRight,
            Channel::FrontCenter,
            Channel::LowFrequency,
            Channel::BackLeft,
            Channel::BackRight,
            Channel::SideLeft,
            Channel::SideRight
        ]
    );
}
