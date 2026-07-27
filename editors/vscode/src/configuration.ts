import * as path from "node:path";
import * as vscode from "vscode";

export function serverPath(context: vscode.ExtensionContext): string {
  const configured = vscode.workspace
    .getConfiguration("rustyHandlebars")
    .get<string>("server.path", "")
    .trim();
  if (configured.length > 0) {
    return configured;
  }

  const executable = process.platform === "win32"
    ? "rusty-handlebars-language-server.exe"
    : "rusty-handlebars-language-server";
  const platform = `${process.platform}-${process.arch}`;
  return context.asAbsolutePath(path.join("server", platform, executable));
}

export function documentSelector(): Array<{
  scheme: string;
  language?: string;
  pattern?: string;
}> {
  const globs = vscode.workspace
    .getConfiguration("rustyHandlebars")
    .get<string[]>("legacyFileGlobs", []);
  const selector = [
    { scheme: "file", language: "rusty-handlebars" },
    { scheme: "untitled", language: "rusty-handlebars" },
    ...globs.map((pattern) => ({ scheme: "file", pattern }))
  ];
  return selector;
}
