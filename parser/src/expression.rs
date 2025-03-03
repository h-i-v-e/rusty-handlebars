use std::{cmp::min, fmt::Display};

use crate::error::{Result, ParseError};

#[derive(Debug, Clone, Copy)]
pub enum ExpressionType{
    Comment, HtmlEscaped, Raw, Open, Close, Escaped
}

#[derive(Debug, Clone, Copy)]
pub struct Expression<'a>{
    pub expression_type: ExpressionType,
    pub prefix: &'a str,
    pub content: &'a str,
    pub postfix: &'a str,
    pub raw: &'a str
}

#[inline]
fn nibble(src: &str, start: usize, len: usize) -> Result<usize>{
    let end = start + len; 
    if end >= src.len(){
        return Err(ParseError::unclosed(src));
    }
    Ok(end)
}

impl<'a> Expression<'a>{
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

    fn check_comment(preffix: &'a str, start: &'a str) -> Result<Self>{
        if let Some(pos) = start.find("--"){
            if pos == 0{
                return Self::close(ExpressionType::Comment, preffix, &start[2 ..], "--}}");
            }
        }
        Self::close(ExpressionType::Comment, preffix, start, "}}")
    }

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

    pub fn next(&self) -> Result<Option<Self>>{
        Self::from(self.postfix)
    }

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