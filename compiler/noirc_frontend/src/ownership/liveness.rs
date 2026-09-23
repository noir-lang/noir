//! This module contains the liveness analysis which is run on each function before the
//! ownership pass.
//!
//! A use of a variable can be *moved* — handed over with no reference-count bump — exactly
//! when the variable is **dead immediately after that use**: no path onwards from that point
//! reaches another use of the variable before the variable is reassigned. Everywhere else the
//! use must be copied, so that a write through the new owner cannot be observed through the old
//! name. That is backward liveness, and this module computes it directly.
//!
//! `visit` walks the expression tree in reverse, threading the set of variables live *after*
//! an expression and returning those live *before* it. At each identifier the decision is one
//! membership test. Branches join by union; loops iterate to a fixpoint so that the back edge
//! is accounted for, and a loop's exit and jump edges are modelled explicitly:
//!
//! - a `while` and a `for` are both tested at their header, so the header has two successors,
//!   the body and the loop exit, and whatever is live after the loop is live at the header.
//!   This is what makes a read that the body reassigns afterwards a copy rather than a move:
//!   the exit edge reaches the post-loop reader without crossing that reassignment. A bare
//!   `loop` has no header test and leaves only via `break`, so its header carries nothing.
//! - a `break`/`continue` does not fall through: the set live before it is the set live at its
//!   jump target, not the set live after it in the tree. A `break`/`continue` written in a
//!   `while` condition targets the *enclosing* loop (consistent with SSA lowering and the
//!   comptime interpreter), so the condition is visited with this loop's targets popped.
//!
//! Aliasing is a separate question that liveness does not answer: a variable that has had a
//! reference taken to it can be observed through that reference even when it is dead by name,
//! so `referenced_variables` blocks moves of such variables outright.
//!
//! This pass is not sophisticated with regard to struct and tuple fields. It currently
//! ignores these entirely and counts each use as a use of the entire variable. This is an
//! area for future optimization. E.g. the program `a.b.c; a.e.f` will result in `a` being
//! cloned in its entirety in the first statement. Note that this is lessened in the overall
//! ownership pass such that only `.c` is cloned but it is still an area for improvement.

use crate::ast::UnaryOp;
use crate::monomorphization::ast::{self, Definition, IdentId, LocalId};
use crate::monomorphization::ast::{Expression, Function, Literal};
use crate::shared::Builtin;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

/// The set of variables live at a program point.
type Live = HashSet<LocalId>;

struct LivenessContext {
    /// Each instance of a variable that can be moved rather than cloned.
    moves: HashMap<LocalId, Vec<IdentId>>,

    /// Variables that have been aliased via a reference expression (`&var` or `&mut var`).
    ///
    /// A reference creates an invisible alias: if the variable were moved (sharing the same
    /// array pointer with refcount=1), a later write through the reference would mutate the
    /// "moved" copy in place, bypassing copy-on-write. Preventing moves of aliased variables
    /// keeps the refcount above 1 so those writes correctly trigger COW.
    referenced_variables: HashSet<LocalId>,

    /// The set live at the exit of each enclosing loop, innermost last. A `break` jumps here.
    break_live: Vec<Live>,

    /// The set live at the header of each enclosing loop, innermost last. A `continue` jumps here.
    continue_live: Vec<Live>,

    /// Fixpoint iterations run with this off, so only the converged pass records moves.
    recording: bool,

    /// For each immutable `let`, the variables whose buffer its value may be. See
    /// [`BufferSources`].
    let_sources: HashMap<LocalId, BufferSources>,
}

/// Traverse the given function and return each use of a local variable that can be moved.
/// A variable may have several such uses: one per branch of an `if`/`match`, for instance.
pub(super) fn find_variables_to_move(function: &Function) -> HashMap<LocalId, Vec<IdentId>> {
    let mut context = LivenessContext {
        moves: HashMap::default(),
        referenced_variables: HashSet::default(),
        break_live: Vec::new(),
        continue_live: Vec::new(),
        recording: true,
        let_sources: collect_let_sources(&function.body),
    };

    // Nothing is live on return: the function's result has already been consumed by the
    // tail expression itself, and parameters are dead once the body finishes.
    context.visit(&function.body, Live::default());

    let mut moves = context.moves;
    moves.retain(|id, _| !context.referenced_variables.contains(id));
    moves
}

