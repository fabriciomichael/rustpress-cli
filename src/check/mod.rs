use std::fs;
use std::path::Path;
use std::process::Command;

/// Verifies that the project contains a readable Cargo.toml file.
pub fn check_manifest(project_path: &Path) -> Result<(), String> {
    let manifest_path = project_path.join("Cargo.toml");

    if !manifest_path.is_file() {
        return Err("Cargo.toml not found.".to_string());
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("Failed to read Cargo.toml: {}", error))?;

    let manifest: toml::Value =
        toml::from_str(&content).map_err(|error| format!("Invalid Cargo.toml: {}", error))?;

    if manifest
        .get("package")
        .and_then(toml::Value::as_table)
        .is_none()
    {
        return Err("Cargo.toml is missing the [package] section.".to_string());
    }

    Ok(())
}

/// Verifies that all essential RustPress project directories exist.
pub fn check_directories(project_path: &Path) -> Result<(), String> {
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

    Ok(())
}

/// Verifies that all essential RustPress project files exist.
pub fn check_files(project_path: &Path) -> Result<(), String> {
    let required_files = ["src/main.rs", "src/routes"];

    let missing_files: Vec<&str> = required_files
        .iter()
        .copied()
        .filter(|file| !project_path.join(file).exists())
        .collect();

    if !missing_files.is_empty() {
        return Err(format!(
            "Missing project files: {}",
            missing_files.join(", ")
        ));
    }

    Ok(())
}

/// Verifies that the project declares the RustPress Web dependency.
pub fn check_rustpress_dependency(project_path: &Path) -> Result<(), String> {
    let manifest_path = project_path.join("Cargo.toml");

    let content = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("Failed to read Cargo.toml: {}", error))?;

    let manifest: toml::Value =
        toml::from_str(&content).map_err(|error| format!("Invalid Cargo.toml: {}", error))?;

    let dependencies = manifest.get("dependencies").and_then(toml::Value::as_table);

    let has_dependency = dependencies.is_some_and(|deps| deps.contains_key("rustpress-web"));

    if !has_dependency {
        return Err("RustPress Web dependency was not found.".to_string());
    }

    Ok(())
}

/// Runs all project integrity checks.
pub fn run_checks() -> bool {
    let project_path = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Failed to get current directory: {}", error);
            return false;
        }
    };

    println!("\nRustPress Project Check");
    println!("=======================\n");

    type CheckFn = fn(&Path) -> Result<(), String>;

    let checks: [(&str, CheckFn); 4] = [
        ("Cargo.toml", check_manifest),
        ("Project directories", check_directories),
        ("Project files", check_files),
        ("RustPress Web dependency", check_rustpress_dependency),
    ];

    let mut has_errors = false;

    for (name, check) in checks {
        match check(&project_path) {
            Ok(()) => println!("[OK]   {}", name),
            Err(error) => {
                println!("[FAIL] {}: {}", name, error);
                has_errors = true;
            }
        }
    }

    if has_errors {
        println!("\nProject check failed. Fix the reported issues.");
        return false;
    }

    println!("\nRunning cargo check...\n");

    match Command::new("cargo")
        .arg("check")
        .current_dir(&project_path)
        .status()
    {
        Ok(status) if status.success() => {
            println!("\n[OK]   Cargo check");
            println!("\nAll checks passed!");
            true
        }
        Ok(_) => {
            println!("\n[FAIL] Cargo check");
            println!("\nProject check failed.");
            false
        }
        Err(error) => {
            println!("\n[FAIL] Could not execute cargo check: {}", error);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_temp_dir() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let path = std::env::temp_dir().join(format!("rustpress-check-{}", timestamp));

        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn detects_valid_manifest() {
        let path = create_temp_dir();
        fs::write(path.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

        assert!(check_manifest(&path).is_ok());

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rejects_missing_manifest() {
        let path = create_temp_dir();

        assert!(check_manifest(&path).is_err());

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rejects_manifest_without_package_section() {
        let path = create_temp_dir();
        fs::write(path.join("Cargo.toml"), "[dependencies]\nserde = \"1\"\n").unwrap();

        assert!(check_manifest(&path).is_err());

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn detects_valid_project_directories() {
        let path = create_temp_dir();

        for directory in ["src", "views", "public", "storage"] {
            fs::create_dir_all(path.join(directory)).unwrap();
        }

        assert!(check_directories(&path).is_ok());

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rejects_missing_project_directories() {
        let path = create_temp_dir();

        let result = check_directories(&path);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("src"));

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn reports_all_missing_directories() {
        let path = create_temp_dir();

        fs::create_dir_all(path.join("src")).unwrap();

        let error = check_directories(&path).unwrap_err();

        assert!(error.contains("views"));
        assert!(error.contains("public"));
        assert!(error.contains("storage"));

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn detects_valid_project_files() {
        let path = create_temp_dir();

        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(path.join("src/main.rs"), "fn main() {}").unwrap();
        fs::create_dir_all(path.join("src/routes")).unwrap();

        assert!(check_files(&path).is_ok());

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rejects_missing_project_files() {
        let path = create_temp_dir();

        let result = check_files(&path);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("src/main.rs"));

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn detects_rustpress_dependency() {
        let path = create_temp_dir();

        fs::write(
        path.join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nrustpress-web = \"0.2\"\n",
    )
    .unwrap();

        let result = check_rustpress_dependency(&path);

        assert!(result.is_ok());

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rejects_missing_rustpress_dependency() {
        let path = create_temp_dir();

        fs::write(
        path.join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nserde = \"1\"\n",
    )
    .unwrap();

        let result = check_rustpress_dependency(&path);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("RustPress Web"));

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rejects_invalid_manifest_for_dependency_check() {
        let path = create_temp_dir();

        fs::write(path.join("Cargo.toml"), "[dependencies\n").unwrap();

        assert!(check_rustpress_dependency(&path).is_err());

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rejects_invalid_manifest_syntax() {
        let path = create_temp_dir();

        fs::write(path.join("Cargo.toml"), "[package\nname = \"test\"\n").unwrap();

        let result = check_manifest(&path);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid Cargo.toml"));

        fs::remove_dir_all(path).unwrap();
    }
}
