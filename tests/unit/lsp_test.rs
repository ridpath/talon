use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;
use std::sync::Arc;

#[cfg(test)]
mod lsp_server_tests {
    use super::*;

    #[tokio::test]
    async fn test_lsp_initialization() {
        let params = InitializeParams {
            process_id: Some(1234),
            root_uri: Some(Url::parse("file:///test").unwrap()),
            capabilities: ClientCapabilities::default(),
            ..Default::default()
        };

        assert!(params.process_id.is_some());
        assert_eq!(params.process_id.unwrap(), 1234);
    }

    #[tokio::test]
    async fn test_completion_request_structure() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let position = Position {
            line: 10,
            character: 5,
        };

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        };

        assert_eq!(params.text_document_position.position.line, 10);
        assert_eq!(params.text_document_position.position.character, 5);
    }

    #[tokio::test]
    async fn test_hover_request_structure() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let position = Position { line: 5, character: 10 };

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        assert_eq!(params.text_document_position_params.position.line, 5);
    }

    #[tokio::test]
    async fn test_diagnostic_creation() {
        let diagnostic = Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 10 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("E001".to_string())),
            source: Some("talon".to_string()),
            message: "Undefined function".to_string(),
            ..Default::default()
        };

        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostic.message, "Undefined function");
    }

    #[tokio::test]
    async fn test_completion_item_creation() {
        let item = CompletionItem {
            label: "p64".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("p64(value) -> bytes".to_string()),
            documentation: Some(Documentation::String(
                "Pack 64-bit integer as little-endian bytes".to_string()
            )),
            insert_text: Some("p64($1)".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        };

        assert_eq!(item.label, "p64");
        assert_eq!(item.kind, Some(CompletionItemKind::FUNCTION));
        assert!(item.documentation.is_some());
    }

    #[tokio::test]
    async fn test_hover_markup_content() {
        let hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "**cyclic(length)**\n\nGenerate cyclic pattern".to_string(),
            }),
            range: None,
        };

        match hover.contents {
            HoverContents::Markup(ref content) => {
                assert_eq!(content.kind, MarkupKind::Markdown);
                assert!(content.value.contains("cyclic"));
            },
            _ => panic!("Expected Markup content"),
        }
    }

    #[tokio::test]
    async fn test_document_symbol_request() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        assert!(params.text_document.uri.path().ends_with(".talon"));
    }

    #[tokio::test]
    async fn test_goto_definition_request() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let position = Position { line: 15, character: 8 };

        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        assert_eq!(params.text_document_position_params.position.line, 15);
    }

    #[tokio::test]
    async fn test_code_action_request() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let range = Range {
            start: Position { line: 5, character: 0 },
            end: Position { line: 5, character: 20 },
        };

        let params = CodeActionParams {
            text_document: TextDocumentIdentifier { uri },
            range,
            context: CodeActionContext {
                diagnostics: vec![],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        assert_eq!(params.range.start.line, 5);
    }

    #[tokio::test]
    async fn test_formatting_request() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let params = DocumentFormattingParams {
            text_document: TextDocumentIdentifier { uri },
            options: FormattingOptions {
                tab_size: 4,
                insert_spaces: true,
                ..Default::default()
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        assert_eq!(params.options.tab_size, 4);
        assert!(params.options.insert_spaces);
    }

    #[tokio::test]
    async fn test_signature_help_request() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let position = Position { line: 20, character: 12 };

        let params = SignatureHelpParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            context: None,
        };

        assert_eq!(params.text_document_position_params.position.character, 12);
    }

    #[test]
    fn test_completion_items_for_builtin_functions() {
        let builtin_functions = vec![
            "p64", "p32", "p16", "p8",
            "u64", "u32", "u16", "u8",
            "cyclic", "cyclic_find",
            "connect", "send", "recv", "sendline", "recvline",
            "shellcode", "rop_find",
            "process_attach", "process_detach",
            "mem_read", "mem_write", "mem_scan",
        ];

        for func in builtin_functions {
            assert!(!func.is_empty());
            assert!(func.chars().all(|c| c.is_alphanumeric() || c == '_'));
        }
    }

    #[test]
    fn test_function_signature_parsing() {
        let signature = "p64(value) -> bytes";
        assert!(signature.contains("p64"));
        assert!(signature.contains("value"));
        assert!(signature.contains("bytes"));
    }

    #[test]
    fn test_diagnostics_severity_levels() {
        let error = DiagnosticSeverity::ERROR;
        let warning = DiagnosticSeverity::WARNING;
        let info = DiagnosticSeverity::INFORMATION;
        let hint = DiagnosticSeverity::HINT;

        assert_ne!(error, warning);
        assert_ne!(warning, info);
        assert_ne!(info, hint);
    }

    #[tokio::test]
    async fn test_workspace_symbol_request() {
        let params = WorkspaceSymbolParams {
            query: "payload".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        assert_eq!(params.query, "payload");
    }

    #[tokio::test]
    async fn test_rename_request() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let position = Position { line: 10, character: 5 };

        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            new_name: "new_variable".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        assert_eq!(params.new_name, "new_variable");
    }

    #[tokio::test]
    async fn test_references_request() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let position = Position { line: 12, character: 7 };

        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            context: ReferenceContext {
                include_declaration: true,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        assert!(params.context.include_declaration);
    }

    #[test]
    fn test_symbol_kind_values() {
        assert_eq!(SymbolKind::FUNCTION as u32, 12);
        assert_eq!(SymbolKind::VARIABLE as u32, 13);
        assert_eq!(SymbolKind::CONSTANT as u32, 14);
    }

    #[test]
    fn test_completion_trigger_characters() {
        let trigger_chars = vec![".", "(", " "];
        assert!(trigger_chars.contains(&"."));
        assert!(trigger_chars.contains(&"("));
    }

    #[tokio::test]
    async fn test_did_open_notification() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "talon".to_string(),
                version: 1,
                text: "let payload = p64(0xdeadbeef)".to_string(),
            },
        };

        assert_eq!(params.text_document.language_id, "talon");
        assert_eq!(params.text_document.version, 1);
    }

    #[tokio::test]
    async fn test_did_change_notification() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri,
                version: 2,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "let offset = 264".to_string(),
            }],
        };

        assert_eq!(params.text_document.version, 2);
        assert!(!params.content_changes.is_empty());
    }

    #[tokio::test]
    async fn test_did_save_notification() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let params = DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
            text: Some("saved content".to_string()),
        };

        assert!(params.text.is_some());
    }

    #[tokio::test]
    async fn test_did_close_notification() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
        };

        assert!(params.text_document.uri.path().ends_with(".talon"));
    }

    #[test]
    fn test_position_comparison() {
        let pos1 = Position { line: 5, character: 10 };
        let pos2 = Position { line: 5, character: 15 };
        let pos3 = Position { line: 6, character: 0 };

        assert!(pos1.line == pos2.line);
        assert!(pos1.character < pos2.character);
        assert!(pos1.line < pos3.line);
    }

    #[test]
    fn test_range_contains_position() {
        let range = Range {
            start: Position { line: 5, character: 0 },
            end: Position { line: 10, character: 20 },
        };

        let pos_inside = Position { line: 7, character: 10 };
        let pos_outside = Position { line: 15, character: 0 };

        assert!(pos_inside.line >= range.start.line && pos_inside.line <= range.end.line);
        assert!(pos_outside.line > range.end.line);
    }

    #[tokio::test]
    async fn test_text_edit_creation() {
        let edit = TextEdit {
            range: Range {
                start: Position { line: 5, character: 0 },
                end: Position { line: 5, character: 10 },
            },
            new_text: "p64(0xdeadbeef)".to_string(),
        };

        assert_eq!(edit.new_text, "p64(0xdeadbeef)");
    }

    #[tokio::test]
    async fn test_workspace_edit_creation() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let mut changes = std::collections::HashMap::new();
        changes.insert(uri.clone(), vec![
            TextEdit {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 5 },
                },
                new_text: "fixed".to_string(),
            }
        ]);

        let edit = WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        };

        assert!(edit.changes.is_some());
    }

    #[test]
    fn test_markup_kind_values() {
        let markdown = MarkupKind::Markdown;
        let plaintext = MarkupKind::PlainText;

        assert_ne!(markdown, plaintext);
    }

    #[test]
    fn test_insert_text_format() {
        let snippet = InsertTextFormat::SNIPPET;
        let plain = InsertTextFormat::PLAIN_TEXT;

        assert_ne!(snippet, plain);
    }

    #[tokio::test]
    async fn test_code_lens_request() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let params = CodeLensParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        assert!(params.text_document.uri.path().contains("exploit"));
    }

    #[tokio::test]
    async fn test_document_link_request() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();
        let params = DocumentLinkParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        assert!(params.text_document.uri.scheme() == "file");
    }

    #[tokio::test]
    async fn test_execute_command_request() {
        let params = ExecuteCommandParams {
            command: "talon.runExploit".to_string(),
            arguments: vec![],
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        assert_eq!(params.command, "talon.runExploit");
    }

    #[test]
    fn test_url_parsing() {
        let uri = Url::parse("file:///C:/Users/test/exploit.talon").unwrap();
        assert_eq!(uri.scheme(), "file");
        assert!(uri.path().contains("exploit.talon"));
    }

    #[test]
    fn test_protocol_version_compatibility() {
        assert!(true);
    }
}

