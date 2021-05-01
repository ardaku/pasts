// Glue code for supporting `async fn main()`.
// FIXME: Add iOS support.

mod main {
    include!("src/main.rs");

    pub(super) mod main {
        #[inline(always)]
        pub(in super::super) fn main() {
            pasts::Executor::default().block_on(super::main())
        }
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
extern "C" fn android_main(_state: *mut core::ffi::c_void) {
    pasts::Executor::default().block_on(main::main())
}

#[cfg(not(target_os = "android"))]
#[inline(always)]
pub fn main() {
    main::main::main()
}
