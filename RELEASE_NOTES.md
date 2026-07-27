# Rusty Handlebars 0.3.0

Version 0.3.0 brings the template-authoring toolchain to RustRover and hardens
the shared language server and native editor packaging while preserving Rusty
Handlebars' compile-time rendering model.

## Highlights

### RustRover support

The new RustRover plugin provides:

- a dedicated `.rhbs` file type and bicycle icon;
- layered HTML and Rusty Handlebars syntax highlighting;
- comments, delimiter matching, quote handling, and Live Templates;
- local diagnostics, completion, hover information, definitions, symbols,
  folding, matching-block highlights, signature help, and selection ranges;
- Cargo-aware struct-field and configured-helper intelligence;
- read-only, in-memory generated Rust;
- project settings for a custom server executable and explicitly opted-in
  legacy `.hbs` globs;
- language-server status, restart, and project-index reload controls.

The plugin targets RustRover 2025.3.1 and newer. It does not claim `.hbs`
globally or depend on JetBrains' Handlebars plugin.

### More portable language-server behavior

File URIs now use standards-compliant conversion instead of hand-written
prefix and space replacement. This supports encoded characters, non-ASCII
paths, Windows drive paths, and UNC paths on their respective platforms while
rejecting non-file URI schemes.

Rust and Cargo changes can refresh the Cargo project index without restarting
the language server. Reloading builds a replacement index first and retains
the last valid index if discovery fails. Both VS Code and RustRover watch the
relevant project inputs, and either client can request an explicit reload.

The server also exposes `--version` and has an in-memory
initialize/open/change/save/close/shutdown/exit lifecycle test using a
RustRover-shaped initialization payload.

### Shared native packaging

VS Code and RustRover releases now consume the same native server artifacts:

- macOS ARM64 and x64;
- Linux musl ARM64 and x64;
- Windows x64.

The RustRover workflow assembles one universal plugin archive. Bundled servers
are extracted into a versioned IDE cache, verified with SHA-256, and made
executable where required. Unsupported platforms can use a custom server
binary.

CI builds and tests the RustRover plugin, inspects the universal archive, and
configures Plugin Verifier coverage for the minimum, latest 2025.3 patch, and
newer RustRover platform lines.

## Compatibility notes

- `.rhbs` remains the recommended extension for new templates.
- Existing `.hbs` templates continue to compile and can opt into editor
  language support with project-relative globs.
- The RustRover LSP adapter requires a commercial IntelliJ-based product;
  RustRover is the supported product for this release.
- Windows ARM64 remains unsupported until its native server can be built and
  exercised in CI.
- Marketplace publication and signing are separate release operations.

## Installation

Use version 0.3.0 of the Rust crate:

```toml
[dependencies]
rusty-handlebars = "0.3.0"
```

For VS Code, install the platform-specific `rusty-handlebars` VSIX attached to
the release.

For RustRover, install the universal plugin ZIP through
**Settings → Plugins → Install Plugin from Disk** and restart the IDE if
requested.
