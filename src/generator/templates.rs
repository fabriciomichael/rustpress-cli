pub const CARGO_TOML: &str = r#"
[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"

[dependencies]
rustpress-web = "0.2.0"
tokio = { version = "1", features = ["full"] }

[workspace]
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

## Requisitos

- Rust
- Cargo

## Executar a aplicação

```bash
cargo run
```

Depois de iniciar o servidor, acede a:

http://127.0.0.1:3000

## Estrutura do projecto

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

## Tecnologias

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
// Módulo reservado para futuras implementações.
"#;

pub const STORAGE_README: &str = r#"
# Storage

Directório destinado ao armazenamento de ficheiros carregados pelos utilizadores.

## Estrutura

- uploads/images: imagens gerais.
- uploads/documents: documentos.
- uploads/receipts: comprovativos de pagamentos.
- uploads/profiles: fotografias de perfil.
- uploads/products: imagens e ficheiros de produtos.
- uploads/others: outros ficheiros.
- temp: ficheiros temporários durante o processamento.

## Segurança

- Não disponibilizes directamente este directório através do servidor HTTP.
- Valida o tipo e o tamanho dos ficheiros antes de os guardar.
- Utiliza nomes de ficheiros gerados pela aplicação.
- Controla o acesso aos ficheiros através das rotas e permissões.
- Remove periodicamente ficheiros temporários que já não sejam necessários.
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
