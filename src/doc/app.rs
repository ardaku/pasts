// Shim for providing async main and handling platform-specific API differences
extern crate alloc;

#[allow(unused_imports)]
use self::main::*;

mod main {
    include!("../src/main.rs");

    #[allow(clippy::module_inception)]
    pub(super) mod main {
        pub(in super::super) async fn main(executor: pasts::Executor) {
            super::main(&executor).await
        }
    }
}

#[cfg_attr(
    all(target_arch = "wasm32", target_os = "none"),
    wasm_bindgen(start)
)]
pub fn main() {
    let executor = pasts::Executor::default();
    executor.spawn(Box::pin(self::main::main::main(executor.clone())));
}
