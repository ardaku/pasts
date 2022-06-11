use std::fs;

fn main() -> std::io::Result<()> {
    fs::create_dir_all("./gen-docs/")?;
    fs::copy("./examples/counter/app/main.rs", "./gen-docs/app.rs")?;
    fs::copy("./examples/counter/src/main.rs", "./gen-docs/counter.rs")?;
    fs::copy("./examples/spawn/src/main.rs", "./gen-docs/spawn.rs")?;
    fs::copy("./examples/slices/src/main.rs", "./gen-docs/slices.rs")?;
    fs::copy("./examples/tasks/src/main.rs", "./gen-docs/tasks.rs")?;
    Ok(())
}
