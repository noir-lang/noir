//! Perform random metamorphic mutations on the AST and check that the
//! execution result does not change.

use crate::{compare_results_compiled, create_ssa_or_die, default_ssa_options};
use arbitrary::{Arbitrary, Unstructured};
use color_eyre::eyre;
use noir_ast_fuzzer::compare::{CompareMorph, CompareOptions};
use noir_ast_fuzzer::visitor;
use noir_ast_fuzzer::{Config, visitor::visit_expr_mut};
use noirc_frontend::ast::UnaryOp;
use noirc_frontend::monomorphization::ast::{
    Call, Definition, Expression, Function, Ident, Program, Unary,
};

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let rules = rules::all();
    let max_rewrites = 10;
    let config = Config {
        avoid_overflow: u.arbitrary()?,
        avoid_large_int_literals: true,
        ..Default::default()
    };
    let inputs = CompareMorph::arb(
        u,
        config,
        |u, mut program| {
            let options = CompareOptions::arbitrary(u)?;
            rewrite_program(u, &mut program, &rules, max_rewrites);
            Ok((program, options))
        },
        |program, options| create_ssa_or_die(program, &options.onto(default_ssa_options()), None),
    )?;

    let result = inputs.exec()?;

    compare_results_compiled(&inputs, &result)
}

fn rewrite_program(
    u: &mut Unstructured,
    program: &mut Program,
    rules: &[rules::Rule],
    max_rewrites: usize,
) {
    for func in program.functions.iter_mut() {
        if func.name.ends_with("_proxy") {
            continue;
        }
        rewrite_function(u, func, rules, max_rewrites);
    }
}

fn rewrite_function(
    u: &mut Unstructured,
    func: &mut Function,
    rules: &[rules::Rule],
    max_rewrites: usize,
) {
    // We can call `rewrite::next_local_and_ident_id` and pass the results to the rewrite rules,
    // if they want to add new variables with new local IDs.
    let ctx = rules::Context { unconstrained: func.unconstrained, ..Default::default() };

    let estimate = estimate_applicable_rules(&ctx, &func.body, rules);
    let mut morph = MorphContext { target: max_rewrites.min(estimate), estimate, count: 0, rules };

    morph.rewrite_expr(&ctx, u, &mut func.body);
}

/// Recursively apply rules while keeping a tally on how many we have done.
struct MorphContext<'a> {
    /// Number of rewrites we want to achieve.
    target: usize,
    /// (Over)estimate of the maximum number we could hope to apply.
    estimate: usize,
    /// Number of rewrites applied so far, up to the `target`.
    count: usize,
    /// Rules to apply.
    rules: &'a [rules::Rule],
}

