use lsp_types::{Position, Range, TextEdit};
use noirc_frontend::{
    ast::ItemVisibility,
    graph::{CrateId, Dependency},
    hir::def_map::ModuleId,
    macros_api::{ModuleDefId, NodeInterner},
    node_interner::ReferenceId,
};

use super::{
    kinds::{FunctionCompletionKind, FunctionKind, RequestedItems},
    name_matches,
    sort_text::auto_import_sort_text,
    NodeFinder,
};

impl<'a> NodeFinder<'a> {
    pub(super) fn complete_auto_imports(&mut self, prefix: &str, requested_items: RequestedItems) {
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
                    FunctionCompletionKind::NameAndParameters,
                    FunctionKind::Any,
                    requested_items,
                ) else {
                    continue;
                };

                let module_full_path;
                if let ModuleDefId::ModuleId(module_id) = module_def_id {
                    module_full_path = module_id_path(
                        module_id,
                        &self.module_id,
                        self.interner,
                        self.dependencies,
                    );
                } else {
                    let Some(parent_module) = get_parent_module(self.interner, *module_def_id)
                    else {
                        continue;
                    };

                    match *visibility {
                        ItemVisibility::Public => (),
                        ItemVisibility::Private => {
                            // Technically this can't be reached because we don't record private items for auto-import,
                            // but this is here for completeness.
                            continue;
                        }
                        ItemVisibility::PublicCrate => {
                            if self.module_id.krate != parent_module.krate {
                                continue;
                            }
                        }
                    }

                    module_full_path = module_id_path(
                        parent_module,
                        &self.module_id,
                        self.interner,
                        self.dependencies,
                    );
                }

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

fn get_parent_module(interner: &NodeInterner, module_def_id: ModuleDefId) -> Option<&ModuleId> {
    let reference_id = module_def_id_to_reference_id(module_def_id);
    interner.reference_module(reference_id)
}

fn module_def_id_to_reference_id(module_def_id: ModuleDefId) -> ReferenceId {
    match module_def_id {
        ModuleDefId::ModuleId(id) => ReferenceId::Module(id),
        ModuleDefId::FunctionId(id) => ReferenceId::Function(id),
        ModuleDefId::TypeId(id) => ReferenceId::Struct(id),
        ModuleDefId::TypeAliasId(id) => ReferenceId::Alias(id),
        ModuleDefId::TraitId(id) => ReferenceId::Trait(id),
        ModuleDefId::GlobalId(id) => ReferenceId::Global(id),
    }
}

/// Computes the path of `module_id` relative to `current_module_id`.
/// If it's not relative, the full path is returned.
fn module_id_path(
    module_id: &ModuleId,
    current_module_id: &ModuleId,
    interner: &NodeInterner,
    dependencies: &[Dependency],
) -> String {
    let mut string = String::new();

    let crate_id = module_id.krate;
    let crate_name = match crate_id {
        CrateId::Root(_) => Some("crate".to_string()),
        CrateId::Crate(_) => dependencies
            .iter()
            .find(|dep| dep.crate_id == crate_id)
            .map(|dep| format!("{}", dep.name)),
        CrateId::Stdlib(_) => Some("std".to_string()),
        CrateId::Dummy => None,
    };

    let wrote_crate = if let Some(crate_name) = crate_name {
        string.push_str(&crate_name);
        true
    } else {
        false
    };

    let Some(module_attributes) = interner.try_module_attributes(module_id) else {
        return string;
    };

    if wrote_crate {
        string.push_str("::");
    }

    let mut segments = Vec::new();
    let mut current_attributes = module_attributes;
    loop {
        let parent_module_id =
            &ModuleId { krate: module_id.krate, local_id: current_attributes.parent };

        // If the parent module is the current module we stop because we want a relative path to the module
        if current_module_id == parent_module_id {
            // When the path is relative we don't want the "crate::" prefix anymore
            string = string.strip_prefix("crate::").unwrap_or(&string).to_string();
            break;
        }

        let Some(parent_attributes) = interner.try_module_attributes(parent_module_id) else {
            break;
        };

        segments.push(&parent_attributes.name);
        current_attributes = parent_attributes;
    }

    for segment in segments.iter().rev() {
        string.push_str(segment);
        string.push_str("::");
    }

    string.push_str(&module_attributes.name);

    string
}
