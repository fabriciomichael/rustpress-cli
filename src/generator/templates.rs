pub const CARGO_TOML: &str = r#"
[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"

[dependencies]
rustpress-web = "0.2.0"
tokio = { version = "1", features = ["full"] }
dotenvy = "0.15"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
"#;

pub const SERVER_CONFIG_RS: &str = r#"
use std::env;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let host = env::var("RUSTPRESS_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("RUSTPRESS_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(3000);

        Self { host, port }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
"#;

pub const DATABASE_CONFIG_RS: &str = r#"
use std::env;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            driver: env::var("DB_DRIVER")
                .unwrap_or_else(|_| "mysql".to_string()),

            host: env::var("DB_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),

            port: env::var("DB_PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(3306),

            name: env::var("DB_NAME")
                .unwrap_or_else(|_| "rustpress_db".to_string()),

            user: env::var("DB_USER")
                .unwrap_or_else(|_| "root".to_string()),

            password: env::var("DB_PASSWORD")
                .unwrap_or_default(),
        }
    }

    pub fn connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user,
            self.password,
            self.host,
            self.port,
            self.name
        )
    }

    pub fn is_mysql(&self) -> bool {
        self.driver.eq_ignore_ascii_case("mysql")
    }
}
"#;

pub const CONFIG_MOD_RS: &str = r#"
pub mod server;
pub mod database;

pub use server::ServerConfig;
pub use database::DatabaseConfig;
"#;

pub const ENV_EXAMPLE: &str = r#"
# RustPress Server
RUSTPRESS_HOST=127.0.0.1
RUSTPRESS_PORT=3000

# Database
DB_DRIVER=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_NAME=rustpress_db
DB_USER=root
DB_PASSWORD=
"#;

pub const MAIN_RS: &str = r#"
mod routes;
mod controllers;
mod models;
mod middleware;
mod config;

use rustpress_web::App;
use config::DatabaseConfig;
use config::ServerConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();

    let server_config = ServerConfig::from_env();
    let address = server_config.address();

    let database_config = DatabaseConfig::from_env();
    let _is_mysql = database_config.is_mysql();
    let _database_url = database_config.connection_string();

    let mut app = App::new();

    routes::web::register(&mut app)?;

    println!("RustPress running at http://{}", address);

    app.listen(&address).await?;

    Ok(())
}
"#;

pub const README_MD: &str = r#"
# {{project_name}}

Web application built with RustPress.

## Requirements

- Rust
- Cargo

## Run the application

```bash
cargo run
```

After starting the server, open the following address in your browser:

http://127.0.0.1:3000

## Project structure

```text
{{project_name}}/
├── src/
│   ├── config/
│   ├── controllers/
│   ├── middleware/
│   ├── models/
│   ├── routes/
│   └── main.rs
├── views/
│   ├── layouts/
│   ├── pages/
│   └── partials/
├── public/
│   ├── css/
│   ├── images/
│   └── js/
├── tests/
├── Cargo.toml
└── README.md
```

## Technologies

- Rust
- RustPress Web
- Tokio
"#;

pub const HOME_HTML: &str = r#"
<section>
    <h2>Começa a desenvolver</h2>

    <p>A tua aplicação web está a funcionar!</p>

    <p>Esta página está localizada em views/pages/home.html.</p>

    <p>Desenvolve a tua aplicação com RustPress e Rust.</p>
</section>
"#;

pub const ROUTES_RS: &str = r#"
use rustpress_web::{App, Request, Response};

pub fn register(app: &mut App) -> Result<(), rustpress_web::HandlerError> {
    app.get("/", |_request: Request, _response: Response| {
        Ok(crate::controllers::home_controller::index())
    })?;

    Ok(())
}
"#;

pub const CONTROLLER_RS: &str = r#"
use rustpress_web::{Response, TemplateContext};

pub fn index() -> Response {
    let mut context = TemplateContext::new();

    context.set("titulo", "{{project_name}}".into());

    Response::new().render(
        "views/pages/home.html",
        "views/layouts/main.html",
        "views/partials",
        &context,
    )
}
"#;

