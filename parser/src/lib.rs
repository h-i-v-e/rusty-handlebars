//! Handlebars template parser and compiler
//!
//! This crate provides the core functionality for parsing and compiling Handlebars templates
//! into Rust code. It's used internally by the `rusty-handlebars` crate to process templates
//! at compile time.
//!
//! # Features
//!
//! - Handlebars template parsing
//! - Template compilation to Rust code
//! - Support for all standard Handlebars features:
//!   - Variables and expressions
//!   - Block helpers (if, unless, each, with)
//!   - Partials
//!   - Comments
//!   - HTML escaping
//!   - Whitespace control
//!   - Subexpressions
//!   - Lookup helpers
//!
//! # Example
//!
//! ```rust
//! use rusty_handlebars_parser::{Compiler, Options, BlockMap};
//! use rusty_handlebars_parser::block::add_builtins;
//!
//! let mut factories = BlockMap::new();
//! add_builtins(&mut factories);
//!
//! let compiler = Compiler::new(Options {
//!     write_var_name: "f",
//!     root_var_name: Some("self")
//! }, factories);
//!
//! let template = "Hello {{name}}!";
//! let rust_code = compiler.compile(template).unwrap();
//! ```
//!
//! # Module Structure
//!
//! - `compiler.rs`: Main compiler implementation
//! - `block.rs`: Block helper implementations
//! - `expression.rs`: Expression parsing and evaluation
//! - `expression_tokenizer.rs`: Tokenization of expressions
//! - `error.rs`: Error types and handling
//! - `build_helper.rs`: Helper functions for template building

mod error;
mod expression;
mod expression_tokenizer;
mod compiler;
mod block;
pub mod build_helper;

pub use compiler::*;
pub use block::*;
pub use error::*;
pub use expression::*;
pub use expression_tokenizer::*;

#[cfg(test)]
mod tests {
    use core::str;

    use block::add_builtins;
    use compiler::{BlockMap, Compiler, Options};

    use crate::*;

    static OPTIONS: Options = Options{
        root_var_name: Some("self"),
        write_var_name: "f"
    };

    fn make_map() -> BlockMap{
        let mut map = BlockMap::new();
        add_builtins(&mut map);
        map
    }

    fn compile(src: &str) -> String{
        Compiler::new(OPTIONS, make_map()).compile(src).unwrap().code
    }

    #[test]
    fn it_works() {
        assert_eq!(
            compile("Hello {{{name}}}!"),
            "write!(f, \"Hello {}!\", self.name.as_display())?;"
        );
    }

    #[test]
    fn test_if(){
        let rust = compile("{{#if some}}Hello{{/if}}");
        assert_eq!(rust, "if self.some.as_bool(){write!(f, \"Hello\")?;}");
    }

    #[test]
    fn test_else(){
        let rust = compile("{{#if some}}Hello{{else}}World{{/if}}");
        assert_eq!(rust, "if self.some.as_bool(){write!(f, \"Hello\")?;}else{write!(f, \"World\")?;}");
    }

    #[test]
    fn test_unless(){
        let rust = compile("{{#unless some}}Hello{{/unless}}");
        assert_eq!(rust, "if !self.some.as_bool(){write!(f, \"Hello\")?;}");
    }

    #[test]
    fn test_each(){
        let rust = compile("{{#each some}}Hello {{this}}{{/each}}");
        assert_eq!(rust, "for this_1 in self.some{write!(f, \"Hello {}\", this_1.as_display_html())?;}");
    }

    #[test]
    fn test_with(){
        let rust = compile("{{#with some}}Hello {{name}}{{/with}}");
        assert_eq!(rust, "{let this_1 = self.some;write!(f, \"Hello {}\", this_1.name.as_display_html())?;}");
    }

    #[test]
    fn test_nesting(){
        let rust = compile("{{#if some}}{{#each some}}Hello {{this}}{{/each}}{{/if}}");
        assert_eq!(rust, "if self.some.as_bool(){for this_2 in self.some{write!(f, \"Hello {}\", this_2.as_display_html())?;}}");
    }

    #[test]
    fn test_as(){
        let rust = compile("{{#if some}}{{#each some as thing}}Hello {{thing}} {{thing.name}}{{/each}}{{/if}}");
        assert_eq!(rust, "if self.some.as_bool(){for thing_2 in self.some{write!(f, \"Hello {} {}\", thing_2.as_display_html(), thing_2.name.as_display_html())?;}}");
    }

    #[test]
    fn test_comment(){
        let rust = compile("Note: {{! This is a comment }} and {{!-- {{so is this}} --}}\\{{{{}}");
        assert_eq!(rust, "write!(f, \"Note:  and {{{{\")?;");
    }

