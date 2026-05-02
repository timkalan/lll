use anyhow::Context;
use clap::{Parser, ValueEnum};
use colored::Colorize;
use lll::{lint, print_editor, print_pretty};

#[derive(Clone, Debug, ValueEnum)]
enum Format {
    Pretty,
    Editor,
}

#[derive(Parser, Debug)]
#[command(name = "lll", version, about = "Large Language Lint")]
struct Args {
    /// Files to lint
    #[arg(num_args = 1..)]
    files: Vec<String>,

    /// Output format
    #[arg(long, short, value_enum, default_value_t = Format::Pretty)]
    format: Format,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let multi = args.files.len() > 1;

    for file in &args.files {
        let code = match std::fs::read_to_string(file)
            .with_context(|| format!("Could not read file: {}", file))
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{e:#}");
                continue;
            }
        };

        let response = match lint(&code, file).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{file}: {e:#}");
                continue;
            }
        };

        match args.format {
            Format::Pretty => {
                if multi {
                    println!("{}\n", file.bold().underline());
                }
                print_pretty(&response, &code);
            }
            Format::Editor => print_editor(&response, file),
        }
    }

    Ok(())
}
