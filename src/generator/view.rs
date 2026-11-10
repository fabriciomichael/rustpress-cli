use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::validation;

pub fn create_view(project_path: &Path, name: &str) -> io::Result<String> {
    let module_name = validation::normalize_module_name(name)?;

    let cargo_file = project_path.join("Cargo.toml");
    let views_dir = project_path.join("views");
    let pages_dir = views_dir.join("pages");

    if !cargo_file.is_file() || !views_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    fs::create_dir_all(&pages_dir)?;

    let view_file = pages_dir.join(format!("{module_name}.html"));

    let view_source = "<h1>{{titulo}}</h1>\n";

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&view_file)?;

    if let Err(error) = file.write_all(view_source.as_bytes()) {
        drop(file);
        let _ = fs::remove_file(&view_file);
        return Err(error);
    }

    Ok(module_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("rustpress-view-test-{timestamp}"))
    }

    fn create_project(path: &Path) {
        fs::create_dir_all(path.join("views")).unwrap();
        fs::write(path.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
    }

    #[test]
    fn creates_view_file() {
        let project = temporary_directory();
        create_project(&project);

        let result = create_view(&project, "home");

        assert_eq!(result.unwrap(), "home");

        let view_file = project.join("views/pages/home.html");

        assert!(view_file.is_file());
        assert_eq!(
            fs::read_to_string(view_file).unwrap(),
            "<h1>{{titulo}}</h1>\n"
        );

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn creates_view_from_pascal_case_name() {
        let project = temporary_directory();
        create_project(&project);

        let result = create_view(&project, "UserProfile");

        assert_eq!(result.unwrap(), "user_profile");
        assert!(project.join("views/pages/user_profile.html").is_file());

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn rejects_duplicate_view_without_overwriting() {
        let project = temporary_directory();
        create_project(&project);

        let pages = project.join("views/pages");
        fs::create_dir_all(&pages).unwrap();

        let view_file = pages.join("home.html");
        fs::write(&view_file, "<h1>Original</h1>").unwrap();

        let result = create_view(&project, "home");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(view_file).unwrap(), "<h1>Original</h1>");

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn rejects_invalid_rustpress_project() {
        let project = temporary_directory();
        fs::create_dir_all(&project).unwrap();

        let result = create_view(&project, "home");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn creates_pages_directory_when_missing() {
        let project = temporary_directory();
        create_project(&project);

        let result = create_view(&project, "dashboard");

        assert!(result.is_ok());
        assert!(project.join("views/pages/dashboard.html").is_file());

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn rejects_invalid_view_name() {
        let project = temporary_directory();
        create_project(&project);

        let result = create_view(&project, "123-invalid");

        assert!(result.is_err());
        assert!(!project.join("views/pages").exists());

        fs::remove_dir_all(project).unwrap();
    }
}
