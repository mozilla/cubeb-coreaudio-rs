# cubeb-coreaudio-rs

Implementation of MacOS Audio backend in CoreAudio framework for [Cubeb][cubeb] written in Rust.

## Current Goals
- Translate [C code][cubeb-au] line by line into Rust
- Create tests for later refactoring

## TODO
- [cubeb-rs][cubeb-rs]
  - Implement `to_owned` in [`StreamParamsRef`][cubeb-rs-stmparamsref]
  - Implement [`stream_register_device_changed_callback` in `capi_new`][cubeb-rs-capi-stm-reg-dev-chg-callback]
    - Land [this][cubeb-backend-stm-reg-dev-chg-cb] on [cubeb-backend][cubeb-backend]
    - Land [this][cubeb-pulse-rs-reg-dev-chg-cb] on [cubeb-pulse-rs][cubeb-pulse-rs]
- Integration Tests
  - Add a test-only API to change the default audio devices
  - Use above API to test the device-changed callback
- Move issues below to github issues.
- Test aggregate devices
- Test for stream operations
- Clean up the tests. Merge the duplicated pieces in to a function.
- Find a way to catch memory leaks
  - Try Instrument on OSX
- Some of bugs are found when adding tests. Search *FIXIT* to find them.
- Maybe it's better to move all `fn some_func(stm: &AudioUnitStream, ...)` functions into `impl AudioUnitStream`.
- Define `noErr` to `0`
- Add comments for APIs in `utils`

## Issues
- Mutex: Find a replacement for [`owned_critical_section`][ocs]
  - A dummy mutex like `Mutex<()>` should work (see [`test_dummy_mutex_multithread`][ocs]) as what `owned_critical_section` does in [_C_ version][ocs], but it doens't has equivalent API for `assert_current_thread_owns`.
  - We implement a [`OwnedCriticalSection` around `pthread_mutex_t`][ocs-rust] like what we do in [_C_ version][ocs] for now.
  - It's hard to debug with the variables using `OwnedCriticalSection`. Within a test with a variable using `OwnedCriticalSection` that will get a panic, if the `OwnedCriticalSection` used in the test isn't be dropped **before** where the code get a panic, then the test might get a crash in `OwnedCriticalSection` rather than the line having a panic. One example is [`test_stream_get_panic_before_releasing_mutex`](src/backend/test.rs). The tests must be created very carefully.
- Atomic:
  - The stable atomic types only support `bool`, `usize`, `isize`, and `ptr`, but we need `u64`, `i64`, and `f32`.
  - Using [atomic-rs](https://github.com/Amanieu/atomic-rs) instead.
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
  1. Pass `AudioUnitContext` across threads. In _C_ version, we [pass the pointer to `cubeb` context across threads][cubeb-au-ptr-across-threads], but it's forbidden in _Rust_. A workaround here is to
      1. Cast the pointer to a `cubeb` context into a `usize` value
      2. Pass that value to threads. The value is actually be **copied** into the code-block that will be run on another thread
      3. When the task on another thread is run, the value is casted to a pointer to a `cubeb` context
  2. We have a [`mutex`][ocs-rust] in `AudioUnitContext`, and we have a _reference_ to `AudioUnitContext` in `AudioUnitStream`. To sync what we do in [_C version_][cubeb-au-init-stream], we need to _lock_ the `mutex` in `AudioUnitContext` then pass a _reference_ to `AudioUnitContext` to `AudioUnitStream::new(...)`. To _lock_ the `mutex` in `AudioUnitContext`, we call `AutoLock::new(&mut AudioUnitContext.mutex)`. That is, we will borrow a reference to `AudioUnitContext` as a mutable first then borrow it again. It's forbidden in _Rust_. Some workarounds are
      1. Replace `AutoLock` by calling `mutex.lock()` and `mutex.unlock()` explicitly.
      2. Save the pointer to `mutex` first, then call `AutoLock::new(unsafe { &mut (*mutex_ptr) })`.
      3. Cast immutable reference to a `*const` then to a `*mut`: `pthread_mutex_lock(&self.mutex as *const pthread_mutex_t as *mut pthread_mutex_t)`
- Complexity of creating unit tests
    - We have lots of dependent APIs, so it's hard to test one API only, specially for those APIs using mutex(`OwnedCriticalSection` actually)
    - It's better to split them into several APIs so it's easier to test them

[cubeb]: https://github.com/kinetiknz/cubeb "Cross platform audio library"
[cubeb-au]: https://github.com/kinetiknz/cubeb/blob/master/src/cubeb_audiounit.cpp "Cubeb AudioUnit"

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
