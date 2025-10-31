//! Perform random metamorphic mutations on the AST and check that the
//! execution result does not change.

use std::cell::{Cell, RefCell};

use crate::targets::default_config;
use crate::{compare_results_compiled, compile_into_circuit_or_die, default_ssa_options};
use arbitrary::{Arbitrary, Unstructured};
use color_eyre::eyre;
use noir_ast_fuzzer::compare::{CompareMorph, CompareOptions};
use noir_ast_fuzzer::rewrite;
use noir_ast_fuzzer::scope::ScopeStack;
use noirc_frontend::ast::UnaryOp;
use noirc_frontend::monomorphization::ast::{
    Call, Definition, Expression, Function, Ident, IdentId, LocalId, Program, Unary,
};
use noirc_frontend::monomorphization::visitor::{visit_expr, visit_expr_be_mut};

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let config = default_config(u)?;
    let rules = rules::collect(&config);
    let max_rewrites = 10;
    let inputs = CompareMorph::arb(
        u,
        config,
        |u, mut program| {
            let options = CompareOptions::arbitrary(u)?;
            rewrite_program(u, &mut program, &rules, max_rewrites);
            Ok((program, options))
        },
        |program, options| {
            compile_into_circuit_or_die(program, &options.onto(default_ssa_options()), None)
        },
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
    let morph = MorphContext {
        target: max_rewrites.min(estimate),
        estimate,
        count: Cell::new(0),
        rules,
        vars: RefCell::new(VariableContext::new(func)),
    };

    morph.rewrite_expr(&ctx, u, &mut func.body);
}

/// Context necessary to generate new local IDs during rewrites.
///
/// Potentially a place to reconstruct local variable scopes.
struct VariableContext {
    next_local_id: u32,
    next_ident_id: u32,
    locals: ScopeStack<LocalId>,
}

impl VariableContext {
    fn new(func: &Function) -> Self {
        let (next_local_id, next_ident_id) = rewrite::next_local_and_ident_id(func);

        let locals = ScopeStack::from_variables(
            func.parameters
                .iter()
                .map(|(id, mutable, name, typ, _vis)| (*id, *mutable, name.clone(), typ.clone())),
        );

        Self { next_local_id, next_ident_id, locals }
    }

    fn next_local_id(&mut self) -> LocalId {
        let id = self.next_local_id;
        self.next_local_id += 1;
        LocalId(id)
    }

    fn next_ident_id(&mut self) -> IdentId {
        let id = self.next_ident_id;
        self.next_ident_id += 1;
        IdentId(id)
    }
}

/// Recursively apply rules while keeping a tally on how many we have done.
struct MorphContext<'a> {
    /// Number of rewrites we want to achieve.
    target: usize,
    /// (Over)estimate of the maximum number we could hope to apply.
    estimate: usize,
    /// Number of rewrites applied so far, up to the `target`.
    count: Cell<usize>,
    /// Rules to apply.
    rules: &'a [rules::Rule],
    /// Book keeping for local variables.
    vars: RefCell<VariableContext>,
}

