use std::{collections::HashMap, fmt::Write};

use crate::{
    error::{ParseError, Result},
    expression::{Expression, ExpressionType},
    expression_tokenizer::{Token, TokenType},
};

/// Binding introduced by a block helper.
pub enum Local {
    /// A name supplied with `as name` or `as |name|`.
    As(String),
    /// The implicit `this` binding.
    This,
    /// A block without a local binding.
    None,
}

/// One open template block and its nesting depth.
pub struct Scope {
    /// Block implementation responsible for the scope.
    pub opened: Box<dyn Block>,
    /// Zero-based nesting depth.
    pub depth: usize,
}

enum PendingWrite<'a> {
    Raw(&'a str),
    Expression((Expression<'a>, DisplayKind)),
    Format((&'a str, &'a str, &'a str)),
}

#[derive(Clone, Copy)]
enum DisplayKind {
    Raw,
    HtmlEscaped,
}

impl DisplayKind {
    fn prefix(self) -> &'static str {
        match self {
            Self::Raw => ", ::rusty_handlebars::AsDisplay::as_display(&",
            Self::HtmlEscaped => ", ::rusty_handlebars::AsDisplayHtml::as_display_html(&",
        }
    }
}

/// Rust source generated for a template.
#[derive(Default)]
pub struct Rust {
    /// Statements that write the rendered template.
    pub code: String,
}

impl Rust {
    /// Creates an empty generated-source buffer.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Compiles the behavior of an open block.
pub trait Block {
    /// Writes code that closes this block.
    fn handle_close<'a>(&self, rust: &mut Rust) {
        rust.code.push_str("}");
    }

    /// Resolves a block variable such as `@index`.
    fn resolve_private<'a>(
        &self,
        _depth: usize,
        expression: &'a Expression<'a>,
        _name: &str,
        _rust: &mut Rust,
    ) -> Result<()> {
        Err(ParseError::new(
            &format!("{} not expected ", expression.content),
            expression,
        ))
    }

    /// Writes the transition to this block's `else` branch.
    fn handle_else<'a>(&self, expression: &'a Expression<'a>, _rust: &mut Rust) -> Result<()> {
        Err(ParseError::new("else not expected here", expression))
    }

    /// Returns the field path used as this block's context, if inherited.
    fn this<'a>(&self) -> Option<&str> {
        None
    }

    /// Returns the binding introduced by this block.
    fn local<'a>(&self) -> &Local {
        &Local::None
    }
}

/// Opens a named block helper.
pub trait BlockFactory {
    /// Writes the block opening and returns its active compiler state.
    fn open<'a>(
        &self,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Box<dyn Block>>;
}

/// Block helpers available to a [`Compiler`].
pub type BlockMap = HashMap<&'static str, &'static dyn BlockFactory>;

/// Mutable state used while one template is compiled.
pub struct Compile<'a> {
    /// Currently open blocks, starting with the root scope.
    pub open_stack: Vec<Scope>,
    /// Block helpers configured on the compiler.
    pub block_map: &'a BlockMap,
    helper_paths: &'a HashMap<String, String>,
}

/// Appends `_<depth>` to a generated local name.
pub fn append_with_depth(depth: usize, var: &str, buffer: &mut String) {
    buffer.push_str(var);
    buffer.push('_');
    write!(buffer, "{depth}").expect("writing to a String cannot fail");
}

struct Root<'a> {
    this: Option<&'a str>,
}

impl<'a> Block for Root<'a> {
    fn this<'b>(&self) -> Option<&str> {
        self.this
    }
}

impl<'a> Compile<'a> {
    fn new(
        this: Option<&'static str>,
        block_map: &'a BlockMap,
        helper_paths: &'a HashMap<String, String>,
    ) -> Self {
        Self {
            open_stack: vec![Scope {
                depth: 0,
                opened: Box::new(Root { this }),
            }],
            block_map,
            helper_paths,
        }
    }

