# RustPress CLI — Command Reference

This reference describes the commands available in RustPress CLI 0.3.x. Run commands that operate on an existing application from that application's root directory, unless stated otherwise.

## `rustpress new`

Creates a new RustPress application.

```bash
rustpress new <project-name>
rustpress new <project-name> --path <destination>
```

| Argument / option | Description |
| --- | --- |
| `<project-name>` | Name of the application to create. |
| `-p`, `--path` | Parent directory where the project will be created. Defaults to the current directory. |

Example:

```bash
rustpress new school_portal
rustpress new school_portal --path ./projects
```

The generator validates project names, creates the source and resource directories, and writes the initial Cargo manifest, application entry point, routes, home controller, home view, default layout, partials, error pages, components directory, and storage guidance.

## `rustpress make`

Generates components and resources inside an existing RustPress project.

```bash
rustpress make <component> <name>
```

Supported component types:

| Component | Example | Generated result |
| --- | --- | --- |
| `controller` | `rustpress make controller UserController` | `src/controllers/user_controller.rs` and module registration. |
| `middleware` | `rustpress make middleware AuthMiddleware` | `src/middleware/auth_middleware.rs` and module registration. |
| `model` | `rustpress make model User` | `src/models/user.rs` and module registration. |
| `view` | `rustpress make view UserProfile` | `views/pages/user_profile.html`. |
| `layout` | `rustpress make layout main` | `views/layouts/main.html`. |
| `partial` | `rustpress make partial navbar` | `views/partials/navbar.html`. |
| `error` | `rustpress make error 404` | `views/errors/404.html`. |
| `components` | `rustpress make components button` | `views/components/button.html`. |
| `all` | `rustpress make all User` | A controller, middleware, model, and view for the supplied name. |
| `resource` | `rustpress make resource Product` | A controller, model, resource routes, four Tera HTML templates, and automatic module and route registration. |

### `make layout`

Generates an HTML layout template.

```bash
rustpress make layout main
```

Generated path:

```text
views/layouts/main.html
```

The default layout includes the HTML document structure, metadata, a title placeholder, and a content placeholder.

### `make partial`

Generates a reusable HTML partial.

```bash
rustpress make partial navbar
```

Generated path:

```text
views/partials/navbar.html
```

The generated partial can be used to organize reusable sections of application views.

### `make error`

Generates an HTTP error page for a supported status code.

```bash
rustpress make error 404
```

Supported HTTP status codes:

- `400` — Bad Request.
- `401` — Unauthorized.
- `403` — Forbidden.
- `404` — Not Found.
- `405` — Method Not Allowed.
- `500` — Internal Server Error.

Generated path:

```text
views/errors/404.html
```

Unsupported status codes are rejected.

### `make components`

Generates reusable HTML components.

```bash
rustpress make components button
```

Built-in component templates:

| Component | Description |
| --- | --- |
| `button` | HTML button element. |
| `input` | Form input with a label. |
| `card` | Card container with title and content. |
| `alert` | Alert message container. |
| `badge` | Small badge element. |

Custom component names are also supported:

```bash
rustpress make components ProductCard
```

Generated path:

```text
views/components/product_card.html
```

Component names are normalized to snake_case for file names. Existing component files are never overwritten.

### `make all`

Generates a controller, middleware, model, and view for the supplied name.

```bash
rustpress make all User
```

Generated paths:

```text
src/controllers/user_controller.rs
src/middleware/user_middleware.rs
src/models/user.rs
views/pages/user.html
```

The generator also updates the relevant `mod.rs` files for Rust modules. Names are normalized to snake_case for file and module names. Existing targets are not overwritten. The combined `all` operation validates targets first and attempts to roll back its generated changes if a later step fails.

### `make resource`

Generates a complete CRUD-oriented resource structure.

```bash
rustpress make resource Product
```

For `Product`, the generator creates:

```text
src/controllers/product_controller.rs
src/models/product.rs
src/routes/products.rs
views/products/index.html
views/products/show.html
views/products/create.html
views/products/edit.html
```

