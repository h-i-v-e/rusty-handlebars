use std::borrow::Cow;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Write;
use optimizer::optimize;
use regex::Captures;

use regex::Regex;

mod expression_tokenizer;
mod optimizer;

use expression_tokenizer::Token;

enum ExpressionType{
    Comment, HtmlEscaped, Raw, Open, Close, Escaped
}

struct Expression<'a>{
    expression_type: ExpressionType,
    prefix: &'a str,
    content: &'a str,
    postfix: &'a str
}

#[derive(Debug)]
pub struct ParseError{
    message: String
}

fn rcap<'a>(src: &'a str) -> &'a str{
    static CAP_AT: usize = 32;

    if src.len() > CAP_AT{
        &src[src.len() - CAP_AT ..]
    } else {
        src
    }
}

impl ParseError{
    fn new(message: String) -> Self{
        Self{
            message
        }
    }

    fn unclosed(preffix: &str) -> Self{
        Self::new(format!("Unclosed block near {}", rcap(preffix)))
    }
}

impl Display for ParseError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ParseError{}

type Result<T> = std::result::Result<T, ParseError>;

impl<'a> Expression<'a>{
    fn close(expression_type: ExpressionType, preffix: &'a str, start: &'a str, end: &'static str) -> Result<Self>{
        match start.find(end){
            Some(mut pos) => {
                if pos == 0{
                    return Err(ParseError::new(format!("empty block near {}", rcap(preffix))));
                }
                let mut postfix = &start[pos + end.len() ..];
                if &start[pos - 1 .. pos] == "~"{
                    postfix = postfix.trim_start();
                    pos -= 1;
                } 
                Ok(Self { expression_type, prefix: preffix, content: &start[.. pos], postfix })
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

    fn from(src: &'a str) -> Result<Option<Self>>{
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

    fn next(&self) -> Result<Option<Self>>{
        Self::from(self.postfix)
    }
}

#[derive(Debug)]
enum OpenType{
    Root, If, Else, Unless, Each, With
}

#[derive(Debug)]
enum Local<'a>{
    As(&'a str),
    This,
    None
}

#[derive(Debug)]
struct Scope<'a>{
    opened: OpenType,
    depth: usize,
    parent: Option<usize>,
    this: Option<&'a str>,
    local: Local<'a>,
    indexer: Option<String>
}

pub struct Uses{
    uses: HashSet<&'static str>
}

pub static USE_AS_DISPLAY: &str = "AsDisplay";
pub static USE_AS_DISPLAY_HTML: &str = "AsDisplayHtml";

impl Uses{
    pub fn new() -> Self{
        Self{
            uses: HashSet::new()
        }
    }

    pub fn insert(&mut self, use_: &'static str){
        self.uses.insert(use_);
    }
}

impl Display for Uses{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("use rusty_handlebars::")?;
        if self.uses.is_empty(){
            f.write_str("DisplayAsHtml")?;
            return Ok(());
        }
        f.write_str("{DisplayAsHtml")?;
        for use_ in &self.uses{
            f.write_char(',')?;
            f.write_str(use_)?;
        }
        f.write_char('}')?;
        Ok(())
    }
}

struct Compile<'a>{
    open_stack: Vec<Scope<'a>>,
    uses: Uses
}

fn contains_indexer(src: &str, mut depth: i32) -> bool{
    match src.find("index"){
        Some(pos) => {
            match src[..pos].rfind('@'){
                Some(start) => {
                    let mut prefix = &src[start + 1 .. pos];
                    while prefix.starts_with("../"){
                        depth -= 1;
                        prefix = &prefix[3 ..];
                    }
                    return depth == 0;
                },
                None => return false
            }
        },
        None => return false
    }
}

fn check_for_indexer(src: &str) -> Result<bool>{
    let mut exp = Expression::from(src)?;
    let mut depth = 1;
    while let Some(expr) = &exp{
        match expr.expression_type{
            ExpressionType::Comment | ExpressionType::Escaped => continue,
            ExpressionType::Open => if contains_indexer(expr.content, depth - 1) {
                return Ok(true);
            } else{
                depth += 1;
            },
            ExpressionType::Close => {
                depth -= 1;
                if depth == 0{
                    return Ok(false);
                }
            },
            _ => if contains_indexer(expr.content, depth - 1) {
                return Ok(true);
            }
        }
        exp = expr.next()?;
    }
    Ok(false)
}

fn append_with_depth(depth: usize, var: &str, buffer: &mut String){
    buffer.push_str(var);
    buffer.push('_');
    buffer.push_str(depth.to_string().as_str());
}

impl<'a> Compile<'a>{
    fn new(this: Option<&'a str>) -> Self{
        Self{
            open_stack: vec![Scope{
                depth: 0,
                parent: None,
                opened: OpenType::Root,
                this,
                local: Local::None,
                indexer: None
            }],
            uses: Uses::new()
        }
    }

