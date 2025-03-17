//! Handlebars template compilation
//!
//! This module provides functionality for compiling Handlebars templates into Rust code.
//! It handles:
//! - Variable resolution and scope management
//! - Block helper compilation
//! - Expression evaluation
//! - HTML escaping
//!
//! # Compilation Process
//!
//! The compilation process involves:
//! 1. Parsing the template into expressions
//! 2. Resolving variables and scopes
//! 3. Compiling block helpers
//! 4. Generating Rust code
//!
//! # Examples
//!
//! Basic usage:
//! ```rust
//! use rusty_handlebars_parser::compiler::{Compiler, Options};
//! use rusty_handlebars_parser::block::add_builtins;
//!
//! let mut block_map = HashMap::new();
//! add_builtins(&mut block_map);
//!
//! let options = Options {
//!     root_var_name: Some("data"),
//!     write_var_name: "write"
//! };
//!
//! let compiler = Compiler::new(options, block_map);
//! let rust = compiler.compile("Hello {{name}}!")?;
//! ```
//!
//! Complex template example:
//! ```rust
//! use rusty_handlebars_parser::compiler::{Compiler, Options};
//! use rusty_handlebars_parser::block::add_builtins;
//!
//! let mut block_map = HashMap::new();
//! add_builtins(&mut block_map);
//!
//! let options = Options {
//!     root_var_name: Some("data"),
//!     write_var_name: "write"
//! };
//!
//! let template = r#"
//! <div class="user-profile">
//!     {{#if user}}
//!         <h1>{{user.name}}</h1>
//!         {{#if user.bio}}
//!             <p class="bio">{{user.bio}}</p>
//!         {{else}}
//!             <p class="no-bio">No bio available</p>
//!         {{/if}}
//!         
//!         {{#if_some user.posts as post}}
//!             <div class="posts">
//!                 <h2>Posts</h2>
//!                 {{#each post as post}}
//!                     <article class="post">
//!                         <h3>{{post.title}}</h3>
//!                         <p>{{post.content}}</p>
//!                         <div class="meta">
//!                             <span>Posted on {{post.date}}</span>
//!                             {{#if post.tags}}
//!                                 <div class="tags">
//!                                     {{#each post.tags as tag}}
//!                                         <span class="tag">{{tag}}</span>
//!                                     {{/each}}
//!                                 </div>
//!                             {{/if}}
//!                         </div>
//!                     </article>
//!                 {{/each}}
//!             </div>
//!         {{/if_some}}
//!     {{else}}
//!         <p>Please log in to view your profile</p>
//!     {{/if}}
//! </div>
//! "#;
//!
//! let compiler = Compiler::new(options, block_map);
//! let rust = compiler.compile(template)?;
//! ```
//!
//! This example demonstrates:
//! - Nested conditional blocks with `if` and `else`
//! - Option handling with `if_some`
//! - Collection iteration with `each`
//! - HTML escaping for safe output
//! - Complex variable resolution
//! - Block scope management
//! - Template structure and formatting

use std::{borrow::Cow, collections::{HashMap, HashSet}, fmt::{Display, Write}};

use regex::{Captures, Regex};

use crate::{error::{ParseError, Result}, expression::{Expression, ExpressionType}, expression_tokenizer::{Token, TokenType}};

/// Local variable declaration in a block
pub enum Local{
    /// Named local variable: `as name`
    As(String),
    /// This context: `this`
    This,
    /// No local variable
    None
}

/// A scope in the template
pub struct Scope{
    /// The block that opened this scope
    pub opened: Box<dyn Block>,
    /// The depth of this scope
    pub depth: usize
}

