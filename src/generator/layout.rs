use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::validation;

const LAYOUT_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }}</title>
</head>
<body>
    {{ content }}
</body>
</html>
"#;

/// Generates a new layout in the RustPress project.
pub fn create_layout(project_path: &Path, name: &str) -> io::Result<String> {
    let layout_name = validation::normalize_module_name(name)?;

    let cargo_file = project_path.join("Cargo.toml");
    let layouts_dir = project_path.join("views/layouts");

    if !cargo_file.is_file() || !layouts_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    let layout_file = layouts_dir.join(format!("{layout_name}.html"));

    if layout_file.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Layout already exists.",
        ));
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&layout_file)?;

    if let Err(error) = file.write_all(LAYOUT_TEMPLATE.as_bytes()) {
        drop(file);
        let _ = fs::remove_file(&layout_file);
        return Err(error);
    }

    Ok(layout_name)
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

        let project_path = std::env::temp_dir().join(format!("rustpress-layout-test-{timestamp}"));

        fs::create_dir_all(project_path.join("views/layouts")).unwrap();

        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test-project\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        project_path
    }

    #[test]
    fn creates_layout_file() {
        let project_path = temporary_project();

        let result = create_layout(&project_path, "AdminLayout");

        assert_eq!(result.unwrap(), "admin_layout");

        let layout_file = project_path.join("views/layouts/admin_layout.html");

        assert!(layout_file.is_file());

        let source = fs::read_to_string(layout_file).unwrap();

        assert!(source.contains("<!DOCTYPE html>"));
        assert!(source.contains("<title>{{ title }}</title>"));
        assert!(source.contains("{{ content }}"));

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_duplicate_layout_without_overwriting() {
        let project_path = temporary_project();

        let layout_file = project_path.join("views/layouts/admin.html");
        let original_content = "<!-- Existing layout -->\n";

        fs::write(&layout_file, original_content).unwrap();

        let result = create_layout(&project_path, "Admin");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);

        assert_eq!(fs::read_to_string(layout_file).unwrap(), original_content);

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_invalid_rustpress_project() {
        let project_path = std::env::temp_dir().join("rustpress-invalid-layout-project");

        let result = create_layout(&project_path, "Admin");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn creates_layout_from_pascal_case_name() {
        let project_path = temporary_project();

        let result = create_layout(&project_path, "AdminPanel");

        assert_eq!(result.unwrap(), "admin_panel");

        assert!(
            project_path
                .join("views/layouts/admin_panel.html")
                .is_file()
        );

        fs::remove_dir_all(project_path).unwrap();
    }
}