    /*fn debug_stack(&self){
        for scope in self.open_stack.iter(){
            println!("{:?}", scope);
        }
    }*/

    fn push_scope_with_local(&mut self, opened: OpenType, local: Local<'a>){
        let depth = self.open_stack.len();
        self.open_stack.push(Scope{
            depth,
            parent: Some(depth - 1),
            opened,
            this: None,
            local,
            indexer: None
        });
    }

    fn push_scope(&mut self, opened: OpenType){
        self.push_scope_with_local(opened, Local::None)
    }

    fn find_scope(&self, var: &'a str) -> Result<(&'a str, &Scope<'a>)>{
        let mut scope = self.open_stack.last().unwrap();
        let mut local = var;
        loop {
            if local.starts_with("../"){
                match scope.parent{
                    Some(offset) => {
                        local = &var[3 ..];
                        scope = self.open_stack.get(offset).unwrap();
                        continue;
                    },
                    None => return Err(ParseError::new(format!("unable to resolve scope for {}", var)))
                }
            }
            return Ok((local, scope));
        }
    }

    fn resolve_local(&self, depth: usize, var: &'a str, local: &'a str, buffer: &mut String) -> bool{
        if var.starts_with(local){
            let len = local.len();
            if var.len() > len{
                if &var[len .. len + 1] != "."{
                    return false;
                }
                append_with_depth(depth, local, buffer);
                buffer.push_str(&var[len ..]);
            }
            else{
                append_with_depth(depth, local, buffer);
            }
            return true;
        }
        return false;
    }

    fn resolve_var(&self, var: &'a str, scope: &Scope<'a>, buffer: &mut String) -> Result<()>{
        if scope.depth == 0{
            if let Some(this) = scope.this{
                buffer.push_str(this);
                buffer.push('.');
            }
            buffer.push_str(var);
            return Ok(());
        }
        if match scope.local{
            Local::As(local) => self.resolve_local(scope.depth, var, local, buffer),
            Local::This => {
                buffer.push_str("this_");
                buffer.push_str(scope.depth.to_string().as_str());
                if var != "this"{
                    buffer.push('.');
                    buffer.push_str(var);
                }
                true
            },
            Local::None => false
        }{
            return Ok(());
        }
        let parent = match scope.parent{
            Some(offset) => self.open_stack.get(offset).unwrap(),
            None => return Err(ParseError::new(format!("variable {} not found", var)))
        };
        if let Some(this) = scope.this{
            self.resolve_var(this, parent, buffer)?;
            if var != this{
                buffer.push('.');
                buffer.push_str(var);
            }
        }
        else{
            self.resolve_var(var, parent, buffer)?;
        }
        return Ok(());
    }

