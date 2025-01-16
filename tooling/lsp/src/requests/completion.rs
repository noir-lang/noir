use std::{
    collections::{BTreeMap, HashMap, HashSet},
    future::{self, Future},
};

use async_lsp::ResponseError;
use completion_items::{
    field_completion_item, simple_completion_item, snippet_completion_item,
    trait_impl_method_completion_item,
};
use convert_case::{Case, Casing};
use fm::{FileId, FileMap, PathString};
use kinds::{FunctionCompletionKind, FunctionKind, RequestedItems};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse};
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ast::{
        AsTraitPath, AttributeTarget, BlockExpression, CallExpression, ConstructorExpression,
        Expression, ExpressionKind, ForLoopStatement, GenericTypeArgs, Ident, IfExpression,
        IntegerBitSize, ItemVisibility, LValue, Lambda, LetStatement, MemberAccessExpression,
        MethodCallExpression, NoirFunction, NoirStruct, NoirTraitImpl, Path, PathKind, Pattern,
        Signedness, Statement, TraitBound, TraitImplItemKind, TypeImpl, TypePath,
        UnresolvedGeneric, UnresolvedGenerics, UnresolvedType, UnresolvedTypeData,
        UnresolvedTypeExpression, UseTree, UseTreeKind, Visitor,
    },
    graph::{CrateId, Dependency},
    hir::{
        def_map::{CrateDefMap, LocalModuleId, ModuleDefId, ModuleId},
        resolution::visibility::{
            item_in_module_is_visible, method_call_is_visible, struct_member_is_visible,
        },
    },
    hir_def::traits::Trait,
    node_interner::{FuncId, NodeInterner, ReferenceId, StructId},
    parser::{Item, ItemKind, ParsedSubModule},
    token::{MetaAttribute, Token, Tokens},
    Kind, ParsedModule, StructType, Type, TypeBinding,
};
use sort_text::underscore_sort_text;

use crate::{
    requests::to_lsp_location, trait_impl_method_stub_generator::TraitImplMethodStubGenerator,
    use_segment_positions::UseSegmentPositions, utils, visibility::module_def_id_is_visible,
    LspState,
};

use super::{process_request, TraitReexport};

mod auto_import;
mod builtins;
mod completion_items;
mod kinds;
mod sort_text;
mod tests;

pub(crate) fn on_completion_request(
    state: &mut LspState,
    params: CompletionParams,
) -> impl Future<Output = Result<Option<CompletionResponse>, ResponseError>> {
    let uri = params.text_document_position.clone().text_document.uri;

    let result = process_request(state, params.text_document_position.clone(), |args| {
        let path = PathString::from_path(uri.to_file_path().unwrap());
        args.files.get_file_id(&path).and_then(|file_id| {
            utils::position_to_byte_index(
                args.files,
                file_id,
                &params.text_document_position.position,
            )
            .and_then(|byte_index| {
                let file = args.files.get_file(file_id).unwrap();
                let source = file.source();
                let byte = source.as_bytes().get(byte_index - 1).copied();
                let (parsed_module, _errors) = noirc_frontend::parse_program(source);

                let mut finder = NodeFinder::new(
                    args.files,
                    file_id,
                    source,
                    byte_index,
                    byte,
                    args.crate_id,
                    args.def_maps,
                    args.dependencies,
                    args.interner,
                );
                finder.find(&parsed_module)
            })
        })
    });
    future::ready(result)
}

struct NodeFinder<'a> {
    files: &'a FileMap,
    file: FileId,
    source: &'a str,
    lines: Vec<&'a str>,
    byte_index: usize,
    byte: Option<u8>,
    /// The module ID in scope. This might change as we traverse the AST
    /// if we are analyzing something inside an inline module declaration.
    module_id: ModuleId,
    def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
    dependencies: &'a Vec<Dependency>,
    interner: &'a NodeInterner,
    /// Completion items we find along the way.
    completion_items: Vec<CompletionItem>,
    /// Local variables in the current scope, mapped to their locations.
    /// As we traverse the AST, we collect local variables.
    local_variables: HashMap<String, Span>,
    /// Type parameters in the current scope. These are collected when entering
    /// a struct, a function, etc., and cleared afterwards.
    type_parameters: HashSet<String>,
    /// ModuleDefIds we already suggested, so we don't offer these for auto-import.
    suggested_module_def_ids: HashSet<ModuleDefId>,
    /// How many nested `mod` we are in deep
    nesting: usize,
    /// The line where an auto_import must be inserted
    auto_import_line: usize,
    use_segment_positions: UseSegmentPositions,
    self_type: Option<Type>,
    in_comptime: bool,
    /// The function we are in, if any
    func_id: Option<FuncId>,
}

