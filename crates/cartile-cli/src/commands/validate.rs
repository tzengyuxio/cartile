use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ValidateArgs {
    /// Path to .cartile file to validate
    pub file: PathBuf,

    /// Only set exit code, suppress output
    #[arg(long)]
    pub quiet: bool,
}

pub fn run(args: ValidateArgs) -> anyhow::Result<()> {
    let map = cartile_format::CartileMap::from_file(&args.file)?;
    match map.validate() {
        Ok(()) => {
            if !args.quiet {
                eprintln!("✓ {} is valid", args.file.display());
            }
            Ok(())
        }
        Err(e) => {
            if !args.quiet {
                eprintln!("✗ {}: {e}", args.file.display());
            }
            std::process::exit(1);
        }
    }
}
