use noirc_errors::Location;

use crate::ast::{
    BlockExpression, CallExpression, Expression, ExpressionKind, Ident, IfExpression, Param, Path,
    PathKind, PathSegment, Pattern, Statement, StatementKind, UnaryOp,
};

use super::Elaborator;

impl<'context> Elaborator<'context> {
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

        let func = path(&["std", "runtime", "is_unconstrained"], location);
        let func = var(func);
        let not = not(call(func, Vec::new()));
        let if_ = if_then(not, consequence);
        let statement = Statement { kind: StatementKind::Expression(if_), location };
        statements.insert(0, statement);
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

                let location = ident.location();
                let segments =
                    ["std", "validation", "AssertsIsValidInput", "assert_is_valid_input"];
                let func = path(&segments, location);
                let func = var(func);
                let argument = var(Path::from_ident(ident.clone()));
                let call = call(func, vec![argument]);
                let call = Statement { kind: StatementKind::Semi(call), location };
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
}

fn path(segments: &[&str], location: Location) -> Path {
    let segments = segments.iter().map(|segment| PathSegment {
        ident: Ident::new(segment.to_string(), location),
        generics: None,
        location,
    });
    Path { segments: segments.collect(), kind: PathKind::Plain, location }
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
