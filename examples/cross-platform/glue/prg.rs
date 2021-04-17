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

#[cfg(target_arch = "wasm32")]
fn main() {
    const LOG: devout::Tag = devout::Tag::new("LOG");

    devout::log!(LOG, "oof");
    std::thread::park();
    devout::log!(LOG, "da oof");
}
