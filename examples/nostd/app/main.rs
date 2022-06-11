// Shim for providing async main and handling platform-specific API differences

#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(default_alloc_error_handler)]
#![feature(core_c_str)]
#![feature(pin_macro)]

//// typical no-std boilerplate ////

mod __ {
    const INNER_ARC_SIZE: usize = ::core::mem::size_of::<usize>() * 2;

    #[global_allocator]
    static ALLOCATOR: ::one_alloc::Allocator<INNER_ARC_SIZE> =
        ::one_alloc::Allocator::new();

    #[panic_handler]
    fn panic(_panic: &::core::panic::PanicInfo<'_>) -> ! {
        loop {}
    }

    #[lang = "eh_personality"]
    extern "C" fn eh_personality() {}

    #[no_mangle]
    extern "C" fn main() -> ! {
        super::main::main::main();
        crate::log::log("main exited!\0");
        unreachable!()
    }
}

//// optional async main shim ////

#[allow(unused_imports)]
use self::main::*;

mod main {
    include!("../src/main.rs");

    #[allow(clippy::module_inception)]
    pub(super) mod main {
        pub(in super::super) fn main() {
            let mut join = super::main();

            // unsafe: Sound because main() is around for 'static
            let join: *mut _ = &mut join;
            let join: &'static mut _ = unsafe { &mut *join };

            ::pasts::Executor::default()
                .spawn(::core::pin::Pin::static_mut(join));
        }
    }
}
