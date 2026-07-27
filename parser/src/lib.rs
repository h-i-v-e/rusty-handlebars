//! Compile Rusty Handlebars template syntax into Rust source statements.
//!
//! ```rust
//! use rusty_handlebars_parser::{add_builtins, BlockMap, Compiler, Options};
//!
//! let mut blocks = BlockMap::new();
//! add_builtins(&mut blocks);
//!
//! let compiler = Compiler::new(Options {
//!     write_var_name: "f",
//!     root_var_name: Some("self"),
//! }, blocks);
//!
//! let rust = compiler.compile("Hello {{name}}!").unwrap();
//! assert!(rust.code.contains("self.name"));
//! ```
//!
//! This crate returns source code; it does not render templates at runtime.
//! Most applications should use the `rusty-handlebars` derive macro instead.

mod block;
#[cfg(feature = "minify-html")]
pub mod build_helper;
mod compiler;
mod error;
mod expression;
mod expression_tokenizer;
mod syntax;

pub use block::*;
pub use compiler::*;
pub use error::*;
pub use expression::*;
pub use expression_tokenizer::*;
pub use syntax::*;

#[cfg(test)]
mod tests {
    use core::str;
    use std::collections::HashMap;

    use block::add_builtins;
    use compiler::{BlockMap, Compiler, Options};

    use crate::*;

    static OPTIONS: Options = Options {
        root_var_name: Some("self"),
        write_var_name: "f",
    };

    fn make_map() -> BlockMap {
        let mut map = BlockMap::new();
        add_builtins(&mut map);
        map
    }

    fn compile(src: &str) -> String {
        Compiler::new(OPTIONS, make_map())
            .compile(src)
            .unwrap()
            .code
    }

    #[test]
    fn it_works() {
        assert_eq!(
            compile("Hello {{{name}}}!"),
            "write!(f, \"Hello {}!\", ::rusty_handlebars::AsDisplay::as_display(&self.name))?;"
        );
    }

    #[test]
    fn test_if() {
        let rust = compile("{{#if some}}Hello{{/if}}");
        assert_eq!(
            rust,
            "if ::rusty_handlebars::AsBool::as_bool(&self.some){write!(f, \"Hello\")?;}"
        );
    }

    #[test]
    fn test_else() {
        let rust = compile("{{#if some}}Hello{{else}}World{{/if}}");
        assert_eq!(rust, "if ::rusty_handlebars::AsBool::as_bool(&self.some){write!(f, \"Hello\")?;}else{write!(f, \"World\")?;}");
    }

    #[test]
    fn test_unless() {
        let rust = compile("{{#unless some}}Hello{{/unless}}");
        assert_eq!(
            rust,
            "if !::rusty_handlebars::AsBool::as_bool(&self.some){write!(f, \"Hello\")?;}"
        );
    }

    #[test]
    fn test_each() {
        let rust = compile("{{#each some}}Hello {{this}}{{/each}}");
        assert_eq!(rust, "for this_1 in self.some{write!(f, \"Hello {}\", ::rusty_handlebars::AsDisplayHtml::as_display_html(&this_1))?;}");
    }

    #[test]
    fn test_each_else_with_comment() {
        let rust = compile("{{#each some}}{{! note }}{{this}}{{else}}empty{{/each}}");
        assert_eq!(rust, "{let mut _empty_1=true;for this_1 in self.some{_empty_1=false;write!(f, \"{}\", ::rusty_handlebars::AsDisplayHtml::as_display_html(&this_1))?;}if _empty_1{write!(f, \"empty\")?;}}");
    }

    #[test]
    fn test_each_ref_index_and_else() {
        let rust = compile("{{#each_ref some}}{{@index}}{{else}}empty{{/each_ref}}");
        assert_eq!(rust, "{let mut _empty_1=true;for (_index_1,this_1) in ::std::iter::IntoIterator::into_iter(&self.some).enumerate(){_empty_1=false;write!(f, \"{}\", ::rusty_handlebars::AsDisplayHtml::as_display_html(&_index_1))?;}if _empty_1{write!(f, \"empty\")?;}}");
    }