impl<'a> NodeFinder<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        files: &'a FileMap,
        file: FileId,
        source: &'a str,
        byte_index: usize,
        byte: Option<u8>,
        krate: CrateId,
        def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
        dependencies: &'a Vec<Dependency>,
        interner: &'a NodeInterner,
    ) -> Self {
        // Find the module the current file belongs to
        let def_map = &def_maps[&krate];
        let local_id = if let Some((module_index, _)) =
            def_map.modules().iter().find(|(_, module_data)| module_data.location.file == file)
        {
            LocalModuleId(module_index)
        } else {
            def_map.root()
        };
        let module_id = ModuleId { krate, local_id };
        Self {
            files,
            file,
            source,
            lines: source.lines().collect(),
            byte_index,
            byte,
            module_id,
            def_maps,
            dependencies,
            interner,
            completion_items: Vec::new(),
            local_variables: HashMap::new(),
            type_parameters: HashSet::new(),
            suggested_module_def_ids: HashSet::new(),
            nesting: 0,
            auto_import_line: 0,
            use_segment_positions: UseSegmentPositions::default(),
            self_type: None,
            in_comptime: false,
            func_id: None,
        }
    }

    fn find(&mut self, parsed_module: &ParsedModule) -> Option<CompletionResponse> {
        parsed_module.accept(self);

        if self.completion_items.is_empty() {
            None
        } else {
            let mut items = std::mem::take(&mut self.completion_items);

            // Show items that start with underscore last in the list
            for item in items.iter_mut() {
                if item.label.starts_with('_') {
                    item.sort_text = Some(underscore_sort_text());
                }
            }

            Some(CompletionResponse::Array(items))
        }
    }

    fn complete_constructor_field_name(&mut self, constructor_expression: &ConstructorExpression) {
        let span = if let UnresolvedTypeData::Named(path, _, _) = &constructor_expression.typ.typ {
            path.last_ident().span()
        } else {
            constructor_expression.typ.span
        };

        let location = Location::new(span, self.file);
        let Some(ReferenceId::Struct(struct_id)) = self.interner.find_referenced(location) else {
            return;
        };

        let struct_type = self.interner.get_struct(struct_id);
        let struct_type = struct_type.borrow();

        // First get all of the struct's fields
        let mut fields: Vec<_> =
            struct_type.get_fields_as_written().into_iter().enumerate().collect();

        // Remove the ones that already exists in the constructor
        for (used_name, _) in &constructor_expression.fields {
            fields.retain(|(_, field)| field.name.0.contents != used_name.0.contents);
        }

        let self_prefix = false;
        for (field_index, field) in &fields {
            self.completion_items.push(self.struct_field_completion_item(
                &field.name.0.contents,
                &field.typ,
                struct_type.id,
                *field_index,
                self_prefix,
            ));
        }
    }

    fn find_in_path(&mut self, path: &Path, requested_items: RequestedItems) {
        self.find_in_path_impl(path, requested_items, false);
    }

    fn find_in_path_impl(
        &mut self,
        path: &Path,
        requested_items: RequestedItems,
        mut in_the_middle: bool,
    ) {
        if !self.includes_span(path.span) {
            return;
        }

        let after_colons = self.byte == Some(b':');

        let mut idents: Vec<Ident> = Vec::new();

        // Find in which ident we are in, and in which part of it
        // (it could be that we are completing in the middle of an ident)
        for segment in &path.segments {
            let ident = &segment.ident;

            // Check if we are at the end of the ident
            if self.byte_index == ident.span().end() as usize {
                idents.push(ident.clone());
                break;
            }

            // Check if we are in the middle of an ident
            if self.includes_span(ident.span()) {
                // If so, take the substring and push that as the list of idents
                // we'll do autocompletion for
                let offset = self.byte_index - ident.span().start() as usize;
                let substring = ident.0.contents[0..offset].to_string();
                let ident = Ident::new(
                    substring,
                    Span::from(ident.span().start()..ident.span().start() + offset as u32),
                );
                idents.push(ident);
                in_the_middle = true;
                break;
            }

            idents.push(ident.clone());

            // Stop if the cursor is right after this ident and '::'
            if after_colons && self.byte_index == ident.span().end() as usize + 2 {
                break;
            }
        }

        if idents.len() < path.segments.len() {
            in_the_middle = true;
        }

        let prefix;
        let at_root;

        if after_colons {
            prefix = String::new();
            at_root = false;
        } else {
            prefix = idents.pop().unwrap().to_string();
            at_root = idents.is_empty();
        }

        let prefix = prefix.to_case(Case::Snake);

        let is_single_segment = !after_colons && idents.is_empty() && path.kind == PathKind::Plain;
        let module_id;

        // When completing in the middle of an ident, we don't want to complete
        // with function parameters because there might already be function parameters,
        // and in the middle of a path it leads to code that won't compile
        let function_completion_kind = if in_the_middle {
            FunctionCompletionKind::Name
        } else {
            FunctionCompletionKind::NameAndParameters
        };

        if idents.is_empty() {
            module_id = self.module_id;
        } else {
            let Some(module_def_id) = self.resolve_path(idents) else {
                return;
            };

            match module_def_id {
                ModuleDefId::ModuleId(id) => module_id = id,
                ModuleDefId::TypeId(struct_id) => {
                    let struct_type = self.interner.get_struct(struct_id);
                    self.complete_type_methods(
                        &Type::Struct(struct_type, vec![]),
                        &prefix,
                        FunctionKind::Any,
                        function_completion_kind,
                        false, // self_prefix
                    );
                    return;
                }
                ModuleDefId::FunctionId(_) => {
                    // There's nothing inside a function
                    return;
                }
                ModuleDefId::TypeAliasId(type_alias_id) => {
                    let type_alias = self.interner.get_type_alias(type_alias_id);
                    let type_alias = type_alias.borrow();
                    self.complete_type_methods(
                        &type_alias.typ,
                        &prefix,
                        FunctionKind::Any,
                        function_completion_kind,
                        false, // self_prefix
                    );
                    return;
                }
                ModuleDefId::TraitId(trait_id) => {
                    let trait_ = self.interner.get_trait(trait_id);
                    self.complete_trait_methods(
                        trait_,
                        &prefix,
                        FunctionKind::Any,
                        function_completion_kind,
                    );
                    return;
                }
                ModuleDefId::GlobalId(_) => return,
            }
        }

        self.complete_in_module(
            module_id,
            &prefix,
            path.kind,
            at_root,
            function_completion_kind,
            requested_items,
        );

        if is_single_segment {
            match requested_items {
                RequestedItems::AnyItems => {
                    self.local_variables_completion(&prefix);
                    self.builtin_functions_completion(&prefix, function_completion_kind);
                    self.builtin_values_completion(&prefix);
                    self.builtin_types_completion(&prefix);
                    self.type_parameters_completion(&prefix);
                    if let Some(self_type) = &self.self_type {
                        let self_prefix = true;
                        self.complete_type_fields_and_methods(
                            &self_type.clone(),
                            &prefix,
                            function_completion_kind,
                            self_prefix,
                        );
                    }
                }
                RequestedItems::OnlyTypes => {
                    self.builtin_types_completion(&prefix);
                    self.type_parameters_completion(&prefix);
                }
                RequestedItems::OnlyTraits | RequestedItems::OnlyAttributeFunctions(..) => (),
            }
            self.complete_auto_imports(&prefix, requested_items, function_completion_kind);
        }
    }

    fn local_variables_completion(&mut self, prefix: &str) {
        for (name, span) in &self.local_variables {
            if name_matches(name, prefix) {
                let location = Location::new(*span, self.file);
                let description = if let Some(ReferenceId::Local(definition_id)) =
                    self.interner.reference_at_location(location)
                {
                    let typ = self.interner.definition_type(definition_id);
                    Some(typ.to_string())
                } else {
                    None
                };

                self.completion_items.push(simple_completion_item(
                    name,
                    CompletionItemKind::VARIABLE,
                    description,
                ));
            }
        }
    }

    fn type_parameters_completion(&mut self, prefix: &str) {
        for name in &self.type_parameters {
            if name_matches(name, prefix) {
                self.completion_items.push(simple_completion_item(
                    name,
                    CompletionItemKind::TYPE_PARAMETER,
                    None,
                ));
            }
        }
    }

    fn find_in_use_tree(&mut self, use_tree: &UseTree, prefixes: &mut Vec<Path>) {
        match &use_tree.kind {
            UseTreeKind::Path(ident, alias) => {
                prefixes.push(use_tree.prefix.clone());
                self.find_in_use_tree_path(prefixes, ident, alias);
                prefixes.pop();
            }
            UseTreeKind::List(use_trees) => {
                prefixes.push(use_tree.prefix.clone());
                for use_tree in use_trees {
                    self.find_in_use_tree(use_tree, prefixes);
                }
                prefixes.pop();
            }
        }
    }

    fn find_in_use_tree_path(
        &mut self,
        prefixes: &Vec<Path>,
        ident: &Ident,
        alias: &Option<Ident>,
    ) {
        if let Some(_alias) = alias {
            // Won't handle completion if there's an alias (for now)
            return;
        }

        let after_colons = self.byte == Some(b':');
        let at_ident_end = self.byte_index == ident.span().end() as usize;
        let at_ident_colons_end =
            after_colons && self.byte_index - 2 == ident.span().end() as usize;

        if !(at_ident_end || at_ident_colons_end) {
            return;
        }

        let path_kind = prefixes[0].kind;

        let mut segments: Vec<Ident> = Vec::new();
        for prefix in prefixes {
            for segment in &prefix.segments {
                segments.push(segment.ident.clone());
            }
        }

        let function_completion_kind = FunctionCompletionKind::Name;
        let requested_items = RequestedItems::AnyItems;

        if after_colons {
            // We are right after "::"
            segments.push(ident.clone());

            if let Some(module_id) = self.resolve_module(segments) {
                let prefix = "";
                let at_root = false;
                self.complete_in_module(
                    module_id,
                    prefix,
                    path_kind,
                    at_root,
                    function_completion_kind,
                    requested_items,
                );
            };
        } else {
            // We are right after the last segment
            let prefix = ident.to_string().to_case(Case::Snake);
            if segments.is_empty() {
                let at_root = true;
                self.complete_in_module(
                    self.module_id,
                    &prefix,
                    path_kind,
                    at_root,
                    function_completion_kind,
                    requested_items,
                );
            } else if let Some(module_id) = self.resolve_module(segments) {
                let at_root = false;
                self.complete_in_module(
                    module_id,
                    &prefix,
                    path_kind,
                    at_root,
                    function_completion_kind,
                    requested_items,
                );
            }
        }
    }

    fn collect_local_variables(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier(ident) => {
                self.local_variables.insert(ident.to_string(), ident.span());
            }
            Pattern::Mutable(pattern, _, _) => self.collect_local_variables(pattern),
            Pattern::Tuple(patterns, _) => {
                for pattern in patterns {
                    self.collect_local_variables(pattern);
                }
            }
            Pattern::Struct(_, patterns, _) => {
                for (_, pattern) in patterns {
                    self.collect_local_variables(pattern);
                }
            }
            Pattern::Interned(..) => (),
        }
    }

    fn collect_type_parameters_in_generics(&mut self, generics: &UnresolvedGenerics) {
        for generic in generics {
            self.collect_type_parameters_in_generic(generic);
        }
    }

    fn collect_type_parameters_in_generic(&mut self, generic: &UnresolvedGeneric) {
        match generic {
            UnresolvedGeneric::Variable(ident) => {
                self.type_parameters.insert(ident.to_string());
            }
            UnresolvedGeneric::Numeric { ident, typ: _ } => {
                self.type_parameters.insert(ident.to_string());
            }
            UnresolvedGeneric::Resolved(..) => (),
        };
    }

    fn complete_type_fields_and_methods(
        &mut self,
        typ: &Type,
        prefix: &str,
        function_completion_kind: FunctionCompletionKind,
        self_prefix: bool,
    ) {
        let typ = &typ;
        match typ {
            Type::Struct(struct_type, generics) => {
                self.complete_struct_fields(&struct_type.borrow(), generics, prefix, self_prefix);
            }
            Type::MutableReference(typ) => {
                return self.complete_type_fields_and_methods(
                    typ,
                    prefix,
                    function_completion_kind,
                    self_prefix,
                );
            }
            Type::Alias(type_alias, _) => {
                let type_alias = type_alias.borrow();
                return self.complete_type_fields_and_methods(
                    &type_alias.typ,
                    prefix,
                    function_completion_kind,
                    self_prefix,
                );
            }
            Type::CheckedCast { to, .. } => {
                return self.complete_type_fields_and_methods(
                    to,
                    prefix,
                    function_completion_kind,
                    self_prefix,
                );
            }
            Type::Tuple(types) => {
                self.complete_tuple_fields(types, self_prefix);
            }
            Type::TypeVariable(var) | Type::NamedGeneric(var, _) => {
                if let TypeBinding::Bound(ref typ) = &*var.borrow() {
                    return self.complete_type_fields_and_methods(
                        typ,
                        prefix,
                        function_completion_kind,
                        self_prefix,
                    );
                }
            }
            Type::FieldElement
            | Type::Array(_, _)
            | Type::Slice(_)
            | Type::Integer(_, _)
            | Type::Bool
            | Type::String(_)
            | Type::FmtString(_, _)
            | Type::Unit
            | Type::TraitAsType(_, _, _)
            | Type::Function(..)
            | Type::Forall(_, _)
            | Type::Constant(..)
            | Type::Quoted(_)
            | Type::InfixExpr(_, _, _)
            | Type::Error => (),
        }

        self.complete_type_methods(
            typ,
            prefix,
            FunctionKind::SelfType(typ),
            function_completion_kind,
            self_prefix,
        );
    }

    fn complete_type_methods(
        &mut self,
        typ: &Type,
        prefix: &str,
        function_kind: FunctionKind,
        function_completion_kind: FunctionCompletionKind,
        self_prefix: bool,
    ) {
        self.complete_trait_constraints_methods(
            typ,
            prefix,
            function_kind,
            function_completion_kind,
        );

        let Some(methods_by_name) = self.interner.get_type_methods(typ) else {
            return;
        };

        let struct_id = get_type_struct_id(typ);
        let is_primitive = typ.is_primitive();
        let has_self_param = matches!(function_kind, FunctionKind::SelfType(..));

        for (name, methods) in methods_by_name {
            if !name_matches(name, prefix) {
                continue;
            }

            for (func_id, trait_id) in
                methods.find_matching_methods(typ, has_self_param, self.interner)
            {
                if let Some(struct_id) = struct_id {
                    let modifiers = self.interner.function_modifiers(&func_id);
                    let visibility = modifiers.visibility;
                    if !struct_member_is_visible(
                        struct_id,
                        visibility,
                        self.module_id,
                        self.def_maps,
                    ) {
                        continue;
                    }
                }

                let mut trait_reexport = None;

                if let Some(trait_id) = trait_id {
                    let modifiers = self.interner.function_modifiers(&func_id);
                    let visibility = modifiers.visibility;
                    let module_def_id = ModuleDefId::TraitId(trait_id);
                    if !module_def_id_is_visible(
                        module_def_id,
                        self.module_id,
                        visibility,
                        None, // defining module
                        self.interner,
                        self.def_maps,
                    ) {
                        // Try to find a visible reexport of the trait
                        // that is visible from the current module
                        let Some((visible_module_id, name, _)) =
                            self.interner.get_trait_reexports(trait_id).iter().find(
                                |(module_id, _, visibility)| {
                                    module_def_id_is_visible(
                                        module_def_id,
                                        self.module_id,
                                        *visibility,
                                        Some(*module_id),
                                        self.interner,
                                        self.def_maps,
                                    )
                                },
                            )
                        else {
                            continue;
                        };

                        trait_reexport = Some(TraitReexport { module_id: visible_module_id, name });
                    }
                }

                if is_primitive
                    && !method_call_is_visible(
                        typ,
                        func_id,
                        self.module_id,
                        self.interner,
                        self.def_maps,
                    )
                {
                    continue;
                }

                let completion_items = self.function_completion_items(
                    name,
                    func_id,
                    function_completion_kind,
                    function_kind,
                    None, // attribute first type
                    trait_id.map(|id| (id, trait_reexport.as_ref())),
                    self_prefix,
                );
                if !completion_items.is_empty() {
                    self.completion_items.extend(completion_items);
                    self.suggested_module_def_ids.insert(ModuleDefId::FunctionId(func_id));
                }
            }
        }
    }

    fn complete_trait_constraints_methods(
        &mut self,
        typ: &Type,
        prefix: &str,
        function_kind: FunctionKind,
        function_completion_kind: FunctionCompletionKind,
    ) {
        let Some(func_id) = self.func_id else {
            return;
        };

        let func_meta = self.interner.function_meta(&func_id);
        for constraint in &func_meta.trait_constraints {
            if *typ == constraint.typ {
                let trait_ = self.interner.get_trait(constraint.trait_bound.trait_id);
                self.complete_trait_methods(
                    trait_,
                    prefix,
                    function_kind,
                    function_completion_kind,
                );
            }
        }
    }

    fn complete_trait_methods(
        &mut self,
        trait_: &Trait,
        prefix: &str,
        function_kind: FunctionKind,
        function_completion_kind: FunctionCompletionKind,
    ) {
        let self_prefix = false;

        for (name, func_id) in &trait_.method_ids {
            if name_matches(name, prefix) {
                let completion_items = self.function_completion_items(
                    name,
                    *func_id,
                    function_completion_kind,
                    function_kind,
                    None, // attribute first type
                    None, // trait_id (we are suggesting methods for `Trait::>|<` so no need to auto-import it)
                    self_prefix,
                );
                if !completion_items.is_empty() {
                    self.completion_items.extend(completion_items);
                    self.suggested_module_def_ids.insert(ModuleDefId::FunctionId(*func_id));
                }
            }
        }
    }

    fn complete_struct_fields(
        &mut self,
        struct_type: &StructType,
        generics: &[Type],
        prefix: &str,
        self_prefix: bool,
    ) {
        for (field_index, (name, visibility, typ)) in
            struct_type.get_fields_with_visibility(generics).iter().enumerate()
        {
            if !struct_member_is_visible(struct_type.id, *visibility, self.module_id, self.def_maps)
            {
                continue;
            }

            if !name_matches(name, prefix) {
                continue;
            }

            self.completion_items.push(self.struct_field_completion_item(
                name,
                typ,
                struct_type.id,
                field_index,
                self_prefix,
            ));
        }
    }

    fn complete_tuple_fields(&mut self, types: &[Type], self_prefix: bool) {
        for (index, typ) in types.iter().enumerate() {
            let name = index.to_string();
            self.completion_items.push(field_completion_item(&name, typ.to_string(), self_prefix));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn complete_in_module(
        &mut self,
        module_id: ModuleId,
        prefix: &str,
        path_kind: PathKind,
        at_root: bool,
        function_completion_kind: FunctionCompletionKind,
        requested_items: RequestedItems,
    ) {
        let def_map = &self.def_maps[&module_id.krate];
        let Some(mut module_data) = def_map.modules().get(module_id.local_id.0) else {
            return;
        };

        if at_root {
            match path_kind {
                PathKind::Crate => {
                    let Some(root_module_data) = def_map.modules().get(def_map.root().0) else {
                        return;
                    };
                    module_data = root_module_data;
                }
                PathKind::Super => {
                    let Some(parent) = module_data.parent else {
                        return;
                    };
                    let Some(parent_module_data) = def_map.modules().get(parent.0) else {
                        return;
                    };
                    module_data = parent_module_data;
                }
                PathKind::Dep => (),
                PathKind::Plain => (),
            }
        }

        let function_kind = FunctionKind::Any;

        for ident in module_data.scope().names() {
            let name = &ident.0.contents;

            if name_matches(name, prefix) {
                let per_ns = module_data.find_name(ident);
                if let Some((module_def_id, visibility, _)) = per_ns.types {
                    if item_in_module_is_visible(
                        self.def_maps,
                        self.module_id,
                        module_id,
                        visibility,
                    ) {
                        let completion_items = self.module_def_id_completion_items(
                            module_def_id,
                            name.clone(),
                            function_completion_kind,
                            function_kind,
                            requested_items,
                        );
                        if !completion_items.is_empty() {
                            self.completion_items.extend(completion_items);
                            self.suggested_module_def_ids.insert(module_def_id);
                        }
                    }
                }

                if let Some((module_def_id, visibility, _)) = per_ns.values {
                    if item_in_module_is_visible(
                        self.def_maps,
                        self.module_id,
                        module_id,
                        visibility,
                    ) {
                        let completion_items = self.module_def_id_completion_items(
                            module_def_id,
                            name.clone(),
                            function_completion_kind,
                            function_kind,
                            requested_items,
                        );
                        if !completion_items.is_empty() {
                            self.completion_items.extend(completion_items);
                            self.suggested_module_def_ids.insert(module_def_id);
                        }
                    }
                }
            }
        }

        if at_root && path_kind == PathKind::Plain {
            for dependency in self.dependencies {
                let dependency_name = dependency.as_name();
                if name_matches(&dependency_name, prefix) {
                    let root_id = self.def_maps[&dependency.crate_id].root();
                    let module_id = ModuleId { krate: dependency.crate_id, local_id: root_id };
                    self.completion_items
                        .push(self.crate_completion_item(dependency_name, module_id));
                }
            }

            if name_matches("crate::", prefix) {
                self.completion_items.push(simple_completion_item(
                    "crate::",
                    CompletionItemKind::KEYWORD,
                    None,
                ));
            }

            if module_data.parent.is_some() && name_matches("super::", prefix) {
                self.completion_items.push(simple_completion_item(
                    "super::",
                    CompletionItemKind::KEYWORD,
                    None,
                ));
            }
        }
    }

    fn resolve_module(&self, segments: Vec<Ident>) -> Option<ModuleId> {
        if let Some(ModuleDefId::ModuleId(module_id)) = self.resolve_path(segments) {
            Some(module_id)
        } else {
            None
        }
    }

    fn resolve_path(&self, segments: Vec<Ident>) -> Option<ModuleDefId> {
        let last_segment = segments.last().unwrap().clone();

        // If we can't resolve a path trough lookup, let's see if the last segment is bound to a type
        let location = Location::new(last_segment.span(), self.file);
        if let Some(reference_id) = self.interner.find_referenced(location) {
            if let Some(id) = module_def_id_from_reference_id(reference_id) {
                return Some(id);
            }
        }

        None
    }

    fn suggest_no_arguments_attributes(&mut self, prefix: &str, attributes: &[&str]) {
        for name in attributes {
            if name_matches(name, prefix) {
                self.completion_items.push(simple_completion_item(
                    *name,
                    CompletionItemKind::METHOD,
                    None,
                ));
            }
        }
    }

    fn suggest_one_argument_attributes(&mut self, prefix: &str, attributes: &[&str]) {
        for name in attributes {
            if name_matches(name, prefix) {
                self.completion_items.push(snippet_completion_item(
                    format!("{}(…)", name),
                    CompletionItemKind::METHOD,
                    format!("{}(${{1:name}})", name),
                    None,
                ));
            }
        }
    }

    fn suggest_trait_impl_function(
        &mut self,
        noir_trait_impl: &NoirTraitImpl,
        noir_function: &NoirFunction,
    ) {
        // First find the trait
        let location = Location::new(noir_trait_impl.trait_name.span(), self.file);
        let Some(ReferenceId::Trait(trait_id)) = self.interner.find_referenced(location) else {
            return;
        };

        let trait_ = self.interner.get_trait(trait_id);

        // Get all methods
        let mut method_ids = trait_.method_ids.clone();

        // Remove the ones that already are implemented
        for item in &noir_trait_impl.items {
            if let TraitImplItemKind::Function(noir_function) = &item.item.kind {
                method_ids.remove(noir_function.name());
            }
        }

        let indent = 0;

        // Suggest the ones that match the name
        let prefix = noir_function.name();
        for (name, func_id) in method_ids {
            if !name_matches(&name, prefix) {
                continue;
            }

            let func_meta = self.interner.function_meta(&func_id);
            let modifiers = self.interner.function_modifiers(&func_id);

            let mut generator = TraitImplMethodStubGenerator::new(
                &name,
                func_meta,
                modifiers,
                trait_,
                noir_trait_impl,
                self.interner,
                self.def_maps,
                self.module_id,
                indent,
            );
            generator.set_body("${1}".to_string());

            let stub = generator.generate();

            // We don't need the initial indent nor the final newlines
            let stub = stub.trim();
            // We also don't need the leading "fn " as that's already in the code;
            let stub = stub.strip_prefix("fn ").unwrap();

            let label = if func_meta.parameters.is_empty() {
                format!("fn {}()", &name)
            } else {
                format!("fn {}(..)", &name)
            };

            let completion_item = trait_impl_method_completion_item(label, stub);
            let completion_item = self
                .completion_item_with_doc_comments(ReferenceId::Function(func_id), completion_item);

            self.completion_items.push(completion_item);
        }
    }

    fn try_set_self_type(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier(ident) => {
                if ident.0.contents == "self" {
                    let location = Location::new(ident.span(), self.file);
                    if let Some(ReferenceId::Local(definition_id)) =
                        self.interner.find_referenced(location)
                    {
                        self.self_type =
                            Some(self.interner.definition_type(definition_id).follow_bindings());
                    }
                }
            }
            Pattern::Mutable(pattern, ..) => self.try_set_self_type(pattern),
            Pattern::Tuple(..) | Pattern::Struct(..) | Pattern::Interned(..) => (),
        }
    }

    fn get_lvalue_type(&self, lvalue: &LValue) -> Option<Type> {
        match lvalue {
            LValue::Ident(ident) => {
                let location = Location::new(ident.span(), self.file);
                if let Some(ReferenceId::Local(definition_id)) =
                    self.interner.find_referenced(location)
                {
                    let typ = self.interner.definition_type(definition_id);
                    Some(typ)
                } else {
                    None
                }
            }
            LValue::MemberAccess { object, field_name, .. } => {
                let typ = self.get_lvalue_type(object)?;
                get_field_type(&typ, &field_name.0.contents)
            }
            LValue::Index { array, .. } => {
                let typ = self.get_lvalue_type(array)?;
                get_array_element_type(typ)
            }
            LValue::Dereference(lvalue, ..) => self.get_lvalue_type(lvalue),
            LValue::Interned(..) => None,
        }
    }

    /// Determine where each segment in a `use` statement is located.

    fn includes_span(&self, span: Span) -> bool {
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}

impl<'a> Visitor for NodeFinder<'a> {
    fn visit_item(&mut self, item: &Item) -> bool {
        if let ItemKind::Import(use_tree, _) = &item.kind {
            if let Some(lsp_location) = to_lsp_location(self.files, self.file, item.span) {
                self.auto_import_line = (lsp_location.range.end.line + 1) as usize;
            }
            self.use_segment_positions.add(use_tree);
        }

        self.includes_span(item.span)
    }

    fn visit_import(
        &mut self,
        use_tree: &UseTree,
        _span: Span,
        _visibility: ItemVisibility,
    ) -> bool {
        let mut prefixes = Vec::new();
        self.find_in_use_tree(use_tree, &mut prefixes);
        false
    }

    fn visit_parsed_submodule(&mut self, parsed_sub_module: &ParsedSubModule, _span: Span) -> bool {
        // Switch `self.module_id` to the submodule
        let previous_module_id = self.module_id;

        let def_map = &self.def_maps[&self.module_id.krate];
        let Some(module_data) = def_map.modules().get(self.module_id.local_id.0) else {
            return false;
        };
        if let Some(child_module) = module_data.children.get(&parsed_sub_module.name) {
            self.module_id = ModuleId { krate: self.module_id.krate, local_id: *child_module };
        }

        let old_auto_import_line = self.auto_import_line;
        self.nesting += 1;

        if let Some(lsp_location) =
            to_lsp_location(self.files, self.file, parsed_sub_module.name.span())
        {
            self.auto_import_line = (lsp_location.range.start.line + 1) as usize;
        }

        parsed_sub_module.accept_children(self);

        // Restore the old module before continuing
        self.module_id = previous_module_id;
        self.nesting -= 1;
        self.auto_import_line = old_auto_import_line;

        false
    }

    fn visit_noir_function(&mut self, noir_function: &NoirFunction, span: Span) -> bool {
        for attribute in noir_function.secondary_attributes() {
            attribute.accept(AttributeTarget::Function, self);
        }

        let old_type_parameters = self.type_parameters.clone();
        self.collect_type_parameters_in_generics(&noir_function.def.generics);

        for param in &noir_function.def.parameters {
            self.try_set_self_type(&param.pattern);
            param.typ.accept(self);
        }

        noir_function.def.return_type.accept(self);

        for constraint in &noir_function.def.where_clause {
            constraint.accept(self);
        }

        self.local_variables.clear();
        for param in &noir_function.def.parameters {
            self.collect_local_variables(&param.pattern);
        }

        let old_in_comptime = self.in_comptime;
        self.in_comptime = noir_function.def.is_comptime;

        if let Some(ReferenceId::Function(func_id)) = self
            .interner
            .reference_at_location(Location::new(noir_function.name_ident().span(), self.file))
        {
            self.func_id = Some(func_id);
        }

        noir_function.def.body.accept(Some(span), self);

        self.func_id = None;

        self.in_comptime = old_in_comptime;
        self.type_parameters = old_type_parameters;
        self.self_type = None;

        false
    }

    fn visit_noir_trait_impl(&mut self, noir_trait_impl: &NoirTraitImpl, _: Span) -> bool {
        self.find_in_path(&noir_trait_impl.trait_name, RequestedItems::OnlyTypes);
        noir_trait_impl.object_type.accept(self);

        self.type_parameters.clear();
        self.collect_type_parameters_in_generics(&noir_trait_impl.impl_generics);

        for item in &noir_trait_impl.items {
            if let TraitImplItemKind::Function(noir_function) = &item.item.kind {
                // Check if it's `fn foo>|<` and neither `(` nor `<` follow
                if noir_function.name_ident().span().end() as usize == self.byte_index
                    && noir_function.parameters().is_empty()
                {
                    let bytes = self.source.as_bytes();
                    let mut cursor = self.byte_index;
                    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                        cursor += 1;
                    }
                    let char = bytes[cursor] as char;
                    if char != '(' && char != '<' {
                        self.suggest_trait_impl_function(noir_trait_impl, noir_function);
                        return false;
                    }
                }
            }

            item.item.accept(self);
        }

        self.type_parameters.clear();

        false
    }

    fn visit_type_impl(&mut self, type_impl: &TypeImpl, _: Span) -> bool {
        type_impl.object_type.accept(self);

        self.type_parameters.clear();
        self.collect_type_parameters_in_generics(&type_impl.generics);

        for (method, span) in &type_impl.methods {
            method.item.accept(*span, self);

            // Optimization: stop looking in functions past the completion cursor
            if span.end() as usize > self.byte_index {
                break;
            }
        }

        self.type_parameters.clear();

        false
    }

    fn visit_noir_struct(&mut self, noir_struct: &NoirStruct, _: Span) -> bool {
        for attribute in &noir_struct.attributes {
            attribute.accept(AttributeTarget::Struct, self);
        }

        self.type_parameters.clear();
        self.collect_type_parameters_in_generics(&noir_struct.generics);

        for field in &noir_struct.fields {
            field.item.typ.accept(self);
        }

        self.type_parameters.clear();

        false
    }

    fn visit_trait_item_function(
        &mut self,
        name: &Ident,
        generics: &UnresolvedGenerics,
        parameters: &[(Ident, UnresolvedType)],
        return_type: &noirc_frontend::ast::FunctionReturnType,
        where_clause: &[noirc_frontend::ast::UnresolvedTraitConstraint],
        body: &Option<BlockExpression>,
    ) -> bool {
        let old_type_parameters = self.type_parameters.clone();
        self.collect_type_parameters_in_generics(generics);

        for (_name, unresolved_type) in parameters {
            unresolved_type.accept(self);
        }

        return_type.accept(self);

        for unresolved_trait_constraint in where_clause {
            unresolved_trait_constraint.accept(self);
        }

        if let Some(body) = body {
            self.local_variables.clear();
            for (name, _) in parameters {
                self.local_variables.insert(name.to_string(), name.span());
            }

            if let Some(ReferenceId::Function(func_id)) =
                self.interner.reference_at_location(Location::new(name.span(), self.file))
            {
                self.func_id = Some(func_id);
            }

            body.accept(None, self);

            self.func_id = None;
        };

        self.type_parameters = old_type_parameters;

        false
    }

    fn visit_call_expression(&mut self, call_expression: &CallExpression, _: Span) -> bool {
        //
        // foo::b>|<(...)
        //
        // In this case we want to suggest items in foo but if they are functions
        // we don't want to insert arguments, because they are already there (even if
        // they could be wrong) just because inserting them would lead to broken code.
        if let ExpressionKind::Variable(path) = &call_expression.func.kind {
            if self.includes_span(path.span) {
                self.find_in_path_impl(path, RequestedItems::AnyItems, true);
                return false;
            }
        }

        // Check if it's this case:
        //
        // foo.>|<(...)
        //
        // "foo." is actually broken, but it's parsed as "foo", so this is seen
        // as "foo(...)" but if we are at a dot right after "foo" it means it's
        // the above case and we want to suggest methods of foo's type.
        let after_dot = self.byte == Some(b'.');
        if after_dot && call_expression.func.span.end() as usize == self.byte_index - 1 {
            let location = Location::new(call_expression.func.span, self.file);
            if let Some(typ) = self.interner.type_at_location(location) {
                let prefix = "";
                let self_prefix = false;
                self.complete_type_fields_and_methods(
                    &typ,
                    prefix,
                    FunctionCompletionKind::Name,
                    self_prefix,
                );
                return false;
            }
        }

        true
    }

    fn visit_method_call_expression(
        &mut self,
        method_call_expression: &MethodCallExpression,
        _: Span,
    ) -> bool {
        // Check if it's this case:
        //
        // foo.b>|<(...)
        //
        // In this case we want to suggest items in foo but if they are functions
        // we don't want to insert arguments, because they are already there (even if
        // they could be wrong) just because inserting them would lead to broken code.
        if self.includes_span(method_call_expression.method_name.span()) {
            let location = Location::new(method_call_expression.object.span, self.file);
            if let Some(typ) = self.interner.type_at_location(location) {
                let prefix = method_call_expression.method_name.to_string();
                let offset =
                    self.byte_index - method_call_expression.method_name.span().start() as usize;
                let prefix = prefix[0..offset].to_string();
                let self_prefix = false;
                self.complete_type_fields_and_methods(
                    &typ,
                    &prefix,
                    FunctionCompletionKind::Name,
                    self_prefix,
                );
                return false;
            }
        }

        true
    }

    fn visit_block_expression(
        &mut self,
        block_expression: &BlockExpression,
        _: Option<Span>,
    ) -> bool {
        let old_local_variables = self.local_variables.clone();
        for statement in &block_expression.statements {
            statement.accept(self);

            // Optimization: stop looking in statements past the completion cursor
            if statement.span.end() as usize > self.byte_index {
                break;
            }
        }
        self.local_variables = old_local_variables;

        false
    }

    fn visit_let_statement(&mut self, let_statement: &LetStatement) -> bool {
        let_statement.accept_children(self);
        self.collect_local_variables(&let_statement.pattern);
        false
    }

    fn visit_global(&mut self, let_statement: &LetStatement, _: Span) -> bool {
        let_statement.accept_children(self);
        false
    }

    fn visit_comptime_statement(&mut self, statement: &Statement) -> bool {
        // When entering a comptime block, regular local variables shouldn't be offered anymore
        let old_local_variables = self.local_variables.clone();
        self.local_variables.clear();

        let old_in_comptime = self.in_comptime;
        self.in_comptime = true;

        statement.accept(self);

        self.in_comptime = old_in_comptime;
        self.local_variables = old_local_variables;

        false
    }

    fn visit_for_loop_statement(&mut self, for_loop_statement: &ForLoopStatement) -> bool {
        let old_local_variables = self.local_variables.clone();
        let ident = &for_loop_statement.identifier;
        self.local_variables.insert(ident.to_string(), ident.span());

        for_loop_statement.accept_children(self);

        self.local_variables = old_local_variables;

        false
    }

    fn visit_lvalue_ident(&mut self, ident: &Ident) {
        // If we have `foo.>|<` we suggest `foo`'s type fields and methods
        if self.byte == Some(b'.') && ident.span().end() as usize == self.byte_index - 1 {
            let location = Location::new(ident.span(), self.file);
            if let Some(ReferenceId::Local(definition_id)) = self.interner.find_referenced(location)
            {
                let typ = self.interner.definition_type(definition_id);
                let prefix = "";
                let self_prefix = false;
                self.complete_type_fields_and_methods(
                    &typ,
                    prefix,
                    FunctionCompletionKind::NameAndParameters,
                    self_prefix,
                );
            }
        }
    }

    fn visit_lvalue_member_access(
        &mut self,
        object: &LValue,
        field_name: &Ident,
        span: Span,
    ) -> bool {
        // If we have `foo.bar.>|<` we solve the type of `foo`, get the field `bar`,
        // then suggest methods of the resulting type.
        if self.byte == Some(b'.') && span.end() as usize == self.byte_index - 1 {
            if let Some(typ) = self.get_lvalue_type(object) {
                if let Some(typ) = get_field_type(&typ, &field_name.0.contents) {
                    let prefix = "";
                    let self_prefix = false;
                    self.complete_type_fields_and_methods(
                        &typ,
                        prefix,
                        FunctionCompletionKind::NameAndParameters,
                        self_prefix,
                    );
                }
            }

            return false;
        }
        true
    }

    fn visit_lvalue_index(&mut self, array: &LValue, _index: &Expression, span: Span) -> bool {
        // If we have `foo[index].>|<` we solve the type of `foo`, then get the array/slice element type,
        // then suggest methods of that type.
        if self.byte == Some(b'.') && span.end() as usize == self.byte_index - 1 {
            if let Some(typ) = self.get_lvalue_type(array) {
                if let Some(typ) = get_array_element_type(typ) {
                    let prefix = "";
                    let self_prefix = false;
                    self.complete_type_fields_and_methods(
                        &typ,
                        prefix,
                        FunctionCompletionKind::NameAndParameters,
                        self_prefix,
                    );
                }
            }
            return false;
        }
        true
    }

    fn visit_lvalue_dereference(&mut self, lvalue: &LValue, span: Span) -> bool {
        if self.byte == Some(b'.') && span.end() as usize == self.byte_index - 1 {
            if let Some(typ) = self.get_lvalue_type(lvalue) {
                let prefix = "";
                let self_prefix = false;
                self.complete_type_fields_and_methods(
                    &typ,
                    prefix,
                    FunctionCompletionKind::NameAndParameters,
                    self_prefix,
                );
            }
            return false;
        }

        true
    }

    fn visit_variable(&mut self, path: &Path, _: Span) -> bool {
        self.find_in_path(path, RequestedItems::AnyItems);
        false
    }

    fn visit_expression(&mut self, expression: &Expression) -> bool {
        expression.accept_children(self);

        // "foo." (no identifier afterwards) is parsed as the expression on the left hand-side of the dot.
        // Here we check if there's a dot at the completion position, and if the expression
        // ends right before the dot. If so, it means we want to complete the expression's type fields and methods.
        // We only do this after visiting nested expressions, because in an expression like `foo & bar.` we want
        // to complete for `bar`, not for `foo & bar`.
        if self.completion_items.is_empty()
            && self.byte == Some(b'.')
            && expression.span.end() as usize == self.byte_index - 1
        {
            let location = Location::new(expression.span, self.file);
            if let Some(typ) = self.interner.type_at_location(location) {
                let prefix = "";
                let self_prefix = false;
                self.complete_type_fields_and_methods(
                    &typ,
                    prefix,
                    FunctionCompletionKind::NameAndParameters,
                    self_prefix,
                );
            }
        }

        false
    }

    fn visit_comptime_expression(
        &mut self,
        block_expression: &BlockExpression,
        span: Span,
    ) -> bool {
        // When entering a comptime block, regular local variables shouldn't be offered anymore
        let old_local_variables = self.local_variables.clone();
        self.local_variables.clear();

        let old_in_comptime = self.in_comptime;
        self.in_comptime = true;

        block_expression.accept(Some(span), self);

        self.in_comptime = old_in_comptime;
        self.local_variables = old_local_variables;

        false
    }

    fn visit_constructor_expression(
        &mut self,
        constructor_expression: &ConstructorExpression,
        _: Span,
    ) -> bool {
        let UnresolvedTypeData::Named(path, _, _) = &constructor_expression.typ.typ else {
            return true;
        };

        self.find_in_path(path, RequestedItems::OnlyTypes);

        // Check if we need to autocomplete the field name
        if constructor_expression
            .fields
            .iter()
            .any(|(field_name, _)| field_name.span().end() as usize == self.byte_index)
        {
            self.complete_constructor_field_name(constructor_expression);
            return false;
        }

        for (_field_name, expression) in &constructor_expression.fields {
            expression.accept(self);
        }

        false
    }

    fn visit_member_access_expression(
        &mut self,
        member_access_expression: &MemberAccessExpression,
        _: Span,
    ) -> bool {
        let ident = &member_access_expression.rhs;

        if self.byte_index == ident.span().end() as usize {
            // Assuming member_access_expression is of the form `foo.bar`, we are right after `bar`
            let location = Location::new(member_access_expression.lhs.span, self.file);
            if let Some(typ) = self.interner.type_at_location(location) {
                let prefix = ident.to_string().to_case(Case::Snake);
                let self_prefix = false;
                self.complete_type_fields_and_methods(
                    &typ,
                    &prefix,
                    FunctionCompletionKind::NameAndParameters,
                    self_prefix,
                );
                return false;
            }
        }

        true
    }

    fn visit_if_expression(&mut self, if_expression: &IfExpression, _: Span) -> bool {
        if_expression.condition.accept(self);

        let old_local_variables = self.local_variables.clone();
        if_expression.consequence.accept(self);
        self.local_variables = old_local_variables;

        if let Some(alternative) = &if_expression.alternative {
            let old_local_variables = self.local_variables.clone();
            alternative.accept(self);
            self.local_variables = old_local_variables;
        }

        false
    }

    fn visit_lambda(&mut self, lambda: &Lambda, _: Span) -> bool {
        for (_, unresolved_type) in &lambda.parameters {
            unresolved_type.accept(self);
        }

        let old_local_variables = self.local_variables.clone();
        for (pattern, _) in &lambda.parameters {
            self.collect_local_variables(pattern);
        }

        lambda.body.accept(self);

        self.local_variables = old_local_variables;

        false
    }

    fn visit_as_trait_path(&mut self, as_trait_path: &AsTraitPath, _: Span) -> bool {
        self.find_in_path(&as_trait_path.trait_path, RequestedItems::OnlyTypes);

        false
    }

    fn visit_unresolved_type(&mut self, unresolved_type: &UnresolvedType) -> bool {
        self.includes_span(unresolved_type.span)
    }

    fn visit_named_type(
        &mut self,
        path: &Path,
        unresolved_types: &GenericTypeArgs,
        _: Span,
    ) -> bool {
        self.find_in_path(path, RequestedItems::OnlyTypes);
        unresolved_types.accept(self);
        false
    }

    fn visit_type_path(&mut self, type_path: &TypePath, _: Span) -> bool {
        if type_path.item.span().end() as usize != self.byte_index {
            return true;
        }

        let typ = match &type_path.typ.typ {
            UnresolvedTypeData::FieldElement => Some(Type::FieldElement),
            UnresolvedTypeData::Integer(signedness, integer_bit_size) => {
                Some(Type::Integer(*signedness, *integer_bit_size))
            }
            UnresolvedTypeData::Bool => Some(Type::Bool),
            UnresolvedTypeData::String(UnresolvedTypeExpression::Constant(value, _)) => {
                Some(Type::String(Box::new(Type::Constant(
                    *value,
                    Kind::Numeric(Box::new(Type::Integer(
                        Signedness::Unsigned,
                        IntegerBitSize::ThirtyTwo,
                    ))),
                ))))
            }
            UnresolvedTypeData::Quoted(quoted_type) => Some(Type::Quoted(*quoted_type)),
            _ => None,
        };

        if let Some(typ) = typ {
            let prefix = &type_path.item.0.contents;
            self.complete_type_methods(
                &typ,
                prefix,
                FunctionKind::Any,
                FunctionCompletionKind::NameAndParameters,
                false, // self_prefix
            );
        }

        false
    }

    fn visit_meta_attribute(&mut self, attribute: &MetaAttribute, target: AttributeTarget) -> bool {
        if self.byte_index == attribute.name.span.end() as usize {
            self.suggest_builtin_attributes(&attribute.name.to_string(), target);
        }

        self.find_in_path(&attribute.name, RequestedItems::OnlyAttributeFunctions(target));

        true
    }

    fn visit_quote(&mut self, tokens: &Tokens) {
        let mut last_was_dollar = false;

        for token in &tokens.0 {
            let span = token.to_span();
            if span.end() as usize > self.byte_index {
                break;
            }

            let token = token.token();

            if let Token::DollarSign = token {
                if span.end() as usize == self.byte_index {
                    self.local_variables_completion("");
                    break;
                }

                last_was_dollar = true;
                continue;
            }

            if span.end() as usize == self.byte_index {
                let prefix = token.to_string();
                if last_was_dollar {
                    self.local_variables_completion(&prefix);
                }
                break;
            }

            last_was_dollar = false;
        }
    }

    fn visit_trait_bound(&mut self, trait_bound: &TraitBound) -> bool {
        self.find_in_path(&trait_bound.trait_path, RequestedItems::OnlyTraits);
        trait_bound.trait_generics.accept(self);
        false
    }
}

