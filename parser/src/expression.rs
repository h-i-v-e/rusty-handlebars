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

    pub fn from(src: &'a str) -> Result<Option<Self>>{
        match src.find("{{"){
            Some(start) => {
                let mut second = start + 3;
                if second >= src.len(){
                    return Err(ParseError::unclosed(src));
                }
                if start > 0 && &src[start - 1 .. start] == "\\"{
                    return Ok(Some(Self::close(ExpressionType::Escaped, &src[.. start - 1], &src[second - 1 ..], "}}")?));
                }
                let mut prefix = &src[.. start];
                let mut marker = &src[start + 2 .. second];
                if marker == "~"{
                    prefix = prefix.trim_end();
                    second += 1;
                    if second >= src.len(){
                        return Err(ParseError::unclosed(src));
                    }
                    marker = &src[start + 3 .. second];
                }
                Ok(Some(match marker{
                    "{" => {
                        let next = second + 1;
                        if next >= src.len(){
                            return Err(ParseError::unclosed(src));
                        }
                        if &src[second .. next] == "~"{
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