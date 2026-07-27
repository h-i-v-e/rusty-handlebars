# Rusty Handlebars RustRover Plugin Plan

## Purpose

Add first-class Rusty Handlebars template authoring to RustRover without
duplicating the parser, compiler, or language intelligence already implemented
in Rust.

The JetBrains plugin should remain a thin editor adapter around
`rusty-handlebars-language-server`. It should provide the pieces that are
necessarily IDE-specific:

- `.rhbs` file recognition and the bicycle file icon;
- syntax highlighting for Rusty Handlebars embedded in HTML;
- editor configuration, brace matching, commenting, and live templates;
- language-server discovery, extraction, startup, restart, and logging;
- JetBrains actions such as **Show Generated Rust**;
- settings for legacy `.hbs` projects and custom server binaries;
- packaging, compatibility verification, and Marketplace metadata.

This document is intended to contain enough context to implement and release
the plugin after conversation history has been compacted or lost.

## Status

Planning and the first implementation were completed on 2026-07-27.

Implemented in `editors/jetbrains`:

- a RustRover 2025.3.1+ Gradle plugin with `.rhbs` registration, distinct file
  and plugin icons, a Rusty Handlebars lexer, layered HTML/template PSI and
  highlighting, delimiter matching, comments, quoting, and Live Templates;
- project-wide LSP startup for `.rhbs` files and explicitly configured legacy
  globs, with a backend working directory rooted at the Cargo project;
- versioned, checksum-verified extraction and backend-platform selection for
  bundled language-server binaries, plus a validated custom-server setting;
- project settings, a language-service status widget, restart and project-index
  reload actions, debounced Rust/Cargo file watching, and read-only in-memory
  generated Rust;
- unit/platform tests for lexer recovery and tokenization, HTML/template view
  provider registration, legacy path matching, and platform mapping;
- a reusable five-platform native server build, a universal RustRover archive
  workflow, archive-content checks, and Plugin Verifier configuration;
- install, development, privacy, legacy-file, and troubleshooting
  documentation.

Implemented in the shared language server:

- standards-compliant file URI conversion, including percent-encoded and
  non-ASCII paths plus platform-specific Windows drive and UNC coverage;
- watched-file and explicit project-index reload support that preserves the
  last valid index if rediscovery fails;
- a reusable connection lifecycle, `--version`, and a
  RustRover-shaped initialize/shutdown/exit smoke test.

The VS Code client now watches Rust and Cargo project inputs so it benefits
from the same project-index refresh behavior.

Local verification completed during implementation:

- Rust language-server tests and warning-free workspace Clippy;
- VS Code type checking and bundling;
- RustRover platform tests and `buildPlugin`;
- Plugin Verifier against the exact 2025.3.1 minimum build, with no
  compatibility or deprecated-API findings.

The remaining release work requires built cross-platform artifacts or external
IDE/Marketplace state: run the universal packaging workflow, complete the
manual oldest/latest IDE and operating-system acceptance matrix, configure
signing secrets, test the signed archive, and publish only after explicit
authorization. Plugin Verifier is configured in CI for RustRover 2025.3.1,
2025.3.6, 2026.1.4, and 2026.2.

Relevant existing components:

- `rusty-handlebars-parser` is the source of truth for template syntax.
- `rusty-handlebars-language-server` is editor-neutral and communicates over
  standard input/output using LSP.
- `editors/vscode` demonstrates the expected editor behavior and contains the
  current TextMate grammar, snippets, icons, settings, and generated-Rust
  command.
- `.github/workflows/release-vscode.yml` already builds native server binaries
  for macOS ARM64/x64, Linux ARM64/x64, and Windows x64.

## Desired Outcome

After installing the plugin, opening a `.rhbs` file in a Cargo project should
provide:

- the Rusty Handlebars bicycle icon;
- HTML and Rusty Handlebars syntax highlighting;
- template comments and delimiter matching;
- diagnostics while editing;
- completions for syntax, built-ins, configured helpers, and Rust context
  fields;
- hover information;
- go to the defining Rust field;
- document structure, breadcrumbs, and folding;
- matching-block highlights;
- signature help;
- extend/shrink selection;
- live templates corresponding to the VS Code snippets;
- a **Show Generated Rust** action;
- a language-service status indicator and restart action.

The plugin and language server must run locally. Template contents and Cargo
metadata must not be sent to an external service.

## Non-Goals for the First Release

The first release will not:

- replace the Rust parser with a Kotlin parser;
- implement complete Rust name resolution;
- implement a formatter unless one is added to the shared language server;
- implement a rendered HTML preview;
- claim every `.hbs` file globally;
- depend on unstable internals of JetBrains' Handlebars/Mustache plugin;
- add refactoring, references, semantic tokens, code actions, or formatting
  that the language server does not advertise;
