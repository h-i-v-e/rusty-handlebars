use crate::error::{rcap, ParseError, Result};

/// Kind of token found inside an expression.
#[derive(Clone)]
pub enum TokenType<'a> {
    /// A parenthesized expression, with its original source.
    SubExpression(&'a str),
    /// A block-provided variable such as `@index`.
    PrivateVariable,
    /// A Rust-style field path or local template binding.
    Variable,
    /// A quoted string or other literal copied into generated Rust.
    Literal,
}

/// A borrowed token and the expression text that follows it.
#[derive(Clone)]
pub struct Token<'a> {
    /// How the compiler resolves this token.
    pub token_type: TokenType<'a>,
    /// The token source without an `@` prefix or subexpression parentheses.
    pub value: &'a str,
    /// Unparsed expression text after this token.
    pub tail: &'a str,
}

fn find_closing(src: &str) -> Result<usize> {
    let mut count = 1;
    let rest = &src[1..];
    for (i, c) in rest.char_indices() {
        match c {
            '(' => count += 1,
            ')' => count -= 1,
            _ => (),
        }
        if count == 0 {
            return Ok(i + 1);
        }
    }
    Err(ParseError {
        message: format!("unmatched brackets near {}", rcap(src)),
    })
}

fn find_end_of_string(src: &str) -> Result<usize> {
    let cliped = &src[1..];
    let mut escaped = false;
    for (i, c) in cliped.char_indices() {
        match c {
            '\\' => escaped = !escaped,
            '"' => {
                if !escaped {
                    return Ok(i + 2);
                }
            }
            _ => (),
        }
    }
    Err(ParseError {
        message: format!("unterminated string near {}", rcap(src)),
    })
}

fn find_end(src: &str) -> usize {
    for (i, c) in src.char_indices() {
        if " (\n\r\t".contains(c) {
            return i;
        }
    }
    src.len()
}

fn invalid_variable_name(src: &str) -> bool {
    if src.starts_with("../") {
        return false;
    }
    return src
        .chars()
        .next()
        .map(|c| !(c.is_alphabetic() || c == '_'))
        .unwrap_or(false);
}

fn parse<'a>(src: &'a str) -> Result<Option<Token<'a>>> {
    Ok(match src.chars().next() {
        Some('@') => {
            let end = find_end(src);
            Some(Token {
                token_type: TokenType::PrivateVariable,
                value: &src[1..end],
                tail: &src[end..].trim_start(),
            })
        }
        Some('(') => {
            let end = find_closing(&src)?;
            Some(Token {
                token_type: TokenType::SubExpression(&src[..end]),
                value: &src[1..end],
                tail: &src[end + 1..].trim_start(),
            })
        }
        None => None,
        _ => {
            let (end, token_type) = if src.starts_with('"') {
                (find_end_of_string(src)?, TokenType::Literal)
            } else {
                (
                    find_end(src),
                    if invalid_variable_name(src) {
                        TokenType::Literal
                    } else {
                        TokenType::Variable
                    },
                )
            };
            Some(Token {
                token_type,
                value: &src[..end],
                tail: &src[end..].trim_start(),
            })
        }
    })
}

impl<'a> Token<'a> {
    /// Parses the first token in an expression.
    pub fn first(src: &'a str) -> Result<Option<Self>> {
        parse(src.trim())
    }

    /// Parses the next token in [`Self::tail`].
    pub fn next(&self) -> Result<Option<Self>> {
        parse(self.tail)
    }
}
