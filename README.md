# RustPress CLI

**The official command-line interface for the RustPress web framework.**

RustPress CLI is a command-line tool written in Rust that simplifies the creation and distribution of web applications built with [RustPress Web](https://github.com/fabriciomichael/rustpress-web).

It automates project scaffolding, generates the initial application structure, and provides a production build command to prepare applications for distribution.

## Features

- Automatic RustPress project creation.
- Project directory and file generation.
- Initial application configuration.
- Organized routes and controllers.
- Models, middleware, and configuration modules.
- Project name validation.
- Protection against reserved Windows names.
- Production compilation in release mode.
- Selective inclusion of resource directories in the final distribution.
- Automatic organization of the executable and application resources.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- Cargo
- [RustPress Web](https://github.com/fabriciomichael/rustpress-web) (automatically added as a dependency to generated projects)

## Installation

Install RustPress CLI using Cargo:

```bash
cargo install rustpress-cli --version 0.2.0
```

After installation, the `rustpress` command will be available in your terminal.

## Usage

### Create a new project

```bash
rustpress new my_app
```

You can also specify a destination directory:

```bash
rustpress new my_app --path ./projects
```

### Run the application

Navigate to the generated project:

```bash
cd my_app
```

Start the development server:

```bash
cargo run
```

By default, the generated application listens on:

```text
http://127.0.0.1:3000
```


### Build a project for distribution

RustPress CLI provides a `build` command that compiles the application in release mode and prepares an organized distribution directory.

Run the command from the root directory of your RustPress application:

```bash
rustpress build
```

#### Including resource directories

The `--include` option accepts a comma-separated list of resource directories to include in the final distribution.

```bash
rustpress build --include views,public,storage
```

If `--include` is omitted, RustPress CLI automatically includes the existing `views`, `public`, and `storage` directories.

#### Selecting a binary target

RustPress CLI supports projects with multiple Cargo binary targets.

Use the `--bin` option to select a specific executable:

```bash
rustpress build --bin admin
```

When no binary is specified:

- If the project has only one binary target, it is selected automatically.
- If multiple binary targets exist and `default-run` is configured in `Cargo.toml`, that binary is selected automatically.
- If multiple binary targets exist without `default-run`, the command reports the available binaries and requires `--bin`.

You can configure the default executable in `Cargo.toml`:

```toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2024"
default-run = "admin"
```

An explicitly provided `--bin` option always takes precedence over `default-run`.

#### Build process

The command performs the following operations:

1. Validates the selected resource directories.
2. Resolves the requested or default Cargo binary target.
3. Compiles the application using `cargo build --release`.
4. Creates a distribution directory.
5. Copies the selected resource directories.
6. Copies the compiled executable into the distribution directory.

The resulting structure looks like this:

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

To run the distributed application on Windows:

```powershell
cd dist/my_app
.\my_app.exe
```

The executable extension depends on the target operating system.

### View available commands

```bash
rustpress --help
```

### View the installed version

```bash
rustpress --version
```

## Generated Project Structure

RustPress CLI generates an initial project structure similar to the following:

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
│   ├── layouts/
│   ├── pages/
│   │   └── home.html
│   └── partials/
├── public/
│   ├── css/
│   ├── js/
│   └── images/
├── storage/
└── tests/
```

## Technologies

- **Rust:** Systems programming language.
- **Clap:** Command-line argument and command parsing.
- **RustPress Web:** Web framework used by generated applications.

## Development

Clone the repository:

```bash
git clone https://github.com/fabriciomichael/rustpress-cli.git
```

Navigate to the project directory:

```bash
cd rustpress-cli
```

Run the tests:

```bash
cargo test
```

Check the project:

```bash
cargo check
```

Format the source code:

```bash
cargo fmt
```

Run the CLI during development:

```bash
cargo run -- new my_app
```

Install the local version:

```bash
cargo install --path . --force
```

## Related Projects

- [RustPress Web](https://github.com/fabriciomichael/rustpress-web)

## License

This project is distributed under the MIT License. See the [LICENSE](LICENSE) file for more information.

## Documentation

- [Configuring the main.rs File](docs/CONFIGURE-MAIN.md): Guide to configuring and starting a RustPress Web application.