#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use std::{
    collections::HashMap,
    future::Future,
    ops::{self, ControlFlow},
    path::{Path, PathBuf},
    pin::Pin,
    task::{self, Poll},
};

use acvm::BlackBoxFunctionSolver;
use async_lsp::{
    router::Router, AnyEvent, AnyNotification, AnyRequest, ClientSocket, Error, LspService,
    ResponseError,
};
use fm::codespan_files as files;
use lsp_types::CodeLens;
use nargo::workspace::Workspace;
use nargo_toml::{find_file_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{file_manager_with_stdlib, prepare_crate, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::{
    graph::{CrateId, CrateName},
    hir::{Context, FunctionNameMatch},
    node_interner::NodeInterner,
};

use notifications::{
    on_did_change_configuration, on_did_change_text_document, on_did_close_text_document,
    on_did_open_text_document, on_did_save_text_document, on_exit, on_initialized,
};
use requests::{
    on_code_lens_request, on_formatting, on_goto_declaration_request, on_goto_definition_request,
    on_initialize, on_profile_run_request, on_shutdown, on_test_run_request, on_tests_request,
};
use serde_json::Value as JsonValue;
use thiserror::Error;
use tower::Service;

mod notifications;
mod requests;
mod solver;
mod types;

use solver::WrapperSolver;
use types::{notification, request, NargoTest, NargoTestId, Position, Range, Url};

#[derive(Debug, Error)]
pub enum LspError {
    /// Error while Resolving Workspace.
    #[error("Failed to Resolve Workspace - {0}")]
    WorkspaceResolutionError(String),
}

// State for the LSP gets implemented on this struct and is internal to the implementation
pub struct LspState {
    root_path: Option<PathBuf>,
    client: ClientSocket,
    solver: WrapperSolver,
    open_documents_count: usize,
    input_files: HashMap<String, String>,
    cached_lenses: HashMap<String, Vec<CodeLens>>,
    cached_definitions: HashMap<String, NodeInterner>,
}

impl LspState {
    fn new(client: &ClientSocket, solver: impl BlackBoxFunctionSolver + 'static) -> Self {
        Self {
            client: client.clone(),
            root_path: None,
            solver: WrapperSolver(Box::new(solver)),
            input_files: HashMap::new(),
            cached_lenses: HashMap::new(),
            cached_definitions: HashMap::new(),
            open_documents_count: 0,
        }
    }
}

pub struct NargoLspService {
    router: Router<LspState>,
}

impl NargoLspService {
    pub fn new(client: &ClientSocket, solver: impl BlackBoxFunctionSolver + 'static) -> Self {
        let state = LspState::new(client, solver);
        let mut router = Router::new(state);
        router
            .request::<request::Initialize, _>(on_initialize)
            .request::<request::Formatting, _>(on_formatting)
            .request::<request::Shutdown, _>(on_shutdown)
            .request::<request::CodeLens, _>(on_code_lens_request)
            .request::<request::NargoTests, _>(on_tests_request)
            .request::<request::NargoTestRun, _>(on_test_run_request)
            .request::<request::NargoProfileRun, _>(on_profile_run_request)
            .request::<request::GotoDefinition, _>(on_goto_definition_request)
            .request::<request::GotoDeclaration, _>(on_goto_declaration_request)
            .notification::<notification::Initialized>(on_initialized)
            .notification::<notification::DidChangeConfiguration>(on_did_change_configuration)
            .notification::<notification::DidOpenTextDocument>(on_did_open_text_document)
            .notification::<notification::DidChangeTextDocument>(on_did_change_text_document)
            .notification::<notification::DidCloseTextDocument>(on_did_close_text_document)
            .notification::<notification::DidSaveTextDocument>(on_did_save_text_document)
            .notification::<notification::Exit>(on_exit);
        Self { router }
    }
}

// This trait implemented as a passthrough to the router, which makes
// our `NargoLspService` a normal Service as far as Tower is concerned.
impl Service<AnyRequest> for NargoLspService {
    type Response = JsonValue;
    type Error = ResponseError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.router.poll_ready(cx)
    }

    fn call(&mut self, req: AnyRequest) -> Self::Future {
        self.router.call(req)
    }
}

// This trait implemented as a passthrough to the router, which makes
// our `NargoLspService` able to accept the `async-lsp` middleware.
impl LspService for NargoLspService {
    fn notify(&mut self, notification: AnyNotification) -> ControlFlow<Result<(), Error>> {
        self.router.notify(notification)
    }

    fn emit(&mut self, event: AnyEvent) -> ControlFlow<Result<(), Error>> {
        self.router.emit(event)
    }
}

fn get_package_tests_in_crate(
    context: &Context,
    crate_id: &CrateId,
    crate_name: &CrateName,
) -> Option<Vec<NargoTest>> {
    let fm = &context.file_manager;
    let files = fm.as_file_map();
    let tests =
        context.get_all_test_functions_in_crate_matching(crate_id, FunctionNameMatch::Anything);

    let package_tests: Vec<_> = tests
        .into_iter()
        .map(|(func_name, test_function)| {
            let location = context.function_meta(&test_function.get_id()).name.location;
            let file_id = location.file;
            let file_path = fm.path(file_id).expect("file must exist to contain tests");
            let range =
                byte_span_to_range(files, file_id, location.span.into()).unwrap_or_default();
            let file_uri = Url::from_file_path(file_path)
                .expect("Expected a valid file path that can be converted into a URI");

            NargoTest {
                id: NargoTestId::new(crate_name.clone(), func_name.clone()),
                label: func_name,
                uri: file_uri,
                range,
            }
        })
        .collect();

    if package_tests.is_empty() {
        None
    } else {
        Some(package_tests)
    }
}

fn byte_span_to_range<'a, F: files::Files<'a> + ?Sized>(
    files: &'a F,
    file_id: F::FileId,
    span: ops::Range<usize>,
) -> Option<Range> {
    if let Ok(codespan_range) = codespan_lsp::byte_span_to_range(files, file_id, span) {
        // We have to manually construct a Range because the codespan_lsp restricts lsp-types to the wrong version range
        // TODO: codespan is unmaintained and we should probably subsume it. Ref https://github.com/brendanzab/codespan/issues/345
        let range = Range {
            start: Position {
                line: codespan_range.start.line,
                character: codespan_range.start.character,
            },
            end: Position {
                line: codespan_range.end.line,
                character: codespan_range.end.character,
            },
        };
        Some(range)
    } else {
        None
    }
}

pub(crate) fn resolve_workspace_for_source_path(file_path: &Path) -> Result<Workspace, LspError> {
    let package_root = find_file_manifest(file_path);

    let toml_path = package_root.ok_or_else(|| {
        LspError::WorkspaceResolutionError(format!(
            "Nargo.toml not found for file: {:?}",
            file_path
        ))
    })?;

    let workspace = resolve_workspace_from_toml(
        &toml_path,
        PackageSelection::All,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )
    .map_err(|err| LspError::WorkspaceResolutionError(err.to_string()))?;

    Ok(workspace)
}

/// Prepares a package from a source string
/// This is useful for situations when we don't need dependencies
/// and just need to operate on single file.
///
/// Use case for this is the LSP server and code lenses
/// which operate on single file and need to understand this file
/// in order to offer code lenses to the user
fn prepare_source(source: String) -> (Context<'static>, CrateId) {
    let root = Path::new("");
    let file_name = Path::new("main.nr");
    let mut file_manager = file_manager_with_stdlib(root);
    file_manager.add_file_with_source(file_name, source).expect(
        "Adding source buffer to file manager should never fail when file manager is empty",
    );

    let mut context = Context::new(file_manager);
    let root_crate_id = prepare_crate(&mut context, file_name);

    (context, root_crate_id)
}

#[test]
fn prepare_package_from_source_string() {
    let source = r#"
    fn main() {
        let x = 1;
        let y = 2;
        let z = x + y;
    }
    "#;

    let (mut context, crate_id) = crate::prepare_source(source.to_string());
    let _check_result = noirc_driver::check_crate(&mut context, crate_id, false, false);
    let main_func_id = context.get_main_function(&crate_id);
    assert!(main_func_id.is_some());
}
