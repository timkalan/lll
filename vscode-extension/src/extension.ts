import * as vscode from "vscode";
import { spawn } from "child_process";

const DIAGNOSTIC_RE =
  /^([^:]+):(\d+):(\d+):(\d+):(\d+):\s+(error|warning|suggestion):\s+(.+)$/;

const SEVERITY_MAP: Record<string, vscode.DiagnosticSeverity> = {
  error: vscode.DiagnosticSeverity.Error,
  warning: vscode.DiagnosticSeverity.Warning,
  suggestion: vscode.DiagnosticSeverity.Hint,
};

let diagnostics: vscode.DiagnosticCollection;
const inFlight = new Map<string, AbortController>();

export function activate(context: vscode.ExtensionContext) {
  diagnostics = vscode.languages.createDiagnosticCollection("lll");
  context.subscriptions.push(diagnostics);

  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((doc) => runLint(doc))
  );
  context.subscriptions.push(
    vscode.workspace.onDidCloseTextDocument((doc) => {
      diagnostics.delete(doc.uri);
      inFlight.get(doc.uri.toString())?.abort();
    })
  );
}

function runLint(doc: vscode.TextDocument) {
  const config = vscode.workspace.getConfiguration("lll");
  const enabled = config.get<string[]>("enabledLanguages") ?? [];
  if (!enabled.includes(doc.languageId)) return;

  const cmd = config.get<string>("command") ?? "lll";
  const file = doc.uri.fsPath;
  const key = doc.uri.toString();

  inFlight.get(key)?.abort();
  const controller = new AbortController();
  inFlight.set(key, controller);

  const proc = spawn(cmd, ["--format", "editor", file], {
    signal: controller.signal,
  });

  let stdout = "";
  let stderr = "";
  proc.stdout.on("data", (chunk) => (stdout += chunk.toString()));
  proc.stderr.on("data", (chunk) => (stderr += chunk.toString()));

  proc.on("close", () => {
    if (controller.signal.aborted) return;
    inFlight.delete(key);

    const diags: vscode.Diagnostic[] = [];
    for (const line of stdout.split("\n")) {
      const m = DIAGNOSTIC_RE.exec(line);
      if (!m) continue;
      const [, , startLine, startCol, endLine, endCol, severity, message] = m;
      // emit format is 1-based, end inclusive.
      // VSCode wants 0-based; end is exclusive but numerically equals 1-based-inclusive.
      const range = new vscode.Range(
        Number(startLine) - 1,
        Number(startCol) - 1,
        Number(endLine) - 1,
        Number(endCol)
      );
      const diag = new vscode.Diagnostic(
        range,
        message,
        SEVERITY_MAP[severity] ?? vscode.DiagnosticSeverity.Information
      );
      diag.source = "lll";
      diags.push(diag);
    }
    diagnostics.set(doc.uri, diags);
  });

  proc.on("error", (err) => {
    if ((err as NodeJS.ErrnoException).code === "ABORT_ERR") return;
    vscode.window.showErrorMessage(
      `lll failed: ${err.message}${stderr ? `\n${stderr}` : ""}`
    );
  });
}

export function deactivate() {
  diagnostics?.dispose();
  for (const controller of inFlight.values()) controller.abort();
}
