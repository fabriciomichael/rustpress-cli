use std::process::Command;

/// Checks whether the Rust compiler is installed.
pub fn check_rustc() -> Result<String, String> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|_| "Rust compiler (rustc) was not found.".to_string())?;

    if !output.status.success() {
        return Err("Failed to retrieve the Rust compiler version.".to_string());
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(version)
}

/// Checks whether Cargo is installed.
pub fn check_cargo() -> Result<String, String> {
    let output = Command::new("cargo")
        .arg("--version")
        .output()
        .map_err(|_| "Cargo package manager was not found.".to_string())?;

    if !output.status.success() {
        return Err("Failed to retrieve the Cargo version.".to_string());
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(version)
}

use std::path::Path;

/// Checks whether the current directory contains a Rust project.
pub fn check_project() -> Result<String, String> {
    let current_dir = std::env::current_dir()
        .map_err(|_| "Failed to determine the current directory.".to_string())?;

    check_project_at(&current_dir)
}

/// Checks whether the specified directory contains a Rust project.
pub fn check_project_at(project_path: &Path) -> Result<String, String> {
    let manifest_path = project_path.join("Cargo.toml");

    if !manifest_path.is_file() {
        return Err("No Cargo.toml file found in the specified directory.".to_string());
    }

    Ok("Rust project detected (Cargo.toml found).".to_string())
}

/// Checks whether the project uses the RustPress Web framework.
pub fn check_rustpress_web(project_path: &Path) -> Result<String, String> {
    let manifest_path = project_path.join("Cargo.toml");

    let manifest = std::fs::read_to_string(&manifest_path)
        .map_err(|_| "Failed to read the project's Cargo.toml file.".to_string())?;

    let has_dependency = manifest.lines().any(|line| {
        let line = line.trim();

        line.starts_with("rustpress-web")
            || line.starts_with("rustpress-web ")
            || line.starts_with("rustpress-web=")
            || line.starts_with("rustpress-web =")
    });

    if !has_dependency {
        return Err("RustPress Web dependency was not found.".to_string());
    }

    Ok("RustPress Web dependency detected.".to_string())
}

/// Checks whether the project contains the essential RustPress directories.
pub fn check_project_structure(project_path: &Path) -> Result<String, String> {
    let required_directories = ["src", "views", "public", "storage"];

    let missing_directories: Vec<&str> = required_directories
        .iter()
        .copied()
        .filter(|directory| !project_path.join(directory).is_dir())
        .collect();

    if !missing_directories.is_empty() {
        return Err(format!(
            "Missing project directories: {}",
            missing_directories.join(", ")
        ));
    }

    Ok("RustPress project structure is valid.".to_string())
}

/// Runs all RustPress environment and project diagnostics.
pub fn run_diagnostics() {
    println!("\nRustPress Doctor");
    println!("==============================");

    match check_rustc() {
        Ok(version) => println!("[OK] {}", version),
        Err(error) => println!("[ERROR] {}", error),
    }

    match check_cargo() {
        Ok(version) => println!("[OK] {}", version),
        Err(error) => println!("[ERROR] {}", error),
    }

    let current_dir = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            println!("[ERROR] Failed to determine current directory: {}", error);
            return;
        }
    };

    match check_project_at(&current_dir) {
        Ok(message) => {
            println!("[OK] {}", message);

            match check_rustpress_web(&current_dir) {
                Ok(message) => println!("[OK] {}", message),
                Err(error) => println!("[WARNING] {}", error),
            }

            match check_project_structure(&current_dir) {
                Ok(message) => println!("[OK] {}", message),
                Err(error) => println!("[WARNING] {}", error),
            }
        }
        Err(error) => println!("[WARNING] {}", error),
    }

    println!("==============================");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rustc_version() {
        let result = check_rustc();

        assert!(result.is_ok());
        assert!(result.unwrap().contains("rustc"));
    }

    #[test]
    fn detects_cargo_version() {
        let result = check_cargo();

        assert!(result.is_ok());
        assert!(result.unwrap().contains("cargo"));
    }

    #[test]
    fn detects_rust_project() {
        let current_dir = std::env::current_dir().unwrap();

        let result = check_project_at(&current_dir);

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Rust project detected"));
    }

    #[test]
    fn rejects_directory_without_cargo_toml() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let temp_dir = std::env::temp_dir().join(format!("rustpress-doctor-{}", timestamp));

        fs::create_dir_all(&temp_dir).unwrap();

        let result = check_project_at(&temp_dir);

        fs::remove_dir_all(&temp_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn detects_rustpress_web_dependency() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let temp_dir = std::env::temp_dir().join(format!("rustpress-doctor-web-{}", timestamp));

        fs::create_dir_all(&temp_dir).unwrap();

        fs::write(
            temp_dir.join("Cargo.toml"),
            "[dependencies]\nrustpress-web = \"0.2.0\"\n",
        )
        .unwrap();

        let result = check_rustpress_web(&temp_dir);

        fs::remove_dir_all(&temp_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_project_without_rustpress_web_dependency() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let temp_dir = std::env::temp_dir().join(format!("rustpress-doctor-no-web-{}", timestamp));

        fs::create_dir_all(&temp_dir).unwrap();

        fs::write(
            temp_dir.join("Cargo.toml"),
            "[dependencies]\nserde = \"1.0\"\n",
        )
        .unwrap();

        let result = check_rustpress_web(&temp_dir);

        fs::remove_dir_all(&temp_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn detects_valid_project_structure() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let temp_dir =
            std::env::temp_dir().join(format!("rustpress-doctor-structure-{}", timestamp));

        fs::create_dir_all(&temp_dir).unwrap();

        for directory in ["src", "views", "public", "storage"] {
            fs::create_dir_all(temp_dir.join(directory)).unwrap();
        }

        let result = check_project_structure(&temp_dir);

        fs::remove_dir_all(&temp_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_incomplete_project_structure() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let temp_dir =
            std::env::temp_dir().join(format!("rustpress-doctor-incomplete-{}", timestamp));

        fs::create_dir_all(temp_dir.join("src")).unwrap();

        let result = check_project_structure(&temp_dir);

        fs::remove_dir_all(&temp_dir).unwrap();

        assert!(result.is_err());
    }
}