- support IntelliJ Community or Android Studio through the LSP adapter, because
  JetBrains' public LSP integration is supplied by commercial IntelliJ-based
  products;
- promise Windows ARM64 until a binary can be built and exercised in CI.

These features can be added later without changing the core architecture.

## Decisions

### Reuse the Rust language server

Do not reimplement template semantics in Kotlin. The existing architecture
must remain:

```text
                                      ┌─> derive macro / generated Display impl
Template ─> rusty-handlebars-parser ──┤
                                      └─> Rust LSP ─> editor-specific client
                                                           ├─> VS Code
                                                           └─> RustRover
```

The Kotlin plugin may contain a lexical highlighter because coloring is an
editor concern. It must not use that lexer to decide whether a template is
valid or to produce semantic diagnostics.

### Keep `.rhbs` as the automatic association

Register `.rhbs` as the Rusty Handlebars file type.

Do not claim `.hbs` globally. Standard Handlebars tooling commonly owns that
extension. Legacy files should be supported by explicit project configuration
or an action that adds a selected path or glob to the Rusty Handlebars plugin
settings.

For configured legacy `.hbs` files:

- retain the IDE's existing file type and highlighting where practical;
- attach the Rusty Handlebars LSP based on the configured path or glob;
- avoid modifying global user file-type associations.

### Target RustRover 2025.3.1 and newer

RustRover 2025.3.1 is the proposed minimum because the IntelliJ Platform LSP
client available by then covers every standard capability currently advertised
by the server:

- diagnostics, completion, hover, and definition;
- folding;
- document highlights and symbols;
- signature help;
- selection ranges.

Use the API available at the minimum supported platform when that provides
binary compatibility with later versions. The LSP API was renamed during the
2026.1 line, so implementation must not casually mix new-only names into a
plugin claiming 2025.3 compatibility.

Before fixing the compatibility range:

1. build against the chosen 2025.3.1 RustRover distribution;
2. run Plugin Verifier against 2025.3.1, the newest 2025.3 patch, 2026.1, and
   the latest stable RustRover;
3. run the plugin manually on both the oldest and latest supported versions;
4. narrow the range or produce separate plugin builds if the LSP API is not
   binary-compatible across those releases.

Do not set an optimistic `until-build` merely to make the Marketplace listing
appear broad.

### Build an independent file type and highlighter

The plugin should not require JetBrains' Handlebars/Mustache plugin. Reusing
its public behavior can be investigated, but depending on its implementation
classes would introduce version coupling and could make Marketplace
compatibility fragile.

Implement Rusty Handlebars as an HTML template language using public IntelliJ
Platform APIs:

- a `Language`/`TemplateLanguage`;
- a `LanguageFileType`;
- a template-aware file view provider;
- a lexer that separates template data from Rusty Handlebars delimiters and
  expressions;
- HTML as the template-data language;
- a small parser definition or template-data element structure where the
  platform requires one;
- a syntax highlighter for Rusty Handlebars tokens.

The highlighter should mirror the current VS Code TextMate grammar:

- normal and triple-brace delimiters;
- comments;
- raw blocks;
- block open/close sigils;
- built-in blocks and helpers;
- strings and escapes;
- numbers and booleans;
- aliases and `else`;
- private values such as `@index`;
- parent paths;
- variable paths and punctuation.

The highlighter is allowed to accept incomplete input. Syntax validity remains
the responsibility of the shared parser and language server.

### Ship one universal plugin archive

The first release should contain these native language-server binaries:

| Runtime | Rust target | Initial support |
| --- | --- | --- |
| macOS ARM64 | `aarch64-apple-darwin` | Required |
| macOS x64 | `x86_64-apple-darwin` | Required |
| Linux ARM64 | `aarch64-unknown-linux-musl` or verified GNU equivalent | Required |
| Linux x64 | `x86_64-unknown-linux-musl` or verified GNU equivalent | Required |
| Windows x64 | `x86_64-pc-windows-msvc` | Required |
| Windows ARM64 | `aarch64-pc-windows-msvc` | Deferred until tested |

A single archive simplifies Marketplace publication and remote-development
installation. The plugin must select the binary using the OS and architecture
of the IDE backend, not the display frontend.

Prefer statically linked musl servers on Linux if all dependencies work
correctly. Otherwise build against a sufficiently old glibc baseline and test
the resulting binary on every Linux baseline that the plugin claims to
support. Do not assume a binary built on the newest Ubuntu runner is compatible
with older supported distributions.

