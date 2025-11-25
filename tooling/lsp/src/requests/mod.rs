use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{collections::HashMap, future::Future};

use crate::notifications::fake_stdlib_workspace;
use crate::notifications::process_workspace;
use crate::{PackageCacheData, insert_all_files_for_workspace_into_file_manager, parse_diff};
use crate::{
    resolve_workspace_for_source_path,
    types::{CodeLensOptions, InitializeParams},
};
use async_lsp::{ErrorCode, ResponseError};
use fm::FileId;
use fm::{FileMap, PathString, codespan_files::Error};
use lsp_types::{
    CodeActionKind, DeclarationCapability, Location, Position, TextDocumentPositionParams,
    TextDocumentSyncCapability, TextDocumentSyncKind, TypeDefinitionProviderCapability, Url,
    WorkDoneProgressOptions,
};
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo_fmt::Config;

use noirc_frontend::ast::Ident;
use noirc_frontend::graph::{CrateGraph, CrateId};
use noirc_frontend::hir::def_map::{CrateDefMap, ModuleId};
use noirc_frontend::node_interner::ReferenceId;
use noirc_frontend::parser::ParserError;
use noirc_frontend::usage_tracker::UsageTracker;
use noirc_frontend::{graph::Dependency, node_interner::NodeInterner};
use serde::{Deserialize, Serialize};

use async_lsp::lsp_types::{
    self, SemanticTokenType, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
};

use crate::{
    LspState,
    types::{InitializeResult, NargoCapability, NargoTestsOptions, ServerCapabilities},
};

pub(crate) use workspace_symbol::WorkspaceSymbolCache;

// Handlers
// The handlers for `request` are not `async` because it compiles down to lifetimes that can't be added to
// the router. To return a future that fits the trait, it is easiest wrap your implementations in an `async {}`
// block but you can also use `std::future::ready`.
//
// Additionally, the handlers for `notification` aren't async at all.
//
// They are not attached to the `NargoLspService` struct so they can be unit tested with only `LspState`
// and params passed in.

mod code_action;
mod code_lens_request;
mod completion;
mod document_symbol;
mod expand;
mod folding_range;
mod goto_declaration;
mod goto_definition;
mod hover;
mod inlay_hint;
mod references;
mod rename;
mod semantic_tokens;
mod signature_help;
mod std_source_code;
mod test_run;
mod tests;
mod workspace_symbol;

pub(crate) use {
    code_action::on_code_action_request, code_lens_request::on_code_lens_request,
    completion::on_completion_request, document_symbol::on_document_symbol_request,
    expand::on_expand_request, folding_range::on_folding_range_request,
    goto_declaration::on_goto_declaration_request, goto_definition::on_goto_definition_request,
    goto_definition::on_goto_type_definition_request, hover::on_hover_request,
    inlay_hint::on_inlay_hint_request, references::on_references_request,
    rename::on_prepare_rename_request, rename::on_rename_request,
    semantic_tokens::on_semantic_tokens_full_request, signature_help::on_signature_help_request,
    std_source_code::on_std_source_code_request, test_run::on_test_run_request,
    tests::on_tests_request, workspace_symbol::on_workspace_symbol_request,
};

