use core::str;
use std::{fs::File, io::Read};

use crate::{error::{parse_error_near, rcap, ParseError}, expression::{Expression, ExpressionType}, Compiler, Options, Result, Uses};
use minify_html::{Cfg, minify};

#[cfg(feature = "minify-html")]
pub static COMPRESS_CONFIG: Cfg = Cfg {
    minify_js: true,
    minify_css: true,
    do_not_minify_doctype: true,
    ensure_spec_compliant_unquoted_attribute_values: true,
    keep_closing_tags: true,
    keep_html_and_head_opening_tags: true,
    keep_spaces_between_attributes: true,
    keep_comments: false,
    keep_input_type_text_attr: false,
    keep_ssi_comments: false,
    preserve_brace_template_syntax: true,
    preserve_chevron_percent_template_syntax: false,
    remove_bangs: false,
    remove_processing_instructions: false
};

fn extract_name_of_writer<'a>(content: &'a str) -> Result<&'a str>{
    if let Some(pos) = content.find('('){
        let content = &content[pos + 1 ..];
        if let Some(pos) = content.find(':') {
            return Ok((&content[.. pos]).trim());
        }
    }
    return parse_error_near!(content, "expected the first argument to be a writer");
}

pub fn generate_function_from_str(src: &str) -> Result<(Uses, String)>{
    let (rest, write, signature) = match Expression::from(src)?{
        Some(first) => match first.expression_type{
            ExpressionType::Comment => {
                let content = first.content.trim();
                let write = extract_name_of_writer(content)?;
                (first.postfix, write, content)
            },
            _ => return Err(ParseError::new("First expression must be a comment containing the function signature".to_string()))
        },
        None => return Err(ParseError::new("Empty source".to_string()))
    };
    let (uses, body) = Compiler::new(Options{
        write_var_name: write,
        root_var_name: None        
    }).compile(rest)?;
    Ok((uses, format!("{}{{{}Ok(())}}", signature, body)))
}

fn read_file(file: &mut File) -> std::result::Result<String, std::io::Error>{
    let mut buffer = String::with_capacity(file.metadata()?.len() as usize);
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

pub fn generate_function_from_file(mut file: File) -> Result<(Uses, String)>{
    match read_file(&mut file) {
        Ok(src) => {
            if cfg!(feature = "minify-html"){
                let src = minify(src.as_bytes(), &COMPRESS_CONFIG);
                generate_function_from_str(match str::from_utf8(src.as_slice()){
                    Ok(src) => src,
                    Err(err) => return Err(ParseError::new(format!("Expected src to be utf-8 encoded: {}", err.to_string())))
                })
            }
            else {
                generate_function_from_str(src.as_str())
            }
        },
        Err(err) => Err(ParseError::new(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_function_from_str(){
        let src = "{{!fn write<W: Write>(writer: &mut W) -> std::io::Result<()>}}Hello, World!";
        let (uses, content) = generate_function_from_str(src).unwrap();
        assert_eq!(uses.to_string(), "");
        assert_eq!(content, r#"fn write<W: Write>(writer: &mut W) -> std::io::Result<()>{write!(writer, "Hello, World!")?;Ok(())}"#);
    }
}