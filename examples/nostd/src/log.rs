//! Logging implementation: link to `puts()`

use core::ffi::CStr;

/// Log a message.  Requires trailing null byte.
pub fn log(text: &str) {
    let text = CStr::from_bytes_with_nul(text.as_bytes()).unwrap();

    #[link(name = "c")]
    extern "C" {
        fn puts(s: *const ()) -> i32;
    }

    unsafe { puts(text.as_ptr().cast()) };
}
