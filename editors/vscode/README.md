# Rusty Handlebars for VS Code

This extension provides syntax highlighting, snippets, diagnostics, completion,
hover information, symbols, folding, selection ranges, matching-block
highlights, signature help, and generated Rust inspection for Rusty Handlebars
templates.

`.rhbs` files activate the extension automatically. The compiler still accepts
`.hbs`; associate legacy files explicitly with the `rusty-handlebars` language
or add project-specific patterns to `rustyHandlebars.legacyFileGlobs`.

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