/// LSP client will send initialization request after the server has started.
/// [InitializeParams].`initialization_options` will contain the options sent from the client.
#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub(crate) struct LspInitializationOptions {
    /// Controls whether code lens is enabled by the server
    /// By default this will be set to true (enabled).
    #[serde(rename = "enableCodeLens", default = "default_enable_code_lens")]
    pub(crate) enable_code_lens: bool,

    #[serde(rename = "enableInlayHints", default = "default_enable_inlay_hints")]
    pub(crate) enable_inlay_hints: bool,

    #[serde(rename = "enableCompletions", default = "default_enable_completions")]
    pub(crate) enable_completions: bool,

    #[serde(rename = "enableSignatureHelp", default = "default_enable_signature_help")]
    pub(crate) enable_signature_help: bool,

    #[serde(rename = "enableCodeActions", default = "default_enable_code_actions")]
    pub(crate) enable_code_actions: bool,

    #[serde(rename = "enableSemanticTokens", default = "default_enable_semantic_tokens")]
    pub(crate) enable_semantic_tokens: bool,

    /// Lightweight mode disables code lens, inlay hints, completions, signature help and code actions
    #[serde(rename = "enableLightweightMode", default = "default_enable_lightweight_mode")]
    pub(crate) enable_lightweight_mode: bool,

    #[serde(rename = "enableParsingCache", default = "default_enable_parsing_cache")]
    pub(crate) enable_parsing_cache: bool,

    #[serde(rename = "inlayHints", default = "default_inlay_hints")]
    pub(crate) inlay_hints: InlayHintsOptions,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub(crate) struct InlayHintsOptions {
    #[serde(rename = "typeHints", default = "default_type_hints")]
    pub(crate) type_hints: TypeHintsOptions,

    #[serde(rename = "parameterHints", default = "default_parameter_hints")]
    pub(crate) parameter_hints: ParameterHintsOptions,

    #[serde(rename = "closingBraceHints", default = "default_closing_brace_hints")]
    pub(crate) closing_brace_hints: ClosingBraceHintsOptions,

    #[serde(rename = "chainingHints", default = "default_chaining_hints")]
    pub(crate) chaining_hints: ChainingHintsOptions,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub(crate) struct TypeHintsOptions {
    #[serde(rename = "enabled", default = "default_type_hints_enabled")]
    pub(crate) enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub(crate) struct ParameterHintsOptions {
    #[serde(rename = "enabled", default = "default_parameter_hints_enabled")]
    pub(crate) enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub(crate) struct ClosingBraceHintsOptions {
    #[serde(rename = "enabled", default = "default_closing_brace_hints_enabled")]
    pub(crate) enabled: bool,

    #[serde(rename = "minLines", default = "default_closing_brace_min_lines")]
    pub(crate) min_lines: u32,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub(crate) struct ChainingHintsOptions {
    #[serde(rename = "enabled", default = "default_chaining_hints_enabled")]
    pub(crate) enabled: bool,
}

fn default_enable_code_lens() -> bool {
    true
}

fn default_enable_inlay_hints() -> bool {
    true
}

fn default_enable_completions() -> bool {
    true
}

fn default_enable_signature_help() -> bool {
    true
}

fn default_enable_code_actions() -> bool {
    true
}

fn default_enable_semantic_tokens() -> bool {
    true
}

fn default_enable_parsing_cache() -> bool {
    true
}

fn default_enable_lightweight_mode() -> bool {
    false
}

fn default_inlay_hints() -> InlayHintsOptions {
    InlayHintsOptions {
        type_hints: default_type_hints(),
        parameter_hints: default_parameter_hints(),
        closing_brace_hints: default_closing_brace_hints(),
        chaining_hints: default_chaining_hints(),
    }
}

fn default_type_hints() -> TypeHintsOptions {
    TypeHintsOptions { enabled: default_type_hints_enabled() }
}

fn default_type_hints_enabled() -> bool {
    true
}

fn default_parameter_hints() -> ParameterHintsOptions {
    ParameterHintsOptions { enabled: default_parameter_hints_enabled() }
}

fn default_parameter_hints_enabled() -> bool {
    true
}

fn default_closing_brace_hints() -> ClosingBraceHintsOptions {
    ClosingBraceHintsOptions {
        enabled: default_closing_brace_hints_enabled(),
        min_lines: default_closing_brace_min_lines(),
    }
}

fn default_closing_brace_hints_enabled() -> bool {
    true
}

fn default_closing_brace_min_lines() -> u32 {
    25
}

fn default_chaining_hints() -> ChainingHintsOptions {
    ChainingHintsOptions { enabled: default_chaining_hints_enabled() }
}

fn default_chaining_hints_enabled() -> bool {
    true
}

impl Default for LspInitializationOptions {
    fn default() -> Self {
        Self {
            enable_code_lens: default_enable_code_lens(),
            enable_parsing_cache: default_enable_parsing_cache(),
            inlay_hints: default_inlay_hints(),
            enable_inlay_hints: default_enable_inlay_hints(),
            enable_completions: default_enable_completions(),
            enable_signature_help: default_enable_signature_help(),
            enable_code_actions: default_enable_code_actions(),
            enable_semantic_tokens: default_enable_semantic_tokens(),
            enable_lightweight_mode: default_enable_lightweight_mode(),
        }
    }
}

#[expect(deprecated)]
pub(crate) fn on_initialize(
    state: &mut LspState,
    params: InitializeParams,
) -> impl Future<Output = Result<InitializeResult, ResponseError>> + use<> {
    state.root_path = params.root_uri.and_then(|root_uri| root_uri.to_file_path().ok());
    let initialization_options: LspInitializationOptions = params
        .initialization_options
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    state.options = initialization_options;

    let enable_code_lens =
        !initialization_options.enable_lightweight_mode && initialization_options.enable_code_lens;
    let enable_inlay_hints = !initialization_options.enable_lightweight_mode
        && initialization_options.enable_inlay_hints;
    let enable_completions = !initialization_options.enable_lightweight_mode
        && initialization_options.enable_completions;
    let enable_signature_help = !initialization_options.enable_lightweight_mode
        && initialization_options.enable_signature_help;
    let enable_code_actions = !initialization_options.enable_lightweight_mode
        && initialization_options.enable_code_actions;
    let enable_semantic_tokens = !initialization_options.enable_lightweight_mode
        && initialization_options.enable_semantic_tokens;

    async move {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_lens_provider: enable_code_lens
                    .then_some(CodeLensOptions { resolve_provider: Some(false) }),
                document_formatting_provider: true,
                nargo: Some(NargoCapability {
                    tests: Some(NargoTestsOptions {
                        fetch: Some(true),
                        run: Some(true),
                        update: Some(true),
                    }),
                }),
                definition_provider: Some(lsp_types::OneOf::Left(true)),
                declaration_provider: Some(DeclarationCapability::Simple(true)),
                type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
                rename_provider: Some(lsp_types::OneOf::Right(lsp_types::RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                })),
                references_provider: Some(lsp_types::OneOf::Right(lsp_types::ReferencesOptions {
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                })),
                hover_provider: Some(lsp_types::OneOf::Right(lsp_types::HoverOptions {
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                })),
                inlay_hint_provider: enable_inlay_hints.then_some(lsp_types::OneOf::Right(
                    lsp_types::InlayHintOptions {
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                        resolve_provider: None,
                    },
                )),
                document_symbol_provider: Some(lsp_types::OneOf::Right(
                    lsp_types::DocumentSymbolOptions {
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                        label: Some("Noir".to_string()),
                    },
                )),
                completion_provider: enable_completions.then_some(lsp_types::OneOf::Right(
                    lsp_types::CompletionOptions {
                        resolve_provider: None,
                        trigger_characters: Some(vec![
                            ".".to_string(), // For method calls
                            ":".to_string(), // For paths
                            "$".to_string(), // For $var inside `quote { ... }`
                        ]),
                        all_commit_characters: None,
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                        completion_item: None,
                    },
                )),
                signature_help_provider: enable_signature_help.then_some(lsp_types::OneOf::Right(
                    lsp_types::SignatureHelpOptions {
                        trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                        retrigger_characters: None,
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
                code_action_provider: enable_code_actions.then_some(lsp_types::OneOf::Right(
                    lsp_types::CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                        resolve_provider: None,
                    },
                )),
                workspace_symbol_provider: Some(lsp_types::OneOf::Right(
                    lsp_types::WorkspaceSymbolOptions {
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                        resolve_provider: None,
                    },
                )),
                folding_range_provider: Some(true),
                semantic_tokens_provider: enable_semantic_tokens.then_some(lsp_types::OneOf::Left(
                    SemanticTokensOptions {
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                        legend: SemanticTokensLegend {
                            token_types: semantic_token_types().to_vec(),
                            token_modifiers: vec![],
                        },
                        range: None,
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                    },
                )),
            },
            server_info: None,
        })
    }
}

pub(crate) fn semantic_token_types() -> [SemanticTokenType; 12] {
    [
        SemanticTokenType::NAMESPACE,
        SemanticTokenType::STRUCT,
        SemanticTokenType::INTERFACE,
        SemanticTokenType::ENUM,
        SemanticTokenType::FUNCTION,
        SemanticTokenType::METHOD,
        SemanticTokenType::VARIABLE,
        SemanticTokenType::PROPERTY,
        SemanticTokenType::NUMBER,
        SemanticTokenType::KEYWORD,
        SemanticTokenType::STRING,
        SemanticTokenType::OPERATOR,
    ]
}

pub(crate) fn semantic_token_types_map() -> HashMap<SemanticTokenType, usize> {
    semantic_token_types().iter().enumerate().map(|(i, typ)| (typ.clone(), i)).collect()
}

pub(crate) fn on_formatting(
    state: &mut LspState,
    params: lsp_types::DocumentFormattingParams,
) -> impl Future<Output = Result<Option<Vec<lsp_types::TextEdit>>, ResponseError>> + use<> {
    std::future::ready(on_formatting_inner(state, params))
}

fn on_formatting_inner(
    state: &LspState,
    params: lsp_types::DocumentFormattingParams,
) -> Result<Option<Vec<lsp_types::TextEdit>>, ResponseError> {
    // The file_path might be Err/None if the action runs against an unsaved file
    let file_path = params.text_document.uri.to_file_path().ok();
    let directory_path = file_path.as_ref().and_then(|path| path.parent());

    let path = params.text_document.uri.to_string();

    if let Some(source) = state.input_files.get(&path) {
        let (module, errors) = noirc_frontend::parse_program_with_dummy_file(source);
        let is_all_warnings = errors.iter().all(ParserError::is_warning);
        if !is_all_warnings {
            return Ok(None);
        }

        let config = read_format_config(directory_path);
        let new_text = nargo_fmt::format(source, module, &config);

        let start_position = Position { line: 0, character: 0 };
        let end_position = Position {
            line: source.lines().count() as u32,
            character: source.chars().count() as u32,
        };

        Ok(Some(vec![lsp_types::TextEdit {
            range: lsp_types::Range::new(start_position, end_position),
            new_text,
        }]))
    } else {
        Ok(None)
    }
}

fn read_format_config(file_path: Option<&Path>) -> Config {
    match file_path {
        Some(file_path) => match Config::read(file_path) {
            Ok(config) => config,
            Err(err) => {
                eprintln!("{err}");
                Config::default()
            }
        },
        None => Config::default(),
    }
}

pub(crate) fn position_to_byte_index<'a, F>(
    files: &'a F,
    file_id: F::FileId,
    position: &Position,
) -> Result<usize, Error>
where
    F: fm::codespan_files::Files<'a> + ?Sized,
{
    let source = files.source(file_id)?;
    let source = source.as_ref();

    let line_span = files.line_range(file_id, position.line as usize)?;

    let line_str = source.get(line_span.clone());

    if let Some(line_str) = line_str {
        let byte_offset = character_to_line_offset(line_str, position.character)?;
        Ok(line_span.start + byte_offset)
    } else {
        Err(Error::InvalidCharBoundary { given: position.line as usize })
    }
}

fn position_to_location(
    files: &FileMap,
    file_path: &PathString,
    position: &Position,
) -> Result<noirc_errors::Location, ResponseError> {
    let file_id = files.get_file_id(file_path).ok_or(ResponseError::new(
        ErrorCode::REQUEST_FAILED,
        format!("Could not find file in file manager. File path: {file_path:?}"),
    ))?;
    let byte_index = position_to_byte_index(files, file_id, position).map_err(|err| {
        ResponseError::new(
            ErrorCode::REQUEST_FAILED,
            format!("Could not convert position to byte index. Error: {err:?}"),
        )
    })?;

    let location = noirc_errors::Location {
        file: file_id,
        span: noirc_errors::Span::single_char(byte_index as u32),
    };

    Ok(location)
}

fn character_to_line_offset(line: &str, character: u32) -> Result<usize, Error> {
    let line_len = line.len();
    let mut character_offset = 0;

    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if character_offset == character {
            let chars_off = chars.as_str().len();
            let ch_off = ch.len_utf8();

            return Ok(line_len - chars_off - ch_off);
        }

        character_offset += ch.len_utf16() as u32;
    }

    // Handle positions after the last character on the line
    if character_offset == character {
        Ok(line_len)
    } else {
        Err(Error::ColumnTooLarge { given: character_offset as usize, max: line.len() })
    }
}