impl LivenessContext {
    /// Returns the set of variables live immediately *before* `expr`, given those live
    /// immediately after it.
    ///
    /// Sub-expressions are visited in reverse of their forward evaluation order, so that the
    /// set threaded into each one is the set live after it actually runs.
    fn visit(&mut self, expr: &Expression, live: Live) -> Live {
        match expr {
            Expression::Ident(ident) => self.visit_ident(ident, live),
            Expression::Literal(literal) => self.visit_literal(literal, live),
            Expression::Block(exprs) => {
                exprs.iter().rev().fold(live, |live, expr| self.visit(expr, live))
            }
            Expression::Unary(unary) => self.visit_unary(unary, live),
            Expression::Binary(binary) => {
                let live = self.visit(&binary.rhs, live);
                self.visit(&binary.lhs, live)
            }
            Expression::Index(index) => {
                // SSA codegen evaluates the index before the collection, so in forward order
                // the collection is used *after* the index.
                let live = self.visit(&index.collection, live);
                self.visit(&index.index, live)
            }
            Expression::Cast(cast) => self.visit(&cast.lhs, live),
            Expression::For(for_expr) => self.visit_for(for_expr, live),
            // A bare `loop` has no header test: it leaves only via `break`, which reads
            // `break_live` directly, so nothing is live at its header on entry.
            Expression::Loop(body) => self.loop_fixpoint(body, None, false, live),
            Expression::While(while_expr) => {
                self.loop_fixpoint(&while_expr.body, Some(&while_expr.condition), true, live)
            }
            Expression::If(if_expr) => self.visit_if(if_expr, live),
            Expression::Match(match_expr) => self.visit_match(match_expr, live),
            Expression::Tuple(elements) => {
                elements.iter().rev().fold(live, |live, elem| self.visit(elem, live))
            }
            Expression::ExtractTupleField(tuple, _) => self.visit(tuple, live),
            Expression::Call(call) => self.visit_call(call, live),
            Expression::Let(let_expr) => {
                // The binding is a definition: the variable is not live before it.
                let mut live = live;
                live.remove(&let_expr.id);
                self.visit(&let_expr.expression, live)
            }
            Expression::Constrain(boolean, _, msg) => {
                let live = match msg {
                    Some(msg) => self.visit(&msg.0, live),
                    None => live,
                };
                self.visit(boolean, live)
            }
            Expression::Assign(assign) => self.visit_assign(assign, live),
            Expression::Semi(expr) => self.visit(expr, live),
            Expression::Clone(_) => unreachable!("liveness is called before clones are inserted"),
            Expression::Drop(_) => unreachable!("liveness is called before drops are inserted"),
            // Control does not fall through a jump, so the incoming set is discarded in
            // favour of the set live at the jump's target.
            Expression::Break => self.break_live.last().cloned().unwrap_or_default(),
            Expression::Continue => self.continue_live.last().cloned().unwrap_or_default(),
        }
    }

    fn visit_ident(&mut self, ident: &ast::Ident, mut live: Live) -> Live {
        if let Definition::Local(local_id) = &ident.definition {
            if self.recording && !live.contains(local_id) {
                self.moves.entry(*local_id).or_default().push(ident.id);
            }
            live.insert(*local_id);
        }
        live
    }

    fn visit_literal(&mut self, literal: &Literal, live: Live) -> Live {
        match literal {
            Literal::Integer(..) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => live,
            Literal::FmtStr(_, _, captures) => self.visit(captures, live),
            Literal::Array(array) | Literal::Vector(array) => {
                array.contents.iter().rev().fold(live, |live, element| self.visit(element, live))
            }
            Literal::Repeated { element, .. } => self.visit(element, live),
        }
    }

    fn visit_unary(&mut self, unary: &ast::Unary, live: Live) -> Live {
        if matches!(unary.operator, UnaryOp::Reference { .. })
            && let Some(local_id) = base_ident_of_field_access(&unary.rhs)
        {
            // Taking a reference to a local variable or one of its fields (e.g. `&mut x` or
            // `&mut x.field`) aliases `x`; no copy of `x` may be a move from here on.
            self.referenced_variables.insert(local_id);
        }
        self.visit(&unary.rhs, live)
    }

