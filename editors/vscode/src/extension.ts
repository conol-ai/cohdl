// RFC-019 (DR-025): the CoHDL VS Code extension.
//
// A THIN packaging layer over the already-Accepted `cohdl lsp` (RFC-014):
// this file adds zero diagnostic logic — it spawns the server and lets
// vscode-languageclient wire up RFC-014's four capabilities (diagnostics,
// hover, goto-definition, references). The spawn shape is identical to the
// doc snippet in docs/lsp.md; the only addition is the `cohdl.path` setting
// (replacing the snippet's hardcoded path) and a visible activation-failure
// notification (RFC-019 Failure modes: a missing binary must never be a
// silent no-op that looks like "the file has no errors").

import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext): void {
  const configuredPath =
    vscode.workspace.getConfiguration("cohdl").get<string>("path")?.trim() ||
    "cohdl";
  const cohdlPath = configuredPath.replace(/\$\{workspaceFolder\}/g, vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || "");

  const serverOptions: ServerOptions = {
    command: cohdlPath,
    args: ["lsp"],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "cohdl" }],
  };

  client = new LanguageClient("cohdl", "CoHDL", serverOptions, clientOptions);

  // A failure to spawn `cohdl lsp` (typo'd `cohdl.path`, binary not built,
  // not on PATH) must be a visible error, not a blank Problems panel.
  client.start().catch((err) => {
    void vscode.window.showErrorMessage(
      `CoHDL: could not start the language server \`${cohdlPath} lsp\` — ` +
        `${err instanceof Error ? err.message : String(err)}. ` +
        `Set \`cohdl.path\` to the built \`cohdl\` binary, or add it to PATH.`
    );
  });

  context.subscriptions.push({ dispose: () => void client?.stop() });
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
