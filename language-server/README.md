# Rusty Handlebars language server

This crate exposes Rusty Handlebars syntax and project information through the
Language Server Protocol over standard input/output. It is primarily packaged
with the VS Code extension in `editors/vscode`, but it has no editor-specific
runtime dependency.

The server reparses open documents after full-document changes. It converts
the parser's byte spans to LSP UTF-16 positions at the protocol boundary.
Structural features work without a Cargo project. When `cargo metadata`
succeeds, the server indexes local Rust structs deriving
`WithRustyHandlebars`, their template paths, named fields, and configured
helpers.

The index deliberately avoids complete Rust name resolution. Unresolved
external and generic types suppress nested semantic claims instead of
reporting speculative errors.

Run it directly with:

```sh
cargo run -p rusty-handlebars-language-server
```
