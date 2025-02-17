use noirc_frontend::{hir::def_map::ModuleDefId, node_interner::Reexport};

use crate::{
    modules::{get_parent_module_reexport, module_def_id_relative_path},
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

            for entry in entries {
                let module_def_id = entry.module_def_id;
                if self.suggested_module_def_ids.contains(&module_def_id) {
                    continue;
                }

                let visibility = entry.visibility;
                let mut defining_module = entry.defining_module.as_ref().cloned();

                // If the item is offered via a re-export of it's parent module, this holds the name of the reexport.
                let mut intermediate_name = None;

                let is_visible =
                    self.module_def_id_is_visible(module_def_id, visibility, defining_module);
                if !is_visible {
                    if let Some(reexport) = self.get_parent_module_reexport(module_def_id) {
                        defining_module = Some(reexport.module_id);
                        intermediate_name = Some(reexport.name.clone());
                    } else {
                        continue;
                    }
                }

                let completion_items = self.module_def_id_completion_items(
                    module_def_id,
                    name.clone(),
                    function_completion_kind,
                    FunctionKind::Any,
                    requested_items,
                );

                if completion_items.is_empty() {
                    continue;
                };

                self.suggested_module_def_ids.insert(module_def_id);

                for mut completion_item in completion_items {
                    let Some(full_path) = module_def_id_relative_path(
                        module_def_id,
                        name,
                        self.module_id,
                        current_module_parent_id,
                        defining_module,
                        &intermediate_name,
                        self.interner,
                    ) else {
                        continue;
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

    fn get_parent_module_reexport(&self, module_def_id: ModuleDefId) -> Option<&Reexport> {
        get_parent_module_reexport(
            module_def_id,
            self.module_id,
            self.interner,
            self.def_maps,
            self.dependencies,
        )
    }
}
