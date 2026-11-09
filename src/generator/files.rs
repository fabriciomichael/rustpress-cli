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

    let controller = templates::CONTROLLER_RS.replace("{{project_name}}", name);

    fs::write(
        root.join("src/controllers/home_controller.rs"),
        controller.trim_start(),
    )?;

    let home_html = templates::HOME_HTML.replace("{{project_name}}", name);

    fs::write(root.join("views/pages/home.html"), home_html.trim_start())?;

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

    fs::write(
        root.join("storage/README.md"),
        templates::STORAGE_README.trim_start(),
    )?;

    fs::write(root.join("storage/.gitkeep"), templates::STORAGE_GITKEEP)?;

    // Páginas de erro padrão.
    fs::write(
        root.join("views/errors/400.html"),
        templates::ERROR_400_HTML.trim_start(),
    )?;

    fs::write(
        root.join("views/errors/401.html"),
        templates::ERROR_401_HTML.trim_start(),
    )?;

    fs::write(
        root.join("views/errors/403.html"),
        templates::ERROR_403_HTML.trim_start(),
    )?;

    fs::write(
        root.join("views/errors/404.html"),
        templates::ERROR_404_HTML.trim_start(),
    )?;

    fs::write(
        root.join("views/errors/405.html"),
        templates::ERROR_405_HTML.trim_start(),
    )?;

    fs::write(
        root.join("views/errors/500.html"),
        templates::ERROR_500_HTML.trim_start(),
    )?;

    // Parciais HTML padrão.
    let header = templates::HEADER_PARTIAL_HTML.replace("{{project_name}}", name);

    fs::write(root.join("views/partials/header.html"), header.trim_start())?;

    let footer = templates::FOOTER_PARTIAL_HTML.replace("{{project_name}}", name);

    fs::write(root.join("views/partials/footer.html"), footer.trim_start())?;

    fs::write(
        root.join("views/partials/navbar.html"),
        templates::NAVBAR_PARTIAL_HTML.trim_start(),
    )?;

    fs::write(
        root.join("views/partials/flash.html"),
        templates::FLASH_PARTIAL_HTML.trim_start(),
    )?;

    fs::write(
        root.join("views/partials/loading.html"),
        templates::LOADING_PARTIAL_HTML.trim_start(),
    )?;

    fs::write(
        root.join("views/layouts/main.html"),
        templates::MAIN_LAYOUT_HTML.trim_start(),
    )?;

    Ok(())
}
