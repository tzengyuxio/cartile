use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "cartile", version, about = "Universal tilemap toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a Tiled JSON map to cartile format
    Convert(commands::convert::ConvertArgs),
    /// Export a cartile map to another format
    Export(commands::export::ExportArgs),
    /// Output the JSON Schema for the cartile map format
    Schema(commands::schema::SchemaArgs),
    /// Validate a cartile map file
    Validate(commands::validate::ValidateArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Convert(args) => commands::convert::run(args),
        Commands::Export(args) => commands::export::run(args),
        Commands::Schema(args) => commands::schema::run(args),
        Commands::Validate(args) => commands::validate::run(args),
    }
}
