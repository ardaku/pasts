# Cross Platform
A pasts example that shows how to set up a project with the pasts executor that
runs both in web assembly and natively on your machine.

# Run Natively
```bash
cargo run
```

# Run On The Web
## Install Dependencies
```bash
cargo install wasm-bindgen-cli https
```

## Build
```bash
cargo build --target=wasm32-unknown-unknown --lib --release
mkdir -p wasm/app/
wasm-bindgen target/wasm32-unknown-unknown/release/cross-platform.wasm\
 --out-dir wasm/app --no-typescript --omit-imports --target web
```

## Run
```bash
http wasm/
```
