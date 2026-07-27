use std::{
    error::Error,
    path::{Path, PathBuf},
    str::FromStr,
};

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    CompletionOptions, CompletionParams, DiagnosticSeverity, DidChangeTextDocumentParams,
    DidChangeWatchedFilesParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, DocumentHighlightParams, DocumentSymbolParams, FoldingRangeParams,
    GotoDefinitionParams, HoverParams, HoverProviderCapability, Location, OneOf,
    PositionEncodingKind, PublishDiagnosticsParams, SelectionRangeParams,
    SelectionRangeProviderCapability, ServerCapabilities, SignatureHelpOptions,
    SignatureHelpParams, TextDocumentIdentifier, TextDocumentSyncCapability, TextDocumentSyncKind,
    Uri,
};
use rusty_handlebars_parser::{
    add_builtins, parse_template, BlockMap, Compiler, Options, Severity,
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use url::Url;

use crate::{
    documents::{byte_to_position, span_to_range, Documents},
    features,
    project::{ProjectIndex, TemplateContext},
};

type ServerResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        position_encoding: Some(PositionEncodingKind::UTF16),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec!["{".to_owned(), "@".to_owned(), "/".to_owned()]),
            ..Default::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec![" ".to_owned(), "(".to_owned()]),
            ..Default::default()
        }),
        document_symbol_provider: Some(OneOf::Left(true)),
        folding_range_provider: Some(true.into()),
        selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
        document_highlight_provider: Some(OneOf::Left(true)),
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    }
}

pub fn run() -> ServerResult<()> {
    let (connection, threads) = Connection::stdio();
    run_connection(connection)?;
    threads.join()?;
    Ok(())
}

fn run_connection(connection: Connection) -> ServerResult<()> {
    let initialization = serde_json::json!({
        "capabilities": capabilities(),
        "serverInfo": {
            "name": "rusty-handlebars-language-server",
            "version": env!("CARGO_PKG_VERSION")
        }
    });
    let parameters = connection.initialize(initialization)?;
    let root = workspace_root(&parameters);
    let project = root
        .as_deref()
        .and_then(|root| ProjectIndex::discover(root).ok())
        .unwrap_or_default();
    serve(connection, root, project)
}

fn serve(
    connection: Connection,
    root: Option<PathBuf>,
    mut project: ProjectIndex,
) -> ServerResult<()> {
    let mut documents = Documents::default();
    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    break;
                }
                handle_request(
                    &connection,
                    &documents,
                    root.as_deref(),
                    &mut project,
                    request,
                )?;
            }
            Message::Notification(notification) => {
                handle_notification(
                    &connection,
                    &mut documents,
                    root.as_deref(),
                    &mut project,
                    notification,
                )?;
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}