    fn write_var(&mut self, rust: &mut String, var: Token) -> Result<()>{
        if var.is_sub_expression{
            return self.resolve(rust, var.value, "(", ")");
        }
        let var = var.value;
        let starts_with = &var[..1];
        match starts_with{
            "&" | "*" => {
                rust.push_str(starts_with);
                let (var, scope) = self.find_scope(&var[1..])?;
                Ok(self.resolve_var(var, scope, rust)?)
            },
            "@" => {
                let (label, scope) = self.find_scope(&var[1..])?;
                match label{
                    "index" => match &scope.indexer{
                        Some(indexer) => Ok(rust.push_str(indexer.as_str())),
                        _ => Err(ParseError::new(format!("{} not in scope", var)))
                    },
                    _ => Err(ParseError::new(format!("{} not implimented", var)))
                }
            },
            _ => {
                let (var, scope) = self.find_scope(var)?;
                Ok(self.resolve_var(var, scope, rust)?)
            }
        }
    }

    fn resolve(&mut self, rust: &mut String, src: &str, prefix: &str, postfix: &str) -> Result<()>{
        let token = match Token::first(src)?{
            Some(token) => token,
            None => return Err(ParseError::new(format!("expected token near {}", prefix)))
        };
        if token.is_sub_expression{
            rust.push_str(prefix);
            self.resolve(rust, token.value, "(", ")")?;
            rust.push_str(postfix);
            return Ok(())
        }
        match token.value{
            "else" => match self.open_stack.last() {
                Some(scope) => match scope.opened{
                    OpenType::If | OpenType::Unless => {
                        self.open_stack.push(Scope{
                            depth: self.open_stack.len(),
                            parent: scope.parent,
                            opened: OpenType::Else,
                            this: None,
                            local: Local::None,
                            indexer: None
                        });
                        rust.push_str("}else{");
                        Ok(())
                    },
                    _ => Err(ParseError::new("else without if or unless".to_string()))
                },
                None => Err(ParseError::new("else without if or unless".to_string()))
            },
            "lookup" => {
                rust.push_str(prefix);
                match token.next()?{
                    Some(token) => {
                        let index = token.next()?;
                        self.write_var(rust, token)?;
                        rust.push('[');
                        match index{
                            Some(token) => {
                                self.write_var(rust, token)?;
                                rust.push(']');
                                rust.push_str(postfix);
                                Ok(())
                            },
                            None => Err(ParseError::new("else without if or unless".to_string()))
                        }
                    },
                    None => Err(ParseError::new("else without if or unless".to_string()))
                }
            },
            _ => {
                rust.push_str(prefix);
                let mut next = token.next()?;
                self.write_var(rust, token)?;
                let mut glue = '(';
                while let Some(token) = next{
                    rust.push(glue);
                    next = token.next()?;
                    self.write_var(rust, token)?;
                    glue = ',';
                }
                if glue != '('{
                    rust.push(')');
                }
                rust.push_str(postfix);
                Ok(())
            }
        }
    }

    fn resolve_if(&mut self, prefix: &str, rust: &mut String, token: Token) -> Result<()>{
        match token.next()? {
            Some(var) => {
                self.uses.insert("AsBool");
                rust.push_str("if ");
                self.write_var(rust, var)?;
                rust.push_str(".as_bool(){");
                self.push_scope(OpenType::If);
                Ok(())
            },
            None => Err(ParseError::new(format!("expected variable after if near {}", prefix)))
        }
    }

    fn resolve_unless(&mut self, prefix: &str, rust: &mut String, token: Token) -> Result<()>{
        match token.next()? {
            Some(var) => {
                self.uses.insert("AsBool");
                rust.push_str("if !");
                self.write_var(rust, var)?;
                rust.push_str(".as_bool(){");
                self.push_scope(OpenType::Unless);
                Ok(())
            }
            None => Err(ParseError::new(format!("expected variable after unless near {}", prefix)))
        }
    }

    fn strip_pipes(token: Token) -> Result<&str>{
        loop{
            match token.next()?{
                Some(token) => {
                    if token.value == "|"{
                        continue;
                    }
                    return Ok(token.value.trim_matches('|'));
                },
                None => return Err(ParseError::new("expected variable after as".to_string()))
            }
        }
    }

