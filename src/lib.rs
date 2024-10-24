// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

extern crate atomic;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate cubeb_backend;
#[macro_use]
extern crate float_cmp;
extern crate mach;

extern crate num;

mod backend;
mod capi;

pub use crate::capi::audiounit_rust_init;
