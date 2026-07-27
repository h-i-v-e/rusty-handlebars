use std::fmt;

/// A half-open byte range in the original template source.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub const fn contains(self, offset: usize) -> bool {
        self.start <= offset && offset <= self.end
    }
}

/// The severity of a template diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
}

/// A stable, machine-readable diagnostic identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    UnclosedExpression,
    UnclosedBlock,
    MismatchedBlock,
    UnexpectedElse,
    DuplicateElse,
    InvalidToken,
    UnterminatedString,
    UnmatchedSubexpression,
    InvalidHelperArguments,
    UnknownPrivateVariable,
}

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnclosedExpression => "unclosed-expression",
            Self::UnclosedBlock => "unclosed-block",
            Self::MismatchedBlock => "mismatched-block",
            Self::UnexpectedElse => "unexpected-else",
            Self::DuplicateElse => "duplicate-else",
            Self::InvalidToken => "invalid-token",
            Self::UnterminatedString => "unterminated-string",
            Self::UnmatchedSubexpression => "unmatched-subexpression",
            Self::InvalidHelperArguments => "invalid-helper-arguments",
            Self::UnknownPrivateVariable => "unknown-private-variable",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A problem found while parsing a template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub span: Span,
    pub message: String,
    pub code: DiagnosticCode,
    pub severity: Severity,
}

impl Diagnostic {
    fn error(code: DiagnosticCode, span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            code,
            severity: Severity::Error,
        }
    }
}

/// The lexical role of a token inside an expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxTokenKind {
    Variable,
    PrivateVariable,
    String,
    Number,
    Keyword,
    Subexpression,
    Punctuation,
}

/// A borrowed expression token with its exact source range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxToken<'source> {
    pub kind: SyntaxTokenKind,
    pub text: &'source str,
    pub span: Span,
}

/// One parsed template node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<'source> {
    pub span: Span,
    pub kind: NodeKind<'source>,
}

/// A syntax node that retains slices of the original template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind<'source> {
    Text(&'source str),
    Comment {
        content: &'source str,
        content_span: Span,
    },
    Interpolation {
        escaped: bool,
        expression_span: Span,
        tokens: Vec<SyntaxToken<'source>>,
    },
    Block(BlockNode<'source>),
    RawBlock {
        name: &'source str,
        name_span: Span,
        content: &'source str,
        content_span: Span,
        open_span: Span,
        close_span: Option<Span>,
    },
    Error(&'source str),
}

/// A block helper and its nested bodies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockNode<'source> {
    pub name: &'source str,
    pub name_span: Span,
    pub open_span: Span,
    pub expression_span: Span,
    pub arguments: Vec<SyntaxToken<'source>>,
    pub alias: Option<SyntaxToken<'source>>,
    pub body: Vec<Node<'source>>,
    pub else_span: Option<Span>,
    pub else_body: Vec<Node<'source>>,
    pub close_span: Option<Span>,
}

impl BlockNode<'_> {
    pub fn full_span(&self) -> Span {
        Span::new(
            self.open_span.start,
            self.close_span.map_or_else(
                || {
                    self.body
                        .last()
                        .map_or(self.open_span.end, |node| node.span.end)
                },
                |span| span.end,
            ),
        )
    }
}

/// A recoverable parse result. Nodes remain useful even when diagnostics exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTemplate<'source> {
    pub source: &'source str,
    pub nodes: Vec<Node<'source>>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ParsedTemplate<'_> {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}

struct Parser<'source> {
    source: &'source str,
    offset: usize,
    diagnostics: Vec<Diagnostic>,
}

enum Terminator<'source> {
    End,
    Else(Span),
    Close {
        name: &'source str,
        name_span: Span,
        span: Span,
    },
}

