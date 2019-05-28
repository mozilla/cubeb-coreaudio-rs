# cubeb-coreaudio-rs

[![Build Status](https://travis-ci.org/ChunMinChang/cubeb-coreaudio-rs.svg?branch=trailblazer)](https://travis-ci.org/ChunMinChang/cubeb-coreaudio-rs)

*Rust* implementation of [Cubeb][cubeb] on the MacOS platform.

## Current Goals
- Rewrite the [C code][cubeb-au] into *Rust* on a line-by-line basis
- Create some tests for later refactoring
- Defuse the `OwnedCriticalSection`. See [proposal][mutex-disposal] here.

## Status

All the lines in [*cubeb_audiounit.cpp*][cubeb-au] are translated.

By applying the [patch][integrate-with-cubeb] to integrate within [Cubeb][cubeb],
it can pass all the tests under *cubeb/test*.

The plain translation version from the C code
is on [plain-translation-from-c][translation-from-c] branch.
The working draft version is on [trailblazer][blazer] branch.
Both branches can pass all the tests on tryserver for firefox.
However, we are replacing our custom mutex,
which is translated from C version directly,
by standard Rust mutex.
The code is on [ocs-disposal][ocs-disposal] branch and [ocs-disposal-stm][ocs-disposal-stm] branch.

The project can also be tracked on [*bugzilla* 1530715][bugzilla-cars].
The [instructios][bugzilla-cars-instruction] to integrate this project into firefox gecko can be found there.
You can also find the formal patches and the reviews there.
The easiest way to integrate this project into firefox gecko is to apply all the patches.

### Defusing the custom mutex
Now all the custom mutexes in cubeb context is replaced.
The code is on [ocs-disposal][ocs-disposal] branch.

The replacement for the custom mutexes in cubeb stream is still a work in process.
The code is in [ocs-disposal-stm][ocs-disposal-stm] branch.

## Test
Please run `sh run_tests.sh`.

Some tests cannot be run in parallel.
They may operate the same device at the same time,
or indirectly fire some system events that are listened by some tests.

The tests that may affect others are marked `#[ignore]`.
They will be run by `cargo test ... -- --ignored ...`
after finishing normal tests.
Most of the tests are executed in `run_tests.sh`.
Only those tests commented with *FIXIT* are left.

### Device Switching
The system default device will be changed during our tests.
All the available devices will take turns being the system default device.
However, after finishing the tests, the default device will be set to the original one.
The sounds in the tests should be able to continue whatever the system default device is.

### Device Plugging/Unplugging
We implement APIs simulating plugging or unplugging a device
by adding or removing an aggregate device programmatically.
It's used to verify our callbacks for minitoring the system devices work.

### Manual Test
- Output devices switching
  - `$ cargo test test_switch_output_device -- --ignored --nocapture`
  - Enter `s` to switch output devices
  - Enter `q` to finish test
- Device change events listener
  - `$ cargo test test_add_then_remove_listeners -- --ignored --nocapture`
  - Plug/Unplug devices or switch input/output devices to see events log.
- Device collection change
  - `cargo test test_device_collection_change -- --ignored --nocapture`
  - Plug/Unplug devices to see events log.

## TODO
See [TO-DOs][todo]

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
- Mutex Borrowing Issues
  - We have a [`mutex`][ocs-rust] in `AudioUnitContext`, and we have a _reference_ to `AudioUnitContext` in `AudioUnitStream`. To sync what we do in [_C version_][cubeb-au-init-stream], we need to _lock_ the `mutex` in `AudioUnitContext` then pass a _reference_ to `AudioUnitContext` to `AudioUnitStream::new(...)`.
  - To _lock_ the `mutex` in `AudioUnitContext`, we call `AutoLock::new(&mut AudioUnitContext.mutex)`. That is, we will borrow a reference to `AudioUnitContext` as a mutable first then borrow it again. It's forbidden in _Rust_.
  - Some workarounds are
      1. Replace `AutoLock` by calling `mutex.lock()` and `mutex.unlock()` explicitly.
      2. Save the pointer to `mutex` first, then call `AutoLock::new(unsafe { &mut (*mutex_ptr) })`.
      3. Cast immutable reference to a `*const` then to a `*mut`: `pthread_mutex_lock(&self.mutex as *const pthread_mutex_t as *mut pthread_mutex_t)`
- No guarantee on `audiounit_set_channel_layout`
  - This call doesn't work all the times
  - Returned `NO_ERR` doesn't guarantee the layout is set to the one we want
  - The layouts on some devices won't be changed even no errors are returned,
    e.g., we can set _stereo_ layout to a _4-channels aggregate device_ with _QUAD_ layout
    (created by Audio MIDI Setup) without any error. However, the layout
    of this 4-channels aggregate device is still QUAD after setting it without error
  - Another weird thing is that we will get a `kAudioUnitErr_InvalidPropertyValue`
    if we set the layout to _QUAD_. It's the same layout as its original one but it cannot be set!

### Test issues
- Complexity of creating unit tests
    - We have lots of dependent APIs, so it's hard to test one API only, specially for those APIs using mutex(`OwnedCriticalSection` actually)
    - It's better to split them into several APIs so it's easier to test them
- Fail to run `test_create_blank_aggregate_device` with `test_add_device_listeners_dont_affect_other_scopes_with_*` at the same time
  - `audiounit_create_blank_aggregate_device` will fire the callbacks in `test_add_device_listeners_dont_affect_other_scopes_with_*`
- Fail to run `test_configure_{input, output}_with_zero_latency_frames` and `test_configure_{input, output}` at the same time.
  - The APIs depending on `audiounit_set_buffer_size` cannot be called in parallel
    - `kAudioDevicePropertyBufferFrameSize` cannot be set when another stream using the same device with smaller buffer size is active. See [here][chg-buf-sz] for reference.
    - The *buffer frame size* within same device may be overwritten (For those *AudioUnit*s using same device ?)
- Fail to run `test_ops_context_register_device_collection_changed_twice_*` on my MacBook Air and Travis CI.
  - A panic in `capi_register_device_collection_changed` causes `EXC_BAD_INSTRUCTION`.
  - Works fine if replacing `register_device_collection_changed: Option<unsafe extern "C" fn(..,) -> c_int>` to `register_device_collection_changed: unsafe extern "C" fn(..,) -> c_int`
  - Test them in `AudioUnitContext` directly instead of calling them via `OPS` for now.
- `TestDeviceSwitcher` cannot work when there is an alive full-duplex stream
  - An aggregate device will be created for a duplex stream when its input and output devices are different.
  - `TestDeviceSwitcher` will cached the available devices, upon it's created, as the candidates for default device
  - Hence the created aggregate device may be cached in `TestDeviceSwitcher`
  - If the aggregate device is destroyed (when the destroying the duplex stream created it) but the `TestDeviceSwitcher` is still working,
    it will set a destroyed device as the default device
  - See details in [device_change.rs](src/backend/tests/device_change.rs)

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

[cubeb-au-init-stream]: https://github.com/kinetiknz/cubeb/blob/9a7a55153e7f9b9e0036ab023909c7bc4a41688b/src/cubeb_audiounit.cpp#L2745-L2748 "Init stream"

[chg-buf-sz]: https://cs.chromium.org/chromium/src/media/audio/mac/audio_manager_mac.cc?l=982-989&rcl=0207eefb445f9855c2ed46280cb835b6f08bdb30 "issue on changing buffer size"

[bugzilla-cars]: https://bugzilla.mozilla.org/show_bug.cgi?id=1530715 "Bug 1530715 - Implement CoreAudio backend for Cubeb in Rust"
[bugzilla-cars-instruction]: https://bugzilla.mozilla.org/show_bug.cgi?id=1530715#c4
[build-within-gecko]: https://github.com/ChunMinChang/gecko-dev/commits/cubeb-coreaudio-rs

[discussion]: https://docs.google.com/document/d/1ZP6R7d5S9I_8bXOXhplnO6qFM1X4VokWtE7w8ExgJEQ/edit?ts=5c6d5f09

[rust-58881]: https://github.com/rust-lang/rust/issues/58881

[mutex-disposal]: mutex-disposal.md

[translation-from-c]: https://github.com/ChunMinChang/cubeb-coreaudio-rs/tree/plain-translation-from-c
[blazer]: https://github.com/ChunMinChang/cubeb-coreaudio-rs/tree/trailblazer
[ocs-disposal]: https://github.com/ChunMinChang/cubeb-coreaudio-rs/tree/ocs-disposal
[ocs-disposal-stm]: https://github.com/ChunMinChang/cubeb-coreaudio-rs/tree/ocs-disposal-stm

[todo]: todo.md