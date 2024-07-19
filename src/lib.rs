use std::fmt::Display;

pub use as_bool::AsBool;
pub use rusty_handlebars_derive::ToHtml;
#[cfg(feature = "parser")]
pub use rusty_handlebars_parser::Compiler;

pub trait ToHtml : Display{}

pub trait HtmlEscaped{
    fn html_escape(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl HtmlEscaped for str{
    fn html_escape(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.chars(){
            match c{
                '&' => write!(f, "&amp;")?,
                '<' => write!(f, "&lt;")?,
                '>' => write!(f, "&gt;")?,
                c => write!(f, "{}", c)?
            }
        }
        Ok(())
    }
}

impl<T: ToString> HtmlEscaped for T{
    fn html_escape(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_string().as_str().html_escape(f)
    }
}

pub fn html_escape<T: HtmlEscaped>(item: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    item.html_escape(f)
}