impl<'source> Parser<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
            offset: 0,
            diagnostics: Vec::new(),
        }
    }

    fn parse(mut self) -> ParsedTemplate<'source> {
        let mut nodes = Vec::new();
        loop {
            let (parsed, terminator) = self.parse_nodes(None);
            nodes.extend(parsed);
            match terminator {
                Terminator::Else(span) => self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnexpectedElse,
                    span,
                    "`else` is only valid inside a block",
                )),
                Terminator::Close {
                    name, name_span, ..
                } => self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::MismatchedBlock,
                    name_span,
                    format!("closing block `{name}` has no matching opening block"),
                )),
                Terminator::End => break,
            }
        }
        validate_private_variables(&nodes, &mut self.diagnostics, &[]);
        ParsedTemplate {
            source: self.source,
            nodes,
            diagnostics: self.diagnostics,
        }
    }

    fn parse_nodes(
        &mut self,
        expected_close: Option<&str>,
    ) -> (Vec<Node<'source>>, Terminator<'source>) {
        let mut nodes = Vec::new();
        while self.offset < self.source.len() {
            let rest = &self.source[self.offset..];
            let Some(relative_open) = rest.find("{{") else {
                self.push_text(&mut nodes, self.offset, self.source.len());
                self.offset = self.source.len();
                return (nodes, Terminator::End);
            };
            let open = self.offset + relative_open;
            if open > self.offset {
                if open > 0 && self.source.as_bytes()[open - 1] == b'\\' {
                    self.push_text(&mut nodes, self.offset, open - 1);
                    if let Some(close) = self.source[open + 2..].find("}}") {
                        let content_start = open + 2;
                        let content_end = content_start + close;
                        let end = content_end + 2;
                        self.push_text(&mut nodes, content_start, content_end);
                        self.offset = end;
                        continue;
                    }
                }
                self.push_text(&mut nodes, self.offset, open);
            }
            self.offset = open;

            if self.source[open..].starts_with("{{{{") {
                self.parse_raw_block(&mut nodes);
                continue;
            }

            let triple = self.source[open..].starts_with("{{{");
            let opening_len = if triple { 3 } else { 2 };
            let closing = if triple { "}}}" } else { "}}" };
            let content_start = open + opening_len;
            let remaining = &self.source[content_start..];
            let long_comment = !triple && remaining.starts_with("!--");
            let relative_close = if long_comment {
                remaining.find("--}}").map(|close| close + 2)
            } else {
                remaining
                    .find(closing)
                    .filter(|close| remaining.find('\n').is_none_or(|newline| newline > *close))
            };
            let Some(relative_close) = relative_close else {
                let recovery_end = self.source[open..]
                    .find('\n')
                    .map_or(self.source.len(), |line| open + line);
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnclosedExpression,
                    Span::new(open, recovery_end),
                    "expression is missing its closing delimiter",
                ));
                nodes.push(Node {
                    span: Span::new(open, recovery_end),
                    kind: NodeKind::Error(&self.source[open..recovery_end]),
                });
                self.offset = recovery_end.max(open + opening_len);
                continue;
            };
            let close_start = content_start + relative_close;
            let end = close_start + closing.len();
            let whole_span = Span::new(open, end);
            let mut inner_start = content_start;
            let mut inner_end = close_start;
            if self.source[inner_start..inner_end].starts_with('~') {
                inner_start += 1;
            }
            if inner_end > inner_start && self.source.as_bytes()[inner_end - 1] == b'~' {
                inner_end -= 1;
            }
            let trimmed_start = inner_start
                + self.source[inner_start..inner_end]
                    .len()
                    .saturating_sub(self.source[inner_start..inner_end].trim_start().len());
            let trimmed_end = inner_end
                - (self.source[inner_start..inner_end].len()
                    - self.source[inner_start..inner_end].trim_end().len());
            let content = &self.source[trimmed_start..trimmed_end];
            self.offset = end;

            if !triple && (content.starts_with('!')) {
                let (comment_start, comment_end) =
                    if content.starts_with("!--") && content.ends_with("--") {
                        (trimmed_start + 3, trimmed_end - 2)
                    } else {
                        (trimmed_start + 1, trimmed_end)
                    };
                nodes.push(Node {
                    span: whole_span,
                    kind: NodeKind::Comment {
                        content: &self.source[comment_start..comment_end],
                        content_span: Span::new(comment_start, comment_end),
                    },
                });
                continue;
            }

            if !triple && content == "else" {
                return (nodes, Terminator::Else(whole_span));
            }

            if !triple && content.starts_with('/') {
                let (name, name_span) = self.name_after_marker(trimmed_start, trimmed_end, 1);
                return (
                    nodes,
                    Terminator::Close {
                        name,
                        name_span,
                        span: whole_span,
                    },
                );
            }

            if !triple && content.starts_with('#') {
                let block = self.parse_block(whole_span, trimmed_start + 1, trimmed_end);
                nodes.push(Node {
                    span: block.full_span(),
                    kind: NodeKind::Block(block),
                });
                continue;
            }

            let expression_span = Span::new(trimmed_start, trimmed_end);
            let tokens = self.tokenize(expression_span);
            if tokens.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidToken,
                    expression_span,
                    "expression must contain a value or helper",
                ));
            }
            nodes.push(Node {
                span: whole_span,
                kind: NodeKind::Interpolation {
                    escaped: !triple,
                    expression_span,
                    tokens,
                },
            });
        }

        if let Some(name) = expected_close {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnclosedBlock,
                Span::new(self.source.len(), self.source.len()),
                format!("block `{name}` is missing `{{{{/{name}}}}}`"),
            ));
        }
        (nodes, Terminator::End)
    }

    fn parse_block(
        &mut self,
        open_span: Span,
        header_start: usize,
        header_end: usize,
    ) -> BlockNode<'source> {
        let expression_span = Span::new(header_start, header_end);
        let tokens = self.tokenize(expression_span);
        let (name, name_span) = tokens
            .first()
            .map_or(("", Span::new(header_start, header_start)), |token| {
                (token.text, token.span)
            });
        if name.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidToken,
                expression_span,
                "block helper name is missing",
            ));
        }
        let arguments = tokens.get(1..).unwrap_or_default().to_vec();
        let alias = arguments
            .iter()
            .position(|token| token.text == "as")
            .and_then(|position| {
                arguments[position + 1..]
                    .iter()
                    .copied()
                    .find(|token| token.kind != SyntaxTokenKind::Punctuation)
            });
        if arguments.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidHelperArguments,
                name_span,
                format!("block helper `{name}` expects an argument"),
            ));
        }

        let (body, terminator) = self.parse_nodes(Some(name));
        let mut else_span = None;
        let mut else_body = Vec::new();
        let close = match terminator {
            Terminator::Else(span) => {
                else_span = Some(span);
                let (parsed_else, else_terminator) = self.parse_nodes(Some(name));
                else_body = parsed_else;
                match else_terminator {
                    Terminator::Else(duplicate) => {
                        self.diagnostics.push(Diagnostic::error(
                            DiagnosticCode::DuplicateElse,
                            duplicate,
                            format!("block `{name}` has more than one `else` branch"),
                        ));
                        None
                    }
                    Terminator::Close {
                        name: close_name,
                        name_span: close_name_span,
                        span,
                    } => self.check_close(name, close_name, close_name_span, span),
                    Terminator::End => None,
                }
            }
            Terminator::Close {
                name: close_name,
                name_span: close_name_span,
                span,
            } => self.check_close(name, close_name, close_name_span, span),
            Terminator::End => None,
        };

        BlockNode {
            name,
            name_span,
            open_span,
            expression_span,
            arguments,
            alias,
            body,
            else_span,
            else_body,
            close_span: close,
        }
    }

    fn check_close(
        &mut self,
        expected: &str,
        actual: &str,
        actual_span: Span,
        span: Span,
    ) -> Option<Span> {
        fn block_family(name: &str) -> &str {
            match name.strip_suffix("_ref").unwrap_or(name) {
                "if_some" => "if",
                name => name,
            }
        }
        let compatible = block_family(expected) == block_family(actual);
        if compatible {
            Some(span)
        } else {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::MismatchedBlock,
                actual_span,
                format!("expected closing block `{expected}`, found `{actual}`"),
            ));
            None
        }
    }

    fn parse_raw_block(&mut self, nodes: &mut Vec<Node<'source>>) {
        let open = self.offset;
        let name_start = open + 4;
        let Some(relative_open_close) = self.source[name_start..].find("}}}}") else {
            self.push_unclosed(nodes, open);
            return;
        };
        let open_close = name_start + relative_open_close;
        let trimmed = self.source[name_start..open_close].trim();
        let leading = self.source[name_start..open_close].len()
            - self.source[name_start..open_close].trim_start().len();
        let name_span = Span::new(name_start + leading, name_start + leading + trimmed.len());
        let content_start = open_close + 4;
        let mut close_marker = String::with_capacity(trimmed.len() + 9);
        close_marker.push_str("{{{{/");
        close_marker.push_str(trimmed);
        close_marker.push_str("}}}}");
        let Some(relative_close) = self.source[content_start..].find(&close_marker) else {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnclosedBlock,
                Span::new(open, open_close + 4),
                format!("raw block `{trimmed}` is missing its closing block"),
            ));
            nodes.push(Node {
                span: Span::new(open, self.source.len()),
                kind: NodeKind::RawBlock {
                    name: trimmed,
                    name_span,
                    content: &self.source[content_start..],
                    content_span: Span::new(content_start, self.source.len()),
                    open_span: Span::new(open, content_start),
                    close_span: None,
                },
            });
            self.offset = self.source.len();
            return;
        };
        let close_start = content_start + relative_close;
        let end = close_start + close_marker.len();
        nodes.push(Node {
            span: Span::new(open, end),
            kind: NodeKind::RawBlock {
                name: trimmed,
                name_span,
                content: &self.source[content_start..close_start],
                content_span: Span::new(content_start, close_start),
                open_span: Span::new(open, content_start),
                close_span: Some(Span::new(close_start, end)),
            },
        });
        self.offset = end;
    }

    fn push_unclosed(&mut self, nodes: &mut Vec<Node<'source>>, start: usize) {
        self.diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnclosedExpression,
            Span::new(start, self.source.len()),
            "expression is missing its closing delimiter",
        ));
        nodes.push(Node {
            span: Span::new(start, self.source.len()),
            kind: NodeKind::Error(&self.source[start..]),
        });
        self.offset = self.source.len();
    }

    fn push_text(&self, nodes: &mut Vec<Node<'source>>, start: usize, end: usize) {
        if start < end {
            nodes.push(Node {
                span: Span::new(start, end),
                kind: NodeKind::Text(&self.source[start..end]),
            });
        }
    }

    fn name_after_marker(
        &self,
        start: usize,
        end: usize,
        marker_len: usize,
    ) -> (&'source str, Span) {
        let marked_start = start + marker_len;
        let raw = &self.source[marked_start..end];
        let leading = raw.len() - raw.trim_start().len();
        let name_start = marked_start + leading;
        let name_end = name_start
            + self.source[name_start..end]
                .find(char::is_whitespace)
                .unwrap_or(end - name_start);
        (
            &self.source[name_start..name_end],
            Span::new(name_start, name_end),
        )
    }

    fn tokenize(&mut self, span: Span) -> Vec<SyntaxToken<'source>> {
        let mut tokens = Vec::new();
        let mut cursor = span.start;
        while cursor < span.end {
            let Some((relative, character)) = self.source[cursor..span.end]
                .char_indices()
                .find(|(_, character)| !character.is_whitespace())
            else {
                break;
            };
            cursor += relative;
            let start = cursor;
            let (end, kind) = match character {
                '"' => self.string_end(start, span.end),
                '(' => self.subexpression_end(start, span.end),
                '|' => (start + 1, SyntaxTokenKind::Punctuation),
                '@' => (
                    self.plain_token_end(start, span.end),
                    SyntaxTokenKind::PrivateVariable,
                ),
                _ => {
                    let end = self.plain_token_end(start, span.end);
                    let text = &self.source[start..end];
                    let kind = if text == "as" || text == "else" {
                        SyntaxTokenKind::Keyword
                    } else if text.parse::<f64>().is_ok()
                        || matches!(text, "true" | "false" | "None")
                    {
                        SyntaxTokenKind::Number
                    } else {
                        SyntaxTokenKind::Variable
                    };
                    (end, kind)
                }
            };
            let safe_end = end.max(start + character.len_utf8()).min(span.end);
            tokens.push(SyntaxToken {
                kind,
                text: &self.source[start..safe_end],
                span: Span::new(start, safe_end),
            });
            cursor = safe_end;
        }
        tokens
    }

    fn string_end(&mut self, start: usize, limit: usize) -> (usize, SyntaxTokenKind) {
        let mut escaped = false;
        for (relative, character) in self.source[start + 1..limit].char_indices() {
            match character {
                '"' if !escaped => return (start + 1 + relative + 1, SyntaxTokenKind::String),
                '\\' => escaped = !escaped,
                _ => escaped = false,
            }
        }
        self.diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnterminatedString,
            Span::new(start, limit),
            "string literal is missing its closing quote",
        ));
        (limit, SyntaxTokenKind::String)
    }

    fn subexpression_end(&mut self, start: usize, limit: usize) -> (usize, SyntaxTokenKind) {
        let mut depth = 0usize;
        let mut in_string = false;
        let mut escaped = false;
        for (relative, character) in self.source[start..limit].char_indices() {
            if in_string {
                match character {
                    '"' if !escaped => in_string = false,
                    '\\' => escaped = !escaped,
                    _ => escaped = false,
                }
                continue;
            }
            match character {
                '"' => in_string = true,
                '(' => depth += 1,
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return (start + relative + 1, SyntaxTokenKind::Subexpression);
                    }
                }
                _ => {}
            }
        }
        self.diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnmatchedSubexpression,
            Span::new(start, limit),
            "subexpression is missing `)`",
        ));
        (limit, SyntaxTokenKind::Subexpression)
    }

    fn plain_token_end(&self, start: usize, limit: usize) -> usize {
        self.source[start..limit]
            .char_indices()
            .find(|(_, character)| {
                character.is_whitespace() || matches!(character, '(' | ')' | '|')
            })
            .map_or(limit, |(relative, _)| start + relative)
    }
}

