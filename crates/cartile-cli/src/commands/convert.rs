use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ConvertArgs {
    /// Input Tiled JSON file
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output cartile file (default: input with .cartile extension)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Keep external tilesets as $ref instead of inlining
    #[arg(long)]
    pub external_tilesets: bool,
}

pub fn run(_args: ConvertArgs) -> anyhow::Result<()> {
    anyhow::bail!("convert not yet implemented")
}
