mod files;
mod templates;
mod validation;

use std::fs;
use std::io;
use std::path::Path;

pub fn create_project(name: &str) -> io::Result<()> {
    validation::validate_project_name(name)?;

    let project_path = Path::new(name);

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

    fs::create_dir_all(project_path.join("public/css"))?;
    fs::create_dir_all(project_path.join("public/js"))?;
    fs::create_dir_all(project_path.join("public/images"))?;

    fs::create_dir_all(project_path.join("tests"))?;

    files::write_project_files(project_path, name)?;

    println!("Project structure created successfully!");

    Ok(())
}