    fn read_local(token: &Token<'a>) -> Result<Local<'a>>{
        match token.next()?{
            Some(token) => match token.value{
                "as" => Ok(Local::As(Self::strip_pipes(token)?)),
                token => Err(ParseError::new(format!("unexpected token {}", token)))
            },
            None => Ok(Local::This)
        }
    }

    fn write_local(&self, rust: &mut String, local: &Local<'a>){
        append_with_depth(self.open_stack.len(), match local{
            Local::As(local) => local,
            _ => "this"
        }, rust);
    }

    fn resolve_each(&mut self, prefix: &'a str, rust: &mut String, token: Token<'a>, suffix: &'a str) -> Result<()>{
        let indexer = check_for_indexer(suffix).map(|found| match found{
            true => {
                let indexer = format!("i_{}", self.open_stack.len());
                rust.push_str("let mut ");
                rust.push_str(indexer.as_str());
                rust.push_str(" = 0;");
                Some(indexer)
            },
            false => None
        })?;
        let next = match token.next()?{
            Some(next) => next,
            None => return Err(ParseError::new(format!("expected variable after each near {}", prefix)))
        };
        let local = Self::read_local(&next)?;
        rust.push_str("for ");
        self.write_local(rust, &local);
        rust.push_str(" in ");
        self.write_var(rust, next)?;
        rust.push('{');
        let depth = self.open_stack.len();
        self.open_stack.push(Scope::<'a>{
            depth,
            parent: Some(depth - 1),
            opened: OpenType::Each,
            this: None,
            local: local,
            indexer
        });
        Ok(())
    }

    fn resolve_with(&mut self, prefix: &str, rust: &mut String, token: Token<'a>) -> Result<()>{
        let depth = self.open_stack.len();
        let token = match token.next()?{
            Some(token) => token,
            None => return Err(ParseError::new(format!("expected variable after with near {}", prefix)))
        };
        let local = Self::read_local(&token)?;
        rust.push_str("{let ");
        self.write_local(rust, &local);
        rust.push_str(" = ");
        self.write_var(rust, token)?;
        rust.push(';');
        self.open_stack.push(Scope::<'a>{
            depth,
            parent: Some(depth - 1),
            opened: OpenType::With,
            this: None,
            local: local,
            indexer: None
        });
        Ok(())
    }

    fn close(&mut self, rust: &mut String, content: &str) -> Result<()>{
        //self.debug_stack();
        let scope = match self.open_stack.pop() {
            Some(scope) => scope,
            None => return Err(ParseError::new(format!("Mismatched block near {}", rcap(content))))
        };
        if !match scope.opened{
            OpenType::If => content == "if",
            OpenType::Else => match self.open_stack.pop(){
                Some(scope) => match scope.opened{
                    OpenType::If | OpenType::Unless => true,
                    _ => false
                },
                None => false
            },
            OpenType::Unless => content == "unless",
            OpenType::Each => content == "each",
            OpenType::With => content == "with",
            _ => false
        }{
            return Err(ParseError::new(format!("closing {} without matching open", content)));
        }
        if let Some(index) = &scope.indexer{
            rust.push_str(index.as_str());
            rust.push_str(" += 1;");
        }
        rust.push('}');
        Ok(())
    }

    fn open(&mut self, rust: &mut String, prefix: &'a str, content: &'a str, suffix: &'a str) -> Result<()>{
        let token = match Token::first(content)?{
            Some(token) => token,
            None => return Err(ParseError::new(format!("expected token near {}", prefix)))
        };
        match token.value{
            "if" => self.resolve_if(prefix, rust, token),
            "unless" => self.resolve_unless(prefix, rust, token),
            "each" => self.resolve_each(prefix, rust, token, suffix),
            "with" => self.resolve_with(prefix, rust, token),
            _ => Err(ParseError::new(format!("unsupported helper {}", content)))
        }
    }
} 

pub struct Compiler{
    clean: Regex
}

impl Compiler {
    pub fn new() -> Self{
        Self{
            clean: Regex::new("[\\\\\"\\{\\}]").unwrap(),
        }
    }

