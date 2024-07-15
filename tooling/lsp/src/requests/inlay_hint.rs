use std::future::{self, Future};

use async_lsp::ResponseError;
use fm::{FileId, PathString};
use lsp_types::{
    InlayHint, InlayHintKind, InlayHintLabel, InlayHintLabelPart, InlayHintParams, Position,
    TextDocumentPositionParams,
};
use noirc_errors::Location;
use noirc_frontend::{
    self,
    ast::{
        BlockExpression, Expression, ExpressionKind, LetStatement, NoirFunction, Pattern,
        Statement, StatementKind, TraitImplItem, TraitItem, UnresolvedTypeData,
    },
    node_interner::ReferenceId,
    parser::{Item, ItemKind},
    ParsedModule, Type, TypeBinding, TypeVariable, TypeVariableKind,
};

use crate::LspState;

use super::{process_request, to_lsp_location, ProcessRequestCallbackArgs};

pub(crate) fn on_inlay_hint_request(
    state: &mut LspState,
    params: InlayHintParams,
) -> impl Future<Output = Result<Option<Vec<InlayHint>>, ResponseError>> {
    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document.clone(),
        position: Position { line: 0, character: 0 },
    };

    let result = process_request(state, text_document_position_params, |args| {
        let path = PathString::from_path(params.text_document.uri.to_file_path().unwrap());
        args.files.get_file_id(&path).map(|file_id| {
            let file = args.files.get_file(file_id).unwrap();
            let source = file.source();
            let (parsed_moduled, _errors) = noirc_frontend::parse_program(source);

            let mut collector = InlayHintCollector::new(args, file_id);
            collector.collect_in_parsed_module(&parsed_moduled);
            collector.inlay_hints
        })
    });
    future::ready(result)
}

pub(crate) struct InlayHintCollector<'a> {
    args: ProcessRequestCallbackArgs<'a>,
    file_id: FileId,
    pub(crate) inlay_hints: Vec<InlayHint>,
}

