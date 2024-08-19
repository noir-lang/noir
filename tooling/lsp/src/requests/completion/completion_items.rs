use lsp_types::{
    Command, CompletionItem, CompletionItemKind, CompletionItemLabelDetails, InsertTextFormat,
};
use noirc_frontend::{
    hir_def::{function::FuncMeta, stmt::HirPattern},
    macros_api::{ModuleDefId, StructId},
    node_interner::{FuncId, GlobalId, TraitId, TypeAliasId},
    Type,
};

use super::{
    sort_text::{
        crate_or_module_sort_text, default_sort_text, new_sort_text, operator_sort_text,
        self_mismatch_sort_text,
    },
    FunctionCompletionKind, FunctionKind, NodeFinder, RequestedItems,
};

impl<'a> NodeFinder<'a> {
    pub(super) fn module_def_id_completion_item(
        &self,
        module_def_id: ModuleDefId,
        name: String,
        function_completion_kind: FunctionCompletionKind,
        function_kind: FunctionKind,
        requested_items: RequestedItems,
    ) -> Option<CompletionItem> {
        match requested_items {
            RequestedItems::OnlyTypes => match module_def_id {
                ModuleDefId::FunctionId(_) | ModuleDefId::GlobalId(_) => return None,
                ModuleDefId::ModuleId(_)
                | ModuleDefId::TypeId(_)
                | ModuleDefId::TypeAliasId(_)
                | ModuleDefId::TraitId(_) => (),
            },
            RequestedItems::AnyItems => (),
        }

        match module_def_id {
            ModuleDefId::ModuleId(_) => Some(module_completion_item(name)),
            ModuleDefId::FunctionId(func_id) => {
                self.function_completion_item(func_id, function_completion_kind, function_kind)
            }
            ModuleDefId::TypeId(struct_id) => Some(self.struct_completion_item(struct_id)),
            ModuleDefId::TypeAliasId(type_alias_id) => {
                Some(self.type_alias_completion_item(type_alias_id))
            }
            ModuleDefId::TraitId(trait_id) => Some(self.trait_completion_item(trait_id)),
            ModuleDefId::GlobalId(global_id) => Some(self.global_completion_item(global_id)),
        }
    }

    fn struct_completion_item(&self, struct_id: StructId) -> CompletionItem {
        let struct_type = self.interner.get_struct(struct_id);
        let struct_type = struct_type.borrow();
        let name = struct_type.name.to_string();

        simple_completion_item(name.clone(), CompletionItemKind::STRUCT, Some(name))
    }

    fn type_alias_completion_item(&self, type_alias_id: TypeAliasId) -> CompletionItem {
        let type_alias = self.interner.get_type_alias(type_alias_id);
        let type_alias = type_alias.borrow();
        let name = type_alias.name.to_string();

        simple_completion_item(name.clone(), CompletionItemKind::STRUCT, Some(name))
    }

    fn trait_completion_item(&self, trait_id: TraitId) -> CompletionItem {
        let trait_ = self.interner.get_trait(trait_id);
        let name = trait_.name.to_string();

        simple_completion_item(name.clone(), CompletionItemKind::INTERFACE, Some(name))
    }

    fn global_completion_item(&self, global_id: GlobalId) -> CompletionItem {
        let global_definition = self.interner.get_global_definition(global_id);
        let name = global_definition.name.clone();

        let global = self.interner.get_global(global_id);
        let typ = self.interner.definition_type(global.definition_id);
        let description = typ.to_string();

        simple_completion_item(name, CompletionItemKind::CONSTANT, Some(description))
    }

