use anyhow::Context;
use clap::{Parser, ValueEnum};
use lll::{lint, print_editor, print_pretty};

#[derive(Clone, Debug, ValueEnum)]
enum Format {
    Pretty,
    Editor,
}

#[derive(Parser, Debug)]
#[command(name = "lll", version, about = "Large Language Lint")]
struct Args {
    /// File to lint
    file: String,

    /// Output format
    #[arg(long, short, value_enum, default_value_t = Format::Pretty)]
    format: Format,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let code = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Could not read file: {}", args.file))?;

    let response = lint(&code, &args.file).await?;

    match args.format {
        Format::Pretty => print_pretty(&response, &code),
        Format::Editor => print_editor(&response, &code, &args.file),
    }

    Ok(())
}
