// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

use backend::TestContext;
// cubeb_backend::{*} is is referred:
// - capi   : cubeb_backend::capi   (cubeb-core/capi.rs)
// - ffi    : cubeb_sys::*          (cubeb-core/lib.rs).
use cubeb_backend::{capi, ffi};
use std::os::raw::{c_char, c_int};

/// Entry point from C code.
#[no_mangle]
pub unsafe extern "C" fn audiounit_rust_init(
    // `ffi::cubeb` is refered to `pub enum cubeb` (cubeb-sys/context.rs):
    // cubeb_backend::ffi (cubeb-backend/capi.rs)
    // -> cubeb_core::ffi (cubeb-core/lib.rs)
    // -> cubeb_sys::context::cubeb (cubeb-sys/context.rs).
    c: *mut *mut ffi::cubeb,
    context_name: *const c_char,
) -> c_int {
    // `capi::capi_init` is referred to `cubeb_backend::capi_init`(cubeb-backend/capi.rs).
    capi::capi_init::<TestContext>(c, context_name)
}
