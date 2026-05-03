use anyhow::Context;
use colored::Colorize;
use rig::client::CompletionClient;
use rig::providers::gemini;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

const PROMPT: &str = "
    You are a code linter. Analyze the provided code and return diagnostics.
    Your goal is to catch things other linters might miss. Do not simply return what a generic
    linter for the given language might produce, but try to catch un-idiomatic behaviour or
    other general weirdness.";

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    gemini_api_key: Option<String>,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug)]
enum Severity {
    Error,
    Warning,
    Suggestion,
}

/// Represents the span of the diagnostic: where the
/// affected code begins and ends
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Span {
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
struct RawDiagnostic {
    severity: Severity,
    code_quote: String,
    message: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct RawLintOutput {
    diagnostics: Vec<RawDiagnostic>,
}

#[derive(Serialize, Debug)]
pub struct Diagnostic {
    severity: Severity,
    code_quote: String,
    message: String,
    span: Option<Span>,
}

#[derive(Serialize, Debug)]
pub struct LintOutput {
    pub diagnostics: Vec<Diagnostic>,
}

/// Tries to resolve a code snippet to its 1-based inclusive on both ends span
/// (which actual lines and columns it includes),
/// returns `None` if no match.
fn resolve_snippet(code: &str, snippet: &str) -> Option<Span> {
    let offset = code.find(snippet)?;
    let before = &code[..offset];
    let start_line = before.matches('\n').count() + 1;
    let start_column = match before.rfind('\n') {
        Some(last_newline) => offset - last_newline,
        None => offset + 1,
    };
    let end_line = start_line + snippet.matches('\n').count();
    let end_column = match snippet.rfind('\n') {
        Some(last_newline) => snippet.len() - last_newline - 1,
        None => snippet.len() + start_column - 1,
    };

    Some(Span {
        start_line,
        start_column,
        end_line,
        end_column,
    })
}

fn load_api_key() -> anyhow::Result<String> {
    if let Ok(key) = std::env::var("GEMINI_API_KEY")
        && !key.is_empty()
    {
        return Ok(key);
    }
    let home = std::env::var_os("HOME").context("HOME not set")?;
    let path = std::path::PathBuf::from(home).join(".config/lll/config.toml");
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("set GEMINI_API_KEY or create {}", path.display()))?;
    let config: Config =
        toml::from_str(&contents).with_context(|| format!("failed to parse {}", path.display()))?;
    config
        .gemini_api_key
        .filter(|k| !k.is_empty())
        .with_context(|| format!("missing `gemini_api_key` in {}", path.display()))
}

/// Prints the diagnostic messages in human-readable form.
pub fn print_pretty(lint_output: &LintOutput, code: &str) {
    let line_width = code.lines().count().to_string().len();

    for diagnostic in &lint_output.diagnostics {
        let severity = match diagnostic.severity {
            Severity::Error => "error".red().bold(),
            Severity::Warning => "warning".yellow().bold(),
            Severity::Suggestion => "suggestion".cyan().bold(),
        };

        let span = &diagnostic.span;

        let quoted = diagnostic
            .code_quote
            .lines()
            .enumerate()
            .map(|(i, l)| match &span {
                Some(position) => {
                    // 'dimmed' adds padding, we need to calculate first, then dim
                    let num =
                        format!("{:>width$}", position.start_line + i, width = line_width).dimmed();
                    format!("{} {} {l}", num, "|".dimmed())
                }
                None => format!("{:width$} {} {l}", "", "|".dimmed(), width = line_width),
            })
            .collect::<Vec<_>>()
            .join("\n");

        println!("[{}] {}\n{}\n", severity, diagnostic.message, quoted);
    }
}

/// Prints the diagnostic messages in 'file:line:col: severity: message' format.
pub fn print_editor(lint_output: &LintOutput, file: &str) {
    for diagnostic in &lint_output.diagnostics {
        let severity = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Suggestion => "suggestion",
        };

        let Some(span) = &diagnostic.span else {
            continue;
        };

        println!(
            "{file}:{}:{}:{}:{}: {severity}: {}",
            span.start_line, span.start_column, span.end_line, span.end_column, diagnostic.message
        )
    }
}

pub async fn lint(code: &str, file: &str) -> anyhow::Result<LintOutput> {
    let client = gemini::Client::new(load_api_key()?)?;

    let linter = client
        .extractor::<RawLintOutput>("gemini-3.1-flash-lite-preview")
        .preamble(PROMPT)
        .additional_params(json!({
            "generationConfig": {
                "temperature": 0
            }
        }))
        .build();

    let raw = linter
        .extract(format!("File: {}\n\n{}", file, code))
        .await?;

    let diagnostics = raw
        .diagnostics
        .into_iter()
        .map(|diagnostic| Diagnostic {
            span: resolve_snippet(code, &diagnostic.code_quote),
            severity: diagnostic.severity,
            code_quote: diagnostic.code_quote,
            message: diagnostic.message,
        })
        .collect();
    Ok(LintOutput { diagnostics })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line_snippet_on_line_2() {
        let span = resolve_snippet("a\nbc", "bc").unwrap();
        assert_eq!(
            span,
            Span {
                start_line: 2,
                start_column: 1,
                end_line: 2,
                end_column: 2
            }
        );
    }

    #[test]
    fn multi_line_snippet() {
        let span = resolve_snippet("xxx\nab\ncd", "ab\ncd").unwrap();
        assert_eq!(
            span,
            Span {
                start_line: 2,
                start_column: 1,
                end_line: 3,
                end_column: 2
            }
        );
    }

    #[test]
    fn hello_snippet() {
        let span = resolve_snippet("hello", "ell").unwrap();
        assert_eq!(
            span,
            Span {
                start_line: 1,
                start_column: 2,
                end_line: 1,
                end_column: 4,
            }
        );
    }
    #[test]
    fn not_found_snippet() {
        assert_eq!(resolve_snippet("foo", "bar"), None)
    }
}