    fn find_scope(&self, var: &'a str) -> Result<(&'a str, &Scope)> {
        let mut scope = self.open_stack.last().unwrap();
        let mut local = var;
        while local.starts_with("../") {
            match scope.depth {
                0 => {
                    return Err(ParseError {
                        message: format!("unable to resolve scope for {}", var),
                    })
                }
                _ => {
                    local = &local[3..];
                    scope = self.open_stack.get(scope.depth - 1).unwrap();
                }
            }
        }
        return Ok((local, scope));
    }

    fn resolve_local(
        &self,
        depth: usize,
        var: &'a str,
        local: &'a str,
        buffer: &mut String,
    ) -> bool {
        if var.starts_with(local) {
            let len = local.len();
            if var.len() > len {
                if &var[len..len + 1] != "." {
                    return false;
                }
                append_with_depth(depth, local, buffer);
                buffer.push_str(&var[len..]);
            } else {
                append_with_depth(depth, local, buffer);
            }
            return true;
        }
        return false;
    }

    fn resolve_var(&self, var: &'a str, scope: &Scope, buffer: &mut String) -> Result<()> {
        if scope.depth == 0 {
            if let Some(this) = scope.opened.this() {
                buffer.push_str(this);
                buffer.push('.');
            }
            buffer.push_str(var);
            return Ok(());
        }
        if match scope.opened.local() {
            Local::As(local) => self.resolve_local(scope.depth, var, local, buffer),
            Local::This => {
                append_with_depth(scope.depth, "this", buffer);
                if var != "this" {
                    buffer.push('.');
                    buffer.push_str(var);
                }
                true
            }
            Local::None => false,
        } {
            return Ok(());
        }
        let parent = &self.open_stack[scope.depth - 1];
        if let Some(this) = scope.opened.this() {
            self.resolve_var(this, parent, buffer)?;
            if var != this {
                buffer.push('.');
                buffer.push_str(var);
            }
        } else {
            self.resolve_var(var, parent, buffer)?;
        }
        Ok(())
    }

    fn resolve_sub_expression(&self, raw: &str, value: &str, rust: &mut Rust) -> Result<()> {
        self.resolve(
            &Expression {
                expression_type: ExpressionType::Raw,
                prefix: "",
                content: value,
                postfix: "",
                raw,
            },
            rust,
        )
    }

    /// Resolves one token and appends its Rust expression.
    pub fn write_var(
        &self,
        expression: &Expression<'a>,
        rust: &mut Rust,
        var: &Token<'a>,
    ) -> Result<()> {
        match var.token_type {
            TokenType::PrivateVariable => {
                let (name, scope) = self.find_scope(var.value)?;
                scope
                    .opened
                    .resolve_private(scope.depth, expression, name, rust)?;
            }
            TokenType::Variable => {
                let (name, scope) = self.find_scope(var.value)?;
                self.resolve_var(name, scope, &mut rust.code)?;
            }
            TokenType::Literal => {
                rust.code.push_str(var.value);
            }
            TokenType::SubExpression(raw) => {
                self.resolve_sub_expression(raw, var.value, rust)?;
            }
        }
        Ok(())
    }

    fn handle_else(&self, expression: &Expression<'a>, rust: &mut Rust) -> Result<()> {
        match self.open_stack.last() {
            Some(scope) => scope.opened.handle_else(expression, rust),
            None => Err(ParseError::new("else not expected here", expression)),
        }
    }

    fn resolve_lookup(
        &self,
        expression: &Expression<'a>,
        prefix: &str,
        postfix: char,
        args: Token<'a>,
        rust: &mut Rust,
    ) -> Result<()> {
        self.write_var(expression, rust, &args)?;
        rust.code.push_str(prefix);
        self.write_var(
            expression,
            rust,
            &args
                .next()?
                .ok_or(ParseError::new("lookup expects 2 arguments", &expression))?,
        )?;
        rust.code.push(postfix);
        Ok(())
    }

