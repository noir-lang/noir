//! Perform random equivalence mutations on the AST and check that the
//! execution result does not change, a.k.a. metamorphic testing.

use crate::{compare_results, create_ssa_or_die, default_ssa_options};
use arbitrary::{Arbitrary, Unstructured};
use color_eyre::eyre;
use noir_ast_fuzzer::compare::{CompareMutants, CompareOptions};
use noir_ast_fuzzer::{Config, visit_expr_mut};
use noirc_frontend::monomorphization::ast::{Expression, Program};

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let rules = rules::all();
    let inputs = CompareMutants::arb(
        u,
        Config::default(),
        |u, program| {
            let options = CompareOptions::arbitrary(u)?;
            let program = apply_rules(u, program, &rules);
            Ok((program, options))
        },
        |program, options| create_ssa_or_die(program, &options.onto(default_ssa_options()), None),
    )?;

    let result = inputs.exec()?;

    compare_results(&inputs, &result)
}

fn apply_rules(u: &mut Unstructured, mut program: Program, rules: &[rules::Rule]) -> Program {
    for func in program.functions.iter_mut() {
        // TODO: Call `rewrite::next_local_and_ident_id`) and pass the results to the rewrite rules,
        // so they can coordinate adding new variables.
        // TODO: Limit the number of rewrites.
        // TODO: Should we visit the AST once to get a sense of how many rules we can apply,
        //       and pick a ratio based on the limit and the number of options?
        visit_expr_mut(&mut func.body, &mut |expr: &mut Expression| {
            for rule in rules {
                if !rule.matches(expr) {
                    // We can't apply this rule, try the next one.
                    continue;
                }
                // TODO: Make the ratio dynamic.
                match u.ratio(1, 10) {
                    Err(_) => {
                        // We ran out of randomness, no point visiting the AST any further.
                        return false;
                    }
                    Ok(false) => {
                        // We didn't pick this rule, try the next one.
                        continue;
                    }
                    Ok(true) => {
                        // Apply this rule.
                        if rule.rewrite(u, expr).is_err() {
                            // We ran out of randomness.
                            return false;
                        } else {
                            // We applied the rule; let's go visit the next node in the AST.
                            break;
                        }
                    }
                }
            }
            // Visit the children of the (modified) expression.
            // For example if `1` is turned into `1 - 0`, then it could be visited again when we visit the children of `-`.
            // Alternatively we could move on to the next sibling by returning `false` if we made a modification.
            true
        });
    }
    program
}

/// Metamorphic transformation rules.
mod rules {
    use arbitrary::{Arbitrary, Unstructured};
    use noir_ast_fuzzer::expr;
    use noirc_frontend::{
        ast::BinaryOpKind,
        monomorphization::ast::{Expression, Literal, Type},
    };

    /// Check if the rule can be applied on an expression.
    type MatchFn = dyn Fn(&Expression) -> bool;
    /// Apply the rule on an expression, mutating/replacing it in-place.
    type RewriteFn = dyn Fn(&mut Unstructured, &mut Expression) -> arbitrary::Result<()>;

    /// Metamorphic transformation rule.
    pub struct Rule {
        pub matches: Box<MatchFn>,
        pub rewrite: Box<RewriteFn>,
    }

    impl Rule {
        pub fn new(
            matches: impl Fn(&Expression) -> bool + 'static,
            rewrite: impl Fn(&mut Unstructured, &mut Expression) -> arbitrary::Result<()> + 'static,
        ) -> Self {
            Self { matches: Box::new(matches), rewrite: Box::new(rewrite) }
        }

        /// Check if the rule can be applied on an expression.
        pub fn matches(&self, expr: &Expression) -> bool {
            (self.matches)(expr)
        }

        /// Apply the rule on an expression, mutating/replacing it in-place.
        pub fn rewrite(
            &self,
            u: &mut Unstructured,
            expr: &mut Expression,
        ) -> arbitrary::Result<()> {
            (self.rewrite)(u, expr)
        }
    }

    /// Construct all rules that we can apply on a program.
    pub fn all() -> Vec<Rule> {
        vec![num_plus_minus_zero()]
    }

    /// Transform any numeric value `x` into `x +/- 0`.
    pub fn num_plus_minus_zero() -> Rule {
        Rule::new(
            |expr| match expr {
                Expression::Ident(ident) => {
                    matches!(ident.typ, Type::Field | Type::Integer(_, _))
                }
                Expression::Literal(literal) => {
                    matches!(literal, Literal::Integer(_, _, _))
                }
                _ => false,
            },
            |u, expr| {
                let typ = match expr {
                    Expression::Ident(ident) => ident.typ.clone(),
                    Expression::Literal(Literal::Integer(_, typ, _)) => typ.clone(),
                    _ => unreachable!(),
                };

                let op =
                    if bool::arbitrary(u)? { BinaryOpKind::Add } else { BinaryOpKind::Subtract };

                expr::replace(expr, |expr| {
                    expr::binary(expr.clone(), op, expr::int_literal(0u32, false, typ))
                });

                Ok(())
            },
        )
    }
}

#[cfg(test)]
mod tests {
    /// ```ignore
    /// NOIR_ARBTEST_SEED=0xb2fb5f0b00100000 \
    /// NOIR_AST_FUZZER_SHOW_AST=1 \
    /// cargo test -p noir_ast_fuzzer_fuzz orig_vs_mutant
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        crate::targets::tests::fuzz_with_arbtest(super::fuzz);
    }
}