impl MorphContext<'_> {
    /// Check if we have reached the target.
    fn limit_reached(&self) -> bool {
        self.target == 0 || self.estimate == 0 || self.count == self.target
    }

    fn rewrite_expr(&mut self, ctx: &rules::Context, u: &mut Unstructured, expr: &mut Expression) {
        visit_expr_mut(expr, &mut |expr: &mut Expression| {
            if self.limit_reached() {
                return false;
            }
            match expr {
                Expression::For(for_) => {
                    // Separate context for just the ranges.
                    let range_ctx = rules::Context { is_in_range: true, ..*ctx };
                    self.rewrite_expr(&range_ctx, u, &mut for_.start_range);
                    self.rewrite_expr(&range_ctx, u, &mut for_.end_range);
                    // Original context for the body.
                    self.rewrite_expr(ctx, u, &mut for_.block);
                    // No need to visit children, we just visited them.
                    false
                }
                Expression::Unary(
                    unary @ Unary { operator: UnaryOp::Reference { mutable: true }, .. },
                ) => {
                    let ctx = rules::Context { is_in_ref_mut: true, ..*ctx };
                    self.rewrite_expr(&ctx, u, &mut unary.rhs);
                    false
                }
                Expression::Call(call) if is_special_call(call) => {
                    let ctx = rules::Context { is_in_special_call: true, ..*ctx };
                    for arg in call.arguments.iter_mut() {
                        self.rewrite_expr(&ctx, u, arg);
                    }
                    false
                }
                // The rest can just have the rules applied on them, using the same context.
                _ => {
                    for rule in self.rules {
                        match self.try_apply_rule(ctx, u, expr, rule) {
                            Ok(false) => {
                                // We couldn't, or decided not to apply this rule; try the next one.
                                continue;
                            }
                            Err(_) => {
                                // We ran out of randomness; stop visiting the AST.
                                return false;
                            }

                            Ok(true) => {
                                // We applied a rule on this expression.
                                self.count += 1;
                                // We could visit the children of this morphed expression, which could result in repeatedly applying
                                // the same rule over and over again. When we have 100% application rate (e.g. a small function),
                                // it would be wasting all the potential on the first rule that matched, e.g. `(x - (0 + (0 - 0)))`.
                                // It would also throw off the estimate if we introduce new items on which we can apply rules.
                                return false;
                            }
                        }
                    }
                    // If we made it this far, we did not apply any rule, so look deeper in the AST.
                    true
                }
            }
        });
    }

    /// Check if a rule can be applied on an expression. If it can, apply it based on some arbitrary
    /// criteria, returning a flag showing whether it was applied.
    fn try_apply_rule(
        &self,
        ctx: &rules::Context,
        u: &mut Unstructured,
        expr: &mut Expression,
        rule: &rules::Rule,
    ) -> arbitrary::Result<bool> {
        if rule.matches(ctx, expr) && u.ratio(self.target, self.estimate)? {
            rule.rewrite(u, expr)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Provide a rough estimate for how many rules can be applied.
///
/// It will overestimate, because it ignores the finer points of when a rule can match.
fn estimate_applicable_rules(
    ctx: &rules::Context,
    expr: &Expression,
    rules: &[rules::Rule],
) -> usize {
    let mut count = 0;
    visitor::visit_expr(expr, &mut |expr| {
        for rule in rules {
            if rule.matches(ctx, expr) {
                count += 1;
            }
        }
        true
    });
    count
}

/// Check if we are calling an oracle or builtin function.
fn is_special_call(call: &Call) -> bool {
    matches!(
        call.func.as_ref(),
        Expression::Ident(Ident {
            definition: Definition::Oracle(_) | Definition::Builtin(_) | Definition::LowLevel(_),
            ..
        })
    )
}

/// Metamorphic transformation rules.
mod rules {
    use arbitrary::{Arbitrary, Unstructured};
    use noir_ast_fuzzer::expr;
    use noirc_frontend::{
        ast::BinaryOpKind,
        monomorphization::ast::{Binary, Expression, Type},
    };

    #[derive(Clone, Debug, Default)]
    pub struct Context {
        /// Is the function we're rewriting unconstrained?
        pub unconstrained: bool,
        /// Are we rewriting an expression which is a `start` or `end` of a `for` loop?
        pub is_in_range: bool,
        /// Are we in an expression that we're just taking a mutable reference to?
        pub is_in_ref_mut: bool,
        /// Are we processing the arguments of an non-user function call, such as an oracle or built-in?
        pub is_in_special_call: bool,
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
        vec![
            num_plus_minus_zero(),
            bool_or_self(),
            bool_xor_self(),
            bool_xor_rand(),
            commutative_arithmetic(),
            inevitable(),
        ]
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
                // Appending 0 to a block would look odd.
                if matches!(expr, Expression::Block(_)) {
                    return false;
                }
                // We can apply this rule on anything that returns a number.
                if let Some(typ) = expr.return_type() {
                    matches!(typ.as_ref(), Type::Field | Type::Integer(_, _))
                } else {
                    false
                }
            },
            |u, expr| {
                let typ = expr.return_type().expect("only called on matching type").into_owned();

                let op =
                    if bool::arbitrary(u)? { BinaryOpKind::Add } else { BinaryOpKind::Subtract };

                expr::replace(expr, |expr| {
                    expr::binary(expr, op, expr::int_literal(0u32, false, typ))
                });

                Ok(())
            },
        )
    }

    /// Transform boolean value `x` into `x | x`.
    pub fn bool_or_self() -> Rule {
        Rule::new(bool_rule_matches, |_u, expr| {
            expr::replace(expr, |expr| expr::binary(expr.clone(), BinaryOpKind::Or, expr));
            Ok(())
        })
    }

    /// Transform boolean value `x` into `x ^ x ^ x`.
    pub fn bool_xor_self() -> Rule {
        Rule::new(bool_rule_matches, |_u, expr| {
            expr::replace(expr, |expr| {
                let rhs = expr::binary(expr.clone(), BinaryOpKind::Xor, expr.clone());
                expr::binary(expr, BinaryOpKind::Xor, rhs)
            });
            Ok(())
        })
    }

    /// Transform boolean value `x` into `rnd ^ x ^ rnd`.
    pub fn bool_xor_rand() -> Rule {
        Rule::new(bool_rule_matches, |u, expr| {
            // This is where we could access the scope to look for a random bool variable.
            let rnd = expr::gen_literal(u, &Type::Bool)?;
            expr::replace(expr, |expr| {
                let rhs = expr::binary(expr, BinaryOpKind::Xor, rnd.clone());
                expr::binary(rnd, BinaryOpKind::Xor, rhs)
            });
            Ok(())
        })
    }

    /// Transform commutative arithmetic operations:
    /// * `a + b` into `b + a`
    /// * `a * b` into `b * a`
    pub fn commutative_arithmetic() -> Rule {
        Rule::new(
            |_ctx, expr| {
                matches!(
                    expr,
                    Expression::Binary(Binary {
                        operator: BinaryOpKind::Add | BinaryOpKind::Multiply,
                        ..
                    })
                ) && !has_side_effect(expr)
            },
            |_u, expr| {
                let Expression::Binary(binary) = expr else {
                    unreachable!("the rule only matches Binary expressions");
                };

                std::mem::swap(&mut binary.lhs, &mut binary.rhs);

                Ok(())
            },
        )
    }

    /// Transform an expression into an if-then-else with the expression
    /// repeated in the _then_ and _else_ branch:
    /// * `x` into `if c { x } else { x }`
    pub fn inevitable() -> Rule {
        Rule::new(
            |ctx, _expr| !ctx.is_in_special_call && !ctx.is_in_ref_mut,
            |u, expr| {
                let typ = expr.return_type().map(|typ| typ.into_owned()).unwrap_or(Type::Unit);
                // *expr = expr::if_else(gen_condition(u)?, expr.clone(), expr.clone(), typ);

                Ok(())
            },
        )
    }

    /// Check if an expression can have a side effect, in which case duplicating or reordering it could
    /// change the behavior of the program.
    fn has_side_effect(expr: &Expression) -> bool {
        expr::exists(expr, |expr| {
            matches!(
                expr,
                Expression::Call(_) // Functions can have side effects, maybe mutating some reference, printing
                | Expression::Assign(_) // Assignment to a mutable variable could double up effects
            )
        })
    }

    /// Common match condition for boolean rules.
    fn bool_rule_matches(ctx: &Context, expr: &Expression) -> bool {
        // If we rewrite `&mut x` into `&mut (x | x)` we will alter the semantics.
        if ctx.is_in_ref_mut {
            return false;
        }
        // We don't want to mess with the arguments of a `println`, because the printer assumes they are bool literals.
        // Similarly a `constrain` call is expected to have a single boolean expression.
        if ctx.is_in_special_call {
            return false;
        }
        // We can apply boolean rule on anything that returns a bool,
        // unless the expression can have a side effect, which we don't want to duplicate.
        if let Some(typ) = expr.return_type() {
            matches!(typ.as_ref(), Type::Bool)
                && !has_side_effect(expr)
                && !expr::exists(expr, |expr| {
                    matches!(
                        expr,
                        Expression::Let(_) // Creating a variable needs a new ID
                    | Expression::Block(_) // Applying logical operations on blocks would look odd
                    )
                })
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    /// ```ignore
    /// NOIR_ARBTEST_SEED=0xb2fb5f0b00100000 \
    /// NOIR_AST_FUZZER_SHOW_AST=1 \
    /// cargo test -p noir_ast_fuzzer_fuzz orig_vs_morph
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        crate::targets::tests::fuzz_with_arbtest(super::fuzz);
    }
}
