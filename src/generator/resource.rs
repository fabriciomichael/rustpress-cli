use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::validation;

struct GeneratedFile {
    path: PathBuf,
    content: String,
}

struct ModuleUpdate {
    path: PathBuf,
    original_content: String,
    updated_content: String,
}

struct ResourcePlan {
    module_name: String,
    resource_name: String,
    files: Vec<GeneratedFile>,
    modules: Vec<ModuleUpdate>,
    web_update: ModuleUpdate,
}

fn to_pascal_case(module_name: &str) -> String {
    module_name
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();

            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn pluralize(module_name: &str) -> String {
    if module_name.ends_with('s')
        || module_name.ends_with('x')
        || module_name.ends_with('z')
        || module_name.ends_with("ch")
        || module_name.ends_with("sh")
    {
        format!("{module_name}es")
    } else if module_name.ends_with('y')
        && module_name
            .chars()
            .rev()
            .nth(1)
            .is_some_and(|character| !"aeiou".contains(character))
    {
        format!("{}ies", &module_name[..module_name.len() - 1])
    } else {
        format!("{module_name}s")
    }
}

fn prepare_module_update(directory: &Path, module_name: &str) -> io::Result<ModuleUpdate> {
    if !directory.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Required directory '{}' does not exist.",
                directory.display()
            ),
        ));
    }

    let path = directory.join("mod.rs");

    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Module file '{}' does not exist.", path.display()),
        ));
    }

    let original_content = fs::read_to_string(&path)?;

    let declaration = format!("pub mod {module_name};");
    let private_declaration = format!("mod {module_name};");

    let count = original_content
        .lines()
        .filter(|line| {
            let line = line.trim();
            line == declaration || line == private_declaration
        })
        .count();

    if count > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Duplicate module declarations for '{module_name}'."),
        ));
    }

    if count == 1 {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Module '{module_name}' is already registered."),
        ));
    }

    let mut updated_content = original_content.clone();

    if !updated_content.is_empty() && !updated_content.ends_with('\n') {
        updated_content.push('\n');
    }

    updated_content.push_str(&declaration);
    updated_content.push('\n');

    Ok(ModuleUpdate {
        path,
        original_content,
        updated_content,
    })
}

fn prepare_web_update(routes_directory: &Path, resource_name: &str) -> io::Result<ModuleUpdate> {
    let path = routes_directory.join("web.rs");

    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Routes file '{}' does not exist.", path.display()),
        ));
    }

    let original_content = fs::read_to_string(&path)?;
    let registration = format!("crate::routes::{resource_name}::register(app)?;");

    let registration_count = original_content
        .lines()
        .filter(|line| line.trim() == registration)
        .count();

    if registration_count > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Duplicate route registrations for '{resource_name}'."),
        ));
    }

    if registration_count == 1 {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Resource routes '{resource_name}' are already registered."),
        ));
    }

    let ok_statement = "    Ok(())";
    let ok_count = original_content
        .lines()
        .filter(|line| line.trim() == "Ok(())")
        .count();

    if ok_count != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Could not identify a unique Ok(()) statement in routes/web.rs.",
        ));
    }

    let position = original_content.find(ok_statement).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Could not locate the route registration insertion point in routes/web.rs.",
        )
    })?;

    let mut updated_content = original_content.clone();

    updated_content.insert_str(position, &format!("    {registration}\n\n"));

    Ok(ModuleUpdate {
        path,
        original_content,
        updated_content,
    })
}

fn prepare_resource(project_path: &Path, name: &str) -> io::Result<ResourcePlan> {
    let module_name = validation::normalize_module_name(name)?;
    let model_name = to_pascal_case(&module_name);
    let resource_name = pluralize(&module_name);

    if !project_path.join("Cargo.toml").is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    let directories = [
        project_path.join("src/controllers"),
        project_path.join("src/models"),
        project_path.join("src/routes"),
        project_path.join("views"),
    ];

    if directories.iter().any(|directory| !directory.is_dir()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Required RustPress project directories are missing.",
        ));
    }

    let targets = [
        project_path.join(format!("src/controllers/{module_name}_controller.rs")),
        project_path.join(format!("src/models/{module_name}.rs")),
        project_path.join(format!("src/routes/{resource_name}.rs")),
    ];

    for target in &targets {
        if target.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Target '{}' already exists.", target.display()),
            ));
        }
    }

    let views_directory = project_path.join("views").join(&resource_name);

    if views_directory.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "Views directory '{}' already exists.",
                views_directory.display()
            ),
        ));
    }

    if !project_path.join("views/layouts/main.html").is_file()
        || !project_path.join("views/partials").is_dir()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The project's main layout or partials directory is missing.",
        ));
    }

    let controller_content = r#"
