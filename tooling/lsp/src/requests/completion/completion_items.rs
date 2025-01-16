use lsp_types::{
    Command, CompletionItem, CompletionItemKind, CompletionItemLabelDetails, Documentation,
    InsertTextFormat, MarkupContent, MarkupKind,
};
use noirc_frontend::{
    ast::AttributeTarget,
    hir::def_map::{ModuleDefId, ModuleId},
    hir_def::{function::FuncMeta, stmt::HirPattern},
    node_interner::{FuncId, GlobalId, ReferenceId, StructId, TraitId, TypeAliasId},
    QuotedType, Type,
};

use crate::{
    modules::{relative_module_full_path, relative_module_id_path},
    use_segment_positions::{
        use_completion_item_additional_text_edits, UseCompletionItemAdditionTextEditsRequest,
    },
};

use super::{
    sort_text::{
        crate_or_module_sort_text, default_sort_text, new_sort_text, operator_sort_text,
        self_mismatch_sort_text,
    },
    FunctionCompletionKind, FunctionKind, NodeFinder, RequestedItems, TraitReexport,
};

impl<'a> NodeFinder<'a> {
    pub(super) fn module_def_id_completion_items(
        &self,
        module_def_id: ModuleDefId,
        name: String,
        function_completion_kind: FunctionCompletionKind,
        function_kind: FunctionKind,
        requested_items: RequestedItems,
    ) -> Vec<CompletionItem> {
        match requested_items {
            RequestedItems::OnlyTypes => match module_def_id {
                ModuleDefId::FunctionId(_) | ModuleDefId::GlobalId(_) => return Vec::new(),
                ModuleDefId::ModuleId(_)
                | ModuleDefId::TypeId(_)
                | ModuleDefId::TypeAliasId(_)
                | ModuleDefId::TraitId(_) => (),
            },
            RequestedItems::OnlyTraits => match module_def_id {
                ModuleDefId::FunctionId(_) | ModuleDefId::GlobalId(_) | ModuleDefId::TypeId(_) => {
                    return Vec::new()
                }
                ModuleDefId::ModuleId(_)
                | ModuleDefId::TypeAliasId(_)
                | ModuleDefId::TraitId(_) => (),
            },
            RequestedItems::OnlyAttributeFunctions(..) => {
                if !matches!(module_def_id, ModuleDefId::FunctionId(..)) {
                    return Vec::new();
                }
            }
            RequestedItems::AnyItems => (),
        }

        let attribute_first_type =
            if let RequestedItems::OnlyAttributeFunctions(target) = requested_items {
                match target {
                    AttributeTarget::Module => Some(Type::Quoted(QuotedType::Module)),
                    AttributeTarget::Struct => Some(Type::Quoted(QuotedType::StructDefinition)),
                    AttributeTarget::Trait => Some(Type::Quoted(QuotedType::TraitDefinition)),
                    AttributeTarget::Function => Some(Type::Quoted(QuotedType::FunctionDefinition)),
                    AttributeTarget::Let => {
                        // No item can be suggested for a let statement attribute
                        return Vec::new();
                    }
                }
            } else {
                None
            };

        match module_def_id {
            ModuleDefId::ModuleId(id) => vec![self.module_completion_item(name, id)],
            ModuleDefId::FunctionId(func_id) => self.function_completion_items(
                &name,
                func_id,
                function_completion_kind,
                function_kind,
                attribute_first_type.as_ref(),
                None,  // trait_id
                false, // self_prefix
            ),
            ModuleDefId::TypeId(struct_id) => vec![self.struct_completion_item(name, struct_id)],
            ModuleDefId::TypeAliasId(id) => vec![self.type_alias_completion_item(name, id)],
            ModuleDefId::TraitId(trait_id) => vec![self.trait_completion_item(name, trait_id)],
            ModuleDefId::GlobalId(global_id) => vec![self.global_completion_item(name, global_id)],
        }
    }

    pub(super) fn crate_completion_item(
        &self,
        name: impl Into<String>,
        id: ModuleId,
    ) -> CompletionItem {
        self.module_completion_item(name, id)
    }

    pub(super) fn module_completion_item(
        &self,
        name: impl Into<String>,
        id: ModuleId,
    ) -> CompletionItem {
        let completion_item = module_completion_item(name);
        self.completion_item_with_doc_comments(ReferenceId::Module(id), completion_item)
    }

    fn struct_completion_item(&self, name: String, struct_id: StructId) -> CompletionItem {
        let completion_item =
            simple_completion_item(name.clone(), CompletionItemKind::STRUCT, Some(name));
        self.completion_item_with_doc_comments(ReferenceId::Struct(struct_id), completion_item)
    }

    pub(super) fn struct_field_completion_item(
        &self,
        field: &str,
        typ: &Type,
        struct_id: StructId,
        field_index: usize,
        self_type: bool,
    ) -> CompletionItem {
        let completion_item = struct_field_completion_item(field, typ, self_type);
        self.completion_item_with_doc_comments(
            ReferenceId::StructMember(struct_id, field_index),
            completion_item,
        )
    }