fn get_field_type(typ: &Type, name: &str) -> Option<Type> {
    match typ {
        Type::Struct(struct_type, generics) => {
            Some(struct_type.borrow().get_field(name, generics)?.0)
        }
        Type::Tuple(types) => {
            if let Ok(index) = name.parse::<i32>() {
                types.get(index as usize).cloned()
            } else {
                None
            }
        }
        Type::Alias(alias_type, generics) => Some(alias_type.borrow().get_type(generics)),
        Type::TypeVariable(var) | Type::NamedGeneric(var, _) => {
            if let TypeBinding::Bound(ref typ) = &*var.borrow() {
                get_field_type(typ, name)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_array_element_type(typ: Type) -> Option<Type> {
    match typ {
        Type::Array(_, typ) | Type::Slice(typ) => Some(*typ),
        Type::Alias(alias_type, generics) => {
            let typ = alias_type.borrow().get_type(&generics);
            get_array_element_type(typ)
        }
        Type::TypeVariable(var) | Type::NamedGeneric(var, _) => {
            if let TypeBinding::Bound(typ) = &*var.borrow() {
                get_array_element_type(typ.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_type_struct_id(typ: &Type) -> Option<StructId> {
    match typ {
        Type::Struct(struct_type, _) => Some(struct_type.borrow().id),
        Type::Alias(type_alias, generics) => {
            let type_alias = type_alias.borrow();
            let typ = type_alias.get_type(generics);
            get_type_struct_id(&typ)
        }
        _ => None,
    }
}

/// Returns true if name matches a prefix written in code.
/// `prefix` must already be in snake case.
/// This method splits both name and prefix by underscore,
/// then checks that every part of name starts with a part of
/// prefix, in order.
///
/// For example:
///
/// // "merk" and "ro" match "merkle" and "root" and are in order
/// name_matches("compute_merkle_root", "merk_ro") == true
///
/// // "ro" matches "root", but "merkle" comes before it, so no match
/// name_matches("compute_merkle_root", "ro_mer") == false
///
/// // neither "compute" nor "merkle" nor "root" start with "oot"
/// name_matches("compute_merkle_root", "oot") == false
fn name_matches(name: &str, prefix: &str) -> bool {
    let name = name.to_case(Case::Snake);
    let name_parts: Vec<&str> = name.split('_').collect();

    let mut last_index: i32 = -1;
    for prefix_part in prefix.split('_') {
        // Look past parts we already matched
        let offset = if last_index >= 0 { last_index as usize + 1 } else { 0 };

        if let Some(mut name_part_index) =
            name_parts.iter().skip(offset).position(|name_part| name_part.starts_with(prefix_part))
        {
            // Need to adjust the index if we skipped some segments
            name_part_index += offset;

            if last_index >= name_part_index as i32 {
                return false;
            }
            last_index = name_part_index as i32;
        } else {
            return false;
        }
    }

    true
}

fn module_def_id_from_reference_id(reference_id: ReferenceId) -> Option<ModuleDefId> {
    match reference_id {
        ReferenceId::Module(module_id) => Some(ModuleDefId::ModuleId(module_id)),
        ReferenceId::Struct(struct_id) => Some(ModuleDefId::TypeId(struct_id)),
        ReferenceId::Trait(trait_id) => Some(ModuleDefId::TraitId(trait_id)),
        ReferenceId::Function(func_id) => Some(ModuleDefId::FunctionId(func_id)),
        ReferenceId::Alias(type_alias_id) => Some(ModuleDefId::TypeAliasId(type_alias_id)),
        ReferenceId::StructMember(_, _)
        | ReferenceId::Global(_)
        | ReferenceId::Local(_)
        | ReferenceId::Reference(_, _) => None,
    }
}

#[cfg(test)]
mod completion_name_matches_tests {
    use crate::requests::completion::name_matches;

    #[test]
    fn test_name_matches() {
        assert!(name_matches("foo", "foo"));
        assert!(name_matches("foo_bar", "bar"));
        assert!(name_matches("FooBar", "foo"));
        assert!(name_matches("FooBar", "bar"));
        assert!(name_matches("FooBar", "foo_bar"));
        assert!(name_matches("bar_baz", "bar_b"));

        assert!(!name_matches("foo_bar", "o_b"));
    }
}