    fn resolve_helper(
        &self,
        expression: &Expression<'a>,
        name: Token<'a>,
        mut args: Token<'a>,
        rust: &mut Rust,
    ) -> Result<()> {
        match name.value {
            "lookup" => self.resolve_lookup(expression, "[", ']', args, rust),
            "try_lookup" => self.resolve_lookup(expression, ".get(", ')', args, rust),
            name => {
                rust.code
                    .push_str(self.helper_paths.get(name).map_or(name, String::as_str));
                rust.code.push('(');
                self.write_var(expression, rust, &args)?;
                loop {
                    args = match args.next()? {
                        Some(token) => {
                            rust.code.push_str(", ");
                            self.write_var(expression, rust, &token)?;
                            token
                        }
                        None => {
                            rust.code.push(')');
                            return Ok(());
                        }
                    };
                }
            }
        }
    }

    fn resolve(&self, expression: &Expression<'a>, rust: &mut Rust) -> Result<()> {
        let token = match Token::first(&expression.content)? {
            Some(token) => token,
            None => return Err(ParseError::new("expected token", &expression)),
        };
        rust.code.push_str(expression.prefix);
        if let TokenType::SubExpression(raw) = token.token_type {
            self.resolve_sub_expression(raw, token.value, rust)?;
        } else if let Some(args) = token.next()? {
            self.resolve_helper(expression, token, args, rust)?;
        } else {
            self.write_var(expression, rust, &token)?;
        }
        rust.code.push_str(expression.postfix);
        Ok(())
    }

    /// Writes a depth-qualified local binding name.
    pub fn write_local(&self, rust: &mut String, local: &Local) {
        append_with_depth(
            self.open_stack.len(),
            match local {
                Local::As(local) => local,
                _ => "this",
            },
            rust,
        );
    }

    fn close(&mut self, expression: Expression<'a>, rust: &mut Rust) -> Result<()> {
        let scope = self
            .open_stack
            .pop()
            .ok_or_else(|| ParseError::new("Mismatched block helper", &expression))?;
        Ok(scope.opened.handle_close(rust))
    }

    fn open(&mut self, expression: Expression<'a>, rust: &mut Rust) -> Result<()> {
        let token = Token::first(&expression.content)?
            .ok_or_else(|| ParseError::new("expected token", &expression))?;
        match self.block_map.get(token.value) {
            Some(block) => {
                self.open_stack.push(Scope {
                    opened: block.open(self, token, &expression, rust)?,
                    depth: self.open_stack.len(),
                });
                Ok(())
            }
            None => Err(ParseError::new(
                &format!("unsupported block helper {}", token.value),
                &expression,
            )),
        }
    }
}

/// Names used in generated Rust expressions.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// Prefix for root template variables, or `None` for variables already in scope.
    pub root_var_name: Option<&'static str>,
    /// Formatter or writer passed as the first argument to generated `write!` calls.
    pub write_var_name: &'static str,
}

/// Compiles templates into Rust source.
pub struct Compiler {
    options: Options,
    block_map: BlockMap,
    helper_paths: HashMap<String, String>,
}

impl Compiler {
    /// Creates a compiler with the supplied generated names and block helpers.
    pub fn new(options: Options, block_map: BlockMap) -> Self {
        Self {
            options,
            block_map,
            helper_paths: HashMap::new(),
        }
    }

    /// Configures Rust function paths for inline helper names.
    pub fn with_helper_paths(mut self, helper_paths: HashMap<String, String>) -> Self {
        self.helper_paths = helper_paths;
        self
    }

    fn write_escaped(content: &str, output: &mut String) {
        let mut start = 0;
        for (index, byte) in content.bytes().enumerate() {
            let escaped = match byte {
                b'{' => "{{",
                b'}' => "}}",
                b'\\' => "\\\\",
                b'"' => "\\\"",
                _ => continue,
            };
            if start < index {
                output.push_str(&content[start..index]);
            }
            output.push_str(escaped);
            start = index + 1;
        }
        if start < content.len() {
            output.push_str(&content[start..]);
        }
    }

