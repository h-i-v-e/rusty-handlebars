use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct ParseError{
    message: String
}

pub(crate) fn rcap<'a>(src: &'a str) -> &'a str{
    static CAP_AT: usize = 32;

    if src.len() > CAP_AT{
        &src[src.len() - CAP_AT ..]
    } else {
        src
    }
}

impl ParseError{
    pub(crate) fn new(message: String) -> Self{
        Self{
            message
        }
    }

    pub fn unclosed(preffix: &str) -> Self{
        Self::new(format!("Unclosed block near {}", rcap(preffix)))
    }
}

impl Display for ParseError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<std::io::Error> for ParseError{
    fn from(err: std::io::Error) -> Self {
        Self::new(err.to_string())
    }
}

impl Error for ParseError{}

pub type Result<T> = std::result::Result<T, ParseError>;

macro_rules! parse_error_near{
    ($near:ident, $pattern:literal,$($rest:ident,)*) => {
        Err(ParseError::new(format!(concat!($pattern, " near {}"), $($arg)*rcap($near))))
    };
    ($near:ident, $pattern:literal) => {
        Err(ParseError::new(format!(concat!($pattern, " near {}"), rcap($near))))
    };
}

pub(crate) use parse_error_near;