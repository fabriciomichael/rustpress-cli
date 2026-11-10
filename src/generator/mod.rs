mod build;
mod files;
mod make;
mod templates;
mod validation;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn create_project(name: &str) -> io::Result<()> {
    create_project_at(name, Path::new("."))
}

pub fn create_project_at(name: &str, base_path: &Path) -> io::Result<()> {
    validation::validate_project_name(name)?;

    let project_path: PathBuf = base_path.join(name);

    if project_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "The directory already exists.",
        ));
    }

    fs::create_dir_all(project_path.join("src/routes"))?;
    fs::create_dir_all(project_path.join("src/controllers"))?;
    fs::create_dir_all(project_path.join("src/models"))?;
    fs::create_dir_all(project_path.join("src/middleware"))?;
    fs::create_dir_all(project_path.join("src/config"))?;

    fs::create_dir_all(project_path.join("views/layouts"))?;
    fs::create_dir_all(project_path.join("views/pages"))?;
    fs::create_dir_all(project_path.join("views/partials"))?;
    fs::create_dir_all(project_path.join("views/errors"))?;

    fs::create_dir_all(project_path.join("public/css"))?;
    fs::create_dir_all(project_path.join("public/js"))?;
    fs::create_dir_all(project_path.join("public/images"))?;

    fs::create_dir_all(project_path.join("tests"))?;

    // Storage structure for files uploaded by users.
    fs::create_dir_all(project_path.join("storage/uploads/images"))?;
    fs::create_dir_all(project_path.join("storage/uploads/documents"))?;
    fs::create_dir_all(project_path.join("storage/uploads/receipts"))?;
    fs::create_dir_all(project_path.join("storage/uploads/profiles"))?;
    fs::create_dir_all(project_path.join("storage/uploads/products"))?;
    fs::create_dir_all(project_path.join("storage/uploads/others"))?;
    fs::create_dir_all(project_path.join("storage/temp"))?;

    files::write_project_files(&project_path, name)?;

    println!("Project structure created successfully!");

    Ok(())
}

pub fn build_project(project_path: &Path, include: &[PathBuf]) -> io::Result<()> {
    build::build_project(project_path, include)
}

pub fn build_project_with_binary(
    project_path: &Path,
    include: &[PathBuf],
    binary_name: Option<&str>,
) -> io::Result<()> {
    build::build_project_with_binary(project_path, include, binary_name)
}

pub fn make_controller(project_path: &Path, name: &str) -> io::Result<String> {
    make::create_controller(project_path, name)
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

        std::env::temp_dir().join(format!("rustpress-test-{}", timestamp))
    }

    #[test]
    fn creates_project_in_custom_directory() {
        let base_path = temporary_directory();

        let result = create_project_at("test-project", &base_path);

        assert!(result.is_ok());

        let project_path = base_path.join("test-project");

        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src/main.rs").exists());
        assert!(project_path.join("src/routes/web.rs").exists());
        assert!(
            project_path
                .join("src/controllers/home_controller.rs")
                .exists()
        );
        assert!(project_path.join("views/pages/home.html").exists());
        assert!(project_path.join("storage/uploads/images").exists());
        assert!(project_path.join("views/errors/400.html").exists());
        assert!(project_path.join("views/errors/401.html").exists());
        assert!(project_path.join("views/errors/403.html").exists());
        assert!(project_path.join("views/errors/404.html").exists());
        assert!(project_path.join("views/errors/405.html").exists());
        assert!(project_path.join("views/errors/500.html").exists());

        assert!(project_path.join("views/layouts/main.html").exists());

        assert!(project_path.join("storage/uploads/documents").exists());
        assert!(project_path.join("storage/uploads/receipts").exists());
        assert!(project_path.join("storage/uploads/profiles").exists());
        assert!(project_path.join("storage/uploads/products").exists());
        assert!(project_path.join("storage/uploads/others").exists());
        assert!(project_path.join("storage/temp").exists());
        assert!(project_path.join("views/errors").exists());

        assert!(project_path.join("storage/README.md").exists());
        assert!(project_path.join("storage/.gitkeep").exists());

        fs::remove_dir_all(base_path).unwrap();
    }

    #[test]
    fn rejects_existing_project_directory() {
        let base_path = temporary_directory();
        let project_path = base_path.join("existing-project");

        fs::create_dir_all(&project_path).unwrap();

        let result = create_project_at("existing-project", &base_path);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);

        fs::remove_dir_all(base_path).unwrap();
    }

    #[test]
    fn rejects_invalid_project_name() {
        let base_path = temporary_directory();

        let result = create_project_at("123invalid", &base_path);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);

        assert!(!base_path.exists());
    }
}
