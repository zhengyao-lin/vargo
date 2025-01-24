use std::io;
use std::fs;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::error::Error;

const DEFAULT_VERUS_REPO: &str = "https://github.com/verus-lang/verus.git";
const DEFAULT_VERUS_COMMIT: &str = "0eedcf063d07ea0359ebece1f142163b8a87e361";

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

/// Build Verus at a checked-out repo
fn build_verus(path: &Path) -> Result<(), Box<dyn Error>> {
    // Download Z3 using a script in Verus (source/tools/get-z3.sh)
    // TODO: write this directly in Rust
    if !path.join("source").join("z3").exists() {
        if !Command::new("sh")
            .current_dir(path.join("source"))
            .args(["tools/get-z3.sh"])
            .status()?.success() {
            Err("Failed to download Z3")?;
        }
    }

    // First call cargo to build verus's internal vargo
    if !Command::new("cargo")
        .current_dir(path.join("tools").join("vargo"))
        .args(["build", "--release"])
        .status()?.success() {
        Err("Failed to build Verus's internal vargo")?;
    }

    // Call internal vargo to build Verus
    if !Command::new(path
            .join("tools").join("vargo")
            .join("target").join("release").join("vargo"))
        .current_dir(path.join("source"))
        .env_clear()
        .env("PATH", env::var("PATH")?)
        .args(["build", "--release"])
        .status()?.success() {
        Err("Failed to build Verus")?;
    }

    Ok(())
}

// Recursively copy a directory
fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?; // Create destination directory

    // Iterate over entries
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            // Recursively copy subdirectories
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Note on environment variables:
    // There are two options for specifying which Verus version to use
    // 1. Use VERUS_REPO and optional VERUS_BRANCH and/or optional VERUS_COMMIT
    // 2. Use VERUS_LOCAL which points to an existing local checked-out Verus repo
    // If none of 1. and 2. are specified, we checkout DEFAULT_VERUS_REPO at DEFAULT_VERUS_COMMIT

    println!("cargo:rerun-if-env-changed=VERUS_REPO");
    println!("cargo:rerun-if-env-changed=VERUS_BRANCH");
    println!("cargo:rerun-if-env-changed=VERUS_COMMIT");
    println!("cargo:rerun-if-env-changed=VERUS_LOCAL");

    if env::var("VERUS_LOCAL").is_ok() && env::var("VERUS_REPO").is_ok() {
        Err("Cannot specify both VERUS_LOCAL and VERUS_REPO")?;
    }

    let tmp_verus_dir = tempdir::TempDir::new("verus")?;

    let verus_repo = if let Ok(local_repo) = env::var("VERUS_LOCAL") {
        let local_repo = PathBuf::from(local_repo);

        // For local Verus repo, we monitor its source changes
        // Add everything in source to the watch list except for target*
        // TOOD: might be incorrect for future versions of Verus
        add_non_target_watch_list(&local_repo.join("source"))?;
        add_non_target_watch_list(&local_repo.join("tools").join("vargo"))?;

        local_repo
    } else {
        // Otherwise check out a remote repo

        let tmp_verus_dir_str = tmp_verus_dir.path().to_str()
            .ok_or("Path contains invalid character")?;
        let (remote_repo, is_default) = if let Ok(remote) = env::var("VERUS_REPO") {
            (remote, false)
        } else {
            (DEFAULT_VERUS_REPO.to_string(), true)
        };

        // Clone the remote repo at `tmp_verus_dir`
        if !Command::new("git")
            .args(["clone", &remote_repo, tmp_verus_dir_str])
            .status()?.success() {
            Err("Failed to clone Verus repo")?;
        }

        // Checkout the target branch if specified
        if let Ok(branch) = env::var("VERUS_BRANCH") {
            if !Command::new("git")
                .current_dir(&tmp_verus_dir)
                .args(["fetch", "origin", &branch])
                .status()?.success() {
                Err("Failed to set Verus branch")?;
            }

            if !Command::new("git")
                .current_dir(&tmp_verus_dir)
                .args(["checkout", &branch])
                .status()?.success() {
                Err("Failed to set Verus branch")?;
            }
        }

        // Checkout the target commit if specified
        if let Ok(commit) = env::var("VERUS_COMMIT").or(
            if is_default { Ok(DEFAULT_VERUS_COMMIT.to_string()) }
            else { Err("No commit specified") } // If a VERUS_REPO is specified, then `DEFAUL_VERUS_COMMIT` is ignored
        ) {
            if !Command::new("git")
                .current_dir(&tmp_verus_dir)
                .args(["checkout", &commit])
                .status()?.success() {
                Err("Failed to set Verus commit")?;
            }
        }

        tmp_verus_dir.path().to_owned()
    };

    build_verus(&verus_repo)?;

    // Finally, copy the compiled targets to MANIFEST_DIR/target
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    copy_dir(
        &verus_repo.join("source").join("target-verus").join("release"),
        &Path::new(&manifest_dir).join("target").join("verus"),
    )?;

    Ok(())
}
