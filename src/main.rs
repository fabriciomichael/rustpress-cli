use clap::{Parser, Subcommand};
use rustpress_cli::generator;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rustpress")]
#[command(version, about = "Official RustPress CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new RustPress project.
    New {
        /// Name of the project to create.
        name: String,

        /// Directory where the project will be created.
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Build a RustPress project for distribution.
    Build {
        /// Directories to include in the final distribution.
        #[arg(long, value_delimiter = ',')]
        include: Vec<PathBuf>,

        /// Name of the Cargo binary target to build.
        #[arg(long)]
        bin: Option<String>,
    },

    /// Diagnose the Rust environment and current RustPress project.
    Doctor,

    /// Run the project in development mode with automatic restarts.
    Dev,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, path } => {
            if let Err(error) = generator::create_project_at(&name, &path) {
                eprintln!("Error: {}", error);
                std::process::exit(1);
            }
        }

        Commands::Build { include, bin } => {
            let project_path = std::env::current_dir().unwrap_or_else(|error| {
                eprintln!("Error getting current directory: {}", error);
                std::process::exit(1);
            });

            if let Err(error) =
                generator::build_project_with_binary(&project_path, &include, bin.as_deref())
            {
                eprintln!("Build error: {}", error);
                std::process::exit(1);
            }
        }

        Commands::Doctor => {
            rustpress_cli::doctor::run_diagnostics();
        }

        Commands::Dev => {
            let project_path = std::env::current_dir().unwrap_or_else(|error| {
                eprintln!("Error getting current directory: {}", error);
                std::process::exit(1);
            });

            if let Err(error) = rustpress_cli::dev::run_dev(&project_path) {
                eprintln!("Dev error: {}", error);
                std::process::exit(1);
            }
        }
    }
}