#[derive(Clone, Copy)]
struct PrivateScope {
    each: bool,
    else_branch: bool,
}

fn validate_private_variables(
    nodes: &[Node<'_>],
    diagnostics: &mut Vec<Diagnostic>,
    scopes: &[PrivateScope],
) {
    for node in nodes {
        match &node.kind {
            NodeKind::Interpolation { tokens, .. } => {
                validate_private_tokens(tokens, diagnostics, scopes);
            }
            NodeKind::Block(block) => {
                validate_private_tokens(&block.arguments, diagnostics, scopes);
                let mut nested = scopes.to_vec();
                nested.push(PrivateScope {
                    each: matches!(block.name, "each" | "each_ref"),
                    else_branch: false,
                });
                validate_private_variables(&block.body, diagnostics, &nested);
                if let Some(scope) = nested.last_mut() {
                    scope.else_branch = true;
                }
                validate_private_variables(&block.else_body, diagnostics, &nested);
            }
            _ => {}
        }
    }
}

fn validate_private_tokens(
    tokens: &[SyntaxToken<'_>],
    diagnostics: &mut Vec<Diagnostic>,
    scopes: &[PrivateScope],
) {
    for token in tokens
        .iter()
        .filter(|token| token.kind == SyntaxTokenKind::PrivateVariable)
    {
        let mut name = token.text.trim_start_matches('@');
        let mut parent_count = 0usize;
        while let Some(parent) = name.strip_prefix("../") {
            name = parent;
            parent_count += 1;
        }
        let scope = scopes
            .iter()
            .rev()
            .filter(|scope| scope.each)
            .nth(parent_count);
        let valid_name = matches!(name, "index" | "key" | "value");
        let valid_scope = scope.is_some_and(|scope| !scope.else_branch);
        if !valid_name || !valid_scope {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnknownPrivateVariable,
                token.span,
                if valid_name {
                    format!("`{}` is not available in this block scope", token.text)
                } else {
                    format!("unknown private variable `{}`", token.text)
                },
            ));
        }
    }
}

