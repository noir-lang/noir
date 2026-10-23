#![forbid(unsafe_code)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    future::Future,
    hash::BuildHasher,
    ops::{self, ControlFlow},
    path::{Path, PathBuf},
    pin::Pin,
    str::FromStr,
    sync::Arc,
    task::{self, Poll},
};

use acvm::{BlackBoxFunctionSolver, FieldElement};
use async_lsp::lsp_types;
use async_lsp::lsp_types::request::{
    CodeActionRequest, Completion, DocumentSymbolRequest, FoldingRangeRequest, HoverRequest,
    InlayHintRequest, PrepareRenameRequest, References, Rename, SemanticTokensFullRequest,
    SignatureHelpRequest, WorkspaceSymbolRequest,
};
use async_lsp::lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
};
use async_lsp::{
    AnyEvent, AnyNotification, AnyRequest, ClientSocket, Error, LspService, ResponseError,
    router::Router,
};
use fm::{FileManager, codespan_files as files};
use nargo::{
    constants::PKG_FILE,
    package::{Package, PackageType},
    parse_all,
    workspace::Workspace,
};
use nargo_toml::{
    ManifestError, PackageSelection, find_file_manifest, resolve_workspace_from_toml,
};
use noirc_driver::{
    CompileOptions, NOIR_ARTIFACT_VERSION_STRING, add_dep, check_crate_keeping_warnings_from,
    prepare_crate, prepare_dependency,
};
use noirc_errors::CustomDiagnostic;
use noirc_frontend::{
    ParsedModule,
    graph::{CrateGraph, CrateId, CrateName},
    hir::{
        CheckedContext, Context, FunctionNameMatch, LspMode, ParsedFiles,
        def_map::{CrateDefMap, parse_file},
    },
    node_interner::NodeInterner,
    parser::ParserError,
    usage_tracker::UsageTracker,
};
use rayon::prelude::*;
use rustc_hash::FxHashSet;

use notifications::{
    on_did_change_configuration, on_did_change_watched_files, on_did_close_text_document,
    on_did_open_text_document, on_did_save_text_document, on_initialized,
};
use requests::{
    LspInitializationOptions, WorkspaceSymbolCache, on_code_action_request, on_code_lens_request,
    on_completion_request, on_document_symbol_request, on_formatting, on_goto_declaration_request,
    on_goto_definition_request, on_goto_type_definition_request, on_hover_request, on_initialize,
    on_inlay_hint_request, on_prepare_rename_request, on_references_request, on_rename_request,
    on_signature_help_request, on_test_run_request, on_tests_request, on_workspace_symbol_request,
};
use serde_json::Value as JsonValue;
use thiserror::Error;
use tower::Service;

mod actor;
mod doc_comments;
mod modules;
mod notifications;
mod requests;
mod server_state_tests;
mod solver;
mod tests;
mod trait_impl_method_stub_generator;
mod types;
mod use_segment_positions;
mod utils;
mod visitor_reference_finder;
mod with_file;

#[cfg(test)]
mod test_utils;

use solver::WrapperSolver;
use types::{NargoTest, NargoTestId, Position, Range, Url, notification, request};
use with_file::parsed_module_with_file;

use crate::{
    actor::{ActorMessage, ActorResponse, CompilerActor},
    requests::{
        on_expand_request, on_folding_range_request, on_semantic_tokens_full_request,
        on_std_source_code_request,
    },
    types::request::{NargoExpand, NargoStdSourceCode},
};

#[derive(Debug, Error)]
pub enum LspError {
    /// Error while Resolving Workspace.
    #[error("Failed to Resolve Workspace - {0}")]
    WorkspaceResolutionError(String),
    /// Error while parsing Nargo.toml.
    #[error("{1}")]
    ManifestError(PathBuf, ManifestError),
}

