extern crate coreaudio_sys;

pub mod string;

// Re-export coreaudio-sys types
pub use coreaudio_sys::*;
pub use string::*;