/// Parses a complete template and recovers useful syntax after errors.
pub fn parse_template(source: &str) -> ParsedTemplate<'_> {
    Parser::new(source).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_blocks_with_spans() {
        let source = "{{#each_ref items as |item|}}{{item}}{{else}}empty{{/each_ref}}";
        let parsed = parse_template(source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let NodeKind::Block(block) = &parsed.nodes[0].kind else {
            panic!("expected a block");
        };
        assert_eq!(block.name, "each_ref");
        assert_eq!(
            &source[block.name_span.start..block.name_span.end],
            "each_ref"
        );
        assert_eq!(block.alias.map(|token| token.text), Some("item"));
        assert!(block.else_span.is_some());
        assert!(block.close_span.is_some());
    }

    #[test]
    fn collects_multiple_diagnostics() {
        let parsed = parse_template("{{#if}}{{\"open}}\n{{value");
        let codes = parsed
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&DiagnosticCode::InvalidHelperArguments));
        assert!(codes.contains(&DiagnosticCode::UnterminatedString));
        assert!(codes.contains(&DiagnosticCode::UnclosedExpression));
        assert!(codes.contains(&DiagnosticCode::UnclosedBlock));
    }

    #[test]
    fn handles_unicode_byte_spans() {
        let source = "🦀 {{name}}";
        let parsed = parse_template(source);
        assert_eq!(parsed.nodes[0].span, Span::new(0, "🦀 ".len()));
        assert_eq!(parsed.nodes[1].span, Span::new("🦀 ".len(), source.len()));
    }

    #[test]
    fn reports_mismatched_blocks() {
        let parsed = parse_template("{{#if ready}}{{/each}}");
        assert!(parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MismatchedBlock));
    }

    #[test]
    fn arbitrary_utf8_always_makes_progress() {
        for source in ["", "{", "{{", "{{{", "é{{", "{{(((((", "{{!--", "\0{{x}}"] {
            let _ = parse_template(source);
        }
    }

    #[test]
    fn validates_private_variable_scope() {
        let parsed =
            parse_template("{{@index}}{{#each values}}{{@index}}{{else}}{{@key}}{{/each}}");
        assert_eq!(
            parsed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == DiagnosticCode::UnknownPrivateVariable)
                .count(),
            2
        );
    }
}
