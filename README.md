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
- Multi thread: Find a replacement for `OwnedCriticalSection`
  - a dummy mutex like `Mutex<u8>` should work, but we don't have replacement for `assert_current_thread_owns`

[cubeb]: https://github.com/kinetiknz/cubeb "Cross platform audio library"
[cubeb-au]: https://github.com/kinetiknz/cubeb/blob/master/src/cubeb_audiounit.cpp "Cubeb AudioUnit"