pub(crate) fn to_lsp_location(
    files: &FileMap,
    file_id: FileId,
    definition_span: noirc_errors::Span,
) -> Option<Location> {
    let range = crate::byte_span_to_range(files, file_id, definition_span.into())?;
    let file_name = files.get_absolute_name(file_id).ok()?;
    let path = file_name.to_string();
    if let Ok(uri) = Url::from_file_path(path.clone()) {
        Some(Location { uri, range })
    } else if path.starts_with("std/") {
        Some(Location { uri: Url::from_str(&format!("noir-std://{path}")).unwrap(), range })
    } else {
        None
    }
}

pub(crate) fn on_shutdown(
    _state: &mut LspState,
    _params: (),
) -> impl Future<Output = Result<(), ResponseError>> + use<> {
    async { Ok(()) }
}

pub(crate) struct ProcessRequestCallbackArgs<'a> {
    pub(crate) location: noirc_errors::Location,
    pub(crate) workspace: &'a Workspace,
    pub(crate) package: &'a Package,
    pub(crate) files: &'a FileMap,
    pub(crate) interner: &'a NodeInterner,
    pub(crate) package_cache: &'a HashMap<PathBuf, PackageCacheData>,
    pub(crate) crate_id: CrateId,
    pub(crate) crate_name: String,
    pub(crate) crate_graph: &'a CrateGraph,
    pub(crate) def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
    pub(crate) usage_tracker: &'a UsageTracker,
}

