# Rusty Handlebars

A Rust implementation of Handlebars templating engine with a focus on type safety and compile-time template processing.

## Features

- **Type-safe templating**: Compile-time template processing with Rust's type system
- **HTML escaping**: Built-in HTML escaping for secure output
- **Optional HTML minification**: Reduce HTML output size with the `minify-html` feature
- **Derive macro support**: Easy integration with Rust structs using `#[derive(WithRustyHandlebars)]`
- **Flexible display traits**: Support for both regular and HTML-safe display implementations
- **Optional parser**: Template parsing functionality available via the `parser` feature

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rusty-handlebars = "0.1.0"
```

For HTML minification support, enable the `minify-html` feature:

```toml
[dependencies]
rusty-handlebars = { version = "0.1.0", features = ["minify-html"] }
```

## Usage

### Basic Template Usage

```rust
use rusty_handlebars::WithRustyHandlebars;

#[derive(WithRustyHandlebars)]
#[template(path = "templates/hello.hbs")]
struct HelloTemplate {
    name: String,
}
```

### HTML-Safe Output

```rust
use rusty_handlebars::AsDisplayHtml;

let html = "<script>alert('xss')</script>";
let safe_html = html.as_display_html().to_string();
// Output: &lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;
```

### Template Features

1. **Variables**: Access struct fields using `{{field_name}}`
2. **Conditionals**: Use `{{#if}}...{{/if}}` for conditional rendering
3. **Loops**: Iterate over collections with `{{#each}}...{{/each}}`
4. **Helpers**: Define custom helpers for complex logic
5. **Partials**: Include reusable template parts

## Features

### `parser` feature
Enables template parsing functionality. This is optional and can be enabled with:

```toml
[dependencies]
rusty-handlebars = { version = "0.1.10", features = ["parser"] }
```

### `minify-html` feature
Enables HTML minification for reduced output size:

```toml
[dependencies]
rusty-handlebars = { version = "0.1.10", features = ["minify-html"] }
```

## Safety

- All HTML output is automatically escaped by default
- Type-safe template processing prevents runtime errors
- Compile-time template validation
- No unsafe code in the main crate

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 