use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::validation;

/// Cria um middleware e regista-o no módulo principal de middlewares.
pub fn create_middleware(project_path: &Path, name: &str) -> io::Result<String> {
    let module_name = validation::normalize_module_name(name)?;

    let cargo_file = project_path.join("Cargo.toml");
    let middleware_dir = project_path.join("src/middleware");
    let modules_file = middleware_dir.join("mod.rs");

    if !cargo_file.is_file() || !middleware_dir.is_dir() || !modules_file.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    let middleware_file = middleware_dir.join(format!("{module_name}.rs"));

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
    let middleware_exists = middleware_file.is_file();

    if middleware_exists && already_registered {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Middleware already exists.",
        ));
    }

    let middleware_source = r#"use rustpress_web::{MiddlewareResult, Request};

/// Executa o middleware.
pub fn handle(request: &Request) -> MiddlewareResult {
    println!(
        "[RustPress] {} {}",
        request.method(),
        request.path()
    );

    MiddlewareResult::Next
}
"#;

    let mut created_middleware = false;

    if !middleware_exists {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&middleware_file)?;

        if let Err(error) = file.write_all(middleware_source.as_bytes()) {
            drop(file);
            let _ = fs::remove_file(&middleware_file);
            return Err(error);
        }

        created_middleware = true;
    }

    if !already_registered {
        if !modules_content.is_empty() && !modules_content.ends_with('\n') {
            modules_content.push('\n');
        }

        modules_content.push_str(&declaration);
        modules_content.push('\n');

        if let Err(error) = fs::write(&modules_file, modules_content) {
            if created_middleware {
                let _ = fs::remove_file(&middleware_file);
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

        let project_path =
            std::env::temp_dir().join(format!("rustpress-middleware-test-{timestamp}"));

        fs::create_dir_all(project_path.join("src/middleware")).unwrap();

        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test-project\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(
            project_path.join("src/middleware/mod.rs"),
            "// Middleware modules.\n",
        )
        .unwrap();

        project_path
    }

    #[test]
    fn creates_middleware_file_and_registers_module() {
        let project_path = temporary_project();

        let result = create_middleware(&project_path, "AuthMiddleware");

        assert_eq!(result.unwrap(), "auth_middleware");

        let middleware_file = project_path.join("src/middleware/auth_middleware.rs");
        let modules_file = project_path.join("src/middleware/mod.rs");

        assert!(middleware_file.is_file());

        let source = fs::read_to_string(middleware_file).unwrap();
        assert!(source.contains("pub fn handle(request: &Request) -> MiddlewareResult"));
        assert!(source.contains("MiddlewareResult::Next"));

        let modules = fs::read_to_string(modules_file).unwrap();
        assert!(modules.contains("pub mod auth_middleware;"));

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_duplicate_middleware_without_overwriting() {
        let project_path = temporary_project();

        create_middleware(&project_path, "AuthMiddleware").unwrap();

        let middleware_file = project_path.join("src/middleware/auth_middleware.rs");
        let original_content = fs::read_to_string(&middleware_file).unwrap();

        let result = create_middleware(&project_path, "AuthMiddleware");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);

        assert_eq!(
            fs::read_to_string(middleware_file).unwrap(),
            original_content
        );

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn registers_existing_middleware_file_without_overwriting() {
        let project_path = temporary_project();

        let middleware_file = project_path.join("src/middleware/auth_middleware.rs");
        let modules_file = project_path.join("src/middleware/mod.rs");

        let original_content = "// Existing middleware.\n";
        fs::write(&middleware_file, original_content).unwrap();

        let result = create_middleware(&project_path, "AuthMiddleware");

        assert_eq!(result.unwrap(), "auth_middleware");

        assert_eq!(
            fs::read_to_string(middleware_file).unwrap(),
            original_content
        );

        let modules = fs::read_to_string(modules_file).unwrap();
        assert!(modules.contains("pub mod auth_middleware;"));

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_directory_that_is_not_a_rustpress_project() {
        let project_path = std::env::temp_dir().join("rustpress-invalid-middleware-project");

        let result = create_middleware(&project_path, "AuthMiddleware");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn rejects_duplicate_module_declarations() {
        let project_path = temporary_project();

        let modules_file = project_path.join("src/middleware/mod.rs");
        let middleware_file = project_path.join("src/middleware/auth_middleware.rs");

        fs::write(
            &modules_file,
            "pub mod auth_middleware;\npub mod auth_middleware;\n",
        )
        .unwrap();

        let result = create_middleware(&project_path, "AuthMiddleware");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
        assert!(!middleware_file.exists());

        fs::remove_dir_all(project_path).unwrap();
    }
}
