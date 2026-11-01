use std::fs;
use std::io;
use std::path::Path;

use super::templates;

pub fn write_project_files(root: &Path, name: &str) -> io::Result<()> {
    let cargo_toml = templates::CARGO_TOML.replace("{{project_name}}", name);

    let readme = templates::README_MD.replace("{{project_name}}", name);

    fs::write(root.join("Cargo.toml"), cargo_toml.trim_start())?;

    fs::write(root.join("README.md"), readme.trim_start())?;

    fs::write(root.join("src/main.rs"), templates::MAIN_RS.trim_start())?;

    fs::write(
        root.join("src/routes/web.rs"),
        templates::ROUTES_RS.trim_start(),
    )?;

    fs::write(
        root.join("src/controllers/home_controller.rs"),
        templates::CONTROLLER_RS.trim_start(),
    )?;

    fs::write(
        root.join("views/pages/home.html"),
        templates::HOME_HTML.trim_start(),
    )?;

    fs::write(
        root.join("src/routes/mod.rs"),
        templates::MODULE_RS.trim_start(),
    )?;

    fs::write(
        root.join("src/controllers/mod.rs"),
        templates::CONTROLLERS_MOD_RS.trim_start(),
    )?;

    fs::write(
        root.join("src/models/mod.rs"),
        templates::EMPTY_MOD_RS.trim_start(),
    )?;

    fs::write(
        root.join("src/middleware/mod.rs"),
        templates::EMPTY_MOD_RS.trim_start(),
    )?;

    fs::write(
        root.join("src/config/mod.rs"),
        templates::EMPTY_MOD_RS.trim_start(),
    )?;

    Ok(())
}