use rustpress_web::{Request, Response, TemplateContext};

pub fn index() -> Response {
    let mut context = TemplateContext::new();
    context.set("titulo", "Listagem de __RESOURCE__".into());

    Response::new().render(
        "views/__RESOURCE__/index.html",
        "views/layouts/main.html",
        "views/partials",
        &context,
    )
}

pub fn show(request: Request) -> Response {
    let mut context = TemplateContext::new();
    context.set("titulo", "Detalhes de __MODEL__".into());
    context.set("id", request.param("id").unwrap_or("").to_string().into());

    Response::new().render(
        "views/__RESOURCE__/show.html",
        "views/layouts/main.html",
        "views/partials",
        &context,
    )
}

pub fn create() -> Response {
    let mut context = TemplateContext::new();
    context.set("titulo", "Criar __MODEL__".into());

    Response::new().render(
        "views/__RESOURCE__/create.html",
        "views/layouts/main.html",
        "views/partials",
        &context,
    )
}

pub fn edit(request: Request) -> Response {
    let mut context = TemplateContext::new();
    context.set("titulo", "Editar __MODEL__".into());
    context.set("id", request.param("id").unwrap_or("").to_string().into());

    Response::new().render(
        "views/__RESOURCE__/edit.html",
        "views/layouts/main.html",
        "views/partials",
        &context,
    )
}

pub fn store() -> Response {
    Response::error(501, "Resource persistence is not implemented yet.")
}

pub fn update() -> Response {
    Response::error(501, "Resource persistence is not implemented yet.")
}

pub fn destroy() -> Response {
    Response::error(501, "Resource persistence is not implemented yet.")
}
"#
    .replace("__RESOURCE__", &resource_name)
    .replace("__MODEL__", &model_name);

    let model_content = format!(
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

    let routes_content = r#"
use rustpress_web::{App, Request, Response};

pub fn register(app: &mut App) -> Result<(), String> {
    app.get("/__RESOURCE__", |_request: Request, _response: Response| {
        Ok(crate::controllers::__MODULE___controller::index())
    })?;

    app.get("/__RESOURCE__/create", |_request: Request, _response: Response| {
        Ok(crate::controllers::__MODULE___controller::create())
    })?;

    app.post("/__RESOURCE__", |_request: Request, _response: Response| {
        Ok(crate::controllers::__MODULE___controller::store())
    })?;

    app.get("/__RESOURCE__/:id", |request: Request, _response: Response| {
        Ok(crate::controllers::__MODULE___controller::show(request))
    })?;

    app.get("/__RESOURCE__/:id/edit", |request: Request, _response: Response| {
        Ok(crate::controllers::__MODULE___controller::edit(request))
    })?;

    app.put("/__RESOURCE__/:id", |_request: Request, _response: Response| {
        crate::controllers::__MODULE___controller::update()
    })?;

    app.delete("/__RESOURCE__/:id", |_request: Request, _response: Response| {
        crate::controllers::__MODULE___controller::destroy()
    })?;

    Ok(())
}
"#
    .replace("__RESOURCE__", &resource_name)
    .replace("__MODULE__", &module_name);

    let files = vec![
        GeneratedFile {
            path: project_path.join(format!("src/controllers/{module_name}_controller.rs")),
            content: controller_content,
        },
        GeneratedFile {
            path: project_path.join(format!("src/models/{module_name}.rs")),
            content: model_content,
        },
        GeneratedFile {
            path: project_path.join(format!("src/routes/{resource_name}.rs")),
            content: routes_content,
        },
         GeneratedFile {
            path: project_path.join(format!("views/{resource_name}/index.html")),
            content: "<h1>Listagem</h1>\n<p>Lista de recursos registados.</p>\n".to_string(),
        },
        GeneratedFile {
            path: project_path.join(format!("views/{resource_name}/show.html")),
            content: "<h1>Detalhes</h1>\n<p>ID: {{id}}</p>\n".to_string(),
        },
        GeneratedFile {
            path: project_path.join(format!("views/{resource_name}/create.html")),
            content: "<h1>Criar</h1>\n<form action=\"/__RESOURCE__\" method=\"POST\">\n    <!-- Adiciona aqui os campos do recurso. -->\n    <button type=\"submit\">Guardar</button>\n</form>\n"
                .replace("__RESOURCE__", &resource_name),
        },
        GeneratedFile {
            path: project_path.join(format!("views/{resource_name}/edit.html")),
            content: "<h1>Editar</h1>\n<p>ID: {{id}}</p>\n<form action=\"/__RESOURCE__/{{id}}\" method=\"POST\">\n    <!-- Adiciona aqui os campos do recurso. -->\n    <button type=\"submit\">Actualizar</button>\n</form>\n"
                .replace("__RESOURCE__", &resource_name),
        },
        ];

    let modules = vec![
        prepare_module_update(&directories[0], &format!("{module_name}_controller"))?,
        prepare_module_update(&directories[1], &module_name)?,
        prepare_module_update(&directories[2], &resource_name)?,
    ];

    let web_update = prepare_web_update(&directories[2], &resource_name)?;

    Ok(ResourcePlan {
        module_name,
        resource_name,
        files,
        modules,
        web_update,
    })
}

