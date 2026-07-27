# Rusty Handlebars

Rusty Handlebars turns a Handlebars-like template file into a Rust
`Display` implementation at compile time.

The template context is a Rust struct. Template paths become ordinary Rust
field accesses, loops become `for` expressions, and conditionals become `if`
expressions. Rendering does not parse a template or build a dynamic context:

```rust
use rusty_handlebars::WithRustyHandlebars;

#[derive(WithRustyHandlebars)]
#[template(path = "examples/templates/more-involved.hbs")]
struct Profile<'a> {
    name: &'a str,
    age: u8,
}

let html = Profile {
    name: "Ada",
    age: 36,
}
.to_string();
```

The generated code is checked by the Rust compiler, so a missing field,
unsupported output type, or invalid ownership operation is reported while the
crate containing the template is compiled.

This crate implements a small, Rust-oriented template language. It is not a
runtime Handlebars interpreter and does not aim to be fully compatible with
Handlebars implementations in other languages.

## Setup

```toml
[dependencies]
rusty-handlebars = "0.3.0"
```

Add `#[derive(WithRustyHandlebars)]` to a struct and provide a template path:

```rust
use rusty_handlebars::WithRustyHandlebars;

#[derive(WithRustyHandlebars)]
#[template(path = "templates/email.rhbs")]
struct Email<'a> {
    recipient: &'a str,
    items: &'a [&'a str],
}
```

Paths are resolved from the Cargo workspace root when the deriving package is
a workspace member. Otherwise they are resolved from that package's manifest
directory. The template is read by the procedural macro and must exist when
the package is compiled.

`.rhbs` is the preferred extension for new Rusty Handlebars templates. It
distinguishes this Rust-oriented language from standard Handlebars tooling.
Existing `.hbs` paths remain fully supported by the compiler.

The derive implements:

- `Display`, which renders the template;
- `WithRustyHandlebars`, a marker for generated renderers;
- `AsDisplay`, so one rendered template can be inserted raw into another.

## Template syntax

| Template | Generated behavior |
| --- | --- |
| `{{value}}` | Calls `AsDisplayHtml` and writes the result |
| `{{{value}}}` | Calls `AsDisplay` and writes the result without escaping |
| `{{#if value}}…{{else}}…{{/if}}` | Tests `value` with `AsBool` |
| `{{#unless value}}…{{/unless}}` | Negated `AsBool` test |
| `{{#if_some value}}…{{/if_some}}` | Matches an `Option` and uses its value as `this` |
| `{{#with value}}…{{/with}}` | Uses `value` as `this` |
| `{{#each values}}…{{else}}…{{/each}}` | Iterates, with optional empty case |
| `{{lookup values index}}` | Generates indexing: `values[index]` |
| `{{try_lookup map key}}` | Generates a lookup: `map.get(key)` |
| `{{format "{:.2}" value}}` | Uses the supplied Rust format specifier |
| `{{! comment }}` | Emits nothing |

`if_some_ref`, `with_ref`, and `each_ref` borrow their input before matching,
binding, or iterating. Prefer these forms for owned fields because `Display::fmt`
only has `&self`; the forms without `_ref` are useful when the field is already
a reference or is `Copy`.

A block value is available as `this` unless it is named with either
`as name` or `as |name|`. Use `../` to resolve from a parent block:

```handlebars
{{#each_ref items as |item|}}
    {{@index}}: {{item}} from {{../recipient}}
{{else}}
    No items
{{/each_ref}}
```

Inside `each`, `@index` is the zero-based position. When iterating key/value
pairs, `@key` and `@value` address the pair members.

`~` next to a delimiter trims adjacent template whitespace. A backslash before
an opening delimiter suppresses interpolation and writes the content between
the delimiters without the braces. A four-brace raw block emits an entire
region without parsing it:

```handlebars
\{{not_an_expression}}
{{{{raw}}}}{{also_not_an_expression}}{{{{/raw}}}}
```

## Output traits and escaping