// State for the LSP gets implemented on this struct and is internal to the implementation
pub struct LspState {
    root_path: Option<PathBuf>,
    client: ClientSocket,
    solver: WrapperSolver,
    input_files: HashMap<String, String>,
    cached_parsed_files: HashMap<PathBuf, (usize, (ParsedModule, Vec<ParserError>))>,
    workspace_cache: HashMap<PathBuf, WorkspaceCacheData>,
    /// The packages found under `root_path` (see [`discover_root_workspace`]), discovered on
    /// first use. `Some(None)` records that there is no usable root workspace.
    root_workspace: Option<Option<Workspace>>,
    workspace_symbol_cache: WorkspaceSymbolCache,

    options: LspInitializationOptions,

    // Tracks files that currently have errors, by workspace root.
    files_with_errors: HashMap<PathBuf, HashSet<Url>>,

    // Tracks Nargo.toml files that currently have diagnostics published to them.
    toml_files_with_errors: HashSet<Url>,
}

/// The analysis of a whole workspace. Every member package is a crate of one `CrateGraph`,
/// elaborated into one `NodeInterner`, so lookups that walk the interner (references, rename)
/// see every package of the workspace and not only the one a file belongs to.
///
/// The analysis is cached in parts so that single file changes can update them in place.
struct WorkspaceCacheData {
    file_manager: Arc<FileManager>,
    crate_graph: CrateGraph,
    node_interner: NodeInterner,
    def_maps: BTreeMap<CrateId, CrateDefMap>,
    usage_tracker: UsageTracker,
    /// The crate of each member package, keyed by the package's root directory.
    package_crate_ids: HashMap<PathBuf, CrateId>,
    /// Set when a single file change re-elaborates a crate that other crates of the analysis
    /// depend on. Those crates still point at the definitions the change replaced, so lookups
    /// across packages (references, rename) need the workspace checked again first.
    dependents_stale: bool,
}

impl LspState {
    pub(crate) fn new(
        client: &ClientSocket,
        solver: impl BlackBoxFunctionSolver<FieldElement> + 'static,
    ) -> Self {
        Self {
            client: client.clone(),
            root_path: None,
            solver: WrapperSolver(Box::new(solver)),
            input_files: HashMap::new(),
            cached_parsed_files: HashMap::new(),
            workspace_cache: HashMap::new(),
            root_workspace: None,
            workspace_symbol_cache: WorkspaceSymbolCache::default(),
            options: Default::default(),
            files_with_errors: HashMap::new(),
            toml_files_with_errors: HashSet::new(),
        }
    }
}

pub struct NargoLspService {
    router: Router<ServerState>,
}

/// State for the LSP main loop. Requests and notifications are forwarded as messages to
/// the compiler actor, which owns the [`LspState`], so handling a message here is always fast
/// no matter how much compiler work it triggers.
///
/// The main loop additionally mirrors the current text of every open document: it is the
/// entry point for all document changes, so its mirror is always up to date, even while the
/// actor is still busy type-checking older text. Requests that need only source text (like
/// formatting, which sits on the save path when format-on-save is enabled) are answered here
/// directly from the mirror instead of queueing behind compiler work.
struct ServerState {
    actor: CompilerActor,
    /// Latest text of every open document, keyed by its URI.
    input_files: HashMap<String, String>,
}

impl ServerState {
    fn open_document(&mut self, params: DidOpenTextDocumentParams) {
        self.input_files
            .insert(params.text_document.uri.to_string(), params.text_document.text.clone());
        notify_actor(&self.actor, on_did_open_text_document, params);
    }

    fn change_document(&mut self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.first() {
            self.input_files.insert(params.text_document.uri.to_string(), change.text.clone());
        }
        // Document changes get a dedicated message so the actor can coalesce consecutive
        // changes to the same document into the latest one.
        self.actor.send(ActorMessage::FileChanged(params));
    }

    fn close_document(&mut self, params: DidCloseTextDocumentParams) {
        self.input_files.remove(&params.text_document.uri.to_string());
        notify_actor(&self.actor, on_did_close_text_document, params);
    }

