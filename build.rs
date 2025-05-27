use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let frontend_dir = "frontend";
    let output_dir = format!("{frontend_dir}/build");
    let target_dir = "static";

    let node_path = r"C:\nvm4w\nodejs"; // or wherever your npm/node is
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{node_path};{current_path}");

    // Set new PATH for the npm build to find node
    unsafe {
    env::set_var("PATH", &new_path);
     } // <— Pass by reference

    let npm = if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    };

    // Run `npm install`
    let status_install = Command::new(npm)
        .arg("install")
        .current_dir(&frontend_dir)
        .status()
        .expect("Failed to run `npm install`");

    if !status_install.success() {
        panic!("npm install failed");
    }

    // Run `npm run build`
    let status_build = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "npm run build"])
            .current_dir(&frontend_dir)
            .status()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("npm run build")
            .current_dir(&frontend_dir)
            .status()
    }
    .expect("Failed to run `npm run build`");

    if !status_build.success() {
        panic!("npm run build failed");
    }

    // Copy frontend/build to static/
    let _ = fs::remove_dir_all(&target_dir);
    fs::create_dir_all(&target_dir).unwrap();
    copy_dir_all(&output_dir, &target_dir).unwrap();
}

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
