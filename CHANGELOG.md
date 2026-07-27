# Changelog

Changes made before version 0.2.0 are recorded in the Git history.

## 0.3.0 - Unreleased

### Added

- A RustRover 2025.3.1 plugin with a dedicated `.rhbs` file type, mixed HTML
  and template highlighting, Live Templates, settings, local LSP integration,
  generated Rust inspection, and language-service controls.
- Shared native language-server build artifacts and universal RustRover plugin
  packaging for macOS ARM64/x64, Linux ARM64/x64, and Windows x64.
- Standards-compliant file URI conversion, project-index reload support, and
  an initialize/shutdown language-server smoke test.

## 0.2.0 - 2026-07-27

### Added

- A span-aware, recoverable public syntax tree with stable diagnostic codes.
- Multiple parse diagnostics for incomplete and malformed templates.
- `.rhbs` as the preferred extension for Rusty Handlebars templates while
  retaining compiler support for `.hbs`.
- A native Rust language server with diagnostics, completion, hover
  information, symbols, folding, selection ranges, matching-block highlights,
  signature help, and generated Rust inspection.
- Cargo workspace and `syn` indexing for template contexts, fields, and
  configured helpers.
- A VS Code extension with HTML-aware syntax highlighting, language
  configuration, snippets, legacy `.hbs` opt-in, and platform-specific server
  packaging.
- Light and dark rusty bicycle file icons for `.rhbs` files.
- Syntax fixtures, UTF-16 position tests, extension build validation, and
  cross-platform CI/release workflows.
- Docker-backed local cross-build and VSIX packaging commands for Linux x64
  and ARM64 language servers.

### Changed

- The compiler now generates Rust directly from the shared syntax tree instead
  of maintaining a separate parsing path.
- Generated Rust consistently uses fully qualified paths and performs fewer
  temporary allocations.
- Documentation now describes the compile-time renderer, template language,
  ownership-oriented block variants, editor support, and current limitations.
- All Rust and editor packages are versioned at 0.2.0 for the coordinated
  release.

### Fixed

- Generic derive support, parent-scope resolution, `each` empty branches, and
  feature propagation across the facade, derive, and parser crates.
- Diagnostic handling for mismatched blocks, duplicate or unexpected `else`,
  unterminated strings, unmatched subexpressions, and invalid private
  variables.
- Existing Clippy warnings across parser and test code.

### Compatibility

- Valid existing `.hbs` templates continue to compile.
- Generated output for existing valid templates remains covered by the
  original golden tests.
- The parser can now reject malformed or mismatched blocks that older versions
  could compile without validating their closing names.
