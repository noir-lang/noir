use lsp_types::{Position, Range, TextEdit};
use noirc_frontend::macros_api::ModuleDefId;

use crate::modules::{get_parent_module_id, module_full_path};

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
        let current_module_parent_id = get_parent_module_id(self.def_maps, self.module_id);

        for (name, entries) in self.interner.get_auto_import_names() {
            if !name_matches(name, prefix) {
                continue;
            }

            for (module_def_id, visibility) in entries {
                if self.suggested_module_def_ids.contains(module_def_id) {
                    continue;
                }

                let Some(mut completion_item) = self.module_def_id_completion_item(
                    *module_def_id,
                    name.clone(),
                    function_completion_kind,
                    FunctionKind::Any,
                    requested_items,
                ) else {
                    continue;
                };

                let Some(module_full_path) = module_full_path(
                    *module_def_id,
                    *visibility,
                    self.module_id,
                    current_module_parent_id,
                    self.interner,
                ) else {
                    continue;
                };

                let full_path = if let ModuleDefId::ModuleId(..) = module_def_id {
                    module_full_path
                } else {
                    format!("{}::{}", module_full_path, name)
                };

                let mut label_details = completion_item.label_details.unwrap();
                label_details.detail = Some(format!("(use {})", full_path));
                completion_item.label_details = Some(label_details);

                let line = self.auto_import_line as u32;
                let character = (self.nesting * 4) as u32;
                let indent = " ".repeat(self.nesting * 4);
                let mut newlines = "\n";

                // If the line we are inserting into is not an empty line, insert an extra line to make some room
                if let Some(line_text) = self.lines.get(line as usize) {
                    if !line_text.trim().is_empty() {
                        newlines = "\n\n";
                    }
                }

                completion_item.additional_text_edits = Some(vec![TextEdit {
                    range: Range {
                        start: Position { line, character },
                        end: Position { line, character },
                    },
                    new_text: format!("use {};{}{}", full_path, newlines, indent),
                }]);

                completion_item.sort_text = Some(auto_import_sort_text());

                self.completion_items.push(completion_item);
                self.suggested_module_def_ids.insert(*module_def_id);
            }
        }
    }
}
