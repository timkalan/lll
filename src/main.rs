use anyhow::Context;
use clap::Parser;
use lll::{lint, print_diagnostics};

#[derive(Parser, Debug)]
#[command(name = "lll", version, about = "Large Language Lint")]
struct Args {
    /// File to lint
    file: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let code = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Could not read file: {}", args.file))?;

    let response = lint(&code, &args.file).await?;

    print_diagnostics(&response, &code);

    Ok(())
}