The generator also registers the Rust modules in their respective `mod.rs` files and adds the resource route registration to `src/routes/web.rs`.

The generated routes cover listing, displaying, creating, storing, editing, updating, and deleting resources.

**Note:** The generated HTML views use Tera templates. The `store`, `update`, and `destroy` controller handlers are placeholders that return HTTP 501. Database operations and persistence logic must be implemented by the application developer.

Existing files, module declarations, and route registrations are protected against accidental overwrites. The generator attempts to roll back its changes if generation fails.

## Tera Template Engine

Applications generated with `rustpress new` use [Tera](https://keats.github.io/tera/) as their template engine.

The CLI configures Tera centrally in:

```text
src/config/templates.rs
```

Templates are loaded from the following directory pattern:

```text
views/**/*.html
```

The template engine is initialized in `src/main.rs` before route registration:

```rust
config::templates::init()?;
```

### Rendering templates in controllers

Generated controllers use `tera::Context` to pass data to templates and the centralized template instance to render HTML:

```rust
use tera::Context;

let mut context = Context::new();
context.insert("titulo", "My Page");

let html = crate::config::templates::get()
    .render("pages/index.html", &context)?;

```

The rendered HTML can then be returned using `Response::html()`.

### Layout inheritance

Views can extend layouts using Tera inheritance:

```html
{% extends "layouts/main.html" %}

{% block content %}
<h1>{{ titulo }}</h1>
{% endblock content %}
```

### Reusable partials

Templates can include reusable partials:

```html
{% include "partials/navbar.html" %}
```

The generated layout already uses partials for shared page sections. Use `rustpress make layout`, `rustpress make partial`, and `rustpress make view` to generate additional template files.

## `rustpress dev`

Builds and runs the current application in development mode:

```bash
rustpress dev
```

The command watches the project recursively and restarts the application after relevant file changes. It ignores changes under `target`, `dist`, and `.git`, and groups bursts of filesystem events with a short debounce interval. If a rebuild fails, development mode stays active and waits for another file change.

Press `Ctrl+C` to stop the watcher and the child application process.

## `rustpress doctor`

Runs environment and project diagnostics:

```bash
rustpress doctor
```

Checks include:

- `rustc --version` and `cargo --version`.
- A `Cargo.toml` file in the current directory.
- A `rustpress-web` dependency in the manifest.
- The `src`, `views`, `public`, and `storage` directories.

The command reports successful checks, errors, and project warnings in the terminal.

## `rustpress routes`

Lists the routes registered in the current RustPress application.

```bash
rustpress routes
```

Run this command from the application root directory.

## `rustpress check`

Validates the structure and configuration of the current RustPress project.

```bash
rustpress check
```

The command checks:

- Whether `Cargo.toml` exists and contains valid TOML.
- Whether the manifest contains a `[package]` section.
- Whether the required project directories exist: `src`, `views`, `public`, and `storage`.
- Whether the required application files exist: `src/main.rs` and `src/routes`.
- Whether `rustpress-web` is listed under `[dependencies]`.

If the structural checks pass, the CLI runs `cargo check` to verify that the project compiles.

The command reports errors and returns a non-zero exit code when validation fails.

## `rustpress build`

Compiles the application in release mode and assembles a distribution directory:

```bash
rustpress build
```

### `--include`

Pass a comma-separated list of resource directories to copy:

```bash
rustpress build --include views,public,storage
```

Without this option, the CLI includes the existing directories among `views`, `public`, and `storage`. At least one resource directory must be available. Paths must be relative to the project root; parent-directory traversal and symbolic links are rejected by the copy operation.

### `--bin`

Select a Cargo binary target:

```bash
rustpress build --bin admin
```

If `--bin` is omitted, the CLI selects the only binary target, or uses the package's `default-run` when multiple targets exist. If multiple targets exist and no default is configured, pass `--bin` explicitly.

The resulting files are placed under:

```text
dist/<project-directory-name>/
```

The directory contains the compiled executable and the selected resource directories.

## Global options

```bash
rustpress --help
rustpress --version
```

Use `--help` after a command to inspect its arguments when supported by the CLI parser.