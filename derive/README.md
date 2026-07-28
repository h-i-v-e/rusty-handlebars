# rusty-handlebars-derive

This package contains the `WithRustyHandlebars` procedural macro used by
[`rusty-handlebars`](https://crates.io/crates/rusty-handlebars). Applications
should normally depend on the facade crate instead of importing this package
directly.

The macro reads a template during compilation, translates its expressions into
Rust, and implements `Display`, `rusty_handlebars::WithRustyHandlebars`, and
`rusty_handlebars::AsDisplay` for the deriving struct.

```rust
use rusty_handlebars::WithRustyHandlebars;

#[derive(WithRustyHandlebars)]
#[template(path = "templates/email.rhbs")]
struct Email<'a> {
    recipient: &'a str,
}
```

The `template` attribute accepts:

- `path = "..."`, which is required;
- `minify = true | false`, which defaults to `true`;
- `helpers = ["crate::path::to_helper", ...]`, which maps inline helper names
  to Rust function paths.

The crate's default `minify-html` feature can be disabled to remove the
minifier dependency. Without that feature, templates are compiled unchanged
even when `minify` is omitted or set to `true`.

When the deriving package is a Cargo workspace member, `path` is relative to
the workspace root. Otherwise it is relative to the package manifest
directory.

The template language and output traits are documented in the
[`rusty-handlebars` README](https://github.com/h-i-v-e/rusty-handlebars#readme).
