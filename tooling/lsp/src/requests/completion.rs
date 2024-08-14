use std::{
    collections::{BTreeMap, HashMap, HashSet},
    future::{self, Future},
};

use async_lsp::ResponseError;
use builtins::{builtin_integer_types, keyword_builtin_function, keyword_builtin_type};
use fm::{FileId, PathString};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionParams,
    CompletionResponse, InsertTextFormat,
};
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ast::{
        ArrayLiteral, AsTraitPath, BlockExpression, CallExpression, CastExpression,
        ConstrainStatement, ConstructorExpression, Expression, ForLoopStatement, ForRange,
        FunctionReturnType, Ident, IfExpression, IndexExpression, InfixExpression, LValue, Lambda,
        LetStatement, Literal, MemberAccessExpression, MethodCallExpression, NoirFunction,
        NoirStruct, NoirTrait, NoirTraitImpl, NoirTypeAlias, Path, PathKind, PathSegment, Pattern,
        Statement, TraitImplItem, TraitItem, TypeImpl, UnresolvedGeneric, UnresolvedGenerics,
        UnresolvedType, UseTree, UseTreeKind,
    },
    graph::{CrateId, Dependency},
    hir::{
        def_map::{CrateDefMap, LocalModuleId, ModuleId},
        resolution::path_resolver::{PathResolver, StandardPathResolver},
    },
    hir_def::{function::FuncMeta, stmt::HirPattern},
    macros_api::{ModuleDefId, NodeInterner, StructId},
    node_interner::{FuncId, GlobalId, ReferenceId, TraitId, TypeAliasId},
    parser::{Item, ItemKind},
    token::Keyword,
    ParsedModule, StructType, Type,
};
use strum::IntoEnumIterator;

use crate::{utils, LspState};

use super::process_request;

mod builtins;

/// When finding items in a module, whether to show only direct children or all visible items.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ModuleCompletionKind {
    // Only show a module's direct children. This is used when completing a use statement
    // or a path after the first segment.
    DirectChildren,
    // Show all of a module's visible items. This is used when completing a path outside
    // of a use statement (in regular code) when the path is just a single segment:
    // we want to find items exposed in the current module.
    AllVisibleItems,
}

/// When suggest a function as a result of completion, whether to autocomplete its name or its name and parameters.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FunctionCompletionKind {
    // Only complete a function's name. This is used in use statement.
    Name,
    // Complete a function's name and parameters (as a snippet). This is used in regular code.
    NameAndParameters,
}

/// Is there a requirement for suggesting functions?
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FunctionKind<'a> {
    /// No requirement: any function is okay to suggest.
    Any,
    /// Only show functions that have the given self type.
    SelfType(&'a Type),
}

/// When requesting completions, whether to list all items or just types.
/// For example, when writing `let x: S` we only want to suggest types at this
/// point (modules too, because they might include types too).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RequestedItems {
    // Suggest any items (types, functions, etc.).
    AnyItems,
    // Only suggest types.
    OnlyTypes,
}

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
                    file_id,
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
    file: FileId,
    byte_index: usize,
    byte: Option<u8>,
    /// The module ID of the current file.
    root_module_id: ModuleId,
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
}

impl<'a> NodeFinder<'a> {
    fn new(
        file: FileId,
        byte_index: usize,
        byte: Option<u8>,
        krate: CrateId,
        def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
        dependencies: &'a Vec<Dependency>,
        interner: &'a NodeInterner,
    ) -> Self {
        // Find the module the current file belongs to
        let def_map = &def_maps[&krate];
        let root_module_id = ModuleId { krate, local_id: def_map.root() };
        let local_id = if let Some((module_index, _)) =
            def_map.modules().iter().find(|(_, module_data)| module_data.location.file == file)
        {
            LocalModuleId(module_index)
        } else {
            def_map.root()
        };
        let module_id = ModuleId { krate, local_id };
        Self {
            file,
            byte_index,
            byte,
            root_module_id,
            module_id,
            def_maps,
            dependencies,
            interner,
            completion_items: Vec::new(),
            local_variables: HashMap::new(),
            type_parameters: HashSet::new(),
        }
    }

