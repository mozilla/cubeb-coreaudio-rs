// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

use crate::backend::{AudioUnitContext, AudioUnitStream};
use coreaudio_sys_utils::sys::os_workgroup_t;
use cubeb_backend::{capi, ffi};
use std::os::raw::{c_char, c_int};
use std::ptr;

/// # Safety
///
/// This function should only be called once per process.
#[no_mangle]
pub unsafe extern "C" fn audiounit_rust_init(
    c: *mut *mut ffi::cubeb,
    context_name: *const c_char,
) -> c_int {
    capi::capi_init::<AudioUnitContext>(c, context_name)
}

/// Returns a retained (+1) reference to the audio workgroup associated with
/// `stm`'s output device, or the input device for input-only streams.  Returns
/// NULL if `stm` is NULL or no workgroup is available.
///
/// The caller owns the returned reference and must release it with `os_release`.
///
/// # Safety
///
/// `stm` must be a valid cubeb stream pointer created by this backend, or NULL.
#[no_mangle]
pub unsafe extern "C" fn audiounit_stream_get_workgroup(
    stm: *mut ffi::cubeb_stream,
) -> os_workgroup_t {
    if stm.is_null() {
        return ptr::null_mut();
    }
    let stream = &*(stm as *const AudioUnitStream);
    stream.workgroup_retained()
}
