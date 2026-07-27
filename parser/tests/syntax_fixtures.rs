use rusty_handlebars_parser::{parse_template, DiagnosticCode, NodeKind};

#[test]
fn valid_fixture_covers_the_language_contract() {
    let source = include_str!("fixtures/syntax-valid.rhbs");
    let parsed = parse_template(source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert!(parsed
        .nodes
        .iter()
        .any(|node| matches!(node.kind, NodeKind::RawBlock { .. })));
}

#[test]
fn invalid_fixture_recovers_multiple_diagnostics() {
    let source = include_str!("fixtures/syntax-invalid.rhbs");
    let parsed = parse_template(source);
    assert!(parsed.diagnostics.len() >= 4, "{:?}", parsed.diagnostics);
    assert!(parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::UnclosedExpression));
    assert!(parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::MismatchedBlock));
}
