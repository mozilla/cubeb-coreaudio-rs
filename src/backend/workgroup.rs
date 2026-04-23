// Copyright © 2026 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

//! RAII wrapper around `os_workgroup_t`.
//!
//! `os_workgroup_t` is a refcounted kernel object.  Functions that return a
//! workgroup (e.g. `AudioObjectGetPropertyData(kAudioDevicePropertyIOThreadOSWorkgroup)`)
//! do so with a retain count of +1, transferring ownership to the caller.
//! `WorkGroup` owns that +1 and releases it on drop.

use super::coreaudio_sys_utils::sys::{os_release, os_retain, os_workgroup_s, os_workgroup_t};
use std::fmt;
use std::os::raw::c_void;

pub struct WorkGroup(os_workgroup_t);

impl WorkGroup {
    /// Take ownership of an `os_workgroup_t` that already has a +1 retain count.
    ///
    /// Returns `None` if `wg` is null.
    ///
    /// # Safety
    ///
    /// `wg` must either be null or a valid workgroup pointer with a +1 retain
    /// that is transferred to the returned `WorkGroup`.  The caller must not
    /// release `wg` after this call.
    pub unsafe fn from_retained(wg: os_workgroup_t) -> Option<Self> {
        if wg.is_null() {
            None
        } else {
            Some(Self(wg))
        }
    }

    /// Return a fresh +1 retained reference to the underlying workgroup.  The
    /// caller is responsible for releasing it via `os_release`.
    pub fn retained(&self) -> *mut os_workgroup_s {
        unsafe { os_retain(self.0 as *mut c_void) as *mut os_workgroup_s }
    }
}

impl Drop for WorkGroup {
    fn drop(&mut self) {
        unsafe { os_release(self.0 as *mut c_void) }
    }
}

// `os_workgroup_t` is a thread-safe refcounted kernel object, so the wrapper
// is safe to send and share across threads.
unsafe impl Send for WorkGroup {}
unsafe impl Sync for WorkGroup {}

impl fmt::Debug for WorkGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WorkGroup").field(&self.0).finish()
    }
}
