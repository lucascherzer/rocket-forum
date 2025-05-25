use std::path::Path;
use std::process::Command;
use std::{env, fs};

fn main() {
    let frontend_dir = "frontend";
    let output_dir = format!("{frontend_dir}/build");
    let target_dir = "static"; // or wherever you're serving files from
    if env::var("NIX_BUILD").ok().is_some() {
        // If we are building with nix, the frontend is already built by nix,
        // so we skip it.
        // We would not have internet access anyways, due to the sandbox.
        return;
    }
    let _status = Command::new("npm")
        .arg("install")
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to build frontend");
    // Step 1: Run `npm run build` in the frontend dir
    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to build frontend");

    if !status.success() {
        panic!("Frontend build failed");
    }

    // Step 2: Copy the dist output to the static folder
    let _ = fs::remove_dir_all(target_dir); // Clean target dir if it exists
    fs::create_dir_all(target_dir).unwrap();
    copy_dir_all(&output_dir, target_dir).unwrap();
}

// Helper to recursively copy files
fn copy_dir_all(src: &str, dst: &str) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = Path::new(dst).join(entry.file_name());

        if file_type.is_dir() {
            fs::create_dir_all(&to)?;
            copy_dir_all(from.to_str().unwrap(), to.to_str().unwrap())?;
        } else {
            fs::copy(from, to)?;
        }
    }
    Ok(())
}
