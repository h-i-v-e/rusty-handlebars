import * as fs from "node:fs";
import * as vscode from "vscode";
import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind
} from "vscode-languageclient/node";
import { documentSelector, serverPath } from "./configuration";

let client: LanguageClient | undefined;
const generatedDocuments = new Map<string, string>();

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const output = vscode.window.createOutputChannel("Rusty Handlebars", {
    log: true
  });
  context.subscriptions.push(output);

  const provider = vscode.workspace.registerTextDocumentContentProvider(
    "rusty-handlebars-generated",
    {
      provideTextDocumentContent(uri): string {
        return generatedDocuments.get(uri.toString()) ?? "// Generated source is unavailable.";
      }
    }
  );
  context.subscriptions.push(provider);

  context.subscriptions.push(
    vscode.commands.registerCommand("rustyHandlebars.showGeneratedRust", showGeneratedRust),
    vscode.commands.registerCommand("rustyHandlebars.restartServer", async () => {
      await stopClient();
      await startClient(context, output);
    }),
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (event.affectsConfiguration("rustyHandlebars")) {
        await stopClient();
        await startClient(context, output);
      }
    })
  );

  await startClient(context, output);
}

export async function deactivate(): Promise<void> {
  await stopClient();
}

async function startClient(
  context: vscode.ExtensionContext,
  output: vscode.LogOutputChannel
): Promise<void> {
  const command = serverPath(context);
  if (!fs.existsSync(command)) {
    const action = await vscode.window.showErrorMessage(
      `Rusty Handlebars language server was not found at ${command}.`,
      "Open Settings"
    );
    if (action === "Open Settings") {
      await vscode.commands.executeCommand(
        "workbench.action.openSettings",
        "rustyHandlebars.server.path"
      );
    }
    return;
  }

  const executable: Executable = {
    command,
    transport: TransportKind.stdio,
    options: { cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath }
  };
  const serverOptions: ServerOptions = {
    run: executable,
    debug: executable
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: documentSelector(),
    outputChannel: output,
    synchronize: {
      configurationSection: "rustyHandlebars",
      fileEvents: vscode.workspace.createFileSystemWatcher(
        "**/{Cargo.toml,Cargo.lock,*.rs}"
      )
    }
  };
  client = new LanguageClient(
    "rusty-handlebars",
    "Rusty Handlebars",
    serverOptions,
    clientOptions
  );
  await client.start();
}

async function stopClient(): Promise<void> {
  const active = client;
  client = undefined;
  if (active !== undefined) {
    await active.stop();
  }
}

async function showGeneratedRust(): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (editor === undefined || client === undefined) {
    void vscode.window.showInformationMessage(
      "Open a Rusty Handlebars template after the language server has started."
    );
    return;
  }

  try {
    const source = await client.sendRequest<string>(
      "rustyHandlebars/showGeneratedRust",
      { uri: editor.document.uri.toString() }
    );
    const uri = vscode.Uri.from({
      scheme: "rusty-handlebars-generated",
      path: `/${encodeURIComponent(editor.document.uri.path)}.rs`
    });
    generatedDocuments.set(uri.toString(), source);
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.languages.setTextDocumentLanguage(document, "rust");
    await vscode.window.showTextDocument(document, {
      preview: true,
      viewColumn: vscode.ViewColumn.Beside
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    void vscode.window.showErrorMessage(
      `Unable to generate Rust for this template: ${message}`
    );
  }
}