    #[test]
    fn test_scoping(){
        let rust = compile("{{#with some}}{{#with other}}Hello {{name}} {{../company}} {{/with}}{{/with}}");
        assert_eq!(rust, "{let this_1 = self.some;{let this_2 = this_1.other;write!(f, \"Hello {} {} \", this_2.name.as_display_html(), this_1.company.as_display_html())?;}}");
    }

    #[test]
    fn test_trimming(){
        let rust = compile("  {{~#if some ~}}   Hello{{~/if~}}");
        assert_eq!(rust, "if self.some.as_bool(){write!(f, \"Hello\")?;}");
    }

    #[test]
    fn test_indexer(){
        let rust = compile("{{#each things}}Hello{{{@index}}}{{#each things}}{{{lookup other @../index}}}{{{@index}}}{{/each}}{{/each}}");
        assert_eq!(rust, "let mut i_1 = 0;for this_1 in self.things{write!(f, \"Hello{}\", i_1.as_display())?;let mut i_2 = 0;for this_2 in this_1.things{write!(f, \"{}{}\", this_2.other[i_1].as_display(), i_2.as_display())?;i_2+=1;}i_1+=1;}");
    }

    #[test]
    fn test_map(){
        let rust = compile("{{#each things}}Hello{{{@key}}}{{#each @value}}{{#if_some (try_lookup other @../key)}}{{{this}}}{{/if_some}}{{{@value}}}{{/each}}{{/each}}");
        assert_eq!(rust, "for this_1 in self.things{write!(f, \"Hello{}\", this_1.0.as_display())?;for this_2 in this_1.1{if let Some(this_3) = this_2.other.get(this_1.0){write!(f, \"{}\", this_3.as_display())?;}write!(f, \"{}\", this_2.1.as_display())?;}}");
    }

    #[test]
    fn test_literals(){
        let rust = compile("{{#if_some (try_lookup thing \"test\")}}{{this}}{{/if_some}} {{#if_some (try_lookup other_thing 123)}}{{this}}{{/if_some}}");
        assert_eq!(rust, "if let Some(this_1) = self.thing.get(\"test\"){write!(f, \"{}\", this_1.as_display_html())?;}write!(f, \" \")?;if let Some(this_1) = self.other_thing.get(123){write!(f, \"{}\", this_1.as_display_html())?;}");
    }

    #[test]
    fn test_subexpression(){
        let rust = compile("{{#each things}}{{#with (lookup ../other @index) as |other|}}{{{../name}}}: {{{other}}}{{/with}}{{/each}}");
        assert_eq!(rust, "let mut i_1 = 0;for this_1 in self.things{{let other_2 = self.other[i_1];write!(f, \"{}: {}\", this_1.name.as_display(), other_2.as_display())?;}i_1+=1;}");
    }

    #[test]
    fn test_selfless(){
        let rust = Compiler::new(Options{
            root_var_name: None,
            write_var_name: "f"
        }, make_map()).compile("{{#each things}}{{#with (lookup ../other @index) as |other|}}{{{../name}}}: {{{other}}}{{/with}}{{/each}}").unwrap();
        assert_eq!(rust.uses().to_string(), "use rusty_handlebars::AsDisplay");
        assert_eq!(rust.code, "let mut i_1 = 0;for this_1 in things{{let other_2 = other[i_1];write!(f, \"{}: {}\", this_1.name.as_display(), other_2.as_display())?;}i_1+=1;}");
    }

    #[test]
    fn javascript(){
        let rust = Compiler::new(OPTIONS, make_map()).compile("<script>if (location.href.contains(\"localhost\")){ console.log(\"\\{{{{}}}}\") }</script>").unwrap();
        assert_eq!(rust.uses().to_string(), "");
        assert_eq!(rust.code, "write!(f, \"<script>if (location.href.contains(\\\"localhost\\\")){{ console.log(\\\"{{{{}}}}\\\") }}</script>\")?;");
    }

    #[test]
    fn if_some(){
        let rust = compile("{{#if_some some}}Hello {{name}}{{else}}Oh dear{{/if_some}}{{#if some}}{{#if_some_ref ../some as |other|}}Hello {{other.name}}{{/if_some}}{{/if}}");
        assert_eq!(rust, "if let Some(this_1) = self.some{write!(f, \"Hello {}\", this_1.name.as_display_html())?;}else{write!(f, \"Oh dear\")?;}if self.some.as_bool(){if let Some(other_2) = &self.some{write!(f, \"Hello {}\", other_2.name.as_display_html())?;}}");
    }

    #[test]
    fn test_escaped(){
        let rust = compile("{{{{skip}}}}wang doodle {{{{/dandy}}}}{{{{/skip}}}}");
        assert_eq!(rust, "write!(f, \"wang doodle {{{{{{{{/dandy}}}}}}}}\")?;");
    }

    #[test]
    fn test_format_number(){
        let rust = compile("Price: ${{format \"{:.2}\" price}}");
        assert_eq!(rust, "write!(f, \"Price: ${:.2}\", self.price)?;");
    }
}