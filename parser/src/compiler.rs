use std::{borrow::Cow, collections::{HashMap, HashSet}, fmt::{Display, Write}};

use regex::{Captures, Regex};

use crate::{error::{ParseError, Result}, expression::{Expression, ExpressionType}, expression_tokenizer::{Token, TokenType}/* , optimizer::optimize*/};

pub enum Local{
    As(String),
    This,
    None
}

pub struct Scope{
    pub opened: Box<dyn Block>,
    pub depth: usize
}

enum PendingWrite<'a>{
    Raw(&'a str),
    Expression((Expression<'a>, &'static str, &'static str))
}

pub struct Rust{
    pub using: HashSet<&'static str>,
    pub code: String
}

pub static USE_AS_DISPLAY: &str = "AsDisplay";
pub static USE_AS_DISPLAY_HTML: &str = "AsDisplayHtml";

pub struct Uses<'a>{
    uses: &'a HashSet<&'static str>
}

impl<'a> Display for Uses<'a>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.uses.len(){
            0 => (),
            1 => write!(f, "use rusty_handlebars::{}", self.uses.iter().next().unwrap())?,
            _ => {
                f.write_str("use rusty_handlebars::")?;
                let mut glue = '{';
                for use_ in self.uses{
                    f.write_char(glue)?;
                    f.write_str(use_)?;
                    glue = ',';
                }
                f.write_str("}")?;
            }
        }
        Ok(())
    }
}

impl Rust{
    pub fn new() -> Self{
        Self{
            using: HashSet::new(),
            code: String::new()
        }
    }

    pub fn uses(&self) -> Uses{
        Uses{ uses: &self.using}
    }
}

pub trait Block{
    fn handle_close<'a>(&self, rust: &mut Rust) {
        rust.code.push_str("}");
    }

    fn resolve_private<'a>(&self, _depth: usize, expression: &'a Expression<'a>, _name: &str, _rust: &mut Rust) -> Result<()>{
        Err(ParseError::new(&format!("{} not expected ", expression.content), expression))
    }

    fn handle_else<'a>(&self, expression: &'a Expression<'a>, _rust: &mut Rust) -> Result<()>{
        Err(ParseError::new("else not expected here", expression))
    }

    fn this<'a>(&self) -> Option<&str>{
        None
    }

    fn local<'a>(&self) -> &Local{
        &Local::None
    }
}

pub trait BlockFactory{
    fn open<'a>(&self, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Box<dyn Block>>;
}

pub type BlockMap = HashMap<&'static str, &'static dyn BlockFactory>;

pub struct Compile<'a>{
    pub open_stack: Vec<Scope>,
    pub block_map: &'a BlockMap
}

pub fn append_with_depth(depth: usize, var: &str, buffer: &mut String){
    buffer.push_str(var);
    buffer.push('_');
    buffer.push_str(depth.to_string().as_str());
}

struct Root<'a>{
    this: Option<&'a str>
}

impl<'a> Block for Root<'a>{
    fn this<'b>(&self) -> Option<&str>{
        self.this
    }
}

impl<'a> Compile<'a>{
    fn new(this: Option<&'static str>, block_map: &'a BlockMap) -> Self{
        Self{
            open_stack: vec![Scope{
                depth: 0,
                opened: Box::new(Root{this})
            }],
            block_map
        }
    }

