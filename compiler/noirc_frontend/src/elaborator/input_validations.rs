use noirc_errors::Location;

use crate::{
    Type,
    ast::{
        BlockExpression, CallExpression, Expression, ExpressionKind, Ident, IfExpression, Param,
        Path, PathKind, PathSegment, Pattern, Statement, StatementKind, UnaryOp,
    },
    hir_def::{
        expr::{HirBlockExpression, HirExpression, HirIdent},
        stmt::{HirLetStatement, HirPattern, HirStatement},
    },
    node_interner::DefinitionKind,
};

use super::Elaborator;

impl Elaborator<'_> {
    /// Adds statements to `statements` to validate the given parameters.
    ///
    /// For example, this function:
    ///
    /// fn main(x: u8, y: u8) {
    ///     assert_eq(x, y);
    /// }
    ///
    /// is transformed into this one:
    ///
    /// fn main(x: u8, y: u8) {
    ///     if !std::runtime::is_unconstrained() {
    ///         std::validation::AssertsIsValidInput::assert_is_valid_input(x);
    ///         std::validation::AssertsIsValidInput::assert_is_valid_input(y);
    ///     }
    ///     assert_eq(x, y);
    /// }
    pub(super) fn add_entry_point_parameters_validation(
        &self,
        params: &[Param],
        statements: &mut Vec<Statement>,
    ) {
        if params.is_empty() {
            return;
        }

        let location = params[0].location;

        let mut consequence_statements = Vec::with_capacity(params.len());
        for param in params {
            self.add_entry_point_pattern_validation(&param.pattern, &mut consequence_statements);
        }

        let consequence = BlockExpression { statements: consequence_statements };
        let consequence = ExpressionKind::Block(consequence);
        let consequence = Expression::new(consequence, location);

        let statement = self.if_not_unconstrained(consequence);
        statements.insert(0, statement);
    }

    // Given an expression, wraps it with input validation.
    //
    // For example, if the expression is this call:
    //
    // foo(...)
    //
    // it's wrapped like this:
    //
    // {
    //     let tmp = foo(..)
    //     if !std::runtime::is_unconstrained() {
    //         std::validation::AssertsIsValidInput::assert_is_valid_input(tmp);
    //     }
    //     tmp
    // }
    pub(super) fn wrap_with_input_validation(
        &mut self,
        hir_expr: HirExpression,
        typ: Type,
        location: Location,
    ) -> (HirExpression, Type) {
        // We can't insert validations if the standard library doesn't exist (mainly in frontend tests)
        if self.crate_graph.try_stdlib_crate_id().is_none() {
            return (hir_expr, typ);
        }

        // We need a new scope for the `tmp` variable
        self.push_scope();

        // At this point the expression wasn't interned, so we do it here
        let expr_id = self.interner.push_expr(hir_expr);
        self.interner.push_expr_location(expr_id, location);
        self.interner.push_expr_type(expr_id, typ.clone());

        // Declare the temporary variable
        let tmp_name = Ident::new("(tmp)".into(), location);
        let kind = DefinitionKind::Local(None);
        let tmp_definition_id =
            self.add_variable_decl(tmp_name.clone(), false, true, false, kind).id;
        self.interner.push_definition_type(tmp_definition_id, typ.clone());
        let tmp_ident = HirIdent::non_trait_method(tmp_definition_id, location);

        // This is `let tmp = ...`;
        let let_statement = HirStatement::Let(HirLetStatement {
            pattern: HirPattern::Identifier(tmp_ident),
            r#type: typ.clone(),
            expression: expr_id,
            attributes: Vec::new(),
            comptime: false,
            is_global_let: false,
        });
        let let_stmt_id = self.interner.push_stmt(let_statement);
        self.interner.push_stmt_location(let_stmt_id, location);

        // This is `if !std::runtime::is_unconstrained() { std::validation::AssertsIsValidInput::assert_is_valid_input(tmp) }`
        let call = self.assert_is_valid_input_call(tmp_name.clone());
        let call_statement = self.if_not_unconstrained(call);
        let (call_stmt_id, _) = self.elaborate_statement(call_statement);

        // This is `tmp`
        let ret = var(Path::from_ident(tmp_name));
        let ret_statement = Statement { kind: StatementKind::Expression(ret), location };
        let (ret_stmt_id, _) = self.elaborate_statement(ret_statement);

        self.pop_scope();

        let hir_block_expr =
            HirBlockExpression { statements: vec![let_stmt_id, call_stmt_id, ret_stmt_id] };

        (HirExpression::Block(hir_block_expr), typ)
    }

    fn add_entry_point_pattern_validation(
        &self,
        pattern: &Pattern,
        statements: &mut Vec<Statement>,
    ) {
        match pattern {
            Pattern::Identifier(ident) => {
                if ident.0.contents == "_" {
                    return;
                }

                let call = self.assert_is_valid_input_call(ident.clone());
                let call =
                    Statement { kind: StatementKind::Semi(call), location: ident.location() };
                statements.push(call);
            }
            Pattern::Mutable(pattern, ..) => {
                self.add_entry_point_pattern_validation(pattern, statements);
            }
            Pattern::Tuple(patterns, ..) => {
                for pattern in patterns {
                    self.add_entry_point_pattern_validation(pattern, statements);
                }
            }
            Pattern::Struct(..) => todo!("add_entry_point_pattern_validation for Struct pattern"),
            Pattern::Interned(interned_pattern, ..) => {
                let pattern = self.interner.get_pattern(*interned_pattern);
                self.add_entry_point_pattern_validation(pattern, statements);
            }
        }
    }

    /// Given `ident`, returns this expression:
    ///
    /// std::validation::AssertIsValidInput::assert_is_valid_input(ident)
    fn assert_is_valid_input_call(&self, ident: Ident) -> Expression {
        let location = ident.location();
        let func = if self.module_id().krate.is_stdlib() {
            let segments = ["validation", "AssertsIsValidInput", "assert_is_valid_input"];
            path(PathKind::Crate, &segments, location)
        } else {
            let segments = ["std", "validation", "AssertsIsValidInput", "assert_is_valid_input"];
            path(PathKind::Plain, &segments, location)
        };
        let func = var(func);
        let argument = var(Path::from_ident(ident.clone()));
        call(func, vec![argument])
    }

    /// Given `consequence`, returns this statement:
    ///
    /// if !std::runtime::is_unconstrained() {
    ///     consequence
    /// }
    fn if_not_unconstrained(&self, consequence: Expression) -> Statement {
        let location = consequence.location;
        let func = if self.module_id().krate.is_stdlib() {
            path(PathKind::Crate, &["runtime", "is_unconstrained"], location)
        } else {
            path(PathKind::Plain, &["std", "runtime", "is_unconstrained"], location)
        };
        let func = var(func);
        let not = not(call(func, Vec::new()));
        let if_ = if_then(not, consequence);
        Statement { kind: StatementKind::Expression(if_), location }
    }
}

fn path(kind: PathKind, segments: &[&str], location: Location) -> Path {
    let segments = segments.iter().map(|segment| PathSegment {
        ident: Ident::new(segment.to_string(), location),
        generics: None,
        location,
    });
    Path { segments: segments.collect(), kind, location, kind_location: location }
}

fn var(path: Path) -> Expression {
    let location = path.location;
    let var = ExpressionKind::Variable(path);
    Expression::new(var, location)
}

fn call(func: Expression, arguments: Vec<Expression>) -> Expression {
    let location = func.location;
    let call = CallExpression { func: Box::new(func), arguments, is_macro_call: false };
    Expression::new(ExpressionKind::Call(Box::new(call)), location)
}

fn not(rhs: Expression) -> Expression {
    let location = rhs.location;
    let not = ExpressionKind::prefix(UnaryOp::Not, rhs);
    Expression::new(not, location)
}

fn if_then(condition: Expression, consequence: Expression) -> Expression {
    let location = condition.location;
    let if_ =
        ExpressionKind::If(Box::new(IfExpression { condition, consequence, alternative: None }));
    Expression::new(if_, location)
}