    #[test]
    fn test_with() {
        let rust = compile("{{#with some}}Hello {{name}}{{/with}}");
        assert_eq!(rust, "{let this_1 = self.some;write!(f, \"Hello {}\", ::rusty_handlebars::AsDisplayHtml::as_display_html(&this_1.name))?;}");
    }

    #[test]
    fn test_nesting() {
        let rust = compile("{{#if some}}{{#each some}}Hello {{this}}{{/each}}{{/if}}");
        assert_eq!(rust, "if ::rusty_handlebars::AsBool::as_bool(&self.some){for this_2 in self.some{write!(f, \"Hello {}\", ::rusty_handlebars::AsDisplayHtml::as_display_html(&this_2))?;}}");
    }

    #[test]
    fn test_as() {
        let rust = compile(
            "{{#if some}}{{#each some as thing}}Hello {{thing}} {{thing.name}}{{/each}}{{/if}}",
        );
        assert_eq!(rust, "if ::rusty_handlebars::AsBool::as_bool(&self.some){for thing_2 in self.some{write!(f, \"Hello {} {}\", ::rusty_handlebars::AsDisplayHtml::as_display_html(&thing_2), ::rusty_handlebars::AsDisplayHtml::as_display_html(&thing_2.name))?;}}");
    }

    #[test]
    fn test_comment() {
        let rust = compile("Note: {{! This is a comment }} and {{!-- {{so is this}} --}}\\{{{{}}");
        assert_eq!(rust, "write!(f, \"Note:  and {{{{\")?;");
    }

    #[test]
    fn test_scoping() {
        let rust = compile(
            "{{#with some}}{{#with other}}Hello {{name}} {{../company}} {{/with}}{{/with}}",
        );
        assert_eq!(rust, "{let this_1 = self.some;{let this_2 = this_1.other;write!(f, \"Hello {} {} \", ::rusty_handlebars::AsDisplayHtml::as_display_html(&this_2.name), ::rusty_handlebars::AsDisplayHtml::as_display_html(&this_1.company))?;}}");
    }

    #[test]
    fn test_trimming() {
        let rust = compile("  {{~#if some ~}}   Hello{{~/if~}}");
        assert_eq!(
            rust,
            "if ::rusty_handlebars::AsBool::as_bool(&self.some){write!(f, \"Hello\")?;}"
        );
    }

    #[test]
    fn test_indexer() {
        let rust = compile("{{#each things}}Hello{{{@index}}}{{#each things}}{{{lookup other @../index}}}{{{@index}}}{{/each}}{{/each}}");
        assert_eq!(rust, "for (_index_1,this_1) in ::std::iter::IntoIterator::into_iter(self.things).enumerate(){write!(f, \"Hello{}\", ::rusty_handlebars::AsDisplay::as_display(&_index_1))?;for (_index_2,this_2) in ::std::iter::IntoIterator::into_iter(this_1.things).enumerate(){write!(f, \"{}{}\", ::rusty_handlebars::AsDisplay::as_display(&this_2.other[_index_1]), ::rusty_handlebars::AsDisplay::as_display(&_index_2))?;}}");
    }

    #[test]
    fn test_map() {
        let rust = compile("{{#each things}}Hello{{{@key}}}{{#each @value}}{{#if_some (try_lookup other @../key)}}{{{this}}}{{/if_some}}{{{@value}}}{{/each}}{{/each}}");
        assert_eq!(rust, "for this_1 in self.things{write!(f, \"Hello{}\", ::rusty_handlebars::AsDisplay::as_display(&this_1.0))?;for this_2 in this_1.1{if let Some(this_3) = this_2.other.get(this_1.0){write!(f, \"{}\", ::rusty_handlebars::AsDisplay::as_display(&this_3))?;}write!(f, \"{}\", ::rusty_handlebars::AsDisplay::as_display(&this_2.1))?;}}");
    }

