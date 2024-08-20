use std::collections::BTreeMap;

use lsp_types::{Position, Range, TextEdit};
use noirc_frontend::{
    ast::ItemVisibility,
    graph::{CrateId, Dependency},
    hir::def_map::{CrateDefMap, ModuleId},
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
                    FunctionCompletionKind::NameAndParameters,
                    FunctionKind::Any,
                    requested_items,
                ) else {
                    continue;
                };

                let module_full_path;
                if let ModuleDefId::ModuleId(module_id) = module_def_id {
                    module_full_path = module_id_path(
                        *module_id,
                        &self.module_id,
                        current_module_parent_id,
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
                        current_module_parent_id,
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

fn get_parent_module(interner: &NodeInterner, module_def_id: ModuleDefId) -> Option<ModuleId> {
    let reference_id = module_def_id_to_reference_id(module_def_id);
    interner.reference_module(reference_id).copied()
}

fn get_parent_module_id(
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    module_id: ModuleId,
) -> Option<ModuleId> {
    let crate_def_map = &def_maps[&module_id.krate];
    let module_data = &crate_def_map.modules()[module_id.local_id.0];
    module_data.parent.map(|parent| ModuleId { krate: module_id.krate, local_id: parent })
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

/// Returns the path to reach an item inside `target_module_id` from inside `current_module_id`.
/// Returns a relative path if possible.
fn module_id_path(
    target_module_id: ModuleId,
    current_module_id: &ModuleId,
    current_module_parent_id: Option<ModuleId>,
    interner: &NodeInterner,
    dependencies: &[Dependency],
) -> String {
    if Some(target_module_id) == current_module_parent_id {
        return "super".to_string();
    }

    let mut segments: Vec<&str> = Vec::new();
    let mut is_relative = false;

    if let Some(module_attributes) = interner.try_module_attributes(&target_module_id) {
        segments.push(&module_attributes.name);

        let mut current_attributes = module_attributes;
        loop {
            let parent_module_id =
                &ModuleId { krate: target_module_id.krate, local_id: current_attributes.parent };

            if current_module_id == parent_module_id {
                is_relative = true;
                break;
            }

            if current_module_parent_id == Some(*parent_module_id) {
                segments.push("super");
                is_relative = true;
                break;
            }

            let Some(parent_attributes) = interner.try_module_attributes(parent_module_id) else {
                break;
            };

            segments.push(&parent_attributes.name);
            current_attributes = parent_attributes;
        }
    }

    let crate_id = target_module_id.krate;
    let crate_name = if is_relative {
        None
    } else {
        match crate_id {
            CrateId::Root(_) => Some("crate".to_string()),
            CrateId::Stdlib(_) => Some("std".to_string()),
            CrateId::Crate(_) => dependencies
                .iter()
                .find(|dep| dep.crate_id == crate_id)
                .map(|dep| dep.name.to_string()),
            CrateId::Dummy => unreachable!("ICE: A dummy CrateId should not be accessible"),
        }
    };

    if let Some(crate_name) = &crate_name {
        segments.push(crate_name);
    };

    segments.reverse();
    segments.join("::")
}
