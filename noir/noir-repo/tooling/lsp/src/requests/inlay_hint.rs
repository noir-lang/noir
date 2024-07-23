use fm::codespan_files::Files;
use std::future::{self, Future};

use async_lsp::ResponseError;
use fm::{FileId, FileMap, PathString};
use lsp_types::{
    InlayHint, InlayHintKind, InlayHintLabel, InlayHintLabelPart, InlayHintParams, Position,
    TextDocumentPositionParams,
};
use noirc_errors::{Location, Span};
use noirc_frontend::{
    self,
    ast::{
        BlockExpression, Expression, ExpressionKind, Ident, LetStatement, NoirFunction, Pattern,
        Statement, StatementKind, TraitImplItem, TraitItem, UnresolvedTypeData,
    },
    hir_def::stmt::HirPattern,
    macros_api::NodeInterner,
    node_interner::ReferenceId,
    parser::{Item, ItemKind},
    ParsedModule, Type, TypeBinding, TypeVariable, TypeVariableKind,
};

use crate::LspState;

use super::{process_request, to_lsp_location, InlayHintsOptions};

pub(crate) fn on_inlay_hint_request(
    state: &mut LspState,
    params: InlayHintParams,
) -> impl Future<Output = Result<Option<Vec<InlayHint>>, ResponseError>> {
    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document.clone(),
        position: Position { line: 0, character: 0 },
    };

    let options = state.options.inlay_hints;

    let result = process_request(state, text_document_position_params, |args| {
        let path = PathString::from_path(params.text_document.uri.to_file_path().unwrap());
        args.files.get_file_id(&path).map(|file_id| {
            let file = args.files.get_file(file_id).unwrap();
            let source = file.source();
            let (parsed_moduled, _errors) = noirc_frontend::parse_program(source);

            let span = range_to_byte_span(args.files, file_id, &params.range)
                .map(|range| Span::from(range.start as u32..range.end as u32));

            let mut collector =
                InlayHintCollector::new(args.files, file_id, args.interner, span, options);
            collector.collect_in_parsed_module(&parsed_moduled);
            collector.inlay_hints
        })
    });
    future::ready(result)
}

pub(crate) struct InlayHintCollector<'a> {
    files: &'a FileMap,
    file_id: FileId,
    interner: &'a NodeInterner,
    span: Option<Span>,
    options: InlayHintsOptions,
    inlay_hints: Vec<InlayHint>,
}

