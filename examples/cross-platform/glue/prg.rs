////////////////////////////////////////////////////////////////////////////////
//                        Platform-specific glue code                         //
////////////////////////////////////////////////////////////////////////////////

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
#[path = "../src/app.rs"]
mod app;

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn main() {
    pasts::block_on(app::run());
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
mod _glue_entry_point {}