/// A pending write operation
enum PendingWrite<'a>{
    /// Raw text to write
    Raw(&'a str),
    /// Expression to evaluate and write
    Expression((Expression<'a>, &'static str, &'static str))
}

/// Rust code generation state
pub struct Rust{
    /// Set of used traits
    pub using: HashSet<String>,
    /// Generated code
    pub code: String
}

/// Trait for HTML escaping
pub static USE_AS_DISPLAY: &str = "AsDisplay";
/// Trait for raw HTML output
pub static USE_AS_DISPLAY_HTML: &str = "AsDisplayHtml";

/// Helper for formatting use statements
pub struct Uses<'a>{
    uses: &'a HashSet<String>
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
    /// Creates a new Rust code generator
    pub fn new() -> Self{
        Self{
            using: HashSet::new(),
            code: String::new()
        }
    }

    /// Returns a formatter for use statements
    pub fn uses(&self) -> Uses{
        Uses{ uses: &self.using}
    }
}

/// Trait for block helpers
pub trait Block{
    /// Handles block closing
    fn handle_close<'a>(&self, rust: &mut Rust) {
        rust.code.push_str("}");
    }

    /// Resolves a private variable
    fn resolve_private<'a>(&self, _depth: usize, expression: &'a Expression<'a>, _name: &str, _rust: &mut Rust) -> Result<()>{
        Err(ParseError::new(&format!("{} not expected ", expression.content), expression))
    }

    /// Handles else block
    fn handle_else<'a>(&self, expression: &'a Expression<'a>, _rust: &mut Rust) -> Result<()>{
        Err(ParseError::new("else not expected here", expression))
    }

    /// Returns the this context
    fn this<'a>(&self) -> Option<&str>{
        None
    }

    /// Returns the local variable
    fn local<'a>(&self) -> &Local{
        &Local::None
    }
}

/// Trait for block helper factories
pub trait BlockFactory{
    /// Opens a new block
    fn open<'a>(&self, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Box<dyn Block>>;
}

/// Map of block helper names to factories
pub type BlockMap = HashMap<&'static str, &'static dyn BlockFactory>;

/// Compiler state
pub struct Compile<'a>{
    /// Stack of open blocks
    pub open_stack: Vec<Scope>,
    /// Map of block helpers
    pub block_map: &'a BlockMap
}

/// Appends a depth suffix to a variable name
pub fn append_with_depth(depth: usize, var: &str, buffer: &mut String){
    buffer.push_str(var);
    buffer.push('_');
    buffer.push_str(depth.to_string().as_str());
}

/// Root block implementation
struct Root<'a>{
    this: Option<&'a str>
}

impl<'a> Block for Root<'a>{
    fn this<'b>(&self) -> Option<&str>{
        self.this
    }
}

impl<'a> Compile<'a>{
    /// Creates a new compiler
    fn new(this: Option<&'static str>, block_map: &'a BlockMap) -> Self{
        Self{
            open_stack: vec![Scope{
                depth: 0,
                opened: Box::new(Root{this})
            }],
            block_map
        }
    }

    /// Finds the scope for a variable
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

    /// Resolves a local variable
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

    /// Resolves a variable in a scope
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

    /// Resolves a sub-expression
    fn resolve_sub_expression(&self, raw: &str, value: &str, rust: &mut Rust) -> Result<()>{
        self.resolve(&Expression { 
            expression_type: ExpressionType::Raw,
            prefix: "",
            content: value,
            postfix: "", 
            raw
        }, rust)
    }

    /// Writes a variable expression
    pub fn write_var(&self, expression: &Expression<'a>, rust: &mut Rust, var: &Token<'a>) -> Result<()>{
        match var.token_type{
            TokenType::PrivateVariable => {
                let (name, scope) = self.find_scope(var.value)?;
                scope.opened.resolve_private(scope.depth, expression, name, rust)?;
            },
            TokenType::Literal => {
                let (name, scope) = self.find_scope(var.value)?;
                self.resolve_var(name, scope, &mut rust.code)?;
            },
            TokenType::SubExpression(raw) => {
                self.resolve_sub_expression(raw, var.value, rust)?;
            }
        }
        Ok(())
    }

    /// Handles an else block
    fn handle_else(&self, expression: &Expression<'a>, rust: &mut Rust) -> Result<()>{
        match self.open_stack.last() {
            Some(scope) => scope.opened.handle_else(expression, rust),
            None => Err(ParseError::new("else not expected here", expression))
        }
    }

