use std::fs;

fn main() -> std::io::Result<()> {
    fs::create_dir_all("./src/doc/")?;
    fs::copy("./examples/counter/app/main.rs", "./src/doc/app.rs")?;
    fs::copy("./examples/counter/src/main.rs", "./src/doc/counter.rs")?;
    fs::copy("./examples/spawn/src/main.rs", "./src/doc/spawn.rs")?;
    fs::copy("./examples/slices/src/main.rs", "./src/doc/slices.rs")?;
    fs::copy("./examples/tasks/src/main.rs", "./src/doc/tasks.rs")?;
    Ok(())
}
