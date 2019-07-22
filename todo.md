# TO DO

## General
- Some of bugs are found when adding tests. Search *FIXIT* to find them.
- Remove `#[allow(non_camel_case_types)]`, `#![allow(unused_assignments)]`, `#![allow(unused_must_use)]`
- Merge `io_side` and `DeviceType`
- Use `ErrorChain`
- Centralize the error log in one place
- Check scope for `audiounit_get_available_samplerate`
- Return `Result` from `audiounit_get_channel_count`
- Refacotr the whole `audiounit_create_device_from_hwdev`
    - Return `cubeb_device_info` in `Result` from `audiounit_create_device_from_hwdev`
    - Decouple the settings of `devid` and `device_id`
    - Split the data retrieve into different functions
- Support `enumerate_devices` with in-out type?

### Generics
- Create a _generics_ for `cubeb_pan_stereo_buffer_{float, int}`
- Create a _generics_ for `input_linear_buffer`

## Aggregate device
### Get sub devices
- Return the device itself if the device has no `kAudioAggregateDevicePropertyActiveSubDeviceList` property
  or hit a `InvalidProperty_Error`
### Set sub devices
- We will add duplicate devices into the array if there are common devices in
  `output_sub_devices` and `input_sub_devices`
  - if they are same device
  - if either one of them or both of them are aggregate devices)
### Setting master device
- Check if the first subdevice of the default output device is in the list of
  sub devices list of the aggregate device
- Check the `name: CFStringRef` of the master device is not `NULL`

## Interface to system types and APIs
- Check if we need `AudioDeviceID` and `AudioObjectID` at the same time
- Create wrapper for `AudioObjectGetPropertyData(Size)` with _qualifier_ info
- Create wrapper for `CF` related types
- Create wrapper struct for `AudioObjectId`
    - Add `get_data`, `get_data_size`, `set_data`
- Create wrapper struct for `AudioUnit`
    - Implement `get_data`, `set_data`
- Create wrapper for `audio_unit_{add, remove}_property_listener`, `audio_object_{add, remove}_property_listener` and their callbacks
    - Add/Remove listener with generic `*mut T` data, fire their callback with generic `*mut T` data


## Interface to other module
- Create a binding layer for the `resampler`

## [Cubeb Interface][cubeb-rs]
- Implement `From` trait for `enum cubeb_device_type` so we can use `devtype.into()` to get `ffi::CUBEB_DEVICE_TYPE_*`.
- Implement `to_owned` in [`StreamParamsRef`][cubeb-rs-stmparamsref]
- Check the passed parameters like what [cubeb.c][cubeb] does!
    - Check the input `StreamParams` parameters properly, or we will set a invalid format into `AudioUnit`.
    - For example, for a duplex stream, the format of the input stream and output stream should be same.
      Using different stream formats will cause memory corruption
      since our resampler assumes the types (_short_ or _float_) of input stream (buffer) and output stream (buffer) are same
      (The resampler will use the format of the input stream if it exists, otherwise it uses the format of the output stream).
    - In fact, we should check **all** the parameters properly so we can make sure we don't mess up the streams/devices settings!

[cubeb-rs]: https://github.com/djg/cubeb-rs "cubeb-rs"
[cubeb-rs-stmparamsref]: https://github.com/djg/cubeb-rs/blob/78ed9459b8ac2ca50ea37bb72f8a06847eb8d379/cubeb-core/src/stream.rs#L61 "StreamParamsRef"

## Test
- Rewrite some tests under _cubeb/test/*_ in _Rust_ as part of the integration tests
    - Add tests for capturing/recording, output, duplex streams
- Tests cleaned up: Only tests under *aggregate_device.rs* left now.