Always retain a custom server-path setting so users on an unsupported platform
can build and select their own server.

## Proposed Repository Layout

```text
editors/
    jetbrains/
        README.md
        CHANGELOG.md
        build.gradle.kts
        settings.gradle.kts
        gradle.properties
        gradlew
        gradlew.bat
        gradle/
            wrapper/
                gradle-wrapper.jar
                gradle-wrapper.properties
        src/
            main/
                kotlin/
                    dev/hive/rustyhandlebars/
                        RustyHandlebarsLanguage.kt
                        RustyHandlebarsFileType.kt
                        RustyHandlebarsIcons.kt
                        editor/
                            RustyHandlebarsLexer.kt
                            RustyHandlebarsParserDefinition.kt
                            RustyHandlebarsSyntaxHighlighter.kt
                            RustyHandlebarsSyntaxHighlighterFactory.kt
                            RustyHandlebarsBraceMatcher.kt
                            RustyHandlebarsCommenter.kt
                            RustyHandlebarsFileViewProvider.kt
                        lsp/
                            RustyHandlebarsLspSupportProvider.kt
                            RustyHandlebarsLspDescriptor.kt
                            RustyHandlebarsProtocol.kt
                        server/
                            ServerBinaryManager.kt
                            ServerPlatform.kt
                        actions/
                            ShowGeneratedRustAction.kt
                            RestartLanguageServerAction.kt
                        settings/
                            RustyHandlebarsSettings.kt
                            RustyHandlebarsConfigurable.kt
                            RustyHandlebarsSettingsComponent.kt
                resources/
                    META-INF/
                        plugin.xml
                        pluginIcon.svg
                    icons/
                        rusty-handlebars.svg
                    liveTemplates/
                        RustyHandlebars.xml
                    messages/
                        RustyHandlebarsBundle.properties
                    server/
                        darwin-arm64/
                        darwin-x64/
                        linux-arm64/
                        linux-x64/
                        win32-x64/
            test/
                kotlin/
                testData/
                    highlighting/
                    completion/
                    diagnostics/
                    fixtures/
```

Exact class names can follow the platform API version selected during the
scaffold. Keep responsibilities separated even if the initial implementations
are small.

Use a stable Marketplace/plugin ID such as
`dev.hive.rusty-handlebars`. Confirm that the ID and display name are available
before the first Marketplace upload. Kotlin packages cannot contain the
hyphens from the GitHub organization, so `dev.hive.rustyhandlebars` is the
proposed source package.

## Plugin Architecture

### File type and editor layer

```text
.rhbs file
    │
    ├─> RustyHandlebarsFileType ─> icon / language identity
    │
    ├─> template lexer
    │      ├─> HTML template data ─> platform HTML highlighting
    │      └─> {{...}} tokens ─────> Rusty Handlebars highlighting
    │
    └─> LSP support provider ─> language server process
```

The file layer must work even if the language server cannot start. Users should
still receive file recognition, highlighting, comments, delimiter handling,
and live templates.

### LSP process layer

Use one project-wide language-server process:

```text
RustRover Project
    │
    ├─> LSP support provider
    │      ├─> accepts *.rhbs
    │      └─> accepts configured legacy globs
    │
    ├─> server descriptor
    │      ├─> project base directory as workspace root
    │      ├─> stdio transport
    │      └─> selected executable
    │
    └─> rusty-handlebars-language-server
           ├─> open-document parser
           └─> Cargo project index
```

Do not start a separate process for every file. The server already indexes a
workspace and stores multiple open documents.

Use the IDE project base directory as both:

- the LSP workspace/root URI;
- the server process working directory.

If the project has no base directory, either decline to start or use the
opened file's nearest sensible directory without claiming Cargo-aware
features.

### Native binary layer

Native binaries packaged as resources cannot be assumed to be directly
executable. `ServerBinaryManager` should:

1. honor a non-empty custom server path first;
2. map the current backend OS and architecture to a packaged target;
3. choose a cache path containing the plugin version and target;
4. calculate or read a packaged checksum;
5. reuse an extracted binary only if its checksum matches;
6. otherwise extract to a temporary sibling path;
7. atomically move the completed extraction into place;
8. add owner execute permission on Unix;
9. verify that the resulting path is a regular executable file;
10. return an actionable error for unsupported systems.

A suitable cache layout is:

```text
<IDE system path>/
    rusty-handlebars/
        <plugin version>/
            <platform>/
                rusty-handlebars-language-server[.exe]
```

Never extract into the source repository, project directory, or a broad shared
temporary path. Versioning the cache prevents an IDE update from reusing an
older server accidentally.

Log the selected platform, executable path, server version, and startup error
to the IDE log. Do not log template contents.

