//! Compile Handlebars-like template files into Rust [`Display`] implementations.
//!
//! [`WithRustyHandlebars`] reads the template named by `#[template(path = "...")]`
//! while the deriving crate is compiled. Template variables are compiled as field
//! accesses on the deriving struct, and rendering writes directly to the supplied
//! formatter.
//!
//! ```rust
//! use rusty_handlebars::WithRustyHandlebars;
//!
//! #[derive(WithRustyHandlebars)]
//! #[template(path = "examples/templates/more-involved.rhbs")]
//! struct Profile<'a> {
//!     name: &'a str,
//!     age: u8,
//! }
//!
//! let output = Profile { name: "Ada", age: 36 }.to_string();
//! assert!(output.contains("Ada"));
//! ```
//!
//! `{{value}}` uses [`AsDisplayHtml`]. `{{{value}}}` uses [`AsDisplay`] and
//! writes its result unchanged. The string implementation of [`AsDisplayHtml`]
//! escapes `&`, `<`, `>`, and `"`, but is not contextual escaping for URLs,
//! JavaScript, CSS, or unquoted attributes.
//!
//! ```rust
//! use rusty_handlebars::AsDisplayHtml;
//!
//! assert_eq!(
//!     "<strong title=\"x\">&</strong>".as_display_html().to_string(),
//!     "&lt;strong title=&quot;x&quot;&gt;&amp;&lt;/strong&gt;"
//! );
//! ```
//!
//! This is a Rust-oriented source generator, not a runtime Handlebars
//! interpreter. See the project README for the supported template syntax.

extern crate self as rusty_handlebars;

use std::fmt::Display;

pub mod as_bool;
pub use as_bool::AsBool;

/// Derives a template-backed [`Display`] implementation.
///
/// The required `template` attribute accepts `path`, plus optional `minify`
/// and `helpers` arguments. See the derive crate documentation for details.
pub use rusty_handlebars_derive::WithRustyHandlebars;

#[cfg(feature = "parser")]
pub use rusty_handlebars_parser::{Compiler, Options};

/// Marker implemented by [`WithRustyHandlebars`] for generated renderers.
pub trait WithRustyHandlebars: Display {}

macro_rules! impl_as_display {
    ($($t:ty),*) => {
        $(
            impl AsDisplay for $t{
                fn as_display(&self) -> impl Display {
                    self
                }
            }
        )*
    }
}

macro_rules! impl_as_display_html {
    ($($t:ty),*) => {
        $(
            impl AsDisplayHtml for $t{
                fn as_display_html(&self) -> impl Display {
                    self
                }
            }
        )*
    }
}

/// Supplies the value written by triple-brace interpolation.
///
/// Implement this for application types used as `{{{value}}}`. The result is
/// written without modification by this crate.
pub trait AsDisplay {
    /// Returns the value to write.
    fn as_display(&self) -> impl Display;
}

/// Supplies the value written by double-brace interpolation.
///
/// String implementations escape `&`, `<`, `>`, and `"`. Implementations for
/// other application types define their own escaping behavior.
pub trait AsDisplayHtml {
    /// Returns the value to write for `{{value}}`.
    fn as_display_html(&self) -> impl Display;
}

impl<T: AsDisplay + ?Sized> AsDisplay for &T {
    fn as_display(&self) -> impl Display {
        AsDisplay::as_display(*self)
    }
}

impl<T: AsDisplayHtml + ?Sized> AsDisplayHtml for &T {
    fn as_display_html(&self) -> impl Display {
        AsDisplayHtml::as_display_html(*self)
    }
}

impl_as_display!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, String, &str, bool
);

impl_as_display_html!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool
);

struct DisplayOption<'a, T> {
    value: &'a Option<T>,
}

impl<T: AsDisplay> Display for DisplayOption<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Some(value) => Display::fmt(&value.as_display(), f),
            None => Ok(()),
        }
    }
}

impl<T: AsDisplay> AsDisplay for Option<T> {
    fn as_display(&self) -> impl Display {
        DisplayOption { value: self }
    }
}

impl<T: AsDisplay> AsDisplay for Box<T> {
    fn as_display(&self) -> impl Display {
        self.as_ref().as_display()
    }
}

struct DisplayHtml<'a> {
    string: &'a str,
}

impl<'a> Display for DisplayHtml<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut start = 0;
        for (index, byte) in self.string.bytes().enumerate() {
            let escaped = match byte {
                b'&' => "&amp;",
                b'<' => "&lt;",
                b'>' => "&gt;",
                b'"' => "&quot;",
                _ => continue,
            };
            if start < index {
                f.write_str(&self.string[start..index])?;
            }
            f.write_str(escaped)?;
            start = index + 1;
        }
        if start < self.string.len() {
            f.write_str(&self.string[start..])?;
        }
        Ok(())
    }
}

impl AsDisplayHtml for &str {
    fn as_display_html(&self) -> impl Display {
        DisplayHtml { string: self }
    }
}

impl AsDisplayHtml for String {
    fn as_display_html(&self) -> impl Display {
        DisplayHtml {
            string: self.as_str(),
        }
    }
}

struct DisplayOptionHtml<'a, T> {
    value: &'a Option<T>,
}

impl<T: AsDisplayHtml> Display for DisplayOptionHtml<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Some(value) => Display::fmt(&value.as_display_html(), f),
            None => Ok(()),
        }
    }
}

impl<T: AsDisplayHtml> AsDisplayHtml for Option<T> {
    fn as_display_html(&self) -> impl Display {
        DisplayOptionHtml { value: self }
    }
}

impl<T: AsDisplayHtml> AsDisplayHtml for Box<T> {
    fn as_display_html(&self) -> impl Display {
        self.as_ref().as_display_html()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(WithRustyHandlebars)]
    #[template(path = "examples/templates/hello-world.rhbs")]
    struct TestTemplate<'a> {
        message: &'a [&'a str],
    }

    #[derive(WithRustyHandlebars)]
    #[template(path = "examples/templates/generic.rhbs", minify = false)]
    struct GenericTemplate<'a, T, const N: usize>
    where
        T: AsDisplayHtml,
    {
        value: &'a T,
        marker: [u8; N],
    }

    #[derive(WithRustyHandlebars)]
    #[template(path = "examples/templates/each-else.rhbs", minify = false)]
    struct EachElseTemplate<'a> {
        values: Vec<&'a str>,
    }

    #[test]
    fn test_with_rusty_handlebars() {
        assert!(!TestTemplate {
            message: &["Hello", "World!"],
        }
        .to_string()
        .is_empty())
    }

    #[test]
    fn options_preserve_display_behavior() {
        assert_eq!(Some("raw").as_display().to_string(), "raw");
        assert_eq!(None::<&str>.as_display().to_string(), "");
        assert_eq!(
            Some("<one> & \"two\"").as_display_html().to_string(),
            "&lt;one&gt; &amp; &quot;two&quot;"
        );
        assert_eq!(None::<&str>.as_display_html().to_string(), "");
    }

    #[test]
    fn derive_supports_bounded_and_const_generics() {
        let value = "generic";
        let template = GenericTemplate {
            value: &value,
            marker: [],
        };
        assert_eq!(template.to_string(), "generic\n");
        assert!(template.marker.is_empty());
    }

    #[test]
    fn each_else_renders_both_branches() {
        assert_eq!(EachElseTemplate { values: vec![] }.to_string(), "empty\n");
        assert_eq!(
            EachElseTemplate {
                values: vec!["one", "two"],
            }
            .to_string(),
            "onetwo\n"
        );
    }
}
