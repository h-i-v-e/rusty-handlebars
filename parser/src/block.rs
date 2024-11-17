use crate::{compiler::{append_with_depth, Block, BlockFactory, BlockMap, Compile, Local, Rust}, error::{ParseError, Result}, expression::{Expression, ExpressionType}, expression_tokenizer::Token};

fn strip_pipes<'a>(token: Token<'a>, expression: &Expression<'a>) -> Result<&'a str>{
    loop{
        match token.next()?{
            Some(token) => {
                if token.value == "|"{
                    continue;
                }
                return Ok(token.value.trim_matches('|'));
            },
            None => return Err(ParseError::new("expected variable after as", expression))
        }
    }
}

fn read_local<'a>(token: &Token<'a>, expression: &Expression<'a>) -> Result<Local>{
    match token.next()?{
        Some(token) => {
            match token.value{
                "as" => Ok(Local::As(strip_pipes(token, expression)?.to_string())),
                token => Err(ParseError::new(&format!("unexpected token {}", token), expression))
            }
        },
        None => Ok(Local::This)
    }
}

struct IfOrUnless{}

impl IfOrUnless{
    pub fn new<'a>(label: &str, prefix: &str, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<IfOrUnless>{
        match token.next()? {
            Some(var) => {
                rust.using.insert("AsBool");
                rust.code.push_str(prefix);
                compile.write_var(expression, rust, var)?;
                rust.code.push_str(".as_bool(){");
                Ok(Self{})
            },
            None => Err(ParseError::new(&format!("expected variable after {}", label), expression))
        }
    }
}

impl Block for IfOrUnless{
    fn handle_else<'a>(&self,  _expression: &'a Expression<'a>, rust: &mut Rust) -> Result<()>{
        rust.code.push_str("}else{");
        Ok(())
    }
}

struct IfFty{}

impl BlockFactory for IfFty{
    fn open<'a>(&self, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Box<dyn Block>>{
        Ok(Box::new(IfOrUnless::new("if", "if ", compile, token, expression, rust)?))
    }
}

struct UnlessFty{}

impl BlockFactory for UnlessFty{
    fn open<'a>(&self, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Box<dyn Block>>{
        Ok(Box::new(IfOrUnless::new("unless", "if !", compile, token, expression, rust)?))
    }
}

struct IfSome{
    local: Local
}

impl IfSome{
    fn new<'a>(by_ref: bool, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Self>{
        let next = token.next()?.ok_or_else(|| ParseError::new(
            &format!("expected variable after if_some{}", if by_ref {"_ref"} else {""}), expression
        ))?;
        let local = read_local(&next, expression)?;
        rust.code.push_str("if let Some(");
        compile.write_local(&mut rust.code, &local);
        rust.code.push_str(") = ");
        if by_ref{
            rust.code.push('&');
        }
        compile.write_var(expression, rust, next)?;
        rust.code.push('{');
        Ok(Self{local})
    }
}

impl Block for IfSome{
    fn handle_else<'a>(&self, _expression: &'a Expression<'a>, rust: &mut Rust) -> Result<()>{
        rust.code.push_str("}else{");
        Ok(())
    }

    fn local<'a>(&self) -> &Local {
        &self.local
    }
}

struct IfSomeFty{}

impl BlockFactory for IfSomeFty{
    fn open<'a>(&self, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Box<dyn Block>>{
        Ok(Box::new(IfSome::new(false, compile, token, expression, rust)?))
    }
}

struct IfSomeRefFty{}

impl BlockFactory for IfSomeRefFty{
    fn open<'a>(&self, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Box<dyn Block>>{
        Ok(Box::new(IfSome::new(true, compile, token, expression, rust)?))
    }
}

struct With{
    local: Local
}

impl With{
    pub fn new<'a>(by_ref: bool, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Self>{
        let next = token.next()?.ok_or_else(|| ParseError::new(
            &format!("expected variable after with{}", if by_ref {"_ref"} else {""}), expression
        ))?;
        let local = read_local(&next, expression)?;
        rust.code.push_str("{let ");
        compile.write_local(&mut rust.code, &local);
        rust.code.push_str(" = ");
        if by_ref{
            rust.code.push('&');
        }
        compile.write_var(expression, rust, next)?;
        rust.code.push(';');
        Ok(Self{local})
    }
}

impl Block for With{
    fn local<'a>(&self) -> &Local {
        &self.local
    }
}

struct WithFty{}

impl BlockFactory for WithFty{
    fn open<'a>(&self, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Box<dyn Block>>{
        Ok(Box::new(With::new(false, compile, token, expression, rust)?))
    }
}

struct WithRefFty{}

impl BlockFactory for WithRefFty{
    fn open<'a>(&self, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Box<dyn Block>>{
        Ok(Box::new(With::new(true, compile, token, expression, rust)?))
    }
}