    pub(super) fn function_completion_item(
        &self,
        func_id: FuncId,
        function_completion_kind: FunctionCompletionKind,
        function_kind: FunctionKind,
    ) -> Option<CompletionItem> {
        let func_meta = self.interner.function_meta(&func_id);
        let name = &self.interner.function_name(&func_id).to_string();

        let func_self_type = if let Some((pattern, typ, _)) = func_meta.parameters.0.first() {
            if self.hir_pattern_is_self_type(pattern) {
                if let Type::MutableReference(mut_typ) = typ {
                    let typ: &Type = mut_typ;
                    Some(typ)
                } else {
                    Some(typ)
                }
            } else {
                None
            }
        } else {
            None
        };

        match function_kind {
            FunctionKind::Any => (),
            FunctionKind::SelfType(mut self_type) => {
                if let Some(func_self_type) = func_self_type {
                    if matches!(self_type, Type::Integer(..))
                        || matches!(self_type, Type::FieldElement)
                    {
                        // Check that the pattern type is the same as self type.
                        // We do this because some types (only Field and integer types)
                        // have their methods in the same HashMap.

                        if let Type::MutableReference(mut_typ) = self_type {
                            self_type = mut_typ;
                        }

                        if self_type != func_self_type {
                            return None;
                        }
                    } else if let Type::Tuple(self_tuple_types) = self_type {
                        // Tuple types of different lengths seem to also have methods defined on all of them,
                        // so here we reject methods for tuples where the length doesn't match.
                        if let Type::Tuple(func_self_tuple_types) = func_self_type {
                            if self_tuple_types.len() != func_self_tuple_types.len() {
                                return None;
                            }
                        }
                    }
                } else {
                    return None;
                }
            }
        }

        let is_operator = if let Some(trait_impl_id) = &func_meta.trait_impl {
            let trait_impl = self.interner.get_trait_implementation(*trait_impl_id);
            let trait_impl = trait_impl.borrow();
            self.interner.is_operator_trait(trait_impl.trait_id)
        } else {
            false
        };
        let description = func_meta_type_to_string(func_meta, func_self_type.is_some());

        let completion_item = match function_completion_kind {
            FunctionCompletionKind::Name => {
                simple_completion_item(name, CompletionItemKind::FUNCTION, Some(description))
            }
            FunctionCompletionKind::NameAndParameters => {
                let kind = CompletionItemKind::FUNCTION;
                let insert_text = self.compute_function_insert_text(func_meta, name, function_kind);
                let label = if insert_text.ends_with("()") {
                    format!("{}()", name)
                } else {
                    format!("{}(…)", name)
                };

                snippet_completion_item(label, kind, insert_text, Some(description))
            }
        };

        let completion_item = if is_operator {
            completion_item_with_sort_text(completion_item, operator_sort_text())
        } else if function_kind == FunctionKind::Any && name == "new" {
            completion_item_with_sort_text(completion_item, new_sort_text())
        } else if function_kind == FunctionKind::Any && func_self_type.is_some() {
            completion_item_with_sort_text(completion_item, self_mismatch_sort_text())
        } else {
            completion_item
        };

        let completion_item = match function_completion_kind {
            FunctionCompletionKind::Name => completion_item,
            FunctionCompletionKind::NameAndParameters => {
                completion_item_with_trigger_parameter_hints_command(completion_item)
            }
        };

        Some(completion_item)
    }

    fn compute_function_insert_text(
        &self,
        func_meta: &FuncMeta,
        name: &str,
        function_kind: FunctionKind,
    ) -> String {
        let mut text = String::new();
        text.push_str(name);
        text.push('(');

        let mut index = 1;
        for (pattern, _, _) in &func_meta.parameters.0 {
            if index == 1 {
                match function_kind {
                    FunctionKind::SelfType(_) => {
                        if self.hir_pattern_is_self_type(pattern) {
                            continue;
                        }
                    }
                    FunctionKind::Any => (),
                }
            }

            if index > 1 {
                text.push_str(", ");
            }

            text.push_str("${");
            text.push_str(&index.to_string());
            text.push(':');
            self.hir_pattern_to_argument(pattern, &mut text);
            text.push('}');

            index += 1;
        }
        text.push(')');
        text
    }

    fn hir_pattern_to_argument(&self, pattern: &HirPattern, text: &mut String) {
        match pattern {
            HirPattern::Identifier(hir_ident) => {
                text.push_str(self.interner.definition_name(hir_ident.id));
            }
            HirPattern::Mutable(pattern, _) => self.hir_pattern_to_argument(pattern, text),
            HirPattern::Tuple(_, _) | HirPattern::Struct(_, _, _) => text.push('_'),
        }
    }