fn write_generated_file(file: &GeneratedFile) -> io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file.path)?;

    if let Err(error) = output.write_all(file.content.as_bytes()) {
        drop(output);
        let _ = fs::remove_file(&file.path);
        return Err(error);
    }

    Ok(())
}

fn update_module(module: &ModuleUpdate) -> io::Result<()> {
    use std::io::Write;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&module.path)?;

    if let Err(error) = file.write_all(module.updated_content.as_bytes()) {
        drop(file);
        let _ = fs::write(&module.path, &module.original_content);
        return Err(error);
    }

    Ok(())
}

pub fn create_resource(project_path: &Path, name: &str) -> io::Result<String> {
    let plan = prepare_resource(project_path, name)?;
    let mut created_files = Vec::new();
    let mut updated_modules: Vec<&ModuleUpdate> = Vec::new();

    let views_directory = project_path.join("views").join(&plan.resource_name);

    fs::create_dir(&views_directory)?;

    for file in &plan.files {
        if let Err(error) = write_generated_file(file) {
            for path in created_files.iter().rev() {
                let _ = fs::remove_file(path);
            }

            let _ = fs::remove_dir(&views_directory);

            return Err(error);
        }

        created_files.push(file.path.clone());
    }

    for module in plan.modules.iter().chain(std::iter::once(&plan.web_update)) {
        if let Err(error) = update_module(module) {
            for path in created_files.iter().rev() {
                let _ = fs::remove_file(path);
            }

            for updated in &updated_modules {
                let _ = fs::write(&updated.path, &updated.original_content);
            }

            let _ = fs::remove_dir(&views_directory);

            return Err(error);
        }

        updated_modules.push(module);
    }

    Ok(plan.module_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_project() -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let project_path =
            std::env::temp_dir().join(format!("rustpress-resource-test-{timestamp}"));

        fs::create_dir_all(project_path.join("src/controllers")).unwrap();
        fs::create_dir_all(project_path.join("src/models")).unwrap();
        fs::create_dir_all(project_path.join("src/routes")).unwrap();
        fs::create_dir_all(project_path.join("views/layouts")).unwrap();
        fs::create_dir_all(project_path.join("views/partials")).unwrap();

        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test-project\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(
            project_path.join("src/controllers/mod.rs"),
            "// Controllers.\n",
        )
        .unwrap();

        fs::write(project_path.join("src/models/mod.rs"), "// Models.\n").unwrap();

        fs::write(project_path.join("src/routes/mod.rs"), "pub mod web;\n").unwrap();

        fs::write(
            project_path.join("src/routes/web.rs"),
            r#"use rustpress_web::{App, Request, Response};

        pub fn register(app: &mut App) -> Result<(), rustpress_web::HandlerError> {
            app.get("/", |_request: Request, _response: Response| {
                Ok(crate::controllers::home_controller::index())
            })?;

            Ok(())
        }
        "#,
        )
        .unwrap();

        fs::write(
            project_path.join("views/layouts/main.html"),
            "<!DOCTYPE html><html><head><title>{{titulo}}</title></head><body>{{{conteudo}}}</body></html>",
        )
        .unwrap();

        project_path
    }

    #[test]
    fn creates_resource_files_and_registers_modules() {
        let project_path = temporary_project();

        let result = create_resource(&project_path, "Product");

        assert_eq!(result.unwrap(), "product");

        assert!(
            project_path
                .join("src/controllers/product_controller.rs")
                .is_file()
        );
        assert!(project_path.join("src/models/product.rs").is_file());
        assert!(project_path.join("src/routes/products.rs").is_file());

        for view in ["index", "show", "create", "edit"] {
            assert!(
                project_path
                    .join(format!("views/products/{view}.html"))
                    .is_file()
            );
        }

        let create_view =
            fs::read_to_string(project_path.join("views/products/create.html")).unwrap();

        assert!(create_view.contains("action=\"/products\""));
        assert!(create_view.contains("method=\"POST\""));

        let edit_view = fs::read_to_string(project_path.join("views/products/edit.html")).unwrap();

        assert!(edit_view.contains("action=\"/products/{{id}}\""));
        assert!(edit_view.contains("method=\"POST\""));

        let controllers = fs::read_to_string(project_path.join("src/controllers/mod.rs")).unwrap();
        let models = fs::read_to_string(project_path.join("src/models/mod.rs")).unwrap();
        let routes = fs::read_to_string(project_path.join("src/routes/mod.rs")).unwrap();

        assert!(controllers.contains("pub mod product_controller;"));
        assert!(models.contains("pub mod product;"));
        assert!(routes.contains("pub mod products;"));

        let web = fs::read_to_string(project_path.join("src/routes/web.rs")).unwrap();

        assert!(web.contains("crate::routes::products::register(app)?;"));
        assert!(web.contains("crate::controllers::home_controller::index()"));

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_duplicate_resource_without_overwriting() {
        let project_path = temporary_project();

        create_resource(&project_path, "Product").unwrap();

        let controller = project_path.join("src/controllers/product_controller.rs");
        let original_content = fs::read_to_string(&controller).unwrap();

        let result = create_resource(&project_path, "Product");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(controller).unwrap(), original_content);

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_existing_views_directory_without_modifying_project() {
        let project_path = temporary_project();

        fs::create_dir(project_path.join("views/products")).unwrap();
        fs::write(
            project_path.join("views/products/custom.html"),
            "<h1>My custom page</h1>",
        )
        .unwrap();

        let result = create_resource(&project_path, "Product");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);

        assert!(project_path.join("views/products/custom.html").is_file());

        assert!(
            !project_path
                .join("src/controllers/product_controller.rs")
                .exists()
        );

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_duplicate_module_declarations_without_creating_files() {
        let project_path = temporary_project();

        fs::write(
            project_path.join("src/controllers/mod.rs"),
            "pub mod product_controller;\npub mod product_controller;\n",
        )
        .unwrap();

        let result = create_resource(&project_path, "Product");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);

        assert!(
            !project_path
                .join("src/controllers/product_controller.rs")
                .exists()
        );

        assert!(!project_path.join("views/products").exists());

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn rejects_existing_route_registration_without_creating_files() {
        let project_path = temporary_project();

        let web_file = project_path.join("src/routes/web.rs");

        let original_content = fs::read_to_string(&web_file).unwrap();

        let modified_content = original_content.replace(
            "    Ok(())",
            "    crate::routes::products::register(app)?;\n\n    Ok(())",
        );

        fs::write(&web_file, &modified_content).unwrap();

        let result = create_resource(&project_path, "Product");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);

        assert!(
            !project_path
                .join("src/controllers/product_controller.rs")
                .exists()
        );
        assert!(!project_path.join("src/models/product.rs").exists());
        assert!(!project_path.join("src/routes/products.rs").exists());
        assert!(!project_path.join("views/products").exists());

        assert_eq!(fs::read_to_string(&web_file).unwrap(), modified_content);

        fs::remove_dir_all(project_path).unwrap();
    }

    #[test]
    fn pluralizes_resource_names() {
        assert_eq!(pluralize("product"), "products");
        assert_eq!(pluralize("category"), "categories");
        assert_eq!(pluralize("box"), "boxes");
        assert_eq!(pluralize("class"), "classes");
        assert_eq!(pluralize("church"), "churches");
        assert_eq!(pluralize("key"), "keys");
    }
}
