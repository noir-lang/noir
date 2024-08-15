use std::future::{self, Future};

use async_lsp::ResponseError;
use fm::{FileId, PathString};
use lsp_types::{
    ParameterInformation, ParameterLabel, SignatureHelp, SignatureHelpParams, SignatureInformation,
};
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ast::{
        ArrayLiteral, BlockExpression, CallExpression, CastExpression, ConstrainStatement,
        ConstructorExpression, Expression, ForLoopStatement, ForRange, FunctionReturnType,
        IfExpression, IndexExpression, InfixExpression, LValue, Lambda, LetStatement, Literal,
        MemberAccessExpression, MethodCallExpression, NoirFunction, NoirTrait, NoirTraitImpl,
        Statement, TraitImplItem, TraitItem, TypeImpl,
    },
    hir_def::{function::FuncMeta, stmt::HirPattern},
    macros_api::NodeInterner,
    node_interner::ReferenceId,
    parser::{Item, ItemKind},
    ParsedModule, Type,
};

use crate::{utils, LspState};

use super::process_request;

pub(crate) fn on_signature_help_request(
    state: &mut LspState,
    params: SignatureHelpParams,
) -> impl Future<Output = Result<Option<SignatureHelp>, ResponseError>> {
    let uri = params.text_document_position_params.clone().text_document.uri;

    let result = process_request(state, params.text_document_position_params.clone(), |args| {
        let path = PathString::from_path(uri.to_file_path().unwrap());
        args.files.get_file_id(&path).and_then(|file_id| {
            utils::position_to_byte_index(
                args.files,
                file_id,
                &params.text_document_position_params.position,
            )
            .and_then(|byte_index| {
                let file = args.files.get_file(file_id).unwrap();
                let source = file.source();
                let (parsed_module, _errors) = noirc_frontend::parse_program(source);

                let mut finder = SignatureFinder::new(file_id, byte_index, args.interner);
                finder.find(&parsed_module)
            })
        })
    });
    future::ready(result)
}

struct SignatureFinder<'a> {
    file: FileId,
    byte_index: usize,
    interner: &'a NodeInterner,
    signature_help: Option<SignatureHelp>,
}

impl<'a> SignatureFinder<'a> {
    fn new(file: FileId, byte_index: usize, interner: &'a NodeInterner) -> Self {
        Self { file, byte_index, interner, signature_help: None }
    }