### Settings

Provide a project settings page containing:

- **Language server path**: empty means bundled server;
- **Legacy template globs**: paths such as `templates/**/*.hbs`;
- optionally **Trace language server communication** if supported cleanly by
  the target platform API.

Store project-specific legacy globs in project settings. The custom binary path
may be application-level or project-level; prefer project-level initially so a
development checkout can select its matching debug server without affecting
other projects.

Validate settings before applying them:

- a custom server path must exist and be a regular file;
- invalid glob syntax must be reported next to the setting;
- blank glob entries should be removed;
- paths should not be resolved through a shell.

Changing the executable path or legacy globs should restart or reconnect the
project language service in a controlled way.

### Legacy `.hbs` matching

Implement legacy matching entirely in the LSP support provider:

1. normalize the project-relative path to `/` separators;
2. always support `.rhbs`;
3. for other files, test the normalized path against configured globs;
4. do not attach merely because the extension is `.hbs`;
5. do not mutate `FileTypeManager` global associations.

Consider an editor/project-view action:

**Rusty Handlebars: Enable Language Support for This File**

It should add either the exact project-relative path or an obvious
directory-scoped glob after showing the proposed value. This is useful but can
follow the first working LSP implementation.

## Shared Language Server Work

The server is already largely ready for RustRover, but complete the following
hardening before claiming cross-platform support.

### Correct file URI handling

The current `uri_path` implementation strips `file://` and replaces `%20`.
Replace it with standards-compliant file-URI conversion.

Requirements:

- correctly decode all percent-encoded path bytes;
- support Windows drive-letter URIs such as `file:///C:/work/template.rhbs`;
- support spaces and non-ASCII paths;
- reject non-file URI schemes;
- preserve valid UNC paths if the selected URI library supports them;
- use the same conversion for initialization roots, document paths, and
  definition targets;
- construct definition URIs through a file-path-to-URI API instead of string
  formatting.

Add unit tests for Unix, Windows, spaces, `#`, `%`, and non-ASCII path
components. Tests that are inherently platform-specific should be gated or
expressed through platform-neutral URI parsing helpers.

### Project-index refresh

The project index is currently created once at server startup. Implement one
of these in priority order:

1. handle `workspace/didChangeWatchedFiles` for relevant `.rs`, `Cargo.toml`,
   and possibly `Cargo.lock` changes;
2. expose a `rustyHandlebars/reloadProject` custom request and invoke it from
   editor clients;
3. retain server restart as the fallback.

Re-indexing should:

- debounce batches of file changes;
- build a new index before replacing the active one;
- retain the previous valid index if discovery fails;
- report failures through logs rather than unrelated template diagnostics;
- avoid scanning `target`, VCS directories, and dependency sources;
- not execute project code.

This work benefits both VS Code and RustRover and should not be implemented
only inside the Kotlin plugin.

### Initialization compatibility

Record initialization payloads from both VS Code and RustRover in tests or
fixtures. Verify:

- `workspaceFolders` and `rootUri` handling;
- UTF-16 position negotiation;
- full-document synchronization;
- `didOpen`, `didChange`, `didSave`, and `didClose`;
- shutdown and exit behavior;
- client handling of server-published diagnostics.

The server should ignore unknown client capabilities and notifications.

### Custom protocol

Keep custom methods documented and editor-neutral:

```text
rustyHandlebars/showGeneratedRust
rustyHandlebars/projectContexts
rustyHandlebars/reloadProject       # if added
```

Add request and response structs to a small shared Rust protocol module where
that reduces duplicated JSON assumptions.

## Editor Features

### Syntax highlighting

Port the intent of
`editors/vscode/syntaxes/rusty-handlebars.tmLanguage.json`, not its JSON
implementation.

Create fixtures for:

- escaped and raw interpolation;
- long comments and inline comments;
- raw blocks;
- whitespace trimming;
- nested blocks and `else`;
- helpers and subexpressions;
- aliases with and without pipes;
- `../` paths;
- `@index`, `@key`, and `@value`;
- unterminated strings and expressions;
- literal HTML, attributes, scripts, and styles around template expressions.

Confirm that template delimiters do not cause the remaining HTML document to
lose highlighting.

### Brace and delimiter behavior

Support:

- `{{` with `}}`;
- `{{{` with `}}}`;
- parentheses inside subexpressions;
- quotes inside expressions;
- block open/close highlighting through LSP document highlights.

Do not implement structural block validation in the brace matcher; the parser
already reports mismatched or missing block closings.

### Comments

Use Rusty Handlebars comments:

```handlebars
{{! comment }}
{{!-- long comment --}}
```