pub const MODULE_RS: &str = r#"
pub mod web;
"#;

pub const CONTROLLERS_MOD_RS: &str = r#"
pub mod home_controller;
"#;

pub const EMPTY_MOD_RS: &str = r#"
// Module reserved for future implementations.
"#;

pub const STORAGE_README: &str = r#"
# Storage

Directory intended for storing files uploaded by users.

## Structure

- uploads/images: general images.
- uploads/documents: documents.
- uploads/receipts: payment receipts.
- uploads/profiles: profile pictures.
- uploads/products: product images and files.
- uploads/others: other files.
- temp: temporary files created during processing.

## Security

- Do not expose this directory directly through the HTTP server.
- Validate file types and sizes before storing them.
- Use filenames generated by the application.
- Control file access through routes and permissions.
- Regularly remove temporary files that are no longer needed.
"#;

pub const STORAGE_GITKEEP: &str = "";

pub const ERROR_400_HTML: &str = r#"
    <section>
        <h1>400</h1>
        <h2>Pedido inválido</h2>
        <p>O servidor não conseguiu compreender o pedido enviado.</p>
        <a href="/">Voltar à página inicial</a>
    </section>
"#;

pub const ERROR_401_HTML: &str = r#"
    <section>
        <h1>401</h1>
        <h2>Autenticação necessária</h2>
        <p>Precisas de iniciar sessão para aceder a este recurso.</p>
        <a href="/">Voltar à página inicial</a>
    </section>
"#;

pub const ERROR_403_HTML: &str = r#"
    <section>
        <h1>403</h1>
        <h2>Acesso proibido</h2>
        <p>Não tens autorização para aceder a esta página.</p>
        <a href="/">Voltar à página inicial</a>
    </section>
"#;

pub const ERROR_404_HTML: &str = r#"
    <section>
        <h1>404</h1>
        <h2>Página não encontrada</h2>
        <p>A página que procuras não existe ou foi movida.</p>
        <a href="/">Voltar à página inicial</a>
    </section>
"#;

pub const ERROR_405_HTML: &str = r#"
    <section>
        <h1>405</h1>
        <h2>Método não permitido</h2>
        <p>O método HTTP utilizado não é permitido neste recurso.</p>
        <a href="/">Voltar à página inicial</a>
    </section>
"#;

pub const ERROR_500_HTML: &str = r#"
    <section>
        <h1>500</h1>
        <h2>Erro interno do servidor</h2>
        <p>Ocorreu um erro inesperado ao processar o teu pedido.</p>
        <a href="/">Voltar à página inicial</a>
    </section>
"#;

pub const HEADER_PARTIAL_HTML: &str = r#"
<header>
    <h1>{{project_name}}</h1>
    <p>Bem-vindo à tua aplicação RustPress.</p>
</header>
"#;

pub const FOOTER_PARTIAL_HTML: &str = r#"
<footer>
    <p>&copy; {{project_name}}. Todos os direitos reservados.</p>
    <p>Desenvolvido com RustPress Web.</p>
</footer>
"#;

pub const NAVBAR_PARTIAL_HTML: &str = r#"
<nav>
    <a href="/">Início</a>
</nav>
"#;

pub const FLASH_PARTIAL_HTML: &str = r#"
<div class="flash-message">
    <!-- Área reservada para mensagens de sucesso, erro ou informação. -->
</div>
"#;

pub const LOADING_PARTIAL_HTML: &str = r#"
<div class="loading" role="status" aria-live="polite">
    <p>A carregar...</p>
</div>
"#;

pub const MAIN_LAYOUT_HTML: &str = r#"
<!DOCTYPE html>
<html lang="pt">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="Aplicação web desenvolvida com RustPress.">
    <title>{{titulo}}</title>
</head>
<body>
    {{> navbar}}
    {{> header}}

    <main>
        {{{conteudo}}}
    </main>

    {{> footer}}
</body>
</html>
"#;
