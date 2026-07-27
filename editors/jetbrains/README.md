# Rusty Handlebars for RustRover

The Rusty Handlebars plugin adds template-authoring support for compile-time
Rust templates to RustRover.

## Requirements

- RustRover 2025.3.1 or newer;
- macOS ARM64 or x64, Linux ARM64 or x64, or Windows x64 when using a release
  archive containing bundled language servers;
- a custom language-server binary on other platforms or in development builds
  that do not contain the native server resources.

The plugin uses the public LSP integration in commercial IntelliJ-based IDEs.
RustRover is the supported and verified product for the first release.

## Features

- a dedicated `.rhbs` file type and bicycle icon;
- HTML and Rusty Handlebars syntax highlighting;
- delimiter matching, template comments, string quote handling, and Live
  Templates;
- diagnostics, completion, hover information, definitions, symbols, folding,
  matching-block highlights, signature help, and selection ranges through the
  shared Rust language server;
- Cargo-aware struct-field and configured-helper information;
- **Tools | Rusty Handlebars: Show Generated Rust**;
- project-index reload and language-server restart actions;
- project settings for an optional server binary and explicitly opted-in
  legacy template globs.

All parsing, Cargo discovery, and language intelligence runs locally. Template
contents and Cargo metadata are not sent to an external service.

## File associations

`.rhbs` files are recognized automatically. The plugin deliberately does not
claim `.hbs`, because standard Handlebars plugins commonly own that extension.

To enable language-server support for an existing `.hbs` file, open
**Settings | Languages & Frameworks | Rusty Handlebars** and add a
project-relative glob, for example:

```text
templates/**/*.hbs
```

This attaches Rusty Handlebars language intelligence without changing the
IDE's global `.hbs` file-type association.

## Installing a release archive

1. Download the universal RustRover ZIP produced by the
   `Package RustRover plugin` workflow.
2. In RustRover, open **Settings | Plugins**.
3. Choose the gear menu, then **Install Plugin from Disk…**.
4. Select the ZIP and restart the IDE if requested.
5. Open a Cargo project and a `.rhbs` file.

Marketplace publication is not implied by the presence of the build workflow.
It remains a separately authorized release step.

## Using a custom language server

Build the server from this repository:

```sh
cargo build -p rusty-handlebars-language-server
```

Set **Language server path** in the Rusty Handlebars project settings to:

```text
<repository>/target/debug/rusty-handlebars-language-server
```

On Windows, use the corresponding `.exe`. The path is executed directly and
is never passed through a shell.

## Development

Use Java 21. From this directory:

```sh
./gradlew test
./gradlew buildPlugin
./gradlew runIde
./gradlew verifyPlugin
```

Before `runIde`, configure a custom debug server in the development IDE or
stage native server resources under:

```text
src/main/resources/server/<platform>/
```

Each staged executable must have an adjacent `.sha256` file. Release CI builds
and stages all five supported targets automatically.

Generated Gradle, Kotlin, IDE sandbox, and build directories are ignored by
Git. The Gradle wrapper JAR and properties are intentionally tracked.

## Troubleshooting

- Use **Tools | Rusty Handlebars: Restart Language Server** after changing a
  custom server binary.
- Use **Tools | Rusty Handlebars: Reload Project Index** after unusual Cargo
  workspace changes. Ordinary Rust and Cargo file changes are watched and
  re-indexed automatically.
- If the bundled binary is unavailable, set a custom server path and confirm
  that the file is executable.
- Open **Help | Show Log in Finder/Explorer** for startup and LSP errors.
- Add `#com.intellij.platform.lsp` under **Help | Diagnostic Tools | Debug Log
  Settings…** for detailed protocol logging.

Cargo discovery uses `cargo metadata` and a conservative source index.
Structural template features continue to work if Cargo discovery fails, but
project-aware fields and helpers may be unavailable. Complete Rust name
resolution and unsaved Rust-source overlays are not part of the first release.
