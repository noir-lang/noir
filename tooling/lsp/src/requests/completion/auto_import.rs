use noirc_frontend::{
    ast::Ident,
    hir::def_map::{ModuleDefId, ModuleId},
};

use crate::{
    modules::{get_parent_module, relative_module_full_path, relative_module_id_path},
    use_segment_positions::{
        use_completion_item_additional_text_edits, UseCompletionItemAdditionTextEditsRequest,
    },
};

use super::{
    kinds::{FunctionCompletionKind, FunctionKind, RequestedItems},
    name_matches,
    sort_text::auto_import_sort_text,
    NodeFinder,
};

impl<'a> NodeFinder<'a> {
    pub(super) fn complete_auto_imports(
        &mut self,
        prefix: &str,
        requested_items: RequestedItems,
        function_completion_kind: FunctionCompletionKind,
    ) {
        let current_module_parent_id = self.module_id.parent(self.def_maps);

        for (name, entries) in self.interner.get_auto_import_names() {
            if !name_matches(name, prefix) {
                continue;
            }

            for (module_def_id, visibility, defining_module) in entries {
                if self.suggested_module_def_ids.contains(module_def_id) {
                    continue;
                }

                let mut defining_module = defining_module.as_ref().cloned();
                let mut intermediate_name = None;

                let is_visible =
                    self.module_def_id_is_visible(*module_def_id, *visibility, defining_module);
                if !is_visible {
                    if let Some((parent_module_reexport, reexport_name)) =
                        self.get_parent_module_reexport(*module_def_id)
                    {
                        defining_module = Some(parent_module_reexport);
                        intermediate_name = Some(reexport_name);
                    } else {
                        continue;
                    }
                }

                let completion_items = self.module_def_id_completion_items(
                    *module_def_id,
                    name.clone(),
                    function_completion_kind,
                    FunctionKind::Any,
                    requested_items,
                );

                if completion_items.is_empty() {
                    continue;
                };

                self.suggested_module_def_ids.insert(*module_def_id);

                for mut completion_item in completion_items {
                    let module_full_path = if let Some(defining_module) = defining_module {
                        relative_module_id_path(
                            defining_module,
                            &self.module_id,
                            current_module_parent_id,
                            self.interner,
                        )
                    } else {
                        let Some(module_full_path) = relative_module_full_path(
                            *module_def_id,
                            self.module_id,
                            current_module_parent_id,
                            self.interner,
                        ) else {
                            continue;
                        };
                        module_full_path
                    };

                    let full_path = if defining_module.is_some()
                        || !matches!(module_def_id, ModuleDefId::ModuleId(..))
                    {
                        if let Some(reexport_name) = &intermediate_name {
                            format!("{}::{}::{}", module_full_path, reexport_name, name)
                        } else {
                            format!("{}::{}", module_full_path, name)
                        }
                    } else {
                        module_full_path
                    };

                    let mut label_details = completion_item.label_details.unwrap();
                    label_details.detail = Some(format!("(use {})", full_path));
                    completion_item.label_details = Some(label_details);
                    completion_item.additional_text_edits =
                        Some(use_completion_item_additional_text_edits(
                            UseCompletionItemAdditionTextEditsRequest {
                                full_path: &full_path,
                                files: self.files,
                                file: self.file,
                                lines: &self.lines,
                                nesting: self.nesting,
                                auto_import_line: self.auto_import_line,
                            },
                            &self.use_segment_positions,
                        ));
                    completion_item.sort_text = Some(auto_import_sort_text());

                    self.completion_items.push(completion_item);
                }
            }
        }
    }

    /// Finds a visible reexport for the parent module of the given ModuleDefId.
    fn get_parent_module_reexport(&self, module_def_id: ModuleDefId) -> Option<(ModuleId, Ident)> {
        let parent_module = get_parent_module(self.interner, module_def_id)?;
        let (parent_module_reexport, name, _) = self
            .interner
            .get_reexports(ModuleDefId::ModuleId(parent_module))
            .iter()
            .find(|(module_id, _, visibility)| {
                self.module_def_id_is_visible(ModuleDefId::ModuleId(*module_id), *visibility, None)
            })?;

        Some((*parent_module_reexport, name.clone()))
    }
}
