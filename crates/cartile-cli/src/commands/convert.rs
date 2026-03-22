use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ConvertArgs {
    /// Input Tiled JSON or LDtk JSON file
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output cartile file (default: input with .cartile extension)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Keep external tilesets as $ref instead of inlining (Tiled only)
    #[arg(long)]
    pub external_tilesets: bool,

    /// LDtk level name to convert (default: first level)
    #[arg(long)]
    pub level: Option<String>,
}

pub fn run(args: ConvertArgs) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(&args.input)?;

    // Auto-detect format
    if cartile_cli::ldtk::convert::is_ldtk_json(&json) {
        return run_ldtk(args, &json);
    }

    run_tiled(args, &json)
}

fn run_tiled(args: ConvertArgs, json: &str) -> anyhow::Result<()> {
    let tiled_map: cartile_cli::tiled::types::TiledMap = serde_json::from_str(json)?;

    if tiled_map.tiledversion.is_none() {
        anyhow::bail!("input does not appear to be a Tiled JSON file (missing 'tiledversion')");
    }

    let map_name = args
        .input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled");

    let input_dir = args.input.parent().unwrap_or(std::path::Path::new("."));

    let output = args
        .output
        .unwrap_or_else(|| args.input.with_extension("cartile"));
    let output_dir = if args.external_tilesets {
        Some(output.parent().unwrap_or(std::path::Path::new(".")))
    } else {
        None
    };

    let (map, warnings) = cartile_cli::tiled::convert::convert_tiled_map(
        &tiled_map, map_name, input_dir, output_dir,
    )?;

    for w in &warnings {
        eprintln!("warning: {w}");
    }

    if let Err(e) = map.validate() {
        anyhow::bail!("conversion produced invalid output: {e}");
    }

    map.to_file(&output)?;
    eprintln!("✓ converted to {}", output.display());

    Ok(())
}

fn run_ldtk(args: ConvertArgs, json: &str) -> anyhow::Result<()> {
    let root: cartile_cli::ldtk::types::LdtkRoot = serde_json::from_str(json)?;

    let (map, warnings) =
        cartile_cli::ldtk::convert::convert_ldtk_project(&root, args.level.as_deref())?;

    for w in &warnings {
        eprintln!("warning: {w}");
    }

    if let Err(e) = map.validate() {
        anyhow::bail!("conversion produced invalid output: {e}");
    }

    let output = args
        .output
        .unwrap_or_else(|| args.input.with_extension("cartile"));

    map.to_file(&output)?;
    eprintln!("✓ converted to {}", output.display());

    Ok(())
}
