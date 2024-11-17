use crate::error::{rcap, ParseError, Result};

#[derive(Clone)]
pub enum TokenType<'a>{
    SubExpression(&'a str),
    PrivateVariable,
    Literal
}

#[derive(Clone)]
pub struct Token<'a>{
    pub token_type: TokenType<'a>,
    pub value: &'a str,
    pub tail: &'a str
}

fn find_closing(src: &str) -> Result<usize>{
    let mut count = 1;
    let rest = &src[1..];
    for (i, c) in rest.char_indices(){
        match c{
            '(' => count += 1,
            ')' => count -= 1,
            _ => ()
        }
        if count == 0{
            return Ok(i + 1);
        }
    }
    Err(ParseError{ message: format!("unmatched brackets near {}", rcap(src))})
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
        Some('@') => {
            let end = find_end(src);
            Some(Token{
                token_type: TokenType::PrivateVariable,
                value: &src[1..end],
                tail: &src[end..].trim_start()
            })
        },
        Some('(') => {
            let end = find_closing(&src)?;
            Some(Token{
                token_type: TokenType::SubExpression(&src[..end]),
                value: &src[1..end],
                tail: &src[end + 1..].trim_start()
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

