import * as vscode from "vscode";
import * as cp from "child_process";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

function findOnPath(name: string): string | undefined {
  try {
    const cmd = process.platform === "win32" ? "where" : "which";
    return cp.execFileSync(cmd, [name], { encoding: "utf-8" }).trim();
  } catch {
    return undefined;
  }
}

function findServerBinary(): string | undefined {
  const configured = vscode.workspace
    .getConfiguration("cohdl")
    .get<string>("serverPath");

  if (configured) {
    return configured;
  }

  return findOnPath("cohdl-lsp");
}

async function startClient(
  context: vscode.ExtensionContext
): Promise<void> {
  const serverPath = findServerBinary();

  if (!serverPath) {
    vscode.window.showErrorMessage(
      "Could not find the cohdl-lsp binary. " +
        "Set cohdl.serverPath in settings or ensure cohdl-lsp is on your PATH."
    );
    return;
  }

  const serverOptions: ServerOptions = {
    command: serverPath,
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "cohdl" }],
    synchronize: {
      fileEvents: [
        vscode.workspace.createFileSystemWatcher("**/*.cohdl"),
        vscode.workspace.createFileSystemWatcher("**/cohdl.toml"),
      ],
    },
  };

  client = new LanguageClient(
    "cohdl-lsp",
    "cohdl Language Server",
    serverOptions,
    clientOptions
  );

  await client.start();
}

export async function activate(
  context: vscode.ExtensionContext
): Promise<void> {
  await startClient(context);

  context.subscriptions.push(
    vscode.commands.registerCommand("cohdl.restartServer", async () => {
      if (client) {
        await client.stop();
        client = undefined;
      }
      await startClient(context);
    })
  );
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
  }
}
