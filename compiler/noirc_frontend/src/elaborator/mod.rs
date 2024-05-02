use crate::{macros_api::{NoirFunction, Expression, BlockExpression, Statement, StatementKind, ExpressionKind, NodeInterner, HirExpression, HirStatement}, node_interner::{FuncId, StmtId, ExprId}, ast::FunctionKind, hir_def::expr::HirBlockExpression, hir::{resolution::errors::ResolverError, def_collector::dc_crate::CompilationError}, Type};

mod scope;

use fm::FileId;
use iter_extended::vecmap;
use scope::Scope;

struct Elaborator {
    globals: Scope,
    local_scopes: Vec<Scope>,

    errors: Vec<CompilationError>,

    interner: NodeInterner,
    file: FileId,

    in_unconstrained_fn: bool,
    nested_loops: usize,
}

impl Elaborator {
    fn elaborate_function(&mut self, function: NoirFunction, id: FuncId) {
        match function.kind {
            FunctionKind::LowLevel => todo!(),
            FunctionKind::Builtin => todo!(),
            FunctionKind::Oracle => todo!(),
            FunctionKind::Recursive => todo!(),
            FunctionKind::Normal => {
                let _body = self.elaborate_block(function.def.body);
            },
        }
    }

    fn elaborate_expression(&mut self, expr: Expression) -> (ExprId, Type) {
        let (hir_expr, typ) = match expr.kind {
            ExpressionKind::Literal(_) => todo!(),
            ExpressionKind::Block(_) => todo!(),
            ExpressionKind::Prefix(_) => todo!(),
            ExpressionKind::Index(_) => todo!(),
            ExpressionKind::Call(_) => todo!(),
            ExpressionKind::MethodCall(_) => todo!(),
            ExpressionKind::Constructor(_) => todo!(),
            ExpressionKind::MemberAccess(_) => todo!(),
            ExpressionKind::Cast(_) => todo!(),
            ExpressionKind::Infix(_) => todo!(),
            ExpressionKind::If(_) => todo!(),
            ExpressionKind::Variable(_) => todo!(),
            ExpressionKind::Tuple(_) => todo!(),
            ExpressionKind::Lambda(_) => todo!(),
            ExpressionKind::Parenthesized(expr) => return self.elaborate_expression(*expr),
            ExpressionKind::Quote(_) => todo!(),
            ExpressionKind::Comptime(_) => todo!(),
            ExpressionKind::Error => (HirExpression::Error, Type::Error),
        };
        let id = self.interner.push_expr(hir_expr);
        self.interner.push_expr_location(id, expr.span, self.file);
        self.interner.push_expr_type(id, typ.clone());
        (id, typ)
    }

    fn elaborate_statement(&mut self, statement: Statement) -> (StmtId, Type) {
        let (hir_statement, typ) = match statement.kind {
            StatementKind::Let(_) => todo!(),
            StatementKind::Constrain(_) => todo!(),
            StatementKind::Expression(_) => todo!(),
            StatementKind::Assign(_) => todo!(),
            StatementKind::For(_) => todo!(),
            StatementKind::Break => self.elaborate_jump(true, statement.span),
            StatementKind::Continue => self.elaborate_jump(false, statement.span),
            StatementKind::Comptime(_) => todo!(),
            StatementKind::Semi(expr) => {
                let (expr, _typ) = self.elaborate_expression(expr);
                (HirStatement::Semi(expr), Type::Unit)
            }
            StatementKind::Error => (HirStatement::Error, Type::Error),
        };
        let id = self.interner.push_stmt(hir_statement);
        self.interner.push_stmt_location(id, statement.span, self.file);
        (id, typ)
    }

    fn elaborate_block(&mut self, block: BlockExpression) -> HirBlockExpression {
        self.push_scope();

        let statements = vecmap(block.statements, |statement| {
            self.elaborate_statement(statement)
        });

        self.pop_scope();
        HirBlockExpression { statements }
    }

    fn push_scope(&mut self) {
        self.local_scopes.push(Scope::default());
    }

    fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }

    fn elaborate_jump(&mut self, is_break: bool, span: noirc_errors::Span) -> (HirStatement, Type) {
        if !self.in_unconstrained_fn {
            self.push_err(ResolverError::JumpInConstrainedFn { is_break, span });
        }
        if self.nested_loops == 0 {
            self.push_err(ResolverError::JumpOutsideLoop { is_break, span });
        }

        let expr = if is_break {
            HirStatement::Break
        } else {
            HirStatement::Continue
        };
        (expr, self.interner.next_type_variable())
    }

    fn push_err(&mut self, error: impl Into<CompilationError>) {
        self.errors.push(error.into());
    }
}
