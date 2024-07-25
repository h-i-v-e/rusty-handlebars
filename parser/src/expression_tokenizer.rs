use crate::Result;

pub(crate) struct Token<'a>{
    pub is_sub_expression: bool,
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
    Err(crate::ParseError { message: format!("unmatched bracket near {}", src) })
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
    if src.is_empty(){
        return Ok(None);
    }
    Ok(Some(match src.chars().next(){
        Some('(') => {
            let end = find_closing(&src[1 ..])?;
            Token{
                is_sub_expression: true,
                value: &src[1..end + 1],
                tail: &src[end + 2..].trim_start()
            }
        },
        _ => {
            let end = find_end(src);
            Token{
                is_sub_expression: false,
                value: &src[..end],
                tail: &src[end..].trim_start()
            }
        }
    }))
}

impl<'a> Token<'a>{
    pub fn first(src: &'a str) -> Result<Option<Self>>{
        parse(src.trim())
    }

    pub fn next(&self) -> Result<Option<Self>>{
        parse(self.tail)
    }
}

