//! Paths to modules and items as a user would write them from a given module: the `use` path
//! for auto-import, the qualified name shown on hover, the path in a generated trait impl stub.

use noirc_frontend::{
    ast::Ident,
    graph::{CrateId, Dependency},
    hir::def_map::{DefMaps, ModuleDefId, ModuleId},
    modules::get_parent_module,
    node_interner::NodeInterner,
};

/// Returns the fully-qualified path of the given `ModuleDefId` relative to `current_module_id`:
/// - If `ModuleDefId` is a module, that module's path is returned
/// - Otherwise, that item's parent module's path is returned
pub(crate) fn relative_module_full_path(
    module_def_id: ModuleDefId,
    current_module_id: ModuleId,
    current_module_parent_id: Option<ModuleId>,
    dependencies: &[Dependency],
    interner: &NodeInterner,
    def_maps: &DefMaps,
) -> Option<String> {
    let full_path;
    if let ModuleDefId::ModuleId(module_id) = module_def_id {
        full_path = relative_module_id_path(
            module_id,
            current_module_id,
            current_module_parent_id,
            dependencies,
            interner,
        );
    } else {
        let parent_module = get_parent_module(module_def_id, interner, def_maps)?;

        // If module_def_id is contained in the current module, the relative path is empty
        if current_module_id == parent_module {
            return None;
        }

        full_path = relative_module_id_path(
            parent_module,
            current_module_id,
            current_module_parent_id,
            dependencies,
            interner,
        );
    }
    Some(full_path)
}

/// Returns the path to reach an item inside `target_module_id` from inside `current_module_id`.
/// Returns a relative path if possible. `dependencies` are those of the current crate.
pub(crate) fn relative_module_id_path(
    target_module_id: ModuleId,
    current_module_id: ModuleId,
    current_module_parent_id: Option<ModuleId>,
    dependencies: &[Dependency],
    interner: &NodeInterner,
) -> String {
    if Some(target_module_id) == current_module_parent_id {
        return "super".to_string();
    }

    let same_crate = target_module_id.krate == current_module_id.krate;
    let mut segments: Vec<String> = Vec::new();
    let mut is_relative = false;

    // Walk up from the target until reaching the current module, its parent or the crate root.
    // The crate graph's root crate has no module attributes at all.
    let mut module_id = target_module_id;
    while let Some(module_attributes) = interner.try_module_attributes(module_id) {
        let Some(parent_local_id) = module_attributes.parent else {
            if !same_crate {
                let name = dependency_name(module_id.krate, dependencies)
                    .unwrap_or_else(|| module_attributes.name.clone());
                segments.push(name);
            }
            break;
        };
        segments.push(module_attributes.name.clone());

        let parent_module_id = ModuleId { krate: module_id.krate, local_id: parent_local_id };
        if current_module_id == parent_module_id {
            is_relative = true;
            break;
        }
        if current_module_parent_id == Some(parent_module_id) {
            segments.push("super".to_string());
            is_relative = true;
            break;
        }
        module_id = parent_module_id;
    }

    if !is_relative && same_crate {
        segments.push("crate".to_string());
    }

    segments.reverse();
    segments.join("::")
}

/// Returns the full path of `module` as seen from the crate `crate_id`, named `crate_name`.
/// `dependencies` are those of that crate.
pub(crate) fn module_full_path(
    module: ModuleId,
    interner: &NodeInterner,
    crate_id: CrateId,
    crate_name: &str,
    dependencies: &[Dependency],
) -> String {
    let mut segments: Vec<String> = Vec::new();

    let mut module_id = module;
    let registered_root_name = loop {
        let Some(module_attributes) = interner.try_module_attributes(module_id) else {
            break None;
        };
        let Some(parent_local_id) = module_attributes.parent else {
            break Some(module_attributes.name.clone());
        };
        segments.push(module_attributes.name.clone());
        module_id = ModuleId { krate: module.krate, local_id: parent_local_id };
    };

    let root_name = if module.krate == crate_id {
        Some(crate_name.to_string())
    } else {
        dependency_name(module.krate, dependencies).or(registered_root_name)
    };
    segments.extend(root_name);

    segments.reverse();
    segments.join("::")
}

/// The name under which a crate with `dependencies` refers to `krate`, if it depends on it
/// directly.
///
/// A crate's root module is registered in the interner under a single name, but every
/// dependent crate may give it a different one, so paths written from a crate use its own
/// dependency names.
fn dependency_name(krate: CrateId, dependencies: &[Dependency]) -> Option<String> {
    dependencies.iter().find(|dependency| dependency.crate_id == krate).map(Dependency::as_name)
}

