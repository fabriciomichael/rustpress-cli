use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Finds the main executable produced by a Cargo project.
pub fn find_binary(project_dir: &Path, profile: &str) -> Result<PathBuf, String> {
    find_binary_internal(project_dir, profile, None)
}

/// Finds a specific executable by its Cargo binary target name.
pub fn find_binary_by_name(
    project_dir: &Path,
    profile: &str,
    binary_name: &str,
) -> Result<PathBuf, String> {
    find_binary_internal(project_dir, profile, Some(binary_name))
}

/// Resolves a Cargo binary target and returns its executable path.
fn find_binary_internal(
    project_dir: &Path,
    profile: &str,
    requested_name: Option<&str>,
) -> Result<PathBuf, String> {
    if !matches!(profile, "debug" | "release") {
        return Err(format!("Unsupported Cargo profile: {}", profile));
    }

    let project_dir = project_dir
        .canonicalize()
        .map_err(|error| format!("Failed to resolve project directory: {}", error))?;

    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .current_dir(&project_dir)
        .output()
        .map_err(|error| format!("Failed to run cargo metadata: {}", error))?;

    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let metadata: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Failed to parse cargo metadata: {}", error))?;

    let expected_manifest = project_dir
        .join("Cargo.toml")
        .canonicalize()
        .map_err(|error| format!("Failed to resolve Cargo.toml: {}", error))?;

    let package = metadata["packages"]
        .as_array()
        .and_then(|packages| {
            packages.iter().find(|package| {
                package["manifest_path"]
                    .as_str()
                    .and_then(|path| Path::new(path).canonicalize().ok())
                    .is_some_and(|path| path == expected_manifest)
            })
        })
        .ok_or_else(|| "Project package not found in Cargo metadata".to_string())?;

    let targets = package["targets"]
        .as_array()
        .ok_or_else(|| "Cargo package has no targets".to_string())?;

    let binary_targets: Vec<&Value> = targets
        .iter()
        .filter(|target| {
            target["kind"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"))
        })
        .collect();

    let binary_target = if let Some(name) = requested_name {
        binary_targets
            .iter()
            .find(|target| target["name"].as_str() == Some(name))
            .copied()
            .ok_or_else(|| {
                let available = binary_targets
                    .iter()
                    .filter_map(|target| target["name"].as_str())
                    .collect::<Vec<_>>()
                    .join(", ");

                format!(
                    "Binary target '{}' not found. Available binaries: {}",
                    name,
                    if available.is_empty() {
                        "none".to_string()
                    } else {
                        available
                    }
                )
            })?
    } else {
        match binary_targets.len() {
            0 => return Err("No binary target found in Cargo project".to_string()),
            1 => binary_targets[0],
            _ => {
                let default_run = package["default_run"].as_str();

                if let Some(default_name) = default_run {
                    binary_targets
                        .iter()
                        .find(|target| target["name"].as_str() == Some(default_name))
                        .copied()
                        .ok_or_else(|| {
                            format!(
                                "The default-run '{}' does not match any binary target.",
                                default_name
                            )
                        })?
                } else {
                    return Err(format!(
                        "Multiple binary targets found: {}. Specify one using --bin <NAME> or define default-run in Cargo.toml.",
                        binary_targets
                            .iter()
                            .filter_map(|target| target["name"].as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
        }
    };

    let binary_name = binary_target["name"]
        .as_str()
        .ok_or_else(|| "Binary target has no name".to_string())?;

    let target_directory = metadata["target_directory"]
        .as_str()
        .ok_or_else(|| "Cargo metadata has no target directory".to_string())?;

    let executable_name = if cfg!(windows) {
        format!("{}.exe", binary_name)
    } else {
        binary_name.to_string()
    };

    Ok(PathBuf::from(target_directory)
        .join(profile)
        .join(executable_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempProject {
        path: PathBuf,
    }

    impl TempProject {
        fn new(default_run: Option<&str>) -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

            let path = env::temp_dir().join(format!(
                "rustpress-cli-metadata-test-{}-{}",
                std::process::id(),
                id
            ));

            fs::create_dir_all(path.join("src/bin")).unwrap();

            let default_run = default_run
                .map(|name| format!("default-run = \"{}\"\n", name))
                .unwrap_or_default();

            let manifest = format!(
                "[package]\n\
                 name = \"metadata-test\"\n\
                 version = \"0.1.0\"\n\
                 edition = \"2024\"\n\
                 {}",
                default_run
            );

            fs::write(path.join("Cargo.toml"), manifest).unwrap();

            fs::write(path.join("src/main.rs"), "fn main() {}").unwrap();

            fs::write(path.join("src/bin/admin.rs"), "fn main() {}").unwrap();

            Self { path }
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn finds_binary_by_name() {
        let project_dir = env::current_dir().unwrap();

        let binary_path = find_binary_by_name(&project_dir, "debug", "rustpress").unwrap();

        assert_eq!(
            binary_path.file_stem().unwrap().to_string_lossy(),
            "rustpress"
        );
    }

    #[test]
    fn rejects_unknown_binary_name() {
        let project_dir = env::current_dir().unwrap();

        let result = find_binary_by_name(&project_dir, "debug", "unknown-binary");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn rejects_unsupported_profile() {
        let project_dir = env::current_dir().unwrap();

        let result = find_binary(&project_dir, "production");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported Cargo profile"));
    }

    #[test]
    fn selects_default_run_when_multiple_binaries_exist() {
        let project = TempProject::new(Some("admin"));

        let binary_path = find_binary(&project.path, "debug").unwrap();

        assert_eq!(binary_path.file_stem().unwrap().to_string_lossy(), "admin");
    }

    #[test]
    fn rejects_multiple_binaries_without_default_run() {
        let project = TempProject::new(None);

        let result = find_binary(&project.path, "debug");

        assert!(result.is_err());

        let error = result.unwrap_err();

        assert!(error.contains("Multiple binary targets found"));
        assert!(error.contains("admin"));
        assert!(error.contains("metadata-test"));
    }

    #[test]
    fn explicit_binary_overrides_default_run() {
        let project = TempProject::new(Some("admin"));

        let binary_path = find_binary_by_name(&project.path, "debug", "metadata-test").unwrap();

        assert_eq!(
            binary_path.file_stem().unwrap().to_string_lossy(),
            "metadata-test"
        );
    }

    #[test]
    fn rejects_invalid_default_run() {
        let project = TempProject::new(Some("unknown"));

        let result = find_binary(&project.path, "debug");

        assert!(result.is_err());
    }
}
