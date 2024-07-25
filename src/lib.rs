use std::fmt::{Display, Write};

pub use as_bool::AsBool;
pub use rusty_handlebars_derive::DisplayAsHtml;
#[cfg(feature = "parser")]
pub use rusty_handlebars_parser::Compiler;

pub trait DisplayAsHtml : Display{}

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

pub trait AsDisplay{
    fn as_display(&self) -> impl Display;
}

pub trait AsDisplayHtml{
    fn as_display_html(&self) -> impl Display;
}

impl_as_display!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, String, &str, bool);

impl_as_display_html!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool);

impl<T: AsDisplay> AsDisplay for Option<T>{
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

/*struct DisplayAsHtmlAsDisplayHtml<'a, T: DisplayAsHtml>{
    item: &'a T
}*/

/*impl<'a, T: DisplayAsHtml> Display for DisplayAsHtmlAsDisplayHtml<'a, T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.item.to_string().as_display_html().fmt(f)
    }
}*/

/*impl <T: DisplayAsHtml> AsDisplayHtml for T{
    fn as_display_html(&self) -> impl Display {
        DisplayAsHtmlAsDisplayHtml{item: self}
    }
}*/

impl AsDisplayHtml for String{
    fn as_display_html(&self) -> impl Display {
        DisplayHtml{string: self.as_str()}
    }
}

impl<T: AsDisplayHtml> AsDisplayHtml for Option<T>{
    fn as_display_html(&self) -> impl Display {
        match self{
            Some(t) => t.as_display_html().to_string(),
            None => "".as_display_html().to_string()
        }
    }
}

impl<T: AsDisplayHtml> AsDisplayHtml for &Option<T>{
    fn as_display_html(&self) -> impl Display {
        match self{
            Some(t) => t.as_display_html().to_string(),
            None => "".as_display_html().to_string()
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

pub fn as_text<T: AsDisplay>(item: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    item.as_display().fmt(f)
}

pub fn as_html<T: AsDisplayHtml>(item: &T, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    item.as_display_html().fmt(f)
}