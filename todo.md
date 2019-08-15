# TO DO

## General
- Some of bugs are found when adding tests. Search *FIXIT* to find them.
- Remove `#[allow(non_camel_case_types)]`, `#![allow(unused_assignments)]`, `#![allow(unused_must_use)]`
- Merge `io_side` and `DeviceType`
- Use `ErrorChain`
- Centralize the error log in one place
- Check scope for `audiounit_get_available_samplerate`
- Create utils in device_property to replace:
  - `audiounit_get_available_samplerate`
  - `audiounit_get_device_presentation_latency`
  - `audiounit_get_acceptable_latency_range`
  - `audiounit_get_default_datasource`
- Merge _property_address.rs_ and _device_property.rs_
- Support `enumerate_devices` with in-out type?
- Monitor `kAudioDevicePropertyDeviceIsAlive` for output device.
- Create a wrapper for `CFArrayCreateMutable` like what we do for `CFMutableDictionaryRef`
- Create a wrapper for property listener’s callback
- Change to official _coreaudio-sys_ after [pull #28](https://github.com/RustAudio/coreaudio-sys/pull/28) is is merged

### Generics
- Create a _generics_ for `cubeb_pan_stereo_buffer_{float, int}`
- Create a _generics_ for `input_linear_buffer`

## Aggregate device
### Get sub devices
- A better pattern for `AggregateDevice::get_sub_devices`
### Set sub devices
- We will add overlapping devices between `input_sub_devices` and `output_sub_devices`.
  - if they are same device
  - if either one of them or both of them are aggregate devices
### Setting master device
- We always set the master device to the first subdevice of the default output device
  but the output device (forming the aggregate device) may not be the default output device
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