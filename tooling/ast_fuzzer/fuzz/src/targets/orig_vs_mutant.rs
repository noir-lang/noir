//! Perform random equivalence mutations on the AST and check that the
//! execution result does not change, a.k.a. metamorphic testing.

use crate::{compare_results, create_ssa_or_die, default_ssa_options};
use arbitrary::{Arbitrary, Unstructured};
use color_eyre::eyre;
use noir_ast_fuzzer::compare::{CompareMutants, CompareOptions};
use noir_ast_fuzzer::{Config, visitor::visit_expr_mut};
use noirc_frontend::ast::UnaryOp;
use noirc_frontend::monomorphization::ast::{Expression, Function, Program, Unary};

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let rules = rules::all();
    let inputs = CompareMutants::arb(
        u,
        Config::default(),
        |u, mut program| {
            let options = CompareOptions::arbitrary(u)?;
            rewrite_program(u, &mut program, &rules);
            Ok((program, options))
        },
        |program, options| create_ssa_or_die(program, &options.onto(default_ssa_options()), None),
    )?;

    let result = inputs.exec()?;

    compare_results(&inputs, &result)
}

fn rewrite_program(u: &mut Unstructured, program: &mut Program, rules: &[rules::Rule]) {
    for func in program.functions.iter_mut() {
        rewrite_function(u, func, rules);
    }
}

fn rewrite_function(u: &mut Unstructured, func: &mut Function, rules: &[rules::Rule]) {
    // TODO: Call `rewrite::next_local_and_ident_id`) and pass the results to the rewrite rules,
    // so they can coordinate adding new variables.
    // TODO: Limit the number of rewrites.
    // TODO: Should we visit the AST once to get a sense of how many rules we can apply,
    //       and pick a ratio based on the limit and the number of options?
    let ctx = rules::Context { unconstrained: func.unconstrained, ..Default::default() };

    rewrite_expr(&ctx, u, &mut func.body, rules);
}

fn rewrite_expr(
    ctx: &rules::Context,
    u: &mut Unstructured,
    expr: &mut Expression,
    rules: &[rules::Rule],
) {
    visit_expr_mut(expr, &mut |expr: &mut Expression| {
        match expr {
            Expression::For(for_) => {
                let range_ctx = rules::Context { is_in_range: true, ..*ctx };
                rewrite_expr(&range_ctx, u, &mut for_.start_range, rules);
                rewrite_expr(&range_ctx, u, &mut for_.end_range, rules);
                rewrite_expr(ctx, u, &mut for_.block, rules);
                // No need to visit children, we just visited them.
                false
            }
            Expression::Unary(
                unary @ Unary { operator: UnaryOp::Reference { mutable: true }, .. },
            ) => {
                let ctx = rules::Context { is_in_ref_mut: true, ..*ctx };
                rewrite_expr(&ctx, u, &mut unary.rhs, rules);
                false
            }
            _ => {
                for rule in rules {
                    match try_apply_rule(ctx, u, expr, rule) {
                        Err(_) => {
                            // We ran out of randomness; stop visiting the AST.
                            return false;
                        }
                        Ok(false) => {
                            // We couldn't, or decided not to apply this rule; try the next one.
                            continue;
                        }
                        Ok(true) => {
                            // We applied a rule on this expression; go to the next expression.
                            break;
                        }
                    }
                }
                // Visit the children of the (modified) expression.
                // For example if `1` is turned into `1 - 0`, then it could be visited again when we visit the children of `-`.
                // Alternatively we could move on to the next sibling by returning `false` if we made a modification.
                true
            }
        }
    });
}

/// Check if a rule can be applied on an expression. If it can, apply it based on some arbitrary
/// criteria, returning a flag showing whether it was applied.
fn try_apply_rule(
    ctx: &rules::Context,
    u: &mut Unstructured,
    expr: &mut Expression,
    rule: &rules::Rule,
) -> arbitrary::Result<bool> {
    // TODO: Make the ratio dynamic.
    if rule.matches(ctx, expr) && u.ratio(1, 10)? {
        rule.rewrite(u, expr)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Metamorphic transformation rules.
mod rules {
    use arbitrary::{Arbitrary, Unstructured};
    use noir_ast_fuzzer::expr;
    use noirc_frontend::{
        ast::BinaryOpKind,
        monomorphization::ast::{Expression, Literal, Type},
    };

    #[derive(Clone, Debug, Default)]
    pub struct Context {
        /// Is the function we're rewriting unconstrained?
        pub unconstrained: bool,
        /// Are we rewriting an expression which is a range of a `for` loop?
        pub is_in_range: bool,
        /// Are we in an expression that we're just taking a mutable reference to?
        pub is_in_ref_mut: bool,
    }

    /// Check if the rule can be applied on an expression.
    type MatchFn = dyn Fn(&Context, &Expression) -> bool;
    /// Apply the rule on an expression, mutating/replacing it in-place.
    type RewriteFn = dyn Fn(&mut Unstructured, &mut Expression) -> arbitrary::Result<()>;

    /// Metamorphic transformation rule.
    pub struct Rule {
        pub matches: Box<MatchFn>,
        pub rewrite: Box<RewriteFn>,
    }

    impl Rule {
        pub fn new(
            matches: impl Fn(&Context, &Expression) -> bool + 'static,
            rewrite: impl Fn(&mut Unstructured, &mut Expression) -> arbitrary::Result<()> + 'static,
        ) -> Self {
            Self { matches: Box::new(matches), rewrite: Box::new(rewrite) }
        }

        /// Check if the rule can be applied on an expression.
        pub fn matches(&self, ctx: &Context, expr: &Expression) -> bool {
            (self.matches)(ctx, expr)
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
            |ctx, expr| {
                // Because of #8305 we can't reliably use expressions in ranges in ACIR.
                if ctx.is_in_range && !ctx.unconstrained {
                    return false;
                }
                // If we rewrite `&mut x` into `&mut (x - 0)` we will alter the semantics.
                if ctx.is_in_ref_mut {
                    return false;
                }
                match expr {
                    Expression::Ident(ident) => {
                        matches!(ident.typ, Type::Field | Type::Integer(_, _))
                    }
                    Expression::Literal(literal) => {
                        matches!(literal, Literal::Integer(_, _, _))
                    }
                    _ => false,
                }
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
