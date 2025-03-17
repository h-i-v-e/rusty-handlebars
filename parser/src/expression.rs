//! Handlebars expression parsing
//!
//! This module provides functionality for parsing Handlebars expressions from template strings.
//! It handles various types of expressions including variables, blocks, comments, and escaped content.
//!
//! # Expression Types
//!
//! The module supports the following types of expressions:
//! - Variables: `{{name}}`
//! - HTML-escaped variables: `{{{name}}}`
//! - Block helpers: `{{#helper}}...{{/helper}}`
//! - Comments: `{{! comment }}` or `{{!-- comment --}}`
//! - Escaped content: `\{{name}}` or `{{{{name}}}}this bit here is not parsed {{not_interpolated}} and output raw{{{{/name}}}}`
//!
//! # Examples
//!
//! ```rust
//! use rusty_handlebars_parser::expression::{Expression, ExpressionType};
//!
//! let template = "Hello {{name}}!";
//! let expr = Expression::from(template).unwrap().unwrap();
//! assert_eq!(expr.expression_type, ExpressionType::HtmlEscaped);
//! assert_eq!(expr.content, "name");
//! ```

use std::{cmp::min, fmt::Display};

use crate::error::{Result, ParseError};

/// Types of Handlebars expressions
#[derive(Debug, Clone, Copy)]
pub enum ExpressionType{
    /// Comment expression: `{{! comment }}`
    Comment, HtmlEscaped, Raw, Open, Close, Escaped
}

/// Represents a parsed Handlebars expression
#[derive(Debug, Clone, Copy)]
pub struct Expression<'a>{
    /// The type of expression
    pub expression_type: ExpressionType,
    /// Text before the expression
    pub prefix: &'a str,
    /// The expression content
    pub content: &'a str,
    /// Text after the expression
    pub postfix: &'a str,
    /// The complete expression including delimiters
    pub raw: &'a str
}

/// Safely extracts a substring of specified length
#[inline]
fn nibble(src: &str, start: usize, len: usize) -> Result<usize>{
    let end = start + len; 
    if end >= src.len(){
        return Err(ParseError::unclosed(src));
    }
    Ok(end)
}

impl<'a> Expression<'a>{
    /// Creates a new expression by finding its closing delimiter
    fn close(expression_type: ExpressionType, preffix: &'a str, start: &'a str, end: &'static str) -> Result<Self>{
        match start.find(end){
            Some(mut pos) => {
                if pos == 0{
                    return Err(ParseError { message: format!("empty block near {}", preffix) });
                }
                let mut postfix = &start[pos + end.len() ..];
                if &start[pos - 1 .. pos] == "~"{
                    postfix = postfix.trim_start();
                    pos -= 1;
                } 
                Ok(Self { expression_type, prefix: preffix, content: &start[.. pos], postfix, raw: &start[.. pos + end.len()] })
            },
            None => Err(ParseError::unclosed(preffix))
        }
    }

    /// Parses a comment expression
    fn check_comment(preffix: &'a str, start: &'a str) -> Result<Self>{
        if let Some(pos) = start.find("--"){
            if pos == 0{
                return Self::close(ExpressionType::Comment, preffix, &start[2 ..], "--}}");
            }
        }
        Self::close(ExpressionType::Comment, preffix, start, "}}")
    }

    /// Finds the closing delimiter for an escaped expression
    fn find_closing_escape(open: Expression<'a>) -> Result<Self>{
        let mut postfix = open.postfix;
        let mut from: usize = 0;
        loop{
            let candidate = postfix.find("{{{{/").ok_or(ParseError::unclosed(&open.raw))?;
            let start = candidate + 5;
            let remains = &postfix[start ..];
            let close = remains.find("}}}}").ok_or(ParseError::unclosed(&open.raw))?;
            let end = start + close + 4;
            if &remains[.. close] == open.content{
                return Ok(Self{
                    expression_type: ExpressionType::Escaped,
                    prefix: open.prefix,
                    content: &open.postfix[.. from + candidate],
                    postfix: &postfix[end ..],
                    raw: open.raw
                })
            }
            from += end;
            postfix = &postfix[from ..];
        }
    }

    /// Parses the next expression from a template string
    pub fn from(src: &'a str) -> Result<Option<Self>>{
        match src.find("{{"){
            Some(start) => {
                let mut second = nibble(src, start, 3)?;
                if start > 0 && &src[start - 1 .. start] == "\\"{
                    return Ok(Some(Self::close(ExpressionType::Escaped, &src[.. start - 1], &src[second - 1 ..], "}}")?));
                }
                let mut prefix = &src[.. start];
                let mut marker = &src[start + 2 .. second];
                if marker == "~"{
                    prefix = prefix.trim_end();
                    second = nibble(src, second, 1)?;
                    marker = &src[start + 3 .. second];
                }
                Ok(Some(match marker{
                    "{" => {
                        let next = nibble(src, second, 1)?;
                        let char = &src[second .. next];
                        if char == "{"{
                            second = next;
                            let next = nibble(src, second, 1)?;
                            if &src[second .. next] == "~"{
                                second = next;
                                prefix = prefix.trim_end();
                            }
                            return Ok(Some(Self::find_closing_escape(Self::close(ExpressionType::Escaped, prefix, &src[second ..], "}}}}")?)?));
                        }
                        if char == "~"{
                            second = next;
                            prefix = prefix.trim_end();
                        }
                        Self::close(ExpressionType::Raw, prefix, &src[second ..], "}}}")?
                    },
                    "!" => Self::check_comment(prefix, &src[second ..])?,
                    "#" => Self::close(ExpressionType::Open, prefix, &src[second ..], "}}")?,
                    "/" => Self::close(ExpressionType::Close, prefix, &src[second ..], "}}")?,
                    _ => Self::close(ExpressionType::HtmlEscaped, prefix, &src[second - 1 ..], "}}")?
                }))
            },
            None => Ok(None)
        }
    }

    /// Parses the next expression after this one
    pub fn next(&self) -> Result<Option<Self>>{
        Self::from(self.postfix)
    }

    /// Returns a string containing the expression and its surrounding context
    pub fn around(&self) -> &str{
        let len = self.raw.len();
        if len == 0{
            return self.raw;
        }
        let start = self.prefix.len();
        let end = start + self.content.len() + 16;
        return &self.raw[min(len - 1, if start > 16{ start - 16 } else {0}) .. min(self.raw.len(), end)];
    }
}

impl<'a> Display for Expression<'a>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.raw)
    }
}