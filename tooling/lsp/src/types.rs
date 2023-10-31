use noirc_frontend::graph::CrateName;
use serde::{Deserialize, Serialize};

// Re-providing lsp_types that we don't need to override
pub(crate) use lsp_types::{
    CodeLens, CodeLensOptions, CodeLensParams, Command, Diagnostic, DiagnosticSeverity,
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializeParams, InitializedParams,
    LogMessageParams, MessageType, Position, PublishDiagnosticsParams, Range, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncOptions, Url,
};

pub(crate) mod request {
    use lsp_types::{request::Request, InitializeParams};

    use super::{
        InitializeResult, NargoTestRunParams, NargoTestRunResult, NargoTestsParams,
        NargoTestsResult,
    };

    // Re-providing lsp_types that we don't need to override
    pub(crate) use lsp_types::request::{CodeLensRequest as CodeLens, Shutdown};

    #[derive(Debug)]
    pub(crate) struct Initialize;
    impl Request for Initialize {
        type Params = InitializeParams;
        type Result = InitializeResult;
        const METHOD: &'static str = "initialize";
    }

    #[derive(Debug)]
    pub(crate) struct NargoTestRun;
    impl Request for NargoTestRun {
        type Params = NargoTestRunParams;
        type Result = NargoTestRunResult;
        const METHOD: &'static str = "nargo/tests/run";
    }

    #[derive(Debug)]
    pub(crate) struct NargoTests;
    impl Request for NargoTests {
        type Params = NargoTestsParams;
        type Result = NargoTestsResult;
        const METHOD: &'static str = "nargo/tests";
    }
}

pub(crate) mod notification {
    use lsp_types::notification::Notification;

    use super::NargoPackageTests;

    // Re-providing lsp_types that we don't need to override
    pub(crate) use lsp_types::notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        DidSaveTextDocument, Exit, Initialized,
    };

    pub(crate) struct NargoUpdateTests;
    impl Notification for NargoUpdateTests {
        type Params = NargoPackageTests;
        const METHOD: &'static str = "nargo/tests/update";
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NargoTestsOptions {
    /// Tests can be requested from the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fetch: Option<bool>,

    /// Tests runs can be requested from the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) run: Option<bool>,

    /// The server will send notifications to update tests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) update: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NargoCapability {
    /// The server will provide various features related to testing within Nargo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tests: Option<NargoTestsOptions>,
}

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServerCapabilities {
    /// Defines how text documents are synced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) text_document_sync: Option<TextDocumentSyncCapability>,

    /// The server provides code lens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) code_lens_provider: Option<CodeLensOptions>,

    /// The server handles and provides custom nargo messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) nargo: Option<NargoCapability>,
}

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InitializeResult {
    /// The capabilities the language server provides.
    pub(crate) capabilities: ServerCapabilities,

    /// Information about the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) server_info: Option<ServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct NargoTestId {
    package: CrateName,
    fully_qualified_path: String,
}

impl TryFrom<String> for NargoTestId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some((crate_name, function_name)) = value.split_once('/') {
            let crate_name = crate_name.parse()?;
            Ok(Self { package: crate_name, fully_qualified_path: function_name.to_string() })
        } else {
            Err("NargoTestId should be serialized as package_name/fully_qualified_path".to_string())
        }
    }
}

impl From<NargoTestId> for String {
    fn from(value: NargoTestId) -> Self {
        format!("{}/{}", value.package, value.fully_qualified_path)
    }
}

impl NargoTestId {
    pub(crate) fn new(crate_name: CrateName, function_name: String) -> Self {
        Self { package: crate_name, fully_qualified_path: function_name }
    }

    pub(crate) fn crate_name(&self) -> &CrateName {
        &self.package
    }

    pub(crate) fn function_name(&self) -> &String {
        &self.fully_qualified_path
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NargoTest {
    pub(crate) id: NargoTestId,
    /// Fully-qualified path to the test within the crate
    pub(crate) label: String,
    pub(crate) range: Range,
    pub(crate) uri: Url,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NargoPackageTests {
    pub(crate) package: String,
    pub(crate) tests: Vec<NargoTest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NargoTestsParams {}

pub(crate) type NargoTestsResult = Option<Vec<NargoPackageTests>>;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NargoTestRunParams {
    pub(crate) id: NargoTestId,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NargoTestRunResult {
    pub(crate) id: NargoTestId,
    pub(crate) result: String,
    pub(crate) message: Option<String>,
}

pub(crate) type CodeLensResult = Option<Vec<CodeLens>>;