    fn format_document(
        &self,
        params: lsp_types::DocumentFormattingParams,
    ) -> impl Future<Output = Result<Option<Vec<lsp_types::TextEdit>>, ResponseError>> + use<> {
        std::future::ready(on_formatting(&self.input_files, params))
    }

    fn document_symbol(
        &self,
        params: lsp_types::DocumentSymbolParams,
    ) -> impl Future<Output = Result<Option<lsp_types::DocumentSymbolResponse>, ResponseError>> + use<>
    {
        std::future::ready(on_document_symbol_request(&self.input_files, params))
    }

    fn folding_range(
        &self,
        params: lsp_types::FoldingRangeParams,
    ) -> impl Future<Output = Result<Option<Vec<lsp_types::FoldingRange>>, ResponseError>> + use<>
    {
        std::future::ready(on_folding_range_request(&self.input_files, params))
    }
}

/// Adapts a request handler over `&mut LspState` into a main-loop handler that forwards the
/// request to the compiler actor and resolves with its reply.
fn forward_request<P, T>(
    handler: fn(&mut LspState, P) -> Result<T, ResponseError>,
) -> impl Fn(&mut ServerState, P) -> ActorResponse<T>
where
    P: Send + 'static,
    T: Send + 'static,
{
    move |state, params| state.actor.request(move |lsp_state| handler(lsp_state, params))
}

/// Adapts a notification handler over `&mut LspState` into a main-loop handler that forwards
/// the notification to the compiler actor.
///
/// The handler cannot influence the main loop (it runs on the actor thread after the
/// main-loop handler already returned `Continue`); an error it returns is only reported.
fn forward_notification<P>(
    handler: fn(&mut LspState, P) -> Result<(), Error>,
) -> impl Fn(&mut ServerState, P) -> ControlFlow<Result<(), Error>>
where
    P: Send + 'static,
{
    move |state, params| {
        notify_actor(&state.actor, handler, params);
        ControlFlow::Continue(())
    }
}

/// Enqueues a notification handler on the actor, reporting (instead of propagating) any
/// error it returns since the main loop has already moved on.
fn notify_actor<P>(
    actor: &CompilerActor,
    handler: fn(&mut LspState, P) -> Result<(), Error>,
    params: P,
) where
    P: Send + 'static,
{
    actor.notify(move |lsp_state| {
        if let Err(error) = handler(lsp_state, params) {
            eprintln!("error processing notification: {error}");
        }
    });
}

