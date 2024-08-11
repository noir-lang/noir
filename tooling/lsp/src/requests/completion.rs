use std::{
    collections::{BTreeMap, HashMap},
    future::{self, Future},
};

use async_lsp::ResponseError;
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
        Statement, TraitImplItem, TraitItem, TypeImpl, UnresolvedType, UseTree, UseTreeKind,
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
    ParsedModule, Type,
};

use crate::{utils, LspState};

use super::process_request;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ModuleCompletionKind {
    DirectChildren,
    AllVisibleItems,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FunctionCompleteKind {
    Name,
    NameAndParameters,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PathCompletionKind {
    Everything,
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
    root_module_id: ModuleId,
    module_id: ModuleId,
    def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
    dependencies: &'a Vec<Dependency>,
    interner: &'a NodeInterner,
    local_variables: HashMap<String, Span>,
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
        let local_variables = HashMap::new();
        Self {
            file,
            byte_index,
            byte,
            root_module_id,
            module_id,
            def_maps,
            dependencies,
            interner,
            local_variables,
        }
    }

    fn find(&mut self, parsed_module: &ParsedModule) -> Option<CompletionResponse> {
        self.find_in_parsed_module(parsed_module)
    }

    fn find_in_parsed_module(
        &mut self,
        parsed_module: &ParsedModule,
    ) -> Option<CompletionResponse> {
        for item in &parsed_module.items {
            if let Some(response) = self.find_in_item(item) {
                return Some(response);
            }
        }

        None
    }

    fn find_in_item(&mut self, item: &Item) -> Option<CompletionResponse> {
        if !self.includes_span(item.span) {
            return None;
        }

        match &item.kind {
            ItemKind::Import(use_tree) => {
                let mut prefixes = Vec::new();
                self.find_in_use_tree(use_tree, &mut prefixes)
            }
            ItemKind::Submodules(parsed_sub_module) => {
                // Switch `self.module_id` to the submodule
                let previous_module_id = self.module_id;

                let def_map = &self.def_maps[&self.module_id.krate];
                let module_data = def_map.modules().get(self.module_id.local_id.0)?;
                if let Some(child_module) = module_data.children.get(&parsed_sub_module.name) {
                    self.module_id =
                        ModuleId { krate: self.module_id.krate, local_id: *child_module };
                }

                let completion = self.find_in_parsed_module(&parsed_sub_module.contents);

                // Restore the old module before continuing
                self.module_id = previous_module_id;

                completion
            }
            ItemKind::Function(noir_function) => self.find_in_noir_function(noir_function),
            ItemKind::TraitImpl(noir_trait_impl) => self.find_in_noir_trait_impl(noir_trait_impl),
            ItemKind::Impl(type_impl) => self.find_in_type_impl(type_impl),
            ItemKind::Global(let_statement) => self.find_in_let_statement(let_statement, false),
            ItemKind::TypeAlias(noir_type_alias) => self.find_in_noir_type_alias(noir_type_alias),
            ItemKind::Struct(noir_struct) => self.find_in_noir_struct(noir_struct),
            ItemKind::Trait(noir_trait) => self.find_in_noir_trait(noir_trait),
            ItemKind::ModuleDecl(_) => None,
        }
    }

    fn find_in_noir_function(
        &mut self,
        noir_function: &NoirFunction,
    ) -> Option<CompletionResponse> {
        for param in &noir_function.def.parameters {
            if let Some(response) = self.find_in_unresolved_type(&param.typ) {
                return Some(response);
            }
        }

        if let Some(response) = self.find_in_function_return_type(&noir_function.def.return_type) {
            return Some(response);
        }

        self.local_variables.clear();
        for param in &noir_function.def.parameters {
            self.collect_local_variables(&param.pattern);
        }

        self.find_in_block_expression(&noir_function.def.body)
    }

    fn find_in_noir_trait_impl(
        &mut self,
        noir_trait_impl: &NoirTraitImpl,
    ) -> Option<CompletionResponse> {
        for item in &noir_trait_impl.items {
            if let Some(completion) = self.find_in_trait_impl_item(item) {
                return Some(completion);
            }
        }
        None
    }

    fn find_in_trait_impl_item(&mut self, item: &TraitImplItem) -> Option<CompletionResponse> {
        match item {
            TraitImplItem::Function(noir_function) => self.find_in_noir_function(noir_function),
            TraitImplItem::Constant(_, _, _) => None,
            TraitImplItem::Type { .. } => None,
        }
    }

    fn find_in_type_impl(&mut self, type_impl: &TypeImpl) -> Option<CompletionResponse> {
        for (method, _) in &type_impl.methods {
            if let Some(completion) = self.find_in_noir_function(method) {
                return Some(completion);
            }
        }

        None
    }

    fn find_in_noir_type_alias(
        &mut self,
        noir_type_alias: &NoirTypeAlias,
    ) -> Option<CompletionResponse> {
        self.find_in_unresolved_type(&noir_type_alias.typ)
    }

    fn find_in_noir_struct(&mut self, noir_struct: &NoirStruct) -> Option<CompletionResponse> {
        for (_name, unresolved_type) in &noir_struct.fields {
            if let Some(response) = self.find_in_unresolved_type(unresolved_type) {
                return Some(response);
            }
        }

        None
    }

    fn find_in_noir_trait(&mut self, noir_trait: &NoirTrait) -> Option<CompletionResponse> {
        for item in &noir_trait.items {
            if let Some(response) = self.find_in_trait_item(item) {
                return Some(response);
            }
        }
        None
    }

    fn find_in_trait_item(&mut self, trait_item: &TraitItem) -> Option<CompletionResponse> {
        match trait_item {
            TraitItem::Function {
                name: _,
                generics: _,
                parameters,
                return_type,
                where_clause,
                body,
            } => {
                for (_name, unresolved_type) in parameters {
                    if let Some(response) = self.find_in_unresolved_type(unresolved_type) {
                        return Some(response);
                    }
                }

                if let Some(response) = self.find_in_function_return_type(return_type) {
                    return Some(response);
                }

                for unresolved_trait_constraint in where_clause {
                    if let Some(response) =
                        self.find_in_unresolved_type(&unresolved_trait_constraint.typ)
                    {
                        return Some(response);
                    }
                }

                if let Some(body) = body {
                    self.local_variables.clear();
                    for (name, _) in parameters {
                        self.local_variables.insert(name.to_string(), name.span());
                    }
                    self.find_in_block_expression(body)
                } else {
                    None
                }
            }
            TraitItem::Constant { name: _, typ, default_value } => {
                if let Some(response) = self.find_in_unresolved_type(typ) {
                    return Some(response);
                }

                if let Some(default_value) = default_value {
                    self.find_in_expression(default_value)
                } else {
                    None
                }
            }
            TraitItem::Type { name: _ } => None,
        }
    }

    fn find_in_block_expression(
        &mut self,
        block_expression: &BlockExpression,
    ) -> Option<CompletionResponse> {
        for statement in &block_expression.statements {
            if let Some(completion) = self.find_in_statement(statement) {
                return Some(completion);
            }
        }
        None
    }

    fn find_in_statement(&mut self, statement: &Statement) -> Option<CompletionResponse> {
        match &statement.kind {
            noirc_frontend::ast::StatementKind::Let(let_statement) => {
                self.find_in_let_statement(let_statement, true)
            }
            noirc_frontend::ast::StatementKind::Constrain(constrain_statement) => {
                self.find_in_constrain_statement(constrain_statement)
            }
            noirc_frontend::ast::StatementKind::Expression(expression) => {
                self.find_in_expression(expression)
            }
            noirc_frontend::ast::StatementKind::Assign(assign_statement) => {
                self.find_in_assign_statement(assign_statement)
            }
            noirc_frontend::ast::StatementKind::For(for_loop_statement) => {
                self.find_in_for_loop_statement(for_loop_statement)
            }
            noirc_frontend::ast::StatementKind::Comptime(statement) => {
                // When entering a comptime block, regular local variables shouldn't be offered anymore
                let old_local_variables = self.local_variables.clone();
                self.local_variables.clear();

                let response = self.find_in_statement(&statement);
                self.local_variables = old_local_variables;
                response
            }
            noirc_frontend::ast::StatementKind::Semi(expression) => {
                self.find_in_expression(expression)
            }
            noirc_frontend::ast::StatementKind::Break
            | noirc_frontend::ast::StatementKind::Continue
            | noirc_frontend::ast::StatementKind::Error => None,
        }
    }

    fn find_in_let_statement(
        &mut self,
        let_statement: &LetStatement,
        collect_local_variables: bool,
    ) -> Option<CompletionResponse> {
        if let Some(response) = self.find_in_expression(&let_statement.expression) {
            return Some(response);
        }

        if collect_local_variables {
            self.collect_local_variables(&let_statement.pattern);
        }

        None
    }

    fn find_in_constrain_statement(
        &mut self,
        constrain_statement: &ConstrainStatement,
    ) -> Option<CompletionResponse> {
        if let Some(response) = self.find_in_expression(&constrain_statement.0) {
            return Some(response);
        }

        if let Some(exp) = &constrain_statement.1 {
            self.find_in_expression(exp)
        } else {
            None
        }
    }

    fn find_in_assign_statement(
        &mut self,
        assign_statement: &noirc_frontend::ast::AssignStatement,
    ) -> Option<CompletionResponse> {
        if let Some(response) = self.find_in_lvalue(&assign_statement.lvalue) {
            return Some(response);
        }

        self.find_in_expression(&assign_statement.expression)
    }

    fn find_in_for_loop_statement(
        &mut self,
        for_loop_statement: &ForLoopStatement,
    ) -> Option<CompletionResponse> {
        let old_local_variables = self.local_variables.clone();
        let ident = &for_loop_statement.identifier;
        self.local_variables.insert(ident.to_string(), ident.span());

        if let Some(response) = self.find_in_for_range(&for_loop_statement.range) {
            return Some(response);
        }

        let response = self.find_in_expression(&for_loop_statement.block);

        self.local_variables = old_local_variables;

        response
    }

    fn find_in_lvalue(&mut self, lvalue: &LValue) -> Option<CompletionResponse> {
        match lvalue {
            LValue::Ident(_) => None,
            LValue::MemberAccess { object, field_name: _, span: _ } => self.find_in_lvalue(object),
            LValue::Index { array, index, span: _ } => {
                if let Some(response) = self.find_in_lvalue(array) {
                    return Some(response);
                }

                self.find_in_expression(index)
            }
            LValue::Dereference(lvalue, _) => self.find_in_lvalue(lvalue),
        }
    }

    fn find_in_for_range(&mut self, for_range: &ForRange) -> Option<CompletionResponse> {
        match for_range {
            ForRange::Range(start, end) => {
                if let Some(response) = self.find_in_expression(start) {
                    return Some(response);
                }

                self.find_in_expression(end)
            }
            ForRange::Array(expression) => self.find_in_expression(expression),
        }
    }

    fn find_in_expressions(&mut self, expressions: &[Expression]) -> Option<CompletionResponse> {
        for expression in expressions {
            if let Some(response) = self.find_in_expression(expression) {
                return Some(response);
            }
        }
        None
    }

    fn find_in_expression(&mut self, expression: &Expression) -> Option<CompletionResponse> {
        match &expression.kind {
            noirc_frontend::ast::ExpressionKind::Literal(literal) => self.find_in_literal(literal),
            noirc_frontend::ast::ExpressionKind::Block(block_expression) => {
                self.find_in_block_expression(block_expression)
            }
            noirc_frontend::ast::ExpressionKind::Prefix(prefix_expression) => {
                self.find_in_expression(&prefix_expression.rhs)
            }
            noirc_frontend::ast::ExpressionKind::Index(index_expression) => {
                self.find_in_index_expression(&index_expression)
            }
            noirc_frontend::ast::ExpressionKind::Call(call_expression) => {
                self.find_in_call_expression(call_expression)
            }
            noirc_frontend::ast::ExpressionKind::MethodCall(method_call_expression) => {
                self.find_in_method_call_expression(method_call_expression)
            }
            noirc_frontend::ast::ExpressionKind::Constructor(constructor_expression) => {
                self.find_in_constructor_expression(constructor_expression)
            }
            noirc_frontend::ast::ExpressionKind::MemberAccess(member_access_expression) => {
                self.find_in_member_access_expression(member_access_expression)
            }
            noirc_frontend::ast::ExpressionKind::Cast(cast_expression) => {
                self.find_in_cast_expression(cast_expression)
            }
            noirc_frontend::ast::ExpressionKind::Infix(infix_expression) => {
                self.find_in_infix_expression(infix_expression)
            }
            noirc_frontend::ast::ExpressionKind::If(if_expression) => {
                self.find_in_if_expression(if_expression)
            }
            noirc_frontend::ast::ExpressionKind::Variable(path) => {
                self.find_in_path(path, PathCompletionKind::Everything)
            }
            noirc_frontend::ast::ExpressionKind::Tuple(expressions) => {
                self.find_in_expressions(expressions)
            }
            noirc_frontend::ast::ExpressionKind::Lambda(lambda) => self.find_in_lambda(lambda),
            noirc_frontend::ast::ExpressionKind::Parenthesized(expression) => {
                self.find_in_expression(expression)
            }
            noirc_frontend::ast::ExpressionKind::Unquote(expression) => {
                self.find_in_expression(expression)
            }
            noirc_frontend::ast::ExpressionKind::Comptime(block_expression, _) => {
                // When entering a comptime block, regular local variables shouldn't be offered anymore
                let old_local_variables = self.local_variables.clone();
                self.local_variables.clear();

                let response = self.find_in_block_expression(&block_expression);
                self.local_variables = old_local_variables;
                response
            }
            noirc_frontend::ast::ExpressionKind::AsTraitPath(as_trait_path) => {
                self.find_in_as_trait_path(as_trait_path)
            }
            noirc_frontend::ast::ExpressionKind::Quote(_)
            | noirc_frontend::ast::ExpressionKind::Resolved(_)
            | noirc_frontend::ast::ExpressionKind::Error => None,
        }
    }

    fn find_in_literal(&mut self, literal: &Literal) -> Option<CompletionResponse> {
        match literal {
            Literal::Array(array_literal) => self.find_in_array_literal(array_literal),
            Literal::Slice(array_literal) => self.find_in_array_literal(array_literal),
            Literal::Bool(_)
            | Literal::Integer(_, _)
            | Literal::Str(_)
            | Literal::RawStr(_, _)
            | Literal::FmtStr(_)
            | Literal::Unit => None,
        }
    }

    fn find_in_array_literal(
        &mut self,
        array_literal: &ArrayLiteral,
    ) -> Option<CompletionResponse> {
        match array_literal {
            ArrayLiteral::Standard(expressions) => self.find_in_expressions(expressions),
            ArrayLiteral::Repeated { repeated_element, length } => {
                if let Some(completion) = self.find_in_expression(repeated_element) {
                    return Some(completion);
                }

                self.find_in_expression(length)
            }
        }
    }

    fn find_in_index_expression(
        &mut self,
        index_expression: &IndexExpression,
    ) -> Option<CompletionResponse> {
        if let Some(response) = self.find_in_expression(&index_expression.collection) {
            return Some(response);
        }

        self.find_in_expression(&index_expression.index)
    }

    fn find_in_call_expression(
        &mut self,
        call_expression: &CallExpression,
    ) -> Option<CompletionResponse> {
        if let Some(response) = self.find_in_expression(&call_expression.func) {
            return Some(response);
        }

        self.find_in_expressions(&call_expression.arguments)
    }

    fn find_in_method_call_expression(
        &mut self,
        method_call_expression: &MethodCallExpression,
    ) -> Option<CompletionResponse> {
        if let Some(response) = self.find_in_expression(&method_call_expression.object) {
            return Some(response);
        }

        self.find_in_expressions(&method_call_expression.arguments)
    }

    fn find_in_constructor_expression(
        &mut self,
        constructor_expression: &ConstructorExpression,
    ) -> Option<CompletionResponse> {
        if let Some(response) =
            self.find_in_path(&constructor_expression.type_name, PathCompletionKind::OnlyTypes)
        {
            return Some(response);
        }

        for (_field_name, expression) in &constructor_expression.fields {
            if let Some(response) = self.find_in_expression(expression) {
                return Some(response);
            }
        }

        None
    }

    fn find_in_member_access_expression(
        &mut self,
        member_access_expression: &MemberAccessExpression,
    ) -> Option<CompletionResponse> {
        self.find_in_expression(&member_access_expression.lhs)
    }

    fn find_in_cast_expression(
        &mut self,
        cast_expression: &CastExpression,
    ) -> Option<CompletionResponse> {
        self.find_in_expression(&cast_expression.lhs)
    }

    fn find_in_infix_expression(
        &mut self,
        infix_expression: &InfixExpression,
    ) -> Option<CompletionResponse> {
        if let Some(response) = self.find_in_expression(&infix_expression.lhs) {
            return Some(response);
        }

        self.find_in_expression(&infix_expression.rhs)
    }

    fn find_in_if_expression(
        &mut self,
        if_expression: &IfExpression,
    ) -> Option<CompletionResponse> {
        if let Some(response) = self.find_in_expression(&if_expression.condition) {
            return Some(response);
        }

        if let Some(response) = self.find_in_expression(&if_expression.consequence) {
            return Some(response);
        }

        if let Some(alternative) = &if_expression.alternative {
            self.find_in_expression(alternative)
        } else {
            None
        }
    }

    fn find_in_lambda(&mut self, lambda: &Lambda) -> Option<CompletionResponse> {
        let old_local_variables = self.local_variables.clone();

        for (pattern, _) in &lambda.parameters {
            self.collect_local_variables(pattern)
        }

        let response = self.find_in_expression(&lambda.body);

        self.local_variables = old_local_variables;

        response
    }

    fn find_in_as_trait_path(&mut self, as_trait_path: &AsTraitPath) -> Option<CompletionResponse> {
        self.find_in_path(&as_trait_path.trait_path, PathCompletionKind::OnlyTypes)
    }

    fn find_in_function_return_type(
        &mut self,
        return_type: &FunctionReturnType,
    ) -> Option<CompletionResponse> {
        match return_type {
            noirc_frontend::ast::FunctionReturnType::Default(_) => None,
            noirc_frontend::ast::FunctionReturnType::Ty(unresolved_type) => {
                self.find_in_unresolved_type(unresolved_type)
            }
        }
    }

    fn find_in_unresolved_types(
        &mut self,
        unresolved_type: &[UnresolvedType],
    ) -> Option<CompletionResponse> {
        for unresolved_type in unresolved_type {
            if let Some(response) = self.find_in_unresolved_type(unresolved_type) {
                return Some(response);
            }
        }

        None
    }

    fn find_in_unresolved_type(
        &mut self,
        unresolved_type: &UnresolvedType,
    ) -> Option<CompletionResponse> {
        if let Some(span) = unresolved_type.span {
            if !self.includes_span(span) {
                return None;
            }
        }

        match &unresolved_type.typ {
            noirc_frontend::ast::UnresolvedTypeData::Array(_, unresolved_type) => {
                self.find_in_unresolved_type(unresolved_type)
            }
            noirc_frontend::ast::UnresolvedTypeData::Slice(unresolved_type) => {
                self.find_in_unresolved_type(unresolved_type)
            }
            noirc_frontend::ast::UnresolvedTypeData::Parenthesized(unresolved_type) => {
                self.find_in_unresolved_type(unresolved_type)
            }
            noirc_frontend::ast::UnresolvedTypeData::Named(path, unresolved_types, _) => {
                if let Some(response) = self.find_in_path(path, PathCompletionKind::OnlyTypes) {
                    return Some(response);
                }

                self.find_in_unresolved_types(unresolved_types)
            }
            noirc_frontend::ast::UnresolvedTypeData::TraitAsType(path, unresolved_types) => {
                if let Some(response) = self.find_in_path(path, PathCompletionKind::OnlyTypes) {
                    return Some(response);
                }

                self.find_in_unresolved_types(unresolved_types)
            }
            noirc_frontend::ast::UnresolvedTypeData::MutableReference(unresolved_type) => {
                self.find_in_unresolved_type(unresolved_type)
            }
            noirc_frontend::ast::UnresolvedTypeData::Tuple(unresolved_types) => {
                self.find_in_unresolved_types(unresolved_types)
            }
            noirc_frontend::ast::UnresolvedTypeData::Function(args, ret, env) => {
                if let Some(response) = self.find_in_unresolved_types(args) {
                    return Some(response);
                }

                if let Some(response) = self.find_in_unresolved_type(ret) {
                    return Some(response);
                }

                self.find_in_unresolved_type(env)
            }
            noirc_frontend::ast::UnresolvedTypeData::AsTraitPath(as_trait_path) => {
                self.find_in_as_trait_path(as_trait_path)
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
            | noirc_frontend::ast::UnresolvedTypeData::Error => None,
        }
    }

    fn find_in_path(
        &mut self,
        path: &Path,
        path_completion_kind: PathCompletionKind,
    ) -> Option<CompletionResponse> {
        // Only offer completions if we are right at the end of the path
        if self.byte_index != path.span.end() as usize {
            return None;
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

        let module_id =
            if idents.is_empty() { Some(self.module_id) } else { self.resolve_module(idents) };
        let Some(module_id) = module_id else {
            return None;
        };

        let module_completion_kind = if after_colons {
            ModuleCompletionKind::DirectChildren
        } else {
            ModuleCompletionKind::AllVisibleItems
        };
        let function_completion_kind = FunctionCompleteKind::NameAndParameters;

        let response = self.complete_in_module(
            module_id,
            &prefix,
            path.kind,
            at_root,
            module_completion_kind,
            function_completion_kind,
            path_completion_kind,
        );

        if is_single_segment {
            match path_completion_kind {
                PathCompletionKind::Everything => {
                    let local_vars_response = self.local_variables_completion(&prefix);
                    let response = merge_completion_responses(response, local_vars_response);

                    let predefined_response = predefined_functions_completion(&prefix);
                    let response = merge_completion_responses(response, predefined_response);

                    response
                }
                PathCompletionKind::OnlyTypes => response,
            }
        } else {
            response
        }
    }

    fn local_variables_completion(&self, prefix: &String) -> Option<CompletionResponse> {
        let mut completion_items = Vec::new();

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

                completion_items.push(simple_completion_item(
                    name,
                    CompletionItemKind::VARIABLE,
                    description,
                ));
            }
        }

        if completion_items.is_empty() {
            None
        } else {
            Some(CompletionResponse::Array(completion_items))
        }
    }

    fn find_in_use_tree(
        &self,
        use_tree: &UseTree,
        prefixes: &mut Vec<Path>,
    ) -> Option<CompletionResponse> {
        match &use_tree.kind {
            UseTreeKind::Path(ident, alias) => {
                prefixes.push(use_tree.prefix.clone());
                let response = self.find_in_use_tree_path(prefixes, ident, alias);
                prefixes.pop();
                response
            }
            UseTreeKind::List(use_trees) => {
                prefixes.push(use_tree.prefix.clone());
                for use_tree in use_trees {
                    if let Some(completion) = self.find_in_use_tree(use_tree, prefixes) {
                        return Some(completion);
                    }
                }
                prefixes.pop();
                None
            }
        }
    }

    fn find_in_use_tree_path(
        &self,
        prefixes: &Vec<Path>,
        ident: &Ident,
        alias: &Option<Ident>,
    ) -> Option<CompletionResponse> {
        if let Some(_alias) = alias {
            // Won't handle completion if there's an alias (for now)
            return None;
        }

        let after_colons = self.byte == Some(b':');
        let at_ident_end = self.byte_index == ident.span().end() as usize;
        let at_ident_colons_end =
            after_colons && self.byte_index - 2 == ident.span().end() as usize;

        if !(at_ident_end || at_ident_colons_end) {
            return None;
        }

        let path_kind = prefixes[0].kind;

        let mut segments: Vec<Ident> = Vec::new();
        for prefix in prefixes {
            for segment in &prefix.segments {
                segments.push(segment.ident.clone());
            }
        }

        let module_completion_kind = ModuleCompletionKind::DirectChildren;
        let function_completion_kind = FunctionCompleteKind::Name;
        let path_completion_kind = PathCompletionKind::Everything;

        if after_colons {
            // We are right after "::"
            segments.push(ident.clone());

            self.resolve_module(segments).and_then(|module_id| {
                let prefix = String::new();
                let at_root = false;
                self.complete_in_module(
                    module_id,
                    &prefix,
                    path_kind,
                    at_root,
                    module_completion_kind,
                    function_completion_kind,
                    path_completion_kind,
                )
            })
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
                    path_completion_kind,
                )
            } else {
                let at_root = false;
                self.resolve_module(segments).and_then(|module_id| {
                    self.complete_in_module(
                        module_id,
                        &prefix,
                        path_kind,
                        at_root,
                        module_completion_kind,
                        function_completion_kind,
                        path_completion_kind,
                    )
                })
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

    fn complete_in_module(
        &self,
        module_id: ModuleId,
        prefix: &String,
        path_kind: PathKind,
        at_root: bool,
        module_completion_kind: ModuleCompletionKind,
        function_completion_kind: FunctionCompleteKind,
        path_completion_kind: PathCompletionKind,
    ) -> Option<CompletionResponse> {
        let def_map = &self.def_maps[&module_id.krate];
        let mut module_data = def_map.modules().get(module_id.local_id.0)?;

        if at_root {
            match path_kind {
                PathKind::Crate => {
                    module_data = def_map.modules().get(def_map.root().0)?;
                }
                PathKind::Super => {
                    module_data = def_map.modules().get(module_data.parent?.0)?;
                }
                PathKind::Dep => (),
                PathKind::Plain => (),
            }
        }

        let mut completion_items = Vec::new();

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
                        path_completion_kind,
                    ) {
                        completion_items.push(completion_item);
                    }
                }

                if let Some((module_def_id, _, _)) = per_ns.values {
                    if let Some(completion_item) = self.module_def_id_completion_item(
                        module_def_id,
                        name.clone(),
                        function_completion_kind,
                        path_completion_kind,
                    ) {
                        completion_items.push(completion_item);
                    }
                }
            }
        }

        if at_root && path_kind == PathKind::Plain {
            for dependency in self.dependencies {
                let dependency_name = dependency.as_name();
                if name_matches(&dependency_name, prefix) {
                    completion_items.push(crate_completion_item(dependency_name));
                }
            }

            if name_matches("crate::", &prefix) {
                completion_items.push(simple_completion_item(
                    "crate::",
                    CompletionItemKind::KEYWORD,
                    None,
                ));
            }

            if module_data.parent.is_some() && name_matches("super::", &prefix) {
                completion_items.push(simple_completion_item(
                    "super::",
                    CompletionItemKind::KEYWORD,
                    None,
                ));
            }
        }

        if completion_items.is_empty() {
            None
        } else {
            Some(CompletionResponse::Array(completion_items))
        }
    }

    fn module_def_id_completion_item(
        &self,
        module_def_id: ModuleDefId,
        name: String,
        function_completion_kind: FunctionCompleteKind,
        path_completion_kind: PathCompletionKind,
    ) -> Option<CompletionItem> {
        match path_completion_kind {
            PathCompletionKind::OnlyTypes => match module_def_id {
                ModuleDefId::FunctionId(_) | ModuleDefId::GlobalId(_) => return None,
                ModuleDefId::ModuleId(_)
                | ModuleDefId::TypeId(_)
                | ModuleDefId::TypeAliasId(_)
                | ModuleDefId::TraitId(_) => (),
            },
            PathCompletionKind::Everything => (),
        }

        let completion_item = match module_def_id {
            ModuleDefId::ModuleId(_) => module_completion_item(name),
            ModuleDefId::FunctionId(func_id) => {
                self.function_completion_item(func_id, function_completion_kind)
            }
            ModuleDefId::TypeId(struct_id) => self.struct_completion_item(struct_id),
            ModuleDefId::TypeAliasId(type_alias_id) => {
                self.type_alias_completion_item(type_alias_id)
            }
            ModuleDefId::TraitId(trait_id) => self.trait_completion_item(trait_id),
            ModuleDefId::GlobalId(global_id) => self.global_completion_item(global_id),
        };
        Some(completion_item)
    }

    fn function_completion_item(
        &self,
        func_id: FuncId,
        function_completion_kind: FunctionCompleteKind,
    ) -> CompletionItem {
        let func_meta = self.interner.function_meta(&func_id);
        let name = self.interner.function_name(&func_id).to_string();

        let mut typ = &func_meta.typ;
        if let Type::Forall(_, typ_) = typ {
            typ = typ_;
        }
        let description = typ.to_string();
        let description = description.strip_suffix(" -> ()").unwrap_or(&description).to_string();

        match function_completion_kind {
            FunctionCompleteKind::Name => {
                simple_completion_item(name, CompletionItemKind::FUNCTION, Some(description))
            }
            FunctionCompleteKind::NameAndParameters => {
                let label = format!("{}(…)", name);
                let kind = CompletionItemKind::FUNCTION;
                let insert_text = self.compute_function_insert_text(&func_meta, &name);

                snippet_completion_item(label, kind, insert_text, Some(description))
            }
        }
    }

    fn compute_function_insert_text(&self, func_meta: &FuncMeta, name: &String) -> String {
        let mut text = String::new();
        text.push_str(name);
        text.push('(');
        for (index, (pattern, _, _)) in func_meta.parameters.0.iter().enumerate() {
            if index > 0 {
                text.push_str(", ");
            }

            text.push_str("${");
            text.push_str(&(index + 1).to_string());
            text.push(':');
            self.hir_pattern_to_argument(pattern, &mut text);
            text.push('}');
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
        let path_segments = segments.into_iter().map(PathSegment::from).collect();
        let path = Path { segments: path_segments, kind: PathKind::Plain, span: Span::default() };

        let path_resolver = StandardPathResolver::new(self.root_module_id);
        match path_resolver.resolve(self.def_maps, path, &mut None) {
            Ok(path_resolution) => Some(path_resolution.module_def_id),
            Err(_) => None,
        }
    }

    fn includes_span(&self, span: Span) -> bool {
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}

fn name_matches(name: &str, prefix: &str) -> bool {
    name.starts_with(prefix)
}

fn predefined_functions_completion(prefix: &String) -> Option<CompletionResponse> {
    let mut completion_items = Vec::new();

    if name_matches("assert", prefix) {
        completion_items.push(snippet_completion_item(
            "assert(…)",
            CompletionItemKind::FUNCTION,
            "assert(${1:predicate})",
            Some("fn(T)".to_string()),
        ));
    }

    if name_matches("assert_eq", prefix) {
        completion_items.push(snippet_completion_item(
            "assert_eq(…)",
            CompletionItemKind::FUNCTION,
            "assert_eq(${1:lhs}, ${2:rhs})",
            Some("fn(T, T)".to_string()),
        ));
    }

    if completion_items.is_empty() {
        None
    } else {
        Some(CompletionResponse::Array(completion_items))
    }
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
        sort_text: None,
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
        sort_text: None,
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

fn merge_completion_responses(
    response1: Option<CompletionResponse>,
    response2: Option<CompletionResponse>,
) -> Option<CompletionResponse> {
    match (response1, response2) {
        (Some(CompletionResponse::Array(mut items1)), Some(CompletionResponse::Array(items2))) => {
            items1.extend(items2);
            Some(CompletionResponse::Array(items1))
        }
        (Some(response), None) | (None, Some(response)) => Some(response),
        _ => None,
    }
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
          mod foo {}

          fn main() {
            f>|<
          }
        "#;
        assert_completion(src, vec![module_completion_item("foo")]).await;
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
    async fn test_complete_predefined_functions() {
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
            some: S>|<
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

          fn foo(x: S>|<) {}
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

          fn foo() -> S>|< {}
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

          type Foo = S>|<
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
            fn foo(s: S>|<);
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
            fn foo() -> S>|<;
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
}
