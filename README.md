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
- Multi thread: Find a replacement for `OwnedCriticalSection`
  - a dummy mutex like `Mutex<()>` should work (see `test_dummy_mutex_multithread`), but we don't have replacement for `assert_current_thread_owns`
- Write a wrapper to replace `dispatch_async`?
  - The second parameter of `dispatch_async` is `dispatch_block_t`, which is defined by `typedef void (^dispatch_block_t)(void)`.
  - The caret symbol `^` defines a [block](https://en.wikipedia.org/wiki/Blocks_(C_language_extension)).
  - The _block_ is a lambda expression-like syntax to create closures. (See Apple's document: [Working with Blocks](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ProgrammingWithObjectiveC/WorkingwithBlocks/WorkingwithBlocks.html))
  - Not sure if Rust can work with it. Rust has its own _closure_.
  - May be we can replace `dispatch_async` by `dispatch_async_f`?
    - Write a wrapper for `dispatch_async_f` and pass _Rust closures_ (instead of Apple's *block*) as parameter
    - prototype on [gist](https://gist.github.com/ChunMinChang/8d13946ebc6c95b2622466c89a0c9bcc)

[cubeb]: https://github.com/kinetiknz/cubeb "Cross platform audio library"
[cubeb-au]: https://github.com/kinetiknz/cubeb/blob/master/src/cubeb_audiounit.cpp "Cubeb AudioUnit"