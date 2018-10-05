// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.
#[macro_use]
extern crate cubeb_backend;

mod backend;
mod capi;

pub use capi::audiounit_rust_init;
