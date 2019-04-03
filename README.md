# cubeb-coreaudio-rs

[![Build Status](https://travis-ci.org/ChunMinChang/cubeb-coreaudio-rs.svg?branch=trailblazer)](https://travis-ci.org/ChunMinChang/cubeb-coreaudio-rs)

*Rust* implementation of [Cubeb][cubeb] on the MacOS platform.

## Current Goals
- Rewrite the [C code][cubeb-au] into *Rust* on a line-by-line basis
  - The coding style is in *C* style rather than *Rust* so it's easier to review
    (and it's easy to re-format the style later by running `rustfmt`)
- Create some tests for later refactoring


## Status

All the lines in [*cubeb_audiounit.cpp*][cubeb-au] are translated.

By applying the [patch][integrate-with-cubeb] to integrate within [Cubeb][cubeb],
it can pass all the tests under *cubeb/test*
and it's able to switch devices when the stream is working
(we are unable to test this automatically yet).

Now the draft version can pass all the tests within *gecko*.
The project can be tracked on [*bugzilla* 1530715][bugzilla-cars].
(Commits to import and build this within *gecko* can be found [here][build-within-gecko])

## Test
Please run `sh run_tests.sh`.

Some tests cannot be run in parallel.
They may operate the same device at the same time,
or indirectly fire some system events that are listened by some tests.
Part of the tests are marked `#[ignore]` due to this problem.
Therefore, the tests should be run part by part.

Most of the tests are executed by running `sh run_tests.sh`.
Only those tests commented with *FIXIT* are left.

<!--
- 🥚 : Not implemented.
- 🐣 : Work in progress. May be implemented partially or blocked by dependent APIs.
- 🐥 : Implemented.
- 🐓 : Already ride the trains.

### Cubeb APIs (Public APIs)
- 🥚 : 0/20 (0%)
- 🐣 : 0/20 (0%)
- 🐥 : 20/20 (100%)

| Cubeb APIs                                    | status |
| --------------------------------------------- | ------ |
| cubub_init                                    | 🐥      |
| cubub_get_backend_id                          | 🐥      |
| cubub_get_max_channel_count                   | 🐥      |
| cubub_get_min_latency                         | 🐥      |
| cubub_get_preferred_sample_rate               | 🐥      |
| cubub_enumerate_devices                       | 🐥      |
| cubeb_device_collection_destroy               | 🐥      |
| cubeb_stream_init                             | 🐥      |
| cubeb_stream_destroy                          | 🐥      |
| cubeb_stream_start                            | 🐥      |
| cubeb_stream_stop                             | 🐥      |
| cubeb_reset_default_device                    | 🐥      |
| cubeb_stream_get_position                     | 🐥      |
| cubeb_stream_get_latency                      | 🐥      |
| cubeb_stream_set_volume                       | 🐥      |
| cubeb_stream_set_panning                      | 🐥      |
| cubeb_stream_get_current_device               | 🐥      |
| cubeb_stream_device_destroy                   | 🐥      |
| cubeb_stream_register_device_changed_callback | 🐥      |
| cubub_register_device_collection_changed      | 🐥      |

### Interanl APIs

- 🥚 : 0/75 (0%)
- 🐣 : 0/75 (0%)
- 🐥 : 74/75 (100%)

| Interanl AudioUnit APIs                     | status |
| ------------------------------------------- | ------ |
| make_sized_audio_channel_layout             | 🐥      |
| to_string                                   | 🐥      |
| has_input                                   | 🐥      |
| has_output                                  | 🐥      |
| channel_label_to_cubeb_channel              | 🐥      |
| cubeb_channel_to_channel_label              | 🐥      |
| audiounit_increment_active_streams          | 🐥      |
| audiounit_decrement_active_streams          | 🐥      |
| audiounit_active_streams                    | 🐥      |
| audiounit_set_global_latency                | 🐥      |
| audiounit_make_silent                       | 🐥      |
| audiounit_render_input                      | 🐥      |
| audiounit_input_callback                    | 🐥      |
| audiounit_mix_output_buffer                 | 🐥      |
| minimum_resampling_input_frames             | 🐥      |
| audiounit_output_callback                   | 🐥      |
| audiounit_set_device_info                   | 🐥      |
| audiounit_reinit_stream                     | 🐥      |
| audiounit_reinit_stream_async               | 🐥      |
| event_addr_to_string                        | 🐥      |
| audiounit_property_listener_callback        | 🐥      |
| audiounit_add_listener                      | 🐥      |
| audiounit_remove_listener                   | 🐥      |
| audiounit_install_device_changed_callback   | 🐥      |
| audiounit_install_system_changed_callback   | 🐥      |
| audiounit_uninstall_device_changed_callback | 🐥      |
| audiounit_uninstall_system_changed_callback | 🐥      |
| audiounit_get_acceptable_latency_range      | 🐥      |
| audiounit_get_default_device_id             | 🐥      |
| audiounit_convert_channel_layout            | 🐥      |
| audiounit_get_preferred_channel_layout      | 🐥      |
| audiounit_get_current_channel_layout        | 🐥      |
| audiounit_destroy                           | 🐥      |
| audio_stream_desc_init                      | 🐥      |
| audiounit_init_mixer                        | 🐥      |
| audiounit_set_channel_layout                | 🐥      |
| audiounit_layout_init                       | 🐥      |
| audiounit_get_sub_devices                   | 🐥      |
| audiounit_create_blank_aggregate_device     | 🐥      |
| get_device_name                             | 🐥      |
| audiounit_set_aggregate_sub_device_list     | 🐥      |
| audiounit_set_master_aggregate_device       | 🐥      |
| audiounit_activate_clock_drift_compensation | 🐥      |
| audiounit_workaround_for_airpod             | 🐥      |
| audiounit_create_aggregate_device           | 🐥      |
| audiounit_destroy_aggregate_device          | 🐥      |
| audiounit_new_unit_instance                 | 🐥      |
| audiounit_enable_unit_scope                 | 🐥      |
| audiounit_create_unit                       | 🐥      |
| audiounit_init_input_linear_buffer          | 🐥      |
| audiounit_clamp_latency                     | 🐥      |
| buffer_size_changed_callback                | 🐥      |
| audiounit_set_buffer_size                   | 🐥      |
| audiounit_configure_input                   | 🐥      |
| audiounit_configure_output                  | 🐥      |
| audiounit_setup_stream                      | 🐥      |
| audiounit_close_stream                      | 🐥      |
| audiounit_stream_destroy_internal           | 🐥      |
| audiounit_stream_destroy                    | 🐥      |
| audiounit_stream_start_internal             | 🐥      |
| audiounit_stream_stop_internal              | 🐥      |
| audiounit_stream_get_volume                 | 🐥      |
| convert_uint32_into_string                  | 🐥      |
| audiounit_get_default_device_datasource     | 🐥      |
| audiounit_get_default_device_name           | 🐥      |
| audiounit_strref_to_cstr_utf8               | 🐥      |
| audiounit_get_channel_count                 | 🐥      |
| audiounit_get_available_samplerate          | 🐥      |
| audiounit_get_device_presentation_latency   | 🐥      |
| audiounit_create_device_from_hwdev          | 🐥      |
| is_aggregate_device                         | 🐥      |
| audiounit_get_devices_of_type               | 🐥      |
| audiounit_collection_changed_callback       | 🐥      |
| audiounit_add_device_listener               | 🐥      |
| audiounit_remove_device_listener            | 🐥      |
-->

## TODO
- Maybe it's better to move all `fn some_func(stm: &AudioUnitStream, ...)` functions into `impl AudioUnitStream` to avoid useless references to `AudioUnitStream`. Perhaps it will help avoiding some borrowing issues.
- Remove `#[allow(non_camel_case_types)]`, `#![allow(unused_assignments)]`, `#![allow(unused_must_use)]` and apply *rust* coding styles
- Use `Atomic{I64, U32, U64}` instead of `Atomic<{i64, u32, u64}>`, once they are stable.
- Tests
  - Rewrite some tests under _cubeb/test/*_ in _Rust_ as part of the integration tests
    - Add tests for capturing/recording, output, duplex streams
  - Tests cleaned up: Only tests under *aggregate_device.rs* left now.
- Some of bugs are found when adding tests. Search *FIXIT* to find them.
- [cubeb-rs][cubeb-rs]
  - Implement `to_owned` in [`StreamParamsRef`][cubeb-rs-stmparamsref]
  - Check the passed parameters like what [cubeb.c][cubeb] does!
    - Check the input `StreamParams` parameters properly, or we will set a invalid format into `AudioUnit`.
    In fact, we should check **all** the parameters properly so we can make sure we don't mess up the streams/devices settings!
- Find a efficient way to catch memory leaks
  - *Instrument* on OSX

## Issues
- See discussion [here][discussion]
- Mutex: Find a replacement for [`owned_critical_section`][ocs]
  - A dummy mutex like `Mutex<()>` should work (see [`test_dummy_mutex_multithread`][ocs-rust]) as what `owned_critical_section` does in [_C_ version][ocs], but it doens't has equivalent API for `assert_current_thread_owns`.
  - We implement a [`OwnedCriticalSection` around `pthread_mutex_t`][ocs-rust] like what we do in [_C_ version][ocs] for now.
  - It's hard to debug with the variables using `OwnedCriticalSection`. Within a test with a variable using `OwnedCriticalSection`, if the `OwnedCriticalSection` used in the test isn't be dropped in a correct order, then the test will get a crash in `OwnedCriticalSection`. The examples are [`test_stream_drop_mutex_(in)correct`](src/backend/tests/test.rs). The tests must be created very carefully.
- Atomic:
  - The stable atomic types only support `bool`, `usize`, `isize`, and `ptr`, but we need `u64`, `i64`, and `f32`.
  - Using [atomic-rs](https://github.com/Amanieu/atomic-rs) instead.
  - *Rust-Nightly* supports `AtomicU32` and `AtomicU64` so we use that.
- Unworkable API: [`dispatch_async`][dis-async] and [`dispatch_sync`][dis-sync]
  - The second parameter of [`dispatch_async`][dis-async] and [`dispatch_sync`][dis-sync] is [`dispatch_block_t`][dis-block], which is defined by `typedef void (^dispatch_block_t)(void)`.
  - The caret symbol `^` defines a [block][c-ext-block].
  - The _block_ is a lambda expression-like syntax to create closures. (See Apple's document: [Working with Blocks][apple-block])
  - Not sure if _Rust_ can work with it. _Rust_ has its own [_closure_][rs-closure].
  - For now, we implement an API [`async_dispatch`][async-dis] and [`sync_dispatch`][sync-dis] to replace [`dispatch_async`][dis-async] and [`dispatch_sync`][dis-sync] (prototype on [gist][osx-dis-gist].)
    - [`async_dispatch`][async-dis] is based on [`dispatch_async_f`][dis-async-f].
    - [`sync_dispatch`][sync-dis] is based on [`dispatch_sync_f`][dis-sync-f].
    - [`async_dispatch`][async-dis] and [`sync_dispatch`][sync-dis] take [_Rust closures_][rs-closure], instead of [Apple's _block_][apple-block], as one of their parameters.
    - The [_Rust closure_][rs-closure] (it's actually a struct) will be `box`ed, which means the _closure_ will be moved into heap, so the _closure_ cannot be optimized as _inline_ code. (Need to find a way to optimize it?)
    - Since the _closure_ will be run on an asynchronous thread, we need to move the _closure_ to heap to make sure it's alive and then it will be destroyed after the task of the _closure_ is done.
- Borrowing Issues
  1. Pass `AudioUnitContext` across threads. In _C_ version, we [pass the pointer to `cubeb` context across threads][cubeb-au-ptr-across-threads], but it's forbidden in _Rust_. A workarounds are
      1. Cast the pointer to a `usize` value so the value can be copied to another thread.
      2. Or Unsafely implements `Send` and `Sync` traits so the compiler ignores the checks.
  2. We have a [`mutex`][ocs-rust] in `AudioUnitContext`, and we have a _reference_ to `AudioUnitContext` in `AudioUnitStream`. To sync what we do in [_C version_][cubeb-au-init-stream], we need to _lock_ the `mutex` in `AudioUnitContext` then pass a _reference_ to `AudioUnitContext` to `AudioUnitStream::new(...)`. To _lock_ the `mutex` in `AudioUnitContext`, we call `AutoLock::new(&mut AudioUnitContext.mutex)`. That is, we will borrow a reference to `AudioUnitContext` as a mutable first then borrow it again. It's forbidden in _Rust_. Some workarounds are
      1. Replace `AutoLock` by calling `mutex.lock()` and `mutex.unlock()` explicitly.
      2. Save the pointer to `mutex` first, then call `AutoLock::new(unsafe { &mut (*mutex_ptr) })`.
      3. Cast immutable reference to a `*const` then to a `*mut`: `pthread_mutex_lock(&self.mutex as *const pthread_mutex_t as *mut pthread_mutex_t)`

### Test issues
- Complexity of creating unit tests
    - We have lots of dependent APIs, so it's hard to test one API only, specially for those APIs using mutex(`OwnedCriticalSection` actually)
    - It's better to split them into several APIs so it's easier to test them
- Fail to run `test_create_blank_aggregate_device` with `test_add_device_listeners_dont_affect_other_scopes_with_*` at the same time
  - I guess `audiounit_create_blank_aggregate_device` will fire the callbacks in `test_add_device_listeners_dont_affect_other_scopes_with_*`
- Fail to run `test_configure_{input, output}_with_zero_latency_frames` and `test_configure_{input, output}` at the same time.
  - The APIs depending on `audiounit_set_buffer_size` cannot be called in parallel
    - `kAudioDevicePropertyBufferFrameSize` cannot be set when another stream using the same device with smaller buffer size is active. See [here][chg-buf-sz] for reference.
    - The *buffer frame size* within same device may be overwritten (For those *AudioUnit*s using same device ?)
- Fail to run `test_ops_context_register_device_collection_changed_twice_*` on my MacBook Air and Travis CI.
  - A panic in `capi_register_device_collection_changed` causes `EXC_BAD_INSTRUCTION`.
  - Works fine if replacing `register_device_collection_changed: Option<unsafe extern "C" fn(..,) -> c_int>` to `register_device_collection_changed: unsafe extern "C" fn(..,) -> c_int`
  - Test them in `AudioUnitContext` directly instead of calling them via `OPS` for now.

[cubeb]: https://github.com/kinetiknz/cubeb "Cross platform audio library"
[cubeb]: https://github.com/kinetiknz/cubeb/blob/master/src/cubeb.c "cubeb.c"
[cubeb-au]: https://github.com/kinetiknz/cubeb/blob/master/src/cubeb_audiounit.cpp "Cubeb AudioUnit"

[integrate-with-cubeb]: https://github.com/ChunMinChang/cubeb-coreaudio-rs/commit/e84c554f18ef054376134c79a112a84cb8f923b4 "patch for integrating within cubeb"

[ocs]: https://github.com/kinetiknz/cubeb/blob/master/src/cubeb_utils_unix.h "owned_critical_section"
[ocs-rust]: src/backend/owned_critical_section.rs "OwnedCriticalSection"

[dis-sync]: https://developer.apple.com/documentation/dispatch/1452870-dispatch_sync "dispatch_sync"
[dis-async]: https://developer.apple.com/documentation/dispatch/1453057-dispatch_async "dispatch_async"
[dis-async-f]: https://developer.apple.com/documentation/dispatch/1452834-dispatch_async_f "dispatch_async_f"
[dis-sync-f]: https://developer.apple.com/documentation/dispatch/1453123-dispatch_sync_f "dispatch_sync_f"
[dis-block]: https://developer.apple.com/documentation/dispatch/dispatch_block_t?language=objc "dispatch_block_t"
[c-ext-block]: https://en.wikipedia.org/wiki/Blocks_(C_language_extension) "Blocks: C language extension"
[apple-block]: https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ProgrammingWithObjectiveC/WorkingwithBlocks/WorkingwithBlocks.html "Working with Blocks"
[rs-closure]: https://doc.rust-lang.org/book/second-edition/ch13-01-closures.html "Closures"
[sync-dis]: src/backend/dispatch_utils.rs
[async-dis]: src/backend/dispatch_utils.rs
[osx-dis-gist]: https://gist.github.com/ChunMinChang/8d13946ebc6c95b2622466c89a0c9bcc "gist"

[cubeb-au-ptr-across-threads]: https://github.com/kinetiknz/cubeb/blob/9a7a55153e7f9b9e0036ab023909c7bc4a41688b/src/cubeb_audiounit.cpp#L3454-L3480 "Pass pointers across threads"
[cubeb-au-init-stream]: https://github.com/kinetiknz/cubeb/blob/9a7a55153e7f9b9e0036ab023909c7bc4a41688b/src/cubeb_audiounit.cpp#L2745-L2748 "Init stream"

[cubeb-rs]: https://github.com/djg/cubeb-rs "cubeb-rs"
[cubeb-rs-stmparamsref]: https://github.com/djg/cubeb-rs/blob/78ed9459b8ac2ca50ea37bb72f8a06847eb8d379/cubeb-core/src/stream.rs#L61 "StreamParamsRef"
[cubeb-rs-capi-stm-reg-dev-chg-callback]: https://github.com/djg/cubeb-rs/blob/78ed9459b8ac2ca50ea37bb72f8a06847eb8d379/cubeb-backend/src/capi.rs#L56 "stream_register_device_changed_callback"
[cubeb-backend]: https://github.com/djg/cubeb-rs/tree/master/cubeb-backend "cubeb-backend"
[cubeb-pulse-rs]: https://github.com/djg/cubeb-pulse-rs "cubeb-pulse-rs"

[cubeb-backend-stm-reg-dev-chg-cb]: cubeb-backend-stream_register_device_changed_callback.diff "Implementation of stream_register_device_changed_callback"
[cubeb-pulse-rs-reg-dev-chg-cb]: cubeb-pulse-rs-register_device_changed_callback.diff "Impelement of register_device_changed_callback"

[chg-buf-sz]: https://cs.chromium.org/chromium/src/media/audio/mac/audio_manager_mac.cc?l=982-989&rcl=0207eefb445f9855c2ed46280cb835b6f08bdb30 "issue on changing buffer size"

[bugzilla-cars]: https://bugzilla.mozilla.org/show_bug.cgi?id=1530715 "Bug 1530715 - Implement CoreAudio backend for Cubeb in Rust"
[build-within-gecko]: https://github.com/ChunMinChang/gecko-dev/commits/cubeb-coreaudio-rs

[discussion]: https://docs.google.com/document/d/1ZP6R7d5S9I_8bXOXhplnO6qFM1X4VokWtE7w8ExgJEQ/edit?ts=5c6d5f09

[rust-58881]: https://github.com/rust-lang/rust/issues/58881