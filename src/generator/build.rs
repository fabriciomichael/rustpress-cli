use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Compiles the Rust project in release mode.
fn compile_project(project_path: &Path) -> io::Result<PathBuf> {
    println!("\nCompiling project in release mode...");

    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(project_path)
        .status()?;

    if !status.success() {
        return Err(io::Error::other("Project compilation failed."));
    }

    let cargo_toml = fs::read_to_string(project_path.join("Cargo.toml"))?;

    let package_name = cargo_toml
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;

            if key.trim() == "name" {
                Some(value.trim().trim_matches('"').to_owned())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Package name not found in Cargo.toml.",
            )
        })?;

    let executable_name = format!("{}.exe", package_name);
    let executable_path = project_path
        .join("target")
        .join("release")
        .join(&executable_name);

    if !executable_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Executable not found: {}", executable_path.display()),
        ));
    }

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

/// Prepares the final distribution of a RustPress project.
pub fn build_project(project_path: &Path, include: &[PathBuf]) -> io::Result<()> {
    println!("Preparing RustPress distribution...");
    println!("Project: {}", project_path.display());

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

    let executable_path = compile_project(project_path)?;

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