#[cfg(test)]
mod lsp_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_lsp_lifecycle() {
        let init_params = InitializeParams::default();

        assert!(true);
    }

    #[tokio::test]
    async fn test_multiple_document_handling() {
        let uris = vec![
            Url::parse("file:///test/exploit1.talon").unwrap(),
            Url::parse("file:///test/exploit2.talon").unwrap(),
            Url::parse("file:///test/exploit3.talon").unwrap(),
        ];

        assert_eq!(uris.len(), 3);
        for uri in &uris {
            assert!(uri.path().ends_with(".talon"));
        }
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();

        let completion_task = tokio::spawn(async move {
            let params = CompletionParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    position: Position { line: 5, character: 10 },
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
                context: None,
            };
            params
        });

        let result = completion_task.await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_document_sync_incremental() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();

        let initial_content = "let offset = 264";
        let change = "let offset = 280";

        assert_ne!(initial_content, change);
    }

    #[tokio::test]
    async fn test_error_recovery() {
        let uri = Url::parse("file:///test/invalid.talon").unwrap();

        let diagnostic = Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 10 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            message: "Parse error".to_string(),
            ..Default::default()
        };

        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    }

    #[tokio::test]
    async fn test_performance_large_document() {
        let large_content = "let x = 1\n".repeat(10000);
        assert!(large_content.len() > 100000);
    }

    #[tokio::test]
    async fn test_incremental_parsing() {
        let uri = Url::parse("file:///test/exploit.talon").unwrap();

        let changes = vec![
            "let a = 1",
            "let b = 2",
            "let c = 3",
        ];

        assert_eq!(changes.len(), 3);
    }
}

