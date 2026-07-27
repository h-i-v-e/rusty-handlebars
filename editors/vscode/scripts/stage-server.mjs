import * as fs from "node:fs";
import * as path from "node:path";

const target = process.argv[3] ?? `${process.platform}-${process.arch}`;
const executable = target.startsWith("win32-")
  ? "rusty-handlebars-language-server.exe"
  : "rusty-handlebars-language-server";
const source = process.argv[2] ?? path.resolve("..", "..", "target", "debug", executable);
const destinationDirectory = path.resolve("server", target);
const destination = path.join(destinationDirectory, executable);

if (!fs.existsSync(source)) {
  throw new Error(
    `Language server not found at ${source}. Run cargo build -p rusty-handlebars-language-server first.`
  );
}

fs.mkdirSync(destinationDirectory, { recursive: true });
fs.copyFileSync(source, destination);
if (process.platform !== "win32") {
  fs.chmodSync(destination, 0o755);
}
console.log(`Staged ${source} at ${destination}`);