    /// Resolves a lookup expression
    fn resolve_lookup(&self, expression: &Expression<'a>, prefix: &str, postfix: char, args: Token<'a>, rust: &mut Rust) -> Result<()>{
        self.write_var(expression, rust, &args)?;
        rust.code.push_str(prefix);
        self.write_var(expression, rust, &args.next()?.ok_or(
            ParseError::new("lookup expects 2 arguments", &expression))?
        )?;
        rust.code.push(postfix);
        Ok(())
    }

    /// Resolves a helper expression
    fn resolve_helper(&self, expression: &Expression<'a>, name: Token<'a>, mut args: Token<'a>, rust: &mut Rust) -> Result<()>{
        match name.value{
            "lookup" => self.resolve_lookup(expression, "[", ']', args, rust),
            "try_lookup" => self.resolve_lookup(expression, ".get(", ')', args, rust),
            name => {
                rust.code.push_str(name);
                rust.code.push('(');
                self.write_var(expression, rust, &args)?;
                loop {
                    args = match args.next()?{
                        Some(token) => {
                            rust.code.push_str(", ");
                            self.write_var(expression, rust, &token)?;
                            token
                        },
                        None => {
                            rust.code.push(')');
                            return Ok(());
                        }
                    };
                }
            }
        }
    }

    /// Resolves an expression
    fn resolve(&self, expression: &Expression<'a>, rust: &mut Rust) -> Result<()>{
        let token = match Token::first(&expression.content)?{
            Some(token) => token,
            None => return Err(ParseError::new("expected token", &expression))
        };
        rust.code.push_str(expression.prefix);
        if let TokenType::SubExpression(raw) = token.token_type{
            self.resolve_sub_expression(raw, token.value, rust)?;
        }
        else if let Some(args) = token.next()?{
            self.resolve_helper(expression, token, args, rust)?;
        }
        else{
            self.write_var(expression, rust, &token)?;
        }
        rust.code.push_str(expression.postfix);
        Ok(())
    }

    /// Writes a local variable declaration
    pub fn write_local(&self, rust: &mut String, local: &Local){
        append_with_depth(self.open_stack.len(), match local{
            Local::As(local) => local,
            _ => "this"
        }, rust);
    }

    /// Closes a block
    fn close(&mut self, expression: Expression<'a>, rust: &mut Rust) -> Result<()>{
        let scope = self.open_stack.pop().ok_or_else(|| ParseError::new("Mismatched block helper", &expression))?;
        Ok(scope.opened.handle_close(rust))
    }

    /// Opens a block
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
            None => Err(ParseError::new(&format!("unsupported block helper {}", token.value), &expression))
        }
    }
}

/// Compiler options
#[derive(Debug, Clone, Copy)]
pub struct Options{
    /// Name of the root variable
    pub root_var_name: Option<&'static str>,
    /// Name of the write function
    pub write_var_name: &'static str
}

/// Main compiler implementation
pub struct Compiler{
    /// Regex for cleaning whitespace
    clean: Regex,
    /// Compiler options
    options: Options,
    /// Map of block helpers
    block_map: BlockMap
}

impl Compiler {
    /// Creates a new compiler
    pub fn new(options: Options, block_map: BlockMap) -> Self{
        Self{
            clean: Regex::new("[\\\\\"\\{\\}]").unwrap(),
            options,
            block_map
        }
    }

    /// Escapes HTML content
    fn escape<'a>(&self, content: &'a str) -> Cow<'a, str> {
        self.clean.replace_all(
            &content, |captures: &Captures| match &captures[0]{
                "{" | "}" => format!("{}{}", &captures[0], &captures[0]),
                _ => format!("\\{}", &captures[0])
            }
        )
    }

    /// Commits pending writes
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
                rust.using.insert(uses.to_string());
            }
        }
        rust.code.push_str(")?;");
        pending.clear();
        Ok(())
    }

    /// Compiles a template
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