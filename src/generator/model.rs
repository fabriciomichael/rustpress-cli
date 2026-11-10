use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::validation;

pub fn create_model(project_path: &Path, name: &str) -> io::Result<String> {
    let module_name = validation::normalize_module_name(name)?;

    let cargo_file = project_path.join("Cargo.toml");
    let models_dir = project_path.join("src/models");
    let modules_file = models_dir.join("mod.rs");

    if !cargo_file.is_file() || !models_dir.is_dir() || !modules_file.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    let model_file = models_dir.join(format!("{module_name}.rs"));

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
    let model_exists = model_file.is_file();

    if model_exists && already_registered {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Model already exists.",
        ));
    }

    // Converte snake_case para PascalCase.
    let model_name = module_name
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();

            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>();

    let model_source = format!(
        r#"/// Represents the {model_name} model.
#[derive(Debug, Clone, Default)]
pub struct {model_name} {{}}

impl {model_name} {{
    /// Creates a new {model_name} instance.
    pub fn new() -> Self {{
        Self::default()
    }}
}}
"#
    );

    let mut created_model = false;

    if !model_exists {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&model_file)?;

        if let Err(error) = file.write_all(model_source.as_bytes()) {
            drop(file);
            let _ = fs::remove_file(&model_file);
            return Err(error);
        }

        created_model = true;
    }

    if !already_registered {
        if !modules_content.is_empty() && !modules_content.ends_with('\n') {
            modules_content.push('\n');
        }

        modules_content.push_str(&declaration);
        modules_content.push('\n');

        if let Err(error) = fs::write(&modules_file, modules_content) {
            if created_model {
                let _ = fs::remove_file(&model_file);
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
            std::env::temp_dir().join(format!("rustpress-make-model-test-{timestamp}"));

        fs::create_dir_all(project_path.join("src/models")).unwrap();

        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test-project\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(
            project_path.join("src/models/mod.rs"),
            "// Existing models.\n",
        )
        .unwrap();

        project_path
    }

    #[test]
    fn creates_model_file_and_registers_module() {
        let project_path = temporary_project();

        let result = create_model(&project_path, "User");

        assert_eq!(result.unwrap(), "user");

        let model_file = project_path.join("src/models/user.rs");
        let modules_file = project_path.join("src/models/mod.rs");

        assert!(model_file.is_file());

        let source = fs::read_to_string(model_file).unwrap();

        assert!(source.contains("pub struct User"));
        assert!(source.contains("pub fn new() -> Self"));
        assert!(source.contains("#[derive(Debug, Clone, Default)]"));

        let modules = fs::read_to_string(modules_file).unwrap();

        assert!(modules.contains("// Existing models."));
        assert!(modules.contains("pub mod user;"));

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn creates_model_from_pascal_case_name() {
        let project_path = temporary_project();

        let result = create_model(&project_path, "UserProfile");

        assert_eq!(result.unwrap(), "user_profile");

        let source = fs::read_to_string(project_path.join("src/models/user_profile.rs")).unwrap();

        assert!(source.contains("pub struct UserProfile"));

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_duplicate_model_without_overwriting() {
        let project_path = temporary_project();

        create_model(&project_path, "User").unwrap();

        let model_file = project_path.join("src/models/user.rs");
        let original_content = fs::read_to_string(&model_file).unwrap();

        let result = create_model(&project_path, "User");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);

        assert_eq!(fs::read_to_string(model_file).unwrap(), original_content);

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn registers_existing_model_file_without_overwriting() {
        let project_path = temporary_project();

        let model_file = project_path.join("src/models/user.rs");
        let original_content = "// Existing model.\n";

        fs::write(&model_file, original_content).unwrap();

        let result = create_model(&project_path, "User");

        assert_eq!(result.unwrap(), "user");

        assert_eq!(fs::read_to_string(&model_file).unwrap(), original_content);

        let modules = fs::read_to_string(project_path.join("src/models/mod.rs")).unwrap();

        assert!(modules.contains("pub mod user;"));

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_invalid_rustpress_project() {
        let project_path = std::env::temp_dir().join("rustpress-invalid-model-project");

        let result = create_model(&project_path, "User");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn rejects_duplicate_module_declarations() {
        let project_path = temporary_project();

        let modules_file = project_path.join("src/models/mod.rs");

        fs::write(&modules_file, "pub mod user;\npub mod user;\n").unwrap();

        let result = create_model(&project_path, "User");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);

        assert!(!project_path.join("src/models/user.rs").exists());

        fs::remove_dir_all(project_path).unwrap();
    }
}
