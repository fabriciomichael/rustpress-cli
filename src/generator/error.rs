use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::templates;

/// Generates an HTTP error page in the RustPress project.
pub fn create_error_page(project_path: &Path, code: &str) -> io::Result<String> {
    let template = match code {
        "400" => templates::ERROR_400_HTML,
        "401" => templates::ERROR_401_HTML,
        "403" => templates::ERROR_403_HTML,
        "404" => templates::ERROR_404_HTML,
        "405" => templates::ERROR_405_HTML,
        "500" => templates::ERROR_500_HTML,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported HTTP error code. Supported codes: 400, 401, 403, 404, 405, 500.",
            ));
        }
    };

    let cargo_file = project_path.join("Cargo.toml");
    let errors_dir = project_path.join("views/errors");

    if !cargo_file.is_file() || !errors_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    let error_file = errors_dir.join(format!("{code}.html"));

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&error_file)?;

    if let Err(error) = file.write_all(template.trim_start().as_bytes()) {
        drop(file);
        let _ = fs::remove_file(&error_file);
        return Err(error);
    }

    Ok(code.to_string())
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

        let project_path = std::env::temp_dir().join(format!("rustpress-error-test-{timestamp}"));

        fs::create_dir_all(project_path.join("views/errors")).unwrap();

        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test-project\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        project_path
    }

    #[test]
    fn creates_error_page() {
        let project_path = temporary_project();

        let result = create_error_page(&project_path, "404");

        assert_eq!(result.unwrap(), "404");

        let error_file = project_path.join("views/errors/404.html");

        assert!(error_file.is_file());

        let source = fs::read_to_string(error_file).unwrap();

        assert!(source.contains("<h1>404</h1>"));
        assert!(source.contains("Página não encontrada"));

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn creates_all_supported_error_pages() {
        let project_path = temporary_project();

        for code in ["400", "401", "403", "404", "405", "500"] {
            create_error_page(&project_path, code).unwrap();

            assert!(
                project_path
                    .join(format!("views/errors/{code}.html"))
                    .is_file()
            );
        }

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_unsupported_error_code() {
        let project_path = temporary_project();

        let result = create_error_page(&project_path, "418");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_duplicate_error_page_without_overwriting() {
        let project_path = temporary_project();

        let error_file = project_path.join("views/errors/404.html");
        let original_content = "<!-- Existing error page -->";

        fs::write(&error_file, original_content).unwrap();

        let result = create_error_page(&project_path, "404");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);

        assert_eq!(fs::read_to_string(error_file).unwrap(), original_content);

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_invalid_rustpress_project() {
        let project_path = std::env::temp_dir().join("rustpress-invalid-error-project");

        let result = create_error_page(&project_path, "404");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }
}
