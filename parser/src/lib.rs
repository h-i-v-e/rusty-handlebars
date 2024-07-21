use std::borrow::Cow;
use std::error::Error;
use std::fmt::Display;
use std::str::SplitWhitespace;
use regex::Captures;

use regex::Regex;

enum ExpressionType{
    Comment, HtmlEscaped, Raw, Open, Close, Escaped
}

struct Expression<'a>{
    expression_type: ExpressionType,
    preffix: &'a str,
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
                let mut postfix = &start[pos + end.len() ..];
                if &start[pos - 1 .. pos] == "~"{
                    postfix = postfix.trim_start();
                    pos -= 1;
                } 
                Ok(Self { expression_type, preffix, content: &start[.. pos], postfix })
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
                    "{" => Self::close(ExpressionType::Raw, prefix, &src[second ..], "}}}")?,
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
struct Scope<'a>{
    opened: OpenType,
    parent: Option<usize>,
    this: Option<&'a str>,
    local: Option<&'a str>
}

struct Compile<'a>{
    rust: String,
    open_stack: Vec<Scope<'a>>
}

impl<'a> Compile<'a>{
    fn new() -> Self{
        Self{
            rust: String::new(),
            open_stack: vec![Scope{
                parent: None,
                opened: OpenType::Root,
                this: Some("self"),
                local: Some("self")
            }]
        }
    }

    /*fn debug_stack(&self){
        for scope in self.open_stack.iter(){
            println!("{:?}", scope);
        }
    }*/

    fn push_scope_with_local(&mut self, opened: OpenType, local: Option<&'a str>){
        self.open_stack.push(Scope{
            parent: Some(self.open_stack.len() - 1),
            opened,
            this: None,
            local
        });
    }

    fn push_scope(&mut self, opened: OpenType){
        self.push_scope_with_local(opened, None)
    }

    fn find_scope(&self, mut var: &'a str) -> Option<(&'a str, &Scope<'a>)>{
        let mut scope = self.open_stack.last().unwrap();
        loop {
            if var.starts_with("../"){
                match scope.parent{
                    Some(offset) => {
                        var = &var[3 ..];
                        scope = self.open_stack.get(offset).unwrap();
                        continue;
                    },
                    None => return None
                }
            }
            return Some((var, scope));
        }
    }

    fn resolve_var(&self, var: &'a str, scope: &Scope<'a>, buffer: &mut String) -> Option<String>{
        if let Some(local) = scope.local{
            buffer.push_str(local);
            if var != local{
                buffer.push('.');
                buffer.push_str(var);
            }
            return None;
        }
        let parent = match scope.parent{
            Some(offset) => self.open_stack.get(offset).unwrap(),
            None => return Some(format!("variable {} not found", var))
        };
        if let Some(this) = scope.this{
            self.resolve_var(this, parent, buffer);
            if var != this{
                buffer.push('.');
                buffer.push_str(var);
            }
        }
        else{
            self.resolve_var(var, parent, buffer);
        }
        return None;
    }

    fn write_var(&mut self, var: &str) -> Option<String>{
        match self.find_scope(var){
            Some((var, scope)) => {
                let mut buffer = String::new();
                match self.resolve_var(var, scope, &mut buffer){
                    Some(error) => Some(error),
                    None => {
                        self.write(&buffer);
                        None
                    }
                }
            },
            None => Some(format!("variable {} not found", var))
        }
    }

    fn resolve(&mut self, src: &str, prefix: &str, postfix: &str){
        let mut tokens = src.split_whitespace();
        let var = match tokens.next(){
            Some(token) => token,
            None => return 
        };
        if var == "else"{
            if let Some(scope) = self.open_stack.last() {
                match scope.opened{
                    OpenType::If | OpenType::Unless => {
                        self.open_stack.push(Scope{
                            parent: scope.parent,
                            opened: OpenType::Else,
                            this: None,
                            local: None
                        });
                        self.rust.push_str("}else{");
                        return;
                    },
                    _ => ()
                }
            }
        }
        self.write(prefix);
        self.write_var(var);
        let mut glue = '(';
        while let Some(token) = tokens.next(){
            self.push(glue);
            self.write_var(token);
            glue = ',';
        }
        if glue != '('{
            self.push(')');
        }
        self.write(postfix);
    }