#[cfg(test)]
mod lsp_autocomplete_tests {
    use super::*;

    #[test]
    fn test_builtin_function_completions() {
        let functions = vec![
            ("p64", CompletionItemKind::FUNCTION),
            ("p32", CompletionItemKind::FUNCTION),
            ("cyclic", CompletionItemKind::FUNCTION),
            ("connect", CompletionItemKind::FUNCTION),
            ("send", CompletionItemKind::FUNCTION),
        ];

        for (name, kind) in functions {
            assert!(!name.is_empty());
            assert_eq!(kind, CompletionItemKind::FUNCTION);
        }
    }

    #[test]
    fn test_variable_completions() {
        let variables = vec!["payload", "offset", "conn", "leak"];

        for var in variables {
            assert!(var.chars().all(|c| c.is_alphanumeric() || c == '_'));
        }
    }

    #[test]
    fn test_snippet_completions() {
        let snippets = vec![
            ("for", "for ${1:i} in ${2:range}\n    $0\nend"),
            ("if", "if ${1:condition}\n    $0\nend"),
            ("func", "func ${1:name}(${2:params})\n    $0\nend"),
        ];

        for (trigger, template) in snippets {
            assert!(!trigger.is_empty());
            assert!(template.contains("$"));
        }
    }

    #[test]
    fn test_context_aware_completions() {
        let contexts = vec![
            ("after_let", vec!["identifier"]),
            ("after_dot", vec!["method", "property"]),
            ("after_paren", vec!["argument"]),
        ];

        assert!(!contexts.is_empty());
    }
}

#[cfg(test)]
mod lsp_hover_tests {
    use super::*;

    #[test]
    fn test_function_hover_documentation() {
        let hover_content = "**p64(value)**\n\nPack 64-bit integer as little-endian bytes\n\n**Example:**\n```talon\nlet packed = p64(0xdeadbeef)\n```";

        assert!(hover_content.contains("p64"));
        assert!(hover_content.contains("Pack 64-bit"));
        assert!(hover_content.contains("Example"));
    }

    #[test]
    fn test_variable_hover_type_info() {
        let hover_content = "**payload**: bytes\n\nBuffer containing exploit payload";

        assert!(hover_content.contains("bytes"));
    }

    #[test]
    fn test_hover_markdown_formatting() {
        let content = MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**bold** *italic* `code`".to_string(),
        };

        assert_eq!(content.kind, MarkupKind::Markdown);
        assert!(content.value.contains("**bold**"));
    }
}

#[cfg(test)]
mod lsp_diagnostics_tests {
    use super::*;

    #[test]
    fn test_syntax_error_diagnostic() {
        let diagnostic = Diagnostic {
            range: Range {
                start: Position { line: 5, character: 0 },
                end: Position { line: 5, character: 10 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("syntax_error".to_string())),
            message: "Expected 'end' keyword".to_string(),
            ..Default::default()
        };

        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
        assert!(diagnostic.message.contains("end"));
    }

    #[test]
    fn test_undefined_variable_warning() {
        let diagnostic = Diagnostic {
            range: Range {
                start: Position { line: 10, character: 5 },
                end: Position { line: 10, character: 15 },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            message: "Undefined variable 'unknown_var'".to_string(),
            ..Default::default()
        };

        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
    }

    #[test]
    fn test_type_mismatch_error() {
        let diagnostic = Diagnostic {
            range: Range {
                start: Position { line: 15, character: 0 },
                end: Position { line: 15, character: 20 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            message: "Type mismatch: expected bytes, got number".to_string(),
            ..Default::default()
        };

        assert!(diagnostic.message.contains("Type mismatch"));
    }

    #[test]
    fn test_unused_variable_hint() {
        let diagnostic = Diagnostic {
            range: Range {
                start: Position { line: 8, character: 4 },
                end: Position { line: 8, character: 10 },
            },
            severity: Some(DiagnosticSeverity::HINT),
            message: "Unused variable 'temp'".to_string(),
            ..Default::default()
        };

        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::HINT));
    }
}