impl<'a> ProcessRequestCallbackArgs<'a> {
    pub(crate) fn dependencies(&self) -> &'a Vec<Dependency> {
        &self.crate_graph[self.crate_id].dependencies
    }
}

pub(crate) fn process_request<F, T>(
    state: &mut LspState,
    text_document_position_params: TextDocumentPositionParams,
    callback: F,
) -> Result<T, ResponseError>
where
    F: FnOnce(ProcessRequestCallbackArgs) -> T,
{
    let type_check = true;
    process_request_impl(state, text_document_position_params, type_check, callback)
}

pub(crate) fn process_request_no_type_check<F, T>(
    state: &mut LspState,
    text_document_position_params: TextDocumentPositionParams,
    callback: F,
) -> Result<T, ResponseError>
where
    F: FnOnce(ProcessRequestCallbackArgs) -> T,
{
    let type_check = false;
    process_request_impl(state, text_document_position_params, type_check, callback)
}

fn process_request_impl<F, T>(
    state: &mut LspState,
    text_document_position_params: TextDocumentPositionParams,
    type_check: bool,
    callback: F,
) -> Result<T, ResponseError>
where
    F: FnOnce(ProcessRequestCallbackArgs) -> T,
{
    let uri = text_document_position_params.text_document.uri.clone();

    let (file_path, workspace) = if uri.scheme() == "noir-std" {
        let workspace = fake_stdlib_workspace();
        let file_path =
            PathBuf::from_str(&format!("{}{}", uri.host().unwrap(), uri.path())).unwrap();
        (file_path, workspace)
    } else {
        let file_path = uri.to_file_path().map_err(|_| {
            ResponseError::new(ErrorCode::REQUEST_FAILED, "URI is not a valid file path")
        })?;

        let workspace = resolve_workspace_for_source_path(file_path.as_path()).unwrap();
        (file_path, workspace)
    };

    // First type-check the workspace if needed
    if type_check && state.workspaces_to_process.remove(&workspace.root_dir) {
        let _ = process_workspace(state, &workspace, false);
    }

    let package = crate::workspace_package_for_file(&workspace, &file_path).ok_or_else(|| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "Could not find package for file")
    })?;

    // In practice when `process_request` is called, a document in the project should already have been
    // open so both the workspace and package cache will have data. However, just in case this isn't true
    // for some reason, and also for tests (some tests just test a request without going through the full
    // LSP workflow), we have a fallback where we type-check the workspace/package, then continue with
    // processing the request.
    let Some(workspace_cache_data) = state.workspace_cache.get(&workspace.root_dir) else {
        return process_request_no_workspace_cache(state, text_document_position_params, callback);
    };

    let Some(package_cache_data) = state.package_cache.get(&package.root_dir) else {
        return process_request_no_workspace_cache(state, text_document_position_params, callback);
    };

    let file_manager = &workspace_cache_data.file_manager;
    let interner = &package_cache_data.node_interner;
    let def_maps = &package_cache_data.def_maps;
    let usage_tracker = &package_cache_data.usage_tracker;
    let crate_graph = &package_cache_data.crate_graph;
    let crate_id = package_cache_data.crate_id;

    let files = file_manager.as_file_map();

    let location = position_to_location(
        files,
        &PathString::from(file_path),
        &text_document_position_params.position,
    )?;

    Ok(callback(ProcessRequestCallbackArgs {
        location,
        workspace: &workspace,
        package,
        files,
        interner,
        package_cache: &state.package_cache,
        crate_id,
        crate_name: package.name.to_string(),
        crate_graph,
        def_maps,
        usage_tracker,
    }))
}

