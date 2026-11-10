use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use super::{make, middleware, model, validation, view};

struct ModuleSnapshot {
    path: PathBuf,
    content: String,
}

fn validate_module_target(directory: &Path, module_name: &str) -> io::Result<ModuleSnapshot> {
    let modules_file = directory.join("mod.rs");
    let target_file = directory.join(format!("{module_name}.rs"));

    if !directory.is_dir() || !modules_file.is_file() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    if target_file.exists() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("Component '{module_name}' already exists."),
        ));
    }

    let content = fs::read_to_string(&modules_file)?;
    let public_declaration = format!("pub mod {module_name};");
    let private_declaration = format!("mod {module_name};");

    let count = content
        .lines()
        .filter(|line| {
            let line = line.trim();
            line == public_declaration || line == private_declaration
        })
        .count();

    if count > 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Duplicate module declarations for '{module_name}'."),
        ));
    }

    if count == 1 {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("Module '{module_name}' is already registered."),
        ));
    }

    Ok(ModuleSnapshot {
        path: modules_file,
        content,
    })
}

fn rollback(
    created_files: &[PathBuf],
    snapshots: &[ModuleSnapshot],
    pages_dir: &Path,
    pages_directory_existed: bool,
) {
    // Remove only files successfully created by this operation.
    for file in created_files.iter().rev() {
        let _ = fs::remove_file(file);
    }

    // Restore the original module declarations.
    for snapshot in snapshots {
        let _ = fs::write(&snapshot.path, &snapshot.content);
    }

    // Remove the pages directory only if it did not exist before.
    if !pages_directory_existed {
        let _ = fs::remove_dir(pages_dir);
    }
}

pub fn create_all(project_path: &Path, name: &str) -> io::Result<String> {
    let module_name = validation::normalize_module_name(name)?;

    let controller_name = format!("{name}Controller");
    let middleware_name = format!("{name}Middleware");

    let controller_module = validation::normalize_module_name(&controller_name)?;
    let middleware_module = validation::normalize_module_name(&middleware_name)?;

    let cargo_file = project_path.join("Cargo.toml");
    let controllers_dir = project_path.join("src/controllers");
    let middleware_dir = project_path.join("src/middleware");
    let models_dir = project_path.join("src/models");
    let views_dir = project_path.join("views");
    let pages_dir = views_dir.join("pages");

    if !cargo_file.is_file() || !views_dir.is_dir() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    if pages_dir.exists() && !pages_dir.is_dir() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "The views/pages path exists but is not a directory.",
        ));
    }

    let snapshots = vec![
        validate_module_target(&controllers_dir, &controller_module)?,
        validate_module_target(&middleware_dir, &middleware_module)?,
        validate_module_target(&models_dir, &module_name)?,
    ];

    let view_file = pages_dir.join(format!("{module_name}.html"));

    if view_file.exists() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("View '{module_name}' already exists."),
        ));
    }

    let pages_directory_existed = pages_dir.is_dir();

    let controller_file = controllers_dir.join(format!("{controller_module}.rs"));
    let middleware_file = middleware_dir.join(format!("{middleware_module}.rs"));
    let model_file = models_dir.join(format!("{module_name}.rs"));

    let mut created_files = Vec::new();

    let result: io::Result<()> = (|| {
        make::create_controller(project_path, &controller_name)?;
        created_files.push(controller_file);

        middleware::create_middleware(project_path, &middleware_name)?;
        created_files.push(middleware_file);

        model::create_model(project_path, name)?;
        created_files.push(model_file);

        view::create_view(project_path, name)?;
        created_files.push(view_file);

        Ok(())
    })();

    if let Err(error) = result {
        rollback(
            &created_files,
            &snapshots,
            &pages_dir,
            pages_directory_existed,
        );

        return Err(error);
    }

    Ok(module_name)
}