impl NargoLspService {
    pub fn new(
        client: &ClientSocket,
        solver: impl BlackBoxFunctionSolver<FieldElement> + Send + 'static,
    ) -> Self {
        let actor = CompilerActor::spawn(client.clone(), solver);
        let mut router = Router::new(ServerState { actor, input_files: HashMap::new() });
        router
            .request::<request::Initialize, _>(forward_request(on_initialize))
            .request::<request::Formatting, _>(|state: &mut ServerState, params| {
                state.format_document(params)
            })
            // `shutdown` (and `exit` below) are deliberate no-ops. The `LifecycleLayer`
            // wrapped around this service (see `nargo_cli`'s `lsp` command) implements the
            // protocol's lifecycle: after `shutdown` it rejects further requests, and on
            // `exit` it stops the main loop. That drops `ServerState`, whose `CompilerActor`
            // disconnects its channel on drop, ending the actor thread. Nothing is forwarded
            // to the actor because it holds no resources needing explicit cleanup; if that
            // ever changes, these handlers are the place to hook it up.
            .request::<request::Shutdown, _>(|_state: &mut ServerState, _params| {
                std::future::ready(Ok(()))
            })
            .request::<request::CodeLens, _>(forward_request(on_code_lens_request))
            .request::<request::NargoTests, _>(forward_request(on_tests_request))
            .request::<request::NargoTestRun, _>(forward_request(on_test_run_request))
            .request::<request::GotoDefinition, _>(forward_request(on_goto_definition_request))
            .request::<request::GotoDeclaration, _>(forward_request(on_goto_declaration_request))
            .request::<request::GotoTypeDefinition, _>(forward_request(
                on_goto_type_definition_request,
            ))
            .request::<SemanticTokensFullRequest, _>(forward_request(
                on_semantic_tokens_full_request,
            ))
            .request::<DocumentSymbolRequest, _>(|state: &mut ServerState, params| {
                state.document_symbol(params)
            })
            .request::<References, _>(forward_request(on_references_request))
            .request::<PrepareRenameRequest, _>(forward_request(on_prepare_rename_request))
            .request::<Rename, _>(forward_request(on_rename_request))
            .request::<HoverRequest, _>(forward_request(on_hover_request))
            .request::<InlayHintRequest, _>(forward_request(on_inlay_hint_request))
            .request::<Completion, _>(forward_request(on_completion_request))
            .request::<SignatureHelpRequest, _>(forward_request(on_signature_help_request))
            .request::<CodeActionRequest, _>(forward_request(on_code_action_request))
            .request::<WorkspaceSymbolRequest, _>(forward_request(on_workspace_symbol_request))
            .request::<FoldingRangeRequest, _>(|state: &mut ServerState, params| {
                state.folding_range(params)
            })
            .request::<NargoExpand, _>(forward_request(on_expand_request))
            .request::<NargoStdSourceCode, _>(forward_request(on_std_source_code_request))
            .notification::<notification::Initialized>(forward_notification(on_initialized))
            .notification::<notification::DidChangeConfiguration>(forward_notification(
                on_did_change_configuration,
            ))
            .notification::<notification::DidOpenTextDocument>(|state: &mut ServerState, params| {
                state.open_document(params);
                ControlFlow::Continue(())
            })
            .notification::<notification::DidChangeTextDocument>(
                |state: &mut ServerState, params| {
                    state.change_document(params);
                    ControlFlow::Continue(())
                },
            )
            .notification::<notification::DidCloseTextDocument>(
                |state: &mut ServerState, params| {
                    state.close_document(params);
                    ControlFlow::Continue(())
                },
            )
            .notification::<notification::DidSaveTextDocument>(forward_notification(
                on_did_save_text_document,
            ))
            .notification::<notification::DidChangeWatchedFiles>(forward_notification(
                on_did_change_watched_files,
            ))
            // No-op: see the `shutdown` handler above.
            .notification::<notification::Exit>(|_state: &mut ServerState, _params| {
                ControlFlow::Continue(())
            });
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
    context: &CheckedContext,
    crate_id: &CrateId,
    crate_name: &CrateName,
) -> Option<Vec<NargoTest>> {
    let fm = &context.file_manager;
    let files = fm.as_file_map();
    let tests =
        context.get_all_test_functions_in_crate_matching(crate_id, &FunctionNameMatch::Anything);

    let package_tests: Vec<_> = tests
        .into_iter()
        .map(|(func_name, test_function)| {
            let location = context.function_meta(&test_function.id).name.location;
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

    if package_tests.is_empty() { None } else { Some(package_tests) }
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

/// Packages under the editor's root are analyzed together up to this many. Past it, each
/// Nargo workspace is analyzed on its own.
const MAX_ROOT_PACKAGES: usize = 200;

/// Finds the set of packages to analyze together for a source file:
/// * every package found under the editor's root, when the file belongs to one of them and
///   root analysis is enabled
/// * otherwise the outermost Nargo workspace containing the file (see [`enclosing_workspace`])
/// * otherwise, without a `Nargo.toml`, the parent directory as a dummy workspace
pub(crate) fn resolve_workspace_for_source_path(
    state: &mut LspState,
    file_path: &Path,
) -> Result<Workspace, LspError> {
    let workspace = resolve_nargo_workspace_for_source_path(file_path)?;
    if workspace.is_assumed || !state.options.enable_root_analysis {
        return Ok(workspace);
    }

    let Some(package_dir) =
        workspace_package_for_file(&workspace, file_path).map(|package| &package.root_dir)
    else {
        return Ok(workspace);
    };
    let package_dir = canonical_or_same(package_dir);

    let Some(root_workspace) = root_workspace(state) else {
        return Ok(workspace);
    };
    let in_root = root_workspace
        .members
        .iter()
        .any(|member| canonical_or_same(&member.root_dir) == package_dir);
    Ok(if in_root { root_workspace.clone() } else { workspace })
}

fn root_workspace(state: &mut LspState) -> Option<&Workspace> {
    if state.root_workspace.is_none() {
        let root_path = state.root_path.clone()?;
        state.root_workspace = Some(discover_root_workspace(&root_path));
    }
    state.root_workspace.as_ref()?.as_ref()
}

/// Collects every package found under `root_path` into one workspace, the way rust-analyzer
/// loads every Cargo workspace under the opened folder. A package listed both by its own
/// manifest and by a workspace manifest is included once. Manifests that fail to resolve are
/// skipped; they get their diagnostics when a file of theirs is opened.
fn discover_root_workspace(root_path: &Path) -> Option<Workspace> {
    let mut members: Vec<Package> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();

    let walker = walkdir::WalkDir::new(root_path).into_iter().filter_entry(|entry| {
        if entry.depth() == 0 || !entry.file_type().is_dir() {
            return true;
        }
        let name = entry.file_name().to_string_lossy();
        name != "target" && name != "node_modules" && !name.starts_with('.')
    });
    for entry in walker.filter_map(Result::ok) {
        if !entry.file_type().is_file() || entry.file_name() != PKG_FILE {
            continue;
        }
        let Ok(workspace) = resolve_workspace_from_toml(
            entry.path(),
            PackageSelection::All,
            Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
        ) else {
            continue;
        };
        for package in workspace.members {
            if seen.insert(canonical_or_same(&package.root_dir)) {
                members.push(package);
            }
        }
    }

    if members.is_empty() || members.len() > MAX_ROOT_PACKAGES {
        return None;
    }
    Some(Workspace {
        root_dir: root_path.to_path_buf(),
        target_dir: None,
        members,
        selected_package_index: None,
        is_assumed: false,
    })
}

/// Create a workspace based on the source file location:
/// * if there is a `Nargo.toml` file, use it to read the workspace
/// * otherwise treat the parent directory as a dummy workspace
fn resolve_nargo_workspace_for_source_path(file_path: &Path) -> Result<Workspace, LspError> {
    if let Some(toml_path) = find_file_manifest(file_path) {
        return match resolve_workspace_from_toml(
            &toml_path,
            PackageSelection::All,
            Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
        ) {
            Ok(workspace) => Ok(enclosing_workspace(workspace, &toml_path)),
            Err(error) => Err(LspError::ManifestError(toml_path, error)),
        };
    }

    let Some(parent_folder) = file_path
        .parent()
        .and_then(|f| f.file_name())
        .and_then(|file_name_os_str| file_name_os_str.to_str())
    else {
        return Err(LspError::WorkspaceResolutionError(format!(
            "Could not resolve parent folder for file: {}",
            file_path.display()
        )));
    };

    let crate_name = match CrateName::from_str(parent_folder) {
        Ok(name) => name,
        Err(error) => {
            eprintln!("{error}");
            CrateName::from_str("root").unwrap()
        }
    };

    let assumed_package = Package {
        version: None,
        compiler_required_version: Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
        compiler_required_unstable_features: Vec::new(),
        root_dir: PathBuf::from(parent_folder),
        package_type: PackageType::Binary,
        entry_path: PathBuf::from(file_path),
        name: crate_name,
        dependencies: BTreeMap::new(),
    };
    let workspace = Workspace {
        root_dir: PathBuf::from(parent_folder),
        members: vec![assumed_package],
        selected_package_index: Some(0),
        is_assumed: true,
        target_dir: None,
    };
    Ok(workspace)
}

/// Widens `workspace`, resolved from the manifest at `toml_path`, to the outermost ancestor
/// workspace that lists it as a member. Analyzing that workspace is what lets a package see the
/// packages that depend on it.
fn enclosing_workspace(workspace: Workspace, toml_path: &Path) -> Workspace {
    let Some(package_dir) = toml_path.parent() else {
        return workspace;
    };
    let package_dir = canonical_or_same(package_dir);

    let mut workspace = workspace;
    for ancestor in package_dir.ancestors().skip(1) {
        let candidate_toml = ancestor.join(PKG_FILE);
        if !candidate_toml.is_file() {
            continue;
        }
        let Ok(candidate) = resolve_workspace_from_toml(
            &candidate_toml,
            PackageSelection::All,
            Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
        ) else {
            continue;
        };
        let is_member = candidate
            .members
            .iter()
            .any(|member| canonical_or_same(&member.root_dir) == package_dir);
        if is_member && canonical_or_same(&candidate.root_dir) != package_dir {
            workspace = candidate;
        }
    }
    workspace
}

fn canonical_or_same(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

pub(crate) fn workspace_package_for_file<'a>(
    workspace: &'a Workspace,
    file_path: &Path,
) -> Option<&'a Package> {
    if workspace.is_assumed {
        workspace.members.first()
    } else {
        // Packages can be nested, so the innermost one containing the file wins.
        workspace
            .members
            .iter()
            .filter(|package| file_path.starts_with(&package.root_dir))
            .max_by_key(|package| package.root_dir.as_os_str().len())
    }
}

/// The root crate of a workspace's analysis is an empty crate that depends on every member
/// package, so that checking it elaborates all of them into one context. Its file exists only
/// in the file manager.
fn workspace_root_file_path(workspace: &Workspace) -> PathBuf {
    workspace.root_dir.join("__nargo_lsp_workspace_root__.nr")
}

/// Adds the file of the workspace's root crate (see [`workspace_root_file_path`]) to a file
/// manager that already holds the workspace's sources, so that [`prepare_workspace`] finds it.
pub(crate) fn add_workspace_root_file(workspace: &Workspace, file_manager: &mut FileManager) {
    if !workspace.is_assumed {
        file_manager.add_file_with_source_canonical_path(
            &workspace_root_file_path(workspace),
            String::new(),
        );
    }
}

pub(crate) struct PreparedWorkspace {
    pub(crate) context: Context,
    /// The crate to check: checking it elaborates every member package.
    pub(crate) root_crate_id: CrateId,
    /// The crate of each member package, keyed by the package's root directory.
    pub(crate) package_crate_ids: HashMap<PathBuf, CrateId>,
}

/// Builds the crate graph of a whole workspace in one context. Member packages that depend on
/// each other share a single crate, as the crate graph identifies crates by their root file.
pub(crate) fn prepare_workspace(
    file_manager: &Arc<FileManager>,
    parsed_files: &Arc<ParsedFiles>,
    workspace: &Workspace,
) -> PreparedWorkspace {
    let mut package_crate_ids = HashMap::new();

    let (mut context, root_crate_id) = if workspace.is_assumed {
        // An assumed workspace has a single package and no manifest, so there is nothing to share.
        let package = workspace.members.first().expect("an assumed workspace has one package");
        let (context, crate_id) = nargo::prepare_package(file_manager, parsed_files, package);
        package_crate_ids.insert(package.root_dir.clone(), crate_id);
        (context, crate_id)
    } else {
        let mut context = Context::new(Arc::clone(file_manager), Arc::clone(parsed_files));
        let root_crate_id = prepare_crate(&mut context, &workspace_root_file_path(workspace));
        for package in &workspace.members {
            let crate_id = prepare_dependency(&mut context, &package.entry_path);
            context
                .required_unstable_features
                .insert(crate_id, package.compiler_required_unstable_features.clone());
            add_dep(&mut context, root_crate_id, crate_id, package.name.clone());
            nargo::prepare_dependencies(&mut context, crate_id, &package.dependencies);
            package_crate_ids.insert(package.root_dir.clone(), crate_id);
        }
        (context, root_crate_id)
    };

    context.activate_lsp_mode(LspMode::Full);
    PreparedWorkspace { context, root_crate_id, package_crate_ids }
}

pub(crate) struct CheckedWorkspace {
    pub(crate) context: CheckedContext,
    pub(crate) file_manager: Arc<FileManager>,
    /// The crate of each member package, keyed by the package's root directory.
    pub(crate) package_crate_ids: HashMap<PathBuf, CrateId>,
    /// Errors from every crate, and warnings from member packages only: an error in a
    /// dependency breaks the packages that use it, but its warnings are not the user's to fix.
    pub(crate) diagnostics: Vec<CustomDiagnostic>,
}

/// Type-checks every member package of `workspace` in one context, reading open documents
/// from their editor contents.
pub(crate) fn check_workspace(state: &mut LspState, workspace: &Workspace) -> CheckedWorkspace {
    let mut file_manager = workspace.new_file_manager();
    if workspace.is_assumed {
        let package = workspace.members.first().unwrap();
        file_manager.add_file_with_source_canonical_path(&package.entry_path, String::new());
    } else {
        insert_all_files_for_workspace_into_file_manager(state, workspace, &mut file_manager);
    }
    add_workspace_root_file(workspace, &mut file_manager);

    let parsed_files = Arc::new(parse_diff(&file_manager, state));
    let file_manager = Arc::new(file_manager);

    let prepared = prepare_workspace(&file_manager, &parsed_files, workspace);
    let member_crates: Vec<CrateId> = prepared.package_crate_ids.values().copied().collect();
    let options = CompileOptions::default();
    let (context, result) = check_crate_keeping_warnings_from(
        prepared.context,
        prepared.root_crate_id,
        &member_crates,
        &options,
        &mut options.comptime_io(),
    );
    let diagnostics = match result {
        Ok(((), diagnostics)) | Err(diagnostics) => diagnostics,
    };

    CheckedWorkspace {
        context,
        file_manager,
        package_crate_ids: prepared.package_crate_ids,
        diagnostics,
    }
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
                        rustc_hash::FxBuildHasher
                            .hash_one(file_manager.fetch_file(file_id).expect("file must exist"))
                            as usize,
                    ))
                } else {
                    None
                }
            })
            .collect();

        let cache_hits = noir_file_hashes
            .par_iter()
            .filter_map(|(file_id, file_path, current_hash)| {
                let cached_version = state.cached_parsed_files.get(file_path);
                if let Some((hash, (parsed_module, errors))) = cached_version
                    && hash == current_hash
                {
                    // The cached ParsedModule might have FileIDs in it that are different than the file_id we get here,
                    // so we must replace all of those FileIDs with the one here.
                    let parsed_module = parsed_module_with_file(parsed_module.clone(), *file_id);
                    return Some((*file_id, (parsed_module, errors.clone())));
                }
                None
            })
            .collect::<Vec<_>>();

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
    let overrides = source_code_overrides(&state.input_files);
    nargo::insert_all_files_for_workspace_into_file_manager_with_overrides(
        workspace,
        file_manager,
        Some(&overrides),
    );
}

// Source code for files we cached override those that are read from disk.
pub fn source_code_overrides(input_files: &HashMap<String, String>) -> HashMap<PathBuf, &str> {
    let mut overrides: HashMap<PathBuf, &str> = HashMap::new();
    for (path, source) in input_files {
        if let Some(path) = path.strip_prefix("file://") {
            overrides.insert(PathBuf::from_str(path).unwrap(), source);
        }
    }
    overrides
}