    #[test]
    fn test_literals() {
        let rust = compile("{{#if_some (try_lookup thing \"test\")}}{{this}}{{/if_some}} {{#if_some (try_lookup other_thing 123)}}{{this}}{{/if_some}}");
        assert_eq!(rust, "if let Some(this_1) = self.thing.get(\"test\"){write!(f, \"{}\", ::rusty_handlebars::AsDisplayHtml::as_display_html(&this_1))?;}write!(f, \" \")?;if let Some(this_1) = self.other_thing.get(123){write!(f, \"{}\", ::rusty_handlebars::AsDisplayHtml::as_display_html(&this_1))?;}");
    }

    #[test]
    fn test_subexpression() {
        let rust = compile("{{#each things}}{{#with (lookup ../other @index) as |other|}}{{{../name}}}: {{{other}}}{{/with}}{{/each}}");
        assert_eq!(rust, "for (_index_1,this_1) in ::std::iter::IntoIterator::into_iter(self.things).enumerate(){{let other_2 = self.other[_index_1];write!(f, \"{}: {}\", ::rusty_handlebars::AsDisplay::as_display(&this_1.name), ::rusty_handlebars::AsDisplay::as_display(&other_2))?;}}");
    }

    #[test]
    fn test_selfless() {
        let rust = Compiler::new(Options{
            root_var_name: None,
            write_var_name: "f"
        }, make_map()).compile("{{#each things}}{{#with (lookup ../other @index) as |other|}}{{{../name}}}: {{{other}}}{{/with}}{{/each}}").unwrap();
        assert_eq!(rust.code, "for (_index_1,this_1) in ::std::iter::IntoIterator::into_iter(things).enumerate(){{let other_2 = other[_index_1];write!(f, \"{}: {}\", ::rusty_handlebars::AsDisplay::as_display(&this_1.name), ::rusty_handlebars::AsDisplay::as_display(&other_2))?;}}");
    }

    #[test]
    fn javascript() {
        let rust = Compiler::new(OPTIONS, make_map()).compile("<script>if (location.href.contains(\"localhost\")){ console.log(\"\\{{{{}}}}\") }</script>").unwrap();
        assert_eq!(rust.code, "write!(f, \"<script>if (location.href.contains(\\\"localhost\\\")){{ console.log(\\\"{{{{}}}}\\\") }}</script>\")?;");
    }

    #[test]
    fn if_some() {
        let rust = compile("{{#if_some some}}Hello {{name}}{{else}}Oh dear{{/if_some}}{{#if some}}{{#if_some_ref ../some as |other|}}Hello {{other.name}}{{/if_some}}{{/if}}");
        assert_eq!(rust, "if let Some(this_1) = self.some{write!(f, \"Hello {}\", ::rusty_handlebars::AsDisplayHtml::as_display_html(&this_1.name))?;}else{write!(f, \"Oh dear\")?;}if ::rusty_handlebars::AsBool::as_bool(&self.some){if let Some(other_2) = &self.some{write!(f, \"Hello {}\", ::rusty_handlebars::AsDisplayHtml::as_display_html(&other_2.name))?;}}");
    }

    #[test]
    fn test_escaped() {
        let rust = compile("{{{{skip}}}}wang doodle {{{{/dandy}}}}{{{{/skip}}}}");
        assert_eq!(rust, "write!(f, \"wang doodle {{{{{{{{/dandy}}}}}}}}\")?;");
    }

    #[test]
    fn test_format_number() {
        let rust = compile("Price: ${{format \"{:.2}\" price}}");
        assert_eq!(rust, "write!(f, \"Price: ${:.2}\", self.price)?;");
    }

    #[test]
    fn test_qualified_helper() {
        let helper_paths = HashMap::from([(
            "capitalize".to_string(),
            "::rusty_handlebars::helpers::capitalize".to_string(),
        )]);
        let rust = Compiler::new(OPTIONS, make_map())
            .with_helper_paths(helper_paths)
            .compile("{{capitalize name}}")
            .unwrap();
        assert_eq!(
            rust.code,
            "write!(f, \"{}\", ::rusty_handlebars::AsDisplayHtml::as_display_html(&::rusty_handlebars::helpers::capitalize(self.name)))?;"
        );
    }
}
