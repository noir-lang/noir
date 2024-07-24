use std::{collections::HashMap, future::Future};

use crate::insert_all_files_for_workspace_into_file_manager;
use crate::{
    parse_diff, resolve_workspace_for_source_path,
    types::{CodeLensOptions, InitializeParams},
};
use async_lsp::{ErrorCode, ResponseError};
use fm::{codespan_files::Error, FileMap, PathString};
use lsp_types::{
    DeclarationCapability, Location, Position, TextDocumentPositionParams,
    TextDocumentSyncCapability, TextDocumentSyncKind, TypeDefinitionProviderCapability, Url,
    WorkDoneProgressOptions,
};
use nargo_fmt::Config;
use noirc_driver::file_manager_with_stdlib;
use noirc_frontend::{graph::Dependency, macros_api::NodeInterner};
use serde::{Deserialize, Serialize};

use crate::{
    types::{InitializeResult, NargoCapability, NargoTestsOptions, ServerCapabilities},
    LspState,
};

// Handlers
// The handlers for `request` are not `async` because it compiles down to lifetimes that can't be added to
// the router. To return a future that fits the trait, it is easiest wrap your implementations in an `async {}`
// block but you can also use `std::future::ready`.
//
// Additionally, the handlers for `notification` aren't async at all.
//
// They are not attached to the `NargoLspService` struct so they can be unit tested with only `LspState`
// and params passed in.

mod code_lens_request;
mod document_symbol;
mod goto_declaration;
mod goto_definition;
mod hover;
mod inlay_hint;
mod profile_run;
mod references;
mod rename;
mod test_run;
mod tests;

pub(crate) use {
    code_lens_request::collect_lenses_for_package, code_lens_request::on_code_lens_request,
    document_symbol::on_document_symbol_request, goto_declaration::on_goto_declaration_request,
    goto_definition::on_goto_definition_request, goto_definition::on_goto_type_definition_request,
    hover::on_hover_request, inlay_hint::on_inlay_hint_request,
    profile_run::on_profile_run_request, references::on_references_request,
    rename::on_prepare_rename_request, rename::on_rename_request, test_run::on_test_run_request,
    tests::on_tests_request,
};

/// LSP client will send initialization request after the server has started.
/// [InitializeParams].`initialization_options` will contain the options sent from the client.
#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub(crate) struct LspInitializationOptions {
    /// Controls whether code lens is enabled by the server
    /// By default this will be set to true (enabled).
    #[serde(rename = "enableCodeLens", default = "default_enable_code_lens")]
    pub(crate) enable_code_lens: bool,

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

fn default_enable_code_lens() -> bool {
    true
}

fn default_enable_parsing_cache() -> bool {
    true
}

fn default_inlay_hints() -> InlayHintsOptions {
    InlayHintsOptions {
        type_hints: default_type_hints(),
        parameter_hints: default_parameter_hints(),
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

impl Default for LspInitializationOptions {
    fn default() -> Self {
        Self {
            enable_code_lens: default_enable_code_lens(),
            enable_parsing_cache: default_enable_parsing_cache(),
            inlay_hints: default_inlay_hints(),
        }
    }
}

pub(crate) fn on_initialize(
    state: &mut LspState,
    params: InitializeParams,
) -> impl Future<Output = Result<InitializeResult, ResponseError>> {
    state.root_path = params.root_uri.and_then(|root_uri| root_uri.to_file_path().ok());
    let initialization_options: LspInitializationOptions = params
        .initialization_options
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    state.options = initialization_options;

    async move {
        let text_document_sync = TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL);

        let code_lens = if initialization_options.enable_code_lens {
            Some(CodeLensOptions { resolve_provider: Some(false) })
        } else {
            None
        };

        let nargo = NargoCapability {
            tests: Some(NargoTestsOptions {
                fetch: Some(true),
                run: Some(true),
                update: Some(true),
            }),
        };

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(text_document_sync),
                code_lens_provider: code_lens,
                document_formatting_provider: true,
                nargo: Some(nargo),
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
                inlay_hint_provider: Some(lsp_types::OneOf::Right(lsp_types::InlayHintOptions {
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                    resolve_provider: None,
                })),
                document_symbol_provider: Some(lsp_types::OneOf::Right(
                    lsp_types::DocumentSymbolOptions {
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                        label: Some("Noir".to_string()),
                    },
                )),
            },
            server_info: None,
        })
    }
}

pub(crate) fn on_formatting(
    state: &mut LspState,
    params: lsp_types::DocumentFormattingParams,
) -> impl Future<Output = Result<Option<Vec<lsp_types::TextEdit>>, ResponseError>> {
    std::future::ready(on_formatting_inner(state, params))
}

