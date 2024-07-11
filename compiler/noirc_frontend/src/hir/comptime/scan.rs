//! This module is for the scanning of the Hir by the interpreter.
//! In this initial step, the Hir is scanned for `Comptime` nodes
//! without actually executing anything until such a node is found.
//! Once such a node is found, the interpreter will call the relevant
//! evaluate method on that node type, insert the result into the Ast,
//! and continue scanning the rest of the program.
//!
//! Since it mostly just needs to recur on the Hir looking for Comptime
//! nodes, this pass is fairly simple. The only thing it really needs to
//! ensure to do is to push and pop scopes on the interpreter as needed
//! so that any variables defined within e.g. an `if` statement containing
//! a `Comptime` block aren't accessible outside of the `if`.
use crate::{
    hir_def::{
        expr::{
            HirArrayLiteral, HirBlockExpression, HirCallExpression, HirConstructorExpression,
            HirIdent, HirIfExpression, HirIndexExpression, HirInfixExpression, HirLambda,
            HirMethodCallExpression,
        },
        stmt::HirForStatement,
    },
    macros_api::{HirExpression, HirLiteral, HirStatement},
    node_interner::{DefinitionKind, ExprId, FuncId, GlobalId, StmtId},
};

use super::{
    errors::{IResult, InterpreterError},
    interpreter::Interpreter,
    Value,
};

use noirc_errors::Location;

#[allow(dead_code)]
impl<'interner> Interpreter<'interner> {
    /// Scan through a function, evaluating any Comptime nodes found.
    /// These nodes will be modified in place, replaced with the
    /// result of their evaluation.
    pub fn scan_function(&mut self, function: FuncId) -> IResult<()> {
        // Don't scan through functions that are already comptime. They may use comptime-only
        // features (most likely HirExpression::Quote) that we'd otherwise error for.
        if self.interner.function_modifiers(&function).is_comptime {
            return Ok(());
        }

        let function = self.interner.function(&function);

        let state = self.enter_function();
        self.scan_expression(function.as_expr())?;
        self.exit_function(state);
        Ok(())
    }

    /// Evaluate this global if it is a comptime global.
    /// Otherwise, scan through its expression for any comptime blocks to evaluate.
    pub fn scan_global(&mut self, global: GlobalId) -> IResult<()> {
        if let Some(let_) = self.interner.get_global_let_statement(global) {
            if let_.comptime {
                self.evaluate_let(let_)?;
            } else {
                self.scan_expression(let_.expression)?;
            }
        }
        Ok(())
    }

    fn scan_expression(&mut self, expr: ExprId) -> IResult<()> {
        match self.interner.expression(&expr) {
            HirExpression::Ident(ident, _) => self.scan_ident(ident, expr),
            HirExpression::Literal(literal) => self.scan_literal(literal),
            HirExpression::Block(block) => self.scan_block(block),
            HirExpression::Prefix(prefix) => self.scan_expression(prefix.rhs),
            HirExpression::Infix(infix) => self.scan_infix(infix),
            HirExpression::Index(index) => self.scan_index(index),
            HirExpression::Constructor(constructor) => self.scan_constructor(constructor),
            HirExpression::MemberAccess(member_access) => self.scan_expression(member_access.lhs),
            HirExpression::Call(call) => self.scan_call(call),
            HirExpression::MethodCall(method_call) => self.scan_method_call(method_call),
            HirExpression::Cast(cast) => self.scan_expression(cast.lhs),
            HirExpression::If(if_) => self.scan_if(if_),
            HirExpression::Tuple(tuple) => self.scan_tuple(tuple),
            HirExpression::Lambda(lambda) => self.scan_lambda(lambda),
            HirExpression::Comptime(block) => {
                let location = self.interner.expr_location(&expr);
                let new_expr_id =
                    self.evaluate_block(block)?.into_hir_expression(self.interner, location)?;
                let new_expr = self.interner.expression(&new_expr_id);
                self.debug_comptime(new_expr_id, location);
                self.interner.replace_expr(&expr, new_expr);
                Ok(())
            }
            HirExpression::Quote(_) => {
                // This error could be detected much earlier in the compiler pipeline but
                // it just makes sense for the comptime code to handle comptime things.
                let location = self.interner.expr_location(&expr);
                Err(InterpreterError::QuoteInRuntimeCode { location })
            }
            HirExpression::Error => Ok(()),

            // Unquote should only be inserted by the comptime interpreter while expanding macros
            // and is removed by the Hir -> Ast conversion pass which converts it into a normal block.
            // If we find one now during scanning it most likely means the Hir -> Ast conversion
            // missed it somehow. In the future we may allow users to manually write unquote
            // expressions in their code but for now this is unreachable.
            HirExpression::Unquote(block) => {
                unreachable!("Found unquote block while scanning: {block:?}")
            }
        }
    }

    // Identifiers have no code to execute but we may need to inline any values
    // of comptime variables into runtime code.
    fn scan_ident(&mut self, ident: HirIdent, id: ExprId) -> IResult<()> {
        let definition = self.interner.definition(ident.id);

        match &definition.kind {
            DefinitionKind::Function(_) => Ok(()),
            _ => {
                // Opportunistically evaluate this identifier to see if it is compile-time known.
                // If so, inline its value.
                if let Ok(value) = self.evaluate_ident(ident, id) {
                    // TODO(#4922): Inlining closures is currently unimplemented
                    if !matches!(value, Value::Closure(..)) {
                        let new_expr = self.inline_expression(value, id)?;
                        let location = self.interner.id_location(id);
                        self.debug_comptime(new_expr, location);
                    }
                }
                Ok(())
            }
        }
    }

