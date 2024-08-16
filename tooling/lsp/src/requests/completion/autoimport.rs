use lsp_types::{Position, Range, TextEdit};
use noirc_frontend::{
    graph::{CrateId, Dependency},
    hir::def_map::ModuleId,
    macros_api::{ModuleDefId, NodeInterner},
    node_interner::ReferenceId,
};

use super::{
    kinds::{FunctionCompletionKind, FunctionKind, RequestedItems},
    name_matches, NodeFinder,
};

impl<'a> NodeFinder<'a> {
    pub(super) fn complete_autoimports(&mut self, prefix: &str, requested_items: RequestedItems) {
        for (name, module_def_ids) in self.interner.get_autoimport_names() {
            if !name_matches(name, prefix) {
                continue;
            }

            for module_def_id in module_def_ids {
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

                let Some(parent_module) = get_parent_module(&self.interner, *module_def_id) else {
                    continue;
                };

                let module_full_path =
                    module_id_full_path(parent_module, self.interner, &self.dependencies);

                let mut label_details = completion_item.label_details.unwrap();
                label_details.detail = Some(format!("(use {}::{})", module_full_path, name));
                completion_item.label_details = Some(label_details);

                completion_item.additional_text_edits = Some(vec![TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    new_text: format!("use {}::{};\n", module_full_path, name),
                }]);

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

fn module_id_full_path(
    module: &ModuleId,
    interner: &NodeInterner,
    dependencies: &[Dependency],
) -> String {
    let mut string = String::new();

    let crate_id = module.krate;
    let crate_name = match crate_id {
        CrateId::Root(_) => None,
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

    let Some(module_attributes) = interner.try_module_attributes(module) else {
        return string;
    };

    if wrote_crate {
        string.push_str("::");
    }

    let mut segments = Vec::new();
    let mut current_attributes = module_attributes;
    while let Some(parent_attributes) = interner.try_module_attributes(&ModuleId {
        krate: module.krate,
        local_id: current_attributes.parent,
    }) {
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
