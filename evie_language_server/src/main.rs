use evie_language_server::EvieLanguageServer;
use lspower::jsonrpc::Result;
use lspower::lsp::*;
use lspower::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
    els: EvieLanguageServer,
}

#[lspower::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, p: InitializeParams) -> Result<InitializeResult> {
        Ok(self.els.initialize(p))
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized hooo!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) -> () {
        let (uri, diags, _version) = self.els.did_change(params);
        self.client
        .log_message(MessageType::INFO, "Send diagnostic")
        .await;
        self.client.publish_diagnostics(uri, diags, None).await;
    }

    async fn completion(&self, d: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.els.completion(d)
    }

    async fn completion_resolve(&self, d: CompletionItem) -> Result<CompletionItem> {
        self.els.completion_resolve(d)
    }


    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.els.hover(params)
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        self.els.signature_help(params)
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>>{
        self.els.goto_definition(params)

    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>>{
        self.els.references(params)

    }
    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        self.els.document_symbol(params)
    }
      
    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        self.els.rename(params)
    }

}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let els = EvieLanguageServer::default();
    let (service, messages) = LspService::new(|client| Backend { client, els });
    Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service)
        .await;
}
