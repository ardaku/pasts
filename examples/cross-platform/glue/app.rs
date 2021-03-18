////////////////////////////////////////////////////////////////////////////////
//                        Platform-specific glue code                         //
////////////////////////////////////////////////////////////////////////////////

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
#[path = "../src/app.rs"]
mod app;

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
fn main() {
    pasts::block_on(app::run());
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
mod _glue_entry_point {
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn main() {
        super::main()
    }

    #[cfg(target_os = "android")]
    extern "C" fn android_main(_state: *mut core::ffi::c_void) {
        super::main()
    }
}
