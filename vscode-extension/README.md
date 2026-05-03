# lll VSCode extension

Wraps the `lll` CLI as a VSCode linter. Runs on save, populates the Problems
panel and inline squigglies via `vscode.DiagnosticCollection`.

## Prereqs (per user)

- `lll` binary on PATH (build from the parent repo: `cargo install --path ..`)
- A Gemini API key, either as `GEMINI_API_KEY` in the environment VSCode
  launches from, or written to `~/.config/lll/config.toml` as
  `gemini_api_key = "..."`. The config file is recommended on macOS — env
  vars set in `~/.zshrc` don't propagate to VSCode launched from Spotlight.

## Build and package

```bash
npm install
npm run compile
npx @vscode/vsce package
```

Produces `lll-0.0.1.vsix`.

## Install (per user)

```bash
code --install-extension lll-0.0.1.vsix
```

## Configuration

Settings (in VSCode `settings.json`):

- `lll.command` — path to the `lll` binary (default: `"lll"`)
- `lll.enabledLanguages` — array of language IDs to lint on save
  (default: TS/JS/Rust/Python/Go)
