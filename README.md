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
- Mutex: Find a replacement for [`owned_critical_section`](https://github.com/kinetiknz/cubeb/blob/master/src/cubeb_utils_unix.h)
  - a dummy mutex like `Mutex<()>` should work (see `test_dummy_mutex_multithread`) as what `owned_critical_section` does in _C_ version, but it doens't has similar API like `assert_current_thread_owns`.
  - We implement a `OwnedCriticalSection` around `pthread_mutex_t` like what we do in _C_ version for now.
- Unworkable API: `dispatch_async`
  - The second parameter of `dispatch_async` is `dispatch_block_t`, which is defined by `typedef void (^dispatch_block_t)(void)`.
  - The caret symbol `^` defines a [block](https://en.wikipedia.org/wiki/Blocks_(C_language_extension)).
  - The _block_ is a lambda expression-like syntax to create closures. (See Apple's document: [Working with Blocks](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ProgrammingWithObjectiveC/WorkingwithBlocks/WorkingwithBlocks.html))
  - Not sure if Rust can work with it. Rust has its own _closure_.
  - For now, we implement an API `async_dispatch` to replace `dispatch_async`
    - `async_dispatch` is based on `dispatch_async_f`.
    - `async_dispatch` takes a _Rust closure_ (instead of Apple's *block*) as one of its parameter.
    - prototype on [gist](https://gist.github.com/ChunMinChang/8d13946ebc6c95b2622466c89a0c9bcc)
      - The _closure_ (it's a struct) will be `box`ed, which means the _closure_ will be moved into heap, so the _closure_ cannot be optimized as *inline* code.
      - Since the _closure_ will be run on an asynchronous thread, we need to move the _closure_ to heap in case it's destroyed when the other thread want to use it.

[cubeb]: https://github.com/kinetiknz/cubeb "Cross platform audio library"
[cubeb-au]: https://github.com/kinetiknz/cubeb/blob/master/src/cubeb_audiounit.cpp "Cubeb AudioUnit"