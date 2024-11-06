#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    future::Future,
    ops::{self, ControlFlow},
    path::{Path, PathBuf},
    pin::Pin,
    str::FromStr,
    task::{self, Poll},
};

use acvm::{BlackBoxFunctionSolver, FieldElement};
use async_lsp::{
    router::Router, AnyEvent, AnyNotification, AnyRequest, ClientSocket, Error, LspService,
    ResponseError,
};
use fm::{codespan_files as files, FileManager};
use fxhash::FxHashSet;
use lsp_types::{
    request::{
        CodeActionRequest, Completion, DocumentSymbolRequest, HoverRequest, InlayHintRequest,
        PrepareRenameRequest, References, Rename, SignatureHelpRequest,
    },
    CodeLens,
};
use nargo::{
    package::{Package, PackageType},
    parse_all,
    workspace::Workspace,
};
use nargo_toml::{find_file_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{file_manager_with_stdlib, prepare_crate, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::{
    graph::{CrateGraph, CrateId, CrateName},
    hir::{
        def_map::{parse_file, CrateDefMap},
        Context, FunctionNameMatch, ParsedFiles,
    },
    node_interner::NodeInterner,
    parser::ParserError,
    usage_tracker::UsageTracker,
    ParsedModule,
};
use rayon::prelude::*;

use notifications::{
    on_did_change_configuration, on_did_change_text_document, on_did_close_text_document,
    on_did_open_text_document, on_did_save_text_document, on_exit, on_initialized,
};
use requests::{
    on_code_action_request, on_code_lens_request, on_completion_request,
    on_document_symbol_request, on_formatting, on_goto_declaration_request,
    on_goto_definition_request, on_goto_type_definition_request, on_hover_request, on_initialize,
    on_inlay_hint_request, on_prepare_rename_request, on_references_request, on_rename_request,
    on_shutdown, on_signature_help_request, on_test_run_request, on_tests_request,
    LspInitializationOptions,
};
use serde_json::Value as JsonValue;
use thiserror::Error;
use tower::Service;

mod attribute_reference_finder;
mod modules;
mod notifications;
mod requests;
mod solver;
mod tests;
mod trait_impl_method_stub_generator;
mod types;
mod use_segment_positions;
mod utils;
mod visibility;

#[cfg(test)]
mod test_utils;

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
    cached_parsed_files: HashMap<PathBuf, (usize, (ParsedModule, Vec<ParserError>))>,
    workspace_cache: HashMap<PathBuf, WorkspaceCacheData>,
    package_cache: HashMap<PathBuf, PackageCacheData>,
    options: LspInitializationOptions,

    // Tracks files that currently have errors, by package root.
    files_with_errors: HashMap<PathBuf, HashSet<Url>>,
}

struct WorkspaceCacheData {
    file_manager: FileManager,
}

struct PackageCacheData {
    crate_id: CrateId,
    crate_graph: CrateGraph,
    node_interner: NodeInterner,
    def_maps: BTreeMap<CrateId, CrateDefMap>,
    usage_tracker: UsageTracker,
}

impl LspState {
    fn new(
        client: &ClientSocket,
        solver: impl BlackBoxFunctionSolver<FieldElement> + 'static,
    ) -> Self {
        Self {
            client: client.clone(),
            root_path: None,
            solver: WrapperSolver(Box::new(solver)),
            input_files: HashMap::new(),
            cached_lenses: HashMap::new(),
            cached_parsed_files: HashMap::new(),
            workspace_cache: HashMap::new(),
            package_cache: HashMap::new(),
            open_documents_count: 0,
            options: Default::default(),
            files_with_errors: HashMap::new(),
        }
    }
}

pub struct NargoLspService {
    router: Router<LspState>,
}

impl NargoLspService {
    pub fn new(
        client: &ClientSocket,
        solver: impl BlackBoxFunctionSolver<FieldElement> + 'static,
    ) -> Self {
        let state = LspState::new(client, solver);
        let mut router = Router::new(state);
        router
            .request::<request::Initialize, _>(on_initialize)
            .request::<request::Formatting, _>(on_formatting)
            .request::<request::Shutdown, _>(on_shutdown)
            .request::<request::CodeLens, _>(on_code_lens_request)
            .request::<request::NargoTests, _>(on_tests_request)
            .request::<request::NargoTestRun, _>(on_test_run_request)
            .request::<request::GotoDefinition, _>(on_goto_definition_request)
            .request::<request::GotoDeclaration, _>(on_goto_declaration_request)
            .request::<request::GotoTypeDefinition, _>(on_goto_type_definition_request)
            .request::<DocumentSymbolRequest, _>(on_document_symbol_request)
            .request::<References, _>(on_references_request)
            .request::<PrepareRenameRequest, _>(on_prepare_rename_request)
            .request::<Rename, _>(on_rename_request)
            .request::<HoverRequest, _>(on_hover_request)
            .request::<InlayHintRequest, _>(on_inlay_hint_request)
            .request::<Completion, _>(on_completion_request)
            .request::<SignatureHelpRequest, _>(on_signature_help_request)
            .request::<CodeActionRequest, _>(on_code_action_request)
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
    if let Some(toml_path) = find_file_manifest(file_path) {
        match resolve_workspace_from_toml(
            &toml_path,
            PackageSelection::All,
            Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
        ) {
            Ok(workspace) => return Ok(workspace),
            Err(error) => {
                eprintln!("Error while processing {:?}: {}", toml_path, error);
            }
        }
    }

    let Some(parent_folder) = file_path
        .parent()
        .and_then(|f| f.file_name())
        .and_then(|file_name_os_str| file_name_os_str.to_str())
    else {
        return Err(LspError::WorkspaceResolutionError(format!(
            "Could not resolve parent folder for file: {:?}",
            file_path
        )));
    };

    let crate_name = match CrateName::from_str(parent_folder) {
        Ok(name) => name,
        Err(error) => {
            eprintln!("{}", error);
            CrateName::from_str("root").unwrap()
        }
    };

    let assumed_package = Package {
        version: None,
        compiler_required_version: Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
        root_dir: PathBuf::from(parent_folder),
        package_type: PackageType::Binary,
        entry_path: PathBuf::from(file_path),
        name: crate_name,
        dependencies: BTreeMap::new(),
        expression_width: None,
    };
    let workspace = Workspace {
        root_dir: PathBuf::from(parent_folder),
        members: vec![assumed_package],
        selected_package_index: Some(0),
        is_assumed: true,
    };
    Ok(workspace)
}

pub(crate) fn workspace_package_for_file<'a>(
    workspace: &'a Workspace,
    file_path: &Path,
) -> Option<&'a Package> {
    if workspace.is_assumed {
        workspace.members.first()
    } else {
        workspace.members.iter().find(|package| file_path.starts_with(&package.root_dir))
    }
}

