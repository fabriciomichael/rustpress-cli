use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let path = std::env::temp_dir().join(format!(
            "rustpress-cli-integration-{}-{timestamp}",
            std::process::id()
        ));

        fs::create_dir_all(path.join("src/controllers")).unwrap();
        fs::create_dir_all(path.join("src/models")).unwrap();
        fs::create_dir_all(path.join("src/middleware")).unwrap();
        fs::create_dir_all(path.join("views")).unwrap();
        fs::create_dir_all(path.join("views/pages")).unwrap();
        fs::create_dir_all(path.join("views/layouts")).unwrap();
        fs::create_dir_all(path.join("views/errors")).unwrap();
        fs::create_dir_all(path.join("views/components")).unwrap();

        fs::write(
            path.join("Cargo.toml"),
            "[package]\nname = \"integration-test\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();

        fs::write(path.join("src/models/mod.rs"), "// Existing models.\n").unwrap();

        fs::write(
            path.join("src/controllers/mod.rs"),
            "// Existing controllers.\n",
        )
        .unwrap();

        fs::write(
            path.join("src/middleware/mod.rs"),
            "// Existing middlewares.\n",
        )
        .unwrap();

        Self { path }
    }

    fn run_make(&self, component: &str, name: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_rustpress"))
            .args(["make", component, name])
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute RustPress CLI")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn creates_middleware_and_registers_module() {
    let project = TemporaryProject::new();

    let output = project.run_make("middleware", "LoggingMiddleware");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let middleware_file = project.path.join("src/middleware/logging_middleware.rs");

    assert!(middleware_file.is_file());

    let source = fs::read_to_string(middleware_file).unwrap();

    assert!(source.contains("pub fn handle(request: &Request) -> MiddlewareResult"));
    assert!(source.contains("MiddlewareResult::Next"));

    let modules = fs::read_to_string(project.path.join("src/middleware/mod.rs")).unwrap();

    assert!(modules.contains("// Existing middlewares."));
    assert!(modules.contains("pub mod logging_middleware;"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Middleware 'logging_middleware' created successfully."));
}

#[test]
fn rejects_duplicate_middleware_without_overwriting() {
    let project = TemporaryProject::new();

    let first = project.run_make("middleware", "AuthMiddleware");
    assert!(first.status.success());

    let middleware_file = project.path.join("src/middleware/auth_middleware.rs");

    let original_content = fs::read_to_string(&middleware_file).unwrap();

    let second = project.run_make("middleware", "AuthMiddleware");

    assert!(!second.status.success());

    assert_eq!(
        fs::read_to_string(middleware_file).unwrap(),
        original_content
    );
}

#[test]
fn creates_controller_and_registers_module() {
    let project = TemporaryProject::new();

    let output = project.run_make("controller", "UserController");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let controller_file = project.path.join("src/controllers/user_controller.rs");

    assert!(controller_file.is_file());

    let source = fs::read_to_string(controller_file).unwrap();
    assert!(source.contains("pub fn index() -> Response"));

    let modules = fs::read_to_string(project.path.join("src/controllers/mod.rs")).unwrap();

    assert!(modules.contains("// Existing controllers."));
    assert!(modules.contains("pub mod user_controller;"));
}

#[test]
fn rejects_unsupported_component() {
    let project = TemporaryProject::new();

    let output = project.run_make("service", "UserService");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Unsupported component type 'service'."));
}

#[test]
fn rejects_invalid_component_name() {
    let project = TemporaryProject::new();

    let output = project.run_make("middleware", "invalid-name");

    assert!(!output.status.success());

    assert!(!project.path.join("src/middleware/invalid_name.rs").exists());
}

#[test]
fn creates_view_and_generates_html_file() {
    let project = TemporaryProject::new();

    let output = project.run_make("view", "UserProfile");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let view_file = project.path.join("views/pages/user_profile.html");

    assert!(view_file.is_file());

    let source = fs::read_to_string(view_file).unwrap();

    assert!(source.contains("{% extends \"layouts/main.html\" %}"));
    assert!(source.contains("{% block content %}"));
    assert!(source.contains("{{ titulo }}"));
    assert!(source.contains("views/pages/user_profile.html"));
    assert!(source.contains("{% endblock content %}"));

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("View 'user_profile' created successfully."));
}

#[test]
fn rejects_duplicate_view_without_overwriting() {
    let project = TemporaryProject::new();

    let first = project.run_make("view", "Dashboard");

    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let view_file = project.path.join("views/pages/dashboard.html");

    let original_content = fs::read_to_string(&view_file).unwrap();

    let second = project.run_make("view", "Dashboard");

    assert!(!second.status.success());

    assert_eq!(fs::read_to_string(view_file).unwrap(), original_content);
}

#[test]
fn creates_all_components_and_registers_modules() {
    let project = TemporaryProject::new();

    let output = project.run_make("all", "User");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Components 'user' created successfully.")
    );

    let controller = project.path.join("src/controllers/user_controller.rs");
    let middleware = project.path.join("src/middleware/user_middleware.rs");
    let model = project.path.join("src/models/user.rs");
    let view = project.path.join("views/pages/user.html");

    assert!(controller.is_file());
    assert!(middleware.is_file());
    assert!(model.is_file());
    assert!(view.is_file());

    assert!(
        fs::read_to_string(controller)
            .unwrap()
            .contains("pub fn index() -> Response")
    );

    assert!(
        fs::read_to_string(middleware)
            .unwrap()
            .contains("MiddlewareResult::Next")
    );

    assert!(
        fs::read_to_string(model)
            .unwrap()
            .contains("pub struct User")
    );

    let view_source = fs::read_to_string(view).unwrap();

    assert!(view_source.contains("{% extends \"layouts/main.html\" %}"));
    assert!(view_source.contains("{% block content %}"));
    assert!(view_source.contains("{{ titulo }}"));
    assert!(view_source.contains("views/pages/user.html"));
    assert!(view_source.contains("{% endblock content %}"));

    assert!(
        fs::read_to_string(project.path.join("src/controllers/mod.rs"))
            .unwrap()
            .contains("pub mod user_controller;")
    );

    assert!(
        fs::read_to_string(project.path.join("src/middleware/mod.rs"))
            .unwrap()
            .contains("pub mod user_middleware;")
    );

    assert!(
        fs::read_to_string(project.path.join("src/models/mod.rs"))
            .unwrap()
            .contains("pub mod user;")
    );
}

