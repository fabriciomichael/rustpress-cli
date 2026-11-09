# Configuring `main.rs` — RustPress Web

This guide explains how to configure the `src/main.rs` file to start a web application using the **RustPress Web** framework, available on [crates.io](https://crates.io/crates/rustpress-web).

## 1. Create the project

If you do not have a Rust project yet, create one using Cargo:

```bash
cargo new my_app
cd my_app
```

## 2. Add the dependencies

Open the `Cargo.toml` file and add the following dependencies:

```toml
[dependencies]
rustpress-web = "0.2.0"
tokio = { version = "1", features = ["full"] }
```

**What are these dependencies used for?**

- `rustpress-web`: provides the HTTP server, routing system, and core RustPress components.
- `tokio`: provides the asynchronous runtime required to run the server.

## 3. Configure the main.rs file

The `src/main.rs` file is the application's entry point. This is where we initialize the framework, register routes, and start the server.

Complete example:

```rust
use rustpress_web::{App, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Create the application
    let mut app = App::new();

    // 2. Register a GET route
    app.get("/", |_request: Request, response: Response| {
        Ok(response.html("<h1>Welcome to RustPress!</h1>"))
    })?;

    // 3. Start the server
    println!("Server available at http://127.0.0.1:3000");

    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
```

### Understanding the code

**1. Import the components**

```rust
use rustpress_web::{App, Request, Response};
```

- `App`: represents the application and allows you to configure the server and its routes.
- `Request`: represents an incoming HTTP request.
- `Response`: allows you to build the response sent to the client.

**2. Create the application**

```rust
let mut app = App::new();
```

Creates a new application instance. The variable is mutable because we need to register routes.

**3. Register a route**

```rust
app.get("/", |_request: Request, response: Response| {
    Ok(response.html("<h1>Welcome to RustPress!</h1>"))
})?;
```

The `get()` method registers an HTTP GET route.

In this example:

- `/` represents the route path.
- `_request` represents the incoming request.
- `response` allows you to build the response.
- `response.html()` creates an HTML response.
- `Ok(...)` returns the result expected by the handler.
- `?` propagates any errors that may occur while registering the route.

**4. Start the server**

```rust
app.listen("127.0.0.1:3000").await?;
```

Starts the HTTP server at the `127.0.0.1` address, using port `3000`.

This method is asynchronous, so it requires `.await`.

## 4. Understanding the tokio::main attribute

```rust
#[tokio::main]
async fn main() {
    // Application code
}
```

The `#[tokio::main]` attribute automatically sets up the Tokio asynchronous runtime.

It is required because the RustPress server uses asynchronous operations.

Without this runtime, we would not be able to execute operations such as:

```rust
app.listen("127.0.0.1:3000").await?;
```

## 5. Separating routes from main.rs

As the application grows, it is recommended to separate route configuration from the entry point.

For example, create the following structure:

```text
my_app/
├── Cargo.toml
└── src/
    ├── main.rs
    └── routes/
        ├── mod.rs
        └── web.rs
```

In `src/routes/mod.rs`:

```rust
pub mod web;
```

In `src/routes/web.rs`:

```rust
use rustpress_web::{App, Request, Response};

pub fn register(app: &mut App) -> Result<(), rustpress_web::HandlerError> {
    app.get("/", |_request: Request, response: Response| {
        Ok(response.html("<h1>Home page</h1>"))
    })?;

    app.get("/about", |_request: Request, response: Response| {
        Ok(response.html("<h1>About us</h1>"))
    })?;

    Ok(())
}
```

In `src/main.rs`:

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

This way, `main.rs` is responsible only for initializing the application, registering the routes, and starting the server.

## 6. Run the application

In the terminal, execute:

```bash
cargo run
```

If everything is configured correctly, the server will start and you can access the application through your browser:

```text
http://127.0.0.1:3000
```

## 7. Summary

The basic process for starting a RustPress application consists of:

1. Adding `rustpress-web` and `tokio` to `Cargo.toml`.
2. Importing the required components.
3. Creating an `App` instance.
4. Registering the routes.
5. Starting the server with `listen().await`.

**Official package documentation:**
https://crates.io/crates/rustpress-web

**Project repository:**
https://github.com/fabriciomichael/rustpress-web