pub(crate) fn process_request_no_workspace_cache<F, T>(
    state: &mut LspState,
    text_document_position_params: TextDocumentPositionParams,
    callback: F,
) -> Result<T, ResponseError>
where
    F: FnOnce(ProcessRequestCallbackArgs) -> T,
{
    let file_path =
        text_document_position_params.text_document.uri.to_file_path().map_err(|_| {
            ResponseError::new(ErrorCode::REQUEST_FAILED, "URI is not a valid file path")
        })?;

    let workspace = resolve_workspace_for_source_path(file_path.as_path()).unwrap();
    let package = crate::workspace_package_for_file(&workspace, &file_path).ok_or_else(|| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "Could not find package for file")
    })?;

    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(
        state,
        &workspace,
        &mut workspace_file_manager,
    );
    let parsed_files = parse_diff(&workspace_file_manager, state);

    let (mut context, crate_id) =
        crate::prepare_package(&workspace_file_manager, &parsed_files, package);

    let interner;
    let def_maps;
    let usage_tracker;
    if let Some(package_cache) = state.package_cache.get(&package.root_dir) {
        interner = &package_cache.node_interner;
        def_maps = &package_cache.def_maps;
        usage_tracker = &package_cache.usage_tracker;
    } else {
        // We ignore the warnings and errors produced by compilation while resolving the definition
        let _ = noirc_driver::check_crate(&mut context, crate_id, &Default::default());
        interner = &context.def_interner;
        def_maps = &context.def_maps;
        usage_tracker = &context.usage_tracker;
    }

    let files = workspace_file_manager.as_file_map();

    let location = position_to_location(
        files,
        &PathString::from(file_path),
        &text_document_position_params.position,
    )?;

    Ok(callback(ProcessRequestCallbackArgs {
        location,
        workspace: &workspace,
        package,
        files,
        interner,
        package_cache: &state.package_cache,
        crate_id,
        crate_name: package.name.to_string(),
        crate_graph: &context.crate_graph,
        def_maps,
        usage_tracker,
    }))
}

