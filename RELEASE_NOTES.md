# Rusty Handlebars 0.2.0

Version 0.2.0 turns Rusty Handlebars into a complete template-authoring
toolchain while preserving its compile-time rendering model.

## Highlights

### First-class editor support

The new VS Code extension registers `.rhbs` as the dedicated Rusty Handlebars
file type and includes:

- HTML-aware syntax highlighting;
- snippets for interpolation, borrowing blocks, iteration, lookups, formatting,
  comments, and raw blocks;
- live structural diagnostics and scoped completion;
- hover documentation, document symbols, folding, selection ranges, signature
  help, and matching-block highlights;
- Cargo-aware field and configured-helper completion;
- field type hovers and go-to-definition into Rust source;
- a custom rusty bicycle file icon for light and dark themes;
- a **Rusty Handlebars: Show Generated Rust** command.

Existing `.hbs` files remain supported. They can use Rusty Handlebars editor
features through manual language selection or
`rustyHandlebars.legacyFileGlobs` without the extension claiming all
Handlebars files globally.

### One parser for the compiler and editor

The parser now exposes a borrowed, span-aware syntax tree and recoverable
diagnostics with stable codes. The compiler consumes that same tree directly,
removing the risk of editor syntax and compiler syntax drifting apart.

Malformed documents can report several useful errors in one pass, including
unclosed expressions and blocks, mismatched closing blocks, invalid helper
arguments, unterminated strings, unmatched subexpressions, and invalid private
variables. Byte spans remain efficient inside Rust and are converted to UTF-16
positions only at the LSP boundary.

### Cargo-aware language server

The native language server uses `cargo metadata` and `syn` to associate
templates with structs deriving `WithRustyHandlebars`. It indexes named fields
and configured helpers conservatively, avoiding speculative diagnostics when
types cannot be resolved.

The server is packaged into platform-specific VSIX artifacts for macOS ARM64
and x64, Linux ARM64 and x64, and Windows x64, so end users do not need a Rust
toolchain.

Linux x64 and ARM64 servers can also be cross-compiled locally from macOS or
Linux with `npm run build-server:linux`. The matching VSIX files can be built
with `npm run package:linux`.

### Compiler and quality improvements

- Valid templates retain their existing generated Rust output.
- Generated source uses fully qualified paths.
- Rendering and compile-time template generation avoid several unnecessary
  allocations and repeated scans.
- Generic derives, parent-scope resolution, `each` empty branches, and feature
  propagation have regression coverage.
- CI now runs formatting, all-feature workspace tests, warning-free Clippy,
  TypeScript checks, bundling, and extension metadata validation.

## Compatibility notes

- `.rhbs` is recommended for new templates, but `.hbs` remains fully supported
  by the derive macro and compiler.
- Borrowing block variants such as `if_some_ref`, `with_ref`, and `each_ref`
  remain preferred for owned struct fields.
- The stricter parser now rejects mismatched closing block names that older
  releases could accept accidentally.
- Formatting, rendered previews, complete cross-crate Rust type inference, and
  live overlays for unsaved Rust source remain intentionally deferred.

## Installation

Use version 0.2.0 of the Rust crate:

```toml
[dependencies]
rusty-handlebars = "0.2.0"
```

For VS Code, install the platform-specific `rusty-handlebars` VSIX attached to
the release. Marketplace publication remains a separately authorized release
step.
