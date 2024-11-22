use std::future::{self, Future};

use async_lsp::ResponseError;
use fm::{FileId, FileMap, PathString};
use lsp_types::{
    InlayHint, InlayHintKind, InlayHintLabel, InlayHintLabelPart, InlayHintParams, Position, Range,
    TextDocumentPositionParams, TextEdit,
};
use noirc_errors::{Location, Span};
use noirc_frontend::{
    self,
    ast::{
        CallExpression, Expression, ExpressionKind, ForLoopStatement, Ident, Lambda, LetStatement,
        MethodCallExpression, NoirFunction, NoirTraitImpl, Pattern, Statement, TypeImpl,
        UnresolvedTypeData, Visitor,
    },
    hir_def::stmt::HirPattern,
    node_interner::{NodeInterner, ReferenceId},
    parser::{Item, ParsedSubModule},
    Kind, Type, TypeBinding, TypeVariable,
};

use crate::{utils, LspState};

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

            let span = utils::range_to_byte_span(args.files, file_id, &params.range)
                .map(|range| Span::from(range.start as u32..range.end as u32));

            let mut collector =
                InlayHintCollector::new(args.files, file_id, args.interner, span, options);
            parsed_moduled.accept(&mut collector);
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

    fn collect_in_ident(&mut self, ident: &Ident, editable: bool) {
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
                        self.push_type_hint(lsp_location, &typ, editable);
                    }
                    ReferenceId::Local(definition_id) => {
                        let typ = self.interner.definition_type(definition_id);
                        self.push_type_hint(lsp_location, &typ, editable);
                    }
                    ReferenceId::StructMember(struct_id, field_index) => {
                        let struct_type = self.interner.get_struct(struct_id);
                        let struct_type = struct_type.borrow();
                        let field = struct_type.field_at(field_index);
                        self.push_type_hint(lsp_location, &field.typ, false);
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

    fn push_type_hint(&mut self, location: lsp_types::Location, typ: &Type, editable: bool) {
        let position = location.range.end;

        let mut parts = Vec::new();
        parts.push(string_part(": "));
        push_type_parts(typ, &mut parts, self.files);

        self.inlay_hints.push(InlayHint {
            position,
            label: InlayHintLabel::LabelParts(parts),
            kind: Some(InlayHintKind::TYPE),
            text_edits: if editable {
                Some(vec![TextEdit {
                    range: Range { start: location.range.end, end: location.range.end },
                    new_text: format!(": {}", typ),
                }])
            } else {
                None
            },
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

    fn show_closing_brace_hint<F>(&mut self, span: Span, f: F)
    where
        F: FnOnce() -> String,
    {
        if self.options.closing_brace_hints.enabled {
            if let Some(lsp_location) = to_lsp_location(self.files, self.file_id, span) {
                let lines = lsp_location.range.end.line - lsp_location.range.start.line + 1;
                if lines >= self.options.closing_brace_hints.min_lines {
                    self.push_text_hint(lsp_location.range.end, f());
                }
            }
        }
    }
}

impl<'a> Visitor for InlayHintCollector<'a> {
    fn visit_item(&mut self, item: &Item) -> bool {
        self.intersects_span(item.span)
    }

    fn visit_noir_trait_impl(&mut self, noir_trait_impl: &NoirTraitImpl, span: Span) -> bool {
        self.show_closing_brace_hint(span, || {
            format!(" impl {} for {}", noir_trait_impl.trait_name, noir_trait_impl.object_type)
        });

        true
    }

    fn visit_type_impl(&mut self, type_impl: &TypeImpl, span: Span) -> bool {
        self.show_closing_brace_hint(span, || format!(" impl {}", type_impl.object_type));

        true
    }

    fn visit_parsed_submodule(&mut self, parsed_submodule: &ParsedSubModule, span: Span) -> bool {
        self.show_closing_brace_hint(span, || {
            if parsed_submodule.is_contract {
                format!(" contract {}", parsed_submodule.name)
            } else {
                format!(" mod {}", parsed_submodule.name)
            }
        });

        true
    }

    fn visit_noir_function(&mut self, noir_function: &NoirFunction, span: Span) -> bool {
        self.show_closing_brace_hint(span, || format!(" fn {}", noir_function.def.name));

        true
    }

    fn visit_statement(&mut self, statement: &Statement) -> bool {
        self.intersects_span(statement.span)
    }

    fn visit_let_statement(&mut self, let_statement: &LetStatement) -> bool {
        // Only show inlay hints for let variables that don't have an explicit type annotation
        if let UnresolvedTypeData::Unspecified = let_statement.r#type.typ {
            let_statement.pattern.accept(self);
        };

        let_statement.expression.accept(self);

        false
    }

    fn visit_for_loop_statement(&mut self, for_loop_statement: &ForLoopStatement) -> bool {
        self.collect_in_ident(&for_loop_statement.identifier, false);
        true
    }

    fn visit_expression(&mut self, expression: &Expression) -> bool {
        self.intersects_span(expression.span)
    }

    fn visit_call_expression(&mut self, call_expression: &CallExpression, _: Span) -> bool {
        self.collect_call_parameter_names(
            get_expression_name(&call_expression.func),
            call_expression.func.span,
            &call_expression.arguments,
        );

        true
    }

    fn visit_method_call_expression(
        &mut self,
        method_call_expression: &MethodCallExpression,
        _: Span,
    ) -> bool {
        self.collect_call_parameter_names(
            Some(method_call_expression.method_name.to_string()),
            method_call_expression.method_name.span(),
            &method_call_expression.arguments,
        );

        true
    }

    fn visit_lambda(&mut self, lambda: &Lambda, _: Span) -> bool {
        for (pattern, typ) in &lambda.parameters {
            if matches!(typ.typ, UnresolvedTypeData::Unspecified) {
                pattern.accept(self);
            }
        }

        lambda.body.accept(self);

        false
    }

    fn visit_pattern(&mut self, _: &Pattern) -> bool {
        self.options.type_hints.enabled
    }

    fn visit_identifier_pattern(&mut self, ident: &Ident) {
        self.collect_in_ident(ident, true);
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
        Type::Function(args, return_type, _env, unconstrained) => {
            if *unconstrained {
                parts.push(string_part("unconstrained "));
            }

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
        Type::TypeVariable(binding) => {
            if let TypeBinding::Unbound(_, kind) = &*binding.borrow() {
                match kind {
                    Kind::Any | Kind::Normal => push_type_variable_parts(binding, parts, files),
                    Kind::Integer => push_type_parts(&Type::default_int_type(), parts, files),
                    Kind::IntegerOrField => parts.push(string_part("Field")),
                    Kind::Numeric(ref typ) => push_type_parts(typ, parts, files),
                }
            } else {
                push_type_variable_parts(binding, parts, files);
            }
        }
        Type::CheckedCast { to, .. } => push_type_parts(to, parts, files),

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
        | Type::InfixExpr(..)
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
        ExpressionKind::Variable(path) => Some(path.last_name().to_string()),
        ExpressionKind::Prefix(prefix) => get_expression_name(&prefix.rhs),
        ExpressionKind::MemberAccess(member_access) => Some(member_access.rhs.to_string()),
        ExpressionKind::Call(call) => get_expression_name(&call.func),
        ExpressionKind::MethodCall(method_call) => Some(method_call.method_name.to_string()),
        ExpressionKind::Cast(cast) => get_expression_name(&cast.lhs),
        ExpressionKind::Parenthesized(expr) => get_expression_name(expr),
        ExpressionKind::AsTraitPath(path) => Some(path.impl_item.to_string()),
        ExpressionKind::TypePath(path) => Some(path.item.to_string()),
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
        | ExpressionKind::Interned(..)
        | ExpressionKind::InternedStatement(..)
        | ExpressionKind::Literal(..)
        | ExpressionKind::Unsafe(..)
        | ExpressionKind::Error => None,
    }
}

#[cfg(test)]
mod inlay_hints_tests {
    use crate::{
        requests::{ClosingBraceHintsOptions, ParameterHintsOptions, TypeHintsOptions},
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
                range: Range {
                    start: Position { line: start_line, character: 0 },
                    end: Position { line: end_line, character: 0 },
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
            closing_brace_hints: ClosingBraceHintsOptions { enabled: false, min_lines: 25 },
        }
    }

    fn type_hints() -> InlayHintsOptions {
        InlayHintsOptions {
            type_hints: TypeHintsOptions { enabled: true },
            parameter_hints: ParameterHintsOptions { enabled: false },
            closing_brace_hints: ClosingBraceHintsOptions { enabled: false, min_lines: 25 },
        }
    }

    fn parameter_hints() -> InlayHintsOptions {
        InlayHintsOptions {
            type_hints: TypeHintsOptions { enabled: false },
            parameter_hints: ParameterHintsOptions { enabled: true },
            closing_brace_hints: ClosingBraceHintsOptions { enabled: false, min_lines: 25 },
        }
    }

    fn closing_braces_hints(min_lines: u32) -> InlayHintsOptions {
        InlayHintsOptions {
            type_hints: TypeHintsOptions { enabled: false },
            parameter_hints: ParameterHintsOptions { enabled: false },
            closing_brace_hints: ClosingBraceHintsOptions { enabled: true, min_lines },
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

        let position = Position { line: 1, character: 11 };

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, position);

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

        assert_eq!(
            inlay_hint.text_edits,
            Some(vec![TextEdit {
                range: Range { start: position, end: position },
                new_text: ": Field".to_string(),
            }])
        );
    }

    #[test]
    async fn test_type_inlay_hints_with_location() {
        let inlay_hints = get_inlay_hints(12, 15, type_hints()).await;
        assert_eq!(inlay_hints.len(), 1);

        let position = Position { line: 13, character: 11 };

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, position);

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

        assert_eq!(
            inlay_hint.text_edits,
            Some(vec![TextEdit {
                range: Range { start: position, end: position },
                new_text: ": Foo".to_string(),
            }])
        );
    }

    #[test]
    async fn test_type_inlay_hints_in_struct_member_pattern() {
        let inlay_hints = get_inlay_hints(94, 96, type_hints()).await;
        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 95, character: 24 });

        if let InlayHintLabel::LabelParts(labels) = &inlay_hint.label {
            assert_eq!(labels.len(), 2);
            assert_eq!(labels[0].value, ": ");
            assert_eq!(labels[0].location, None);
            assert_eq!(labels[1].value, "i32");
        } else {
            panic!("Expected InlayHintLabel::LabelParts, got {:?}", inlay_hint.label);
        }

        assert_eq!(inlay_hint.text_edits, None);
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

        assert_eq!(inlay_hint.text_edits, None);
    }

    #[test]
    async fn test_type_inlay_hints_in_global() {
        let inlay_hints = get_inlay_hints(19, 21, type_hints()).await;
        assert_eq!(inlay_hints.len(), 1);

        let position = Position { line: 20, character: 10 };

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, position);

        if let InlayHintLabel::LabelParts(labels) = &inlay_hint.label {
            assert_eq!(labels.len(), 2);
            assert_eq!(labels[0].value, ": ");
            assert_eq!(labels[0].location, None);
            assert_eq!(labels[1].value, "Field");
        } else {
            panic!("Expected InlayHintLabel::LabelParts, got {:?}", inlay_hint.label);
        }

        assert_eq!(
            inlay_hint.text_edits,
            Some(vec![TextEdit {
                range: Range { start: position, end: position },
                new_text: ": Field".to_string(),
            }])
        );
    }

    #[test]
    async fn test_type_inlay_hints_in_lambda() {
        let inlay_hints = get_inlay_hints(102, 105, type_hints()).await;
        assert_eq!(inlay_hints.len(), 1);

        let position = Position { line: 104, character: 35 };

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, position);

        if let InlayHintLabel::LabelParts(labels) = &inlay_hint.label {
            assert_eq!(labels.len(), 2);
            assert_eq!(labels[0].value, ": ");
            assert_eq!(labels[0].location, None);
            assert_eq!(labels[1].value, "i32");
        } else {
            panic!("Expected InlayHintLabel::LabelParts, got {:?}", inlay_hint.label);
        }

        assert_eq!(
            inlay_hint.text_edits,
            Some(vec![TextEdit {
                range: Range { start: position, end: position },
                new_text: ": i32".to_string(),
            }])
        );
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
        assert_eq!(inlay_hint.text_edits, None);
        if let InlayHintLabel::String(label) = &inlay_hint.label {
            assert_eq!(label, "one: ");
        } else {
            panic!("Expected InlayHintLabel::String, got {:?}", inlay_hint.label);
        }

        let inlay_hint = &inlay_hints[1];
        assert_eq!(inlay_hint.position, Position { line: 25, character: 15 });
        assert_eq!(inlay_hint.text_edits, None);
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
        assert_eq!(inlay_hint.text_edits, None);
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

    #[test]
    async fn test_does_not_show_closing_brace_inlay_hints_if_disabled() {
        let inlay_hints = get_inlay_hints(41, 46, no_hints()).await;
        assert!(inlay_hints.is_empty());
    }

    #[test]
    async fn test_does_not_show_closing_brace_inlay_hints_if_enabled_but_not_lines() {
        let inlay_hints = get_inlay_hints(41, 46, closing_braces_hints(6)).await;
        assert!(inlay_hints.is_empty());
    }

    #[test]
    async fn test_shows_closing_brace_inlay_hints_for_a_function() {
        let inlay_hints = get_inlay_hints(41, 46, closing_braces_hints(5)).await;
        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 45, character: 1 });
        assert_eq!(inlay_hint.text_edits, None);
        if let InlayHintLabel::String(label) = &inlay_hint.label {
            assert_eq!(label, " fn call_where_name_matches");
        } else {
            panic!("Expected InlayHintLabel::String, got {:?}", inlay_hint.label);
        }
    }

    #[test]
    async fn test_shows_closing_brace_inlay_hints_for_impl() {
        let inlay_hints = get_inlay_hints(32, 34, closing_braces_hints(2)).await;
        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 34, character: 1 });
        assert_eq!(inlay_hint.text_edits, None);
        if let InlayHintLabel::String(label) = &inlay_hint.label {
            assert_eq!(label, " impl SomeStruct");
        } else {
            panic!("Expected InlayHintLabel::String, got {:?}", inlay_hint.label);
        }
    }

    #[test]
    async fn test_shows_closing_brace_inlay_hints_for_trait_impl() {
        let inlay_hints = get_inlay_hints(111, 113, closing_braces_hints(2)).await;
        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 113, character: 1 });
        assert_eq!(inlay_hint.text_edits, None);
        if let InlayHintLabel::String(label) = &inlay_hint.label {
            assert_eq!(label, " impl SomeTrait for SomeStruct");
        } else {
            panic!("Expected InlayHintLabel::String, got {:?}", inlay_hint.label);
        }
    }

    #[test]
    async fn test_shows_closing_brace_inlay_hints_for_module() {
        let inlay_hints = get_inlay_hints(115, 117, closing_braces_hints(2)).await;
        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 117, character: 1 });
        assert_eq!(inlay_hint.text_edits, None);
        if let InlayHintLabel::String(label) = &inlay_hint.label {
            assert_eq!(label, " mod some_module");
        } else {
            panic!("Expected InlayHintLabel::String, got {:?}", inlay_hint.label);
        }
    }

    #[test]
    async fn test_shows_closing_brace_inlay_hints_for_contract() {
        let inlay_hints = get_inlay_hints(119, 121, closing_braces_hints(2)).await;
        assert_eq!(inlay_hints.len(), 1);

        let inlay_hint = &inlay_hints[0];
        assert_eq!(inlay_hint.position, Position { line: 121, character: 1 });
        assert_eq!(inlay_hint.text_edits, None);
        if let InlayHintLabel::String(label) = &inlay_hint.label {
            assert_eq!(label, " contract some_contract");
        } else {
            panic!("Expected InlayHintLabel::String, got {:?}", inlay_hint.label);
        }
    }
}
