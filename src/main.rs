use clap::{Parser, Subcommand};
use rustpress_cli::generator;

#[derive(Parser)]
#[command(name = "rustpress")]
#[command(version, about = "Official RustPress CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New { name: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => {
            if let Err(error) = generator::create_project(&name) {
                eprintln!("Error: {}", error);
            }
        }
    }
}
