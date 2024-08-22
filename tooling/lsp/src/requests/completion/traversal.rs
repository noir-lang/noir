/// This file includes the completion logic that's just about
/// traversing the AST without any additional logic.
use noirc_frontend::{
    ast::{
        ArrayLiteral, AssignStatement, CastExpression, ConstrainStatement, Expression, ForRange,
        FunctionReturnType, GenericTypeArgs, IndexExpression, InfixExpression, Literal, NoirTrait,
        NoirTypeAlias, TraitImplItem, UnresolvedType,
    },
    ParsedModule,
};

use super::NodeFinder;

impl<'a> NodeFinder<'a> {
    pub(super) fn find_in_parsed_module(&mut self, parsed_module: &ParsedModule) {
        for item in &parsed_module.items {
            self.find_in_item(item);
        }
    }

    pub(super) fn find_in_trait_impl_item(&mut self, item: &TraitImplItem) {
        match item {
            TraitImplItem::Function(noir_function) => self.find_in_noir_function(noir_function),
            TraitImplItem::Constant(_, _, _) => (),
            TraitImplItem::Type { .. } => (),
        }
    }

    pub(super) fn find_in_noir_trait(&mut self, noir_trait: &NoirTrait) {
        for item in &noir_trait.items {
            self.find_in_trait_item(item);
        }
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

    pub(super) fn find_in_cast_expression(&mut self, cast_expression: &CastExpression) {
        self.find_in_expression(&cast_expression.lhs);
    }

    pub(super) fn find_in_infix_expression(&mut self, infix_expression: &InfixExpression) {
        self.find_in_expression(&infix_expression.lhs);
        self.find_in_expression(&infix_expression.rhs);
    }

    pub(super) fn find_in_unresolved_types(&mut self, unresolved_type: &[UnresolvedType]) {
        for unresolved_type in unresolved_type {
            self.find_in_unresolved_type(unresolved_type);
        }
    }

    pub(super) fn find_in_type_args(&mut self, generics: &GenericTypeArgs) {
        self.find_in_unresolved_types(&generics.ordered_args);
        for (_name, typ) in &generics.named_args {
            self.find_in_unresolved_type(typ);
        }
    }

    pub(super) fn find_in_function_return_type(&mut self, return_type: &FunctionReturnType) {
        match return_type {
            noirc_frontend::ast::FunctionReturnType::Default(_) => (),
            noirc_frontend::ast::FunctionReturnType::Ty(unresolved_type) => {
                self.find_in_unresolved_type(unresolved_type);
            }
        }
    }

    pub(super) fn find_in_noir_type_alias(&mut self, noir_type_alias: &NoirTypeAlias) {
        self.find_in_unresolved_type(&noir_type_alias.typ);
    }
}