    fn find_scope(&self, var: &'a str) -> Result<(&'a str, &Scope)>{
        let mut scope = self.open_stack.last().unwrap();
        let mut local = var;
        loop {
            if local.starts_with("../"){
                match scope.depth{
                    0 => return Err(ParseError{ message: format!("unable to resolve scope for {}", var)}),
                    _ => {
                        local = &var[3 ..];
                        scope = self.open_stack.get(scope.depth - 1).unwrap();
                        continue;
                    }
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

    fn resolve_var(&self, var: &'a str, scope: &Scope, buffer: &mut String) -> Result<()>{
        if scope.depth == 0{
            if let Some(this) = scope.opened.this(){
                buffer.push_str(this);
                buffer.push('.');
            }
            buffer.push_str(var);
            return Ok(());
        }
        if match scope.opened.local(){
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
        let parent = &self.open_stack[scope.depth - 1];
        if let Some(this) = scope.opened.this(){
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

    fn resolve_sub_expression(&self, raw: &str, value: &str, rust: &mut Rust) -> Result<()>{
        self.resolve(&Expression { 
            expression_type: ExpressionType::Raw,
            prefix: "(",
            content: value,
            postfix: ")", 
            raw
        }, rust)
    }

    pub fn write_var(&self, expression: &Expression<'a>, rust: &mut Rust, var: Token<'a>) -> Result<Option<Token<'a>>>{
        match var.token_type{
            TokenType::PrivateVariable => {
                let (name, scope) = self.find_scope(var.value)?;
                scope.opened.resolve_private(scope.depth, expression, name, rust)?;
                Ok(Some(var))
            },
            TokenType::Literal => {
                let (name, scope) = self.find_scope(var.value)?;
                self.resolve_var(name, scope, &mut rust.code)?;
                Ok(Some(var))
            },
            TokenType::SubExpression(raw) => {
                self.resolve_sub_expression(raw, var.value, rust)?;
                Ok(Some(var))
            }
        }
    }

    fn handle_else(&self, expression: &Expression<'a>, rust: &mut Rust) -> Result<()>{
        match self.open_stack.last() {
            Some(scope) => scope.opened.handle_else(expression, rust),
            None => Err(ParseError::new("else not expected here", expression))
        }
    }

    fn resolve(&self, expression: &Expression<'a>, rust: &mut Rust) -> Result<()>{
        let token = match Token::first(&expression.content)?{
            Some(token) => token,
            None => return Err(ParseError::new("expected token", &expression))
        };
        if let TokenType::SubExpression(raw) = token.token_type{
            rust.code.push_str(expression.prefix);
            self.resolve_sub_expression(raw, token.value, rust)?;
            rust.code.push_str(expression.postfix);
            return Ok(())
        }
        match token.value{
            "lookup" => {
                rust.code.push_str(expression.prefix);
                match token.next()?{
                    Some(token) => {
                        let index = match match self.write_var(expression, rust, token)?{
                            Some(token) => token.next()?,
                            None => None
                        }{
                            Some(token) => token,
                            None => return Err(ParseError::new("lookup expects 2 arguments", &expression))
                        };
                        rust.code.push_str(".get(");
                        self.write_var(expression, rust, index)?;
                        rust.code.push(')');
                        rust.code.push_str(expression.postfix);
                        Ok(())
                    },
                    None => Err(ParseError::new("calling lookup without argments", &expression))
                }
            },
            _ => {
                rust.code.push_str(expression.prefix);
                let mut next = match self.write_var(expression, rust, token)?{
                    Some(token) => token.next()?,
                    None => None
                };
                let mut glue = '(';
                while let Some(token) = next{
                    rust.code.push(glue);
                    next = match self.write_var(expression, rust, token)?{
                        Some(token) => token.next()?,
                        None => None
                    };
                    glue = ',';
                }
                if glue == ','{
                    rust.code.push(')');
                }
                rust.code.push_str(expression.postfix);
                Ok(())
            }
        }
    }


   pub fn write_local(&self, rust: &mut String, local: &Local){
        append_with_depth(self.open_stack.len(), match local{
            Local::As(local) => local,
            _ => "this"
        }, rust);
    }

    fn close(&mut self, expression: Expression<'a>, rust: &mut Rust) -> Result<()>{
        let scope = self.open_stack.pop().ok_or_else(|| ParseError::new("Mismatched block", &expression))?;
        Ok(scope.opened.handle_close(rust))
    }

    fn open(&mut self, expression: Expression<'a>, rust: &mut Rust) -> Result<()>{
        let token = Token::first(&expression.content)?.ok_or_else(|| ParseError::new("expected token", &expression))?;
        match self.block_map.get(token.value){
            Some(block) => {
                self.open_stack.push(Scope{
                    opened: block.open(self, token, &expression, rust)?,
                    depth: self.open_stack.len()
                });
                Ok(())
            },
            None => Err(ParseError::new(&format!("unsupported helper {}", token.value), &expression))
        }
    }
}

#[derive(Clone, Copy)]
pub struct Options{
    pub root_var_name: Option<&'static str>,
    pub write_var_name: &'static str
}

pub struct Compiler{
    clean: Regex,
    options: Options,
    block_map: BlockMap
}

impl Compiler {
    pub fn new(options: Options, block_map: BlockMap) -> Self{
        Self{
            clean: Regex::new("[\\\\\"\\{\\}]").unwrap(),
            options,
            block_map
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

    fn commit_pending<'a>(&self, pending: &mut Vec<PendingWrite<'a>>, compile: &mut Compile<'a>, rust: &mut Rust) -> Result<()>{
        if pending.is_empty(){
            return Ok(());
        }
        rust.code.push_str("write!(");
        rust.code.push_str(self.options.write_var_name);
        rust.code.push_str(", \"");
        for pending in pending.iter(){
            match pending{
                PendingWrite::Raw(raw) => rust.code.push_str(self.escape(raw).as_ref()),
                PendingWrite::Expression(_) => rust.code.push_str("{}")
            }
        }
        rust.code.push('"');
        for pending in pending.iter(){
            if let PendingWrite::Expression((expression, uses, display)) = pending{
                compile.resolve(&Expression{
                    expression_type: ExpressionType::Raw,
                    prefix: ", ",
                    content: expression.content,
                    postfix: display,
                    raw: expression.raw
                }, rust)?;
                rust.using.insert(uses);
            }
        }
        rust.code.push_str(")?;");
        pending.clear();
        Ok(())
    }

    pub fn compile(&self, src: &str) -> Result<Rust>{
        let mut compile = Compile::new(self.options.root_var_name, &self.block_map);
        let mut rust = Rust::new();
        let mut pending: Vec<PendingWrite> = Vec::new();
        let mut rest = src;
        let mut expression = Expression::from(src)?;
        while let Some(expr) = expression{
            let Expression{
                expression_type,
                prefix,
                content,
                postfix,
                raw: _
            } = &expr;
            rest = postfix; 
            if !prefix.is_empty(){
                pending.push(PendingWrite::Raw(prefix));
            }
            match expression_type{
                ExpressionType::Raw => pending.push(PendingWrite::Expression((expr.clone(), USE_AS_DISPLAY, ".as_display()"))),
                ExpressionType::HtmlEscaped => if *content == "else" {
                    self.commit_pending(&mut pending, &mut compile, &mut rust)?;
                    compile.handle_else(&expr, &mut rust)?
                } else {
                    pending.push(PendingWrite::Expression((expr.clone(), USE_AS_DISPLAY_HTML, ".as_display_html()")))
                },
                ExpressionType::Open => {
                    self.commit_pending(&mut pending, &mut compile, &mut rust)?;
                    compile.open(expr, &mut rust)?
                },
                ExpressionType::Close => {
                    self.commit_pending(&mut pending, &mut compile, &mut rust)?;
                    compile.close(expr, &mut rust)?
                },
                ExpressionType::Escaped => pending.push(PendingWrite::Raw(content)),
                _ => ()
            };
            expression = expr.next()?;
        }
        if !rest.is_empty(){
            pending.push(PendingWrite::Raw(rest));
        }
        self.commit_pending(&mut pending, &mut compile, &mut rust)?;
        Ok(rust)
    }
}