If the JetBrains commenter API supports only one block-comment form, use the
long form for block comment/uncomment and leave the short form highlighted and
understood by the parser.

### Live templates

Convert every useful entry in
`editors/vscode/snippets/rusty-handlebars.json` to a JetBrains Live Template.

At minimum include:

- interpolation and raw interpolation;
- `if`/`else` and `unless`;
- `if_some` and `if_some_ref`;
- `with` and `with_ref`;
- `each` and `each_ref`;
- `lookup`, `try_lookup`, and `format`;
- comments and raw blocks.

Use a dedicated Rusty Handlebars live-template context so suggestions do not
appear in unrelated files.

### Diagnostics and completion

Rely on standard LSP behavior. Do not duplicate server results through
annotators or IDE-native completion contributors.

Test that:

- diagnostics update after a full-document change;
- diagnostic ranges containing non-ASCII text are correct;
- syntax completions appear after `{`, `@`, and `/`;
- project fields and configured helpers are available;
- completions insert valid text without duplicate delimiters.

### Hover and navigation

Verify standard LSP hover and definition handling. A definition from a
template field should open the relevant Rust source file at the field name.

The language server deliberately does not perform complete Rust name
resolution. Unresolved external or generic types should suppress speculative
nested field features rather than create false diagnostics.

### Symbols, folding, highlights, signature help, and selection

These should require little or no Kotlin feature code once the platform LSP
client is connected. Add integration coverage because availability differs
across RustRover platform versions.

If a feature is absent on the minimum supported RustRover despite the platform
documentation, either:

- raise the minimum version;
- document the version-dependent feature;
- or implement a small native fallback only if it can remain consistent with
  the shared parser.

Do not silently duplicate an LSP feature through two providers.

### Show Generated Rust

Implement a JetBrains action corresponding to the VS Code command:

1. enable only for a supported Rusty Handlebars editor;
2. send `rustyHandlebars/showGeneratedRust` with the document URI;
3. use the current in-memory document, as the server already does;
4. create or update a read-only `LightVirtualFile` or equivalent;
5. assign the Rust file type so RustRover highlights the generated code;
6. open it in a split editor where the public API permits;
7. display a concise notification if generation fails.

Do not write generated source into the user's project or operating-system temp
directory merely to display it.

The IntelliJ LSP API requires an explicit custom LSP4J request interface for
non-standard requests. Isolate that version-sensitive code in
`RustyHandlebarsProtocol.kt`.

### Restart and status

Provide:

- **Rusty Handlebars: Restart Language Server**;
- a Language Services status-bar item with the bicycle/plugin icon;
- a link from startup errors to the Rusty Handlebars settings page.

Expected failure messages include:

- no bundled binary for this platform;
- custom binary not found;
- permission denied;
- executable exited during startup;
- Cargo project discovery failed.

Cargo discovery failure should not prevent structural template support. The
server already falls back to an empty project index.

## Gradle and Plugin Metadata

Use the IntelliJ Platform Gradle Plugin 2.x and Kotlin DSL. Use the current
stable plugin/Gradle versions at implementation time, subject to the minimum
RustRover compatibility requirement.

The Gradle project should:

- target a RustRover distribution rather than generic IntelliJ IDEA;
- use Java 21 bytecode only if the minimum RustRover runtime supports it;
- patch `since-build` from the declared compatibility baseline;
- run unit and platform tests;
- build an installable ZIP with `buildPlugin`;
- run Plugin Verifier;
- support signing and Marketplace publishing through environment variables;
- never place signing keys or Marketplace tokens in the repository.

`plugin.xml` should declare at least the modules required for:

- the IntelliJ Platform language/file APIs;
- the commercial LSP integration;
- RustRover compatibility where an explicit Rust-capable module is needed.

Verify the final dependency set from the built plugin. Marketplace product
compatibility is derived from these module dependencies, so an unnecessary
module can incorrectly exclude IDEs and a missing module can make installation
succeed but startup fail.

Metadata must include:

- stable plugin ID;
- display name `Rusty Handlebars`;
- vendor name and contact details;
- repository, issue tracker, and documentation URLs;
- MIT license;
- concise description of compile-time Rust template support;
- change notes;
- a compliant 40×40 SVG plugin logo that does not resemble JetBrains product
  branding.

Reuse the bicycle concept, but create a Marketplace/plugin logo variant that
remains legible at 40×40. Keep the file icon and Marketplace logo separate if
their required visual weights differ.

## Native Build and Packaging Pipeline

Refactor native server compilation so VS Code and JetBrains packages consume
the same release-quality artifacts.

Suggested workflow:

