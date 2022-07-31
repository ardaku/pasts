// Purposely not included when publishing; for tests/examples only
fn main() {
    let async_main = format!("{}/main.rs", std::env::var("OUT_DIR").unwrap());
    std::fs::write(
        async_main,
        r#"
        #[cfg_attr(feature = "pasts/web", wasm_bindgen(start))]
        pub fn main() {
            let executor = Executor::default();
            executor.spawn(App::main(executor.clone()));
        }"#,
    )
    .unwrap();
}
