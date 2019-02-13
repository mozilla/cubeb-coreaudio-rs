use cubeb_backend::SampleFormat as fmt;
use std::mem;

pub fn cubeb_sample_size(format: fmt) -> usize {
    match format {
        fmt::S16LE | fmt::S16BE | fmt::S16NE => mem::size_of::<i16>(),
        fmt::Float32LE | fmt::Float32BE | fmt::Float32NE => mem::size_of::<f32>(),
    }
}

#[test]
fn test_cubeb_sample_size() {
    let pairs = [
        (fmt::S16LE, mem::size_of::<i16>()),
        (fmt::S16BE, mem::size_of::<i16>()),
        (fmt::S16NE, mem::size_of::<i16>()),
        (fmt::Float32LE, mem::size_of::<f32>()),
        (fmt::Float32BE, mem::size_of::<f32>()),
        (fmt::Float32NE, mem::size_of::<f32>()),
    ];

    for pair in pairs.iter() {
        let (fotmat, size) = pair;
        assert_eq!(cubeb_sample_size(*fotmat), *size);
    }
}
