
use std::collections::HashMap;
use std::vec;

use lspower::lsp::{CompletionOptions, InitializeParams, InitializeResult, ServerCapabilities, CompletionParams, CompletionResponse, CompletionItem, Diagnostic, DidChangeTextDocumentParams, self, DiagnosticSeverity, HoverProviderCapability, TextDocumentSyncCapability, TextDocumentSyncKind, HoverParams, Hover, Range, HoverContents, MarkupKind, MarkupContent, SignatureHelpOptions, SignatureHelp, SignatureInformation, ParameterInformation, Documentation, ParameterLabel, SignatureHelpParams, OneOf, GotoDefinitionParams, GotoDefinitionResponse, Location, Position, ReferenceParams, DocumentSymbolParams, DocumentSymbolResponse, SymbolInformation, SymbolKind, RenameParams, WorkspaceEdit, TextEdit};
use lspower::jsonrpc::{Result};
#[derive(Default)]
pub struct EvieLanguageServer {}

impl EvieLanguageServer {
    pub fn initialize(&self, _params: InitializeParams) -> InitializeResult {
        let capabilities     = ServerCapabilities {
            completion_provider: 
                Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec![".".to_string(), ",".to_string()]),
                    ..Default::default()
                })
            ,
            hover_provider:  Some(HoverProviderCapability::Simple(true)),
            signature_help_provider: Some(SignatureHelpOptions{
                trigger_characters: Some(vec!["(".to_string()]),
                retrigger_characters: Some(vec![",".to_string()]),
                ..Default::default()
            }),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            rename_provider:  Some(OneOf::Left(true)),
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::INCREMENTAL)),
            ..Default::default()
        };
        InitializeResult {
            capabilities,
            ..Default::default()
        }
    }

    pub fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(
            vec![CompletionItem::new_simple("label".to_string(), "item1".to_string())])))
    }

    pub fn completion_resolve(&self, _params: CompletionItem) -> Result<CompletionItem> {
       Ok(CompletionItem::new_simple("label".to_string(), "item1".to_string()))
    }

    pub fn did_change(&self, params: DidChangeTextDocumentParams) -> (lsp::Url, Vec<lsp::Diagnostic>, Option<i32>) {
        let changes = params.content_changes;
        let diagnostics: Vec<Diagnostic> = changes.into_iter().map(|t| {
            {
                let mut d = Diagnostic::new_simple(t.range.unwrap(), "A simple error".to_string());
                d.severity = Some(DiagnosticSeverity::WARNING);
                d.source = Some("evie".to_string());
                d
            }
        }).collect();
        (params.text_document.uri.clone(), diagnostics, Some(params.text_document.version))
    }

    pub fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let position = params.text_document_position_params.position;
        let markdown = MarkupContent {
            kind: MarkupKind::Markdown,
            value: [
                "### Header",
                "Some text",
                "```typescript",
                "someCode();",
                "```"
            ]
            .join("\n"),
        };
        Ok(Some(Hover{
            contents: HoverContents::Markup(markdown),
            range: Some(Range::new(position, position))
        }))
    }

    pub fn signature_help(&self, _params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let signature = MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Function to poop".to_string(),
        };
        let parameter = MarkupContent {
            kind: MarkupKind::Markdown,
            value: "poop".to_string(),
        };
        let sig =  SignatureInformation{
            label: "Signature".to_string(),
            documentation: Some(Documentation::MarkupContent(signature)),
            parameters: Some(vec![
                ParameterInformation{ label: ParameterLabel::Simple("param1".to_string()), documentation: Some(Documentation::MarkupContent(parameter.clone()))},
                ParameterInformation{ label: ParameterLabel::Simple("param2".to_string()), documentation: Some(Documentation::MarkupContent(parameter))}
                ]),
            active_parameter: Some(0)
        };
        Ok(Some(SignatureHelp {
            signatures: vec![sig],
            active_signature: Some(0),
            active_parameter: Some(0),
        }))
    }

    pub fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let position = Range::new(Position::new(0, 3), Position::new(0, 5));
        Ok(Some(GotoDefinitionResponse::Scalar(Location::new(params.text_document_position_params.text_document.uri, position))))
    }

    pub fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        Ok(Some(vec![
            Location::new(uri.clone(), Range::new(Position::new(0, 3), Position::new(0, 5))),
            Location::new(uri, Range::new(Position::new(2, 3), Position::new(2, 5)))
        ]))
    }

    pub fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        #[allow(deprecated)]
        let symbol = SymbolInformation{
            name: "hoo".to_string(),
            kind: SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            location: Location::new(uri, Range::new(Position::new(0, 3), Position::new(0, 5))),
            container_name: Some("Hooo".to_string())
        };
        let d = DocumentSymbolResponse::Flat(vec![
            symbol
        ]);
        Ok(Some(d))
    }

    pub fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let new_name = params.new_name;
        let mut changes = HashMap::new();
        changes.insert(uri, vec![
            TextEdit::new(Range::new(Position::new(0, 3), Position::new(0, 5)), new_name)
        ]);
        #[allow(deprecated)]
        let edit = WorkspaceEdit{
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        };
        Ok(Some(edit))
    }
}