```text
build-server matrix
    ├─> darwin-arm64 artifact
    ├─> darwin-x64 artifact
    ├─> linux-arm64 artifact
    ├─> linux-x64 artifact
    └─> win32-x64 artifact
              │
              ├─> package VS Code platform VSIX files
              └─> package one universal JetBrains ZIP
```

Each native build must:

- use `cargo build --locked --release`;
- build only `rusty-handlebars-language-server`;
- record the Rust target and commit SHA;
- upload the raw executable with an unambiguous artifact name;
- generate a SHA-256 checksum;
- preserve executable permissions where artifact transport permits;
- run `--version` if that option is implemented, or perform an LSP
  initialize/shutdown smoke test.

The JetBrains packaging job must fail if any required binary or checksum is
missing. After `buildPlugin`, inspect the ZIP and verify:

- plugin descriptor and JAR are present;
- all five server binaries are included exactly once;
- no Cargo `target`, Gradle cache, IDE sandbox, test data not required at
  runtime, or signing material is included;
- the plugin version and embedded server version are intentional.

### Versioning

Initially coordinate the JetBrains plugin version with the Rust workspace and
VS Code extension version, because each archive embeds a particular server.

Use a separate tag namespace:

```text
jetbrains-v0.3.0
```

This permits editor-only patch releases without confusing crates.io tags. If
editor release cadence diverges later, document the embedded server version in
the plugin metadata and allow plugin versions to advance independently.

## Git Ignore Requirements

Before running the Gradle build, add ignore rules for generated content:

```gitignore
editors/jetbrains/.gradle/
editors/jetbrains/.intellijPlatform/
editors/jetbrains/build/
editors/jetbrains/out/
editors/jetbrains/.idea/
editors/jetbrains/*.zip
```

Keep the Gradle wrapper JAR and wrapper properties tracked. Check
`git status --short` after the first:

- Gradle sync;
- `runIde`;
- test run;
- plugin build;
- Plugin Verifier run.

No step should result in thousands of generated files being proposed for
version control.

## Testing Strategy

### Rust tests

Add or retain tests for:

- parser syntax fixtures;
- UTF-16 byte/position conversion;
- file URI/path conversion;
- project discovery;
- completion, hover, definitions, symbols, folding, highlights, signatures,
  and selection ranges;
- custom generated-Rust request;
- LSP initialize/open/change/close/shutdown lifecycle;
- project re-indexing if implemented.

Run:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Kotlin unit and platform tests

Test:

- OS/architecture mapping;
- unsupported-platform errors;
- custom server path validation;
- extraction, checksum reuse, replacement, and Unix permissions;
- legacy glob normalization and matching;
- lexer tokenization and restart states;
- highlighting fixtures;
- file-type and icon registration;
- settings persistence;
- action enablement;
- generated-Rust virtual file creation.

Avoid tests that depend on the developer's installed RustRover or global plugin
settings.

### LSP integration tests

Run a real test server binary where practical. Cover:

1. open a `.rhbs` fixture;
2. receive a known diagnostic;
3. change the document and observe the diagnostic clear;
4. request completion;
5. request hover and definition using a small Cargo fixture;
6. request generated Rust;
7. shut the project and confirm the process exits.

Use timeouts and capture server stderr so CI failures are diagnosable.

### Plugin Verifier

Verify the built ZIP against:

- the exact minimum supported RustRover;
- the newest patch of the minimum platform line;
- each intervening major platform line;
- the current stable RustRover;
- optionally the current EAP as a non-blocking early-warning job.

Treat use of internal APIs, missing dependencies, and binary incompatibilities
as release blockers.

### Manual acceptance matrix

Perform the following on at least macOS ARM64, Linux x64, and Windows x64.
Exercise the other packaged architectures through CI smoke tests or real
hardware before Marketplace publication.

Manual checks:

- install the ZIP from disk into a clean RustRover profile;
- open an existing Cargo project using Rusty Handlebars;
- create and open a `.rhbs` file;
- confirm icon and mixed HTML/template highlighting;
- trigger structural and project-aware completions;
- introduce and fix syntax errors;
- hover a field and navigate to its Rust declaration;
- exercise folding, structure, highlights, signatures, and selection;
- open generated Rust;
- change settings and restart the server;
- configure one legacy `.hbs` path without affecting unrelated `.hbs` files;
- close the project and check that the server process exits;
- uninstall the plugin and confirm no project files were modified.

## Documentation

Add `editors/jetbrains/README.md` containing:

- supported RustRover versions and operating systems;
- feature list;
- privacy/local-processing statement;
- `.rhbs` and legacy `.hbs` behavior;
- Marketplace and install-from-disk instructions;
- custom language-server path instructions;
- development commands;
- troubleshooting and IDE log locations;
- known project-index limitations.

