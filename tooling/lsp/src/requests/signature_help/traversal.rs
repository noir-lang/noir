/// This file includes the signature help logic that's just about
/// traversing the AST without any additional logic.
use super::SignatureFinder;

use noirc_frontend::{
    ast::{
        ArrayLiteral, AssignStatement, BlockExpression, CastExpression, ConstrainStatement,
        ConstructorExpression, Expression, ExpressionKind, ForLoopStatement, ForRange,
        IfExpression, IndexExpression, InfixExpression, LValue, Lambda, LetStatement, Literal,
        MemberAccessExpression, NoirFunction, NoirTrait, NoirTraitImpl, Statement, StatementKind,
        TraitImplItem, TraitItem, TypeImpl,
    },
    parser::{Item, ItemKind},
    ParsedModule,
};

impl<'a> SignatureFinder<'a> {
    pub(super) fn find_in_parsed_module(&mut self, parsed_module: &ParsedModule) {
        for item in &parsed_module.items {
            self.find_in_item(item);
        }
    }

    pub(super) fn find_in_item(&mut self, item: &Item) {
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

    pub(super) fn find_in_noir_function(&mut self, noir_function: &NoirFunction) {
        self.find_in_block_expression(&noir_function.def.body);
    }

    pub(super) fn find_in_noir_trait_impl(&mut self, noir_trait_impl: &NoirTraitImpl) {
        for item in &noir_trait_impl.items {
            self.find_in_trait_impl_item(item);
        }
    }

    pub(super) fn find_in_trait_impl_item(&mut self, item: &TraitImplItem) {
        match item {
            TraitImplItem::Function(noir_function) => self.find_in_noir_function(noir_function),
            TraitImplItem::Constant(_, _, _) => (),
            TraitImplItem::Type { .. } => (),
        }
    }

    pub(super) fn find_in_type_impl(&mut self, type_impl: &TypeImpl) {
        for (method, span) in &type_impl.methods {
            if self.includes_span(*span) {
                self.find_in_noir_function(method);
            }
        }
    }

    pub(super) fn find_in_noir_trait(&mut self, noir_trait: &NoirTrait) {
        for item in &noir_trait.items {
            self.find_in_trait_item(item);
        }
    }

    pub(super) fn find_in_trait_item(&mut self, trait_item: &TraitItem) {
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

    pub(super) fn find_in_block_expression(&mut self, block_expression: &BlockExpression) {
        for statement in &block_expression.statements {
            if self.includes_span(statement.span) {
                self.find_in_statement(statement);
            }
        }
    }

    pub(super) fn find_in_statement(&mut self, statement: &Statement) {
        if !self.includes_span(statement.span) {
            return;
        }

        match &statement.kind {
            StatementKind::Let(let_statement) => {
                self.find_in_let_statement(let_statement);
            }
            StatementKind::Constrain(constrain_statement) => {
                self.find_in_constrain_statement(constrain_statement);
            }
            StatementKind::Expression(expression) => {
                self.find_in_expression(expression);
            }
            StatementKind::Assign(assign_statement) => {
                self.find_in_assign_statement(assign_statement);
            }
            StatementKind::For(for_loop_statement) => {
                self.find_in_for_loop_statement(for_loop_statement);
            }
            StatementKind::Comptime(statement) => {
                self.find_in_statement(statement);
            }
            StatementKind::Semi(expression) => {
                self.find_in_expression(expression);
            }
            StatementKind::Break | StatementKind::Continue | StatementKind::Error => (),
        }
    }

    pub(super) fn find_in_let_statement(&mut self, let_statement: &LetStatement) {
        self.find_in_expression(&let_statement.expression);
    }

    pub(super) fn find_in_constrain_statement(&mut self, constrain_statement: &ConstrainStatement) {
        self.find_in_expression(&constrain_statement.0);

        if let Some(exp) = &constrain_statement.1 {
            self.find_in_expression(exp);
        }
    }

    pub(super) fn find_in_assign_statement(&mut self, assign_statement: &AssignStatement) {
        self.find_in_lvalue(&assign_statement.lvalue);
        self.find_in_expression(&assign_statement.expression);
    }

    pub(super) fn find_in_for_loop_statement(&mut self, for_loop_statement: &ForLoopStatement) {
        self.find_in_for_range(&for_loop_statement.range);
        self.find_in_expression(&for_loop_statement.block);
    }

    pub(super) fn find_in_lvalue(&mut self, lvalue: &LValue) {
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

    pub(super) fn find_in_for_range(&mut self, for_range: &ForRange) {
        match for_range {
            ForRange::Range(start, end) => {
                self.find_in_expression(start);
                self.find_in_expression(end);
            }
            ForRange::Array(expression) => self.find_in_expression(expression),
        }
    }

    pub(super) fn find_in_expressions(&mut self, expressions: &[Expression]) {
        for expression in expressions {
            self.find_in_expression(expression);
        }
    }

    pub(super) fn find_in_expression(&mut self, expression: &Expression) {
        match &expression.kind {
            ExpressionKind::Literal(literal) => self.find_in_literal(literal),
            ExpressionKind::Block(block_expression) => {
                self.find_in_block_expression(block_expression);
            }
            ExpressionKind::Prefix(prefix_expression) => {
                self.find_in_expression(&prefix_expression.rhs);
            }
            ExpressionKind::Index(index_expression) => {
                self.find_in_index_expression(index_expression);
            }
            ExpressionKind::Call(call_expression) => {
                self.find_in_call_expression(call_expression, expression.span);
            }
            ExpressionKind::MethodCall(method_call_expression) => {
                self.find_in_method_call_expression(method_call_expression, expression.span);
            }
            ExpressionKind::Constructor(constructor_expression) => {
                self.find_in_constructor_expression(constructor_expression);
            }
            ExpressionKind::MemberAccess(member_access_expression) => {
                self.find_in_member_access_expression(member_access_expression);
            }
            ExpressionKind::Cast(cast_expression) => {
                self.find_in_cast_expression(cast_expression);
            }
            ExpressionKind::Infix(infix_expression) => {
                self.find_in_infix_expression(infix_expression);
            }
            ExpressionKind::If(if_expression) => {
                self.find_in_if_expression(if_expression);
            }
            ExpressionKind::Tuple(expressions) => {
                self.find_in_expressions(expressions);
            }
            ExpressionKind::Lambda(lambda) => self.find_in_lambda(lambda),
            ExpressionKind::Parenthesized(expression) => {
                self.find_in_expression(expression);
            }
            ExpressionKind::Unquote(expression) => {
                self.find_in_expression(expression);
            }
            ExpressionKind::Comptime(block_expression, _) => {
                self.find_in_block_expression(block_expression);
            }
            ExpressionKind::Unsafe(block_expression, _) => {
                self.find_in_block_expression(block_expression);
            }
            ExpressionKind::Variable(_)
            | ExpressionKind::AsTraitPath(_)
            | ExpressionKind::Quote(_)
            | ExpressionKind::Resolved(_)
            | ExpressionKind::Error => (),
        }
    }

    pub(super) fn find_in_literal(&mut self, literal: &Literal) {
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

    pub(super) fn find_in_array_literal(&mut self, array_literal: &ArrayLiteral) {
        match array_literal {
            ArrayLiteral::Standard(expressions) => self.find_in_expressions(expressions),
            ArrayLiteral::Repeated { repeated_element, length } => {
                self.find_in_expression(repeated_element);
                self.find_in_expression(length);
            }
        }
    }

    pub(super) fn find_in_index_expression(&mut self, index_expression: &IndexExpression) {
        self.find_in_expression(&index_expression.collection);
        self.find_in_expression(&index_expression.index);
    }

    pub(super) fn find_in_constructor_expression(
        &mut self,
        constructor_expression: &ConstructorExpression,
    ) {
        for (_field_name, expression) in &constructor_expression.fields {
            self.find_in_expression(expression);
        }
    }

    pub(super) fn find_in_member_access_expression(
        &mut self,
        member_access_expression: &MemberAccessExpression,
    ) {
        self.find_in_expression(&member_access_expression.lhs);
    }

    pub(super) fn find_in_cast_expression(&mut self, cast_expression: &CastExpression) {
        self.find_in_expression(&cast_expression.lhs);
    }

    pub(super) fn find_in_infix_expression(&mut self, infix_expression: &InfixExpression) {
        self.find_in_expression(&infix_expression.lhs);
        self.find_in_expression(&infix_expression.rhs);
    }

    pub(super) fn find_in_if_expression(&mut self, if_expression: &IfExpression) {
        self.find_in_expression(&if_expression.condition);
        self.find_in_expression(&if_expression.consequence);

        if let Some(alternative) = &if_expression.alternative {
            self.find_in_expression(alternative);
        }
    }

    pub(super) fn find_in_lambda(&mut self, lambda: &Lambda) {
        self.find_in_expression(&lambda.body);
    }
}
