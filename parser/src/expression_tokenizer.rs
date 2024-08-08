use crate::error::{Result, ParseError, rcap, parse_error_near};

#[derive(Clone)]
pub enum TokenType{
    SubExpression,
    Manipulator,
    Literal
}

#[derive(Clone)]
pub(crate) struct Token<'a>{
    pub token_type: TokenType,
    pub value: &'a str,
    tail: &'a str
}

fn find_closing(src: &str) -> Result<usize>{
    let mut count = 1;
    for (i, c) in src.char_indices(){
        match c{
            '(' => count += 1,
            ')' => count -= 1,
            _ => ()
        }
        if count == 0{
            return Ok(i);
        }
    }
    parse_error_near!(src, "unmatched bracket")
}

fn find_end(src: &str) -> usize{
    for (i, c) in src.char_indices(){
        if " (\n\r\t".contains(c){
            return i
        }
    }
    src.len()
}

fn parse<'a>(src: &'a str) -> Result<Option<Token<'a>>>{
    Ok(match src.chars().next(){
        Some('@' | '&' | '*') => Some(Token{
            token_type: TokenType::Manipulator,
            value: &src[..1],
            tail: &src[1..].trim_start()
        }),
        Some('(') => {
            let end = find_closing(&src[1 ..])?;
            Some(Token{
                token_type: TokenType::SubExpression,
                value: &src[1..end + 1],
                tail: &src[end + 2..].trim_start()
            })
        },
        None => None,
        _ => {
            let end = find_end(src);
            Some(Token{
                token_type: TokenType::Literal,
                value: &src[..end],
                tail: &src[end..].trim_start()
            })
        }
    })
}

impl<'a> Token<'a>{
    pub fn first(src: &'a str) -> Result<Option<Self>>{
        parse(src.trim())
    }

    pub fn next(&self) -> Result<Option<Self>>{
        parse(self.tail)
    }
}

