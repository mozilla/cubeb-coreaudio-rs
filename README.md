# cubeb-coreaudio-rs

Implementation of MacOS Audio backend in CoreAudio framework for [Cubeb][cubeb] written in Rust.

## Current Goals
- Translate [C code][cubeb-au] line by line into Rust
- Create tests for later refactoring

[cubeb]: https://github.com/kinetiknz/cubeb "Cross platform audio library"
[cubeb-au]: https://github.com/kinetiknz/cubeb/blob/master/src/cubeb_audiounit.cpp "Cubeb AudioUnit"