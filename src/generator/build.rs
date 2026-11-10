use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

fn compile_project(project_path: &Path, binary_name: Option<&str>) -> io::Result<PathBuf> {
    // Resolve the binary target before compiling.
    let executable_path = match binary_name {
        Some(name) => crate::cargo_metadata::find_binary_by_name(project_path, "release", name),
        None => crate::cargo_metadata::find_binary(project_path, "release"),
    }
    .map_err(io::Error::other)?;

    println!("\nCompiling project in release mode...");

    let mut command = Command::new("cargo");
    command.arg("build").arg("--release");

    if let Some(name) = binary_name {
        command.arg("--bin").arg(name);
    }

    let status = command.current_dir(project_path).status()?;

    if !status.success() {
        return Err(io::Error::other("Project compilation failed."));
    }

    if !executable_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Executable not found: {}", executable_path.display()),
        ));
    }

    let executable_name = executable_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");

    println!("Compilation successful: {}", executable_name);

    Ok(executable_path)
}

/// Recursively copies a directory.
fn copy_directory(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        let metadata = fs::symlink_metadata(&source_path)?;

        if metadata.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Symbolic links are not supported: {}",
                    source_path.display()
                ),
            ));
        }

        if metadata.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else if metadata.is_file() {
            fs::copy(&source_path, &destination_path)?;
        }
    }

    Ok(())
}

/// Returns the default directories included in a RustPress distribution.
fn default_include_directories(project_path: &Path) -> Vec<PathBuf> {
    ["views", "public", "storage"]
        .iter()
        .map(PathBuf::from)
        .filter(|directory| project_path.join(directory).is_dir())
        .collect()
}

/// Prepares the final distribution of a RustPress project.
pub fn build_project(project_path: &Path, include: &[PathBuf]) -> io::Result<()> {
    build_project_with_binary(project_path, include, None)
}

/// Prepares the distribution using a specific Cargo binary target.
pub fn build_project_with_binary(
    project_path: &Path,
    include: &[PathBuf],
    binary_name: Option<&str>,
) -> io::Result<()> {
    println!("Preparing RustPress distribution...");
    println!("Project: {}", project_path.display());

    let default_directories;

    let include = if include.is_empty() {
        default_directories = default_include_directories(project_path);
        default_directories.as_slice()
    } else {
        include
    };

    if include.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No distribution directories found. Create views/, public/, or storage/.",
        ));
    }

    for directory in include {
        if directory.is_absolute()
            || directory
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid directory path: {}", directory.display()),
            ));
        }

        let source = project_path.join(directory);

        if !source.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Directory not found: {}", directory.display()),
            ));
        }

        if !source.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Path is not a directory: {}", directory.display()),
            ));
        }

        println!("Validated: {}", directory.display());
    }

    let executable_path = compile_project(project_path, binary_name)?;

    let distribution_path = project_path.join("dist");

    fs::create_dir_all(&distribution_path)?;

    let package_name = project_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid project directory name.",
            )
        })?;

    let final_path = distribution_path.join(package_name);

    fs::create_dir_all(&final_path)?;

    for directory in include {
        let source = project_path.join(directory);
        let destination = final_path.join(directory);

        copy_directory(&source, &destination)?;

        println!("Copied: {}", directory.display());
    }

    let executable_name = executable_path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Invalid executable filename.")
    })?;

    fs::copy(&executable_path, final_path.join(executable_name))?;

    println!("Executable copied successfully.");
    println!("Final distribution: {}", final_path.display());

    println!(
        "Distribution directories created at: {}",
        distribution_path.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory() -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("rustpress-build-test-{}", timestamp))
    }

    #[test]
    fn rejects_missing_include_directory() {
        let project_path = temporary_directory();

        fs::create_dir_all(&project_path).unwrap();

        let include = vec![PathBuf::from("views")];

        let result = build_project(&project_path, &include);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_file_instead_of_directory() {
        let project_path = temporary_directory();

        fs::create_dir_all(&project_path).unwrap();

        fs::write(project_path.join("README.md"), "Test").unwrap();

        let include = vec![PathBuf::from("README.md")];

        let result = build_project(&project_path, &include);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_parent_directory_traversal() {
        let project_path = temporary_directory();

        fs::create_dir_all(&project_path).unwrap();

        let include = vec![PathBuf::from("../outside")];

        let result = build_project(&project_path, &include);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn copies_directory_recursively() {
        let base_path = temporary_directory();

        let source = base_path.join("source");
        let destination = base_path.join("destination");

        fs::create_dir_all(source.join("css")).unwrap();

        fs::write(source.join("index.html"), "<h1>RustPress</h1>").unwrap();
        fs::write(source.join("css/style.css"), "body {}").unwrap();

        let result = copy_directory(&source, &destination);

        assert!(result.is_ok());

        assert!(destination.join("index.html").is_file());
        assert!(destination.join("css/style.css").is_file());

        assert_eq!(
            fs::read_to_string(destination.join("index.html")).unwrap(),
            "<h1>RustPress</h1>"
        );

        assert_eq!(
            fs::read_to_string(destination.join("css/style.css")).unwrap(),
            "body {}"
        );

        fs::remove_dir_all(base_path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symbolic_links() {
        use std::os::unix::fs::symlink;

        let base_path = temporary_directory();
        let source = base_path.join("source");
        let destination = base_path.join("destination");

        fs::create_dir_all(&source).unwrap();

        let target = base_path.join("target.txt");
        fs::write(&target, "Sensitive content").unwrap();

        symlink(&target, source.join("link.txt")).unwrap();

        let result = copy_directory(&source, &destination);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);

        fs::remove_dir_all(base_path).unwrap();
    }

    #[test]
    fn detects_existing_default_directories() {
        let project_path = temporary_directory();

        fs::create_dir_all(project_path.join("views")).unwrap();
        fs::create_dir_all(project_path.join("public")).unwrap();
        fs::create_dir_all(project_path.join("src")).unwrap();

        let directories = default_include_directories(&project_path);

        assert_eq!(
            directories,
            vec![PathBuf::from("views"), PathBuf::from("public")]
        );

        fs::remove_dir_all(project_path).unwrap();
    }
}