    fn visit_if(&mut self, if_expr: &ast::If, live: Live) -> Live {
        // A variable is live before the branch if it is live entering either arm.
        let mut merged = self.visit(&if_expr.consequence, live.clone());
        match &if_expr.alternative {
            Some(alt) => merged.extend(self.visit(alt, live)),
            // The implicit "do nothing" path falls straight through.
            None => merged.extend(live),
        }
        self.visit(&if_expr.condition, merged)
    }

    fn visit_match(&mut self, match_expr: &ast::Match, live: Live) -> Live {
        // Note: `variable_to_match` is not a use here. It is a `LocalId` referring to a
        // variable bound earlier, and the analysis for it happens at its actual use sites.
        let mut merged = Live::default();
        for case in &match_expr.cases {
            let mut case_live = self.visit(&case.branch, live.clone());
            // The case's bindings are defined by the match, so they are not live before it.
            for (argument, _) in &case.arguments {
                case_live.remove(argument);
            }
            merged.extend(case_live);
        }
        match &match_expr.default_case {
            Some(default_case) => merged.extend(self.visit(default_case, live)),
            // With no default case some path skips every arm.
            None => merged.extend(live),
        }
        merged
    }

    fn visit_call(&mut self, call: &ast::Call, live: Live) -> Live {
        // A reference passed directly as a call argument (e.g. `foo(&mut x)`) is temporary:
        // it only lives for the duration of the call. After the call returns, `x` is no longer
        // aliased, so future copies of `x` don't need to clone.
        //
        // We must fall back to conservative (mark `x` as aliased) if the reference could escape:
        // 1. The call returns a reference type — the passed reference might be returned.
        // 2. Another argument has type `&mut T` where `T` contains a reference — the function
        //    could write the passed reference into `*that_arg`, making it escape without returning.
        let conservative = call.return_type.contains_reference()
            || call.arguments.iter().any(arg_can_store_reference);

        let mut live = live;
        for (index, arg) in call.arguments.iter().enumerate().rev() {
            if !conservative
                && let Expression::Unary(unary) = arg
                && matches!(unary.operator, UnaryOp::Reference { .. })
                && let Some(base) = base_ident_of_field_access(&unary.rhs)
                // Even though the reference cannot escape the call, the *other* arguments of
                // this same call are evaluated and handed to the callee while the reference is
                // live. If one of them mentions `x` (e.g. `foo(&mut x, x)`), moving that use
                // would let the callee's writes through the reference be observed through a
                // by-value argument, so `x` must be treated as aliased.
                && !call.arguments.iter().enumerate().any(|(other_index, other_arg)| {
                    other_index != index && local_occurs_in(base, other_arg)
                })
            {
                // Count the use of the variable inside the reference, but skip the unary
                // handler, which would mark the variable as aliased.
                live = self.visit(&unary.rhs, live);
            } else {
                live = self.visit(arg, live);
            }
        }
        self.visit(&call.func, live)
    }

    fn visit_assign(&mut self, assign: &ast::Assign, live: Live) -> Live {
        if let ast::LValue::Ident(ast::Ident { definition: Definition::Local(local_id), .. }) =
            &assign.lvalue
        {
            // Assigning the whole variable ends its old value, so a use of it inside the
            // right-hand side can be the last one.
            let mut live = live;
            live.remove(local_id);
            let mut live = self.visit(&assign.expression, live);

            if carries_another_variable(&self.let_sources, &assign.expression, *local_id) {
                // The right-hand side may be another variable's buffer, handed over by moves
                // alone. Each such move is justified by liveness, since the moved-from name is not
                // read again; but inside a loop a chain of them — `t = a; a = c; c = t` — lowers to
                // loop-header parameters permuted across the back edge with no `inc_rc` among
                // them, and `rc_invariant` reasons about SSA values, not about which names are
                // dead, so it cannot tell that permutation from an alias and rejects the next
                // in-place write. Keeping the variable live for uses that reach here from before
                // the assignment clones the earlier use and puts an `inc_rc` on the chain. Uses
                // inside the right-hand side are unaffected: nothing runs between them and the
                // overwrite except the rest of the right-hand side.
                live.insert(*local_id);
            }

            return live;
        }

        // A compound lvalue (e.g. `a[i] = expr`) reads `a` as well as writing it, so it is
        // not a definition. SSA codegen accesses the lvalue after evaluating the RHS.
        let live = self.visit_lvalue(&assign.lvalue, false, live);
        self.visit(&assign.expression, live)
    }

