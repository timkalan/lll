use futures::future::join_all;
use std::process::ExitCode;

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

    /// JSON array of diagnostics already reported by the editor's LSP/linters,
    /// so the model can avoid duplicating them. Intended for editor integrations.
    #[arg(long, short)]
    diagnostics: Option<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    let multi = args.files.len() > 1;
    let mut had_error = false;

    let diagnostics = args.diagnostics.as_deref();
    let jobs = args.files.iter().map(|file| async move {
        let code = tokio::fs::read_to_string(file)
            .await
            .with_context(|| format!("Could not read file: {file}"))?;
        let response = lint(&code, file, diagnostics).await?;
        Ok::<_, anyhow::Error>((file.clone(), code, response))
    });

    // TODO: concurrency limit this
    let results = join_all(jobs).await;

    for result in results {
        match result {
            Ok((file, code, response)) => match args.format {
                Format::Pretty => {
                    if multi {
                        println!("{}\n", file.bold().underline());
                    }
                    print_pretty(&response, &code);
                }
                Format::Editor => print_editor(&response, &file),
            },
            Err(e) => {
                eprintln!("{e:#}");
                had_error = true;
                continue;
            }
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
