pub const CARGO_TOML: &str = r#"
[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"

[dependencies]
rustpress-web = "0.1.0"
tokio = { version = "1", features = ["full"] }
"#;

pub const MAIN_RS: &str = r#"
mod routes;
mod controllers;
mod models;
mod middleware;
mod config;

use rustpress_web::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = App::new();

    routes::web::register(&mut app)?;

    println!("RustPress running at http://127.0.0.1:3000");

    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
"#;

pub const README_MD: &str = r#"
# {{project_name}}

Aplicação web desenvolvida com RustPress.

## Executar

```bash
cargo run

"#;

pub const HOME_HTML: &str = r#"

"#;

pub const ROUTES_RS: &str = r#"
use rustpress_web::{App, Request, Response};

pub fn register(app: &mut App) -> Result<(), rustpress_web::HandlerError> {
    app.get("/", |_request: Request, response: Response| {
        Ok(response.html("<h1>RustPress está a funcionar!</h1>"))
    })?;

    Ok(())
}
"#;

pub const CONTROLLER_RS: &str = r#"
// Controllers da aplicação.
"#;

pub const MODULE_RS: &str = r#"
pub mod web;
"#;

pub const CONTROLLERS_MOD_RS: &str = r#"
pub mod home_controller;
"#;

pub const EMPTY_MOD_RS: &str = r#"
// Módulo reservado para futuras implementações.
"#;