    fn escape<'a>(&self, content: &'a str) -> Cow<'a, str> {
        self.clean.replace_all(
            &content, |captures: &Captures| match &captures[0]{
                "{" | "}" => format!("{}{}", &captures[0], &captures[0]),
                _ => format!("\\{}", &captures[0])
            }
        )
    }

    fn write_str(&self, rust: &mut String, content: &str) {
        if content.is_empty(){
            return;
        }
        rust.push_str("write!(f, \"");
        rust.push_str(self.escape(content).as_ref());
        rust.push_str("\")?;");
    }

    fn write_resolve(&self, compile: &mut Compile, rust: &mut String, content: &str, display: &str, uses: &'static str) -> Result<()> {
        compile.uses.insert(uses);
        compile.resolve(rust, content, "write!(f, \"{}\", ", display)
    }

    pub fn compile(&self, this: Option<&str>, src: &str) -> Result<(Uses, String)>{
        let mut compile = Compile::new(this);
        let mut rust = String::new();
        let mut rest = src;
        let mut expression = Expression::from(src)?;
        while let Some(expr) = expression{
            let Expression{
                expression_type,
                prefix,
                content,
                postfix
            } = &expr;
            rest = postfix;
            self.write_str(&mut rust, prefix);
            match expression_type{
                ExpressionType::Raw => self.write_resolve(&mut compile, &mut rust, content, ".as_display())?;", USE_AS_DISPLAY)?,
                ExpressionType::HtmlEscaped => self.write_resolve(&mut compile, &mut rust, content, ".as_display_html())?;", USE_AS_DISPLAY_HTML)?,
                ExpressionType::Open => compile.open(&mut rust, prefix, content, postfix)?,
                ExpressionType::Close => compile.close(&mut rust, expr.content.trim())?,
                ExpressionType::Escaped => self.write_str(&mut rust, content),
                _ => ()
            };
            expression = expr.next()?;
        }
        self.write_str(&mut rust, rest);
        Ok((compile.uses, optimize(rust)))
    }
}

#[cfg(test)]
mod tests {
    use core::str;

    use crate::*;
    use minify_html::{minify, Cfg};

