use std::fs;

fn main() -> std::io::Result<()> {
    fs::remove_dir_all("./gen-docs/")?;
    fs::create_dir_all("./gen-docs/")?;
    fs::copy("./examples/counter/build.rs", "./gen-docs/build.rs")?;
    fs::copy("./examples/counter/src/main.rs", "./gen-docs/counter.rs")?;
    fs::copy("./examples/spawn.rs", "./gen-docs/spawn.rs")?;
    fs::copy("./examples/slices.rs", "./gen-docs/slices.rs")?;
    fs::copy("./examples/tasks.rs", "./gen-docs/tasks.rs")?;
    Ok(())
}
