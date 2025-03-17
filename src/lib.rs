//! Rusty Handlebars - A type-safe Handlebars templating engine for Rust
//!
//! This crate provides a type-safe implementation of the Handlebars templating engine
//! with a focus on compile-time template processing and HTML safety.
//!
//! # Features
//!
//! - Type-safe templating with compile-time validation
//! - HTML escaping for secure output
//! - Optional HTML minification
//! - Derive macro support for easy integration
//! - Flexible display traits for both regular and HTML-safe output
//! - Optional parser functionality
//!
//! # Examples
//!
//! ```rust
//! use rusty_handlebars::WithRustyHandlebars;
//!
//! #[derive(WithRustyHandlebars)]
//! #[template(path = "templates/hello.hbs")]
//! struct HelloTemplate {
//!     name: String,
//! }
//! ```
//!
//! For HTML-safe output:
//!
//! ```rust
//! use rusty_handlebars::AsDisplayHtml;
//!
//! let html = "<script>alert('xss')</script>";
//! let safe_html = html.as_display_html().to_string();
//! // Output: &lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;
//! ```

use std::fmt::{Display, Write};

pub mod as_bool;
pub use as_bool::AsBool;

/// Derive macro for implementing Handlebars templating on structs
pub use rusty_handlebars_derive::WithRustyHandlebars;

#[cfg(feature = "parser")]
pub use rusty_handlebars_parser::{Compiler, Options};

/// Trait that must be implemented for types that can be used with Handlebars templates
pub trait WithRustyHandlebars : Display{}

macro_rules! impl_as_display {
    ($($t:ty),*) => {
        $(
            impl AsDisplay for $t{
                fn as_display(&self) -> impl Display {
                    self
                }
            }

            impl AsDisplay for &$t{
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

            impl AsDisplayHtml for &$t{
                fn as_display_html(&self) -> impl Display {
                    self
                }
            }
        )*
    }
}

/// Trait for converting values to a Display implementation
pub trait AsDisplay{
    /// Returns a type that implements Display
    fn as_display(&self) -> impl Display;
}

/// Trait for converting values to an HTML-safe Display implementation
pub trait AsDisplayHtml{
    /// Returns a type that implements Display with HTML escaping
    fn as_display_html(&self) -> impl Display;
}

impl_as_display!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, String, &str, bool);

impl_as_display_html!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool);

impl<T: AsDisplay> AsDisplay for Option<T> {
    fn as_display(&self) -> impl Display {
        match self{
            Some(t) => t.as_display().to_string(),
            None => "".as_display().to_string()
        }
    }
}

impl<T: AsDisplay> AsDisplay for &Option<T>{
    fn as_display(&self) -> impl Display {
        match self{
            Some(t) => t.as_display().to_string(),
            None => "".as_display().to_string()
        }
    }
}

impl<T: AsDisplay> AsDisplay for Box<T>{
    fn as_display(&self) -> impl Display {
        self.as_ref().as_display()
    }
}

impl<T: AsDisplay> AsDisplay for &Box<T>{
    fn as_display(&self) -> impl Display {
        self.as_ref().as_display()
    }
}

struct DisplayHtml<'a>{
    string: &'a str
}

impl<'a> Display for DisplayHtml<'a>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.string.chars(){
            match c{
                '&' => f.write_str("&amp;")?,
                '<' => f.write_str("&lt;")?,
                '>' => f.write_str("&gt;")?,
                '"' => f.write_str("&quot;")?,
                c => f.write_char(c)?
            }
        }
        Ok(())
    }
}

impl AsDisplayHtml for &str{
    fn as_display_html(&self) -> impl Display {
        DisplayHtml{string: self}
    }
}

impl AsDisplayHtml for &&str{
    fn as_display_html(&self) -> impl Display {
        DisplayHtml{string: *self}
    }
}

impl AsDisplayHtml for String{
    fn as_display_html(&self) -> impl Display {
        DisplayHtml{string: self.as_str()}
    }
}

impl<T: AsDisplayHtml> AsDisplayHtml for Option<T>{
    fn as_display_html(&self) -> impl Display {
        match self{
            Some(t) => t.as_display_html().to_string(),
            None => "".to_string()
        }
    }
}

impl<T: AsDisplayHtml> AsDisplayHtml for &Option<T>{
    fn as_display_html(&self) -> impl Display {
        match self{
            Some(t) => t.as_display_html().to_string(),
            None => "".to_string()
        }
    }
}

impl<T: AsDisplayHtml> AsDisplayHtml for Box<T>{
    fn as_display_html(&self) -> impl Display {
        self.as_ref().as_display_html()
    }
}

impl<T: AsDisplayHtml> AsDisplayHtml for &Box<T>{
    fn as_display_html(&self) -> impl Display {
        self.as_ref().as_display_html()
    }
}