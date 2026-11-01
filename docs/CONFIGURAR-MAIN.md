# Configurando o `main.rs` — RustPress Web

Este guia explica como configurar o ficheiro `src/main.rs` para iniciar uma aplicação web utilizando o framework **RustPress Web**, disponível no [crates.io](https://crates.io/crates/rustpress-web).

## 1. Criar o projecto

Se ainda não tens um projecto Rust, cria um utilizando Cargo:

```bash
cargo new minha_app
cd minha_app
```

## 2. Adicionar as dependências

Abre o ficheiro `Cargo.toml` e adiciona as seguintes dependências:

```toml
[dependencies]
rustpress-web = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

**Para que servem estas dependências?**

- `rustpress-web`: fornece o servidor HTTP, o sistema de rotas e os componentes principais do RustPress.
- `tokio`: fornece o runtime assíncrono necessário para executar o servidor.

## 3. Configurar o ficheiro main.rs

O ficheiro `src/main.rs` é o ponto de entrada da aplicação. É nele que inicializamos o framework, registamos as rotas e iniciamos o servidor.

Exemplo completo:

```rust
use rustpress_web::{App, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Criar a aplicação
    let mut app = App::new();

    // 2. Registar uma rota GET
    app.get("/", |_request: Request, response: Response| {
        Ok(response.html("<h1>Bem-vindo ao RustPress!</h1>"))
    })?;

    // 3. Iniciar o servidor
    println!("Servidor disponível em http://127.0.0.1:3000");

    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
```

### Compreendendo o código

**1. Importar os componentes**

```rust
use rustpress_web::{App, Request, Response};
```

- `App`: representa a aplicação e permite configurar o servidor e as rotas.
- `Request`: representa o pedido HTTP recebido.
- `Response`: permite construir a resposta enviada ao cliente.

**2. Criar a aplicação**

```rust
let mut app = App::new();
```

Cria uma nova instância da aplicação. A variável é mutável porque precisamos registar as rotas.

**3. Registar uma rota**

```rust
app.get("/", |_request: Request, response: Response| {
    Ok(response.html("<h1>Bem-vindo ao RustPress!</h1>"))
})?;
```

O método `get()` regista uma rota HTTP GET.

Neste exemplo:

- `/` representa o caminho da rota.
- `_request` representa o pedido recebido.
- `response` permite construir a resposta.
- `response.html()` produz uma resposta HTML.
- `Ok(...)` devolve o resultado esperado pelo handler.
- `?` permite propagar eventuais erros durante o registo da rota.

**4. Iniciar o servidor**

```rust
app.listen("127.0.0.1:3000").await?;
```

Inicia o servidor HTTP no endereço `127.0.0.1`, utilizando a porta `3000`.

O método é assíncrono, por isso precisa de `.await`.

## 4. Compreender o atributo tokio::main

```rust
#[tokio::main]
async fn main() {
    // Código da aplicação
}
```

O atributo `#[tokio::main]` prepara automaticamente o runtime assíncrono do Tokio.

É necessário porque o servidor RustPress utiliza operações assíncronas.

Sem esse runtime, não poderíamos executar directamente operações como:

```rust
app.listen("127.0.0.1:3000").await?;
```

## 5. Separar as rotas do main.rs

À medida que a aplicação cresce, é recomendável separar a configuração das rotas do ponto de entrada.

Por exemplo, cria a seguinte estrutura:

```text
minha_app/
├── Cargo.toml
└── src/
    ├── main.rs
    └── routes/
        ├── mod.rs
        └── web.rs
```

No ficheiro `src/routes/mod.rs`:

```rust
pub mod web;
```

No ficheiro `src/routes/web.rs`:

```rust
use rustpress_web::{App, Request, Response};

pub fn register(app: &mut App) -> Result<(), rustpress_web::HandlerError> {
    app.get("/", |_request: Request, response: Response| {
        Ok(response.html("<h1>Página inicial</h1>"))
    })?;

    app.get("/sobre", |_request: Request, response: Response| {
        Ok(response.html("<h1>Sobre nós</h1>"))
    })?;

    Ok(())
}
```

No ficheiro `src/main.rs`:

```rust
mod routes;

use rustpress_web::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = App::new();

    routes::web::register(&mut app)?;

    println!("RustPress running at http://127.0.0.1:3000");

    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
```

Desta forma, o `main.rs` fica responsável apenas por inicializar a aplicação, registar as rotas e iniciar o servidor.

## 6. Executar a aplicação

No terminal, executa:

```bash
cargo run
```

Se tudo estiver correctamente configurado, o servidor será iniciado e poderás aceder à aplicação através do navegador:

```text
http://127.0.0.1:3000
```

## 7. Resumo

O processo básico de inicialização de uma aplicação RustPress consiste em:

1. Adicionar `rustpress-web` e `tokio` ao `Cargo.toml`.
2. Importar os componentes necessários.
3. Criar uma instância de `App`.
4. Registar as rotas.
5. Iniciar o servidor com `listen().await`.

**Documentação oficial do pacote:**  
https://crates.io/crates/rustpress-web

**Repositório do projecto:**  
https://github.com/fabriciomichael/rustpress-web