impl<'a> InlayHintCollector<'a> {
    fn new(args: ProcessRequestCallbackArgs<'a>, file_id: FileId) -> InlayHintCollector<'a> {
        InlayHintCollector { args, file_id, inlay_hints: Vec::new() }
    }
    fn collect_in_parsed_module(&mut self, parsed_module: &ParsedModule) {
        for item in &parsed_module.items {
            self.collect_in_item(item);
        }
    }

    fn collect_in_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Function(noir_function) => self.collect_in_noir_function(noir_function),
            ItemKind::Trait(noir_trait) => {
                for item in &noir_trait.items {
                    self.collect_in_trait_item(item)
                }
            }
            ItemKind::TraitImpl(noir_trait_impl) => {
                for item in &noir_trait_impl.items {
                    self.collect_in_trait_impl_item(item)
                }
            }
            ItemKind::Impl(type_impl) => {
                for (noir_function, _) in &type_impl.methods {
                    self.collect_in_noir_function(&noir_function)
                }
            }
            ItemKind::Global(let_statement) => self.collect_in_let_statement(let_statement),
            ItemKind::Submodules(parsed_submodule) => {
                self.collect_in_parsed_module(&parsed_submodule.contents);
            }
            ItemKind::ModuleDecl(_) => (),
            ItemKind::Import(_) => (),
            ItemKind::Struct(_) => (),
            ItemKind::TypeAlias(_) => (),
        }
    }

    fn collect_in_trait_item(&mut self, item: &TraitItem) {
        match item {
            TraitItem::Function { body, .. } => {
                if let Some(body) = body {
                    self.collect_in_block_expression(body);
                }
            }
            TraitItem::Constant { name: _, typ: _, default_value } => {
                // TODO: show hint for constant?
                if let Some(default_value) = default_value {
                    self.collect_in_expression(default_value);
                }
            }
            TraitItem::Type { .. } => (),
        }
    }

    fn collect_in_trait_impl_item(&mut self, item: &TraitImplItem) {
        match item {
            TraitImplItem::Function(noir_function) => self.collect_in_noir_function(&noir_function),
            TraitImplItem::Constant(_name, _typ, default_value) => {
                // TODO: show hint for constant?
                self.collect_in_expression(default_value);
            }
            TraitImplItem::Type { .. } => (),
        }
    }

    fn collect_in_noir_function(&mut self, noir_function: &NoirFunction) {
        self.collect_in_block_expression(&noir_function.def.body);
    }

    fn collect_in_let_statement(&mut self, let_statement: &LetStatement) {
        // Only show inlay hints for let variables that don't have an explicit type annotation
        if let UnresolvedTypeData::Unspecified = let_statement.r#type.typ {
            self.collect_in_pattern(&let_statement.pattern);
        };

        self.collect_in_expression(&let_statement.expression);
    }

    fn collect_in_block_expression(&mut self, block_expression: &BlockExpression) {
        for statement in &block_expression.statements {
            self.collect_in_statement(statement)
        }
    }

    fn collect_in_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Let(let_statement) => self.collect_in_let_statement(let_statement),
            StatementKind::Constrain(constrain_statement) => {
                self.collect_in_expression(&constrain_statement.0)
            }
            StatementKind::Expression(expression) => self.collect_in_expression(expression),
            StatementKind::Assign(assign_statement) => {
                self.collect_in_expression(&assign_statement.expression)
            }
            StatementKind::For(for_loop_statement) => {
                // TODO: show type for for identifier
                self.collect_in_expression(&for_loop_statement.block)
            }
            StatementKind::Comptime(statement) => self.collect_in_statement(statement),
            StatementKind::Semi(expression) => self.collect_in_expression(expression),
            StatementKind::Break => (),
            StatementKind::Continue => (),
            StatementKind::Error => (),
        }
    }

    fn collect_in_expression(&mut self, expression: &Expression) {
        match &expression.kind {
            ExpressionKind::Block(block_expression) => {
                self.collect_in_block_expression(block_expression);
            }
            ExpressionKind::Prefix(prefix_expression) => {
                self.collect_in_expression(&prefix_expression.rhs);
            }
            ExpressionKind::Index(index_expression) => {
                self.collect_in_expression(&index_expression.collection);
                self.collect_in_expression(&index_expression.index);
            }
            ExpressionKind::Call(call_expression) => {
                self.collect_in_expression(&call_expression.func);
                for arg in &call_expression.arguments {
                    self.collect_in_expression(arg);
                }
            }
            ExpressionKind::MethodCall(method_call_expression) => {
                self.collect_in_expression(&method_call_expression.object);
                for arg in &method_call_expression.arguments {
                    self.collect_in_expression(arg);
                }
            }
            ExpressionKind::Constructor(constructor_expression) => {
                for (_name, expr) in &constructor_expression.fields {
                    self.collect_in_expression(expr);
                }
            }
            ExpressionKind::MemberAccess(member_access_expression) => {
                self.collect_in_expression(&member_access_expression.lhs);
            }
            ExpressionKind::Cast(cast_expression) => {
                self.collect_in_expression(&cast_expression.lhs);
            }
            ExpressionKind::Infix(infix_expression) => {
                self.collect_in_expression(&infix_expression.lhs);
                self.collect_in_expression(&infix_expression.rhs);
            }
            ExpressionKind::If(if_expression) => {
                self.collect_in_expression(&if_expression.condition);
                self.collect_in_expression(&if_expression.consequence);
                if let Some(alternative) = &if_expression.alternative {
                    self.collect_in_expression(alternative);
                }
            }
            ExpressionKind::Variable(..) => (),
            ExpressionKind::Tuple(expressions) => {
                for expression in expressions {
                    self.collect_in_expression(expression);
                }
            }
            ExpressionKind::Lambda(lambda) => self.collect_in_expression(&lambda.body),
            ExpressionKind::Parenthesized(parenthesized) => {
                self.collect_in_expression(&parenthesized);
            }
            ExpressionKind::Unquote(expression) => {
                self.collect_in_expression(expression);
            }
            ExpressionKind::Comptime(block_expression, _span) => {
                self.collect_in_block_expression(block_expression);
            }
            ExpressionKind::Literal(..)
            | ExpressionKind::Quote(..)
            | ExpressionKind::Resolved(..)
            | ExpressionKind::Error => (),
        }
    }

    fn collect_in_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier(ident) => {
                let span = ident.span();
                let location = Location::new(ident.span(), self.file_id);
                if let Some(lsp_location) = to_lsp_location(self.args.files, self.file_id, span) {
                    if let Some(referenced) = self.args.interner.find_referenced(location) {
                        match referenced {
                            ReferenceId::Global(global_id) => {
                                let global_info = self.args.interner.get_global(global_id);
                                let definition_id = global_info.definition_id;
                                let typ = self.args.interner.definition_type(definition_id);
                                self.push_type_hint(lsp_location, &typ);
                            }
                            ReferenceId::Local(definition_id) => {
                                let typ = self.args.interner.definition_type(definition_id);
                                self.push_type_hint(lsp_location, &typ);
                            }
                            ReferenceId::StructMember(struct_id, field_index) => {
                                let struct_type = self.args.interner.get_struct(struct_id);
                                let struct_type = struct_type.borrow();
                                let (_field_name, field_type) = struct_type.field_at(field_index);
                                self.push_type_hint(lsp_location, field_type);
                            }
                            ReferenceId::Module(_)
                            | ReferenceId::Struct(_)
                            | ReferenceId::Trait(_)
                            | ReferenceId::Function(_)
                            | ReferenceId::Alias(_)
                            | ReferenceId::Reference(..) => {
                                panic!("Unexpected reference for a pattern: {:?}", referenced)
                            }
                        }
                    }
                }
            }
            Pattern::Mutable(pattern, _span, _is_synthesized) => {
                self.collect_in_pattern(pattern);
            }
            Pattern::Tuple(patterns, _span) => {
                for pattern in patterns {
                    self.collect_in_pattern(pattern);
                }
            }
            Pattern::Struct(_path, patterns, _span) => {
                for (_ident, pattern) in patterns {
                    self.collect_in_pattern(pattern);
                }
            }
        }
    }

    fn push_type_hint(&mut self, location: lsp_types::Location, typ: &Type) {
        let position = location.range.end;

        let mut parts = Vec::new();
        parts.push(str_part(": "));
        self.push_type_parts(typ, &mut parts);

        self.inlay_hints.push(InlayHint {
            position,
            label: InlayHintLabel::LabelParts(parts),
            kind: Some(InlayHintKind::TYPE),
            text_edits: None,
            tooltip: None,
            padding_left: None,
            padding_right: None,
            data: None,
        });
    }

    fn push_type_parts(&self, typ: &Type, parts: &mut Vec<InlayHintLabelPart>) {
        match typ {
            Type::Array(size, typ) => {
                parts.push(str_part("["));
                self.push_type_parts(typ, parts);
                parts.push(str_part("; "));
                self.push_type_parts(size, parts);
                parts.push(str_part("]"));
            }
            Type::Slice(typ) => {
                parts.push(str_part("["));
                self.push_type_parts(typ, parts);
                parts.push(str_part("]"));
            }
            Type::Tuple(types) => {
                parts.push(str_part("("));
                for (index, typ) in types.iter().enumerate() {
                    self.push_type_parts(typ, parts);
                    if index != types.len() - 1 {
                        parts.push(str_part(", "))
                    }
                }
                parts.push(str_part(")"));
            }
            Type::Struct(struct_type, generics) => {
                let struct_type = struct_type.borrow();
                let location = Location::new(struct_type.name.span(), struct_type.location.file);
                parts.push(self.text_part_with_location(struct_type.name.to_string(), location));
                if !generics.is_empty() {
                    parts.push(str_part("<"));
                    for (index, generic) in generics.iter().enumerate() {
                        self.push_type_parts(generic, parts);
                        if index != generics.len() - 1 {
                            parts.push(str_part(", "))
                        }
                    }
                    parts.push(str_part(">"));
                }
            }
            Type::Alias(type_alias, generics) => {
                let type_alias = type_alias.borrow();
                let location = Location::new(type_alias.name.span(), type_alias.location.file);
                parts.push(self.text_part_with_location(type_alias.name.to_string(), location));
                if !generics.is_empty() {
                    parts.push(str_part("<"));
                    for (index, generic) in generics.iter().enumerate() {
                        self.push_type_parts(generic, parts);
                        if index != generics.len() - 1 {
                            parts.push(str_part(", "))
                        }
                    }
                    parts.push(str_part(">"));
                }
            }
            Type::Function(args, return_type, _env) => {
                parts.push(str_part("fn("));
                for (index, arg) in args.iter().enumerate() {
                    self.push_type_parts(arg, parts);
                    if index != args.len() - 1 {
                        parts.push(str_part(", "))
                    }
                }
                parts.push(str_part(") -> "));
                self.push_type_parts(return_type, parts);
            }
            Type::MutableReference(typ) => {
                parts.push(str_part("&mut "));
                self.push_type_parts(typ, parts);
            }
            Type::TypeVariable(var, TypeVariableKind::Normal) => {
                self.push_type_variable_parts(var, parts);
            }
            Type::TypeVariable(binding, TypeVariableKind::Integer) => {
                if let TypeBinding::Unbound(_) = &*binding.borrow() {
                    self.push_type_parts(&Type::default_int_type(), parts)
                } else {
                    self.push_type_variable_parts(binding, parts);
                }
            }
            Type::TypeVariable(binding, TypeVariableKind::IntegerOrField) => {
                if let TypeBinding::Unbound(_) = &*binding.borrow() {
                    parts.push(str_part("Field"));
                } else {
                    self.push_type_variable_parts(binding, parts);
                }
            }
            Type::TypeVariable(binding, TypeVariableKind::Constant(n)) => {
                if let TypeBinding::Unbound(_) = &*binding.borrow() {
                    // TypeVariableKind::Constant(n) binds to Type::Constant(n) by default, so just show that.
                    parts.push(string_part(n.to_string()));
                } else {
                    self.push_type_variable_parts(binding, parts);
                }
            }

            Type::FieldElement
            | Type::Integer(..)
            | Type::Bool
            | Type::String(..)
            | Type::FmtString(..)
            | Type::Unit
            | Type::TraitAsType(..)
            | Type::NamedGeneric(..)
            | Type::Forall(..)
            | Type::Constant(..)
            | Type::Quoted(..)
            | Type::Error => {
                parts.push(string_part(typ.to_string()));
            }
        }
    }
    fn push_type_variable_parts(&self, var: &TypeVariable, parts: &mut Vec<InlayHintLabelPart>) {
        let var = &*var.borrow();
        match var {
            TypeBinding::Bound(typ) => {
                self.push_type_parts(&typ, parts);
            }
            TypeBinding::Unbound(..) => {
                parts.push(string_part(var.to_string()));
            }
        }
    }

    fn text_part_with_location(&self, str: String, location: Location) -> InlayHintLabelPart {
        InlayHintLabelPart {
            value: str,
            location: to_lsp_location(self.args.files, location.file, location.span),
            tooltip: None,
            command: None,
        }
    }
}

fn string_part(str: String) -> InlayHintLabelPart {
    InlayHintLabelPart { value: str, location: None, tooltip: None, command: None }
}

fn str_part(str: &str) -> InlayHintLabelPart {
    string_part(str.to_string())
}