Update the root `README.md` editor-support section once the plugin is usable.
Link both editor implementations and this plan.

Update `CHANGELOG.md` and `RELEASE_NOTES.md` only when the implementation is
ready for a release. Avoid promising Marketplace availability before approval.

## Implementation Phases

### Phase 0: Compatibility spike

- [ ] Create the smallest Gradle/Kotlin plugin under `editors/jetbrains`.
- [ ] Target RustRover 2025.3.1.
- [ ] Register a temporary `.rhbs` file type.
- [ ] Start the development IDE using `runIde`.
- [ ] Connect a minimal project-wide LSP descriptor to a locally configured
      debug server.
- [ ] Confirm diagnostics and completion in a real `.rhbs` file.
- [ ] Build the plugin against the old LSP API and verify it against the latest
      RustRover.
- [ ] Decide whether one binary-compatible build can cover the planned range.
- [ ] Record the exact `since-build` and verified platform builds in this
      document.

Exit criterion: a local RustRover instance starts the existing language server
and displays at least one diagnostic and one completion.

### Phase 1: Server hardening

- [ ] Replace hand-written file URI conversion.
- [ ] Add Unix and Windows URI tests.
- [ ] Verify initialization data sent by RustRover.
- [ ] Add project re-indexing or a documented reload request.
- [ ] Add an initialize/shutdown smoke-test harness suitable for native CI.
- [ ] Keep VS Code behavior passing after the shared changes.

Exit criterion: the server handles paths and lifecycle correctly on every
packaged operating system, and Rust source changes can be reflected without
manually killing an orphan process.

### Phase 2: File type and declarative editing

- [ ] Register the final Rusty Handlebars language and `.rhbs` file type.
- [ ] Add file and plugin icons.
- [ ] Implement HTML template-data support.
- [ ] Implement the Rusty Handlebars lexer and syntax highlighter.
- [ ] Add color settings if useful and stable.
- [ ] Add brace matching, quoting, and comments.
- [ ] Port VS Code snippets to Live Templates.
- [ ] Add highlighting and editor fixture tests.

Exit criterion: `.rhbs` is pleasant to edit without a running language server,
including mixed HTML/template highlighting.

### Phase 3: Complete standard LSP integration

- [ ] Implement the project-wide support provider and descriptor.
- [ ] Support `.rhbs` plus configured legacy globs.
- [ ] Pass the project root and working directory correctly.
- [ ] Integrate bundled/custom server selection.
- [ ] Confirm every advertised standard LSP feature on the minimum and latest
      RustRover.
- [ ] Add restart behavior and status-bar integration.
- [ ] Add integration tests for process lifecycle and representative features.

Exit criterion: standard language features have practical parity with the VS
Code extension on supported RustRover versions.

### Phase 4: Settings and custom actions

- [ ] Add persistent project settings and validation.
- [ ] Restart the language service after relevant setting changes.
- [ ] Implement custom LSP4J protocol types.
- [ ] Implement **Show Generated Rust** using an in-memory Rust file.
- [ ] Add **Restart Language Server**.
- [ ] Optionally add **Enable Language Support for This File** for legacy
      templates.
- [ ] Test actions and settings.

Exit criterion: users can inspect generated code, recover from server failures,
and configure legacy or development environments without editing IDE internals.

### Phase 5: Universal native packaging

- [ ] Refactor the native build matrix into reusable artifacts.
- [ ] Select and validate the Linux linking strategy.
- [ ] Build the five required server targets.
- [ ] Generate checksums.
- [ ] Implement safe versioned extraction.
- [ ] Assemble one JetBrains plugin ZIP.
- [ ] Inspect archive contents in CI.
- [ ] Smoke-test each executable.

Exit criterion: the installable ZIP contains and launches the correct server on
every claimed OS/architecture without requiring Rust or Cargo to build the
server.

Cargo remains necessary only when project-aware indexing calls
`cargo metadata`; structural template support should still work when Cargo
metadata is unavailable.

### Phase 6: Release readiness

- [ ] Run Rust, Kotlin, integration, and Plugin Verifier suites.
- [ ] Complete the manual acceptance matrix.
- [ ] Write JetBrains plugin documentation.
- [ ] Update root editor-support documentation.
- [ ] Add release notes and change log entries.
- [ ] Create Marketplace-ready description, vendor data, license link, and
      40×40 logo.
- [ ] Configure signing secrets outside the repository.
- [ ] Produce a signed release candidate.
- [ ] Install and test the exact signed archive.

Exit criterion: the signed ZIP is suitable for install-from-disk distribution
and Marketplace submission.

