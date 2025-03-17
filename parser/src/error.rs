//! Error handling for the Handlebars parser
//!
//! This module provides error types and handling for the template parsing process.
//! It includes detailed error messages with context about where parsing errors occurred.

use std::{error::Error, fmt::Display};
use crate::expression::Expression;

/// Error type for template parsing failures
///
/// This error type provides detailed information about parsing errors,
/// including the location and context of the error.
#[derive(Debug)]
pub struct ParseError{
    pub(crate) message: String
}

/// Returns the last 32 characters of a string for error context
pub(crate) fn rcap<'a>(src: &'a str) -> &'a str{
    static CAP_AT: usize = 32;

    if src.len() > CAP_AT{
        &src[src.len() - CAP_AT ..]
    } else {
        src
    }
}

impl ParseError{
    /// Creates a new parse error with context from an expression
    pub(crate) fn new(message: &str, expression: &Expression<'_>) -> Self{
        Self{
            message: format!("{} near \"{}\"", message, expression.around())
        }
    }

    /// Creates an error for unclosed blocks
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

/// Result type for template parsing operations
pub type Result<T> = std::result::Result<T, ParseError>;