    fn commit_pending<'a>(
        &self,
        pending: &mut Vec<PendingWrite<'a>>,
        compile: &mut Compile<'a>,
        rust: &mut Rust,
    ) -> Result<()> {
        if pending.is_empty() {
            return Ok(());
        }
        rust.code.push_str("write!(");
        rust.code.push_str(self.options.write_var_name);
        rust.code.push_str(", \"");
        for pending in pending.iter() {
            match pending {
                PendingWrite::Raw(raw) => Self::write_escaped(raw, &mut rust.code),
                PendingWrite::Expression(_) => rust.code.push_str("{}"),
                PendingWrite::Format((_, format, _)) => rust.code.push_str(format),
            }
        }
        rust.code.push('"');
        for pending in pending.iter() {
            match pending {
                PendingWrite::Expression((expression, display)) => {
                    compile.resolve(
                        &Expression {
                            expression_type: ExpressionType::Raw,
                            prefix: display.prefix(),
                            content: expression.content,
                            postfix: ")",
                            raw: expression.raw,
                        },
                        rust,
                    )?;
                }
                PendingWrite::Format((raw, _, content)) => {
                    compile.resolve(
                        &Expression {
                            expression_type: ExpressionType::Raw,
                            prefix: ", ",
                            content,
                            postfix: "",
                            raw,
                        },
                        rust,
                    )?;
                }
                _ => (),
            }
        }
        rust.code.push_str(")?;");
        pending.clear();
        Ok(())
    }

    fn select_write<'a>(
        expression: &Expression<'a>,
        display: DisplayKind,
    ) -> Result<PendingWrite<'a>> {
        if let Some(token) = Token::first(&expression.content)? {
            if let TokenType::Variable = token.token_type {
                if token.value != "format" {
                    return Ok(PendingWrite::Expression((*expression, display)));
                }
                let pattern = match token.next()? {
                    Some(token) => token,
                    _ => return Ok(PendingWrite::Expression((*expression, display))),
                };
                let value = match pattern.next() {
                    Ok(Some(token)) => token,
                    _ => return Err(ParseError::new("format requires 2 arguments", expression)),
                };
                if let TokenType::Literal = pattern.token_type {
                    if pattern.value.starts_with('"') && pattern.value.ends_with('"') {
                        return Ok(PendingWrite::Format((
                            expression.raw,
                            &pattern.value[1..pattern.value.len() - 1],
                            value.value,
                        )));
                    }
                }
                return Err(ParseError::new(
                    "first argument of format must be a string literal",
                    expression,
                ));
            }
        }
        Ok(PendingWrite::Expression((*expression, display)))
    }

    /// Compiles `src` into Rust statements.
    ///
    /// The returned source expects the configured root and writer names to be
    /// valid in the context where the statements are inserted.
    pub fn compile(&self, src: &str) -> Result<Rust> {
        let mut compile = Compile::new(
            self.options.root_var_name,
            &self.block_map,
            &self.helper_paths,
        );
        let mut rust = Rust {
            code: String::with_capacity(src.len().saturating_mul(2)),
        };
        let mut pending: Vec<PendingWrite> = Vec::with_capacity(16);
        let mut rest = src;
        let mut expression = Expression::from(src)?;
        while let Some(expr) = expression {
            let Expression {
                expression_type,
                prefix,
                content,
                postfix,
                raw: _,
            } = &expr;
            rest = postfix;
            if !prefix.is_empty() {
                pending.push(PendingWrite::Raw(prefix));
            }
            match expression_type {
                ExpressionType::Raw => pending.push(Self::select_write(&expr, DisplayKind::Raw)?),
                ExpressionType::HtmlEscaped => {
                    if *content == "else" {
                        self.commit_pending(&mut pending, &mut compile, &mut rust)?;
                        compile.handle_else(&expr, &mut rust)?
                    } else {
                        pending.push(Self::select_write(&expr, DisplayKind::HtmlEscaped)?)
                    }
                }
                ExpressionType::Open => {
                    self.commit_pending(&mut pending, &mut compile, &mut rust)?;
                    compile.open(expr, &mut rust)?
                }
                ExpressionType::Close => {
                    self.commit_pending(&mut pending, &mut compile, &mut rust)?;
                    compile.close(expr, &mut rust)?
                }
                ExpressionType::Escaped => pending.push(PendingWrite::Raw(content)),
                _ => (),
            };
            expression = expr.next()?;
        }
        if !rest.is_empty() {
            pending.push(PendingWrite::Raw(rest));
        }
        self.commit_pending(&mut pending, &mut compile, &mut rust)?;
        Ok(rust)
    }
}
