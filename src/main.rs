use anyhow::Context;
use clap::Parser;
use rig::client::{CompletionClient, ProviderClient};
use rig::providers::gemini;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let code = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Could not read file: {}", args.file))?;

    let client = gemini::Client::from_env();

    let linter = client
        .extractor::<LintOutput>("gemini-3.1-flash-lite-preview")
        .preamble(
            "You are a code linter. Analyze the provided code and return diagnostics. \
             Each diagnostic must have: \
                - severity: one of Error, Warning, or Suggestion \
                - code_quote: the exact code snippet from the input that the diagnostic refers to \
                - message: a short explanation of the issue \
            Return your findings using the provided tool.",
        )
        .build();

    let response = linter.extract(&code).await?;

    println!("{response:#?}");

    Ok(())
}
