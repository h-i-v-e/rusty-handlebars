# Rusty Handlebars Parser

A type-safe Handlebars template parser and compiler for Rust. This crate provides the core parsing and compilation functionality for the `rusty-handlebars` templating engine.

## Features

- Type-safe template parsing and compilation
- HTML escaping and safety
- Optional HTML minification
- Block helper support
- Expression evaluation
- Variable resolution and scope management
- Compile-time template validation

## Components

### Expression Parser
The expression parser handles various types of Handlebars expressions:
- Variables: `{{name}}`
- HTML-escaped variables: `{{{name}}}`
- Block helpers: `{{#helper}}...{{/helper}}`
- Comments: `{{! comment }}`
- Escaped content: `\{{name}}`

### Block Helpers
Built-in block helpers include:
- `if`/`unless` for conditional rendering
- `if_some`/`if_some_ref` for Option handling
- `with`/`with_ref` for context changes
- `each`/`each_ref` for collection iteration

### Compiler
The compiler transforms Handlebars templates into Rust code:
- Resolves variables and scopes
- Compiles block helpers
- Handles HTML escaping
- Generates type-safe code

## Usage

### Basic Template Compilation

```rust
use rusty_handlebars_parser::{Compiler, Options};
use rusty_handlebars_parser::block::add_builtins;

let mut block_map = HashMap::new();
add_builtins(&mut block_map);

let options = Options {
    root_var_name: Some("data"),
    write_var_name: "write"
};

let compiler = Compiler::new(options, block_map);
let rust = compiler.compile("Hello {{name}}!")?;
```

### Complex Template Example

```rust
let template = r#"
<div class="user-profile">
    {{#if user}}
        <h1>{{user.name}}</h1>
        {{#if user.bio}}
            <p class="bio">{{user.bio}}</p>
        {{else}}
            <p class="no-bio">No bio available</p>
        {{/if}}
        
        {{#if_some user.posts as post}}
            <div class="posts">
                <h2>Posts</h2>
                {{#each post as post}}
                    <article class="post">
                        <h3>{{post.title}}</h3>
                        <p>{{post.content}}</p>
                        <div class="meta">
                            <span>Posted on {{post.date}}</span>
                            {{#if post.tags}}
                                <div class="tags">
                                    {{#each post.tags as tag}}
                                        <span class="tag">{{tag}}</span>
                                    {{/each}}
                                </div>
                            {{/if}}
                        </div>
                    </article>
                {{/each}}
            </div>
        {{/if_some}}
    {{else}}
        <p>Please log in to view your profile</p>
    {{/if}}
</div>
"#;

let rust = compiler.compile(template)?;
```

## HTML Minification

When the `minify-html` feature is enabled, the parser can optimize HTML output:

```rust
#[cfg(feature = "minify-html")]
use rusty_handlebars_parser::build_helper::COMPRESS_CONFIG;
```

The minification configuration:
- Preserves Handlebars template syntax
- Maintains HTML validity
- Optimizes JavaScript and CSS
- Keeps essential HTML structure

## Error Handling

The parser provides detailed error information:

```rust
use rusty_handlebars_parser::error::{Result, ParseError};

fn compile_template(template: &str) -> Result<String> {
    let compiler = Compiler::new(options, block_map);
    compiler.compile(template)
}
```

## Module Structure

- `compiler.rs`: Core compilation logic
- `block.rs`: Block helper implementations
- `expression.rs`: Expression parsing
- `expression_tokenizer.rs`: Tokenization of expressions
- `error.rs`: Error handling
- `build_helper.rs`: Build-time configuration

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 