    fn write(&mut self, content: &str){
        self.rust.push_str(content);
    }

    fn push(&mut self, c: char){
        self.rust.push(c);
    }

    fn resolve_if(&mut self, mut tokens: SplitWhitespace<'a>) -> Option<String>{
        self.write("if ");
        if let Some(var) = tokens.next() {
            self.write_var(var);
            self.write(".as_bool(){");
            self.push_scope(OpenType::If);
            return None
        }
        Some("expected variable after if".to_string())
    }

    fn resolve_unless(&mut self, mut tokens: SplitWhitespace) -> Option<String>{
        self.write("if ");
        if let Some(var) = tokens.next() {
            self.write("!");
            self.write_var(var);
            self.write(".as_bool(){");
            self.push_scope(OpenType::Unless);
            return None
        }
        Some("expected variable after unless".to_string())
    }

    fn strip_pipes(mut tokens: SplitWhitespace<'a>) -> Option<&'a str>{
        loop{
            match tokens.next(){
                Some(token) => {
                    if token == "|"{
                        continue;
                    }
                    return Some(token.trim_matches('|'));
                },
                None => return None
            }
        }
    }

    fn resolve_each(&mut self, mut tokens: SplitWhitespace<'a>) -> Option<String>{
        let next = tokens.next().unwrap();
        let local = match tokens.next(){
            Some("as") => match Self::strip_pipes(tokens){
                Some(local) => local,
                None => return Some("expected variable after as".to_string())
            }
            Some(token) => return Some(format!("unexpected token {}", token)),
            None => "this"
        };
        self.write("for ");
        self.write(local);
        self.write(" in ");
        self.write_var(next);
        self.push('{');
        self.push_scope_with_local(OpenType::Each, Some(local));
        None
    }

    fn resolve_with(&mut self, mut tokens: SplitWhitespace<'a>) -> Option<String>{
        self.open_stack.push(Scope{
            parent: Some(self.open_stack.len() - 1),
            opened: OpenType::With,
            this: Some(match tokens.next(){
                Some(token) => token,
                None => return Some("expected variable after with".to_string())
            }),
            local: None
        });
        None
    }

    fn close(&mut self, content: &str) -> Result<()>{
        //self.debug_stack();
        let scope = match self.open_stack.pop() {
            Some(scope) => scope,
            None => return Err(ParseError::new(format!("Mismatched block near {}", rcap(content))))
        };
        let with = content == "with";
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
            OpenType::With => with,
            _ => false
        }{
            return Err(ParseError::new(format!("closing {} without matching open", content)));
        }
        if !with{
            self.push('}');
        }
        Ok(())
    }

    fn open(&mut self, content: &'a str) -> Result<()>{
        let mut tokens = content.split_whitespace();
        match match tokens.next(){
            Some("if") => self.resolve_if(tokens),
            Some("unless") => self.resolve_unless(tokens),
            Some("each") => self.resolve_each(tokens),
            Some("with") => self.resolve_with(tokens),
            _ => return Err(ParseError::new(format!("unsupported helper {}", content)))
        }{
            None => Ok(()),
            Some(error) => Err(ParseError::new(format!("{} near {}", error, content)))
        }
    }
} 

pub struct Compiler{
    clean: Regex
}

impl Compiler {
    pub fn new() -> Self{
        Self{
            clean: Regex::new("[\\\\\"]").unwrap()
        }
    }