impl MorphContext<'_> {
    /// Check if we have reached the target.
    fn limit_reached(&self) -> bool {
        self.target == 0 || self.estimate == 0 || self.count.get() == self.target
    }

    fn rewrite_expr(&self, ctx: &rules::Context, u: &mut Unstructured, expr: &mut Expression) {
        visit_expr_be_mut(
            expr,
            &mut |expr: &mut Expression| {
                if self.limit_reached() {
                    return (false, false);
                }

                // Check if we are entering a new scope.
                let entered = matches!(
                    expr,
                    Expression::Block(_)
                        | Expression::For(_)
                        | Expression::Loop(_)
                        | Expression::While(_)
                        | Expression::If(_)
                );

                if entered {
                    self.vars.borrow_mut().locals.enter();
                }

                // We apply the rules on this expression, but its
                // children will be visited after this call.
                let cont = self.apply_rules(ctx, u, expr);

                (cont, entered)
            },
            &mut |expr, entered| {
                // A `let` variable becomes visible *after* we have have processed all its children,
                // so, only its siblings can see it.
                if let Expression::Let(let_) = expr {
                    let typ = let_.expression.return_type().expect("let should have a type");
                    self.vars.borrow_mut().locals.add(
                        let_.id,
                        let_.mutable,
                        let_.name.clone(),
                        typ.into_owned(),
                    );
                }
                if entered {
                    self.vars.borrow_mut().locals.exit();
                }
            },
            &mut |_| {},
        );
    }

    fn apply_rules(
        &self,
        ctx: &rules::Context,
        u: &mut Unstructured,
        expr: &mut Expression,
    ) -> bool {
        match expr {
            Expression::For(for_) => {
                // Separate context for just the ranges.
                let range_ctx = rules::Context { is_in_range: true, ..*ctx };
                self.rewrite_expr(&range_ctx, u, &mut for_.start_range);
                self.rewrite_expr(&range_ctx, u, &mut for_.end_range);
                // Original context for the body.
                self.vars.borrow_mut().locals.add(
                    for_.index_variable,
                    false,
                    for_.index_name.clone(),
                    for_.index_type.clone(),
                );
                self.rewrite_expr(ctx, u, &mut for_.block);
                self.vars.borrow_mut().locals.remove(&for_.index_variable);
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
                            self.count.set(self.count.get() + 1);
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
            rule.rewrite(u, &mut self.vars.borrow_mut(), expr)?;
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
    visit_expr(expr, &mut |expr| {
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
    use crate::targets::orig_vs_morph::{
        VariableContext,
        helpers::{has_side_effect, reassign_ids},
    };

    use super::helpers::gen_expr;
    use acir::{AcirField, FieldElement};
    use arbitrary::Unstructured;
    use noir_ast_fuzzer::{Config, expr, types};
    use noirc_frontend::{
        ast::BinaryOpKind,
        monomorphization::ast::{Binary, Definition, Expression, Ident, Literal, Type},
        signed_field::SignedField,
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
    type RewriteFn =
        dyn Fn(&mut Unstructured, &mut VariableContext, &mut Expression) -> arbitrary::Result<()>;

    /// Metamorphic transformation rule.
    pub struct Rule {
        pub matches: Box<MatchFn>,
        pub rewrite: Box<RewriteFn>,
    }

    impl Rule {
        pub fn new(
            matches: impl Fn(&Context, &Expression) -> bool + 'static,
            rewrite: impl Fn(
                &mut Unstructured,
                &mut VariableContext,
                &mut Expression,
            ) -> arbitrary::Result<()>
            + 'static,
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
            vars: &mut VariableContext,
            expr: &mut Expression,
        ) -> arbitrary::Result<()> {
            (self.rewrite)(u, vars, expr)
        }
    }

    /// Construct all rules that we can apply on a program.
    pub fn collect(config: &Config) -> Vec<Rule> {
        let mut rules = vec![
            num_add_zero(),
            num_sub_zero(),
            num_mul_one(),
            num_div_one(),
            bool_or_self(),
            bool_xor_self(),
            bool_xor_rand(),
            any_inevitable(),
            int_break_up(),
        ];
        if config.avoid_overflow {
            // When we can overflowing instruction, then swapping around the LHS and RHS
            // of a binary operation can swap failures. We could visit the expressions to rule
            // out a potential failure on both sides at the same time, or just skip this rule.
            rules.push(num_commute());
        }
        rules
    }

    /// Transform any numeric value `x` into `x <op> <rhs>`
    fn num_op(op: BinaryOpKind, rhs: u32) -> Rule {
        Rule::new(num_rule_matches, move |_u, _locals, expr| {
            let typ = expr.return_type().expect("only called on matching type").into_owned();
            expr::replace(expr, |expr| expr::binary(expr, op, expr::int_literal(rhs, false, typ)));
            Ok(())
        })
    }

    /// Transform any numeric value `x` into `x+0`
    pub fn num_add_zero() -> Rule {
        num_op(BinaryOpKind::Add, 0)
    }

    /// Transform any numeric value `x` into `x-0`
    pub fn num_sub_zero() -> Rule {
        num_op(BinaryOpKind::Subtract, 0)
    }

    /// Transform any numeric value `x` into `x*1`
    pub fn num_mul_one() -> Rule {
        num_op(BinaryOpKind::Multiply, 1)
    }

    /// Transform any numeric value `x` into `x/1`
    pub fn num_div_one() -> Rule {
        num_op(BinaryOpKind::Divide, 1)
    }

    /// Break an integer literal `a` into `b + c`.
    pub fn int_break_up() -> Rule {
        Rule::new(
            |ctx, expr| {
                if ctx.is_in_range && !ctx.unconstrained || ctx.is_in_ref_mut {
                    return false;
                }
                matches!(expr, Expression::Literal(Literal::Integer(_, Type::Integer(_, _), _)))
            },
            |u, _locals, expr| {
                let Expression::Literal(Literal::Integer(a, typ, loc)) = expr else {
                    unreachable!("matched only integer literals, got {expr}");
                };
                let mut b_expr = expr::gen_literal(u, typ, &Config::default())?;
                let Expression::Literal(Literal::Integer(b, _, _)) = &mut b_expr else {
                    unreachable!("generated a literal of the same type");
                };

                // Make them have the same sign, so they are on the same side of 0 and a single number
                // can add up to them without overflow. (e.g. there is no x such that `i32::MIN + x == i32::MAX`)
                if a.is_negative() && !b.is_negative() {
                    *b = SignedField::negative(b.absolute_value());
                } else if !a.is_negative() && b.is_negative() {
                    *b = SignedField::positive(b.absolute_value() - FieldElement::one()); // -1 just to avoid the potential of going from e.g. i8 -128 to 128 where the maximum is 127.
                }

                let (op, c) = if *a >= *b {
                    (BinaryOpKind::Add, (*a - *b))
                } else {
                    (BinaryOpKind::Subtract, (*b - *a))
                };

                let c_expr = Expression::Literal(Literal::Integer(c, typ.clone(), *loc));

                *expr = expr::binary(b_expr, op, c_expr);

                Ok(())
            },
        )
    }

    /// Transform boolean value `x` into `x | x`.
    pub fn bool_or_self() -> Rule {
        Rule::new(bool_rule_matches, |_u, _locals, expr| {
            expr::replace(expr, |expr| expr::binary(expr.clone(), BinaryOpKind::Or, expr));
            Ok(())
        })
    }

    /// Transform boolean value `x` into `x ^ x ^ x`.
    pub fn bool_xor_self() -> Rule {
        Rule::new(bool_rule_matches, |_u, _locals, expr| {
            expr::replace(expr, |expr| {
                let rhs = expr::binary(expr.clone(), BinaryOpKind::Xor, expr.clone());
                expr::binary(expr, BinaryOpKind::Xor, rhs)
            });
            Ok(())
        })
    }

    /// Transform boolean value `x` into `rnd ^ x ^ rnd`.
    pub fn bool_xor_rand() -> Rule {
        Rule::new(bool_rule_matches, |u, _locals, expr| {
            // This is where we could access the scope to look for a random bool variable.
            let rnd = expr::gen_literal(u, &Type::Bool, &Config::default())?;
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
    pub fn num_commute() -> Rule {
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
            |_u, _locals, expr| {
                let Expression::Binary(binary) = expr else {
                    unreachable!("the rule only matches Binary expressions");
                };

                std::mem::swap(&mut binary.lhs, &mut binary.rhs);

                Ok(())
            },
        )
    }

    /// Transform any expression into an if-then-else with the itself
    /// repeated in the _then_ and _else_ branch:
    /// * `x` into `if c { x } else { x }`
    pub fn any_inevitable() -> Rule {
        Rule::new(
            |ctx, expr| {
                !ctx.is_in_special_call
                    && !ctx.is_in_ref_mut
                    // If we're in ACIR then don't turn loop ranges into non-constant expressions.
                    && (ctx.unconstrained || !ctx.is_in_range)
                    // `let x = 1;` transformed into `if true { let x = 1; } else { let x = 1; }` would leave `x` undefined.
                    && !matches!(expr, Expression::Let(_))
                    // We can't return references from an `if` statement
                    && expr.return_type().map(|typ| !types::contains_reference(typ.as_ref())).unwrap_or(true)
            },
            |u, vars, expr| {
                let typ = expr.return_type().map(|typ| typ.into_owned()).unwrap_or(Type::Unit);

                // Find a bool expression we can use. For simplicity just consider actual bool variables,
                // not things that can produce variables, so we have less logic to repeat for the `FunctionContext`.
                let bool_vars = vars
                    .locals
                    .current()
                    .variables()
                    .filter_map(|(id, (_, _, typ))| (*typ == Type::Bool).then_some(id))
                    .collect::<Vec<_>>();

                // If we don't have a bool variable, generate some random expression.
                let cond = if bool_vars.is_empty() {
                    gen_expr(u, &Type::Bool, 2)?
                } else {
                    let id = u.choose_iter(bool_vars)?;
                    let (mutable, name, typ) = vars.locals.current().get_variable(id);
                    Expression::Ident(Ident {
                        location: None,
                        definition: Definition::Local(*id),
                        mutable: *mutable,
                        name: name.clone(),
                        typ: typ.clone(),
                        id: vars.next_ident_id(),
                    })
                };

                // Duplicate the expression, then assign new IDs to all variables created in it.
                let mut alt = expr.clone();

                reassign_ids(vars, &mut alt);

                expr::replace(expr, |expr| expr::if_else(cond, expr, alt, typ));
                Ok(())
            },
        )
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
                        Expression::Let(_)     // Creating a variable needs a new ID
                        | Expression::Match(_) // Match creates variables which would need new IDs
                        | Expression::Block(_) // Applying logical operations on blocks would look odd
                    )
                })
        } else {
            false
        }
    }

    /// Common condition for numeric rules
    fn num_rule_matches(ctx: &Context, expr: &Expression) -> bool {
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
    }
}

mod helpers {
    use std::{cell::RefCell, collections::HashMap, sync::OnceLock};

    use arbitrary::Unstructured;
    use noir_ast_fuzzer::{Config, expr, types};
    use noirc_frontend::{
        ast::{IntegerBitSize, UnaryOp},
        monomorphization::{
            ast::{BinaryOp, Definition, Expression, LocalId, Type},
            visitor::visit_expr_be_mut,
        },
        shared::Signedness,
    };
    use strum::IntoEnumIterator;

    use crate::targets::orig_vs_morph::VariableContext;

    /// Check if an expression can have a side effect, in which case duplicating or reordering it could
    /// change the behavior of the program. This doesn't concern about failures, just observable changes
    /// the state of the program.
    pub(super) fn has_side_effect(expr: &Expression) -> bool {
        expr::exists(expr, |expr| {
            matches!(
                expr,
                Expression::Call(_) // Functions can have side effects, maybe mutating some reference, printing
                | Expression::Assign(_) // Assignment to a mutable variable could double up effects
            )
        })
    }

    /// Generate an arbitrary pure (free of side effects) expression, returning a specific type.
    pub(super) fn gen_expr(
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Expression> {
        if max_depth > 0 {
            let idx = u.choose_index(3)?;
            if idx == 0 {
                if let Some(expr) = gen_unary(u, typ, max_depth)? {
                    return Ok(expr);
                }
            }
            if idx == 1 {
                if let Some(expr) = gen_binary(u, typ, max_depth)? {
                    return Ok(expr);
                }
            }
        }
        expr::gen_literal(u, typ, &Config::default())
    }

    /// Generate an arbitrary unary expression, returning a specific type.
    pub(super) fn gen_unary(
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<Expression>> {
        if !types::can_unary_return(typ) {
            return Ok(None);
        }
        let mut make_unary = |op| {
            let expr = gen_expr(u, typ, max_depth.saturating_sub(1))?;
            Ok(Some(expr::unary(op, expr, typ.clone())))
        };
        if types::is_numeric(typ) {
            make_unary(UnaryOp::Minus)
        } else if types::is_bool(typ) {
            make_unary(UnaryOp::Not)
        } else {
            Ok(None)
        }
    }

    /// Generate an arbitrary binary expression, returning a specific type.
    ///
    /// The operation should not have any side effects, ie. it must not fail.
    pub(super) fn gen_binary(
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<Expression>> {
        // Collect the operations can return the expected type.
        // Do not introduce new errors in randomly generated code that only exists in the morph.
        let ops = BinaryOp::iter()
            .filter(|op| {
                types::can_binary_op_return(op, typ)
                    && !types::can_binary_op_overflow(op)
                    && !types::can_binary_op_err_by_zero(op)
            })
            .collect::<Vec<_>>();

        // Ideally we checked that the target type can be returned, but just in case.
        if ops.is_empty() {
            return Ok(None);
        }

        // Choose a random operation.
        let op = u.choose_iter(ops)?;

        let type_options = TYPES.get_or_init(|| {
            let mut types = vec![Type::Bool, Type::Field];

            for sign in [Signedness::Signed, Signedness::Unsigned] {
                for size in IntegerBitSize::iter() {
                    if sign.is_signed() && size.bit_size() == 1 {
                        continue;
                    }
                    // Avoid negative literals; the frontend makes them difficult to work with in expressions
                    // where no type inference information is available.
                    if sign.is_signed() {
                        continue;
                    }
                    // Avoid large integers; frontend doesn't like them.
                    if size.bit_size() > 32 {
                        continue;
                    }
                    types.push(Type::Integer(sign, size));
                }
            }
            types
        });

        // Select input types that can produce the output we want.
        let type_options = type_options
            .iter()
            .filter(|input| types::can_binary_op_return_from_input(&op, input, typ))
            .collect::<Vec<_>>();

        // Choose a type for the LHS and RHS.
        let lhs_type = u.choose_iter(type_options)?;

        // Generate expressions for LHS and RHS.
        let lhs_expr = gen_expr(u, lhs_type, max_depth.saturating_sub(1))?;
        let rhs_expr = gen_expr(u, lhs_type, max_depth.saturating_sub(1))?;

        let mut expr = expr::binary(lhs_expr, op, rhs_expr);

        // If we have chosen e.g. u8 and need u32 we need to cast.
        if !(lhs_type == typ || types::is_bool(typ) && op.is_comparator()) {
            expr = expr::cast(expr, typ.clone());
        }

        Ok(Some(expr))
    }

    /// Types we can consider using in this context.
    static TYPES: OnceLock<Vec<Type>> = OnceLock::new();

    /// Assign new IDs to variables and identifiers created in the expression.
    pub(super) fn reassign_ids(vars: &mut VariableContext, expr: &mut Expression) {
        fn replace_local_id(
            vars: &mut VariableContext,
            replacements: &mut HashMap<LocalId, LocalId>,
            id: &mut LocalId,
        ) {
            let curr = *id;
            let next = vars.next_local_id();
            replacements.insert(curr, next);
            *id = next;
        }

        let replacements = RefCell::new(HashMap::new());

        visit_expr_be_mut(
            expr,
            // Assign a new ID where variables are created, and remember what original value they replaced.
            &mut |expr| {
                match expr {
                    Expression::Ident(ident) => {
                        ident.id = vars.next_ident_id();
                    }
                    Expression::Let(let_) => {
                        replace_local_id(vars, &mut replacements.borrow_mut(), &mut let_.id)
                    }
                    Expression::For(for_) => replace_local_id(
                        vars,
                        &mut replacements.borrow_mut(),
                        &mut for_.index_variable,
                    ),
                    Expression::Match(match_) => {
                        let mut replacements = replacements.borrow_mut();
                        if let Some(replacement) = replacements.get(&match_.variable_to_match.0) {
                            match_.variable_to_match.0 = *replacement;
                        }
                        for case in match_.cases.iter_mut() {
                            for (arg, _) in case.arguments.iter_mut() {
                                replace_local_id(vars, &mut replacements, arg);
                            }
                        }
                    }
                    _ => (),
                }
                (true, ())
            },
            &mut |_, _| {},
            // Update the IDs in identifiers based on the replacements we remember from above.
            &mut |ident| {
                if let Definition::Local(id) = &mut ident.definition {
                    if let Some(replacement) = replacements.borrow().get(id) {
                        *id = *replacement;
                    }
                }
            },
        );
    }
}

#[cfg(test)]
mod tests {
    /// ```ignore
    /// NOIR_AST_FUZZER_SEED=0xb2fb5f0b00100000 \
    /// cargo test -p noir_ast_fuzzer_fuzz orig_vs_morph
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        crate::targets::tests::fuzz_with_arbtest(super::fuzz, 10000);
    }
}
