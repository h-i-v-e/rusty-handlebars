use std::cell::Cell;

use crate::{
    compiler::{append_with_depth, Block, BlockFactory, BlockMap, Compile, Local, Rust},
    error::{ParseError, Result},
    expression::Expression,
    expression_tokenizer::Token,
};
fn strip_pipes<'a>(token: Token<'a>, expression: &Expression<'a>) -> Result<&'a str> {
    loop {
        match token.next()? {
            Some(token) => {
                if token.value == "|" {
                    continue;
                }
                return Ok(token.value.trim_matches('|'));
            }
            None => return Err(ParseError::new("expected variable after as", expression)),
        }
    }
}
fn read_local<'a>(token: &Token<'a>, expression: &Expression<'a>) -> Result<Local> {
    match token.next()? {
        Some(token) => match token.value {
            "as" => Ok(Local::As(strip_pipes(token, expression)?.to_string())),
            token => Err(ParseError::new(
                &format!("unexpected token {}", token),
                expression,
            )),
        },
        None => Ok(Local::This),
    }
}
struct IfOrUnless {}

impl IfOrUnless {
    pub fn new<'a>(
        label: &str,
        prefix: &str,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<IfOrUnless> {
        match token.next()? {
            Some(var) => {
                rust.code.push_str(prefix);
                rust.code.push_str("::rusty_handlebars::AsBool::as_bool(&");
                compile.write_var(expression, rust, &var)?;
                rust.code.push_str("){");
                Ok(Self {})
            }
            None => Err(ParseError::new(
                &format!("expected variable after {}", label),
                expression,
            )),
        }
    }
}

impl Block for IfOrUnless {
    fn handle_else<'a>(&self, _expression: &'a Expression<'a>, rust: &mut Rust) -> Result<()> {
        rust.code.push_str("}else{");
        Ok(())
    }
}
struct IfFty {}

impl BlockFactory for IfFty {
    fn open<'a>(
        &self,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Box<dyn Block>> {
        Ok(Box::new(IfOrUnless::new(
            "if", "if ", compile, token, expression, rust,
        )?))
    }
}
struct UnlessFty {}

impl BlockFactory for UnlessFty {
    fn open<'a>(
        &self,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Box<dyn Block>> {
        Ok(Box::new(IfOrUnless::new(
            "unless", "if !", compile, token, expression, rust,
        )?))
    }
}
struct IfSome {
    local: Local,
}

impl IfSome {
    fn new<'a>(
        by_ref: bool,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Self> {
        let next = token.next()?.ok_or_else(|| {
            ParseError::new(
                &format!(
                    "expected variable after if_some{}",
                    if by_ref { "_ref" } else { "" }
                ),
                expression,
            )
        })?;
        let local = read_local(&next, expression)?;
        rust.code.push_str("if let Some(");
        compile.write_local(&mut rust.code, &local);
        rust.code.push_str(") = ");
        if by_ref {
            rust.code.push('&');
        }
        compile.write_var(expression, rust, &next)?;
        rust.code.push('{');
        Ok(Self { local })
    }
}

impl Block for IfSome {
    fn handle_else<'a>(&self, _expression: &'a Expression<'a>, rust: &mut Rust) -> Result<()> {
        rust.code.push_str("}else{");
        Ok(())
    }
    fn local<'a>(&self) -> &Local {
        &self.local
    }
}
struct IfSomeFty {}

impl BlockFactory for IfSomeFty {
    fn open<'a>(
        &self,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Box<dyn Block>> {
        Ok(Box::new(IfSome::new(
            false, compile, token, expression, rust,
        )?))
    }
}
struct IfSomeRefFty {}

impl BlockFactory for IfSomeRefFty {
    fn open<'a>(
        &self,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Box<dyn Block>> {
        Ok(Box::new(IfSome::new(
            true, compile, token, expression, rust,
        )?))
    }
}
struct With {
    local: Local,
}

impl With {
    pub fn new<'a>(
        by_ref: bool,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Self> {
        let next = token.next()?.ok_or_else(|| {
            ParseError::new(
                &format!(
                    "expected variable after with{}",
                    if by_ref { "_ref" } else { "" }
                ),
                expression,
            )
        })?;
        let local = read_local(&next, expression)?;
        rust.code.push_str("{let ");
        compile.write_local(&mut rust.code, &local);
        rust.code.push_str(" = ");
        if by_ref {
            rust.code.push('&');
        }
        compile.write_var(expression, rust, &next)?;
        rust.code.push(';');
        Ok(Self { local })
    }
}

impl Block for With {
    fn local<'a>(&self) -> &Local {
        &self.local
    }
}
struct WithFty {}

impl BlockFactory for WithFty {
    fn open<'a>(
        &self,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Box<dyn Block>> {
        Ok(Box::new(With::new(
            false, compile, token, expression, rust,
        )?))
    }
}
struct WithRefFty {}

impl BlockFactory for WithRefFty {
    fn open<'a>(
        &self,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Box<dyn Block>> {
        Ok(Box::new(With::new(true, compile, token, expression, rust)?))
    }
}
struct Each {
    local: Local,
    depth: usize,
    code_start: usize,
    pattern_start: usize,
    pattern_end: usize,
    expression_start: usize,
    expression_end: usize,
    body_start: Cell<usize>,
    header_shift: Cell<usize>,
    uses_index: Cell<bool>,
    has_else: Cell<bool>,
}

