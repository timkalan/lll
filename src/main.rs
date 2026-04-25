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

/// Tries to resolve a code snippet to its 1-indexed (line, col),
/// returns `None` if no match.
///
/// ```
/// let (line, col) = resolve_snippet("a\nbc", "bc").unwrap();
/// assert_eq!((line, col), (2, 1));
/// ```
fn resolve_snippet(code: &str, snippet: &str) -> Option<(usize, usize)> {
    let offset = code.find(snippet)?;
    let before = &code[..offset];
    let line = before.matches('\n').count() + 1;
    let col = match before.rfind('\n') {
        Some(last_newline) => offset - last_newline,
        None => offset + 1,
    };
    Some((line, col))
}

fn print_diagnostics(lint_output: &LintOutput, code: &str) {
    let line_width = code.lines().count().to_string().len();

    for diagnostic in &lint_output.diagnostics {
        let severity = match diagnostic.severity {
            Severity::Error => "error".red().bold(),
            Severity::Warning => "warning".yellow().bold(),
            Severity::Suggestion => "suggestion".cyan().bold(),
        };

        let position = resolve_snippet(code, &diagnostic.code_quote);

        let quoted = diagnostic
            .code_quote
            .lines()
            .enumerate()
            .map(|(i, l)| match position {
                // Add line numbers, if available
                Some((line, _)) => {
                    let num = format!("{:>width$}", line + i, width = line_width).dimmed();
                    format!("{} {} {l}", num, "|".dimmed())
                }
                None => format!("{:width$} {} {l}", "", "|".dimmed(), width = line_width),
            })
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

    print_diagnostics(&response, &code);

    Ok(())
}