    /// A variable in an lvalue position is never moved (otherwise you wouldn't be able to
    /// access the variable you assigned to afterward). It is still a use, which keeps earlier
    /// reads from being moved. However, the index in an array expression `a[i] = ...` is an
    /// arbitrary expression that is actually in an rvalue position and can thus be moved.
    ///
    /// The `nested` parameter indicates whether this lvalue is nested inside another lvalue.
    /// A top-level identifier is overwritten rather than read, so it is not a use; an
    /// identifier reached through an index or field (`ident[index] = ...`) is read.
    fn visit_lvalue(&mut self, lvalue: &ast::LValue, nested: bool, live: Live) -> Live {
        match lvalue {
            ast::LValue::Ident(ident) => match (nested, &ident.definition) {
                (true, Definition::Local(local_id)) => {
                    let mut live = live;
                    live.insert(*local_id);
                    live
                }
                _ => live,
            },
            ast::LValue::Index { array, index, .. } => {
                // As in the rvalue Index case, SSA codegen evaluates the index before
                // touching the array in an lvalue position.
                let live = self.visit_lvalue(array, true, live);
                self.visit(index, live)
            }
            ast::LValue::MemberAccess { object, .. } => self.visit_lvalue(object, true, live),
            ast::LValue::Dereference { reference, .. } => self.visit_lvalue(reference, true, live),
            ast::LValue::Clone(_) => {
                unreachable!("LValue::Clone should only be inserted by the ownership pass")
            }
        }
    }

    fn visit_for(&mut self, for_expr: &ast::For, live: Live) -> Live {
        // The ranges are evaluated once, before the loop, so they sit outside the fixpoint.
        // `for` carries no condition *expression*, but `index < end_range` is still tested at
        // the header, so the loop exits from there just as a `while` does.
        let mut header = self.loop_fixpoint(&for_expr.block, None, true, live.clone());
        // The loop header defines the index variable on every iteration.
        header.remove(&for_expr.index_variable);

        // The header's other successor is the loop exit.
        let live = header.union(&live).copied().collect::<Live>();
        let live = self.visit(&for_expr.end_range, live);
        self.visit(&for_expr.start_range, live)
    }

    /// Returns the set live at the loop's header, given the set live after the loop.
    ///
    /// The back edge makes the header's live-in depend on itself, so iterate: the set only
    /// grows, the lattice of variable sets is finite, and in practice two passes suffice.
    /// The iterations run with recording off, since a use recorded against a live set that is
    /// still growing could claim a move the converged answer rejects.
    fn loop_fixpoint(
        &mut self,
        body: &Expression,
        condition: Option<&Expression>,
        exits_from_header: bool,
        live_after: Live,
    ) -> Live {
        let recording = std::mem::replace(&mut self.recording, false);

        // When the loop can exit from its header, the header's successors include the code
        // after the loop, so whatever that code needs is live at the header. This is what keeps
        // a read inside the body live when the body reassigns the variable afterwards: the exit
        // edge reaches the post-loop reader without passing that reassignment again.
        let mut header = if exits_from_header { live_after.clone() } else { Live::default() };
        loop {
            let next = self.visit_loop_once(body, condition, &header, &live_after);
            if next.is_subset(&header) {
                break;
            }
            header.extend(next);
        }

        self.recording = recording;
        self.visit_loop_once(body, condition, &header, &live_after)
    }

    fn visit_loop_once(
        &mut self,
        body: &Expression,
        condition: Option<&Expression>,
        header: &Live,
        live_after: &Live,
    ) -> Live {
        self.break_live.push(live_after.clone());
        self.continue_live.push(header.clone());

        // The body's successor is the back edge to the header.
        let body_in = self.visit(body, header.clone());

        self.continue_live.pop();
        self.break_live.pop();

        match condition {
            // The condition's successors are the body and the loop exit. A `break`/`continue`
            // written in the condition targets the *enclosing* loop, so the condition is
            // visited with this loop's jump targets already popped.
            Some(condition) => {
                let condition_out = body_in.union(live_after).copied().collect::<Live>();
                self.visit(condition, condition_out)
            }
            None => body_in,
        }
    }
}