impl Each {
    pub fn new<'a>(
        by_ref: bool,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Self> {
        let next = match token.next()? {
            Some(next) => next,
            None => {
                return Err(ParseError::new(
                    &format!(
                        "expected variable after {}",
                        if by_ref { "each_ref" } else { "each" }
                    ),
                    expression,
                ))
            }
        };
        let local = read_local(&next, expression)?;
        let depth = compile.open_stack.len();
        let code_start = rust.code.len();
        rust.code.push_str("for ");
        let pattern_start = rust.code.len();
        compile.write_local(&mut rust.code, &local);
        let pattern_end = rust.code.len();
        rust.code.push_str(" in ");
        let expression_start = rust.code.len();
        if by_ref {
            rust.code.push('&');
        }
        compile.write_var(expression, rust, &next)?;
        let expression_end = rust.code.len();
        rust.code.push('{');
        let body_start = rust.code.len();
        Ok(Self {
            local,
            depth,
            code_start,
            pattern_start,
            pattern_end,
            expression_start,
            expression_end,
            body_start: Cell::new(body_start),
            header_shift: Cell::new(0),
            uses_index: Cell::new(false),
            has_else: Cell::new(false),
        })
    }

    fn write_indexer(&self, rust: &mut Rust) {
        if !self.uses_index.replace(true) {
            const ITERATOR_PREFIX: &str = "::std::iter::IntoIterator::into_iter(";
            const ITERATOR_SUFFIX: &str = ").enumerate()";
            let shift = self.header_shift.get();
            rust.code
                .insert_str(self.expression_end + shift, ITERATOR_SUFFIX);
            rust.code
                .insert_str(self.expression_start + shift, ITERATOR_PREFIX);
            rust.code.insert(self.pattern_end + shift, ')');

            let mut pattern_prefix = String::from("(");
            append_with_depth(self.depth, "_index", &mut pattern_prefix);
            pattern_prefix.push(',');
            rust.code
                .insert_str(self.pattern_start + shift, &pattern_prefix);

            self.body_start.set(
                self.body_start.get()
                    + ITERATOR_SUFFIX.len()
                    + ITERATOR_PREFIX.len()
                    + 1
                    + pattern_prefix.len(),
            );
        }
        append_with_depth(self.depth, "_index", &mut rust.code);
    }

    fn write_map_var<'a>(&self, depth: usize, suffix: &str, rust: &mut Rust) {
        append_with_depth(
            depth,
            if let Local::As(name) = &self.local {
                name.as_str()
            } else {
                "this"
            },
            &mut rust.code,
        );
        rust.code.push_str(suffix)
    }
}

impl Block for Each {
    fn handle_else<'a>(&self, expression: &'a Expression<'a>, rust: &mut Rust) -> Result<()> {
        if self.has_else.replace(true) {
            return Err(ParseError::new("duplicate else", expression));
        }

        let mut assignment = String::new();
        append_with_depth(self.depth, "_empty", &mut assignment);
        assignment.push_str("=false;");
        rust.code.insert_str(self.body_start.get(), &assignment);

        let mut opening = String::from("{let mut ");
        append_with_depth(self.depth, "_empty", &mut opening);
        opening.push_str("=true;");
        rust.code.insert_str(self.code_start, &opening);
        self.header_shift
            .set(self.header_shift.get() + opening.len());

        rust.code.push_str("}if ");
        append_with_depth(self.depth, "_empty", &mut rust.code);
        rust.code.push('{');
        Ok(())
    }

    fn resolve_private<'a>(
        &self,
        depth: usize,
        expression: &'a Expression<'a>,
        name: &str,
        rust: &mut Rust,
    ) -> Result<()> {
        Ok(match name {
            "index" if !self.has_else.get() => self.write_indexer(rust),
            "index" => Err(ParseError::new(
                "@index is not available in an each else branch",
                expression,
            ))?,
            "key" => self.write_map_var(depth, ".0", rust),
            "value" => self.write_map_var(depth, ".1", rust),
            _ => Err(ParseError::new(
                &format!("unexpected variable {}", name),
                expression,
            ))?,
        })
    }

    fn handle_close<'a>(&self, rust: &mut Rust) {
        rust.code.push('}');
        if self.has_else.get() {
            rust.code.push('}');
        }
    }

    fn local<'a>(&self) -> &Local {
        &self.local
    }
}
struct EachFty {}

impl BlockFactory for EachFty {
    fn open<'a>(
        &self,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Box<dyn Block>> {
        Ok(Box::new(Each::new(
            false, compile, token, expression, rust,
        )?))
    }
}
struct EachRefFty {}

impl BlockFactory for EachRefFty {
    fn open<'a>(
        &self,
        compile: &'a Compile<'a>,
        token: Token<'a>,
        expression: &'a Expression<'a>,
        rust: &mut Rust,
    ) -> Result<Box<dyn Block>> {
        Ok(Box::new(Each::new(true, compile, token, expression, rust)?))
    }
}

const IF: IfFty = IfFty {};
const UNLESS: UnlessFty = UnlessFty {};
const IF_SOME: IfSomeFty = IfSomeFty {};
const IF_SOME_REF: IfSomeRefFty = IfSomeRefFty {};
const WITH: WithFty = WithFty {};
const WITH_REF: WithRefFty = WithRefFty {};
const EACH: EachFty = EachFty {};
const EACH_REF: EachRefFty = EachRefFty {};
/// Registers the block helpers supported by the built-in template syntax.
pub fn add_builtins(map: &mut BlockMap) {
    map.reserve(8);
    map.insert("if", &IF);
    map.insert("unless", &UNLESS);
    map.insert("if_some", &IF_SOME);
    map.insert("if_some_ref", &IF_SOME_REF);
    map.insert("with", &WITH);
    map.insert("with_ref", &WITH_REF);
    map.insert("each", &EACH);
    map.insert("each_ref", &EACH_REF);
}
