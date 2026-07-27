# Rusty Handlebars VS Code Extension Plan

## Purpose

Build first-class authoring support for Rusty Handlebars templates without
creating a second, incompatible implementation of the template language.

The extension should initially target VS Code, but the language intelligence
should live in a reusable Rust language server so it can later support other
editors that implement the Language Server Protocol (LSP).

This document is intended to be sufficient context for continuing the work in
a future session without relying on conversation history.

## Implementation Status

The first shippable implementation was added on 2026-07-27:

- Phase 0 fixtures cover valid and invalid syntax.
- Phase 1 provides a borrowed, span-aware, recoverable syntax tree with stable
  diagnostics. The compiler now generates Rust directly from that tree.
- Phase 2 provides the `.rhbs` language, TextMate grammar, language
  configuration, snippets, local bundling, and VSIX packaging.
- Phase 3 provides document synchronization, UTF-16 diagnostic conversion,
  completion, hover, symbols, folding, selection ranges, block highlights,
  signature help, and generated Rust inspection.
- Phase 4 provides Cargo metadata discovery and a conservative `syn` index for
  template contexts, fields, and configured helpers. It powers root field and
  helper completion, known-invalid diagnostics, field type hover, and field
  definitions.
- Phase 5 has CI quality gates and a platform-specific release matrix for
  macOS ARM64/x64, Linux ARM64/x64, and Windows x64.

The intentionally deferred formatter, live preview, complete Rust name
resolution, and browser-hosted extension remain deferred. Marketplace
publication also remains an explicitly authorized release action. Project
indexing currently reflects saved Rust sources at server startup; live
in-memory Rust source overlays and incremental re-indexing are the next
semantic hardening task.

## Decisions

### Use `.rhbs` as the preferred template extension

New templates and documentation should use `.rhbs`, meaning "Rusty
Handlebars."

Reasons:

- It can be associated directly with a `rusty-handlebars` VS Code language ID.
- It avoids conflicts with standard Handlebars extensions that claim `.hbs`.
- Standard Handlebars validators will not report false errors for
  Rusty Handlebars-specific constructs such as `if_some_ref`.
- It makes Rusty Handlebars templates easy to identify in repository searches.
- It is more descriptive than `.rhb`.

The derive macro already accepts arbitrary paths, so the library does not need
special handling for the new extension:

```rust
#[derive(rusty_handlebars::WithRustyHandlebars)]
#[template(path = "templates/profile.rhbs")]
struct Profile<'a> {
    name: &'a str,
}
```

Existing `.hbs` templates must continue to compile. The VS Code extension
should treat `.rhbs` as the automatic association and allow `.hbs` support
through one or more of:

- explicit VS Code file association;
- manual selection of the Rusty Handlebars language mode;
- a configurable list of template globs;
- project-aware detection of paths referenced by `#[template(path = "...")]`.

Do not perform an automatic bulk rename of existing `.hbs` files. Migration
should be a separate, reviewable change that updates template paths and
documentation together.

### Keep one parser

The existing `rusty-handlebars-parser` crate must remain the source of truth
for template syntax. The extension must not implement a separate parser in
TypeScript.

The parser should expose a syntax-oriented, span-aware API. Both the existing
Rust source generator and the language server should consume that API:

```text
                         ┌─> Rust source compiler ─> derive macro
Template ─> parser/AST ──┤
                         └─> language server ──────> VS Code
```

This prevents a template from being accepted by the editor but rejected by
the compiler, or vice versa.

### Use a Rust language server and a thin TypeScript client

The language server should be a native Rust binary communicating over standard
input/output using LSP. The VS Code extension should be responsible only for:

- registering the language and TextMate grammar;
- starting and stopping the server;
- translating VS Code configuration into server initialization options;
- registering VS Code-specific commands;
- locating the bundled server executable.

The server can use the `lsp-server` and `lsp-types` crates. The workload is
primarily synchronous parsing and indexing, so an async runtime is not
required initially.

## Current Relevant Architecture

The workspace currently contains:

- the root `rusty-handlebars` facade crate;
- `derive`, containing `WithRustyHandlebars`;
- `parser`, containing the template parser and Rust source generator;
- `examples`, containing example contexts and templates.

The derive macro reads the path from `#[template(...)]`, reads the template at
compile time, and calls `rusty_handlebars_parser::Compiler`.

