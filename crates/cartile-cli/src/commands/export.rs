use clap::Args;
use std::path::PathBuf;

use cartile_format::CartileMap;

#[derive(Args)]
pub struct ExportArgs {
    /// Input cartile map file
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output file (default: input with target format extension)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Target format (currently only "tiled-json")
    #[arg(long, value_name = "FORMAT")]
    pub to: String,
}

pub fn run(args: ExportArgs) -> anyhow::Result<()> {
    match args.to.as_str() {
        "tiled-json" => run_tiled_json(args),
        other => anyhow::bail!("unsupported export format: '{other}' (supported: tiled-json)"),
    }
}

fn run_tiled_json(args: ExportArgs) -> anyhow::Result<()> {
    let map = CartileMap::from_file(&args.input)?;

    let (tiled_map, warnings) = cartile_cli::tiled::export::export_to_tiled(&map)?;

    for w in &warnings {
        eprintln!("warning: {w}");
    }

    let output = args
        .output
        .unwrap_or_else(|| args.input.with_extension("json"));

    let file = std::fs::File::create(&output)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &tiled_map)?;

    eprintln!("✓ exported to {}", output.display());
    Ok(())
}