    static CFG: Cfg = Cfg {
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

    fn compile(src: &str) -> String{
        Compiler::new().compile(Some("self"), src).unwrap().1
    }

    #[test]
    fn it_works() {
        assert_eq!(
            compile("Hello {{{name}}}!"),
            "write!(f, \"Hello {}!\", self.name.as_display())?;"
        );
    }

    #[test]
    fn test_if(){
        let rust = compile("{{#if some}}Hello{{/if}}");
        assert_eq!(rust, "if self.some.as_bool(){write!(f, \"Hello\")?;}");
    }

    #[test]
    fn test_else(){
        let rust = compile("{{#if some}}Hello{{else}}World{{/if}}");
        assert_eq!(rust, "if self.some.as_bool(){write!(f, \"Hello\")?;}else{write!(f, \"World\")?;}");
    }

    #[test]
    fn test_unless(){
        let rust = compile("{{#unless some}}Hello{{/unless}}");
        assert_eq!(rust, "if !self.some.as_bool(){write!(f, \"Hello\")?;}");
    }

    #[test]
    fn test_each(){
        let rust = compile("{{#each some}}Hello {{this}}{{/each}}");
        assert_eq!(rust, "for this_1 in self.some{write!(f, \"Hello {}\", this_1.as_display_html())?;}");
    }

    #[test]
    fn test_with(){
        let rust = compile("{{#with some}}Hello {{name}}{{/with}}");
        assert_eq!(rust, "{let this_1 = self.some;write!(f, \"Hello {}\", this_1.name.as_display_html())?;}");
    }

    #[test]
    fn test_nesting(){
        let rust = compile("{{#if some}}{{#each some}}Hello {{this}}{{/each}}{{/if}}");
        assert_eq!(rust, "if self.some.as_bool(){for this_2 in self.some{write!(f, \"Hello {}\", this_2.as_display_html())?;}}");
    }

    #[test]
    fn test_as(){
        let rust = compile("{{#if some}}{{#each some as thing}}Hello {{thing}} {{thing.name}}{{/each}}{{/if}}");
        assert_eq!(rust, "if self.some.as_bool(){for thing_2 in self.some{write!(f, \"Hello {} {}\", thing_2.as_display_html(), thing_2.name.as_display_html())?;}}");
    }

    #[test]
    fn test_comment(){
        let rust = compile("Note: {{! This is a comment }} and {{!-- {{so is this}} --}}\\{{{{}}");
        assert_eq!(rust, "write!(f, \"Note:  and {{{{\")?;");
    }

    #[test]
    fn test_scoping(){
        let rust = compile("{{#with some}}{{#with other}}Hello {{name}} {{../company}} {{/with}}{{/with}}");
        assert_eq!(rust, "{let this_1 = self.some;{let this_2 = this_1.other;write!(f, \"Hello {} {} \", this_2.name.as_display_html(), this_1.company.as_display_html())?;}}");
    }

    #[test]
    fn test_trimming(){
        let rust = compile("  {{~#if some ~}}   Hello{{~/if~}}");
        assert_eq!(rust, "if self.some.as_bool(){write!(f, \"Hello\")?;}");
    }

    #[test]
    fn test_indexer(){
        let rust = compile("{{#each things}}Hello{{{@index}}}{{#each things}}{{{lookup other @../index}}}{{{@index}}}{{/each}}{{/each}}");
        assert_eq!(rust, "let mut i_1 = 0;for this_1 in self.things{write!(f, \"Hello{}\", i_1.as_display())?;let mut i_2 = 0;for this_2 in this_1.things{write!(f, \"{}{}\", this_2.other[i_1].as_display(), i_2.as_display())?;i_2 += 1;}i_1 += 1;}");
    }

    #[test]
    fn test_subexpression(){
        let rust = compile("{{#each things}}{{#with (lookup ../other @index) as |other|}}{{{../name}}}: {{{other}}}{{/with}}{{/each}}");
        assert_eq!(rust, "let mut i_1 = 0;for this_1 in self.things{{let other_2 = (self.other[i_1]);write!(f, \"{}: {}\", this_1.name.as_display(), other_2.as_display())?;}i_1 += 1;}");
    }

    #[test]
    fn test_selfless(){
        let (uses, rust) = Compiler::new().compile(None, "{{#each things}}{{#with (lookup ../other @index) as |other|}}{{{../name}}}: {{{other}}}{{/with}}{{/each}}").unwrap();
        assert_eq!(uses.to_string(), "use rusty_handlebars::{DisplayAsHtml,AsDisplay}");
        assert_eq!(rust, "let mut i_1 = 0;for this_1 in things{{let other_2 = (other[i_1]);write!(f, \"{}: {}\", this_1.name.as_display(), other_2.as_display())?;}i_1 += 1;}");
    }

    #[test]
    fn javascript(){
        let (uses, rust) = Compiler::new().compile(None, "<script>if (location.href.contains(\"localhost\")){ console.log(\"\\{{{{}}}}\") }</script>").unwrap();
        assert_eq!(uses.to_string(), "use rusty_handlebars::DisplayAsHtml");
        assert_eq!(rust, "write!(f, \"<script>if (location.href.contains(\\\"localhost\\\")){{ console.log(\\\"{{{{}}}}\\\") }}</script>\")?;");
    }

    #[test]
    fn complex(){
        let (uses, rust) = Compiler::new().compile(None, unsafe { str::from_utf8_unchecked(minify(include_bytes!("../../examples/templates/index.hbs"), &CFG).as_slice()) }).unwrap();
        assert_eq!(uses.to_string(), "use rusty_handlebars::{DisplayAsHtml,AsDisplay}");
        assert_eq!(rust, "write!(f, \"<script>if (location.href.contains(\\\"localhost\\\")){{ console.log(\\\"{{{{}}}}\\\") }}</script>\")?;");
    }
}