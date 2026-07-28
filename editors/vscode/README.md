# Rusty Handlebars for VS Code

This extension provides syntax highlighting, snippets, diagnostics, completion,
hover information, symbols, folding, selection ranges, matching-block
highlights, signature help, and generated Rust inspection for Rusty Handlebars
templates.

`.rhbs` files activate the extension automatically. The compiler still accepts
`.hbs`; associate legacy files explicitly with the `rusty-handlebars` language
or add project-specific patterns to `rustyHandlebars.legacyFileGlobs`.
A rusty side-profile bicycle icon distinguishes `.rhbs` files in the Explorer
and editor tabs when the active file-icon theme uses language icons. Separate
light and dark variants preserve its contrast.

The extension runs a native language server locally. Template text and Cargo
workspace metadata are not sent to a remote service. Project indexing reads
workspace Rust sources to associate `#[template(path = "...")]` structs with
their templates. It does not execute project code.

Use **Rusty Handlebars: Show Generated Rust** to inspect the Rust statements
generated for the active in-memory template.

## Development

Build the server and extension:

```sh
cargo build -p rusty-handlebars-language-server
cd editors/vscode
npm install
npm run check
npm run compile
```

Set `rustyHandlebars.server.path` to the development server binary, or copy it
under `server/<platform>-<architecture>/`.

## Cross-compiling Linux packages

Linux x64 and ARM64 language servers can be built from macOS or Linux with
[`cross`](https://github.com/cross-rs/cross). Docker 20.10 or newer (or Podman
3.4 or newer) must be installed and running.

Install `cross` once:

```sh
cargo install cross --git https://github.com/cross-rs/cross --locked
```

Then, from `editors/vscode`, build both Linux servers:

```sh
npm run build-server:linux
```

Build only one architecture by passing `x64` or `arm64`:

```sh
npm run build-server:linux -- x64
npm run build-server:linux -- arm64
```

The binaries are written below:

```text
target/cross/<rust-target>/<rust-target>/release/
```

To build both servers and package a separate platform-specific VSIX for each:

```sh
npm run package:linux
```

This creates:

```text
rusty-handlebars-<version>-linux-x64.vsix
rusty-handlebars-<version>-linux-arm64.vsix
```

## Universal Marketplace package

The GitHub release workflow also creates `rusty-handlebars-universal.vsix`.
It contains language servers for macOS ARM64/x64, Linux ARM64/x64, and Windows
ARM64/x64. The extension selects the matching bundled server at runtime.

Pushing a `vscode-v*` tag creates a GitHub Release and attaches the universal
VSIX together with all six platform-specific packages. Download the universal
VSIX from the repository's Releases page and upload it through the Marketplace
web portal when publishing the platform-specific packages with `vsce` is not
available. Manually dispatched workflow runs keep the packages as workflow
artifacts and do not create a GitHub Release.