### Phase 7: Marketplace publication

This phase is an external release action and requires explicit authorization.

- [ ] Confirm the final commit and tag.
- [ ] Create or select the Marketplace vendor profile.
- [ ] Accept the applicable Marketplace agreement.
- [ ] Create the plugin listing and reserve the plugin ID.
- [ ] Upload the verified signed archive.
- [ ] Complete any review-requested corrections.
- [ ] After approval, verify installation from the Marketplace in RustRover.
- [ ] Publish final links in project documentation and release notes.

Do not claim completion merely because an archive was uploaded; confirm its
Marketplace status and installability.

## First-Release Acceptance Criteria

The first RustRover release is complete only when all of the following are
true:

- [ ] `.rhbs` is registered with the correct icon.
- [ ] HTML and Rusty Handlebars syntax are both highlighted.
- [ ] Unrelated `.hbs` files are not claimed.
- [ ] The bundled server launches on every claimed platform.
- [ ] A custom server path works.
- [ ] Diagnostics, completion, hover, definition, symbols, folding,
      highlights, signature help, and selection ranges have been verified.
- [ ] Rust project field/helper intelligence works in a representative Cargo
      workspace.
- [ ] Generated Rust opens as a read-only in-memory Rust document.
- [ ] Closing the project stops the server.
- [ ] File paths with spaces and non-ASCII characters work.
- [ ] Windows file URIs work.
- [ ] No project source or configuration is modified without an explicit user
      action.
- [ ] No template contents or Cargo metadata leave the machine.
- [ ] Tests, linting, archive inspection, and Plugin Verifier pass.
- [ ] The install-from-disk instructions have been exercised with the exact
      release archive.

Marketplace approval is a separate external status. The implementation can be
considered release-ready before approval, but should not be described as
Marketplace-available until installation from the approved listing is
confirmed.

## Deferred Enhancements

After the initial release, consider:

- Windows ARM64 packaging;
- semantic tokens generated by the Rust parser;
- formatting and range formatting;
- quick fixes for missing/mismatched closing blocks;
- rename and references for aliases or project fields;
- code actions;
- rendered HTML preview;
- in-memory overlays for unsaved Rust source changes;
- multi-root workspace indexing;
- richer Rust type resolution;
- automatic but conservative discovery of `.hbs` paths referenced by
  `#[template(path = "...")]`;
- support for other commercial IntelliJ-based IDEs with Rust capability;
- separate lightweight support for IDEs without JetBrains' LSP module.

Each semantic feature should be added to the shared Rust server when possible
so VS Code and JetBrains clients remain consistent.

## Recommended Execution Order

The safest implementation order is:

```text
compatibility spike
    ↓
URI/lifecycle hardening
    ↓
file type + mixed highlighting
    ↓
standard LSP features
    ↓
settings + generated Rust action
    ↓
universal native packaging
    ↓
verification + documentation
    ↓
explicitly authorized Marketplace publication
```

Do not begin with Marketplace automation or a full native PSI model. The
critical uncertainty is the RustRover LSP compatibility range; resolve that
with a minimal running plugin before investing in the presentation and release
layers.

## Implementation References

Consult the current official documentation during implementation because the
IntelliJ Platform LSP API is still evolving:

- LSP integration:
  <https://plugins.jetbrains.com/docs/intellij/language-server-protocol.html>
- language and file-type registration:
  <https://plugins.jetbrains.com/docs/intellij/language-and-filetype.html>
- custom language support:
  <https://plugins.jetbrains.com/docs/intellij/custom-language-support.html>
- syntax and error highlighting:
  <https://plugins.jetbrains.com/docs/intellij/syntax-highlighting-and-error-highlighting.html>
- IntelliJ Platform Gradle Plugin:
  <https://plugins.jetbrains.com/docs/intellij/tools-intellij-platform-gradle-plugin.html>
- Gradle platform dependencies, including RustRover:
  <https://plugins.jetbrains.com/docs/intellij/tools-intellij-platform-gradle-plugin-dependencies-extension.html>
- Plugin Verifier configuration:
  <https://plugins.jetbrains.com/docs/intellij/tools-intellij-platform-gradle-plugin-extension.html>
- product/module compatibility:
  <https://plugins.jetbrains.com/docs/intellij/plugin-compatibility.html>
- plugin signing:
  <https://plugins.jetbrains.com/docs/intellij/plugin-signing.html>
- Marketplace upload:
  <https://plugins.jetbrains.com/docs/marketplace/uploading-a-new-plugin.html>
- Marketplace approval requirements:
  <https://plugins.jetbrains.com/docs/marketplace/jetbrains-marketplace-approval-guidelines.html>
