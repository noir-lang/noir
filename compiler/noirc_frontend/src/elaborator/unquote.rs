use crate::{
    ast::{
        ArrayLiteral, AssignStatement, ConstrainStatement, ConstructorExpression, IfExpression,
        InfixExpression, Lambda,
    },
    macros_api::{
        BlockExpression, CallExpression, CastExpression, Expression, ExpressionKind,
        ForLoopStatement, ForRange, IndexExpression, LetStatement, Literal, MemberAccessExpression,
        MethodCallExpression, PrefixExpression, Statement, StatementKind,
    },
    node_interner::ExprId,
};

use super::Elaborator;

impl<'a> Elaborator<'a> {
    pub fn find_unquoted_exprs_in_block(
        &mut self,
        block: &mut BlockExpression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        for statement in &mut block.statements {
            self.find_unquoted_exprs_in_statement(statement, unquoted_exprs);
        }
    }

    fn find_unquoted_exprs_in_statement(
        &mut self,
        statement: &mut Statement,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        match &mut statement.kind {
            StatementKind::Let(let_) => self.find_unquoted_exprs_in_let(let_, unquoted_exprs),
            StatementKind::Constrain(constrain) => {
                self.find_unquoted_exprs_in_constrain(constrain, unquoted_exprs);
            }
            StatementKind::Expression(expr) => {
                self.find_unquoted_exprs_in_expr(expr, unquoted_exprs);
            }
            StatementKind::Assign(assign) => {
                self.find_unquoted_exprs_in_assign(assign, unquoted_exprs);
            }
            StatementKind::For(for_) => self.find_unquoted_exprs_in_for(for_, unquoted_exprs),
            StatementKind::Break => (),
            StatementKind::Continue => (),
            StatementKind::Comptime(comptime) => {
                self.find_unquoted_exprs_in_statement(comptime, unquoted_exprs);
            }
            StatementKind::Semi(expr) => self.find_unquoted_exprs_in_expr(expr, unquoted_exprs),
            StatementKind::Error => (),
        }
    }

