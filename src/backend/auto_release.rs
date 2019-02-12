use std::fmt::Debug;
use std::mem;
use std::os::raw::c_void;
use std::ptr;

#[derive(Debug)]
pub struct AutoRelease<T: Debug> {
    ptr: *mut T,
    release_func: unsafe extern fn(*mut T)
}

impl<T: Debug> AutoRelease<T> {
    pub fn new(ptr: *mut T, release_func: unsafe extern fn(*mut T)) -> Self {
        Self {
            ptr,
            release_func
        }
    }

    pub fn reset(&mut self, ptr: *mut T) {
        self.release();
        self.ptr = ptr;
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }

    fn release(&self) {
        if !self.ptr.is_null() {
            unsafe { (self.release_func)(self.ptr); }
        }
    }
}

impl<T: Debug> Drop for AutoRelease<T> {
    fn drop(&mut self) {
        self.release();
    }
}

#[test]
fn test_auto_release() {
    unsafe extern fn allocate() -> *mut c_void {
        // println!("Allocate!");
        libc::calloc(1, mem::size_of::<u32>())
    }

    unsafe extern fn deallocate(ptr: *mut c_void) {
        // println!("Deallocate!");
        libc::free(ptr);
    }

    let mut auto_release = AutoRelease::new(ptr::null_mut(), deallocate);
    let ptr = unsafe { allocate() };
    auto_release.reset(ptr);
    assert_eq!(auto_release.as_mut_ptr(), ptr);
}