    fn scan_literal(&mut self, literal: HirLiteral) -> IResult<()> {
        match literal {
            HirLiteral::Array(elements) | HirLiteral::Slice(elements) => match elements {
                HirArrayLiteral::Standard(elements) => {
                    for element in elements {
                        self.scan_expression(element)?;
                    }
                    Ok(())
                }
                HirArrayLiteral::Repeated { repeated_element, length: _ } => {
                    self.scan_expression(repeated_element)
                }
            },
            HirLiteral::Bool(_)
            | HirLiteral::Integer(_, _)
            | HirLiteral::Str(_)
            | HirLiteral::FmtStr(_, _)
            | HirLiteral::Unit => Ok(()),
        }
    }

    fn scan_block(&mut self, block: HirBlockExpression) -> IResult<()> {
        self.push_scope();
        for statement in &block.statements {
            self.scan_statement(*statement)?;
        }
        self.pop_scope();
        Ok(())
    }

    fn scan_infix(&mut self, infix: HirInfixExpression) -> IResult<()> {
        self.scan_expression(infix.lhs)?;
        self.scan_expression(infix.rhs)
    }

    fn scan_index(&mut self, index: HirIndexExpression) -> IResult<()> {
        self.scan_expression(index.collection)?;
        self.scan_expression(index.index)
    }

    fn scan_constructor(&mut self, constructor: HirConstructorExpression) -> IResult<()> {
        for (_, field) in constructor.fields {
            self.scan_expression(field)?;
        }
        Ok(())
    }

    fn scan_call(&mut self, call: HirCallExpression) -> IResult<()> {
        self.scan_expression(call.func)?;
        for arg in call.arguments {
            self.scan_expression(arg)?;
        }
        Ok(())
    }

    fn scan_method_call(&mut self, method_call: HirMethodCallExpression) -> IResult<()> {
        self.scan_expression(method_call.object)?;
        for arg in method_call.arguments {
            self.scan_expression(arg)?;
        }
        Ok(())
    }

    fn scan_if(&mut self, if_: HirIfExpression) -> IResult<()> {
        self.scan_expression(if_.condition)?;

        self.push_scope();
        self.scan_expression(if_.consequence)?;
        self.pop_scope();

        if let Some(alternative) = if_.alternative {
            self.push_scope();
            self.scan_expression(alternative)?;
            self.pop_scope();
        }
        Ok(())
    }

    fn scan_tuple(&mut self, tuple: Vec<ExprId>) -> IResult<()> {
        for field in tuple {
            self.scan_expression(field)?;
        }
        Ok(())
    }

    fn scan_lambda(&mut self, lambda: HirLambda) -> IResult<()> {
        self.scan_expression(lambda.body)
    }

    fn scan_statement(&mut self, statement: StmtId) -> IResult<()> {
        match self.interner.statement(&statement) {
            HirStatement::Let(let_) => self.scan_expression(let_.expression),
            HirStatement::Constrain(constrain) => self.scan_expression(constrain.0),
            HirStatement::Assign(assign) => self.scan_expression(assign.expression),
            HirStatement::For(for_) => self.scan_for(for_),
            HirStatement::Break => Ok(()),
            HirStatement::Continue => Ok(()),
            HirStatement::Expression(expression) => self.scan_expression(expression),
            HirStatement::Semi(semi) => self.scan_expression(semi),
            HirStatement::Error => Ok(()),
            HirStatement::Comptime(comptime) => {
                let location = self.interner.statement_location(comptime);
                let new_expr = self
                    .evaluate_comptime(comptime)?
                    .into_hir_expression(self.interner, location)?;
                self.debug_comptime(new_expr, location);
                self.interner.replace_statement(statement, HirStatement::Expression(new_expr));
                Ok(())
            }
        }
    }

    fn scan_for(&mut self, for_: HirForStatement) -> IResult<()> {
        // We don't need to set self.in_loop since we're not actually evaluating this loop.
        // We just need to push a scope so that if there's a `comptime { .. }` expr inside this
        // loop, any variables it defines aren't accessible outside of it.
        self.push_scope();
        self.scan_expression(for_.block)?;
        self.pop_scope();
        Ok(())
    }

    fn inline_expression(&mut self, value: Value, expr: ExprId) -> IResult<ExprId> {
        let location = self.interner.expr_location(&expr);
        let new_expr_id = value.into_hir_expression(self.interner, location)?;
        let new_expr = self.interner.expression(&new_expr_id);
        self.interner.replace_expr(&expr, new_expr);
        Ok(new_expr_id)
    }

    fn debug_comptime(&mut self, expr: ExprId, location: Location) {
        if Some(location.file) == self.debug_comptime_in_file {
            let expr = expr.to_display_ast(self.interner);
            self.debug_comptime_evaluations
                .push(InterpreterError::debug_evaluate_comptime(expr, location));
        }
    }
}
