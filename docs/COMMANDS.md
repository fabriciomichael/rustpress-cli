# RustPress CLI — Command Reference

This reference describes the commands available in RustPress CLI 0.2.x. Run commands that operate on an existing application from that application's root directory, unless stated otherwise.

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

The generator validates project names, creates the source and resource directories, and writes the initial Cargo manifest, application entry point, routes, home controller, home view, default layout, partials, error pages, and storage guidance.

## `rustpress make`

Generates a component inside an existing RustPress project.

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
| `all` | `rustpress make all User` | A controller, middleware, model, and view for the supplied name. |

For `make all User`, the generated paths are:

```text
src/controllers/user_controller.rs
src/middleware/user_middleware.rs
src/models/user.rs
views/pages/user.html
```

The generator also updates the relevant `mod.rs` files for Rust modules. Names are normalized to snake_case for file and module names. Existing targets are not overwritten. The combined `all` operation validates targets first and attempts to roll back its generated changes if a later step fails.

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
