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
- Write a wrapper for `dispatch_async`?
  - The second parameter of `dispatch_async` is `dispatch_block_t`, which is defined by `typedef void (^dispatch_block_t)(void)`.
  - The caret symbol `^` defines a block but not sure if Rust can work with it.
  - Apple's document: [Working with Blocks](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ProgrammingWithObjectiveC/WorkingwithBlocks/WorkingwithBlocks.html)
  - May be we can replace it by `dispatch_async_f`?

[cubeb]: https://github.com/kinetiknz/cubeb "Cross platform audio library"
[cubeb-au]: https://github.com/kinetiknz/cubeb/blob/master/src/cubeb_audiounit.cpp "Cubeb AudioUnit"