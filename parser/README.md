# rusty-handlebars-parser

This is the source-code generator used by the `rusty-handlebars` derive macro.
It parses the crate's Handlebars-like syntax and returns Rust statements that
write a rendered template into a formatter.

It is not a runtime renderer: `Compiler::compile` returns Rust source as a
`String`. Callers are responsible for placing that source in a suitable Rust
context.

```rust
use rusty_handlebars_parser::{add_builtins, BlockMap, Compiler, Options};

let mut blocks = BlockMap::new();
add_builtins(&mut blocks);

let compiler = Compiler::new(
    Options {
        root_var_name: Some("self"),
        write_var_name: "f",
    },
    blocks,
);

let rust = compiler.compile("Hello {{name}}!").unwrap();
assert!(rust.code.contains("self.name"));
```

`root_var_name` is prepended to root template variables. Set it to `None` when
those variables already exist in the generated code's local scope.
`write_var_name` names the `std::fmt::Write` destination used in generated
`write!` calls.

`add_builtins` installs the supported block helpers: `if`, `unless`,
`if_some`, `if_some_ref`, `with`, `with_ref`, `each`, and `each_ref`.
`Compiler::with_helper_paths` maps inline helper names to Rust function paths.

The complete template syntax is documented in the
[`rusty-handlebars` README](https://github.com/h-i-v-e/rusty-handlebars#readme).