/// The variables whose buffer an expression may evaluate to by data movement alone: through
/// identifiers, immutable `let`s, tuple fields, casts, identity conversions and block tails.
/// `None` means it may be any existing buffer.
///
/// Calls that remain calls in SSA and allocations end the chain: `rc_invariant` treats their
/// results as fresh storage. Identity conversion builtins simplify to their input SSA value,
/// so they preserve its source and can participate in the loop-header parameter permutation
/// that the guard in `visit_assign` exists to break.
type BufferSources = Option<HashSet<LocalId>>;

fn union_sources(sources: impl IntoIterator<Item = BufferSources>) -> BufferSources {
    let mut all = HashSet::default();
    for source in sources {
        all.extend(source?);
    }
    Some(all)
}

/// Returns the [`BufferSources`] of `expr`, looking immutable `let`s up in `let_sources`.
///
/// A mutable variable or a parameter is its own source: it may be reassigned, so the name is all
/// that is known about its buffer. An index, a dereference or a global may be any existing buffer.
fn buffer_sources(
    let_sources: &HashMap<LocalId, BufferSources>,
    expr: &Expression,
) -> BufferSources {
    let sources = |expr| buffer_sources(let_sources, expr);
    match expr {
        Expression::Ident(ident) => match ident.definition {
            Definition::Local(id) => match let_sources.get(&id) {
                Some(of_let) => of_let.clone(),
                None => Some(HashSet::from_iter([id])),
            },
            _ => None,
        },
        Expression::Literal(literal) => match literal {
            Literal::Array(_)
            | Literal::Vector(_)
            | Literal::Repeated { .. }
            | Literal::Integer(..)
            | Literal::Bool(_)
            | Literal::Unit
            | Literal::Str(_) => Some(HashSet::default()),
            Literal::FmtStr(..) => None,
        },
        Expression::Call(call) => {
            if let Expression::Ident(ident) = call.func.as_ref()
                && matches!(
                    ident.definition,
                    Definition::Builtin(Builtin::StrAsBytes | Builtin::ArrayAsStrUnchecked)
                        | Definition::LowLevel(Builtin::StrAsBytes | Builtin::ArrayAsStrUnchecked)
                )
            {
                // These conversions simplify to their input SSA value, preserving its buffer.
                call.arguments.first().and_then(sources)
            } else {
                Some(HashSet::default())
            }
        }
        Expression::Cast(cast) => sources(&cast.lhs),
        Expression::ExtractTupleField(tuple, _) => sources(tuple),
        Expression::Tuple(elements) => union_sources(elements.iter().map(sources)),
        Expression::Block(exprs) => match exprs.last() {
            Some(tail) => sources(tail),
            None => Some(HashSet::default()),
        },
        Expression::If(if_expr) => union_sources([
            sources(&if_expr.consequence),
            if_expr.alternative.as_ref().map_or(Some(HashSet::default()), |alt| sources(alt)),
        ]),
        Expression::Match(match_expr) => union_sources(
            match_expr
                .cases
                .iter()
                .map(|case| sources(&case.branch))
                .chain(match_expr.default_case.as_ref().map(|default| sources(default))),
        ),
        Expression::Binary(_)
        | Expression::Let(_)
        | Expression::Assign(_)
        | Expression::Constrain(..)
        | Expression::Semi(_)
        | Expression::For(_)
        | Expression::Loop(_)
        | Expression::While(_)
        | Expression::Break
        | Expression::Continue => Some(HashSet::default()),
        Expression::Unary(_)
        | Expression::Index(_)
        | Expression::Clone(_)
        | Expression::Drop(_) => None,
    }
}

