// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

// TODO: Remove `integer_atomics` after `AtomicU32` is stable.
#![feature(integer_atomics)]

extern crate atomic;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate cubeb_backend;

mod backend;
mod capi;

pub use capi::audiounit_rust_init;