impl<'a> InlayHintCollector<'a> {
    fn new(
        files: &'a FileMap,
        file_id: FileId,
        interner: &'a NodeInterner,
        span: Option<Span>,
        options: InlayHintsOptions,
    ) -> InlayHintCollector<'a> {
        InlayHintCollector { files, file_id, interner, span, options, inlay_hints: Vec::new() }
    }
    fn collect_in_parsed_module(&mut self, parsed_module: &ParsedModule) {
        for item in &parsed_module.items {
            self.collect_in_item(item);
        }
    }

    fn collect_in_item(&mut self, item: &Item) {
        if !self.intersects_span(item.span) {
            return;
        }

        match &item.kind {
            ItemKind::Function(noir_function) => self.collect_in_noir_function(noir_function),
            ItemKind::Trait(noir_trait) => {
                for item in &noir_trait.items {
                    self.collect_in_trait_item(item);
                }
            }
            ItemKind::TraitImpl(noir_trait_impl) => {
                for item in &noir_trait_impl.items {
                    self.collect_in_trait_impl_item(item);
                }
            }
            ItemKind::Impl(type_impl) => {
                for (noir_function, _) in &type_impl.methods {
                    self.collect_in_noir_function(noir_function);
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
                if let Some(default_value) = default_value {
                    self.collect_in_expression(default_value);
                }
            }
            TraitItem::Type { .. } => (),
        }
    }

    fn collect_in_trait_impl_item(&mut self, item: &TraitImplItem) {
        match item {
            TraitImplItem::Function(noir_function) => self.collect_in_noir_function(noir_function),
            TraitImplItem::Constant(_name, _typ, default_value) => {
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
            self.collect_in_statement(statement);
        }
    }

    fn collect_in_statement(&mut self, statement: &Statement) {
        if !self.intersects_span(statement.span) {
            return;
        }

        match &statement.kind {
            StatementKind::Let(let_statement) => self.collect_in_let_statement(let_statement),
            StatementKind::Constrain(constrain_statement) => {
                self.collect_in_expression(&constrain_statement.0);
            }
            StatementKind::Expression(expression) => self.collect_in_expression(expression),
            StatementKind::Assign(assign_statement) => {
                self.collect_in_expression(&assign_statement.expression);
            }
            StatementKind::For(for_loop_statement) => {
                self.collect_in_ident(&for_loop_statement.identifier);
                self.collect_in_expression(&for_loop_statement.block);
            }
            StatementKind::Comptime(statement) => self.collect_in_statement(statement),
            StatementKind::Semi(expression) => self.collect_in_expression(expression),
            StatementKind::Break => (),
            StatementKind::Continue => (),
            StatementKind::Error => (),
        }
    }

    fn collect_in_expression(&mut self, expression: &Expression) {
        if !self.intersects_span(expression.span) {
            return;
        }

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
                self.collect_call_parameter_names(
                    get_expression_name(&call_expression.func),
                    call_expression.func.span,
                    &call_expression.arguments,
                );

                self.collect_in_expression(&call_expression.func);
                for arg in &call_expression.arguments {
                    self.collect_in_expression(arg);
                }
            }
            ExpressionKind::MethodCall(method_call_expression) => {
                self.collect_call_parameter_names(
                    Some(method_call_expression.method_name.to_string()),
                    method_call_expression.method_name.span(),
                    &method_call_expression.arguments,
                );

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
            ExpressionKind::Tuple(expressions) => {
                for expression in expressions {
                    self.collect_in_expression(expression);
                }
            }
            ExpressionKind::Lambda(lambda) => self.collect_in_expression(&lambda.body),
            ExpressionKind::Parenthesized(parenthesized) => {
                self.collect_in_expression(parenthesized);
            }
            ExpressionKind::Unquote(expression) => {
                self.collect_in_expression(expression);
            }
            ExpressionKind::Comptime(block_expression, _span) => {
                self.collect_in_block_expression(block_expression);
            }
            ExpressionKind::Literal(..)
            | ExpressionKind::Variable(..)
            | ExpressionKind::Quote(..)
            | ExpressionKind::Resolved(..)
            | ExpressionKind::Error => (),
        }
    }

    fn collect_in_pattern(&mut self, pattern: &Pattern) {
        if !self.options.type_hints.enabled {
            return;
        }

        match pattern {
            Pattern::Identifier(ident) => {
                self.collect_in_ident(ident);
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

    fn collect_in_ident(&mut self, ident: &Ident) {
        if !self.options.type_hints.enabled {
            return;
        }

        let span = ident.span();
        let location = Location::new(ident.span(), self.file_id);
        if let Some(lsp_location) = to_lsp_location(self.files, self.file_id, span) {
            if let Some(referenced) = self.interner.find_referenced(location) {
                match referenced {
                    ReferenceId::Global(global_id) => {
                        let global_info = self.interner.get_global(global_id);
                        let definition_id = global_info.definition_id;
                        let typ = self.interner.definition_type(definition_id);
                        self.push_type_hint(lsp_location, &typ);
                    }
                    ReferenceId::Local(definition_id) => {
                        let typ = self.interner.definition_type(definition_id);
                        self.push_type_hint(lsp_location, &typ);
                    }
                    ReferenceId::StructMember(struct_id, field_index) => {
                        let struct_type = self.interner.get_struct(struct_id);
                        let struct_type = struct_type.borrow();
                        let (_field_name, field_type) = struct_type.field_at(field_index);
                        self.push_type_hint(lsp_location, field_type);
                    }
                    ReferenceId::Module(_)
                    | ReferenceId::Struct(_)
                    | ReferenceId::Trait(_)
                    | ReferenceId::Function(_)
                    | ReferenceId::Alias(_)
                    | ReferenceId::Reference(..) => (),
                }
            }
        }
    }

    fn push_type_hint(&mut self, location: lsp_types::Location, typ: &Type) {
        let position = location.range.end;

        let mut parts = Vec::new();
        parts.push(string_part(": "));
        push_type_parts(typ, &mut parts, self.files);

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

    fn collect_call_parameter_names(
        &mut self,
        function_name: Option<String>,
        at: Span,
        arguments: &[Expression],
    ) {
        if !self.options.parameter_hints.enabled {
            return;
        }

        // The `at` span might be the span of a path like `Foo::bar`.
        // In order to find the function behind it, we use a span that is just the last char.
        let at = Span::single_char(at.end() - 1);

        let referenced = self.interner.find_referenced(Location::new(at, self.file_id));
        if let Some(ReferenceId::Function(func_id)) = referenced {
            let func_meta = self.interner.function_meta(&func_id);

            let mut parameters = func_meta.parameters.iter().peekable();
            let mut parameters_count = func_meta.parameters.len();

            // Skip `self` parameter
            if let Some((pattern, _, _)) = parameters.peek() {
                if self.is_self_parameter(pattern) {
                    parameters.next();
                    parameters_count -= 1;
                }
            }

            for (call_argument, (pattern, _, _)) in arguments.iter().zip(parameters) {
                let Some(lsp_location) =
                    to_lsp_location(self.files, self.file_id, call_argument.span)
                else {
                    continue;
                };

                let Some(parameter_name) = self.get_pattern_name(pattern) else {
                    continue;
                };

                if parameter_name.starts_with('_') {
                    continue;
                }

                if parameters_count == 1 {
                    if parameter_name.len() == 1
                        || parameter_name == "other"
                        || parameter_name == "value"
                    {
                        continue;
                    }

                    if let Some(function_name) = &function_name {
                        if function_name.ends_with(&parameter_name) {
                            continue;
                        }
                    }
                }

                if let Some(call_argument_name) = get_expression_name(call_argument) {
                    if parameter_name == call_argument_name
                        || call_argument_name.ends_with(&parameter_name)
                    {
                        continue;
                    }
                }

                self.push_parameter_hint(lsp_location.range.start, &parameter_name);
            }
        }
    }

    fn get_pattern_name(&self, pattern: &HirPattern) -> Option<String> {
        match pattern {
            HirPattern::Identifier(ident) => {
                let definition = self.interner.definition(ident.id);
                Some(definition.name.clone())
            }
            HirPattern::Mutable(pattern, _location) => self.get_pattern_name(pattern),
            HirPattern::Tuple(..) | HirPattern::Struct(..) => None,
        }
    }

    fn push_parameter_hint(&mut self, position: Position, str: &str) {
        self.push_text_hint(position, format!("{}: ", str));
    }

    fn push_text_hint(&mut self, position: Position, str: String) {
        self.inlay_hints.push(InlayHint {
            position,
            label: InlayHintLabel::String(str),
            kind: Some(InlayHintKind::PARAMETER),
            text_edits: None,
            tooltip: None,
            padding_left: None,
            padding_right: None,
            data: None,
        });
    }

    fn is_self_parameter(&self, pattern: &HirPattern) -> bool {
        match pattern {
            HirPattern::Identifier(ident) => {
                let definition_info = self.interner.definition(ident.id);
                definition_info.name == "self"
            }
            HirPattern::Mutable(pattern, _location) => self.is_self_parameter(pattern),
            HirPattern::Tuple(..) | HirPattern::Struct(..) => false,
        }
    }

    fn intersects_span(&self, other_span: Span) -> bool {
        self.span.map_or(true, |span| span.intersects(&other_span))
    }
}

fn string_part(str: impl Into<String>) -> InlayHintLabelPart {
    InlayHintLabelPart { value: str.into(), location: None, tooltip: None, command: None }
}

fn text_part_with_location(str: String, location: Location, files: &FileMap) -> InlayHintLabelPart {
    InlayHintLabelPart {
        value: str,
        location: to_lsp_location(files, location.file, location.span),
        tooltip: None,
        command: None,
    }
}

fn push_type_parts(typ: &Type, parts: &mut Vec<InlayHintLabelPart>, files: &FileMap) {
    match typ {
        Type::Array(size, typ) => {
            parts.push(string_part("["));
            push_type_parts(typ, parts, files);
            parts.push(string_part("; "));
            push_type_parts(size, parts, files);
            parts.push(string_part("]"));
        }
        Type::Slice(typ) => {
            parts.push(string_part("["));
            push_type_parts(typ, parts, files);
            parts.push(string_part("]"));
        }
        Type::Tuple(types) => {
            parts.push(string_part("("));
            for (index, typ) in types.iter().enumerate() {
                push_type_parts(typ, parts, files);
                if index != types.len() - 1 {
                    parts.push(string_part(", "));
                }
            }
            parts.push(string_part(")"));
        }
        Type::Struct(struct_type, generics) => {
            let struct_type = struct_type.borrow();
            let location = Location::new(struct_type.name.span(), struct_type.location.file);
            parts.push(text_part_with_location(struct_type.name.to_string(), location, files));
            if !generics.is_empty() {
                parts.push(string_part("<"));
                for (index, generic) in generics.iter().enumerate() {
                    push_type_parts(generic, parts, files);
                    if index != generics.len() - 1 {
                        parts.push(string_part(", "));
                    }
                }
                parts.push(string_part(">"));
            }
        }
        Type::Alias(type_alias, generics) => {
            let type_alias = type_alias.borrow();
            let location = Location::new(type_alias.name.span(), type_alias.location.file);
            parts.push(text_part_with_location(type_alias.name.to_string(), location, files));
            if !generics.is_empty() {
                parts.push(string_part("<"));
                for (index, generic) in generics.iter().enumerate() {
                    push_type_parts(generic, parts, files);
                    if index != generics.len() - 1 {
                        parts.push(string_part(", "));
                    }
                }
                parts.push(string_part(">"));
            }
        }
        Type::Function(args, return_type, _env) => {
            parts.push(string_part("fn("));
            for (index, arg) in args.iter().enumerate() {
                push_type_parts(arg, parts, files);
                if index != args.len() - 1 {
                    parts.push(string_part(", "));
                }
            }
            parts.push(string_part(") -> "));
            push_type_parts(return_type, parts, files);
        }
        Type::MutableReference(typ) => {
            parts.push(string_part("&mut "));
            push_type_parts(typ, parts, files);
        }
        Type::TypeVariable(var, TypeVariableKind::Normal) => {
            push_type_variable_parts(var, parts, files);
        }
        Type::TypeVariable(binding, TypeVariableKind::Integer) => {
            if let TypeBinding::Unbound(_) = &*binding.borrow() {
                push_type_parts(&Type::default_int_type(), parts, files);
            } else {
                push_type_variable_parts(binding, parts, files);
            }
        }
        Type::TypeVariable(binding, TypeVariableKind::IntegerOrField) => {
            if let TypeBinding::Unbound(_) = &*binding.borrow() {
                parts.push(string_part("Field"));
            } else {
                push_type_variable_parts(binding, parts, files);
            }
        }
        Type::TypeVariable(binding, TypeVariableKind::Constant(n)) => {
            if let TypeBinding::Unbound(_) = &*binding.borrow() {
                // TypeVariableKind::Constant(n) binds to Type::Constant(n) by default, so just show that.
                parts.push(string_part(n.to_string()));
            } else {
                push_type_variable_parts(binding, parts, files);
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

fn push_type_variable_parts(
    var: &TypeVariable,
    parts: &mut Vec<InlayHintLabelPart>,
    files: &FileMap,
) {
    let var = &*var.borrow();
    match var {
        TypeBinding::Bound(typ) => {
            push_type_parts(typ, parts, files);
        }
        TypeBinding::Unbound(..) => {
            parts.push(string_part(var.to_string()));
        }
    }
}

fn get_expression_name(expression: &Expression) -> Option<String> {
    match &expression.kind {
        ExpressionKind::Variable(path, _) => Some(path.last_segment().to_string()),
        ExpressionKind::Prefix(prefix) => get_expression_name(&prefix.rhs),
        ExpressionKind::MemberAccess(member_access) => Some(member_access.rhs.to_string()),
        ExpressionKind::Call(call) => get_expression_name(&call.func),
        ExpressionKind::MethodCall(method_call) => Some(method_call.method_name.to_string()),
        ExpressionKind::Cast(cast) => get_expression_name(&cast.lhs),
        ExpressionKind::Parenthesized(expr) => get_expression_name(expr),
        ExpressionKind::Constructor(..)
        | ExpressionKind::Infix(..)
        | ExpressionKind::Index(..)
        | ExpressionKind::Block(..)
        | ExpressionKind::If(..)
        | ExpressionKind::Lambda(..)
        | ExpressionKind::Tuple(..)
        | ExpressionKind::Quote(..)
        | ExpressionKind::Unquote(..)
        | ExpressionKind::Comptime(..)
        | ExpressionKind::Resolved(..)
        | ExpressionKind::Literal(..)
        | ExpressionKind::Error => None,
    }
}

// These functions are copied from the codespan_lsp crate, except that they never panic
// (the library will sometimes panic, so functions returning Result are not always accurate)

fn range_to_byte_span(
    files: &FileMap,
    file_id: FileId,
    range: &lsp_types::Range,
) -> Option<std::ops::Range<usize>> {
    Some(
        position_to_byte_index(files, file_id, &range.start)?
            ..position_to_byte_index(files, file_id, &range.end)?,
    )
}

fn position_to_byte_index(
    files: &FileMap,
    file_id: FileId,
    position: &lsp_types::Position,
) -> Option<usize> {
    let Ok(source) = files.source(file_id) else {
        return None;
    };

    let Ok(line_span) = files.line_range(file_id, position.line as usize) else {
        return None;
    };
    let line_str = source.get(line_span.clone())?;

    let byte_offset = character_to_line_offset(line_str, position.character)?;

    Some(line_span.start + byte_offset)
}

fn character_to_line_offset(line: &str, character: u32) -> Option<usize> {
    let line_len = line.len();
    let mut character_offset = 0;

    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if character_offset == character {
            let chars_off = chars.as_str().len();
            let ch_off = ch.len_utf8();

            return Some(line_len - chars_off - ch_off);
        }

        character_offset += ch.len_utf16() as u32;
    }

    // Handle positions after the last character on the line
    if character_offset == character {
        Some(line_len)
    } else {
        None
    }
}

#[cfg(test)]
mod inlay_hints_tests {
    use crate::{
        requests::{ParameterHintsOptions, TypeHintsOptions},
        test_utils,
    };

    use super::*;
    use lsp_types::{Range, TextDocumentIdentifier, WorkDoneProgressParams};
    use tokio::test;

    async fn get_inlay_hints(
        start_line: u32,
        end_line: u32,
        options: InlayHintsOptions,
    ) -> Vec<InlayHint> {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("inlay_hints").await;
        state.options.inlay_hints = options;

        on_inlay_hint_request(
            &mut state,
            InlayHintParams {
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                text_document: TextDocumentIdentifier { uri: noir_text_document },
                range: lsp_types::Range {
                    start: lsp_types::Position { line: start_line, character: 0 },
                    end: lsp_types::Position { line: end_line, character: 0 },
                },
            },
        )
        .await
        .expect("Could not execute on_inlay_hint_request")
        .unwrap()
    }

    fn no_hints() -> InlayHintsOptions {
        InlayHintsOptions {
            type_hints: TypeHintsOptions { enabled: false },
            parameter_hints: ParameterHintsOptions { enabled: false },
        }
    }

    fn type_hints() -> InlayHintsOptions {
        InlayHintsOptions {
            type_hints: TypeHintsOptions { enabled: true },
            parameter_hints: ParameterHintsOptions { enabled: false },
        }
    }

    fn parameter_hints() -> InlayHintsOptions {
        InlayHintsOptions {
            type_hints: TypeHintsOptions { enabled: false },
            parameter_hints: ParameterHintsOptions { enabled: true },
        }
    }

    #[test]
    async fn test_do_not_collect_type_hints_if_disabled() {
        let inlay_hints = get_inlay_hints(0, 3, no_hints()).await;
        assert!(inlay_hints.is_empty());
    }

    #[test]
    async fn test_type_inlay_hints_without_location() {
        let inlay_hints = get_inlay_hints(0, 3, type_hints()).await;
        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 1, character: 11 });

        if let InlayHintLabel::LabelParts(labels) = &inlay_hint.label {
            assert_eq!(labels.len(), 2);
            assert_eq!(labels[0].value, ": ");
            assert_eq!(labels[0].location, None);
            assert_eq!(labels[1].value, "Field");

            // Field can't be reached (there's no source code for it)
            assert_eq!(labels[1].location, None);
        } else {
            panic!("Expected InlayHintLabel::LabelParts, got {:?}", inlay_hint.label);
        }
    }

    #[test]
    async fn test_type_inlay_hints_with_location() {
        let inlay_hints = get_inlay_hints(12, 15, type_hints()).await;
        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 13, character: 11 });

        if let InlayHintLabel::LabelParts(labels) = &inlay_hint.label {
            assert_eq!(labels.len(), 2);
            assert_eq!(labels[0].value, ": ");
            assert_eq!(labels[0].location, None);
            assert_eq!(labels[1].value, "Foo");

            // Check that it points to "Foo" in `struct Foo`
            let location = labels[1].location.clone().expect("Expected a location");
            assert_eq!(
                location.range,
                Range {
                    start: Position { line: 4, character: 7 },
                    end: Position { line: 4, character: 10 }
                }
            );
        } else {
            panic!("Expected InlayHintLabel::LabelParts, got {:?}", inlay_hint.label);
        }
    }

    #[test]
    async fn test_type_inlay_hints_in_for() {
        let inlay_hints = get_inlay_hints(16, 18, type_hints()).await;
        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 17, character: 9 });

        if let InlayHintLabel::LabelParts(labels) = &inlay_hint.label {
            assert_eq!(labels.len(), 2);
            assert_eq!(labels[0].value, ": ");
            assert_eq!(labels[0].location, None);
            assert_eq!(labels[1].value, "u32");
        } else {
            panic!("Expected InlayHintLabel::LabelParts, got {:?}", inlay_hint.label);
        }
    }

    #[test]
    async fn test_type_inlay_hints_in_global() {
        let inlay_hints = get_inlay_hints(19, 21, type_hints()).await;
        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 20, character: 10 });

        if let InlayHintLabel::LabelParts(labels) = &inlay_hint.label {
            assert_eq!(labels.len(), 2);
            assert_eq!(labels[0].value, ": ");
            assert_eq!(labels[0].location, None);
            assert_eq!(labels[1].value, "Field");
        } else {
            panic!("Expected InlayHintLabel::LabelParts, got {:?}", inlay_hint.label);
        }
    }

    #[test]
    async fn test_do_not_panic_when_given_line_is_too_big() {
        let inlay_hints = get_inlay_hints(0, 100000, type_hints()).await;
        assert!(!inlay_hints.is_empty());
    }

    #[test]
    async fn test_do_not_collect_parameter_inlay_hints_if_disabled() {
        let inlay_hints = get_inlay_hints(24, 26, no_hints()).await;
        assert!(inlay_hints.is_empty());
    }

    #[test]
    async fn test_collect_parameter_inlay_hints_in_function_call() {
        let inlay_hints = get_inlay_hints(24, 26, parameter_hints()).await;
        assert_eq!(inlay_hints.len(), 2);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 25, character: 12 });
        if let InlayHintLabel::String(label) = &inlay_hint.label {
            assert_eq!(label, "one: ");
        } else {
            panic!("Expected InlayHintLabel::String, got {:?}", inlay_hint.label);
        }

        let inlay_hint = &inlay_hints[1];
        assert_eq!(inlay_hint.position, Position { line: 25, character: 15 });
        if let InlayHintLabel::String(label) = &inlay_hint.label {
            assert_eq!(label, "two: ");
        } else {
            panic!("Expected InlayHintLabel::String, got {:?}", inlay_hint.label);
        }
    }

    #[test]
    async fn test_collect_parameter_inlay_hints_in_method_call() {
        let inlay_hints = get_inlay_hints(36, 39, parameter_hints()).await;
        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 38, character: 18 });
        if let InlayHintLabel::String(label) = &inlay_hint.label {
            assert_eq!(label, "one: ");
        } else {
            panic!("Expected InlayHintLabel::String, got {:?}", inlay_hint.label);
        }
    }

    #[test]
    async fn test_do_not_show_parameter_inlay_hints_if_name_matches_var_name() {
        let inlay_hints = get_inlay_hints(41, 45, parameter_hints()).await;
        assert!(inlay_hints.is_empty());
    }

    #[test]
    async fn test_do_not_show_parameter_inlay_hints_if_name_matches_member_name() {
        let inlay_hints = get_inlay_hints(48, 52, parameter_hints()).await;
        assert!(inlay_hints.is_empty());
    }

    #[test]
    async fn test_do_not_show_parameter_inlay_hints_if_name_matches_call_name() {
        let inlay_hints = get_inlay_hints(57, 60, parameter_hints()).await;
        assert!(inlay_hints.is_empty());
    }

    #[test]
    async fn test_do_not_show_parameter_inlay_hints_if_single_param_name_is_suffix_of_function_name(
    ) {
        let inlay_hints = get_inlay_hints(64, 67, parameter_hints()).await;
        assert!(inlay_hints.is_empty());
    }

    #[test]
    async fn test_do_not_show_parameter_inlay_hints_if_param_name_starts_with_underscore() {
        let inlay_hints = get_inlay_hints(71, 73, parameter_hints()).await;
        assert!(inlay_hints.is_empty());
    }

    #[test]
    async fn test_do_not_show_parameter_inlay_hints_if_single_argument_with_single_letter() {
        let inlay_hints = get_inlay_hints(77, 79, parameter_hints()).await;
        assert!(inlay_hints.is_empty());
    }

    #[test]
    async fn test_do_not_show_parameter_inlay_hints_if_param_name_is_suffix_of_arg_name() {
        let inlay_hints = get_inlay_hints(89, 92, parameter_hints()).await;
        assert!(inlay_hints.is_empty());
    }
}
