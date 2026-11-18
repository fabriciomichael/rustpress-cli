use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::validation;

const PARTIAL_TEMPLATE: &str = "<!-- Partial -->\n";

pub fn create_partial(project_path: &Path, name: &str) -> io::Result<String> {
    let partial_name = validation::normalize_module_name(name)?;

    let cargo_file = project_path.join("Cargo.toml");
    let views_dir = project_path.join("views");
    let partials_dir = views_dir.join("partials");

    if !cargo_file.is_file() || !views_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    fs::create_dir_all(&partials_dir)?;

    let partial_file = partials_dir.join(format!("{partial_name}.html"));

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial_file)?;

    if let Err(error) = file.write_all(PARTIAL_TEMPLATE.as_bytes()) {
        drop(file);
        let _ = fs::remove_file(&partial_file);
        return Err(error);
    }

    Ok(partial_name)
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

        let project = std::env::temp_dir().join(format!("rustpress-partial-test-{timestamp}"));

        fs::create_dir_all(project.join("views")).unwrap();

        fs::write(
            project.join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        project
    }

    #[test]
    fn creates_partial_file() {
        let project = temporary_project();

        let result = create_partial(&project, "navbar");

        assert_eq!(result.unwrap(), "navbar");

        let partial_file = project.join("views/partials/navbar.html");

        assert!(partial_file.is_file());
        assert_eq!(fs::read_to_string(partial_file).unwrap(), PARTIAL_TEMPLATE);

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn creates_partial_from_pascal_case_name() {
        let project = temporary_project();

        let result = create_partial(&project, "MainNavbar");

        assert_eq!(result.unwrap(), "main_navbar");
        assert!(project.join("views/partials/main_navbar.html").is_file());

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn rejects_duplicate_partial_without_overwriting() {
        let project = temporary_project();

        let partials = project.join("views/partials");
        fs::create_dir_all(&partials).unwrap();

        let partial_file = partials.join("navbar.html");
        let original_content = "<!-- Existing navbar -->\n";

        fs::write(&partial_file, original_content).unwrap();

        let result = create_partial(&project, "navbar");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);

        assert_eq!(fs::read_to_string(partial_file).unwrap(), original_content);

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn rejects_invalid_rustpress_project() {
        let project = std::env::temp_dir().join("rustpress-invalid-partial-project");

        let result = create_partial(&project, "navbar");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn rejects_invalid_partial_name() {
        let project = temporary_project();

        let result = create_partial(&project, "123-navbar");

        assert!(result.is_err());
        assert!(!project.join("views/partials").exists());

        fs::remove_dir_all(project).unwrap();
    }
}
