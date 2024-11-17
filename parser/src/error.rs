use std::{error::Error, fmt::Display};
use crate::expression::Expression;

#[derive(Debug)]
pub struct ParseError{
    pub(crate) message: String
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
    pub(crate) fn new(message: &str, expression: &Expression<'_>) -> Self{
        Self{
            message: format!("{} near \"{}\"", message, expression.around())
        }
    }

    pub(crate) fn unclosed(preffix: &str) -> Self{
        Self{
            message: format!("unclosed block near {}", rcap(preffix))
        }
    }
}

impl Display for ParseError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<std::io::Error> for ParseError{
    fn from(err: std::io::Error) -> Self {
        Self{ message: err.to_string()}
    }
}

impl Error for ParseError{}

pub type Result<T> = std::result::Result<T, ParseError>;