fn on_formatting_inner(
    state: &LspState,
    params: lsp_types::DocumentFormattingParams,
) -> Result<Option<Vec<lsp_types::TextEdit>>, ResponseError> {
    let path = params.text_document.uri.to_string();

    if let Some(source) = state.input_files.get(&path) {
        let (module, errors) = noirc_frontend::parse_program(source);
        if !errors.is_empty() {
            return Ok(None);
        }

        let new_text = nargo_fmt::format(source, module, &Config::default());

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
        format!("Could not find file in file manager. File path: {:?}", file_path),
    ))?;
    let byte_index = position_to_byte_index(files, file_id, position).map_err(|err| {
        ResponseError::new(
            ErrorCode::REQUEST_FAILED,
            format!("Could not convert position to byte index. Error: {:?}", err),
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

fn to_lsp_location<'a, F>(
    files: &'a F,
    file_id: F::FileId,
    definition_span: noirc_errors::Span,
) -> Option<Location>
where
    F: fm::codespan_files::Files<'a> + ?Sized,
{
    let range = crate::byte_span_to_range(files, file_id, definition_span.into())?;
    let file_name = files.name(file_id).ok()?;

    let path = file_name.to_string();
    let uri = Url::from_file_path(path).ok()?;

    Some(Location { uri, range })
}

pub(crate) fn on_shutdown(
    _state: &mut LspState,
    _params: (),
) -> impl Future<Output = Result<(), ResponseError>> {
    async { Ok(()) }
}

pub(crate) struct ProcessRequestCallbackArgs<'a> {
    location: noirc_errors::Location,
    files: &'a FileMap,
    interner: &'a NodeInterner,
    interners: &'a HashMap<String, NodeInterner>,
    root_crate_name: String,
    root_crate_dependencies: &'a Vec<Dependency>,
}

pub(crate) fn process_request<F, T>(
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

    let workspace =
        resolve_workspace_for_source_path(file_path.as_path(), &state.root_path).unwrap();
    let package = crate::workspace_package_for_file(&workspace, &file_path).ok_or_else(|| {
        ResponseError::new(ErrorCode::REQUEST_FAILED, "Could not find package for file")
    })?;

    let package_root_path: String = package.root_dir.as_os_str().to_string_lossy().into();

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(
        state,
        &workspace,
        &mut workspace_file_manager,
    );
    let parsed_files = parse_diff(&workspace_file_manager, state);

    let (mut context, crate_id) =
        crate::prepare_package(&workspace_file_manager, &parsed_files, package);

    let interner;
    if let Some(def_interner) = state.cached_definitions.get(&package_root_path) {
        interner = def_interner;
    } else {
        // We ignore the warnings and errors produced by compilation while resolving the definition
        let _ = noirc_driver::check_crate(&mut context, crate_id, false, false, None);
        interner = &context.def_interner;
    }

    let files = context.file_manager.as_file_map();

    let location = position_to_location(
        files,
        &PathString::from(file_path),
        &text_document_position_params.position,
    )?;

    Ok(callback(ProcessRequestCallbackArgs {
        location,
        files,
        interner,
        interners: &state.cached_definitions,
        root_crate_name: package.name.to_string(),
        root_crate_dependencies: &context.crate_graph[context.root_crate_id()].dependencies,
    }))
}
pub(crate) fn find_all_references_in_workspace(
    location: noirc_errors::Location,
    interner: &NodeInterner,
    cached_interners: &HashMap<String, NodeInterner>,
    files: &FileMap,
    include_declaration: bool,
    include_self_type_name: bool,
) -> Option<Vec<Location>> {
    // First find the node that's referenced by the given location, if any
    let referenced = interner.find_referenced(location);

    if let Some(referenced) = referenced {
        // If we found the referenced node, find its location
        let referenced_location = interner.reference_location(referenced);

        // Now we find all references that point to this location, in all interners
        // (there's one interner per package, and all interners in a workspace rely on the
        // same FileManager so a Location/FileId in one package is the same as in another package)
        let mut locations = find_all_references(
            referenced_location,
            interner,
            files,
            include_declaration,
            include_self_type_name,
        );
        for interner in cached_interners.values() {
            locations.extend(find_all_references(
                referenced_location,
                interner,
                files,
                include_declaration,
                include_self_type_name,
            ));
        }

        // The LSP client usually removes duplicate loctions, but we do it here just in case they don't
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

        if locations.is_empty() {
            None
        } else {
            Some(locations)
        }
    } else {
        None
    }
}

pub(crate) fn find_all_references(
    referenced_location: noirc_errors::Location,
    interner: &NodeInterner,
    files: &FileMap,
    include_declaration: bool,
    include_self_type_name: bool,
) -> Vec<Location> {
    interner
        .find_all_references(referenced_location, include_declaration, include_self_type_name)
        .map(|locations| {
            locations
                .iter()
                .filter_map(|location| to_lsp_location(files, location.file, location.span))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod initialization {
    use acvm::blackbox_solver::StubbedBlackBoxSolver;
    use async_lsp::ClientSocket;
    use lsp_types::{
        CodeLensOptions, InitializeParams, TextDocumentSyncCapability, TextDocumentSyncKind,
    };
    use tokio::test;

    use crate::{requests::on_initialize, types::ServerCapabilities, LspState};

    #[test]
    async fn test_on_initialize() {
        let client = ClientSocket::new_closed();
        let mut state = LspState::new(&client, StubbedBlackBoxSolver);
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
