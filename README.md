# cubeb-coreaudio-rs

Implementation of MacOS Audio backend in CoreAudio framework for [Cubeb][cubeb] written in Rust.

## Current Goals
- Translate [C code][cubeb-au] line by line into Rust
- Create tests for later refactoring

## TODO
- Test aggregate devices
- Test for stream operations
- Clean up the tests. Merge the duplicated pieces in to a function.
- Find a way to catch memory leaks
  - Try Instrument on OSX

## Issues
- Mutex: Find a replacement for [`owned_critical_section`][osc]
  - A dummy mutex like `Mutex<()>` should work (see `test_dummy_mutex_multithread`) as what `owned_critical_section` does in [_C_ version][osc], but it doens't has equivalent API for `assert_current_thread_owns`.
  - We implement a [`OwnedCriticalSection` around `pthread_mutex_t`][ocs-rust] like what we do in [_C_ version][osc] for now.
- Unworkable API: [`dispatch_async`][dis-async]
  - The second parameter of [`dispatch_async`][dis-async] is [`dispatch_block_t`][dis-block], which is defined by `typedef void (^dispatch_block_t)(void)`.
  - The caret symbol `^` defines a [block][c-ext-block].
  - The _block_ is a lambda expression-like syntax to create closures. (See Apple's document: [Working with Blocks][apple-block])
  - Not sure if _Rust_ can work with it. _Rust_ has its own [_closure_][rs-closure].
  - For now, we implement an API [`async_dispatch`][async-dis] to replace [`dispatch_async`][dis-async] (prototype on [gist][async-dis-gist].)
    - [`async_dispatch`][async-dis] is based on [`dispatch_async_f`][dis-async-f].
    - [`async_dispatch`][async-dis] takes a [_Rust closure_][rs-closure], instead of [Apple's _block_][apple-block], as one of its parameter.
    - The [_Rust closure_][rs-closure] (it's actually a struct) will be `box`ed, which means the _closure_ will be moved into heap, so the _closure_ cannot be optimized as _inline_ code. (Need to find a way to optimize it?)
    - Since the _closure_ will be run on an asynchronous thread, we need to move the _closure_ to heap to make sure it's alive and then it will be destroyed after the task of the _closure_ is done.
- Borrowing Issues
  1. Pass `AudioUnitContext` across threads. In _C_ version, we [pass the pointer to `cubeb` context across threads][cubeb-au-ptr-across-threads], but it's forbidden in _Rust_. A workaround here is to
    1. Cast the pointer to a `cubeb` context into a `usize` value
    2. Pass the value to threads. The value is actually be copied into the code-block that will be run on another thread
    3. When the task on another thread is run, the value is casted to a pointer to a `cubeb` context
  2. We have a [`mutex`][ocs-rust] in `AudioUnitContext`, and we have a _reference_ to `AudioUnitContext` in `AudioUnitStream`. To sync what we do in [_C version_][cubeb-au-init-stream], we need to _lock_ the [`mutex`][ocs-rust] in `AudioUnitContext` then pass a _reference_ to `AudioUnitContext` to `AudioUnitStream::new(...)`. To _lock_ the [`mutex`][ocs-rust] in `AudioUnitContext`, we call [`AutoLock::new(&mut AudioUnitContext.mutex)`][ocs-rust]. That is, we will borrow a reference to `AudioUnitContext` as a mutable first then borrow it again. It's forbidden in _Rust_ to do that. A workaround here is to
    1. Either replace `AutoLock` by calling `mutex.lock()` and `mutex.unlock()` explicitly.
    2. Or save the pointer to `mutex` first, then use `AutoLock::new(unsafe { &mut (*mutex_ptr) })`

[cubeb]: https://github.com/kinetiknz/cubeb "Cross platform audio library"
[cubeb-au]: https://github.com/kinetiknz/cubeb/blob/master/src/cubeb_audiounit.cpp "Cubeb AudioUnit"

[ocs]: https://github.com/kinetiknz/cubeb/blob/master/src/cubeb_utils_unix.h "owned_critical_section"
[ocs-rust]: src/backend/owned_critical_section.rs "OwnedCriticalSection"

[dis-async]: https://developer.apple.com/documentation/dispatch/1453057-dispatch_async "dispatch_async"
[dis-async-f]: https://developer.apple.com/documentation/dispatch/1452834-dispatch_async_f "dispatch_async_f"
[dis-block]: https://developer.apple.com/documentation/dispatch/dispatch_block_t?language=objc "dispatch_block_t"
[c-ext-block]: https://en.wikipedia.org/wiki/Blocks_(C_language_extension) "Blocks: C language extension"
[apple-block]: https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ProgrammingWithObjectiveC/WorkingwithBlocks/WorkingwithBlocks.html "Working with Blocks"
[rs-closure]: https://doc.rust-lang.org/book/second-edition/ch13-01-closures.html "Closures"
[async-dis]: src/backend/async_dispatch.rs
[async-dis-gist]: https://gist.github.com/ChunMinChang/8d13946ebc6c95b2622466c89a0c9bcc "gist"

[cubeb-au-ptr-across-threads]: https://github.com/kinetiknz/cubeb/blob/9a7a55153e7f9b9e0036ab023909c7bc4a41688b/src/cubeb_audiounit.cpp#L3454-L3480 "Pass pointers across threads"
[cubeb-au-init-stream]: https://github.com/kinetiknz/cubeb/blob/9a7a55153e7f9b9e0036ab023909c7bc4a41688b/src/cubeb_audiounit.cpp#L2745-L2748 "Init stream"