/// Returns the relative path to reach `module_def_id` named `name` starting from `current_module_id`.
///
/// - `defining_module` might be `Some` if the item is reexported from another module
/// - `intermediate_name` might be `Some` if the item's parent module is reexport from another module
///   (this will be the name of the reexport)
///
/// Returns `None` if `module_def_id` isn't visible from the current module, neither directly, neither via
/// any of its reexports (or parent module reexports).
#[allow(clippy::too_many_arguments)]
pub(crate) fn module_def_id_relative_path(
    module_def_id: ModuleDefId,
    name: &str,
    current_module_id: ModuleId,
    current_module_parent_id: Option<ModuleId>,
    defining_module: Option<ModuleId>,
    intermediate_name: &Option<Ident>,
    dependencies: &[Dependency],
    interner: &NodeInterner,
    def_maps: &DefMaps,
) -> Option<String> {
    let module_path = if let Some(defining_module) = defining_module {
        relative_module_id_path(
            defining_module,
            current_module_id,
            current_module_parent_id,
            dependencies,
            interner,
        )
    } else {
        relative_module_full_path(
            module_def_id,
            current_module_id,
            current_module_parent_id,
            dependencies,
            interner,
            def_maps,
        )?
    };

    let path = if defining_module.is_some() || !matches!(module_def_id, ModuleDefId::ModuleId(..)) {
        if let Some(reexport_name) = &intermediate_name {
            format!("{module_path}::{reexport_name}::{name}")
        } else {
            format!("{module_path}::{name}")
        }
    } else {
        module_path
    };

    Some(path)
}

#[cfg(test)]
mod tests {
    use async_lsp::lsp_types::{
        CompletionParams, CompletionResponse, HoverContents, HoverParams, PartialResultParams,
        TextDocumentIdentifier, TextDocumentPositionParams, WorkDoneProgressParams,
    };

    use crate::{
        requests::{on_completion_request, on_hover_request},
        test_utils,
        tests::apply_text_edits,
    };

    // In `test_programs/aliased_dependency`, package `a` is a dependency of `b` under the name
    // `x` and of `c` under the name `y`. All three are members of one workspace, so whatever the
    // order the packages are collected in, `a` goes by at least two names in one interner.
    const DEPENDENTS: [(&str, &str); 2] = [("b", "x"), ("c", "y")];

    #[test]
    fn auto_import_uses_the_name_the_current_package_gives_a_dependency() {
        for (package, alias) in DEPENDENTS {
            let src = "fn main() {\n    function_in_aliased_cr>|<\n}\n";
            let (mut state, file_uri, position, src) =
                test_utils::init_lsp_server_with_inline_source_and_cursor(
                    "aliased_dependency",
                    &format!("{package}/src/lib.nr"),
                    src,
                );

            let response = on_completion_request(
                &mut state,
                CompletionParams {
                    text_document_position: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri: file_uri },
                        position,
                    },
                    work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                    partial_result_params: PartialResultParams { partial_result_token: None },
                    context: None,
                },
            )
            .expect("Could not execute on_completion_request");
            let Some(CompletionResponse::Array(items)) = response else {
                panic!("Expected completion items");
            };
            let item = items
                .into_iter()
                .find(|item| item.additional_text_edits.is_some())
                .expect("Expected an auto-import completion");

            let changed = apply_text_edits(&src, &item.additional_text_edits.unwrap());
            let expected_import = format!("use {alias}::sub::function_in_aliased_crate;\n");
            assert!(
                changed.starts_with(&expected_import),
                "unexpected auto-import in `{package}`: {changed}"
            );
        }
    }

    #[test]
    fn hover_uses_the_name_the_current_package_gives_a_dependency() {
        for (package, alias) in DEPENDENTS {
            let src =
                format!("fn main() {{\n    {alias}::sub::function_in_aliased_cr>|<ate();\n}}\n");
            let (mut state, file_uri, position, _) =
                test_utils::init_lsp_server_with_inline_source_and_cursor(
                    "aliased_dependency",
                    &format!("{package}/src/lib.nr"),
                    &src,
                );

            let hover = on_hover_request(
                &mut state,
                HoverParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri: file_uri },
                        position,
                    },
                    work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                },
            )
            .expect("Could not execute hover")
            .expect("Expected a hover");
            let HoverContents::Markup(markup) = hover.contents else {
                panic!("Expected hover contents to be Markup");
            };

            assert!(
                markup.value.starts_with(&format!("    {alias}::sub\n")),
                "unexpected hover in `{package}`: {}",
                markup.value
            );
        }
    }
}
