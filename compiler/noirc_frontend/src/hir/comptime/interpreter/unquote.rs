use fm::FileId;
use noirc_errors::Location;

use crate::{
    ast::{
        ArrayLiteral, AssignStatement, ConstrainStatement, ConstructorExpression, IfExpression,
        InfixExpression, Lambda,
    },
    hir::comptime::{errors::IResult, Value},
    macros_api::{
        BlockExpression, CallExpression, CastExpression, Expression, ExpressionKind,
        ForLoopStatement, ForRange, IndexExpression, LetStatement, Literal, MemberAccessExpression,
        MethodCallExpression, PrefixExpression, Statement, StatementKind,
    },
};

use super::Interpreter;

pub(super) struct UnquoteArgs {
    pub(super) values: Vec<Value>,
    pub(super) file: FileId,
}

impl<'a> Interpreter<'a> {
    pub(super) fn substitute_unquoted_values_into_block(
        &mut self,
        block: &mut BlockExpression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        for statement in &mut block.statements {
            self.substitute_unquoted_into_statement(statement, args)?;
        }
        Ok(())
    }

    fn substitute_unquoted_into_statement(
        &mut self,
        statement: &mut Statement,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        match &mut statement.kind {
            StatementKind::Let(let_) => self.substitute_unquoted_into_let(let_, args),
            StatementKind::Constrain(constrain) => {
                self.substitute_unquoted_into_constrain(constrain, args)
            }
            StatementKind::Expression(expr) => self.substitute_unquoted_into_expr(expr, args),
            StatementKind::Assign(assign) => self.substitute_unquoted_into_assign(assign, args),
            StatementKind::For(for_) => self.substitute_unquoted_into_for(for_, args),
            StatementKind::Break => Ok(()),
            StatementKind::Continue => Ok(()),
            StatementKind::Comptime(comptime) => {
                self.substitute_unquoted_into_statement(comptime, args)
            }
            StatementKind::Semi(expr) => self.substitute_unquoted_into_expr(expr, args),
            StatementKind::Error => Ok(()),
        }
    }

    fn substitute_unquoted_into_constrain(
        &mut self,
        constrain: &mut ConstrainStatement,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        self.substitute_unquoted_into_expr(&mut constrain.0, args)?;
        if let Some(msg) = constrain.1.as_mut() {
            self.substitute_unquoted_into_expr(msg, args)?;
        }
        Ok(())
    }

    fn substitute_unquoted_into_let(
        &mut self,
        let_: &mut LetStatement,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        self.substitute_unquoted_into_expr(&mut let_.expression, args)
    }

    fn substitute_unquoted_into_assign(
        &mut self,
        assign: &mut AssignStatement,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_for(
        &mut self,
        for_: &mut ForLoopStatement,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        match &mut for_.range {
            ForRange::Range(start, end) => {
                self.substitute_unquoted_into_expr(start, args)?;
                self.substitute_unquoted_into_expr(end, args)?;
            }
            ForRange::Array(array) => {
                self.substitute_unquoted_into_expr(array, args)?;
            }
        };
        self.substitute_unquoted_into_expr(&mut for_.block, args)
    }

    fn substitute_unquoted_into_expr(
        &mut self,
        expr: &mut Expression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        match &mut expr.kind {
            ExpressionKind::Literal(literal) => {
                self.substitute_unquoted_into_literal(literal, args)
            }
            ExpressionKind::Block(block) => self.substitute_unquoted_values_into_block(block, args),
            ExpressionKind::Prefix(prefix) => self.substitute_unquoted_into_prefix(prefix, args),
            ExpressionKind::Index(index) => self.substitute_unquoted_into_index(index, args),
            ExpressionKind::Call(call) => self.substitute_unquoted_into_call(call, args),
            ExpressionKind::MethodCall(call) => {
                self.substitute_unquoted_into_method_call(call, args)
            }
            ExpressionKind::Constructor(constructor) => {
                self.substitute_unquoted_into_constructor(constructor, args)
            }
            ExpressionKind::MemberAccess(access) => {
                self.substitute_unquoted_into_access(access, args)
            }
            ExpressionKind::Cast(cast) => self.substitute_unquoted_into_cast(cast, args),
            ExpressionKind::Infix(infix) => self.substitute_unquoted_into_infix(infix, args),
            ExpressionKind::If(if_) => self.substitute_unquoted_into_if(if_, args),
            ExpressionKind::Variable(_, _) => Ok(()),
            ExpressionKind::Tuple(tuple) => self.substitute_unquoted_into_tuple(tuple, args),
            ExpressionKind::Lambda(lambda) => self.substitute_unquoted_into_lambda(lambda, args),
            ExpressionKind::Parenthesized(parenthesized) => {
                self.substitute_unquoted_into_parenthesized(parenthesized, args)
            }
            ExpressionKind::Quote(quote) => self.substitute_unquoted_values_into_block(quote, args),
            ExpressionKind::Unquote(unquote) => {
                self.substitute_unquoted_into_unquote(unquote, args)
            }
            ExpressionKind::Comptime(comptime) => {
                self.substitute_unquoted_into_comptime(comptime, args)
            }
            ExpressionKind::Resolved(resolved) => Ok(()),
            ExpressionKind::Error => Ok(()),
            ExpressionKind::UnquoteMarker(index) => {
                let location = Location::new(expr.span, args.file);
                *expr = args.values[*index].clone().into_expression(self.interner, location)?;
                Ok(())
            }
        }
    }

    fn substitute_unquoted_into_literal(
        &mut self,
        literal: &mut Literal,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        match literal {
            Literal::Array(array) | Literal::Slice(array) => match array {
                ArrayLiteral::Standard(elements) => {
                    for element in elements {
                        self.substitute_unquoted_into_expr(element, args)?;
                    }
                    Ok(())
                }
                ArrayLiteral::Repeated { repeated_element, length } => {
                    self.substitute_unquoted_into_expr(repeated_element, args)?;
                    self.substitute_unquoted_into_expr(length, args)?;
                    Ok(())
                }
            },
            Literal::Bool(_)
            | Literal::Integer(_, _)
            | Literal::Str(_)
            | Literal::RawStr(_, _)
            | Literal::FmtStr(_)
            | Literal::Unit => Ok(()),
        }
    }

    fn substitute_unquoted_into_prefix(
        &mut self,
        prefix: &mut PrefixExpression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_index(
        &mut self,
        index: &mut IndexExpression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_call(
        &mut self,
        call: &mut CallExpression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_method_call(
        &mut self,
        call: &mut MethodCallExpression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_constructor(
        &mut self,
        constructor: &mut ConstructorExpression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_access(
        &mut self,
        member_access: &mut MemberAccessExpression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_cast(
        &mut self,
        cast: &mut CastExpression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_infix(
        &mut self,
        infix: &mut InfixExpression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        self.substitute_unquoted_into_expr(&mut infix.lhs, args)?;
        self.substitute_unquoted_into_expr(&mut infix.rhs, args)
    }

    fn substitute_unquoted_into_if(
        &mut self,
        r#if: &mut IfExpression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_tuple(
        &mut self,
        tuple: &mut [Expression],
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_lambda(
        &mut self,
        lambda: &mut Lambda,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_parenthesized(
        &mut self,
        parenthesized: &mut Expression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_unquote(
        &mut self,
        unquote: &mut Expression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }

    fn substitute_unquoted_into_comptime(
        &mut self,
        comptime: &mut BlockExpression,
        args: &UnquoteArgs,
    ) -> IResult<()> {
        todo!()
    }
}
