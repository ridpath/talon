use std::fs;

pub fn analyze_dotnet(path: &str) -> Result<(), String> {
    let content = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    if content.windows(4).any(|w| w == b"BSJB") {
        println!("[.NET] [OK] Managed metadata header found");
    } else {
        println!("[.NET] [ERROR] Not a valid .NET assembly");
    }
    Ok(())
}
