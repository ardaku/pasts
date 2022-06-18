//! Use pasts with no-std + faux_alloc.

#![no_std]
#![no_main]

use pasts::prelude::*;

#[link(name = "c")]
extern "C" {
    fn puts(s: *const ()) -> i32;
}

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

async fn app() {
    // unsafe: null terminated and static string
    unsafe { puts(b"Hello, world!\0".as_ptr().cast()) };
}

#[no_mangle]
pub extern "C" fn main() {
    let app = &mut app();
    // unsafe: main runs for the entirety of the runtime, making app: 'static
    let app = Box::into_pin(unsafe { Box::from_raw(app) });
    Executor::default().spawn(app);
}
