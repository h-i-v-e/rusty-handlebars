use crate::{expression::Expression, Diagnostic};
use std::{error::Error, fmt::Display};

/// A template parsing or source-generation error.
#[derive(Debug)]
pub struct ParseError {
    pub(crate) message: String,
}

pub(crate) fn rcap(src: &str) -> &str {
    static CAP_AT: usize = 32;

    if src.len() > CAP_AT {
        &src[src.len() - CAP_AT..]
    } else {
        src
    }
}

impl ParseError {
    pub(crate) fn new(message: &str, expression: &Expression<'_>) -> Self {
        Self {
            message: format!("{} near \"{}\"", message, expression.around()),
        }
    }

    pub(crate) fn unclosed(preffix: &str) -> Self {
        Self {
            message: format!("unclosed block near {}", rcap(preffix)),
        }
    }

    pub(crate) fn from_diagnostic(diagnostic: &Diagnostic) -> Self {
        Self {
            message: format!(
                "{} [{}] at bytes {}..{}",
                diagnostic.message, diagnostic.code, diagnostic.span.start, diagnostic.span.end
            ),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl Error for ParseError {}

/// Result returned by parser and compiler operations.
pub type Result<T> = std::result::Result<T, ParseError>;
