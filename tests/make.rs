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
        fs::create_dir_all(path.join("src/middleware")).unwrap();
        fs::create_dir_all(path.join("views")).unwrap();

        fs::write(
            path.join("Cargo.toml"),
            "[package]\nname = \"integration-test\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();

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

    assert_eq!(source, "<h1>{{titulo}}</h1>\n");

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
