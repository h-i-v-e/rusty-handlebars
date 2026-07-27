use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, DocumentHighlight,
    DocumentHighlightKind, DocumentSymbol, Documentation, FoldingRange, FoldingRangeKind, Hover,
    HoverContents, MarkupContent, MarkupKind, Position, SelectionRange, SignatureHelp,
    SignatureInformation, SymbolKind,
};
use rusty_handlebars_parser::{
    parse_template, BlockNode, Node, NodeKind, Span, SyntaxToken, SyntaxTokenKind,
};

use crate::documents::{position_to_byte, span_to_range};
use crate::project::{FieldInfo, TemplateContext};

const BLOCKS: &[(&str, &str)] = &[
    ("if", "Render a body when a value is truthy."),
    ("unless", "Render a body when a value is falsey."),
    ("if_some", "Match an `Option` and bind its contained value."),
    (
        "if_some_ref",
        "Borrow an `Option` and bind its contained value.",
    ),
    ("with", "Use a value as the current template context."),
    (
        "with_ref",
        "Borrow a value as the current template context.",
    ),
    (
        "each",
        "Iterate over a value, with an optional empty branch.",
    ),
    ("each_ref", "Borrow and iterate over a value."),
];

const HELPERS: &[(&str, &str, &str)] = &[
    (
        "lookup",
        "lookup values index",
        "Index a value with `values[index]`.",
    ),
    (
        "try_lookup",
        "try_lookup map key",
        "Look up a value with `map.get(key)`.",
    ),
    (
        "format",
        "format \"{specifier}\" value",
        "Render a value with a Rust format specifier.",
    ),
];

pub struct ProjectDiagnostic {
    pub span: Span,
    pub code: &'static str,
    pub message: String,
}

pub fn completions(source: &str, position: Position) -> CompletionResponse {
    let offset = position_to_byte(source, position);
    let parsed = parse_template(source);
    let mut items = BLOCKS
        .iter()
        .map(|(name, documentation)| CompletionItem {
            label: (*name).to_owned(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Rusty Handlebars block".to_owned()),
            documentation: Some(Documentation::String((*documentation).to_owned())),
            insert_text: Some(format!("{name} ")),
            ..Default::default()
        })
        .chain(
            HELPERS
                .iter()
                .map(|(name, signature, documentation)| CompletionItem {
                    label: (*name).to_owned(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some((*signature).to_owned()),
                    documentation: Some(Documentation::String((*documentation).to_owned())),
                    insert_text: Some(format!("{name} ")),
                    ..Default::default()
                }),
        )
        .collect::<Vec<_>>();
    collect_scope_completions(&parsed.nodes, offset, &mut items);
    CompletionResponse::Array(items)
}

pub fn add_project_completions(response: &mut CompletionResponse, contexts: &[TemplateContext]) {
    let CompletionResponse::Array(items) = response else {
        return;
    };
    for context in contexts {
        for field in &context.fields {
            if items.iter().any(|item| item.label == field.name) {
                continue;
            }
            items.push(CompletionItem {
                label: field.name.clone(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(format!("{} field: {}", context.name, field.ty)),
                ..Default::default()
            });
        }
        for helper in &context.helpers {
            let name = helper.rsplit("::").next().unwrap_or(helper);
            if items.iter().any(|item| item.label == name) {
                continue;
            }
            items.push(CompletionItem {
                label: name.to_owned(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("Configured helper `{helper}`")),
                ..Default::default()
            });
        }
    }
}

fn collect_scope_completions(nodes: &[Node<'_>], offset: usize, items: &mut Vec<CompletionItem>) {
    for node in nodes {
        let NodeKind::Block(block) = &node.kind else {
            continue;
        };
        if !node.span.contains(offset) {
            continue;
        }
        if let Some(alias) = block.alias {
            items.push(variable_completion(
                alias.text.trim_matches('|'),
                "Block alias",
            ));
        } else if matches!(
            block.name,
            "with" | "with_ref" | "if_some" | "if_some_ref" | "each" | "each_ref"
        ) {
            items.push(variable_completion("this", "Current block value"));
        }
        if matches!(block.name, "each" | "each_ref")
            && block.else_span.is_none_or(|span| offset < span.start)
        {
            for (name, detail) in [
                ("@index", "Zero-based iteration index"),
                ("@key", "Pair key (when the item is pair-like)"),
                ("@value", "Pair value (when the item is pair-like)"),
            ] {
                items.push(variable_completion(name, detail));
            }
        }
        let active = if block.else_span.is_some_and(|span| offset > span.end) {
            &block.else_body
        } else {
            &block.body
        };
        collect_scope_completions(active, offset, items);
    }
}

fn variable_completion(name: &str, detail: &str) -> CompletionItem {
    CompletionItem {
        label: name.to_owned(),
        kind: Some(CompletionItemKind::VARIABLE),
        detail: Some(detail.to_owned()),
        ..Default::default()
    }
}

pub fn hover(source: &str, position: Position) -> Option<Hover> {
    let offset = position_to_byte(source, position);
    let parsed = parse_template(source);
    let token = find_token(&parsed.nodes, offset)?;
    let name = token.text.trim_matches('|');
    let documentation = BLOCKS
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, documentation)| *documentation)
        .or_else(|| {
            HELPERS
                .iter()
                .find(|(candidate, _, _)| *candidate == name)
                .map(|(_, _, documentation)| *documentation)
        })
        .or(match name {
            "@index" => Some("Zero-based index in the nearest `each` block."),
            "@key" => Some("First member of a pair-like item in an `each` block."),
            "@value" => Some("Second member of a pair-like item in an `each` block."),
            "this" => Some("The current block context."),
            _ => None,
        })?;
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("**`{name}`**\n\n{documentation}"),
        }),
        range: Some(span_to_range(source, token.span)),
    })
}

