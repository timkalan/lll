use rig::client::{CompletionClient, ProviderClient};
use rig::providers::openrouter;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    let code = r#"
  fn calculate_average(numbers: &[f64]) -> f64 {
      let mut sum = 0.0;
      for i in 0..numbers.len() {
          sum += numbers[i];
      }
      sum / numbers.len() as f64  // crashes on empty slice
  }
  "#;

    let client = openrouter::Client::from_env();

    let linter = client
        .extractor::<LintOutput>("deepseek/deepseek-v3.2")
        .preamble(
            "You are a code linter. Analyze the provided code and return diagnostics. \
             Each diagnostic must have: \
                - severity: one of Error, Warning, or Suggestion \
                - code_quote: the exact code snippet from the input that the diagnostic refers to \
                - message: a short explanation of the issue \
            Return your findings using the provided tool.",
        )
        .build();

    let response = linter.extract(code).await?;

    println!("{response:#?}");

    Ok(())
}