    fn hir_pattern_is_self_type(&self, pattern: &HirPattern) -> bool {
        match pattern {
            HirPattern::Identifier(hir_ident) => {
                let name = self.interner.definition_name(hir_ident.id);
                name == "self" || name == "_self"
            }
            HirPattern::Mutable(pattern, _) => self.hir_pattern_is_self_type(pattern),
            HirPattern::Tuple(_, _) | HirPattern::Struct(_, _, _) => false,
        }
    }
}

pub(super) fn module_completion_item(name: impl Into<String>) -> CompletionItem {
    completion_item_with_sort_text(
        simple_completion_item(name, CompletionItemKind::MODULE, None),
        crate_or_module_sort_text(),
    )
}

pub(super) fn crate_completion_item(name: impl Into<String>) -> CompletionItem {
    completion_item_with_sort_text(
        simple_completion_item(name, CompletionItemKind::MODULE, None),
        crate_or_module_sort_text(),
    )
}

fn func_meta_type_to_string(func_meta: &FuncMeta, has_self_type: bool) -> String {
    let mut typ = &func_meta.typ;
    if let Type::Forall(_, typ_) = typ {
        typ = typ_;
    }

    if let Type::Function(args, ret, _env, unconstrained) = typ {
        let mut string = String::new();
        if *unconstrained {
            string.push_str("unconstrained ");
        }
        string.push_str("fn(");
        for (index, arg) in args.iter().enumerate() {
            if index > 0 {
                string.push_str(", ");
            }
            if index == 0 && has_self_type {
                type_to_self_string(arg, &mut string);
            } else {
                string.push_str(&arg.to_string());
            }
        }
        string.push(')');

        let ret: &Type = ret;
        if let Type::Unit = ret {
            // Nothing
        } else {
            string.push_str(" -> ");
            string.push_str(&ret.to_string());
        }
        string
    } else {
        typ.to_string()
    }
}

fn type_to_self_string(typ: &Type, string: &mut String) {
    if let Type::MutableReference(..) = typ {
        string.push_str("&mut self");
    } else {
        string.push_str("self");
    }
}

pub(super) fn struct_field_completion_item(field: &str, typ: &Type) -> CompletionItem {
    field_completion_item(field, typ.to_string())
}

pub(super) fn field_completion_item(field: &str, typ: impl Into<String>) -> CompletionItem {
    simple_completion_item(field, CompletionItemKind::FIELD, Some(typ.into()))
}

pub(super) fn simple_completion_item(
    label: impl Into<String>,
    kind: CompletionItemKind,
    description: Option<String>,
) -> CompletionItem {
    CompletionItem {
        label: label.into(),
        label_details: Some(CompletionItemLabelDetails { detail: None, description }),
        kind: Some(kind),
        detail: None,
        documentation: None,
        deprecated: None,
        preselect: None,
        sort_text: Some(default_sort_text()),
        filter_text: None,
        insert_text: None,
        insert_text_format: None,
        insert_text_mode: None,
        text_edit: None,
        additional_text_edits: None,
        command: None,
        commit_characters: None,
        data: None,
        tags: None,
    }
}

pub(super) fn snippet_completion_item(
    label: impl Into<String>,
    kind: CompletionItemKind,
    insert_text: impl Into<String>,
    description: Option<String>,
) -> CompletionItem {
    CompletionItem {
        label: label.into(),
        label_details: Some(CompletionItemLabelDetails { detail: None, description }),
        kind: Some(kind),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        insert_text: Some(insert_text.into()),
        detail: None,
        documentation: None,
        deprecated: None,
        preselect: None,
        sort_text: Some(default_sort_text()),
        filter_text: None,
        insert_text_mode: None,
        text_edit: None,
        additional_text_edits: None,
        command: None,
        commit_characters: None,
        data: None,
        tags: None,
    }
}

pub(super) fn completion_item_with_sort_text(
    completion_item: CompletionItem,
    sort_text: String,
) -> CompletionItem {
    CompletionItem { sort_text: Some(sort_text), ..completion_item }
}

pub(super) fn completion_item_with_trigger_parameter_hints_command(
    completion_item: CompletionItem,
) -> CompletionItem {
    CompletionItem {
        command: Some(Command {
            title: "Trigger parameter hints".to_string(),
            command: "editor.action.triggerParameterHints".to_string(),
            arguments: None,
        }),
        ..completion_item
    }
}