    fn type_alias_completion_item(&self, name: String, id: TypeAliasId) -> CompletionItem {
        let completion_item =
            simple_completion_item(name.clone(), CompletionItemKind::STRUCT, Some(name));
        self.completion_item_with_doc_comments(ReferenceId::Alias(id), completion_item)
    }

    fn trait_completion_item(&self, name: String, trait_id: TraitId) -> CompletionItem {
        let completion_item =
            simple_completion_item(name.clone(), CompletionItemKind::INTERFACE, Some(name));
        self.completion_item_with_doc_comments(ReferenceId::Trait(trait_id), completion_item)
    }

    fn global_completion_item(&self, name: String, global_id: GlobalId) -> CompletionItem {
        let global = self.interner.get_global(global_id);
        let typ = self.interner.definition_type(global.definition_id);
        let description = typ.to_string();

        let completion_item =
            simple_completion_item(name, CompletionItemKind::CONSTANT, Some(description));
        self.completion_item_with_doc_comments(ReferenceId::Global(global_id), completion_item)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn function_completion_items(
        &self,
        name: &String,
        func_id: FuncId,
        function_completion_kind: FunctionCompletionKind,
        function_kind: FunctionKind,
        attribute_first_type: Option<&Type>,
        trait_info: Option<(TraitId, Option<&TraitReexport>)>,
        self_prefix: bool,
    ) -> Vec<CompletionItem> {
        let func_meta = self.interner.function_meta(&func_id);

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

        if let Some(attribute_first_type) = attribute_first_type {
            if func_meta.parameters.is_empty() {
                return Vec::new();
            }

            let (_, typ, _) = &func_meta.parameters.0[0];
            if typ != attribute_first_type {
                return Vec::new();
            }
        }

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
                            return Vec::new();
                        }
                    } else if let Type::Tuple(self_tuple_types) = self_type {
                        // Tuple types of different lengths seem to also have methods defined on all of them,
                        // so here we reject methods for tuples where the length doesn't match.
                        if let Type::Tuple(func_self_tuple_types) = func_self_type {
                            if self_tuple_types.len() != func_self_tuple_types.len() {
                                return Vec::new();
                            }
                        }
                    }
                } else {
                    return Vec::new();
                }
            }
        }

        let make_completion_item = |is_macro_call| {
            self.function_completion_item(
                name,
                func_id,
                func_meta,
                func_self_type,
                function_completion_kind,
                function_kind,
                attribute_first_type,
                trait_info,
                self_prefix,
                is_macro_call,
            )
        };

        // When suggesting functions in attributes, never suggest a macro call
        if attribute_first_type.is_some() {
            return vec![make_completion_item(false)];
        }

        // Special case: the `unquote` macro
        // (it's unlikely users will define a function named `unquote` that does something different than std's unquote)
        if name == "unquote" {
            return vec![make_completion_item(true)];
        }

        let modifiers = self.interner.function_modifiers(&func_id);
        if modifiers.is_comptime
            && matches!(func_meta.return_type(), Type::Quoted(QuotedType::Quoted))
        {
            if self.in_comptime {
                vec![make_completion_item(false), make_completion_item(true)]
            } else {
                // If not in a comptime block we can't operate with comptime values so the only thing
                // we can do is call a macro.
                vec![make_completion_item(true)]
            }
        } else {
            vec![make_completion_item(false)]
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn function_completion_item(
        &self,
        name: &String,
        func_id: FuncId,
        func_meta: &FuncMeta,
        func_self_type: Option<&Type>,
        function_completion_kind: FunctionCompletionKind,
        function_kind: FunctionKind,
        attribute_first_type: Option<&Type>,
        trait_info: Option<(TraitId, Option<&TraitReexport>)>,
        self_prefix: bool,
        is_macro_call: bool,
    ) -> CompletionItem {
        let is_operator = if let Some(trait_impl_id) = &func_meta.trait_impl {
            let trait_impl = self.interner.get_trait_implementation(*trait_impl_id);
            let trait_impl = trait_impl.borrow();
            self.interner.is_operator_trait(trait_impl.trait_id)
        } else {
            false
        };
        let name = if self_prefix { format!("self.{}", name) } else { name.clone() };
        let name = if is_macro_call { format!("{}!", name) } else { name };
        let name = &name;
        let description = func_meta_type_to_string(func_meta, func_self_type.is_some());
        let mut has_arguments = false;

        let completion_item = match function_completion_kind {
            FunctionCompletionKind::Name => simple_completion_item(
                name,
                CompletionItemKind::FUNCTION,
                Some(description.clone()),
            ),
            FunctionCompletionKind::NameAndParameters => {
                let kind = CompletionItemKind::FUNCTION;
                let skip_first_argument = attribute_first_type.is_some();
                let insert_text = self.compute_function_insert_text(
                    func_meta,
                    name,
                    function_kind,
                    skip_first_argument,
                );

                if insert_text.ends_with("()") {
                    let label =
                        if skip_first_argument { name.to_string() } else { format!("{}()", name) };
                    simple_completion_item(label, kind, Some(description.clone()))
                } else {
                    has_arguments = true;
                    snippet_completion_item(
                        format!("{}(…)", name),
                        kind,
                        insert_text,
                        Some(description.clone()),
                    )
                }
            }
        };

        let completion_item = completion_item_with_detail(completion_item, description);

        let completion_item = if is_operator {
            completion_item_with_sort_text(completion_item, operator_sort_text())
        } else if function_kind == FunctionKind::Any && name == "new" {
            completion_item_with_sort_text(completion_item, new_sort_text())
        } else if function_kind == FunctionKind::Any && func_self_type.is_some() {
            completion_item_with_sort_text(completion_item, self_mismatch_sort_text())
        } else {
            completion_item
        };

        let mut completion_item = match function_completion_kind {
            FunctionCompletionKind::Name => completion_item,
            FunctionCompletionKind::NameAndParameters => {
                if has_arguments {
                    completion_item_with_trigger_parameter_hints_command(completion_item)
                } else {
                    completion_item
                }
            }
        };

        self.auto_import_trait_if_trait_method(func_id, trait_info, &mut completion_item);

        self.completion_item_with_doc_comments(ReferenceId::Function(func_id), completion_item)
    }

    fn auto_import_trait_if_trait_method(
        &self,
        func_id: FuncId,
        trait_info: Option<(TraitId, Option<&TraitReexport>)>,
        completion_item: &mut CompletionItem,
    ) -> Option<()> {
        // If this is a trait method, check if the trait is in scope
        let (trait_id, trait_reexport) = trait_info?;

        let trait_name = if let Some(trait_reexport) = trait_reexport {
            trait_reexport.name
        } else {
            let trait_ = self.interner.get_trait(trait_id);
            &trait_.name
        };

        let module_data =
            &self.def_maps[&self.module_id.krate].modules()[self.module_id.local_id.0];
        if !module_data.scope().find_name(trait_name).is_none() {
            return None;
        }

        // If not, automatically import it
        let current_module_parent_id = self.module_id.parent(self.def_maps);
        let module_full_path = if let Some(reexport_data) = trait_reexport {
            relative_module_id_path(
                *reexport_data.module_id,
                &self.module_id,
                current_module_parent_id,
                self.interner,
            )
        } else {
            relative_module_full_path(
                ModuleDefId::FunctionId(func_id),
                self.module_id,
                current_module_parent_id,
                self.interner,
            )?
        };
        let full_path = format!("{}::{}", module_full_path, trait_name);
        let mut label_details = completion_item.label_details.clone().unwrap();
        label_details.detail = Some(format!("(use {})", full_path));
        completion_item.label_details = Some(label_details);
        completion_item.additional_text_edits = Some(use_completion_item_additional_text_edits(
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

        None
    }

    fn compute_function_insert_text(
        &self,
        func_meta: &FuncMeta,
        name: &str,
        function_kind: FunctionKind,
        skip_first_argument: bool,
    ) -> String {
        let mut text = String::new();
        text.push_str(name);
        text.push('(');

        let mut parameters = func_meta.parameters.0.iter();
        if skip_first_argument {
            parameters.next();
        }

        let mut index = 1;
        for (pattern, _, _) in parameters {
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

    pub(super) fn completion_item_with_doc_comments(
        &self,
        id: ReferenceId,
        completion_item: CompletionItem,
    ) -> CompletionItem {
        if let Some(doc_comments) = self.interner.doc_comments(id) {
            let docs = doc_comments.join("\n");
            CompletionItem {
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: docs,
                })),
                ..completion_item
            }
        } else {
            completion_item
        }
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

pub(super) fn trait_impl_method_completion_item(
    label: impl Into<String>,
    insert_text: impl Into<String>,
) -> CompletionItem {
    snippet_completion_item(label, CompletionItemKind::METHOD, insert_text, None)
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

pub(super) fn struct_field_completion_item(
    field: &str,
    typ: &Type,
    self_type: bool,
) -> CompletionItem {
    field_completion_item(field, typ.to_string(), self_type)
}

pub(super) fn field_completion_item(
    field: &str,
    typ: impl Into<String>,
    self_type: bool,
) -> CompletionItem {
    if self_type {
        simple_completion_item(format!("self.{field}"), CompletionItemKind::FIELD, Some(typ.into()))
    } else {
        simple_completion_item(field, CompletionItemKind::FIELD, Some(typ.into()))
    }
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

pub(super) fn completion_item_with_detail(
    completion_item: CompletionItem,
    detail: String,
) -> CompletionItem {
    CompletionItem { detail: Some(detail), ..completion_item }
}