    fn find(&mut self, parsed_module: &ParsedModule) -> Option<SignatureHelp> {
        self.find_in_parsed_module(parsed_module);

        self.signature_help.clone()
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
            ItemKind::Submodules(parsed_sub_module) => {
                self.find_in_parsed_module(&parsed_sub_module.contents);
            }
            ItemKind::Function(noir_function) => self.find_in_noir_function(noir_function),
            ItemKind::TraitImpl(noir_trait_impl) => self.find_in_noir_trait_impl(noir_trait_impl),
            ItemKind::Impl(type_impl) => self.find_in_type_impl(type_impl),
            ItemKind::Global(let_statement) => self.find_in_let_statement(let_statement),
            ItemKind::Trait(noir_trait) => self.find_in_noir_trait(noir_trait),
            ItemKind::Import(..)
            | ItemKind::TypeAlias(_)
            | ItemKind::Struct(_)
            | ItemKind::ModuleDecl(_) => (),
        }
    }

    fn find_in_noir_function(&mut self, noir_function: &NoirFunction) {
        self.find_in_block_expression(&noir_function.def.body);
    }

    fn find_in_noir_trait_impl(&mut self, noir_trait_impl: &NoirTraitImpl) {
        for item in &noir_trait_impl.items {
            self.find_in_trait_impl_item(item);
        }
    }

    fn find_in_trait_impl_item(&mut self, item: &TraitImplItem) {
        match item {
            TraitImplItem::Function(noir_function) => self.find_in_noir_function(noir_function),
            TraitImplItem::Constant(_, _, _) => (),
            TraitImplItem::Type { .. } => (),
        }
    }

    fn find_in_type_impl(&mut self, type_impl: &TypeImpl) {
        for (method, span) in &type_impl.methods {
            if self.includes_span(*span) {
                self.find_in_noir_function(method);
            }
        }
    }

    fn find_in_noir_trait(&mut self, noir_trait: &NoirTrait) {
        for item in &noir_trait.items {
            self.find_in_trait_item(item);
        }
    }

    fn find_in_trait_item(&mut self, trait_item: &TraitItem) {
        match trait_item {
            TraitItem::Function { body, .. } => {
                if let Some(body) = body {
                    self.find_in_block_expression(body);
                };
            }
            TraitItem::Constant { default_value, .. } => {
                if let Some(default_value) = default_value {
                    self.find_in_expression(default_value);
                }
            }
            TraitItem::Type { .. } => (),
        }
    }

    fn find_in_block_expression(&mut self, block_expression: &BlockExpression) {
        for statement in &block_expression.statements {
            if self.includes_span(statement.span) {
                self.find_in_statement(statement);
            }
        }
    }

    fn find_in_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            noirc_frontend::ast::StatementKind::Let(let_statement) => {
                self.find_in_let_statement(let_statement);
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
                self.find_in_statement(statement);
            }
            noirc_frontend::ast::StatementKind::Semi(expression) => {
                self.find_in_expression(expression);
            }
            noirc_frontend::ast::StatementKind::Break
            | noirc_frontend::ast::StatementKind::Continue
            | noirc_frontend::ast::StatementKind::Error => (),
        }
    }

    fn find_in_let_statement(&mut self, let_statement: &LetStatement) {
        self.find_in_expression(&let_statement.expression);
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
        self.find_in_for_range(&for_loop_statement.range);
        self.find_in_expression(&for_loop_statement.block);
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
                self.find_in_call_expression(call_expression, expression.span);
            }
            noirc_frontend::ast::ExpressionKind::MethodCall(method_call_expression) => {
                self.find_in_method_call_expression(method_call_expression, expression.span);
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
                self.find_in_block_expression(block_expression);
            }
            noirc_frontend::ast::ExpressionKind::Unsafe(block_expression, _) => {
                self.find_in_block_expression(block_expression);
            }
            noirc_frontend::ast::ExpressionKind::Variable(_)
            | noirc_frontend::ast::ExpressionKind::AsTraitPath(_)
            | noirc_frontend::ast::ExpressionKind::Quote(_)
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

    fn find_in_constructor_expression(&mut self, constructor_expression: &ConstructorExpression) {
        for (_field_name, expression) in &constructor_expression.fields {
            self.find_in_expression(expression);
        }
    }

    fn find_in_member_access_expression(
        &mut self,
        member_access_expression: &MemberAccessExpression,
    ) {
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
        self.find_in_expression(&if_expression.consequence);

        if let Some(alternative) = &if_expression.alternative {
            self.find_in_expression(alternative);
        }
    }

    fn find_in_lambda(&mut self, lambda: &Lambda) {
        self.find_in_expression(&lambda.body);
    }

    fn find_in_call_expression(&mut self, call_expression: &CallExpression, span: Span) {
        self.find_in_expression(&call_expression.func);
        self.find_in_expressions(&call_expression.arguments);

        let arguments_span = Span::from(call_expression.func.span.end() + 1..span.end() - 1);
        let span = call_expression.func.span;
        let name_span = Span::from(span.end() - 1..span.end());
        let has_self = false;

        self.try_compute_signature_help(
            &call_expression.arguments,
            arguments_span,
            name_span,
            has_self,
        );
    }

    fn find_in_method_call_expression(
        &mut self,
        method_call_expression: &MethodCallExpression,
        span: Span,
    ) {
        self.find_in_expression(&method_call_expression.object);
        self.find_in_expressions(&method_call_expression.arguments);

        let arguments_span =
            Span::from(method_call_expression.method_name.span().end() + 1..span.end() - 1);
        let name_span = method_call_expression.method_name.span();
        let has_self = true;

        self.try_compute_signature_help(
            &method_call_expression.arguments,
            arguments_span,
            name_span,
            has_self,
        );
    }

    fn try_compute_signature_help(
        &mut self,
        arguments: &[Expression],
        arguments_span: Span,
        name_span: Span,
        has_self: bool,
    ) {
        if self.signature_help.is_some() {
            return;
        }

        if !self.includes_span(arguments_span) {
            return;
        }

        let mut active_parameter = None;
        for (index, arg) in arguments.iter().enumerate() {
            if self.includes_span(arg.span) || arg.span.start() as usize >= self.byte_index {
                active_parameter = Some(index as u32);
                break;
            }
        }

        if active_parameter.is_none() {
            active_parameter = Some(arguments.len() as u32);
        }

        let location = Location::new(name_span, self.file);
        if let Some(ReferenceId::Function(func_id)) = self.interner.find_referenced(location) {
            let name = self.interner.function_name(&func_id);
            let func_meta = self.interner.function_meta(&func_id);

            let signature_information =
                self.func_meta_signature_information(func_meta, name, active_parameter, has_self);
            let signature_help = SignatureHelp {
                active_parameter: signature_information.active_parameter.clone(),
                signatures: vec![signature_information],
                active_signature: Some(0),
            };
            self.signature_help = Some(signature_help);
            return;
        }

        if let Some(mut typ) = self.interner.type_at_location(location) {
            if let Type::Forall(_, forall_typ) = typ {
                typ = *forall_typ;
            }
            if let Type::Function(args, ret, _, unconstrained) = typ {
                let signature_information = self.function_type_signature_information(
                    &args,
                    &ret,
                    unconstrained,
                    active_parameter,
                );
                let signature_help = SignatureHelp {
                    active_parameter: signature_information.active_parameter.clone(),
                    signatures: vec![signature_information],
                    active_signature: Some(0),
                };
                self.signature_help = Some(signature_help);
                return;
            }
        }
    }

    fn func_meta_signature_information(
        &self,
        func_meta: &FuncMeta,
        name: &str,
        active_parameter: Option<u32>,
        has_self: bool,
    ) -> SignatureInformation {
        let mut label = String::new();
        let mut parameters = Vec::new();

        label.push_str(name);
        label.push('(');
        for (index, (pattern, typ, _)) in func_meta.parameters.0.iter().enumerate() {
            if index > 0 {
                label.push_str(", ");
            }

            if has_self && index == 0 {
                if let Type::MutableReference(..) = typ {
                    label.push_str("&mut self");
                } else {
                    label.push_str("self");
                }
            } else {
                let parameter_start = label.chars().count();

                self.hir_pattern_to_argument(pattern, &mut label);
                label.push_str(": ");
                label.push_str(&typ.to_string());

                let parameter_end = label.chars().count();

                parameters.push(ParameterInformation {
                    label: ParameterLabel::LabelOffsets([
                        parameter_start as u32,
                        parameter_end as u32,
                    ]),
                    documentation: None,
                })
            }
        }
        label.push(')');

        match &func_meta.return_type {
            FunctionReturnType::Default(_) => (),
            FunctionReturnType::Ty(typ) => {
                label.push_str(" -> ");
                label.push_str(&typ.to_string());
            }
        }

        SignatureInformation {
            label,
            documentation: None,
            parameters: Some(parameters),
            active_parameter,
        }
    }

    fn function_type_signature_information(
        &self,
        args: &[Type],
        return_type: &Type,
        unconstrained: bool,
        active_parameter: Option<u32>,
    ) -> SignatureInformation {
        let mut label = String::new();
        let mut parameters = Vec::new();

        if unconstrained {
            label.push_str("unconstrained ");
        }
        label.push_str("fn(");
        for (index, typ) in args.iter().enumerate() {
            if index > 0 {
                label.push_str(", ");
            }

            let parameter_start = label.chars().count();
            label.push_str(&typ.to_string());
            let parameter_end = label.chars().count();

            parameters.push(ParameterInformation {
                label: ParameterLabel::LabelOffsets([parameter_start as u32, parameter_end as u32]),
                documentation: None,
            })
        }
        label.push(')');

        if let Type::Unit = return_type {
            // Nothing
        } else {
            label.push_str(" -> ");
            label.push_str(&return_type.to_string());
        }

        SignatureInformation {
            label,
            documentation: None,
            parameters: Some(parameters),
            active_parameter,
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

    fn includes_span(&self, span: Span) -> bool {
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}

#[cfg(test)]
mod signature_help_tests {
    use crate::{
        notifications::on_did_open_text_document, requests::on_signature_help_request, test_utils,
    };

    use lsp_types::{
        DidOpenTextDocumentParams, ParameterLabel, Position, SignatureHelp, SignatureHelpParams,
        TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams,
        WorkDoneProgressParams,
    };
    use tokio::test;

    async fn get_signature_help(src: &str) -> SignatureHelp {
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

        on_signature_help_request(
            &mut state,
            SignatureHelpParams {
                context: None,
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: noir_text_document },
                    position: Position { line: line as u32, character: column as u32 },
                },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            },
        )
        .await
        .expect("Could not execute on_signature_help_request")
        .unwrap()
    }

    fn check_label(signature_label: &str, parameter_label: &ParameterLabel, expected_string: &str) {
        let ParameterLabel::LabelOffsets(offsets) = parameter_label else {
            panic!("Expected label to be LabelOffsets, got {:?}", parameter_label);
        };

        assert_eq!(&signature_label[offsets[0] as usize..offsets[1] as usize], expected_string);
    }

    #[test]
    async fn test_signature_help_for_call_at_first_argument() {
        let src = r#"
            fn foo(x: i32, y: Field) -> u32 { 0 }
            fn wrapper(x: u32) {}

            fn bar() {
                wrapper(foo(>|<1, 2));
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.label, "foo(x: i32, y: Field) -> u32");

        let params = signature.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 2);

        check_label(&signature.label, &params[0].label, "x: i32");
        check_label(&signature.label, &params[1].label, "y: Field");

        assert_eq!(signature.active_parameter, Some(0));
    }

    #[test]
    async fn test_signature_help_for_call_between_arguments() {
        let src = r#"
            fn foo(x: i32, y: Field) -> u32 { 0 }

            fn bar() {
                foo(1,>|< 2);
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.active_parameter, Some(1));
    }

    #[test]
    async fn test_signature_help_for_call_at_second_argument() {
        let src = r#"
            fn foo(x: i32, y: Field) -> u32 { 0 }

            fn bar() {
                foo(1, >|<2);
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.active_parameter, Some(1));
    }

    #[test]
    async fn test_signature_help_for_call_past_last_argument() {
        let src = r#"
            fn foo(x: i32, y: Field) -> u32 { 0 }

            fn bar() {
                foo(1, 2, >|<);
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.active_parameter, Some(2));
    }

    #[test]
    async fn test_signature_help_for_method_call() {
        let src = r#"
            struct Foo {}

            impl Foo {
              fn foo(self, x: i32, y: Field) -> u32 { 0 }
            }

            fn wrapper(x: u32) {}

            fn bar(f: Foo) {
                wrapper(f.foo(>|<1, 2));
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.label, "foo(self, x: i32, y: Field) -> u32");

        let params = signature.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 2);

        check_label(&signature.label, &params[0].label, "x: i32");
        check_label(&signature.label, &params[1].label, "y: Field");

        assert_eq!(signature.active_parameter, Some(0));
    }

    #[test]
    async fn test_signature_help_for_fn_call() {
        let src = r#"
            fn foo(x: i32, y: Field) -> u32 { 0 }

            fn bar() {
                let f = foo;
                f(>|<1, 2);
            }
        "#;

        let signature_help = get_signature_help(src).await;
        assert_eq!(signature_help.signatures.len(), 1);

        let signature = &signature_help.signatures[0];
        assert_eq!(signature.label, "fn(i32, Field) -> u32");

        let params = signature.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 2);

        check_label(&signature.label, &params[0].label, "i32");
        check_label(&signature.label, &params[1].label, "Field");

        assert_eq!(signature.active_parameter, Some(0));
    }
}