pub(crate) fn prepare_package<'file_manager, 'parsed_files>(
    file_manager: &'file_manager FileManager,
    parsed_files: &'parsed_files ParsedFiles,
    package: &Package,
) -> (Context<'file_manager, 'parsed_files>, CrateId) {
    let (mut context, crate_id) = nargo::prepare_package(file_manager, parsed_files, package);
    context.activate_lsp_mode();
    (context, crate_id)
}

/// Prepares a package from a source string
/// This is useful for situations when we don't need dependencies
/// and just need to operate on single file.
///
/// Use case for this is the LSP server and code lenses
/// which operate on single file and need to understand this file
/// in order to offer code lenses to the user
fn prepare_source(source: String, state: &mut LspState) -> (Context<'static, 'static>, CrateId) {
    let root = Path::new("");
    let file_name = Path::new("main.nr");
    let mut file_manager = file_manager_with_stdlib(root);
    file_manager.add_file_with_source(file_name, source).expect(
        "Adding source buffer to file manager should never fail when file manager is empty",
    );
    let parsed_files = parse_diff(&file_manager, state);

    let mut context = Context::new(file_manager, parsed_files);
    context.activate_lsp_mode();

    let root_crate_id = prepare_crate(&mut context, file_name);

    (context, root_crate_id)
}

fn parse_diff(file_manager: &FileManager, state: &mut LspState) -> ParsedFiles {
    if state.options.enable_parsing_cache {
        let noir_file_hashes: Vec<_> = file_manager
            .as_file_map()
            .all_file_ids()
            .par_bridge()
            .filter_map(|&file_id| {
                let file_path = file_manager.path(file_id).expect("expected file to exist");
                let file_extension =
                    file_path.extension().expect("expected all file paths to have an extension");
                if file_extension == "nr" {
                    Some((
                        file_id,
                        file_path.to_path_buf(),
                        fxhash::hash(file_manager.fetch_file(file_id).expect("file must exist")),
                    ))
                } else {
                    None
                }
            })
            .collect();

        let cache_hits: Vec<_> = noir_file_hashes
            .par_iter()
            .filter_map(|(file_id, file_path, current_hash)| {
                let cached_version = state.cached_parsed_files.get(file_path);
                if let Some((hash, cached_parsing)) = cached_version {
                    if hash == current_hash {
                        return Some((*file_id, cached_parsing.clone()));
                    }
                }
                None
            })
            .collect();

        let cache_hits_ids: FxHashSet<_> = cache_hits.iter().map(|(file_id, _)| *file_id).collect();

        let cache_misses: Vec<_> = noir_file_hashes
            .into_par_iter()
            .filter(|(id, _, _)| !cache_hits_ids.contains(id))
            .map(|(file_id, path, hash)| (file_id, path, hash, parse_file(file_manager, file_id)))
            .collect();

        cache_misses.iter().for_each(|(_, path, hash, parse_results)| {
            state.cached_parsed_files.insert(path.clone(), (*hash, parse_results.clone()));
        });

        cache_misses
            .into_iter()
            .map(|(id, _, _, parse_results)| (id, parse_results))
            .chain(cache_hits)
            .collect()
    } else {
        parse_all(file_manager)
    }
}

pub fn insert_all_files_for_workspace_into_file_manager(
    state: &LspState,
    workspace: &Workspace,
    file_manager: &mut FileManager,
) {
    // Source code for files we cached override those that are read from disk.
    let mut overrides: HashMap<&Path, &str> = HashMap::new();
    for (path, source) in &state.input_files {
        let path = path.strip_prefix("file://").unwrap();
        overrides.insert(Path::new(path), source);
    }

    nargo::insert_all_files_for_workspace_into_file_manager_with_overrides(
        workspace,
        file_manager,
        &overrides,
    );
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

    let client = ClientSocket::new_closed();
    let mut state = LspState::new(&client, acvm::blackbox_solver::StubbedBlackBoxSolver);

    let (mut context, crate_id) = prepare_source(source.to_string(), &mut state);
    let _check_result = noirc_driver::check_crate(&mut context, crate_id, &Default::default());
    let main_func_id = context.get_main_function(&crate_id);
    assert!(main_func_id.is_some());
}
