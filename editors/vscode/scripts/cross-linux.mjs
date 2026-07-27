import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionDirectory = path.resolve(scriptDirectory, "..");
const repositoryRoot = path.resolve(extensionDirectory, "..", "..");
const packageManifest = JSON.parse(
  fs.readFileSync(path.join(extensionDirectory, "package.json"), "utf8")
);
const packageExtension = process.argv.includes("--package");
const requested = process.argv
  .slice(2)
  .find((argument) => !argument.startsWith("--")) ?? "all";

const targets = {
  x64: {
    rust: "x86_64-unknown-linux-gnu",
    vscode: "linux-x64"
  },
  arm64: {
    rust: "aarch64-unknown-linux-gnu",
    vscode: "linux-arm64"
  }
};

const selected = requested === "all"
  ? Object.values(targets)
  : [targets[requested]];

if (selected.some((target) => target === undefined)) {
  throw new Error("Target must be one of: x64, arm64, all");
}

if (packageExtension) {
  run("npm", ["run", "check"], extensionDirectory);
  run("npm", ["run", "compile"], extensionDirectory);
}

for (const target of selected) {
  const targetDirectory = path.join("target", "cross", target.rust);
  run(
    "cross",
    [
      "build",
      "--locked",
      "--release",
      "--package",
      "rusty-handlebars-language-server",
      "--target",
      target.rust
    ],
    repositoryRoot,
    {
      CARGO_TARGET_DIR: targetDirectory
    }
  );

  const binary = path.join(
    repositoryRoot,
    targetDirectory,
    target.rust,
    "release",
    "rusty-handlebars-language-server"
  );
  if (!fs.existsSync(binary)) {
    throw new Error(`Cross build completed without producing ${binary}`);
  }
  console.log(`Built ${target.vscode}: ${binary}`);

  if (!packageExtension) {
    continue;
  }

  const stagedServers = path.join(extensionDirectory, "server");
  fs.rmSync(stagedServers, { recursive: true, force: true });
  run(
    process.execPath,
    [
      path.join(scriptDirectory, "stage-server.mjs"),
      binary,
      target.vscode
    ],
    extensionDirectory
  );

  const output = `rusty-handlebars-${packageManifest.version}-${target.vscode}.vsix`;
  run(
    "npx",
    [
      "--no-install",
      "vsce",
      "package",
      "--no-dependencies",
      "--target",
      target.vscode,
      "--out",
      output
    ],
    extensionDirectory
  );
  console.log(`Packaged ${path.join(extensionDirectory, output)}`);
}

function run(command, arguments_, cwd, extraEnvironment = {}) {
  const result = spawnSync(command, arguments_, {
    cwd,
    env: {
      ...process.env,
      ...extraEnvironment
    },
    stdio: "inherit"
  });
  if (result.error !== undefined) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status}`);
  }
}