/// Records the [`BufferSources`] of every immutable `let` in `body`. Each `let` is recorded after
/// its initializer has been walked, so a `let` nested inside another's initializer is known by
/// the time the outer one is resolved.
fn collect_let_sources(body: &Expression) -> HashMap<LocalId, BufferSources> {
    let mut let_sources = HashMap::default();
    crate::monomorphization::visitor::visit_expr_be(
        body,
        &mut |_| (true, ()),
        &mut |expr, ()| {
            if let Expression::Let(let_expr) = expr
                && !let_expr.mutable
            {
                let sources = buffer_sources(&let_sources, &let_expr.expression);
                let_sources.insert(let_expr.id, sources);
            }
        },
        &mut |_| {},
    );
    let_sources
}

/// Returns `true` if assigning `rhs` to `assigned` may hand it a buffer that some other variable
/// held, by data movement alone.
fn carries_another_variable(
    let_sources: &HashMap<LocalId, BufferSources>,
    rhs: &Expression,
    assigned: LocalId,
) -> bool {
    match buffer_sources(let_sources, rhs) {
        Some(sources) => sources.iter().any(|source| *source != assigned),
        None => true,
    }
}

/// Returns `true` if `local_id` occurs anywhere within `expr`, in value position (`x`, `x[i]`)
/// or as the base of an assignment lvalue (`x[i] = ..`). Monomorphized `LocalId`s are unique,
/// so any occurrence is a free occurrence.
fn local_occurs_in(local_id: LocalId, expr: &Expression) -> bool {
    let mut occurs = false;
    crate::monomorphization::visitor::visit_expr_be(
        expr,
        &mut |_| (true, ()),
        &mut |_, ()| {},
        &mut |ident: &ast::Ident| {
            if let Definition::Local(id) = ident.definition
                && id == local_id
            {
                occurs = true;
            }
        },
    );
    occurs
}

/// Given an expression that is the operand of a reference (`&expr` or `&mut expr`),
/// walk through any chain of struct-field accesses (`expr.field` = `ExtractTupleField`)
/// and return the `LocalId` of the base variable, if it is a local variable.
///
/// For example:
/// - `&mut x`         → `Some(x_id)`
/// - `&mut x.field`   → `Some(x_id)`  (field is `ExtractTupleField(x, _)`)
/// - `&mut x.a.b`     → `Some(x_id)`
/// - `&mut some_call()` → `None`
///
/// Block expressions intentionally return `None`: `&mut { ...; expr }` always allocates
/// fresh storage and copies the tail's value into it, so the operand is no longer a
/// "direct reference" to a local. The clones needed to keep refcounts honest are
/// inserted by the forward pass when it processes the block's tail in normal context
/// (see `handle_reference_expression`).
fn base_ident_of_field_access(expr: &Expression) -> Option<LocalId> {
    match expr {
        Expression::Ident(ident) => {
            if let Definition::Local(local_id) = ident.definition {
                Some(local_id)
            } else {
                None
            }
        }
        Expression::ExtractTupleField(inner, _) => base_ident_of_field_access(inner),
        _ => None,
    }
}

/// Returns `true` if `arg`'s type can be used to store a reference, i.e. the type
/// contains a `&mut T` (at any depth) where `T` itself contains a reference.
///
/// When this is true for any argument, all `&mut x` arguments in the call must conservatively
/// be treated as aliasing `x`, because the callee might write the reference into the location
/// reachable through that argument.
fn arg_can_store_reference(arg: &Expression) -> bool {
    match arg.return_type() {
        Some(typ) => type_can_store_reference(&typ),
        None => true,
    }
}

/// Returns `true` if `typ` contains — at any depth — a `&mut T` where `T` itself contains
/// a reference. Such a type allows a reference to be written somewhere persistent.
fn type_can_store_reference(typ: &ast::Type) -> bool {
    use ast::Type;
    match typ {
        Type::Reference(inner, true /* mutable */) => inner.contains_reference(),
        Type::Reference(inner, false) => type_can_store_reference(inner),
        Type::Tuple(elements) => elements.iter().any(type_can_store_reference),
        Type::Array(_, elem) | Type::Vector(elem) | Type::FmtString(_, elem) => {
            type_can_store_reference(elem)
        }
        Type::Function(args, ret, env, _) => {
            args.iter().any(type_can_store_reference)
                || type_can_store_reference(ret)
                || type_can_store_reference(env)
        }
        Type::Field | Type::Integer(..) | Type::Bool | Type::String(..) | Type::Unit => false,
    }
}
