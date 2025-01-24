use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::error::Error;

/// Add all subdirectories and files of `dir` to the watch list
/// if they don't start with "target"
fn add_non_target_watch_list(dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    for entry in dir.read_dir()? {
        let entry = entry?;
        let file_name = entry.file_name().to_str().ok_or("File name contains invalid character")?.to_string();
        let path = entry.path();

        if !(path.is_dir() && file_name.starts_with("target")) {
            println!("cargo:rerun-if-changed={}", path.to_str().ok_or("Path contains invalid character")?);
        }
    }
    Ok(())
}

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

    // Add everything in source to the watch list except for target*
    // TOOD: might be incorrect for future versions of Verus
    add_non_target_watch_list(&verus_dir.join("source"))?;
    add_non_target_watch_list(&verus_dir.join("tools").join("vargo"))?;

    // Download Z3 using a script in Verus (source/tools/get-z3.sh)
    // TODO: write this directly in Rust
    if !verus_dir.join("source").join("z3").exists() {
        if !Command::new("sh")
            .current_dir(verus_dir.join("source"))
            .args(["tools/get-z3.sh"])
            .status()?.success() {
            Err("Failed to download Z3")?;
        }
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