Double-brace interpolation requires `AsDisplayHtml`. It is implemented for
strings, numbers, booleans, `Option<T>`, `Box<T>`, and references when their
inner type also implements it. Strings escape `&`, `<`, `>`, and `"`.

Triple-brace interpolation requires `AsDisplay`. It is implemented for the
same scalar families and writes strings unchanged. Implement these traits for
application types that appear in templates:

```rust
use rusty_handlebars::{AsDisplay, AsDisplayHtml};

struct Amount(u32);

impl AsDisplay for Amount {
    fn as_display(&self) -> impl std::fmt::Display {
        self.0
    }
}

impl AsDisplayHtml for Amount {
    fn as_display_html(&self) -> impl std::fmt::Display {
        self.0
    }
}
```

Escaping is text-oriented, not context-aware. It does not make an arbitrary
value safe for JavaScript, CSS, URLs, or unquoted HTML attributes. Use
triple-brace interpolation only for content whose safety is established by the
application.

`if` and `unless` use `AsBool`. Built-in implementations treat zero numbers,
empty strings and collections, `'\0'`, `None`, `Err`, and `()` as false. For
`Some` and `Ok`, truthiness is delegated to the contained value.

## Inline helpers

The `helpers` attribute maps helper names in a template to Rust function paths:

```rust
#[derive(rusty_handlebars::WithRustyHandlebars)]
#[template(
    path = "templates/report.hbs",
    helpers = ["crate::helpers::format_date"]
)]
struct Report {
    created_at: u64,
}
```

`{{format_date created_at}}` then generates a call to
`crate::helpers::format_date(self.created_at)`. The helper name is the final
segment of its configured path. Arguments are template variables, literals, or
subexpressions, and the return type must implement the output trait required by
the surrounding braces.

## HTML minification

The derive crate includes HTML minification support by default. Each template
is minified unless its attribute sets `minify = false`:

```rust
#[derive(rusty_handlebars::WithRustyHandlebars)]
#[template(path = "templates/plain-text.hbs", minify = false)]
struct PlainTextEmail<'a> {
    body: &'a str,
}
```

The facade's default `minify-html` Cargo feature controls whether the minifier
dependency is compiled at all. Use `default-features = false` on the
`rusty-handlebars` dependency to omit it; in that configuration templates are
compiled without minification regardless of the attribute value.

The root crate's `parser` feature exposes `Compiler` and `Options` from the
low-level parser package. Applications using the derive macro do not need that
feature.

See [`examples`](examples) for templates covering nested data, options,
lookups, maps, formatting, and template composition.

## Editor support

The VS Code extension in [`editors/vscode`](editors/vscode) registers `.rhbs`
as the `rusty-handlebars` language and includes highlighting, snippets,
diagnostics, completion, hover information, symbols, folding, matching blocks,
Cargo context discovery, field definitions, and a **Show Generated Rust**
command.

The extension does not claim `.hbs` globally. For a legacy template, select
the Rusty Handlebars language mode manually or add a workspace-specific glob
to `rustyHandlebars.legacyFileGlobs`.

See [`VSCODE_EXTENSION_PLAN.md`](VSCODE_EXTENSION_PLAN.md) for the design,
implementation phases, known semantic limits, and release strategy.

The RustRover plugin in [`editors/jetbrains`](editors/jetbrains) provides the
same shared language intelligence together with a native `.rhbs` editor,
mixed HTML/template highlighting, Live Templates, project settings, safe
bundled-server extraction, generated Rust inspection, and language-service
restart/reload actions.

RustRover releases are assembled as one universal plugin ZIP containing
servers for macOS ARM64/x64, Linux ARM64/x64, and Windows x64. See the
[RustRover installation and development guide](editors/jetbrains/README.md)
and [`RUSTROVER_PLUGIN_PLAN.md`](RUSTROVER_PLUGIN_PLAN.md).

## License

MIT. See [`LICENSE.txt`](LICENSE.txt).