#[test]
fn rejects_make_all_when_any_target_exists_without_partial_generation() {
    let project = TemporaryProject::new();

    let existing_model = project.path.join("src/models/user.rs");
    let original_model = "// Preserve this model.\n";

    fs::write(&existing_model, original_model).unwrap();

    let controller_modules = project.path.join("src/controllers/mod.rs");
    let middleware_modules = project.path.join("src/middleware/mod.rs");
    let model_modules = project.path.join("src/models/mod.rs");

    let original_controllers = fs::read_to_string(&controller_modules).unwrap();
    let original_middlewares = fs::read_to_string(&middleware_modules).unwrap();
    let original_models = fs::read_to_string(&model_modules).unwrap();

    let output = project.run_make("all", "User");

    assert!(!output.status.success());

    assert!(
        !project
            .path
            .join("src/controllers/user_controller.rs")
            .exists()
    );

    assert!(
        !project
            .path
            .join("src/middleware/user_middleware.rs")
            .exists()
    );

    assert!(!project.path.join("views/pages/user.html").exists());

    assert_eq!(fs::read_to_string(existing_model).unwrap(), original_model);

    assert_eq!(
        fs::read_to_string(controller_modules).unwrap(),
        original_controllers
    );

    assert_eq!(
        fs::read_to_string(middleware_modules).unwrap(),
        original_middlewares
    );

    assert_eq!(fs::read_to_string(model_modules).unwrap(), original_models);
}

#[test]
fn creates_layout_and_generates_html_file() {
    let project = TemporaryProject::new();

    let output = project.run_make("layout", "AdminPanel");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let layout_file = project.path.join("views/layouts/admin_panel.html");

    assert!(layout_file.is_file());

    let source = fs::read_to_string(layout_file).unwrap();

    assert!(source.contains("<!DOCTYPE html>"));
    assert!(source.contains("<title>{{ title }}</title>"));
    assert!(source.contains("{{ content }}"));

    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("Layout 'admin_panel' created successfully.")
    );
}

#[test]
fn rejects_duplicate_layout_without_overwriting() {
    let project = TemporaryProject::new();

    let first = project.run_make("layout", "Admin");
    assert!(first.status.success());

    let layout_file = project.path.join("views/layouts/admin.html");
    let original_content = fs::read_to_string(&layout_file).unwrap();

    let second = project.run_make("layout", "Admin");

    assert!(!second.status.success());

    assert_eq!(fs::read_to_string(layout_file).unwrap(), original_content);
}

#[test]
fn creates_partial_and_generates_html_file() {
    let project = TemporaryProject::new();

    let output = project.run_make("partial", "MainNavbar");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let partial_file = project.path.join("views/partials/main_navbar.html");

    assert!(partial_file.is_file());

    let source = fs::read_to_string(partial_file).unwrap();

    assert_eq!(source, "<!-- Partial -->\n");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Partial 'main_navbar' created successfully."));
}

#[test]
fn rejects_duplicate_partial_without_overwriting() {
    let project = TemporaryProject::new();

    let first = project.run_make("partial", "Navbar");

    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let partial_file = project.path.join("views/partials/navbar.html");

    let original_content = fs::read_to_string(&partial_file).unwrap();

    let second = project.run_make("partial", "Navbar");

    assert!(!second.status.success());

    assert_eq!(fs::read_to_string(partial_file).unwrap(), original_content);
}

#[test]
fn creates_error_page_and_generates_html_file() {
    let project = TemporaryProject::new();

    let output = project.run_make("error", "404");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let error_file = project.path.join("views/errors/404.html");

    assert!(error_file.is_file());

    let source = fs::read_to_string(error_file).unwrap();

    assert!(source.contains("<h1>404</h1>"));
    assert!(source.contains("Página não encontrada"));

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Error page '404' created successfully."));
}

#[test]
fn rejects_duplicate_error_page_without_overwriting() {
    let project = TemporaryProject::new();

    let first = project.run_make("error", "404");

    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let error_file = project.path.join("views/errors/404.html");
    let original_content = fs::read_to_string(&error_file).unwrap();

    let second = project.run_make("error", "404");

    assert!(!second.status.success());

    assert_eq!(fs::read_to_string(error_file).unwrap(), original_content);
}

#[test]
fn creates_component_and_generates_html_file() {
    let project = TemporaryProject::new();

    let output = project.run_make("components", "button");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let component_file = project.path.join("views/components/button.html");

    assert!(component_file.is_file());

    let content = fs::read_to_string(component_file).unwrap();

    assert!(content.contains("<button"));
    assert!(content.contains("Button"));
}

#[test]
fn rejects_duplicate_component_without_overwriting() {
    let project = TemporaryProject::new();

    let first_output = project.run_make("components", "button");

    assert!(first_output.status.success());

    let component_file = project.path.join("views/components/button.html");

    let original_content = fs::read_to_string(&component_file).unwrap();

    let second_output = project.run_make("components", "button");

    assert!(!second_output.status.success());

    let current_content = fs::read_to_string(component_file).unwrap();

    assert_eq!(current_content, original_content);
}