    fn find_unquoted_exprs_in_constrain(
        &mut self,
        constrain: &mut ConstrainStatement,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut constrain.0, unquoted_exprs);
        if let Some(msg) = constrain.1.as_mut() {
            self.find_unquoted_exprs_in_expr(msg, unquoted_exprs);
        }
    }

    fn find_unquoted_exprs_in_let(
        &mut self,
        let_: &mut LetStatement,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut let_.expression, unquoted_exprs);
    }

    fn find_unquoted_exprs_in_assign(
        &mut self,
        assign: &mut AssignStatement,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut assign.expression, unquoted_exprs);
    }

    fn find_unquoted_exprs_in_for(
        &mut self,
        for_: &mut ForLoopStatement,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        match &mut for_.range {
            ForRange::Range(start, end) => {
                self.find_unquoted_exprs_in_expr(start, unquoted_exprs);
                self.find_unquoted_exprs_in_expr(end, unquoted_exprs);
            }
            ForRange::Array(array) => {
                self.find_unquoted_exprs_in_expr(array, unquoted_exprs);
            }
        };
        self.find_unquoted_exprs_in_expr(&mut for_.block, unquoted_exprs);
    }

    fn find_unquoted_exprs_in_expr(
        &mut self,
        expr: &mut Expression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        match &mut expr.kind {
            ExpressionKind::Literal(literal) => {
                self.find_unquoted_exprs_in_literal(literal, unquoted_exprs);
            }
            ExpressionKind::Block(block) => {
                self.find_unquoted_exprs_in_block(block, unquoted_exprs);
            }
            ExpressionKind::Prefix(prefix) => {
                self.find_unquoted_exprs_in_prefix(prefix, unquoted_exprs);
            }
            ExpressionKind::Index(index) => {
                self.find_unquoted_exprs_in_index(index, unquoted_exprs);
            }
            ExpressionKind::Call(call) => self.find_unquoted_exprs_in_call(call, unquoted_exprs),
            ExpressionKind::MethodCall(call) => {
                self.find_unquoted_exprs_in_method_call(call, unquoted_exprs);
            }
            ExpressionKind::Constructor(constructor) => {
                self.find_unquoted_exprs_in_constructor(constructor, unquoted_exprs);
            }
            ExpressionKind::MemberAccess(access) => {
                self.find_unquoted_exprs_in_access(access, unquoted_exprs);
            }
            ExpressionKind::Cast(cast) => self.find_unquoted_exprs_in_cast(cast, unquoted_exprs),
            ExpressionKind::Infix(infix) => {
                self.find_unquoted_exprs_in_infix(infix, unquoted_exprs);
            }
            ExpressionKind::If(if_) => self.find_unquoted_exprs_in_if(if_, unquoted_exprs),
            ExpressionKind::Variable(_, _) => (),
            ExpressionKind::Tuple(tuple) => {
                self.find_unquoted_exprs_in_tuple(tuple, unquoted_exprs);
            }
            ExpressionKind::Lambda(lambda) => {
                self.find_unquoted_exprs_in_lambda(lambda, unquoted_exprs);
            }
            ExpressionKind::Parenthesized(expr) => {
                self.find_unquoted_exprs_in_expr(expr, unquoted_exprs);
            }
            ExpressionKind::Quote(quote, _) => {
                self.find_unquoted_exprs_in_block(quote, unquoted_exprs);
            }
            ExpressionKind::Comptime(block, _) => {
                self.find_unquoted_exprs_in_block(block, unquoted_exprs);
            }
            ExpressionKind::Resolved(_) => (),
            ExpressionKind::Error => (),
            ExpressionKind::UnquoteMarker(_) => (),
            ExpressionKind::Unquote(unquoted) => {
                // Avoid an expensive clone for unquoted
                let empty_expr = Expression::new(ExpressionKind::Error, unquoted.span);
                let unquote = std::mem::replace(unquoted.as_mut(), empty_expr);
                self.replace_unquote(expr, unquote, unquoted_exprs);
            }
        }
    }

    fn find_unquoted_exprs_in_literal(
        &mut self,
        literal: &mut Literal,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        match literal {
            Literal::Array(array) | Literal::Slice(array) => match array {
                ArrayLiteral::Standard(elements) => {
                    for element in elements {
                        self.find_unquoted_exprs_in_expr(element, unquoted_exprs);
                    }
                }
                ArrayLiteral::Repeated { repeated_element, length } => {
                    self.find_unquoted_exprs_in_expr(repeated_element, unquoted_exprs);
                    self.find_unquoted_exprs_in_expr(length, unquoted_exprs);
                }
            },
            Literal::Bool(_)
            | Literal::Integer(_, _)
            | Literal::Str(_)
            | Literal::RawStr(_, _)
            | Literal::FmtStr(_)
            | Literal::Unit => (),
        }
    }

    fn find_unquoted_exprs_in_prefix(
        &mut self,
        prefix: &mut PrefixExpression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut prefix.rhs, unquoted_exprs);
    }

    fn find_unquoted_exprs_in_index(
        &mut self,
        index: &mut IndexExpression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut index.collection, unquoted_exprs);
        self.find_unquoted_exprs_in_expr(&mut index.index, unquoted_exprs);
    }

    fn find_unquoted_exprs_in_call(
        &mut self,
        call: &mut CallExpression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut call.func, unquoted_exprs);

        for arg in &mut call.arguments {
            self.find_unquoted_exprs_in_expr(arg, unquoted_exprs);
        }
    }

    fn find_unquoted_exprs_in_method_call(
        &mut self,
        call: &mut MethodCallExpression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut call.object, unquoted_exprs);

        for arg in &mut call.arguments {
            self.find_unquoted_exprs_in_expr(arg, unquoted_exprs);
        }
    }

    fn find_unquoted_exprs_in_constructor(
        &mut self,
        constructor: &mut ConstructorExpression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        for (_, field) in &mut constructor.fields {
            self.find_unquoted_exprs_in_expr(field, unquoted_exprs);
        }
    }

    fn find_unquoted_exprs_in_access(
        &mut self,
        member_access: &mut MemberAccessExpression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut member_access.lhs, unquoted_exprs);
    }

    fn find_unquoted_exprs_in_cast(
        &mut self,
        cast: &mut CastExpression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut cast.lhs, unquoted_exprs);
    }

    fn find_unquoted_exprs_in_infix(
        &mut self,
        infix: &mut InfixExpression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut infix.lhs, unquoted_exprs);
        self.find_unquoted_exprs_in_expr(&mut infix.rhs, unquoted_exprs);
    }

    fn find_unquoted_exprs_in_if(
        &mut self,
        if_: &mut IfExpression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut if_.condition, unquoted_exprs);
        self.find_unquoted_exprs_in_expr(&mut if_.consequence, unquoted_exprs);

        if let Some(alternate) = if_.alternative.as_mut() {
            self.find_unquoted_exprs_in_expr(alternate, unquoted_exprs);
        }
    }

    fn find_unquoted_exprs_in_tuple(
        &mut self,
        tuple: &mut [Expression],
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        for field in tuple {
            self.find_unquoted_exprs_in_expr(field, unquoted_exprs);
        }
    }

    fn find_unquoted_exprs_in_lambda(
        &mut self,
        lambda: &mut Lambda,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        self.find_unquoted_exprs_in_expr(&mut lambda.body, unquoted_exprs);
    }

    /// Elaborate and store the unquoted expression in the given vector, then
    /// replace it with an unquote expression with an UnquoteMarker expression to mark the position
    /// to replace it with later.
    fn replace_unquote(
        &mut self,
        expr: &mut Expression,
        unquoted: Expression,
        unquoted_exprs: &mut Vec<ExprId>,
    ) {
        let (expr_id, _) = self.elaborate_expression(unquoted);
        let unquote_marker_id = unquoted_exprs.len();
        unquoted_exprs.push(expr_id);
        expr.kind = ExpressionKind::UnquoteMarker(unquote_marker_id);
    }
}
