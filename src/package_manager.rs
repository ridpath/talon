use std::fs;
use std::io;
use std::path::Path;

/// Install a package from the `talon_std/` directory into current working directory.
///
/// Supports both `.my` files and whole directories (e.g. `exploit/`).
pub fn install_package(name: &str) {
    let src_path = Path::new("talon_std").join(name);
    let dest_path = Path::new(".").join(name);

    if !src_path.exists() {
        eprintln!(
            "[PACKAGE] [ERROR] Package not found: {}",
            src_path.display()
        );
        return;
    }

    if src_path.is_dir() {
        if let Err(e) = copy_dir_recursive(&src_path, &dest_path) {
            eprintln!("[PACKAGE] [ERROR] Failed to install directory: {}", e);
            return;
        }
        println!("[PACKAGE] Installed directory: {}", name);
    } else {
        if let Err(e) = fs::copy(&src_path, &dest_path) {
            eprintln!("[PACKAGE] [ERROR] Failed to copy file: {}", e);
            return;
        }
        println!("[PACKAGE] Installed file: {}", name);
    }
}

/// Recursively copy a directory and its contents.
fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest)?;
        } else {
            fs::copy(&path, &dest)?;
        }
    }

    Ok(())
}