    fn find(&mut self, parsed_module: &ParsedModule) -> Option<CompletionResponse> {
        self.find_in_parsed_module(parsed_module);

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

    fn find_in_parsed_module(&mut self, parsed_module: &ParsedModule) {
        for item in &parsed_module.items {
            self.find_in_item(item);
        }
    }

    fn find_in_item(&mut self, item: &Item) {
        if !self.includes_span(item.span) {
            return;
        }

        match &item.kind {
            ItemKind::Import(use_tree) => {
                let mut prefixes = Vec::new();
                self.find_in_use_tree(use_tree, &mut prefixes);
            }
            ItemKind::Submodules(parsed_sub_module) => {
                // Switch `self.module_id` to the submodule
                let previous_module_id = self.module_id;

                let def_map = &self.def_maps[&self.module_id.krate];
                let Some(module_data) = def_map.modules().get(self.module_id.local_id.0) else {
                    return;
                };
                if let Some(child_module) = module_data.children.get(&parsed_sub_module.name) {
                    self.module_id =
                        ModuleId { krate: self.module_id.krate, local_id: *child_module };
                }

                self.find_in_parsed_module(&parsed_sub_module.contents);

                // Restore the old module before continuing
                self.module_id = previous_module_id;
            }
            ItemKind::Function(noir_function) => self.find_in_noir_function(noir_function),
            ItemKind::TraitImpl(noir_trait_impl) => self.find_in_noir_trait_impl(noir_trait_impl),
            ItemKind::Impl(type_impl) => self.find_in_type_impl(type_impl),
            ItemKind::Global(let_statement) => self.find_in_let_statement(let_statement, false),
            ItemKind::TypeAlias(noir_type_alias) => self.find_in_noir_type_alias(noir_type_alias),
            ItemKind::Struct(noir_struct) => self.find_in_noir_struct(noir_struct),
            ItemKind::Trait(noir_trait) => self.find_in_noir_trait(noir_trait),
            ItemKind::ModuleDecl(_) => (),
        }
    }

    fn find_in_noir_function(&mut self, noir_function: &NoirFunction) {
        let old_type_parameters = self.type_parameters.clone();
        self.collect_type_parameters_in_generics(&noir_function.def.generics);

        for param in &noir_function.def.parameters {
            self.find_in_unresolved_type(&param.typ);
        }

        self.find_in_function_return_type(&noir_function.def.return_type);

        self.local_variables.clear();
        for param in &noir_function.def.parameters {
            self.collect_local_variables(&param.pattern);
        }

        self.find_in_block_expression(&noir_function.def.body);

        self.type_parameters = old_type_parameters;
    }

    fn find_in_noir_trait_impl(&mut self, noir_trait_impl: &NoirTraitImpl) {
        self.type_parameters.clear();
        self.collect_type_parameters_in_generics(&noir_trait_impl.impl_generics);

        for item in &noir_trait_impl.items {
            self.find_in_trait_impl_item(item);
        }

        self.type_parameters.clear();
    }

    fn find_in_trait_impl_item(&mut self, item: &TraitImplItem) {
        match item {
            TraitImplItem::Function(noir_function) => self.find_in_noir_function(noir_function),
            TraitImplItem::Constant(_, _, _) => (),
            TraitImplItem::Type { .. } => (),
        }
    }

    fn find_in_type_impl(&mut self, type_impl: &TypeImpl) {
        self.type_parameters.clear();
        self.collect_type_parameters_in_generics(&type_impl.generics);

        for (method, span) in &type_impl.methods {
            self.find_in_noir_function(method);

            // Optimization: stop looking in functions past the completion cursor
            if span.end() as usize > self.byte_index {
                break;
            }
        }

        self.type_parameters.clear();
    }

    fn find_in_noir_type_alias(&mut self, noir_type_alias: &NoirTypeAlias) {
        self.find_in_unresolved_type(&noir_type_alias.typ);
    }

    fn find_in_noir_struct(&mut self, noir_struct: &NoirStruct) {
        self.type_parameters.clear();
        self.collect_type_parameters_in_generics(&noir_struct.generics);

        for (_name, unresolved_type) in &noir_struct.fields {
            self.find_in_unresolved_type(unresolved_type);
        }

        self.type_parameters.clear();
    }

    fn find_in_noir_trait(&mut self, noir_trait: &NoirTrait) {
        for item in &noir_trait.items {
            self.find_in_trait_item(item);
        }
    }

    fn find_in_trait_item(&mut self, trait_item: &TraitItem) {
        match trait_item {
            TraitItem::Function {
                name: _,
                generics,
                parameters,
                return_type,
                where_clause,
                body,
            } => {
                let old_type_parameters = self.type_parameters.clone();
                self.collect_type_parameters_in_generics(generics);

                for (_name, unresolved_type) in parameters {
                    self.find_in_unresolved_type(unresolved_type);
                }

                self.find_in_function_return_type(return_type);

                for unresolved_trait_constraint in where_clause {
                    self.find_in_unresolved_type(&unresolved_trait_constraint.typ);
                }

                if let Some(body) = body {
                    self.local_variables.clear();
                    for (name, _) in parameters {
                        self.local_variables.insert(name.to_string(), name.span());
                    }
                    self.find_in_block_expression(body);
                };

                self.type_parameters = old_type_parameters;
            }
            TraitItem::Constant { name: _, typ, default_value } => {
                self.find_in_unresolved_type(typ);

                if let Some(default_value) = default_value {
                    self.find_in_expression(default_value);
                }
            }
            TraitItem::Type { name: _ } => (),
        }
    }

    fn find_in_block_expression(&mut self, block_expression: &BlockExpression) {
        let old_local_variables = self.local_variables.clone();
        for statement in &block_expression.statements {
            self.find_in_statement(statement);

            // Optimization: stop looking in statements past the completion cursor
            if statement.span.end() as usize > self.byte_index {
                break;
            }
        }
        self.local_variables = old_local_variables;
    }

    fn find_in_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            noirc_frontend::ast::StatementKind::Let(let_statement) => {
                self.find_in_let_statement(let_statement, true);
            }
            noirc_frontend::ast::StatementKind::Constrain(constrain_statement) => {
                self.find_in_constrain_statement(constrain_statement);
            }
            noirc_frontend::ast::StatementKind::Expression(expression) => {
                self.find_in_expression(expression);
            }
            noirc_frontend::ast::StatementKind::Assign(assign_statement) => {
                self.find_in_assign_statement(assign_statement);
            }
            noirc_frontend::ast::StatementKind::For(for_loop_statement) => {
                self.find_in_for_loop_statement(for_loop_statement);
            }
            noirc_frontend::ast::StatementKind::Comptime(statement) => {
                // When entering a comptime block, regular local variables shouldn't be offered anymore
                let old_local_variables = self.local_variables.clone();
                self.local_variables.clear();

                self.find_in_statement(statement);

                self.local_variables = old_local_variables;
            }
            noirc_frontend::ast::StatementKind::Semi(expression) => {
                self.find_in_expression(expression);
            }
            noirc_frontend::ast::StatementKind::Break
            | noirc_frontend::ast::StatementKind::Continue
            | noirc_frontend::ast::StatementKind::Error => (),
        }
    }

    fn find_in_let_statement(
        &mut self,
        let_statement: &LetStatement,
        collect_local_variables: bool,
    ) {
        self.find_in_unresolved_type(&let_statement.r#type);
        self.find_in_expression(&let_statement.expression);

        if collect_local_variables {
            self.collect_local_variables(&let_statement.pattern);
        }
    }

    fn find_in_constrain_statement(&mut self, constrain_statement: &ConstrainStatement) {
        self.find_in_expression(&constrain_statement.0);

        if let Some(exp) = &constrain_statement.1 {
            self.find_in_expression(exp);
        }
    }

    fn find_in_assign_statement(
        &mut self,
        assign_statement: &noirc_frontend::ast::AssignStatement,
    ) {
        self.find_in_lvalue(&assign_statement.lvalue);
        self.find_in_expression(&assign_statement.expression);
    }

    fn find_in_for_loop_statement(&mut self, for_loop_statement: &ForLoopStatement) {
        let old_local_variables = self.local_variables.clone();
        let ident = &for_loop_statement.identifier;
        self.local_variables.insert(ident.to_string(), ident.span());

        self.find_in_for_range(&for_loop_statement.range);
        self.find_in_expression(&for_loop_statement.block);

        self.local_variables = old_local_variables;
    }

    fn find_in_lvalue(&mut self, lvalue: &LValue) {
        match lvalue {
            LValue::Ident(_) => (),
            LValue::MemberAccess { object, field_name: _, span: _ } => self.find_in_lvalue(object),
            LValue::Index { array, index, span: _ } => {
                self.find_in_lvalue(array);
                self.find_in_expression(index);
            }
            LValue::Dereference(lvalue, _) => self.find_in_lvalue(lvalue),
        }
    }

    fn find_in_for_range(&mut self, for_range: &ForRange) {
        match for_range {
            ForRange::Range(start, end) => {
                self.find_in_expression(start);
                self.find_in_expression(end);
            }
            ForRange::Array(expression) => self.find_in_expression(expression),
        }
    }

    fn find_in_expressions(&mut self, expressions: &[Expression]) {
        for expression in expressions {
            self.find_in_expression(expression);
        }
    }

    fn find_in_expression(&mut self, expression: &Expression) {
        // "foo." (no identifier afterwards) is parsed as the expression on the left hand-side of the dot.
        // Here we check if there's a dot at the completion position, and if the expression
        // ends right before the dot. If so, it means we want to complete the expression's type fields and methods.
        if self.byte == Some(b'.') && expression.span.end() as usize == self.byte_index - 1 {
            let location = Location::new(expression.span, self.file);
            if let Some(typ) = self.interner.type_at_location(location) {
                let typ = typ.follow_bindings();
                let prefix = "";
                self.complete_type_fields_and_methods(&typ, prefix);
                return;
            }
        }

        match &expression.kind {
            noirc_frontend::ast::ExpressionKind::Literal(literal) => self.find_in_literal(literal),
            noirc_frontend::ast::ExpressionKind::Block(block_expression) => {
                self.find_in_block_expression(block_expression);
            }
            noirc_frontend::ast::ExpressionKind::Prefix(prefix_expression) => {
                self.find_in_expression(&prefix_expression.rhs);
            }
            noirc_frontend::ast::ExpressionKind::Index(index_expression) => {
                self.find_in_index_expression(index_expression);
            }
            noirc_frontend::ast::ExpressionKind::Call(call_expression) => {
                self.find_in_call_expression(call_expression);
            }
            noirc_frontend::ast::ExpressionKind::MethodCall(method_call_expression) => {
                self.find_in_method_call_expression(method_call_expression);
            }
            noirc_frontend::ast::ExpressionKind::Constructor(constructor_expression) => {
                self.find_in_constructor_expression(constructor_expression);
            }
            noirc_frontend::ast::ExpressionKind::MemberAccess(member_access_expression) => {
                self.find_in_member_access_expression(member_access_expression);
            }
            noirc_frontend::ast::ExpressionKind::Cast(cast_expression) => {
                self.find_in_cast_expression(cast_expression);
            }
            noirc_frontend::ast::ExpressionKind::Infix(infix_expression) => {
                self.find_in_infix_expression(infix_expression);
            }
            noirc_frontend::ast::ExpressionKind::If(if_expression) => {
                self.find_in_if_expression(if_expression);
            }
            noirc_frontend::ast::ExpressionKind::Variable(path) => {
                self.find_in_path(path, RequestedItems::AnyItems);
            }
            noirc_frontend::ast::ExpressionKind::Tuple(expressions) => {
                self.find_in_expressions(expressions);
            }
            noirc_frontend::ast::ExpressionKind::Lambda(lambda) => self.find_in_lambda(lambda),
            noirc_frontend::ast::ExpressionKind::Parenthesized(expression) => {
                self.find_in_expression(expression);
            }
            noirc_frontend::ast::ExpressionKind::Unquote(expression) => {
                self.find_in_expression(expression);
            }
            noirc_frontend::ast::ExpressionKind::Comptime(block_expression, _) => {
                // When entering a comptime block, regular local variables shouldn't be offered anymore
                let old_local_variables = self.local_variables.clone();
                self.local_variables.clear();

                self.find_in_block_expression(block_expression);

                self.local_variables = old_local_variables;
            }
            noirc_frontend::ast::ExpressionKind::AsTraitPath(as_trait_path) => {
                self.find_in_as_trait_path(as_trait_path);
            }
            noirc_frontend::ast::ExpressionKind::Quote(_)
            | noirc_frontend::ast::ExpressionKind::Resolved(_)
            | noirc_frontend::ast::ExpressionKind::Error => (),
        }
    }

    fn find_in_literal(&mut self, literal: &Literal) {
        match literal {
            Literal::Array(array_literal) => self.find_in_array_literal(array_literal),
            Literal::Slice(array_literal) => self.find_in_array_literal(array_literal),
            Literal::Bool(_)
            | Literal::Integer(_, _)
            | Literal::Str(_)
            | Literal::RawStr(_, _)
            | Literal::FmtStr(_)
            | Literal::Unit => (),
        }
    }

    fn find_in_array_literal(&mut self, array_literal: &ArrayLiteral) {
        match array_literal {
            ArrayLiteral::Standard(expressions) => self.find_in_expressions(expressions),
            ArrayLiteral::Repeated { repeated_element, length } => {
                self.find_in_expression(repeated_element);
                self.find_in_expression(length);
            }
        }
    }

    fn find_in_index_expression(&mut self, index_expression: &IndexExpression) {
        self.find_in_expression(&index_expression.collection);
        self.find_in_expression(&index_expression.index);
    }

    fn find_in_call_expression(&mut self, call_expression: &CallExpression) {
        self.find_in_expression(&call_expression.func);
        self.find_in_expressions(&call_expression.arguments);
    }

    fn find_in_method_call_expression(&mut self, method_call_expression: &MethodCallExpression) {
        self.find_in_expression(&method_call_expression.object);
        self.find_in_expressions(&method_call_expression.arguments);
    }

    fn find_in_constructor_expression(&mut self, constructor_expression: &ConstructorExpression) {
        self.find_in_path(&constructor_expression.type_name, RequestedItems::OnlyTypes);

        for (_field_name, expression) in &constructor_expression.fields {
            self.find_in_expression(expression);
        }
    }

    fn find_in_member_access_expression(
        &mut self,
        member_access_expression: &MemberAccessExpression,
    ) {
        let ident = &member_access_expression.rhs;

        if self.byte_index == ident.span().end() as usize {
            // Assuming member_access_expression is of the form `foo.bar`, we are right after `bar`
            let location = Location::new(member_access_expression.lhs.span, self.file);
            if let Some(typ) = self.interner.type_at_location(location) {
                let typ = typ.follow_bindings();
                let prefix = ident.to_string();
                self.complete_type_fields_and_methods(&typ, &prefix);
                return;
            }
        }

        self.find_in_expression(&member_access_expression.lhs);
    }

    fn find_in_cast_expression(&mut self, cast_expression: &CastExpression) {
        self.find_in_expression(&cast_expression.lhs);
    }

    fn find_in_infix_expression(&mut self, infix_expression: &InfixExpression) {
        self.find_in_expression(&infix_expression.lhs);
        self.find_in_expression(&infix_expression.rhs);
    }

    fn find_in_if_expression(&mut self, if_expression: &IfExpression) {
        self.find_in_expression(&if_expression.condition);

        let old_local_variables = self.local_variables.clone();
        self.find_in_expression(&if_expression.consequence);
        self.local_variables = old_local_variables;

        if let Some(alternative) = &if_expression.alternative {
            let old_local_variables = self.local_variables.clone();
            self.find_in_expression(alternative);
            self.local_variables = old_local_variables;
        }
    }

    fn find_in_lambda(&mut self, lambda: &Lambda) {
        for (_, unresolved_type) in &lambda.parameters {
            self.find_in_unresolved_type(unresolved_type);
        }

        let old_local_variables = self.local_variables.clone();
        for (pattern, _) in &lambda.parameters {
            self.collect_local_variables(pattern);
        }

        self.find_in_expression(&lambda.body);

        self.local_variables = old_local_variables;
    }

    fn find_in_as_trait_path(&mut self, as_trait_path: &AsTraitPath) {
        self.find_in_path(&as_trait_path.trait_path, RequestedItems::OnlyTypes);
    }

    fn find_in_function_return_type(&mut self, return_type: &FunctionReturnType) {
        match return_type {
            noirc_frontend::ast::FunctionReturnType::Default(_) => (),
            noirc_frontend::ast::FunctionReturnType::Ty(unresolved_type) => {
                self.find_in_unresolved_type(unresolved_type);
            }
        }
    }

    fn find_in_unresolved_types(&mut self, unresolved_type: &[UnresolvedType]) {
        for unresolved_type in unresolved_type {
            self.find_in_unresolved_type(unresolved_type);
        }
    }

    fn find_in_unresolved_type(&mut self, unresolved_type: &UnresolvedType) {
        if let Some(span) = unresolved_type.span {
            if !self.includes_span(span) {
                return;
            }
        }

        match &unresolved_type.typ {
            noirc_frontend::ast::UnresolvedTypeData::Array(_, unresolved_type) => {
                self.find_in_unresolved_type(unresolved_type);
            }
            noirc_frontend::ast::UnresolvedTypeData::Slice(unresolved_type) => {
                self.find_in_unresolved_type(unresolved_type);
            }
            noirc_frontend::ast::UnresolvedTypeData::Parenthesized(unresolved_type) => {
                self.find_in_unresolved_type(unresolved_type);
            }
            noirc_frontend::ast::UnresolvedTypeData::Named(path, unresolved_types, _) => {
                self.find_in_path(path, RequestedItems::OnlyTypes);
                self.find_in_unresolved_types(unresolved_types);
            }
            noirc_frontend::ast::UnresolvedTypeData::TraitAsType(path, unresolved_types) => {
                self.find_in_path(path, RequestedItems::OnlyTypes);
                self.find_in_unresolved_types(unresolved_types);
            }
            noirc_frontend::ast::UnresolvedTypeData::MutableReference(unresolved_type) => {
                self.find_in_unresolved_type(unresolved_type);
            }
            noirc_frontend::ast::UnresolvedTypeData::Tuple(unresolved_types) => {
                self.find_in_unresolved_types(unresolved_types);
            }
            noirc_frontend::ast::UnresolvedTypeData::Function(args, ret, env) => {
                self.find_in_unresolved_types(args);
                self.find_in_unresolved_type(ret);
                self.find_in_unresolved_type(env);
            }
            noirc_frontend::ast::UnresolvedTypeData::AsTraitPath(as_trait_path) => {
                self.find_in_as_trait_path(as_trait_path);
            }
            noirc_frontend::ast::UnresolvedTypeData::Expression(_)
            | noirc_frontend::ast::UnresolvedTypeData::FormatString(_, _)
            | noirc_frontend::ast::UnresolvedTypeData::String(_)
            | noirc_frontend::ast::UnresolvedTypeData::Unspecified
            | noirc_frontend::ast::UnresolvedTypeData::Quoted(_)
            | noirc_frontend::ast::UnresolvedTypeData::FieldElement
            | noirc_frontend::ast::UnresolvedTypeData::Integer(_, _)
            | noirc_frontend::ast::UnresolvedTypeData::Bool
            | noirc_frontend::ast::UnresolvedTypeData::Unit
            | noirc_frontend::ast::UnresolvedTypeData::Resolved(_)
            | noirc_frontend::ast::UnresolvedTypeData::Error => (),
        }
    }

    fn find_in_path(&mut self, path: &Path, requested_items: RequestedItems) {
        // Only offer completions if we are right at the end of the path
        if self.byte_index != path.span.end() as usize {
            return;
        }

        let after_colons = self.byte == Some(b':');

        let mut idents: Vec<Ident> =
            path.segments.iter().map(|segment| segment.ident.clone()).collect();
        let prefix;
        let at_root;

        if after_colons {
            prefix = String::new();
            at_root = false;
        } else {
            prefix = idents.pop().unwrap().to_string();
            at_root = idents.is_empty();
        }

        let is_single_segment = !after_colons && idents.is_empty() && path.kind == PathKind::Plain;
        let module_id;

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
                    self.complete_type_methods(&type_alias.typ, &prefix, FunctionKind::Any);
                    return;
                }
                ModuleDefId::TraitId(_) => {
                    // For now we don't suggest trait methods
                    return;
                }
                ModuleDefId::GlobalId(_) => return,
            }
        }

        let module_completion_kind = if after_colons {
            ModuleCompletionKind::DirectChildren
        } else {
            ModuleCompletionKind::AllVisibleItems
        };
        let function_completion_kind = FunctionCompletionKind::NameAndParameters;

        self.complete_in_module(
            module_id,
            &prefix,
            path.kind,
            at_root,
            module_completion_kind,
            function_completion_kind,
            requested_items,
        );

        if is_single_segment {
            match requested_items {
                RequestedItems::AnyItems => {
                    self.local_variables_completion(&prefix);
                    self.builtin_functions_completion(&prefix);
                    self.builtin_values_completion(&prefix);
                }
                RequestedItems::OnlyTypes => {
                    self.builtin_types_completion(&prefix);
                    self.type_parameters_completion(&prefix);
                }
            }
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

        let module_completion_kind = ModuleCompletionKind::DirectChildren;
        let function_completion_kind = FunctionCompletionKind::Name;
        let requested_items = RequestedItems::AnyItems;

        if after_colons {
            // We are right after "::"
            segments.push(ident.clone());

            if let Some(module_id) = self.resolve_module(segments) {
                let prefix = String::new();
                let at_root = false;
                self.complete_in_module(
                    module_id,
                    &prefix,
                    path_kind,
                    at_root,
                    module_completion_kind,
                    function_completion_kind,
                    requested_items,
                );
            };
        } else {
            // We are right after the last segment
            let prefix = ident.to_string();
            if segments.is_empty() {
                let at_root = true;
                self.complete_in_module(
                    self.module_id,
                    &prefix,
                    path_kind,
                    at_root,
                    module_completion_kind,
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
                    module_completion_kind,
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

    fn complete_type_fields_and_methods(&mut self, typ: &Type, prefix: &str) {
        match typ {
            Type::Struct(struct_type, generics) => {
                self.complete_struct_fields(&struct_type.borrow(), generics, prefix);
            }
            Type::MutableReference(typ) => {
                return self.complete_type_fields_and_methods(typ, prefix);
            }
            Type::Alias(type_alias, _) => {
                let type_alias = type_alias.borrow();
                return self.complete_type_fields_and_methods(&type_alias.typ, prefix);
            }
            Type::FieldElement
            | Type::Array(_, _)
            | Type::Slice(_)
            | Type::Integer(_, _)
            | Type::Bool
            | Type::String(_)
            | Type::FmtString(_, _)
            | Type::Unit
            | Type::Tuple(_)
            | Type::TypeVariable(_, _)
            | Type::TraitAsType(_, _, _)
            | Type::NamedGeneric(_, _, _)
            | Type::Function(_, _, _)
            | Type::Forall(_, _)
            | Type::Constant(_)
            | Type::Quoted(_)
            | Type::InfixExpr(_, _, _)
            | Type::Error => (),
        }

        self.complete_type_methods(typ, prefix, FunctionKind::SelfType(typ));
    }

    fn complete_type_methods(&mut self, typ: &Type, prefix: &str, function_kind: FunctionKind) {
        let Some(methods_by_name) = self.interner.get_type_methods(typ) else {
            return;
        };

        for (name, methods) in methods_by_name {
            for func_id in methods.iter() {
                if name_matches(name, prefix) {
                    if let Some(completion_item) = self.function_completion_item(
                        func_id,
                        FunctionCompletionKind::NameAndParameters,
                        function_kind,
                    ) {
                        self.completion_items.push(completion_item);
                    }
                }
            }
        }
    }

    fn complete_struct_fields(
        &mut self,
        struct_type: &StructType,
        generics: &[Type],
        prefix: &str,
    ) {
        for (name, typ) in struct_type.get_fields(generics) {
            if name_matches(&name, prefix) {
                self.completion_items.push(simple_completion_item(
                    name,
                    CompletionItemKind::FIELD,
                    Some(typ.to_string()),
                ));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn complete_in_module(
        &mut self,
        module_id: ModuleId,
        prefix: &str,
        path_kind: PathKind,
        at_root: bool,
        module_completion_kind: ModuleCompletionKind,
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

        let items = match module_completion_kind {
            ModuleCompletionKind::DirectChildren => module_data.definitions(),
            ModuleCompletionKind::AllVisibleItems => module_data.scope(),
        };

        for ident in items.names() {
            let name = &ident.0.contents;

            if name_matches(name, prefix) {
                let per_ns = module_data.find_name(ident);
                if let Some((module_def_id, _, _)) = per_ns.types {
                    if let Some(completion_item) = self.module_def_id_completion_item(
                        module_def_id,
                        name.clone(),
                        function_completion_kind,
                        function_kind,
                        requested_items,
                    ) {
                        self.completion_items.push(completion_item);
                    }
                }

                if let Some((module_def_id, _, _)) = per_ns.values {
                    if let Some(completion_item) = self.module_def_id_completion_item(
                        module_def_id,
                        name.clone(),
                        function_completion_kind,
                        function_kind,
                        requested_items,
                    ) {
                        self.completion_items.push(completion_item);
                    }
                }
            }
        }

        if at_root && path_kind == PathKind::Plain {
            for dependency in self.dependencies {
                let dependency_name = dependency.as_name();
                if name_matches(&dependency_name, prefix) {
                    self.completion_items.push(crate_completion_item(dependency_name));
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

    fn module_def_id_completion_item(
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

    fn function_completion_item(
        &self,
        func_id: FuncId,
        function_completion_kind: FunctionCompletionKind,
        function_kind: FunctionKind,
    ) -> Option<CompletionItem> {
        let func_meta = self.interner.function_meta(&func_id);
        let name = &self.interner.function_name(&func_id).to_string();

        let func_self_type = if let Some((pattern, typ, _)) = func_meta.parameters.0.get(0) {
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

    fn resolve_module(&self, segments: Vec<Ident>) -> Option<ModuleId> {
        if let Some(ModuleDefId::ModuleId(module_id)) = self.resolve_path(segments) {
            Some(module_id)
        } else {
            None
        }
    }

    fn resolve_path(&self, segments: Vec<Ident>) -> Option<ModuleDefId> {
        let last_segment = segments.last().unwrap().clone();

        let path_segments = segments.into_iter().map(PathSegment::from).collect();
        let path = Path { segments: path_segments, kind: PathKind::Plain, span: Span::default() };

        let path_resolver = StandardPathResolver::new(self.root_module_id);
        if let Ok(path_resolution) = path_resolver.resolve(self.def_maps, path, &mut None) {
            return Some(path_resolution.module_def_id);
        }

        // If we can't resolve a path trough lookup, let's see if the last segment is bound to a type
        let location = Location::new(last_segment.span(), self.file);
        if let Some(reference_id) = self.interner.find_referenced(location) {
            if let Some(id) = module_def_id_from_reference_id(reference_id) {
                return Some(id);
            }
        }

        None
    }

    fn builtin_functions_completion(&mut self, prefix: &str) {
        for keyword in Keyword::iter() {
            if let Some(func) = keyword_builtin_function(&keyword) {
                if name_matches(func.name, prefix) {
                    self.completion_items.push(snippet_completion_item(
                        format!("{}(…)", func.name),
                        CompletionItemKind::FUNCTION,
                        format!("{}({})", func.name, func.parameters),
                        Some(func.description.to_string()),
                    ));
                }
            }
        }
    }

    fn builtin_values_completion(&mut self, prefix: &str) {
        for keyword in ["false", "true"] {
            if name_matches(keyword, prefix) {
                self.completion_items.push(simple_completion_item(
                    keyword,
                    CompletionItemKind::KEYWORD,
                    Some("bool".to_string()),
                ));
            }
        }
    }

    fn builtin_types_completion(&mut self, prefix: &str) {
        for keyword in Keyword::iter() {
            if let Some(typ) = keyword_builtin_type(&keyword) {
                if name_matches(typ, prefix) {
                    self.completion_items.push(simple_completion_item(
                        typ,
                        CompletionItemKind::STRUCT,
                        Some(typ.to_string()),
                    ));
                }
            }
        }

        for typ in builtin_integer_types() {
            if name_matches(typ, prefix) {
                self.completion_items.push(simple_completion_item(
                    typ,
                    CompletionItemKind::STRUCT,
                    Some(typ.to_string()),
                ));
            }
        }
    }

    fn includes_span(&self, span: Span) -> bool {
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}

fn name_matches(name: &str, prefix: &str) -> bool {
    name.starts_with(prefix)
}

fn module_completion_item(name: impl Into<String>) -> CompletionItem {
    simple_completion_item(name, CompletionItemKind::MODULE, None)
}

fn crate_completion_item(name: impl Into<String>) -> CompletionItem {
    simple_completion_item(name, CompletionItemKind::MODULE, None)
}

fn simple_completion_item(
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

fn snippet_completion_item(
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

fn completion_item_with_sort_text(
    completion_item: CompletionItem,
    sort_text: String,
) -> CompletionItem {
    CompletionItem { sort_text: Some(sort_text), ..completion_item }
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

fn func_meta_type_to_string(func_meta: &FuncMeta, has_self_type: bool) -> String {
    let mut typ = &func_meta.typ;
    if let Type::Forall(_, typ_) = typ {
        typ = typ_;
    }

    if let Type::Function(args, ret, _env) = typ {
        let mut string = String::new();
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

/// Sort text for "new" methods: we want these to show up before anything else,
/// if we are completing at something like `Foo::`
fn new_sort_text() -> String {
    "3".to_string()
}

/// This is the default sort text.
fn default_sort_text() -> String {
    "5".to_string()
}

/// When completing something like `Foo::`, we want to show methods that take
/// self after the other ones.
fn self_mismatch_sort_text() -> String {
    "7".to_string()
}

/// We want to show operator methods last.
fn operator_sort_text() -> String {
    "8".to_string()
}

/// If a name begins with underscore it's likely something that's meant to
/// be private (but visibility doesn't exist everywhere yet, so for now
/// we assume that)
fn underscore_sort_text() -> String {
    "9".to_string()
}

#[cfg(test)]
mod completion_tests {
    use crate::{notifications::on_did_open_text_document, test_utils};

    use super::*;
    use lsp_types::{
        DidOpenTextDocumentParams, PartialResultParams, Position, TextDocumentIdentifier,
        TextDocumentItem, TextDocumentPositionParams, WorkDoneProgressParams,
    };
    use tokio::test;

    async fn assert_completion(src: &str, expected: Vec<CompletionItem>) {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("document_symbol").await;

        let (line, column) = src
            .lines()
            .enumerate()
            .filter_map(|(line_index, line)| {
                line.find(">|<").map(|char_index| (line_index, char_index))
            })
            .next()
            .expect("Expected to find one >|< in the source code");

        let src = src.replace(">|<", "");

        on_did_open_text_document(
            &mut state,
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: noir_text_document.clone(),
                    language_id: "noir".to_string(),
                    version: 0,
                    text: src.to_string(),
                },
            },
        );

        // Get inlay hints. These should now be relative to the changed text,
        // not the saved file's text.
        let response = on_completion_request(
            &mut state,
            CompletionParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: noir_text_document },
                    position: Position { line: line as u32, character: column as u32 },
                },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                partial_result_params: PartialResultParams { partial_result_token: None },
                context: None,
            },
        )
        .await
        .expect("Could not execute on_completion_request")
        .unwrap();

        let CompletionResponse::Array(items) = response else {
            panic!("Expected response to be CompletionResponse::Array");
        };

        let mut items = items.clone();
        items.sort_by_key(|item| item.label.clone());

        let mut expected = expected.clone();
        expected.sort_by_key(|item| item.label.clone());

        if items != expected {
            println!(
                "Items: {:?}",
                items.iter().map(|item| item.label.clone()).collect::<Vec<_>>()
            );
            println!(
                "Expected: {:?}",
                expected.iter().map(|item| item.label.clone()).collect::<Vec<_>>()
            );
        }

        assert_eq!(items, expected);
    }

    #[test]
    async fn test_use_first_segment() {
        let src = r#"
            mod foo {}
            mod foobar {}
            use f>|<
        "#;

        assert_completion(
            src,
            vec![module_completion_item("foo"), module_completion_item("foobar")],
        )
        .await;
    }

    #[test]
    async fn test_use_second_segment() {
        let src = r#"
            mod foo {
                mod bar {}
                mod baz {}
            }
            use foo::>|<
        "#;

        assert_completion(src, vec![module_completion_item("bar"), module_completion_item("baz")])
            .await;
    }

    #[test]
    async fn test_use_second_segment_after_typing() {
        let src = r#"
            mod foo {
                mod bar {}
                mod brave {}
            }
            use foo::ba>|<
        "#;

        assert_completion(src, vec![module_completion_item("bar")]).await;
    }

    #[test]
    async fn test_use_struct() {
        let src = r#"
            mod foo {
                struct Foo {}
            }
            use foo::>|<
        "#;

        assert_completion(
            src,
            vec![simple_completion_item(
                "Foo",
                CompletionItemKind::STRUCT,
                Some("Foo".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_use_function() {
        let src = r#"
            mod foo {
                fn bar(x: i32) -> u64 { 0 }
            }
            use foo::>|<
        "#;

        assert_completion(
            src,
            vec![simple_completion_item(
                "bar",
                CompletionItemKind::FUNCTION,
                Some("fn(i32) -> u64".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_use_after_crate_and_letter() {
        // Prove that "std" shows up
        let src = r#"
            use s>|<
        "#;
        assert_completion(src, vec![crate_completion_item("std")]).await;

        // "std" doesn't show up anymore because of the "crate::" prefix
        let src = r#"
            mod something {}
            use crate::s>|<
        "#;
        assert_completion(src, vec![module_completion_item("something")]).await;
    }

    #[test]
    async fn test_use_suggests_hardcoded_crate() {
        let src = r#"
            use c>|<
        "#;

        assert_completion(
            src,
            vec![simple_completion_item("crate::", CompletionItemKind::KEYWORD, None)],
        )
        .await;
    }

    #[test]
    async fn test_use_in_tree_after_letter() {
        let src = r#"
            mod foo {
                mod bar {}
            }
            use foo::{b>|<}
        "#;

        assert_completion(src, vec![module_completion_item("bar")]).await;
    }

    #[test]
    async fn test_use_in_tree_after_colons() {
        let src = r#"
            mod foo {
                mod bar {
                    mod baz {}
                }
            }
            use foo::{bar::>|<}
        "#;

        assert_completion(src, vec![module_completion_item("baz")]).await;
    }

    #[test]
    async fn test_use_in_tree_after_colons_after_another_segment() {
        let src = r#"
            mod foo {
                mod bar {}
                mod qux {}
            }
            use foo::{bar, q>|<}
        "#;

        assert_completion(src, vec![module_completion_item("qux")]).await;
    }

    #[test]
    async fn test_use_in_nested_module() {
        let src = r#"
            mod foo {
                mod something {}

                use s>|<
            }
        "#;

        assert_completion(
            src,
            vec![
                module_completion_item("something"),
                crate_completion_item("std"),
                simple_completion_item("super::", CompletionItemKind::KEYWORD, None),
            ],
        )
        .await;
    }

    #[test]
    async fn test_use_after_super() {
        let src = r#"
            mod foo {}

            mod bar {
                mod something {}

                use super::f>|<
            }
        "#;

        assert_completion(src, vec![module_completion_item("foo")]).await;
    }

    #[test]
    async fn test_use_after_crate_and_letter_nested_in_module() {
        let src = r#"
            mod something {
                mod something_else {}
                use crate::s>|<
            }
            
        "#;
        assert_completion(src, vec![module_completion_item("something")]).await;
    }

    #[test]
    async fn test_use_after_crate_segment_and_letter_nested_in_module() {
        let src = r#"
            mod something {
                mod something_else {}
                use crate::something::s>|<
            }
            
        "#;
        assert_completion(src, vec![module_completion_item("something_else")]).await;
    }

    #[test]
    async fn test_complete_path_shows_module() {
        let src = r#"
          mod foobar {}

          fn main() {
            fo>|<
          }
        "#;
        assert_completion(src, vec![module_completion_item("foobar")]).await;
    }

    #[test]
    async fn test_complete_path_after_colons_shows_submodule() {
        let src = r#"
          mod foo {
            mod bar {}
          }

          fn main() {
            foo::>|<
          }
        "#;
        assert_completion(src, vec![module_completion_item("bar")]).await;
    }

    #[test]
    async fn test_complete_path_after_colons_and_letter_shows_submodule() {
        let src = r#"
          mod foo {
            mod bar {}
          }

          fn main() {
            foo::b>|<
          }
        "#;
        assert_completion(src, vec![module_completion_item("bar")]).await;
    }

    #[test]
    async fn test_complete_path_with_local_variable() {
        let src = r#"
          fn main() {
            let local = 1;
            l>|<
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "local",
                CompletionItemKind::VARIABLE,
                Some("Field".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_with_shadowed_local_variable() {
        let src = r#"
          fn main() {
            let local = 1;
            let local = true;
            l>|<
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "local",
                CompletionItemKind::VARIABLE,
                Some("bool".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_with_function_argument() {
        let src = r#"
          fn main(local: Field) {
            l>|<
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "local",
                CompletionItemKind::VARIABLE,
                Some("Field".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_function_without_arguments() {
        let src = r#"
          fn hello() { }

          fn main() {
            h>|<
          }
        "#;
        assert_completion(
            src,
            vec![snippet_completion_item(
                "hello()",
                CompletionItemKind::FUNCTION,
                "hello()",
                Some("fn()".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_function() {
        let src = r#"
          fn hello(x: i32, y: Field) { }

          fn main() {
            h>|<
          }
        "#;
        assert_completion(
            src,
            vec![snippet_completion_item(
                "hello(…)",
                CompletionItemKind::FUNCTION,
                "hello(${1:x}, ${2:y})",
                Some("fn(i32, Field)".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_builtin_functions() {
        let src = r#"
          fn main() {
            a>|<
          }
        "#;
        assert_completion(
            src,
            vec![
                snippet_completion_item(
                    "assert(…)",
                    CompletionItemKind::FUNCTION,
                    "assert(${1:predicate})",
                    Some("fn(T)".to_string()),
                ),
                snippet_completion_item(
                    "assert_constant(…)",
                    CompletionItemKind::FUNCTION,
                    "assert_constant(${1:x})",
                    Some("fn(T)".to_string()),
                ),
                snippet_completion_item(
                    "assert_eq(…)",
                    CompletionItemKind::FUNCTION,
                    "assert_eq(${1:lhs}, ${2:rhs})",
                    Some("fn(T, T)".to_string()),
                ),
            ],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_in_impl() {
        let src = r#"
          struct SomeStruct {}

          impl SomeStruct {
            fn foo() {
                S>|<
            }
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "SomeStruct",
                CompletionItemKind::STRUCT,
                Some("SomeStruct".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_in_trait_impl() {
        let src = r#"
          struct SomeStruct {}
          trait Trait {}

          impl Trait for SomeStruct {
            fn foo() {
                S>|<
            }
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "SomeStruct",
                CompletionItemKind::STRUCT,
                Some("SomeStruct".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_with_for_argument() {
        let src = r#"
          fn main() {
            for index in 0..10 {
                i>|<
            }
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "index",
                CompletionItemKind::VARIABLE,
                Some("u32".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_complete_path_with_lambda_argument() {
        let src = r#"
          fn lambda(f: fn(i32)) { }

          fn main() {
            lambda(|var| v>|<)
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "var",
                CompletionItemKind::VARIABLE,
                Some("_".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_struct_field_type() {
        let src = r#"
          struct Something {}

          fn SomeFunction() {}

          struct Another {
            some: So>|<
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_function_parameter() {
        let src = r#"
          struct Something {}

          fn foo(x: So>|<) {}
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_function_return_type() {
        let src = r#"
          struct Something {}

          fn foo() -> So>|< {}
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_type_alias() {
        let src = r#"
          struct Something {}

          type Foo = So>|<
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_trait_function() {
        let src = r#"
          struct Something {}

          trait Trait {
            fn foo(s: So>|<);
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_trait_function_return_type() {
        let src = r#"
          struct Something {}

          trait Trait {
            fn foo() -> So>|<;
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_let_type() {
        let src = r#"
          struct Something {}

          fn main() {
            let x: So>|<
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_type_in_lambda_parameter() {
        let src = r#"
          struct Something {}

          fn main() {
            foo(|s: So>|<| s)
          }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "Something",
                CompletionItemKind::STRUCT,
                Some("Something".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_builtin_types() {
        let src = r#"
            fn foo(x: i>|<) {}
        "#;
        assert_completion(
            src,
            vec![
                simple_completion_item("i8", CompletionItemKind::STRUCT, Some("i8".to_string())),
                simple_completion_item("i16", CompletionItemKind::STRUCT, Some("i16".to_string())),
                simple_completion_item("i32", CompletionItemKind::STRUCT, Some("i32".to_string())),
                simple_completion_item("i64", CompletionItemKind::STRUCT, Some("i64".to_string())),
            ],
        )
        .await;
    }

    #[test]
    async fn test_suggest_true() {
        let src = r#"
            fn main() {
                let x = t>|<
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "true",
                CompletionItemKind::KEYWORD,
                Some("bool".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_regarding_if_scope() {
        let src = r#"
            fn main() {
                let good = 1;
                if true {
                    let great = 2;
                    g>|<
                } else {
                    let greater = 3;
                }
            }
        "#;
        assert_completion(
            src,
            vec![
                simple_completion_item(
                    "good",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
                simple_completion_item(
                    "great",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
            ],
        )
        .await;

        let src = r#"
            fn main() {
                let good = 1;
                if true {
                    let great = 2;
                } else {
                    let greater = 3;
                    g>|<
                }
            }
        "#;
        assert_completion(
            src,
            vec![
                simple_completion_item(
                    "good",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
                simple_completion_item(
                    "greater",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
            ],
        )
        .await;

        let src = r#"
            fn main() {
                let good = 1;
                if true {
                    let great = 2;
                } else {
                    let greater = 3;
                }
                g>|<
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "good",
                CompletionItemKind::VARIABLE,
                Some("Field".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_regarding_block_scope() {
        let src = r#"
            fn main() {
                let good = 1;
                {
                    let great = 2;
                    g>|<
                }
            }
        "#;
        assert_completion(
            src,
            vec![
                simple_completion_item(
                    "good",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
                simple_completion_item(
                    "great",
                    CompletionItemKind::VARIABLE,
                    Some("Field".to_string()),
                ),
            ],
        )
        .await;

        let src = r#"
            fn main() {
                let good = 1;
                {
                    let great = 2;
                }
                g>|<
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "good",
                CompletionItemKind::VARIABLE,
                Some("Field".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggest_struct_type_parameter() {
        let src = r#"
            struct Foo<Context> {
                context: C>|<
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item("Context", CompletionItemKind::TYPE_PARAMETER, None)],
        )
        .await;
    }

    #[test]
    async fn test_suggest_impl_type_parameter() {
        let src = r#"
            struct Foo<Context> {}

            impl <TypeParam> Foo<TypeParam> {
                fn foo() {
                    let x: TypeP>|<
                }
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item("TypeParam", CompletionItemKind::TYPE_PARAMETER, None)],
        )
        .await;
    }

    #[test]
    async fn test_suggest_trait_impl_type_parameter() {
        let src = r#"
            struct Foo {}
            trait Trait<Context> {}

            impl <TypeParam> Trait<TypeParam> for Foo {
                fn foo() {
                    let x: TypeP>|<
                }
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item("TypeParam", CompletionItemKind::TYPE_PARAMETER, None)],
        )
        .await;
    }

    #[test]
    async fn test_suggest_trait_function_type_parameter() {
        let src = r#"
            struct Foo {}
            trait Trait {
                fn foo<TypeParam>() {
                    let x: TypeP>|<
                }
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item("TypeParam", CompletionItemKind::TYPE_PARAMETER, None)],
        )
        .await;
    }

    #[test]
    async fn test_suggest_function_type_parameters() {
        let src = r#"
            fn foo<Context>(x: C>|<) {}
        "#;
        assert_completion(
            src,
            vec![simple_completion_item("Context", CompletionItemKind::TYPE_PARAMETER, None)],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_field_after_dot_and_letter() {
        let src = r#"
            struct Some {
                property: i32,
            }

            fn foo(s: Some) {
                s.p>|<
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "property",
                CompletionItemKind::FIELD,
                Some("i32".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_field_after_dot_and_letter_for_generic_type() {
        let src = r#"
            struct Some<T> {
                property: T,
            }

            fn foo(s: Some<i32>) {
                s.p>|<
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "property",
                CompletionItemKind::FIELD,
                Some("i32".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_field_after_dot_followed_by_brace() {
        let src = r#"
            struct Some {
                property: i32,
            }

            fn foo(s: Some) {
                s.>|<
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item(
                "property",
                CompletionItemKind::FIELD,
                Some("i32".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_field_after_dot_chain() {
        let src = r#"
            struct Some {
                property: Other,
            }

            struct Other {
                bar: i32,
            }

            fn foo(some: Some) {
                some.property.>|<
            }
        "#;
        assert_completion(
            src,
            vec![simple_completion_item("bar", CompletionItemKind::FIELD, Some("i32".to_string()))],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_impl_method() {
        let src = r#"
            struct Some {
            }

            impl Some {
                fn foobar(self, x: i32) {}
                fn foobar2(&mut self, x: i32) {}
                fn foobar3(y: i32) {}
            }

            fn foo(some: Some) {
                some.f>|<
            }
        "#;
        assert_completion(
            src,
            vec![
                snippet_completion_item(
                    "foobar(…)",
                    CompletionItemKind::FUNCTION,
                    "foobar(${1:x})",
                    Some("fn(self, i32)".to_string()),
                ),
                snippet_completion_item(
                    "foobar2(…)",
                    CompletionItemKind::FUNCTION,
                    "foobar2(${1:x})",
                    Some("fn(&mut self, i32)".to_string()),
                ),
            ],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_trait_impl_method() {
        let src = r#"
            struct Some {
            }

            trait SomeTrait {
                fn foobar(self, x: i32);
                fn foobar2(y: i32);
            }

            impl SomeTrait for Some {
                fn foobar(self, x: i32) {}
                fn foobar2(y: i32) {}
            }

            fn foo(some: Some) {
                some.f>|<
            }
        "#;
        assert_completion(
            src,
            vec![snippet_completion_item(
                "foobar(…)",
                CompletionItemKind::FUNCTION,
                "foobar(${1:x})",
                Some("fn(self, i32)".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggests_primitive_trait_impl_method() {
        let src = r#"
            trait SomeTrait {
                fn foobar(self, x: i32);
                fn foobar2(y: i32);
            }

            impl SomeTrait for Field {
                fn foobar(self, x: i32) {}
                fn foobar2(y: i32) {}
            }

            fn foo(field: Field) {
                field.f>|<
            }
        "#;
        assert_completion(
            src,
            vec![snippet_completion_item(
                "foobar(…)",
                CompletionItemKind::FUNCTION,
                "foobar(${1:x})",
                Some("fn(self, i32)".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_methods_after_colons() {
        let src = r#"
            struct Some {
            }

            impl Some {
                fn foobar(self, x: i32) {}
                fn foobar2(&mut self, x: i32) {}
                fn foobar3(y: i32) {}
            }

            fn foo() {
                Some::>|<
            }
        "#;
        assert_completion(
            src,
            vec![
                completion_item_with_sort_text(
                    snippet_completion_item(
                        "foobar(…)",
                        CompletionItemKind::FUNCTION,
                        "foobar(${1:self}, ${2:x})",
                        Some("fn(self, i32)".to_string()),
                    ),
                    self_mismatch_sort_text(),
                ),
                completion_item_with_sort_text(
                    snippet_completion_item(
                        "foobar2(…)",
                        CompletionItemKind::FUNCTION,
                        "foobar2(${1:self}, ${2:x})",
                        Some("fn(&mut self, i32)".to_string()),
                    ),
                    self_mismatch_sort_text(),
                ),
                snippet_completion_item(
                    "foobar3(…)",
                    CompletionItemKind::FUNCTION,
                    "foobar3(${1:y})",
                    Some("fn(i32)".to_string()),
                ),
            ],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_behind_alias_methods_after_dot() {
        let src = r#"
            struct Some {
            }

            type Alias = Some;

            impl Some {
                fn foobar(self, x: i32) {}
            }

            fn foo(some: Alias) {
                some.>|<
            }
        "#;
        assert_completion(
            src,
            vec![snippet_completion_item(
                "foobar(…)",
                CompletionItemKind::FUNCTION,
                "foobar(${1:x})",
                Some("fn(self, i32)".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_suggests_struct_behind_alias_methods_after_colons() {
        let src = r#"
            struct Some {
            }

            type Alias = Some;

            impl Some {
                fn foobar(self, x: i32) {}
                fn foobar2(&mut self, x: i32) {}
                fn foobar3(y: i32) {}
            }

            fn foo() {
                Alias::>|<
            }
        "#;
        assert_completion(
            src,
            vec![
                completion_item_with_sort_text(
                    snippet_completion_item(
                        "foobar(…)",
                        CompletionItemKind::FUNCTION,
                        "foobar(${1:self}, ${2:x})",
                        Some("fn(self, i32)".to_string()),
                    ),
                    self_mismatch_sort_text(),
                ),
                completion_item_with_sort_text(
                    snippet_completion_item(
                        "foobar2(…)",
                        CompletionItemKind::FUNCTION,
                        "foobar2(${1:self}, ${2:x})",
                        Some("fn(&mut self, i32)".to_string()),
                    ),
                    self_mismatch_sort_text(),
                ),
                snippet_completion_item(
                    "foobar3(…)",
                    CompletionItemKind::FUNCTION,
                    "foobar3(${1:y})",
                    Some("fn(i32)".to_string()),
                ),
            ],
        )
        .await;
    }
}