pub(crate) fn find_all_references_in_workspace(
    location: noirc_errors::Location,
    interner: &NodeInterner,
    package_cache: &HashMap<PathBuf, PackageCacheData>,
    files: &FileMap,
    include_declaration: bool,
    include_self_type_name: bool,
) -> Option<Vec<Location>> {
    // First find the node that's referenced by the given location, if any
    let referenced = interner.find_referenced(location)?;
    let name = get_reference_name(referenced, interner)?;

    // If we found the referenced node, find its location
    let referenced_location = interner.reference_location(referenced);

    // Now we find all references that point to this location, in all interners
    // (there's one interner per package, and all interners in a workspace rely on the
    // same FileManager so a Location/FileId in one package is the same as in another package)
    let mut locations = find_all_references(
        referenced_location,
        interner,
        include_declaration,
        include_self_type_name,
    );
    for cache_data in package_cache.values() {
        locations.extend(find_all_references(
            referenced_location,
            &cache_data.node_interner,
            include_declaration,
            include_self_type_name,
        ));
    }

    // Only keep locations whose span, when read from the file, matches "name"
    // (it might not match because of macro expansions)
    locations.retain(|location| {
        let Some(file) = files.get_file(location.file) else {
            return false;
        };

        let Some(substring) =
            file.source().get(location.span.start() as usize..location.span.end() as usize)
        else {
            return false;
        };

        substring == name
    });

    let mut locations = locations
        .iter()
        .filter_map(|location| to_lsp_location(files, location.file, location.span))
        .collect::<Vec<_>>();

    // The LSP client usually removes duplicate locations, but we do it here just in case they don't
    locations.sort_by_key(|location| {
        (
            location.uri.to_string(),
            location.range.start.line,
            location.range.start.character,
            location.range.end.line,
            location.range.end.character,
        )
    });
    locations.dedup();

    if locations.is_empty() { None } else { Some(locations) }
}