pub fn project_field_at<'context>(
    source: &str,
    position: Position,
    contexts: &'context [TemplateContext],
) -> Option<&'context FieldInfo> {
    let token = token_at(source, position)?;
    let root = token
        .text
        .trim_start_matches("../")
        .split('.')
        .next()
        .unwrap_or(token.text);
    contexts
        .iter()
        .flat_map(|context| &context.fields)
        .find(|field| field.name == root)
}

pub fn project_hover(
    source: &str,
    position: Position,
    contexts: &[TemplateContext],
) -> Option<Hover> {
    let token = token_at(source, position)?;
    let field = project_field_at(source, position, contexts)?;
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**`{}.{}`**\n\nRust type: `{}`",
                contexts.first()?.name,
                field.name,
                field.ty
            ),
        }),
        range: Some(span_to_range(source, token.span)),
    })
}

pub fn project_diagnostics(source: &str, contexts: &[TemplateContext]) -> Vec<ProjectDiagnostic> {
    if contexts.is_empty() {
        return Vec::new();
    }
    let parsed = parse_template(source);
    let mut diagnostics = Vec::new();
    collect_project_diagnostics(&parsed.nodes, contexts, true, &mut diagnostics);
    diagnostics
}

fn collect_project_diagnostics(
    nodes: &[Node<'_>],
    contexts: &[TemplateContext],
    root_context: bool,
    diagnostics: &mut Vec<ProjectDiagnostic>,
) {
    for node in nodes {
        match &node.kind {
            NodeKind::Interpolation { tokens, .. } => {
                if let Some(first) = tokens.first() {
                    if tokens.len() == 1 && root_context {
                        check_root_field(*first, contexts, diagnostics);
                    } else if tokens.len() > 1 {
                        check_helper(*first, contexts, diagnostics);
                        if root_context {
                            for argument in &tokens[1..] {
                                check_root_field(*argument, contexts, diagnostics);
                            }
                        }
                    }
                }
            }
            NodeKind::Block(block) => {
                if root_context {
                    if let Some(argument) = block.arguments.first() {
                        check_root_field(*argument, contexts, diagnostics);
                    }
                }
                let preserves_root = root_context && matches!(block.name, "if" | "unless");
                collect_project_diagnostics(&block.body, contexts, preserves_root, diagnostics);
                collect_project_diagnostics(
                    &block.else_body,
                    contexts,
                    preserves_root,
                    diagnostics,
                );
            }
            _ => {}
        }
    }
}

fn check_root_field(
    token: SyntaxToken<'_>,
    contexts: &[TemplateContext],
    diagnostics: &mut Vec<ProjectDiagnostic>,
) {
    if token.kind != SyntaxTokenKind::Variable {
        return;
    }
    let root = token
        .text
        .trim_start_matches("../")
        .split('.')
        .next()
        .unwrap_or(token.text);
    if matches!(root, "this" | "true" | "false" | "None" | "as") || root.is_empty() {
        return;
    }
    if contexts
        .iter()
        .any(|context| context.fields.iter().any(|field| field.name == root))
    {
        return;
    }
    diagnostics.push(ProjectDiagnostic {
        span: token.span,
        code: "unknown-field",
        message: format!(
            "field `{root}` does not exist on any context associated with this template"
        ),
    });
}

fn check_helper(
    token: SyntaxToken<'_>,
    contexts: &[TemplateContext],
    diagnostics: &mut Vec<ProjectDiagnostic>,
) {
    if token.kind != SyntaxTokenKind::Variable
        || HELPERS.iter().any(|(name, _, _)| *name == token.text)
        || contexts.iter().any(|context| {
            context
                .helpers
                .iter()
                .any(|helper| helper.rsplit("::").next() == Some(token.text))
        })
    {
        return;
    }
    diagnostics.push(ProjectDiagnostic {
        span: token.span,
        code: "unknown-helper",
        message: format!(
            "helper `{}` is not built in or configured for this template",
            token.text
        ),
    });
}

pub fn document_symbols(source: &str) -> Vec<DocumentSymbol> {
    let parsed = parse_template(source);
    symbols_for_nodes(source, &parsed.nodes)
}

#[allow(deprecated)]
fn symbols_for_nodes(source: &str, nodes: &[Node<'_>]) -> Vec<DocumentSymbol> {
    nodes
        .iter()
        .filter_map(|node| {
            let NodeKind::Block(block) = &node.kind else {
                return None;
            };
            Some(DocumentSymbol {
                name: block.name.to_owned(),
                detail: Some(
                    source[block.expression_span.start..block.expression_span.end].to_owned(),
                ),
                kind: SymbolKind::NAMESPACE,
                tags: None,
                deprecated: None,
                range: span_to_range(source, node.span),
                selection_range: span_to_range(source, block.name_span),
                children: Some(
                    symbols_for_nodes(source, &block.body)
                        .into_iter()
                        .chain(symbols_for_nodes(source, &block.else_body))
                        .collect(),
                ),
            })
        })
        .collect()
}

pub fn folding_ranges(source: &str) -> Vec<FoldingRange> {
    let parsed = parse_template(source);
    let mut ranges = Vec::new();
    collect_folding(source, &parsed.nodes, &mut ranges);
    ranges
}

fn collect_folding(source: &str, nodes: &[Node<'_>], ranges: &mut Vec<FoldingRange>) {
    for node in nodes {
        let NodeKind::Block(block) = &node.kind else {
            continue;
        };
        let start = span_to_range(source, block.open_span).end;
        let end_span = block.close_span.unwrap_or(node.span);
        let end = span_to_range(source, end_span).start;
        if start.line < end.line {
            ranges.push(FoldingRange {
                start_line: start.line,
                start_character: Some(start.character),
                end_line: end.line,
                end_character: Some(end.character),
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: Some(format!("{} …", block.name)),
            });
        }
        collect_folding(source, &block.body, ranges);
        collect_folding(source, &block.else_body, ranges);
    }
}

pub fn selection_ranges(source: &str, positions: &[Position]) -> Vec<SelectionRange> {
    let parsed = parse_template(source);
    positions
        .iter()
        .map(|position| {
            let offset = position_to_byte(source, *position);
            let mut spans = vec![Span::new(0, source.len())];
            collect_containing_spans(&parsed.nodes, offset, &mut spans);
            spans.sort_by_key(|span| span.end - span.start);
            spans.dedup();
            spans
                .into_iter()
                .rev()
                .fold(None, |parent, span| {
                    Some(Box::new(SelectionRange {
                        range: span_to_range(source, span),
                        parent,
                    }))
                })
                .map(|range| *range)
                .unwrap_or_default()
        })
        .collect()
}

fn collect_containing_spans(nodes: &[Node<'_>], offset: usize, spans: &mut Vec<Span>) {
    for node in nodes.iter().filter(|node| node.span.contains(offset)) {
        spans.push(node.span);
        match &node.kind {
            NodeKind::Interpolation {
                expression_span,
                tokens,
                ..
            } => {
                spans.push(*expression_span);
                if let Some(token) = tokens.iter().find(|token| token.span.contains(offset)) {
                    spans.push(token.span);
                }
            }
            NodeKind::Block(block) => {
                spans.push(block.expression_span);
                collect_containing_spans(&block.body, offset, spans);
                collect_containing_spans(&block.else_body, offset, spans);
            }
            _ => {}
        }
    }
}

pub fn document_highlights(source: &str, position: Position) -> Vec<DocumentHighlight> {
    let offset = position_to_byte(source, position);
    let parsed = parse_template(source);
    let Some(block) = find_block_at(&parsed.nodes, offset) else {
        return Vec::new();
    };
    let mut spans = vec![block.name_span];
    if let Some(close) = block.close_span {
        if let Some(relative) = source[close.start..close.end].find('/') {
            let start = close.start + relative + 1;
            spans.push(Span::new(
                start,
                start + block.name.len().min(close.end - start),
            ));
        }
    }
    spans
        .into_iter()
        .map(|span| DocumentHighlight {
            range: span_to_range(source, span),
            kind: Some(DocumentHighlightKind::TEXT),
        })
        .collect()
}

pub fn signature_help(source: &str, position: Position) -> Option<SignatureHelp> {
    let offset = position_to_byte(source, position);
    let parsed = parse_template(source);
    let token = find_token(&parsed.nodes, offset)?;
    let (_, signature, documentation) = HELPERS.iter().find(|(name, _, _)| *name == token.text)?;
    Some(SignatureHelp {
        signatures: vec![SignatureInformation {
            label: (*signature).to_owned(),
            documentation: Some(Documentation::String((*documentation).to_owned())),
            parameters: None,
            active_parameter: None,
        }],
        active_signature: Some(0),
        active_parameter: None,
    })
}

fn find_token<'source>(nodes: &[Node<'source>], offset: usize) -> Option<SyntaxToken<'source>> {
    for node in nodes.iter().filter(|node| node.span.contains(offset)) {
        match &node.kind {
            NodeKind::Interpolation { tokens, .. } => {
                return tokens
                    .iter()
                    .copied()
                    .find(|token| token.span.contains(offset));
            }
            NodeKind::Block(block) => {
                if block.name_span.contains(offset) {
                    return Some(SyntaxToken {
                        kind: SyntaxTokenKind::Keyword,
                        text: block.name,
                        span: block.name_span,
                    });
                }
                if let Some(found) =
                    find_token(&block.body, offset).or_else(|| find_token(&block.else_body, offset))
                {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn token_at(source: &str, position: Position) -> Option<SyntaxToken<'_>> {
    let offset = position_to_byte(source, position);
    let parsed = parse_template(source);
    find_token(&parsed.nodes, offset)
}

fn find_block_at<'source>(
    nodes: &'source [Node<'source>],
    offset: usize,
) -> Option<&'source BlockNode<'source>> {
    for node in nodes.iter().filter(|node| node.span.contains(offset)) {
        let NodeKind::Block(block) = &node.kind else {
            continue;
        };
        if block.name_span.contains(offset)
            || block.close_span.is_some_and(|span| span.contains(offset))
        {
            return Some(block);
        }
        if let Some(nested) =
            find_block_at(&block.body, offset).or_else(|| find_block_at(&block.else_body, offset))
        {
            return Some(nested);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_completion_excludes_each_values_from_else() {
        let source = "{{#each_ref values}}{{this}}{{else}}{{/each_ref}}";
        let body = completions(source, Position::new(0, 24));
        let else_branch = completions(source, Position::new(0, 40));
        let labels = |response: CompletionResponse| match response {
            CompletionResponse::Array(items) => {
                items.into_iter().map(|item| item.label).collect::<Vec<_>>()
            }
            _ => Vec::new(),
        };
        assert!(labels(body).contains(&"@index".to_owned()));
        assert!(!labels(else_branch).contains(&"@index".to_owned()));
    }
}
