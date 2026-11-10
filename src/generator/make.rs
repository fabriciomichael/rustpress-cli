use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::validation;

pub fn create_controller(project_path: &Path, name: &str) -> io::Result<String> {
    let module_name = validation::normalize_module_name(name)?;

    let cargo_file = project_path.join("Cargo.toml");
    let controllers_dir = project_path.join("src/controllers");
    let modules_file = controllers_dir.join("mod.rs");

    if !cargo_file.is_file() || !controllers_dir.is_dir() || !modules_file.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    let controller_file = controllers_dir.join(format!("{module_name}.rs"));

    let declaration = format!("pub mod {module_name};");
    let private_declaration = format!("mod {module_name};");

    let mut modules_content = fs::read_to_string(&modules_file)?;

    let registration_count = modules_content
        .lines()
        .filter(|line| {
            let line = line.trim();

            line == declaration || line == private_declaration
        })
        .count();

    if registration_count > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Duplicate module declarations for '{module_name}'."),
        ));
    }

    let already_registered = registration_count == 1;

    let controller_exists = controller_file.is_file();

    // Não permite duplicar um controlador já registado.
    if controller_exists && already_registered {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Controller already exists.",
        ));
    }

    let controller_source = r#"use rustpress_web::Response;

pub fn index() -> Response {
    Response::new()
}
"#;

    let mut created_controller = false;

    // Cria o controlador apenas quando o ficheiro ainda não existe.
    if !controller_exists {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&controller_file)?;

        if let Err(error) = file.write_all(controller_source.as_bytes()) {
            drop(file);
            let _ = fs::remove_file(&controller_file);
            return Err(error);
        }

        created_controller = true;
    }

    // Regista o módulo apenas quando ainda não está declarado.
    if !already_registered {
        if !modules_content.is_empty() && !modules_content.ends_with('\n') {
            modules_content.push('\n');
        }

        modules_content.push_str(&declaration);
        modules_content.push('\n');

        if let Err(error) = fs::write(&modules_file, modules_content) {
            if created_controller {
                let _ = fs::remove_file(&controller_file);
            }

            return Err(error);
        }
    }

    Ok(module_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_project() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let project_path = std::env::temp_dir().join(format!("rustpress-make-test-{timestamp}"));

        fs::create_dir_all(project_path.join("src/controllers")).unwrap();

        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test-project\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(
            project_path.join("src/controllers/mod.rs"),
            "pub mod home_controller;\n",
        )
        .unwrap();

        project_path
    }

    #[test]
    fn creates_controller_file_and_registers_module() {
        let project_path = temporary_project();

        let result = create_controller(&project_path, "UserController");

        assert_eq!(result.unwrap(), "user_controller");

        let controller_file = project_path.join("src/controllers/user_controller.rs");
        let modules_file = project_path.join("src/controllers/mod.rs");

        assert!(controller_file.is_file());

        let source = fs::read_to_string(controller_file).unwrap();
        assert!(source.contains("pub fn index() -> Response"));

        let modules = fs::read_to_string(modules_file).unwrap();
        assert!(modules.contains("pub mod home_controller;"));
        assert!(modules.contains("pub mod user_controller;"));

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_duplicate_controller_without_overwriting() {
        let project_path = temporary_project();

        create_controller(&project_path, "UserController").unwrap();

        let controller_file = project_path.join("src/controllers/user_controller.rs");
        let original_content = fs::read_to_string(&controller_file).unwrap();

        let result = create_controller(&project_path, "UserController");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);

        assert_eq!(
            fs::read_to_string(controller_file).unwrap(),
            original_content
        );

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn registers_existing_controller_file_without_overwriting() {
        let project_path = temporary_project();

        let controller_file = project_path.join("src/controllers/user_controller.rs");
        let modules_file = project_path.join("src/controllers/mod.rs");

        let original_content = "// Existing controller\n";
        fs::write(&controller_file, original_content).unwrap();

        let result = create_controller(&project_path, "UserController");

        assert_eq!(result.unwrap(), "user_controller");

        assert_eq!(
            fs::read_to_string(&controller_file).unwrap(),
            original_content
        );

        let modules = fs::read_to_string(modules_file).unwrap();

        assert!(modules.contains("pub mod user_controller;"));

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_directory_that_is_not_a_rustpress_project() {
        let project_path = std::env::temp_dir().join("rustpress-invalid-project");

        let result = create_controller(&project_path, "UserController");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn rejects_duplicate_module_declarations() {
        let project_path = temporary_project();

        let modules_file = project_path.join("src/controllers/mod.rs");
        let controller_file = project_path.join("src/controllers/user_controller.rs");

        fs::write(
            &modules_file,
            "pub mod home_controller;\npub mod user_controller;\npub mod user_controller;\n",
        )
        .unwrap();

        let result = create_controller(&project_path, "UserController");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);

        assert!(!controller_file.exists());

        fs::remove_dir_all(project_path).unwrap();
    }
}