pub(crate) fn find_all_references(
    referenced_location: noirc_errors::Location,
    interner: &NodeInterner,
    include_declaration: bool,
    include_self_type_name: bool,
) -> Vec<noirc_errors::Location> {
    interner
        .find_all_references(referenced_location, include_declaration, include_self_type_name)
        .unwrap_or_default()
}

fn get_reference_name(reference: ReferenceId, interner: &NodeInterner) -> Option<String> {
    match reference {
        ReferenceId::Module(module_id) => {
            Some(interner.try_module_attributes(module_id)?.name.clone())
        }
        ReferenceId::Type(type_id) => Some(interner.get_type(type_id).borrow().name.to_string()),
        ReferenceId::StructMember(type_id, index) => {
            Some(interner.get_type(type_id).borrow().field_at(index).name.to_string())
        }
        ReferenceId::EnumVariant(type_id, index) => {
            Some(interner.get_type(type_id).borrow().variant_at(index).name.to_string())
        }
        ReferenceId::Trait(trait_id) => Some(interner.get_trait(trait_id).name.to_string()),
        ReferenceId::TraitAssociatedType(id) => {
            Some(interner.get_trait_associated_type(id).name.to_string())
        }
        ReferenceId::Global(global_id) => Some(interner.get_global(global_id).ident.to_string()),
        ReferenceId::Function(func_id) => Some(interner.function_name(&func_id).to_string()),
        ReferenceId::Alias(type_alias_id) => {
            Some(interner.get_type_alias(type_alias_id).borrow().name.to_string())
        }
        ReferenceId::Local(definition_id) => {
            Some(interner.definition_name(definition_id).to_string())
        }
        ReferenceId::Reference(location, _) => {
            get_reference_name(interner.find_referenced(location)?, interner)
        }
    }
}

/// Represents a trait reexported from a given module with a name.
pub(crate) struct TraitReexport {
    pub(super) module_id: ModuleId,
    pub(super) name: Ident,
}

#[cfg(test)]
mod initialization {
    use acvm::blackbox_solver::StubbedBlackBoxSolver;
    use async_lsp::ClientSocket;
    use async_lsp::lsp_types::{
        CodeLensOptions, InitializeParams, TextDocumentSyncCapability, TextDocumentSyncKind,
    };
    use tokio::test;

    use crate::{LspState, requests::on_initialize, types::ServerCapabilities};

    #[test]
    async fn test_on_initialize() {
        let client = ClientSocket::new_closed();
        let mut state = LspState::new(&client, StubbedBlackBoxSolver::default());
        let params = InitializeParams::default();
        let response = on_initialize(&mut state, params).await.unwrap();
        assert!(matches!(
            response.capabilities,
            ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL
                )),
                code_lens_provider: Some(CodeLensOptions { resolve_provider: Some(false) }),
                document_formatting_provider: true,
                ..
            }
        ));
        assert!(response.server_info.is_none());
    }
}

#[cfg(test)]
mod character_to_line_offset_tests {
    use super::*;

    #[test]
    fn test_character_to_line_offset() {
        let line = "Hello, dark!";
        let character = 8;

        let result = character_to_line_offset(line, character).unwrap();
        assert_eq!(result, 8);

        // In the case of a multi-byte character, the offset should be the byte index of the character
        // byte offset for 8 character (黑) is expected to be 10
        let line = "Hello, 黑!";
        let character = 8;

        let result = character_to_line_offset(line, character).unwrap();
        assert_eq!(result, 10);
    }
}