    fn escape<'a>(&self, content: &'a str) -> Cow<'a, str> {
        self.clean.replace_all(
            &content, |captures: &Captures| format!("\\{}", &captures[0])
        )
    }

    fn write_str(&self, out: &mut Compile, content: &str) {
        if content.is_empty(){
            return;
        }
        out.write("f.write_str(\"");
        out.write(self.escape(content).as_ref());
        out.write("\")?;");
    }

    pub fn compile(&self, src: &str) -> Result<String>{
        let mut compile = Compile::new();
        let mut suffix = src;
        let mut expression = Expression::from(src)?;
        while let Some(expr) = expression{
            suffix = expr.postfix;
            self.write_str(&mut compile, expr.preffix);
            match expr.expression_type{
                ExpressionType::Raw => compile.resolve(expr.content, "as_text(&(", "),f)?;"),
                ExpressionType::HtmlEscaped => compile.resolve(expr.content, "as_html(&(", "), f)?;"),
                ExpressionType::Open => compile.open(expr.content)?,
                ExpressionType::Close => compile.close(expr.content.trim())?,
                ExpressionType::Escaped => self.write_str(&mut compile, expr.content),
                _ => ()
            }
            expression = expr.next()?;
        }
        self.write_str(&mut compile, suffix);
        Ok(compile.rust)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let compiler = Compiler::new();
        let src = "Hello {{{name}}}!";
        let rust = compiler.compile(src).unwrap();
        assert_eq!(rust, "f.write_str(\"Hello \")?;as_text(&(self.name),f)?;f.write_str(\"!\")?;");
    }

    #[test]
    fn test_if(){
        let rust = Compiler::new().compile("{{#if some}}Hello{{/if}}").unwrap();
        assert_eq!(rust, "if self.some.as_bool(){f.write_str(\"Hello\")?;}");
    }

    #[test]
    fn test_else(){
        let rust = Compiler::new().compile("{{#if some}}Hello{{else}}World{{/if}}").unwrap();
        assert_eq!(rust, "if self.some.as_bool(){f.write_str(\"Hello\")?;}else{f.write_str(\"World\")?;}");
    }

    #[test]
    fn test_unless(){
        let rust = Compiler::new().compile("{{#unless some}}Hello{{/unless}}").unwrap();
        assert_eq!(rust, "if !self.some.as_bool(){f.write_str(\"Hello\")?;}");
    }

    #[test]
    fn test_each(){
        let rust = Compiler::new().compile("{{#each some}}Hello {{this}}{{/each}}").unwrap();
        assert_eq!(rust, "for this in self.some{f.write_str(\"Hello \")?;as_html(&(this), f)?;}");
    }

    #[test]
    fn test_with(){
        let rust = Compiler::new().compile("{{#with some}}Hello {{name}}{{/with}}").unwrap();
        assert_eq!(rust, "f.write_str(\"Hello \")?;as_html(&(self.some.name), f)?;");
    }

    #[test]
    fn test_nesting(){
        let rust = Compiler::new().compile("{{#if some}}{{#each some}}Hello {{this}}{{/each}}{{/if}}").unwrap();
        assert_eq!(rust, "if self.some.as_bool(){for this in self.some{f.write_str(\"Hello \")?;as_html(&(this), f)?;}}");
    }

    #[test]
    fn test_as(){
        let rust = Compiler::new().compile("{{#if some}}{{#each some as thing}}Hello {{thing}}{{/each}}{{/if}}").unwrap();
        assert_eq!(rust, "if self.some.as_bool(){for thing in self.some{f.write_str(\"Hello \")?;as_html(&(thing), f)?;}}");
    }

    #[test]
    fn test_comment(){
        let rust = Compiler::new().compile("Note: {{! This is a comment }} and {{!-- {{so is this}} --}}\\{{{{}}").unwrap();
        assert_eq!(rust, "f.write_str(\"Note: \")?;f.write_str(\" and \")?;f.write_str(\"{{\")?;");
    }

    #[test]
    fn test_scoping(){
        let rust = Compiler::new().compile("{{#with some}}{{#with other}}Hello {{name}} {{../company}} {{/with}}{{/with}}").unwrap();
        assert_eq!(rust, "f.write_str(\"Hello \")?;as_html(&(self.some.other.name), f)?;f.write_str(\" \")?;as_html(&(self.some.company), f)?;f.write_str(\" \")?;");
    }

    #[test]
    fn test_trimming(){
        let rust = Compiler::new().compile("  {{~#if some ~}}   Hello{{~/if~}}").unwrap();
        assert_eq!(rust, "if self.some.as_bool(){f.write_str(\"Hello\")?;}");
    }
}