use colored::*;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BuildTarget {
    pub name: String,
    pub triple: String,
    pub description: String,
}

impl BuildTarget {
    pub fn new(name: &str, triple: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            triple: triple.to_string(),
            description: description.to_string(),
        }
    }
}

pub struct MatrixBuilder {
    targets: Vec<BuildTarget>,
    output_dir: String,
}

impl MatrixBuilder {
    pub fn new() -> Self {
        Self {
            targets: Self::default_targets(),
            output_dir: "matrix_builds".to_string(),
        }
    }

    fn default_targets() -> Vec<BuildTarget> {
        vec![
            BuildTarget::new(
                "linux-x64",
                "x86_64-unknown-linux-musl",
                "Static Linux x64 (Primary deployment target)",
            ),
            BuildTarget::new(
                "linux-x86",
                "i686-unknown-linux-musl",
                "Static Linux x86 32-bit (Required for Narnia challenges)",
            ),
            BuildTarget::new(
                "linux-arm64",
                "aarch64-unknown-linux-musl",
                "Static ARM64 (IoT/Mobile research)",
            ),
            BuildTarget::new(
                "windows-x64",
                "x86_64-pc-windows-gnu",
                "Static Windows x64 (Cross-platform deployment)",
            ),
        ]
    }

    pub fn set_output_dir(&mut self, dir: &str) {
        self.output_dir = dir.to_string();
    }

    pub fn ensure_targets_installed(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "[*] Verifying rustup targets...".cyan());

        for target in &self.targets {
            println!("  [*] Checking target: {}", target.triple.yellow());

            let output = Command::new("rustup")
                .args(&["target", "list", "--installed"])
                .output()?;

            let installed = String::from_utf8_lossy(&output.stdout);

            if !installed.contains(&target.triple) {
                println!(
                    "  [-] Target {} not installed, installing...",
                    target.triple.red()
                );

                let status = Command::new("rustup")
                    .args(&["target", "add", &target.triple])
                    .status()?;

                if !status.success() {
                    return Err(format!("Failed to install target: {}", target.triple).into());
                }

                println!("  [+] Installed: {}", target.triple.green());
            } else {
                println!("  [+] Already installed: {}", target.triple.green());
            }
        }

        Ok(())
    }

    pub fn build_matrix(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n{}", "=== TALON Matrix Build ===".bold().cyan());
        println!(
            "{}\n",
            "Building static binaries for all supported architectures...".cyan()
        );

        self.ensure_targets_installed()?;

        fs::create_dir_all(&self.output_dir)?;

        let mut build_results = Vec::new();

        for target in &self.targets {
            println!("\n{}", format!("--- Building: {} ---", target.name).bold().yellow());
            println!("  Target: {}", target.triple.cyan());
            println!("  Description: {}", target.description);

            let start_time = std::time::Instant::now();

            let status = Command::new("cargo")
                .args(&[
                    "build",
                    "--release",
                    "--target",
                    &target.triple,
                ])
                .env("RUSTFLAGS", "-C target-feature=+crt-static")
                .status();

            let duration = start_time.elapsed();

            match status {
                Ok(s) if s.success() => {
                    println!("  [+] Build succeeded in {:.2}s", duration.as_secs_f64());

                    let binary_name = if target.triple.contains("windows") {
                        "talon.exe"
                    } else {
                        "talon"
                    };

                    let source_path = format!("target/{}/release/{}", target.triple, binary_name);
                    let dest_path = format!("{}/talon-{}", self.output_dir, target.name);
                    let dest_path = if target.triple.contains("windows") {
                        format!("{}.exe", dest_path)
                    } else {
                        dest_path
                    };

                    if Path::new(&source_path).exists() {
                        fs::copy(&source_path, &dest_path)?;
                        let metadata = fs::metadata(&dest_path)?;
                        let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;

                        println!(
                            "  [+] Copied to: {} ({:.2} MB)",
                            dest_path.green(),
                            size_mb
                        );

                        build_results.push((target.name.clone(), true, size_mb, duration));
                    } else {
                        println!(
                            "  [-] Warning: Binary not found at {}",
                            source_path.yellow()
                        );
                        build_results.push((target.name.clone(), false, 0.0, duration));
                    }
                }
                Ok(_) => {
                    println!("  [-] Build failed for {}", target.name.red());
                    build_results.push((target.name.clone(), false, 0.0, duration));
                }
                Err(e) => {
                    println!("  [!] Error building {}: {}", target.name.red(), e);
                    build_results.push((target.name.clone(), false, 0.0, duration));
                }
            }
        }

        self.print_build_summary(&build_results);

        Ok(())
    }

    fn print_build_summary(&self, results: &[(String, bool, f64, std::time::Duration)]) {
        println!("\n{}", "=== Build Summary ===".bold().cyan());
        println!();

        let mut successful = 0;
        let mut failed = 0;

        for (name, success, size, duration) in results {
            if *success {
                println!(
                    "  [+] {} - {:.2} MB ({:.2}s)",
                    name.green(),
                    size,
                    duration.as_secs_f64()
                );
                successful += 1;
            } else {
                println!("  [-] {} - FAILED", name.red());
                failed += 1;
            }
        }

        println!();
        println!("  Total: {} successful, {} failed", successful, failed);
        println!("  Output directory: {}", self.output_dir.cyan());
        println!();

        if failed == 0 {
            println!(
                "{}",
                "[+] Matrix build complete. All targets built successfully.".green().bold()
            );
        } else {
            println!(
                "{}",
                format!("[!] Matrix build completed with {} failures.", failed)
                    .yellow()
                    .bold()
            );
        }
    }

    pub fn verify_static_linking(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n{}", "=== Verifying Static Linking ===".bold().cyan());

        for target in &self.targets {
            let binary_name = if target.triple.contains("windows") {
                format!("{}/talon-{}.exe", self.output_dir, target.name)
            } else {
                format!("{}/talon-{}", self.output_dir, target.name)
            };

            if !Path::new(&binary_name).exists() {
                println!("  [-] Skipping {} (not built)", target.name.yellow());
                continue;
            }

            println!("\n  [*] Verifying: {}", target.name.cyan());

            if target.triple.contains("linux") {
                let output = Command::new("ldd").arg(&binary_name).output();

                match output {
                    Ok(out) => {
                        let result = String::from_utf8_lossy(&out.stdout);
                        if result.contains("statically linked")
                            || result.contains("not a dynamic executable")
                        {
                            println!("{}", "    [+] Statically linked: YES".green());
                        } else {
                            println!("{}", "    [-] Statically linked: NO".red());
                            println!("    Output: {}", result);
                        }
                    }
                    Err(_) => {
                        println!("{}", "    [!] ldd command not available (running on Windows?)".yellow());
                    }
                }

                let output = Command::new("file").arg(&binary_name).output();

                match output {
                    Ok(out) => {
                        let result = String::from_utf8_lossy(&out.stdout);
                        println!("    File info: {}", result.trim());
                    }
                    Err(_) => {
                        println!("{}", "    [!] file command not available".yellow());
                    }
                }
            } else if target.triple.contains("windows") {
                println!("    [*] Windows binary verification requires manual testing");
            }
        }

        Ok(())
    }
}
