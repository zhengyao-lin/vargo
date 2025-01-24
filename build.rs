use std::env;
use std::path::Path;
use std::process::Command;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;

    // Initialize the Verus submodule via git
    if !Command::new("git")
        .current_dir(&manifest_dir)
        .args(["submodule", "update", "--init", "--recursive"])
        .status()?.success() {
        Err("Failed to initialize submodules")?;
    }

    // Build Verus
    let verus_dir = Path::new(&manifest_dir).join("verus");

    println!("cargo:rerun-if-changed={}", verus_dir.to_str().ok_or("Verus path contains invalid character")?);

    // Download Z3 using a script in Verus (source/tools/get-z3.sh)
    // TODO: write this directly in Rust
    if !Command::new("sh")
        .current_dir(verus_dir.join("source"))
        .args(["tools/get-z3.sh"])
        .status()?.success() {
        Err("Failed to download Z3")?;
    }

    // First call cargo to build verus's internal vargo
    if !Command::new("cargo")
        .current_dir(verus_dir.join("tools").join("vargo"))
        .args(["build", "--release"])
        .status()?.success() {
        Err("Failed to build Verus's internal vargo")?;
    }

    // Call internal vargo to build Verus
    if !Command::new(verus_dir
            .join("tools").join("vargo")
            .join("target").join("release").join("vargo"))
        .current_dir(verus_dir.join("source"))
        .env_clear()
        .env("PATH", env::var("PATH")?)
        .args(["build", "--release"])
        .status()?.success() {
        Err("Failed to build Verus")?;
    }

    Ok(())
}