The parser currently exposes borrowed `Expression` and `Token` values and
compiles directly into a Rust `String`. Its errors contain human-readable
messages but not structured source spans. Compilation normally stops at the
first error. That is appropriate for code generation but insufficient for an
editor, which needs precise ranges, multiple diagnostics, partial syntax trees,
and recovery while the user is typing.

Supported syntax currently includes:

- HTML-escaped interpolation: `{{value}}`;
- raw interpolation: `{{{value}}}`;
- comments;
- raw/escaped blocks;
- whitespace trimming with `~`;
- parent paths using `../`;
- aliases using `as name` and `as |name|`;
- private block values such as `@index`, `@key`, and `@value`;
- subexpressions;
- built-in inline helpers such as `lookup`, `try_lookup`, and `format`;
- configured Rust helper functions;
- blocks including `if`, `unless`, `if_some`, `with`, and `each`;
- borrowing block variants ending in `_ref`;
- `else` branches.

## Proposed Repository Layout

Add the following workspace components:

```text
language-server/
    Cargo.toml
    src/
        main.rs
        server.rs
        documents.rs
        diagnostics.rs
        completion.rs
        project.rs

editors/
    vscode/
        package.json
        package-lock.json
        tsconfig.json
        esbuild.js
        language-configuration.json
        syntaxes/
            rusty-handlebars.tmLanguage.json
        snippets/
            rusty-handlebars.json
        src/
            extension.ts
            configuration.ts
        test/
        .vscodeignore
```

Keep the parser and compiler in the existing `parser` crate initially. Create a
separate syntax crate only if the parser/compiler boundary becomes difficult to
maintain; an extra crate is not required merely for organizational symmetry.

Add `language-server` to the Cargo workspace. The TypeScript extension remains
outside Cargo's package model.

## Parser Foundation

### Required public data

Introduce structured source locations and diagnostics. Exact names can change
during implementation, but the API should express the following concepts:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub span: Span,
    pub message: String,
    pub code: DiagnosticCode,
    pub severity: Severity,
}

pub struct ParsedTemplate<'source> {
    pub nodes: Vec<Node<'source>>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse_template(source: &str) -> ParsedTemplate<'_>;