fn handle_notification(
    connection: &Connection,
    documents: &mut Documents,
    root: Option<&Path>,
    project: &mut ProjectIndex,
    notification: Notification,
) -> ServerResult<()> {
    match notification.method.as_str() {
        "textDocument/didOpen" => {
            let params: DidOpenTextDocumentParams = from_value(notification.params)?;
            let document = params.text_document;
            documents.open(document.uri.clone(), document.text, document.version);
            publish_diagnostics(connection, documents, project, &document.uri)?;
        }
        "textDocument/didChange" => {
            let params: DidChangeTextDocumentParams = from_value(notification.params)?;
            if let Some(change) = params.content_changes.into_iter().last() {
                let uri = params.text_document.uri;
                documents.change(&uri, change.text, params.text_document.version);
                publish_diagnostics(connection, documents, project, &uri)?;
            }
        }
        "textDocument/didClose" => {
            let params: DidCloseTextDocumentParams = from_value(notification.params)?;
            documents.close(&params.text_document.uri);
            send_notification(
                connection,
                "textDocument/publishDiagnostics",
                PublishDiagnosticsParams::new(params.text_document.uri, Vec::new(), None),
            )?;
        }
        "textDocument/didSave" => {
            let params: DidSaveTextDocumentParams = from_value(notification.params)?;
            publish_diagnostics(connection, documents, project, &params.text_document.uri)?;
        }
        "workspace/didChangeWatchedFiles" => {
            let params: DidChangeWatchedFilesParams = from_value(notification.params)?;
            if params
                .changes
                .iter()
                .any(|change| uri_path(&change.uri).is_some_and(|path| is_project_input(&path)))
            {
                reload_project(root, project);
                publish_all_diagnostics(connection, documents, project)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_request(
    connection: &Connection,
    documents: &Documents,
    root: Option<&Path>,
    project: &mut ProjectIndex,
    request: Request,
) -> ServerResult<()> {
    let id = request.id.clone();
    match request.method.as_str() {
        "textDocument/completion" => {
            let params: CompletionParams = from_value(request.params)?;
            let contexts = uri_path(&params.text_document_position.text_document.uri)
                .map(|path| project.contexts_for(&path))
                .unwrap_or_default();
            with_document(
                connection,
                documents,
                id,
                &params.text_document_position.text_document.uri,
                |text| {
                    let mut response =
                        features::completions(text, params.text_document_position.position);
                    features::add_project_completions(&mut response, contexts);
                    response
                },
            )?;
        }
        "textDocument/hover" => {
            let params: HoverParams = from_value(request.params)?;
            let contexts = uri_path(&params.text_document_position_params.text_document.uri)
                .map(|path| project.contexts_for(&path))
                .unwrap_or_default();
            with_document(
                connection,
                documents,
                id,
                &params.text_document_position_params.text_document.uri,
                |text| {
                    features::project_hover(
                        text,
                        params.text_document_position_params.position,
                        contexts,
                    )
                    .or_else(|| {
                        features::hover(text, params.text_document_position_params.position)
                    })
                },
            )?;
        }
        "textDocument/definition" => {
            let params: GotoDefinitionParams = from_value(request.params)?;
            let uri = &params.text_document_position_params.text_document.uri;
            let contexts = uri_path(uri)
                .map(|path| project.contexts_for(&path))
                .unwrap_or_default();
            with_document(connection, documents, id, uri, |text| {
                definition_location(
                    text,
                    params.text_document_position_params.position,
                    contexts,
                )
            })?;
        }
        "textDocument/documentSymbol" => {
            let params: DocumentSymbolParams = from_value(request.params)?;
            with_document(
                connection,
                documents,
                id,
                &params.text_document.uri,
                features::document_symbols,
            )?;
        }
        "textDocument/foldingRange" => {
            let params: FoldingRangeParams = from_value(request.params)?;
            with_document(
                connection,
                documents,
                id,
                &params.text_document.uri,
                features::folding_ranges,
            )?;
        }
        "textDocument/selectionRange" => {
            let params: SelectionRangeParams = from_value(request.params)?;
            with_document(
                connection,
                documents,
                id,
                &params.text_document.uri,
                |text| features::selection_ranges(text, &params.positions),
            )?;
        }
        "textDocument/documentHighlight" => {
            let params: DocumentHighlightParams = from_value(request.params)?;
            with_document(
                connection,
                documents,
                id,
                &params.text_document_position_params.text_document.uri,
                |text| {
                    features::document_highlights(
                        text,
                        params.text_document_position_params.position,
                    )
                },
            )?;
        }
        "textDocument/signatureHelp" => {
            let params: SignatureHelpParams = from_value(request.params)?;
            with_document(
                connection,
                documents,
                id,
                &params.text_document_position_params.text_document.uri,
                |text| {
                    features::signature_help(text, params.text_document_position_params.position)
                },
            )?;
        }
        "rustyHandlebars/showGeneratedRust" => {
            let params: TextDocumentIdentifier = from_value(request.params)?;
            with_document(connection, documents, id, &params.uri, generated_rust)?;
        }
        "rustyHandlebars/projectContexts" => {
            let params: TextDocumentIdentifier = from_value(request.params)?;
            let contexts = uri_path(&params.uri)
                .map(|path| project.contexts_for(&path))
                .unwrap_or_default();
            send_response(connection, id, contexts)?;
        }
        "rustyHandlebars/reloadProject" => {
            let reloaded = reload_project(root, project);
            if reloaded {
                publish_all_diagnostics(connection, documents, project)?;
            }
            send_response(connection, id, reloaded)?;
        }
        _ => send_error(connection, id, -32601, "method not found")?,
    }
    Ok(())
}

fn generated_rust(source: &str) -> Result<String, String> {
    let mut blocks = BlockMap::new();
    add_builtins(&mut blocks);
    Compiler::new(
        Options {
            root_var_name: Some("self"),
            write_var_name: "f",
        },
        blocks,
    )
    .compile(source)
    .map(|rust| rust.code)
    .map_err(|error| error.to_string())
}

fn definition_location(
    source: &str,
    position: lsp_types::Position,
    contexts: &[TemplateContext],
) -> Option<Location> {
    let field = features::project_field_at(source, position, contexts)?;
    let rust_source = std::fs::read_to_string(&field.source).ok()?;
    let needle = format!("{}:", field.name);
    let start = rust_source.find(&needle)?;
    let end = start + field.name.len();
    let uri = path_uri(&field.source)?;
    Some(Location::new(
        uri,
        lsp_types::Range::new(
            byte_to_position(&rust_source, start),
            byte_to_position(&rust_source, end),
        ),
    ))
}

fn publish_diagnostics(
    connection: &Connection,
    documents: &Documents,
    project: &ProjectIndex,
    uri: &Uri,
) -> ServerResult<()> {
    let Some(document) = documents.get(uri) else {
        return Ok(());
    };
    let parsed = parse_template(&document.text);
    let mut diagnostics = parsed
        .diagnostics
        .into_iter()
        .map(|diagnostic| lsp_types::Diagnostic {
            range: span_to_range(&document.text, diagnostic.span),
            severity: Some(match diagnostic.severity {
                Severity::Error => DiagnosticSeverity::ERROR,
                Severity::Warning => DiagnosticSeverity::WARNING,
            }),
            code: Some(lsp_types::NumberOrString::String(
                diagnostic.code.as_str().to_owned(),
            )),
            source: Some("rusty-handlebars".to_owned()),
            message: diagnostic.message,
            ..Default::default()
        })
        .collect::<Vec<_>>();
    let contexts = uri_path(uri)
        .map(|path| project.contexts_for(&path))
        .unwrap_or_default();
    diagnostics.extend(
        features::project_diagnostics(&document.text, contexts)
            .into_iter()
            .map(|diagnostic| lsp_types::Diagnostic {
                range: span_to_range(&document.text, diagnostic.span),
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(lsp_types::NumberOrString::String(
                    diagnostic.code.to_owned(),
                )),
                source: Some("rusty-handlebars".to_owned()),
                message: diagnostic.message,
                ..Default::default()
            }),
    );
    send_notification(
        connection,
        "textDocument/publishDiagnostics",
        PublishDiagnosticsParams::new(uri.clone(), diagnostics, Some(document.version)),
    )
}

fn with_document<T: serde::Serialize>(
    connection: &Connection,
    documents: &Documents,
    id: RequestId,
    uri: &Uri,
    operation: impl FnOnce(&str) -> T,
) -> ServerResult<()> {
    match documents.get(uri) {
        Some(document) => send_response(connection, id, operation(&document.text)),
        None => send_error(connection, id, -32602, "document is not open"),
    }
}

fn send_response(
    connection: &Connection,
    id: RequestId,
    value: impl serde::Serialize,
) -> ServerResult<()> {
    connection.sender.send(Message::Response(Response::new_ok(
        id,
        serde_json::to_value(value)?,
    )))?;
    Ok(())
}

fn send_error(
    connection: &Connection,
    id: RequestId,
    code: i32,
    message: &str,
) -> ServerResult<()> {
    connection.sender.send(Message::Response(Response::new_err(
        id,
        code,
        message.to_owned(),
    )))?;
    Ok(())
}

fn send_notification(
    connection: &Connection,
    method: &str,
    params: impl serde::Serialize,
) -> ServerResult<()> {
    connection
        .sender
        .send(Message::Notification(Notification::new(
            method.to_owned(),
            params,
        )))?;
    Ok(())
}

fn from_value<T: DeserializeOwned>(value: Value) -> ServerResult<T> {
    Ok(serde_json::from_value(value)?)
}

fn workspace_root(parameters: &Value) -> Option<PathBuf> {
    parameters
        .get("workspaceFolders")
        .and_then(Value::as_array)
        .and_then(|folders| folders.first())
        .and_then(|folder| folder.get("uri"))
        .and_then(Value::as_str)
        .and_then(|uri| Uri::from_str(uri).ok())
        .and_then(|uri| uri_path(&uri))
        .or_else(|| {
            parameters
                .get("rootUri")
                .and_then(Value::as_str)
                .and_then(|uri| Uri::from_str(uri).ok())
                .and_then(|uri| uri_path(&uri))
        })
}

fn uri_path(uri: &Uri) -> Option<PathBuf> {
    Url::parse(uri.as_str()).ok()?.to_file_path().ok()
}

fn path_uri(path: &Path) -> Option<Uri> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    Uri::from_str(Url::from_file_path(absolute).ok()?.as_str()).ok()
}

fn is_project_input(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        name == "Cargo.toml"
            || name == "Cargo.lock"
            || path.extension().is_some_and(|ext| ext == "rs")
    })
}

fn reload_project(root: Option<&Path>, project: &mut ProjectIndex) -> bool {
    let Some(root) = root else {
        return false;
    };
    match ProjectIndex::discover(root) {
        Ok(reloaded) => {
            *project = reloaded;
            true
        }
        Err(error) => {
            eprintln!("rusty-handlebars-language-server: project reload failed: {error}");
            false
        }
    }
}

fn publish_all_diagnostics(
    connection: &Connection,
    documents: &Documents,
    project: &ProjectIndex,
) -> ServerResult<()> {
    for uri in documents.uris() {
        publish_diagnostics(connection, documents, project, uri)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use lsp_server::{Connection, Message, Notification, Request, RequestId};
    use serde_json::json;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn converts_encoded_file_uris() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("space # percent % ü.rhbs");
        let uri = path_uri(&path).unwrap();

        assert_eq!(uri_path(&uri).as_deref(), Some(path.as_path()));
        assert!(uri.as_str().contains("%20"));
        assert!(uri.as_str().contains("%23"));
        assert!(uri.as_str().contains("%25"));
        assert!(uri.as_str().contains("%C3%BC"));
    }

    #[test]
    fn rejects_non_file_uris() {
        let uri = Uri::from_str("https://example.com/template.rhbs").unwrap();
        assert_eq!(uri_path(&uri), None);
    }

    #[cfg(windows)]
    #[test]
    fn converts_windows_drive_file_uris() {
        let uri = Uri::from_str("file:///C:/work/my%20template.rhbs").unwrap();
        assert_eq!(
            uri_path(&uri),
            Some(PathBuf::from(r"C:\work\my template.rhbs"))
        );
    }

    #[cfg(windows)]
    #[test]
    fn converts_unc_file_uris() {
        let uri = Uri::from_str("file://server/share/template.rhbs").unwrap();
        assert_eq!(
            uri_path(&uri),
            Some(PathBuf::from(r"\\server\share\template.rhbs"))
        );
    }

    #[test]
    fn recognizes_only_project_index_inputs() {
        assert!(is_project_input(Path::new("src/main.rs")));
        assert!(is_project_input(Path::new("Cargo.toml")));
        assert!(is_project_input(Path::new("Cargo.lock")));
        assert!(!is_project_input(Path::new("templates/page.rhbs")));
    }

    #[test]
    fn prefers_workspace_folders_to_the_legacy_root_uri() {
        let workspace = tempdir().unwrap();
        let fallback = tempdir().unwrap();
        let workspace_uri = path_uri(workspace.path()).unwrap();
        let fallback_uri = path_uri(fallback.path()).unwrap();
        let parameters = json!({
            "workspaceFolders": [{
                "uri": workspace_uri.as_str(),
                "name": "workspace"
            }],
            "rootUri": fallback_uri.as_str()
        });

        assert_eq!(
            workspace_root(&parameters).as_deref(),
            Some(workspace.path())
        );
    }

    #[test]
    fn handles_a_rustrover_document_lifecycle_over_lsp() {
        let (server, client) = Connection::memory();
        let server_thread = thread::spawn(move || run_connection(server).unwrap());
        let directory = tempdir().unwrap();
        let root_uri = path_uri(directory.path()).unwrap();
        let document_uri = path_uri(&directory.path().join("template.rhbs")).unwrap();
        client
            .sender
            .send(Message::Request(Request::new(
                RequestId::from(1),
                "initialize".to_owned(),
                json!({
                    "processId": null,
                    "rootUri": root_uri.as_str(),
                    "capabilities": {
                        "general": {
                            "positionEncodings": ["utf-16"]
                        },
                        "textDocument": {
                            "synchronization": {
                                "dynamicRegistration": true,
                                "didSave": true
                            }
                        }
                    },
                    "clientInfo": {
                        "name": "RustRover",
                        "version": "2025.3.1"
                    }
                }),
            )))
            .unwrap();
        let response = receive(&client);
        assert!(
            matches!(response, Message::Response(response) if response.response_result.is_ok())
        );
        client
            .sender
            .send(Message::Notification(Notification::new(
                "initialized".to_owned(),
                json!({}),
            )))
            .unwrap();

        client
            .sender
            .send(Message::Notification(Notification::new(
                "textDocument/didOpen".to_owned(),
                json!({
                    "textDocument": {
                        "uri": document_uri.as_str(),
                        "languageId": "rusty-handlebars",
                        "version": 1,
                        "text": "{{#if value}}"
                    }
                }),
            )))
            .unwrap();
        assert!(!receive_diagnostics(&client).diagnostics.is_empty());

        client
            .sender
            .send(Message::Notification(Notification::new(
                "textDocument/didChange".to_owned(),
                json!({
                    "textDocument": {
                        "uri": document_uri.as_str(),
                        "version": 2
                    },
                    "contentChanges": [{"text": "plain text"}]
                }),
            )))
            .unwrap();
        let diagnostics = receive_diagnostics(&client);
        assert!(diagnostics.diagnostics.is_empty());
        assert_eq!(diagnostics.version, Some(2));

        client
            .sender
            .send(Message::Notification(Notification::new(
                "textDocument/didSave".to_owned(),
                json!({
                    "textDocument": {
                        "uri": document_uri.as_str()
                    }
                }),
            )))
            .unwrap();
        assert!(receive_diagnostics(&client).diagnostics.is_empty());

        client
            .sender
            .send(Message::Notification(Notification::new(
                "textDocument/didClose".to_owned(),
                json!({
                    "textDocument": {
                        "uri": document_uri.as_str()
                    }
                }),
            )))
            .unwrap();
        assert!(receive_diagnostics(&client).diagnostics.is_empty());

        client
            .sender
            .send(Message::Request(Request::new(
                RequestId::from(2),
                "shutdown".to_owned(),
                Value::Null,
            )))
            .unwrap();
        let response = receive(&client);
        assert!(
            matches!(response, Message::Response(response) if response.response_result.is_ok())
        );
        client
            .sender
            .send(Message::Notification(Notification::new(
                "exit".to_owned(),
                Value::Null,
            )))
            .unwrap();
        server_thread.join().unwrap();
    }

    fn receive(connection: &Connection) -> Message {
        connection
            .receiver
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
    }

    fn receive_diagnostics(connection: &Connection) -> PublishDiagnosticsParams {
        let Message::Notification(notification) = receive(connection) else {
            panic!("expected a diagnostics notification");
        };
        assert_eq!(notification.method, "textDocument/publishDiagnostics");
        serde_json::from_value(notification.params).unwrap()
    }
}
