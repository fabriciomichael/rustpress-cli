<div align="center">
  <img src="assets/rustpress-cli-logo.svg" alt="RustPress CLI" width="720">
  <br>
  <strong>The official command-line interface for the RustPress web framework.</strong>
  <br><br>
  <a href="https://crates.io/crates/rustpress-cli">crates.io</a> ·
  <a href="https://github.com/fabriciomichael/rustpress-cli">GitHub</a> ·
  <a href="https://github.com/fabriciomichael/rustpress-web">RustPress Web</a>
</div>

---

RustPress CLI is a command-line tool written in Rust that helps developers create, run, diagnose, and prepare applications built with [RustPress Web](https://github.com/fabriciomichael/rustpress-web).

It provides project scaffolding, component generators, a development watcher with automatic restarts, environment diagnostics, and production distribution builds.

## Features

- Create a new RustPress application with a ready-to-use project structure.
- Generate controllers, middleware, models, and HTML views.
- Generate a complete set of components with one `make all` command.
- Run applications in development mode and restart them after relevant file changes.
- Diagnose the Rust toolchain and the current RustPress project.
- Compile applications in release mode and assemble a distribution directory.
- Select the binary target and resource directories included in a distribution.
- Validate project names and prevent accidental overwrites during component generation.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) and Cargo.
- A RustPress Web project for commands that operate on an existing application.
- The `rustpress-web` dependency is added automatically to projects created with `rustpress new`.

## Installation

Install the published CLI with Cargo:

```bash
cargo install rustpress-cli
```

To install a specific version:

```bash
cargo install rustpress-cli --version 0.2.0
```

Confirm the installation:

```bash
rustpress --version
rustpress --help
```

## Quick start

Create an application:

```bash
rustpress new my_app
cd my_app
```

Start it in development mode:

```bash
rustpress dev
```

The generated application listens at `http://127.0.0.1:3000` by default. Open that address in your browser.

## Commands

| Command | Description |
| --- | --- |
| `rustpress new <name>` | Create a new RustPress project. |
| `rustpress build` | Build and prepare a production distribution. |
| `rustpress dev` | Build, run, watch files, and restart the application after changes. |
| `rustpress doctor` | Diagnose Rust, Cargo, and the current project. |
| `rustpress make <component> <name>` | Generate a project component. |
| `rustpress --help` | Display CLI help. |
| `rustpress --version` | Display the installed version. |

See the [CLI command reference](docs/COMMANDS.md) for options, examples, and generated file paths.

## Create a project

```bash
rustpress new my_app
```

Choose a destination directory with `--path`:

```bash
rustpress new my_app --path ./projects
```

Then enter the generated directory and start the application:

```bash
cd my_app
rustpress dev
```

## Generate components

Run `make` from the root of a generated RustPress project.

```bash
rustpress make controller UserController
rustpress make middleware AuthMiddleware
rustpress make model User
rustpress make view UserProfile
```

Generate a controller, middleware, model, and view together:

```bash
rustpress make all User
```

The combined generator creates the corresponding files and registers the Rust modules where applicable. It validates all targets before writing and rolls back generated files if the operation fails.

## Development mode

```bash
rustpress dev
```

The development command builds and starts the application, watches the project recursively, and restarts the process when relevant files change. Changes under `target`, `dist`, and `.git` are ignored. If a rebuild fails, the watcher remains active and waits for another change. Press `Ctrl+C` to stop the development process.

## Diagnose a project

Run this command from the project root:

```bash
rustpress doctor
```

The diagnostic checks include:

- Availability and versions of `rustc` and Cargo.
- Presence of `Cargo.toml`.
- Presence of the `rustpress-web` dependency.
- Essential project directories: `src`, `views`, `public`, and `storage`.

Warnings about missing project components do not necessarily mean the Rust toolchain itself is broken.

## Build for distribution

Run from the root of the application:

```bash
rustpress build
```

When `--include` is omitted, the CLI includes whichever of these directories exist: `views`, `public`, and `storage`. Specify a comma-separated list to choose resource directories explicitly:

```bash
rustpress build --include views,public,storage
```

If the project defines multiple Cargo binary targets, select one with `--bin`:

```bash
rustpress build --bin admin
```

Binary selection follows these rules:

1. An explicit `--bin` value takes precedence.
2. If there is one binary target, it is selected automatically.
3. If there are multiple targets and `default-run` is set in `Cargo.toml`, that target is selected.
4. If multiple targets exist without `default-run`, specify `--bin`.

Example `Cargo.toml` configuration:

```toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2024"
default-run = "admin"
```

The CLI compiles with `cargo build --release`, then copies the selected executable and resource directories into `dist/<project-directory-name>/`.

Example output:

```text
my_app/
├── Cargo.toml
├── src/
├── views/
├── public/
└── dist/
    └── my_app/
        ├── my_app.exe
        ├── views/
        └── public/
```

The executable extension depends on the target operating system. The build command prepares the application files; it does not create an installer.

## Generated project structure

A newly generated project includes a structure similar to this:

```text
my_app/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── routes/
│   │   ├── mod.rs
│   │   └── web.rs
│   ├── controllers/
│   │   ├── mod.rs
│   │   └── home_controller.rs
│   ├── models/
│   │   └── mod.rs
│   ├── middleware/
│   │   └── mod.rs
│   └── config/
│       └── mod.rs
├── views/
│   ├── errors/
│   ├── layouts/
│   ├── pages/
│   └── partials/
├── public/
│   ├── css/
│   ├── images/
│   └── js/
├── storage/
└── tests/
```

## Documentation

- [CLI command reference](docs/COMMANDS.md)
- [Configuring `main.rs` for RustPress Web](docs/CONFIGURE-MAIN.md)

## Development

Clone the repository:

```bash
git clone https://github.com/fabriciomichael/rustpress-cli.git
cd rustpress-cli
```

Run the test suite and checks:

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Run the CLI locally:

```bash
cargo run -- --help
cargo run -- new my_app
```

Install the local checkout:

```bash
cargo install --path . --force
```

## Related project

- [RustPress Web](https://github.com/fabriciomichael/rustpress-web)

## License

RustPress CLI is distributed under the MIT License. See [LICENSE](LICENSE) for details.
