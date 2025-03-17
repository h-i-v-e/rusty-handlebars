//! Handlebars expression tokenization
//!
//! This module provides functionality for tokenizing Handlebars expressions into their component parts.
//! It handles various token types including:
//! - Literals: Plain text values
//! - Private variables: Variables prefixed with @ (e.g. @index)
//! - Sub-expressions: Parenthesized expressions
//!
//! # Token Types
//!
//! ## Literals
//! Plain text values that are not special tokens:
//! ```
//! name
//! user.age
//! ```
//!
//! ## Private Variables
//! Variables prefixed with @ that have special meaning:
//! ```
//! @index
//! @first
//! @last
//! ```
//!
//! ## Sub-expressions
//! Parenthesized expressions that are evaluated first:
//! ```
//! (helper arg1 arg2)
//! (math.add 1 2)
//! ```
//!
//! # Examples
//!
//! ```rust
//! use rusty_handlebars_parser::expression_tokenizer::{Token, TokenType};
//!
//! let src = "user.name (helper arg) @index";
//! let token = Token::first(src).unwrap().unwrap();
//! assert_eq!(token.value, "user.name");
//! assert_eq!(token.token_type, TokenType::Literal);
//! ```

use crate::error::{rcap, ParseError, Result};

/// Types of tokens that can be parsed from an expression
#[derive(Clone)]
pub enum TokenType<'a> {
    /// A parenthesized sub-expression
    SubExpression(&'a str),
    /// A private variable prefixed with @
    PrivateVariable,
    /// A plain text literal
    Literal
}

/// A token parsed from an expression
#[derive(Clone)]
pub struct Token<'a> {
    /// The type of token
    pub token_type: TokenType<'a>,
    /// The token's value
    pub value: &'a str,
    /// The remaining text after this token
    pub tail: &'a str
}

/// Finds the closing parenthesis for a sub-expression
fn find_closing(src: &str) -> Result<usize> {
    let mut count = 1;
    let rest = &src[1..];
    for (i, c) in rest.char_indices() {
        match c {
            '(' => count += 1,
            ')' => count -= 1,
            _ => ()
        }
        if count == 0 {
            return Ok(i + 1);
        }
    }
    Err(ParseError{ message: format!("unmatched brackets near {}", rcap(src))})
}

/// Finds the end of a token by looking for whitespace or special characters
fn find_end(src: &str) -> usize {
    for (i, c) in src.char_indices() {
        if " (\n\r\t".contains(c) {
            return i
        }
    }
    src.len()
}

/// Parses a single token from the input string
fn parse<'a>(src: &'a str) -> Result<Option<Token<'a>>> {
    Ok(match src.chars().next() {
        Some('@') => {
            let end = find_end(src);
            Some(Token {
                token_type: TokenType::PrivateVariable,
                value: &src[1..end],
                tail: &src[end..].trim_start()
            })
        },
        Some('(') => {
            let end = find_closing(&src)?;
            Some(Token {
                token_type: TokenType::SubExpression(&src[..end]),
                value: &src[1..end],
                tail: &src[end + 1..].trim_start()
            })
        },
        None => None,
        _ => {
            let end = find_end(src);
            Some(Token {
                token_type: TokenType::Literal,
                value: &src[..end],
                tail: &src[end..].trim_start()
            })
        }
    })
}

impl<'a> Token<'a> {
    /// Parses the first token from a string
    pub fn first(src: &'a str) -> Result<Option<Self>> {
        parse(src.trim())
    }

    /// Parses the next token after this one
    pub fn next(&self) -> Result<Option<Self>> {
        parse(self.tail)
    }
}