struct Each{
    local: Local,
    indexer: Option<String>,
    has_else: bool
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

fn check_for_else(src: &str) -> Result<bool>{
    let mut exp = Expression::from(src)?;
    let mut depth = 1;
    while let Some(expr) = &exp{
        match expr.expression_type{
            ExpressionType::Comment | ExpressionType::Escaped => continue,
            ExpressionType::Open => depth += 1,
            ExpressionType::Close => {
                depth -= 1;
                if depth == 0{
                    return Ok(false);
                }
            },
            _ => if expr.content == "else" && depth == 1{
                return Ok(true);
            }
        }
        exp = expr.next()?;
    }
    Ok(false)
}

impl Each{
    pub fn new<'a>(by_ref: bool, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Self>{
        let next = match token.next()?{
            Some(next) => next,
            None => return Err(ParseError::new(&format!("expected variable after {}", if by_ref{"each_ref"}else{"each"}), expression))
        };
        let indexer = check_for_indexer(&expression.postfix).map(|found| match found{
            true => {
                let indexer = format!("i_{}", compile.open_stack.len());
                rust.code.push_str("let mut ");
                rust.code.push_str(indexer.as_str());
                rust.code.push_str(" = 0;");
                Some(indexer)
            },
            false => None
        })?;
        let local = read_local(&next, expression)?;
        let has_else = check_for_else(&expression.postfix)?;
        if has_else{
            rust.code.push_str("{let mut empty = true;");
        }
        rust.code.push_str("for ");
        compile.write_local(&mut rust.code, &local);
        rust.code.push_str(" in ");
        if by_ref{
            rust.code.push('&');
        }
        compile.write_var(expression, rust, next)?;
        rust.code.push('{');
        if has_else{
            rust.code.push_str("empty = false;");
        }
        Ok(Self{
            local,
            indexer,
            has_else
        })
    }

    fn write_map_var<'a>(&self, depth: usize, suffix: &str, rust: &mut Rust){
        append_with_depth(depth, if let Local::As(name) = &self.local{
            name.as_str()
        } else{
            "this"
        }, &mut rust.code);
        rust.code.push_str(suffix)
    }

    fn write_indexer<'a>(&self, rust: &mut Rust){
        if let Some(indexer) = &self.indexer{
            rust.code.push_str(indexer);
            rust.code.push_str(" += 1;");
        }
    }
}

impl Block for Each{
    fn handle_else<'a>(&self, _expression: &'a Expression<'a>, rust: &mut Rust) -> Result<()> {
        self.write_indexer(rust);
        rust.code.push_str("} if empty {");
        Ok(())
    }

    fn resolve_private<'a>(&self, depth: usize, expression: &'a Expression<'a>, name: &str, rust: &mut Rust) -> Result<()>{
        Ok(match name{
            "index" => rust.code.push_str(&self.indexer.as_ref().unwrap()),
            "key" => self.write_map_var(depth, ".0", rust),
            "value" => self.write_map_var(depth, ".1", rust),
            _ => Err(ParseError::new(&format!("unexpected variable {}", name), expression))?
        })
    }

    fn handle_close<'a>(&self, rust: &mut Rust) {
        if self.has_else{
            rust.code.push_str("}}");
        }
        else{
            self.write_indexer(rust);
            rust.code.push('}');
        }
    }

    fn local<'a>(&self) -> &Local {
        &self.local
    }
}

struct EachFty{}

impl BlockFactory for EachFty{
    fn open<'a>(&self, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Box<dyn Block>>{
        Ok(Box::new(Each::new(false, compile, token, expression, rust)?))
    }
}

struct EachRefFty{}

impl BlockFactory for EachRefFty{
    fn open<'a>(&self, compile: &'a Compile<'a>, token: Token<'a>, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<Box<dyn Block>>{
        Ok(Box::new(Each::new(true, compile, token, expression, rust)?))
    }
}

static IF: IfFty = IfFty{};
static UNLESS: UnlessFty = UnlessFty{};
static IF_SOME: IfSomeFty = IfSomeFty{};
static IF_SOME_REF: IfSomeRefFty = IfSomeRefFty{};
static WITH: WithFty = WithFty{};
static WITH_REF: WithRefFty = WithRefFty{};
static EACH: EachFty = EachFty{};
static EACH_REF: EachRefFty = EachRefFty{};

pub fn add_builtins(map: &mut BlockMap){
    map.insert("if", &IF);
    map.insert("unless", &UNLESS);
    map.insert("if_some", &IF_SOME);
    map.insert("if_some_ref", &IF_SOME_REF);
    map.insert("with", &WITH);
    map.insert("with_ref", &WITH_REF);
    map.insert("each", &EACH);
    map.insert("each_ref", &EACH_REF);
}