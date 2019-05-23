// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

// Use Atomic{I64, U32, U64} once they are stable.
// #![feature(integer_atomics)]

extern crate atomic;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate cubeb_backend;

mod backend;
mod capi;

pub use crate::capi::audiounit_rust_init;