```

Spans should use byte offsets internally because that matches Rust string
slicing. The language server must convert byte offsets to LSP UTF-16 line and
character positions. Keep that conversion at the LSP boundary.

Every syntactic item needed by editor features should have a span:

- opening and closing delimiters;
- expression content;
- helper/block name;
- every argument token;
- variable path segments;
- alias declaration;
- block body and `else` body;
- comment and raw-block content.

### Syntax tree

Use a lossless or near-lossless tree that preserves source slices. It should be
possible to reproduce the original template or associate every node with the
original text.

The tree needs to represent incomplete input, for example:

- an opening `{{` without a close;
- a partially typed helper name;
- a block missing its closing block;
- a subexpression missing `)`;
- an unterminated string.

Do not require the compiler to accept incomplete nodes. The compiler should
reject a parsed template containing error diagnostics, while the language
server should retain and analyze all valid surrounding nodes.

### Error recovery

The parser should collect multiple useful diagnostics in one pass. Recovery
boundaries can include:

- the next complete closing delimiter;
- the next line for an unterminated simple expression;
- the next block delimiter at the current nesting depth;
- end of file.

Avoid cascades. One missing close should not create a diagnostic on every
subsequent expression.

Give diagnostics stable codes, such as:

- `unclosed-expression`;
- `unclosed-block`;
- `mismatched-block`;
- `unexpected-else`;
- `duplicate-else`;
- `invalid-token`;
- `unterminated-string`;
- `unmatched-subexpression`;
- `invalid-helper-arguments`;
- `unknown-private-variable`.

Stable codes make tests, documentation, suppression, and future quick fixes
more reliable than matching message strings.

### Compiler migration

Change `Compiler::compile` to parse through the new syntax API and compile the
resulting nodes. Preserve the existing public compiler behavior unless a
separate API addition is needed.

Before refactoring, retain the current generated-source tests as golden tests.
Afterward, all existing templates must produce identical Rust code unless a
change is intentionally reviewed.

## VS Code Declarative Support

The first usable extension does not need the language server to provide basic
editing behavior.

### Language contribution

Register:

- language ID: `rusty-handlebars`;
- display name: `Rusty Handlebars`;
- preferred extension: `.rhbs`;
- language configuration file;
- TextMate grammar;
- snippets.

Do not claim every `.hbs` file globally by default.

### TextMate grammar

The grammar should highlight:

- `{{`/`}}` and `{{{`/`}}}` delimiters;
- block markers `#` and `/`;
- built-in block names;
- inline helper names;
- variables and individual path segments;
- `../`, `this`, and aliases;
- `@index`, `@key`, and `@value`;
- string and numeric literals;
- subexpression parentheses;
- `as` and alias pipes;
- comments;
- raw blocks;
- trim markers.

TextMate highlighting is deliberately lexical. It should remain stable with
partially typed input and should not attempt scope validation.

Where feasible, treat the surrounding document as HTML so normal HTML tags,
attributes, CSS, and JavaScript retain their existing highlighting. Template
patterns must take precedence over embedded HTML patterns.

Create grammar fixture files that cover every syntax form documented in the
root README.

### Language configuration

Configure:

- automatic closing for double and triple braces where practical;
- surrounding pairs;
- comment syntax for `{{! ... }}`;
- indentation around block opening, `else`, and closing expressions;
- word patterns that include Rust-like field paths and private variables;
- folding markers only as a fallback to LSP folding.

Be conservative with automatic insertion. Triple/four-brace constructs and
normal Rust/JavaScript braces inside a template can make aggressive auto-close
rules frustrating.

### Snippets

Initial snippets:

- escaped and raw interpolation;
- `if`/`else`;
- `unless`;
- `if_some_ref`/`else`;
- `with_ref`;
- `each_ref`/`else`;
- alias syntax;
- `lookup`;
- `try_lookup`;
- `format`;
- comment;
- raw block.

Prefer borrowing `_ref` variants in snippets for owned struct fields.

## Language Server

### Document management

Maintain an in-memory document store keyed by URI. Support:

- `textDocument/didOpen`;
- `textDocument/didChange`;
- `textDocument/didClose`;
- `textDocument/didSave`.

Start with full-document synchronization for simplicity. Templates are usually
small, and the parser is fast. Incremental parsing can be considered only
after profiling demonstrates a need.

Parse on open and change, with a short debounce if VS Code sends changes faster
than useful diagnostics can be published.

### Phase-one LSP features

These require template syntax but not Rust project type information:

- parse diagnostics with exact ranges;
- mismatched block diagnostics;
- block-helper completion;
- inline-helper completion;
- scope-aware completion for aliases and private variables;
- hover documentation for built-ins;
- document symbols for blocks;
- folding ranges for block bodies;
- selection ranges for expressions and nested blocks;
- matching/highlighting an opening and closing block;
- basic signature help for helper arguments.

Completion should respect scope:

- `@index` is available in an `each` item body when the loop is enumerated;
- `@key` and `@value` are meaningful for pair-like iteration;
- aliases are available only inside the declaring block;
- parent traversal should offer valid visible parent scopes;
- an item binding is not available in an `each` `else` branch.

Some Rust types are needed to know whether iteration produces pairs. Until
project types are known, completion can offer `@key` and `@value` with a note
that their validity depends on the iterated type.

### Project discovery

Locate the workspace using Cargo manifests and `cargo metadata`. Respect
multi-package workspaces and the same path-resolution rules used by the derive
macro.

Project indexing should:

1. Find Rust files in relevant workspace packages.
2. Parse them with `syn`.
3. Locate structs deriving `WithRustyHandlebars`.
4. Parse their `#[template(...)]` attributes.
5. Resolve each template path.
6. Record the associated context struct and configured helper paths.
7. Build a lightweight index of structs, fields, aliases, and locally
   resolvable field types.

Extract shared template attribute and path-resolution behavior from `derive`
when necessary. Do not copy it into the language server.

The server must handle:

- one struct using one template;
- multiple structs using the same template;
- multiple Cargo packages containing templates with the same relative name;
- unsaved Rust source changes;
- missing or temporarily renamed template files;
- generic context structs;
- tuple and unit structs, even if only to report that field completion is
  unavailable.

If several context structs use one template, show the possible contexts and
avoid claiming that a field is invalid unless it is invalid for all applicable
contexts.

### Project-aware features

Once the template-to-context index exists, add:

- root struct-field completion;
- nested field completion for types resolvable from the local source index;
- unknown-field diagnostics;
- helper completion from `helpers = [...]`;
- unknown-helper diagnostics;
- hover information with Rust field type and source location;
- go to definition from a template path to a Rust field;
- go to definition from a helper call to the configured Rust function;
- references between a Rust `#[template(path)]` and its template file.

Do not initially attempt complete Rust name resolution. A `syn`-based index can
support common local structs. Unknown external or highly generic types should
produce incomplete completion, not false errors.

Do not depend on rust-analyzer internals for the first implementation. The
extension should work when rust-analyzer is absent, and internal integration
would create a fragile version dependency.

### Generated Rust command

Add a command such as `Rusty Handlebars: Show Generated Rust`.

It should compile the current in-memory template with the known helper
configuration and open the generated statements in a read-only virtual Rust
document. This is useful for diagnosing ownership and generated-path behavior
without pretending to be a live rendered preview.

## Deferred Features

### Formatter

Defer formatting until the parser and selection ranges are mature. Literal
whitespace, trim markers, raw blocks, embedded HTML, and plain-text templates
make formatting semantically risky.

If formatting is later added:

- never change literal text without an explicit rule;
- preserve raw blocks exactly;
- test idempotence;
- test that generated output is unchanged;
- consider formatting only expression interiors initially.

### Live preview

Defer live rendered previews. Rendering requires real Rust values for the
context struct, and arbitrary project code should not be built or executed
automatically by an editor extension.

A future preview design should require an explicit user-configured preview
command or fixture and respect VS Code Workspace Trust.

### Complete Rust semantic analysis

Defer cross-crate trait and type inference, method resolution, and exact
ownership diagnostics. Rust compilation and rust-analyzer remain authoritative
for these concerns.

### Browser-hosted extension

The initial server will be a native binary. A future web extension could
compile the syntax/parser layer to WebAssembly, but Cargo workspace discovery
and native filesystem access would require separate design.

## Testing Strategy

### Parser tests

- Exact spans for every token and node type.
- Unicode input and byte-to-UTF-16 conversion.
- Multiple diagnostics in one document.
- Recovery after malformed expressions.
- Recovery after mismatched nested blocks.
- Incomplete input representative of active typing.
- No diagnostic cascades.
- Existing source-generation golden tests.

Property or fuzz tests would be valuable for asserting that parsing arbitrary
UTF-8 never panics and always makes progress.

### Language server tests

Test LSP requests and notifications without launching VS Code:

- open/change/close document lifecycle;
- diagnostic ranges and stable codes;
- completion contents at representative cursor locations;
- completion scope inside nested blocks and `else`;
- hover contents;
- document symbols and folding;
- template-to-context discovery in a multi-package fixture workspace;
- ambiguous contexts;
- missing Cargo metadata or malformed Rust files.

Use position marker helpers in fixtures rather than hard-coded line/column
numbers wherever possible.

### VS Code extension tests

- Extension activation for `.rhbs`.
- Server binary discovery and startup.
- Configuration forwarding.
- Grammar tokenization fixtures.
- snippets load under the correct language ID;
- `Show Generated Rust` command integration;
- graceful error message when the server cannot start.

### Manual acceptance workspace

Create a small fixture workspace containing:

- a simple context;
- nested structs;
- an owned collection requiring `each_ref`;
- optional fields;
- helper functions;
- multiple contexts sharing a template;
- an invalid template;
- both `.rhbs` and legacy `.hbs` templates.

## Packaging and Release

Bundle the TypeScript extension into a small JavaScript artifact using esbuild.
Do not ship `node_modules`.

Build and package native server binaries for at least:

- macOS ARM64;
- macOS x64;
- Linux x64;
- Linux ARM64;
- Windows x64;
- Windows ARM64 if CI and dependencies support it reliably.

Publish platform-specific VSIX packages so users do not need a Rust toolchain.
The extension should choose the server bundled for its platform and
architecture.

CI should:

1. run Rust formatting, clippy, and tests;
2. run TypeScript linting, compilation, and tests;
3. build each supported server target;
4. package platform-specific VSIX files;
5. smoke-test that each package contains the expected executable;
6. attach VSIX artifacts to tagged releases;
7. publish to the VS Code Marketplace only from an explicitly authorized
   release workflow.

Keep the language server version and extension version independently visible.
The extension package should record which server version it bundles.

## Documentation

Update the root README when `.rhbs` becomes the preferred extension:

- use `.rhbs` in new examples;
- state that `.hbs` remains supported by the compiler;
- link to the VS Code extension;
- document how to enable Rusty Handlebars mode for legacy `.hbs`;
- document available commands and settings.

The extension README should document:

- supported syntax;
- features by maturity;
- installation;
- workspace requirements;
- `.hbs` opt-in;
- server logs and troubleshooting;
- privacy/security behavior;
- known limits of Rust field/type analysis.

## Implementation Phases and Acceptance Criteria

### Phase 0: Syntax contract and fixtures

Work:

- collect all supported syntax in parser fixtures;
- make the README syntax examples executable/tested where practical;
- define diagnostic codes and syntax node requirements.

Complete when:

- every documented construct has a fixture;
- expected valid/invalid behavior is recorded;
- no editor-specific code is needed to understand the syntax contract.

### Phase 1: Span-aware recoverable parser

Work:

- add spans, nodes, diagnostics, and recovery;
- migrate the compiler to the parsed representation;
- preserve existing generated output.

Complete when:

- all existing workspace tests pass;
- malformed documents can return several precise diagnostics;
- arbitrary/incomplete input does not panic or loop;
- compiler golden output is unchanged.

### Phase 2: Declarative VS Code extension

Work:

- scaffold `editors/vscode`;
- register `.rhbs`;
- add grammar, language configuration, snippets, tests, and local VSIX
  packaging.

Complete when:

- `.rhbs` opens in Rusty Handlebars mode automatically;
- every syntax fixture has sensible highlighting;
- common blocks can be authored using snippets;
- a locally packaged VSIX installs and activates.

This phase can proceed partly in parallel with Phase 1 because the TextMate
grammar does not require the final parser API.

### Phase 3: Structural language server

Work:

- scaffold `language-server`;
- implement document synchronization;
- publish parser diagnostics;
- add built-in completion, hover, symbols, folding, and selection ranges;
- connect it through the VS Code language client.

Complete when:

- diagnostics update as the user types;
- all diagnostic ranges point to the relevant source;
- completions respect template block scope;
- the extension installs with a bundled server and requires no Rust toolchain.

### Phase 4: Cargo and context indexing

Work:

- discover workspace packages;
- index derive structs, template paths, fields, and helpers;
- add project-aware completion, diagnostics, hover, and definitions.

Complete when:

- the server maps `.rhbs` files to their deriving structs;
- root fields and configured helpers complete correctly;
- common nested local struct fields resolve;
- ambiguous contexts and unresolved external types degrade gracefully;
- legacy `.hbs` files referenced by derives can be recognized.

### Phase 5: Release hardening

Work:

- cross-platform builds;
- platform-specific VSIX packaging;
- CI release workflow;
- performance profiling;
- documentation and troubleshooting;
- marketplace metadata.

Complete when:

- supported packages pass smoke tests on their target platforms;
- opening or editing a large template remains responsive;
- server crashes are reported cleanly and can be diagnosed from logs;
- install, upgrade, and uninstall behavior is documented.

## Risks and Mitigations

### Parser refactor changes generated Rust

Mitigation: lock current output with golden tests before changing parser
architecture and review generated-source diffs explicitly.

### TextMate grammar and parser disagree

Mitigation: derive grammar fixtures from the parser syntax contract. Treat the
grammar as approximate coloring and the Rust parser as authoritative.

### `.hbs` association conflicts

Mitigation: claim only `.rhbs` automatically. Make legacy `.hbs` activation
explicit or project-aware.

### Project indexing reports false field errors

Mitigation: distinguish "known invalid" from "could not resolve." Suppress
unknown-field diagnostics when type/context resolution is incomplete.

### Native binary packaging becomes burdensome

Mitigation: automate a small, explicit platform matrix in CI and keep the
TypeScript client independent from the binary build.

### Editor work drives compiler-specific hacks into the parser

Mitigation: keep syntax, semantic analysis, and source generation as separate
layers with tests at each boundary.

## Suggested First Pull Request

The first implementation PR should be narrowly scoped to:

1. add syntax fixtures covering the current language;
2. introduce `Span` and structured diagnostics;
3. provide recoverable parsing and a syntax tree;
4. make the existing compiler consume that representation;
5. prove generated Rust remains unchanged.

The VS Code scaffold and grammar can be a separate PR immediately afterward,
or can be developed independently once the syntax fixture set is stable.

The first coding session should begin by inventorying every parser/compiler
test and converting representative templates into reusable fixture files.
