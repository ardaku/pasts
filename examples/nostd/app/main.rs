// Shim for providing async main and handling platform-specific API differences

#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(default_alloc_error_handler)]
#![feature(core_c_str)]

// Logging mechanism
mod log {
    #[link(name = "c")]
    extern "C" {
        fn puts(s: *const ()) -> i32;
    }

    pub fn println(text: &core::ffi::CStr) {
        unsafe {
            puts(text.as_ptr().cast());
        }
    }
}

mod __ {
    use core::ffi::CStr;

    use one_alloc::Allocator;

    use super::log;

    const INNER_ARC_SIZE: usize = core::mem::size_of::<usize>() * 2;

    #[global_allocator]
    static ALLOCATOR: Allocator<INNER_ARC_SIZE> = Allocator::new();

    use core::panic::PanicInfo;

    #[panic_handler]
    fn panic(_panic: &PanicInfo<'_>) -> ! {
        log::println(CStr::from_bytes_with_nul(b"panicked!\0").unwrap());
        loop {}
    }

    #[lang = "eh_personality"]
    extern "C" fn eh_personality() {}

    #[no_mangle]
    extern "C" fn main() -> ! {
        loop {
            let executor = pasts::Executor::default();
            executor.spawn(super::main::main::main(executor.clone()));
            log::println(CStr::from_bytes_with_nul(b"main exited!\0").unwrap());
        }
    }
}

#[allow(unused_imports)]
use self::main::*;

mod main {
    include!("../src/main.rs");

    #[allow(clippy::module_inception)]
    pub(super) mod main {
        pub(in super::super) fn main(
            executor: pasts::Executor,
        ) -> impl core::future::Future<Output = ()> + Unpin {
            super::main(&executor)
        }
    }
}
