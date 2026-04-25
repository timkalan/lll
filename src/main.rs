use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use rig::client::{CompletionClient, ProviderClient};
use rig::providers::gemini;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

const PROMPT: &str = "
    You are a code linter. Analyze the provided code and return diagnostics.
    Your goal is to catch things other linters might miss. Do not simply return what a generic
    linter for the given language might produce, but try to catch un-idiomatic behaviour or
    other general weirdness.";

#[derive(Parser, Debug)]
#[command(name = "lll", version, about = "Large Language Lint")]
struct Args {
    /// File to lint
    file: String,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug)]
enum Severity {
    Error,
    Warning,
    Suggestion,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug)]
struct Diagnostic {
    severity: Severity,
    code_quote: String,
    message: String,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug)]
struct LintOutput {
    diagnostics: Vec<Diagnostic>,
}

fn print_diagnostics(lint_output: &LintOutput) {
    for diagnostic in &lint_output.diagnostics {
        let severity = match diagnostic.severity {
            Severity::Error => "error".red().bold(),
            Severity::Warning => "warning".yellow().bold(),
            Severity::Suggestion => "suggestion".cyan().bold(),
        };

        let quoted = diagnostic
            .code_quote
            .lines()
            .map(|l| format!("  {} {l}", "|".dimmed()))
            .collect::<Vec<_>>()
            .join("\n");

        println!("[{}] {}\n{}\n", severity, diagnostic.message, quoted);
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let code = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Could not read file: {}", args.file))?;

    let client = gemini::Client::from_env();

    let linter = client
        .extractor::<LintOutput>("gemini-3.1-flash-lite-preview")
        .preamble(PROMPT)
        .additional_params(json!({
            "generationConfig": {
                "temperature": 0
            }
        }))
        .build();

    let response = linter
        .extract(format!("File: {}\n\n{}", args.file, code))
        .await?;

    print_diagnostics(&response);

    Ok(())
}
