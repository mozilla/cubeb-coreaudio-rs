// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.
use cubeb_backend::SampleFormat as fmt;
use std::mem;

pub fn allocate_array_by_size<T>(size: usize) -> Vec<T> {
    assert_eq!(size % mem::size_of::<T>(), 0);
    let elements = size / mem::size_of::<T>();
    allocate_array::<T>(elements)
}

pub fn allocate_array<T>(elements: usize) -> Vec<T> {
    let mut array = Vec::<T>::with_capacity(elements);
    unsafe {
        array.set_len(elements);
    }
    array
}

// Reference: leak_vec and retake_leaked_vec
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=6d6c2271e3811d55f740b20a00975ecf
pub fn leak_vec<T>(v: Vec<T>) -> (*mut T, usize) {
    // Drop any excess capacity by into_boxed_slice.
    let mut slice = v.into_boxed_slice();
    let ptr_and_len = (slice.as_mut_ptr(), slice.len());
    mem::forget(slice); // Leak the memory to the external code.
    ptr_and_len
}

pub fn retake_leaked_vec<T>(ptr: *mut T, len: usize) -> Vec<T> {
    // TODO: It's better to set ptr to null and len to 0.
    //       so the address won't be used again.
    unsafe { Vec::from_raw_parts(ptr, len, len) }
}

pub fn cubeb_sample_size(format: fmt) -> usize {
    match format {
        fmt::S16LE | fmt::S16BE | fmt::S16NE => mem::size_of::<i16>(),
        fmt::Float32LE | fmt::Float32BE | fmt::Float32NE => mem::size_of::<f32>(),
    }
}

#[test]
fn test_leak_vec_and_retake_it() {
    let expected: Vec<u32> = (10..20).collect();
    let leaked = expected.clone();
    let (ptr, len) = leak_vec(leaked);
    let retaken = retake_leaked_vec(ptr, len);
    for (idx, data) in retaken.iter().enumerate() {
        assert_eq!(*data, expected[idx]);
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
