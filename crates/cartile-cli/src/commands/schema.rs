use clap::Args;

#[derive(Args)]
pub struct SchemaArgs {
    /// Output file (default: stdout)
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

pub fn run(args: SchemaArgs) -> anyhow::Result<()> {
    let schema = cartile_format::generate_map_schema();
    if let Some(path) = args.output {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &schema)?;
        eprintln!("schema written to {}", path.display());
    } else {
        println!("{schema}");
    }
    Ok(())
}
