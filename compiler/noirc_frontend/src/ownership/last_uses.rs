//! This module contains the last use analysis pass which is run on each function before
//! the ownership pass.
//!
//! The purpose of this pass is to find which instance of a variable is the variable's
//! last use. Note that a variable may have multiple last uses. This can happen if the
//! variable's last use is within an `if` expression or similar. It could be last used
//! in one place in the `then` branch and in another place in the `else` branch as long
//! as no code after the `if` expression uses the same variable.
//!
//! This pass works by traversing the expression tree in **reverse** order. The first
//! encounter of each variable (in reverse) is its last use. For if/match branches,
//! each branch is processed independently (starting from a shared "seen" set), so a
//! variable can accumulate multiple last uses — one per branch.
//!
//! Loop handling: variables declared outside a loop cannot be moved inside it (the loop
//! may execute multiple times). When leaving a loop, any pending last uses for variables
//! declared outside the loop are removed.
//!
//! Assignment handling: when `x = expr` is encountered (in reverse), `x` is removed from
//! the "seen" set so that code before the assignment can independently identify the last
//! use of the old value of `x`.
//!
//! ## Field-path-aware optimization
//!
//! After the main reverse-order analysis, a separate post-processing pass scans the
//! function body (in reverse) to find variables accessed exclusively through
//! `ExtractTupleField` with disjoint field paths (e.g. `x.0` and `x.1`). For these
//! variables, all field-path uses are marked as moves, since disjoint paths cannot alias.
//! This is kept as a separate pass for clarity — the traversal is fast.

use crate::ast::UnaryOp;
use crate::monomorphization::ast::{self, Definition, IdentId, LocalId};
use crate::monomorphization::ast::{Expression, Function, Literal};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::Context;

struct LastUseContext {
    /// Variables already encountered in the reverse traversal.
    /// The first encounter of a variable (in reverse) is its last use in forward order.
    seen: HashSet<LocalId>,

    /// Accumulated last uses for each variable. May be truncated on loop exit
    /// for variables declared outside the loop.
    pending_last_uses: HashMap<LocalId, Vec<IdentId>>,

    /// Last uses confirmed by an assignment (`x = f(x)`). The use of `x` in `f(x)` is
    /// confirmed because the assignment kills the old value — these survive loop truncation.
    confirmed_moves: HashMap<LocalId, Vec<IdentId>>,

    /// Loop depth at which each variable was declared.
    /// 0 = function body (not in any loop).
    declaration_depth: HashMap<LocalId, usize>,

    /// Current loop nesting depth.
    loop_depth: usize,

    /// Variables that have been aliased via a reference expression (`&var` or `&mut var`).
    ///
    /// When a variable is referenced, any subsequent copy of it must be a clone (not a move).
    /// This is because the reference creates an invisible alias: if the variable were moved
    /// (sharing the same array pointer with refcount=1), a later write through the reference
    /// would mutate the "moved" copy in place (bypassing copy-on-write semantics), since the
    /// refcount would be 1 and no COW would be triggered.
    ///
    /// By preventing moves of aliased variables, we ensure that subsequent copies increment
    /// the refcount, so that writes through the reference correctly trigger COW.
    referenced_variables: HashSet<LocalId>,
}

impl Context {
    /// Traverse the given function and return the last use(s) of each local variable.
    /// A variable may have multiple last uses if it was last used within a conditional expression.
    pub(super) fn find_last_uses_of_variables(
        function: &Function,
    ) -> HashMap<LocalId, Vec<IdentId>> {
        let mut context = LastUseContext {
            seen: HashSet::default(),
            pending_last_uses: HashMap::default(),
            confirmed_moves: HashMap::default(),
            declaration_depth: HashMap::default(),
            loop_depth: 0,
            referenced_variables: HashSet::default(),
        };

        for (parameter, ..) in &function.parameters {
            context.declare_variable(*parameter);
        }
        context.find_last_uses_in_expression(&function.body);
        let mut moves = context.into_variables_to_move();

        // Post-process: find additional moves for disjoint field-path accesses.
        // Kept as a separate pass for clarity — the traversal is fast.
        add_field_disjoint_moves(&function.body, &mut moves);

        moves
    }
}

impl LastUseContext {
    fn declare_variable(&mut self, id: LocalId) {
        self.declaration_depth.insert(id, self.loop_depth);
    }

    /// Record a use of a local variable. If this is the first encounter in the
    /// reverse traversal (i.e. the last use in forward order), add it to pending_last_uses.
    fn use_variable(&mut self, id: LocalId, ident_id: IdentId) {
        if self.seen.insert(id) {
            self.pending_last_uses.entry(id).or_default().push(ident_id);
        }
    }

    /// Collect all last uses, excluding variables that have been aliased via references.
    fn into_variables_to_move(self) -> HashMap<LocalId, Vec<IdentId>> {
        let mut moves = self.confirmed_moves;
        for (id, uses) in self.pending_last_uses {
            if !uses.is_empty() {
                moves.entry(id).or_default().extend(uses);
            }
        }
        moves.retain(|id, _| !self.referenced_variables.contains(id));
        moves
    }

    // --- Expression traversal (reverse order) ---
    //
    // Sub-expressions are processed in reverse of their forward evaluation order
    // so that the first encounter in our traversal corresponds to the last use.

    fn find_last_uses_in_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Ident(ident) => self.find_last_uses_in_ident(ident),
            Expression::Literal(literal) => self.find_last_uses_in_literal(literal),
            Expression::Block(exprs) => {
                for expr in exprs.iter().rev() {
                    self.find_last_uses_in_expression(expr);
                }
            }
            Expression::Unary(unary) => self.find_last_uses_in_unary(unary),
            Expression::Binary(binary) => {
                self.find_last_uses_in_expression(&binary.rhs);
                self.find_last_uses_in_expression(&binary.lhs);
            }
            Expression::Index(index) => {
                // SSA codegen evaluates the index before the collection, so in forward
                // order the collection is used *after* the index. Reverse that here: visit
                // the collection first so it (not the index) becomes the last use.
                self.find_last_uses_in_expression(&index.collection);
                self.find_last_uses_in_expression(&index.index);
            }
            Expression::Cast(cast) => self.find_last_uses_in_expression(&cast.lhs),
            Expression::For(for_expr) => self.find_last_uses_in_for(for_expr),
            Expression::Loop(body) => self.find_last_uses_in_loop_body(&[body]),
            Expression::While(while_expr) => {
                // Both condition and body are re-evaluated each iteration
                self.find_last_uses_in_loop_body(&[&while_expr.body, &while_expr.condition]);
            }
            Expression::If(if_expr) => self.find_last_uses_in_if(if_expr),
            Expression::Match(match_expr) => self.find_last_uses_in_match(match_expr),
            Expression::Tuple(elements) => {
                for elem in elements.iter().rev() {
                    self.find_last_uses_in_expression(elem);
                }
            }
            Expression::ExtractTupleField(tuple, _) => {
                self.find_last_uses_in_expression(tuple);
            }
            Expression::Call(call) => self.find_last_uses_in_call(call),
            Expression::Let(let_expr) => self.find_last_uses_in_let(let_expr),
            Expression::Constrain(boolean, _, msg) => {
                if let Some(msg) = msg {
                    self.find_last_uses_in_expression(&msg.0);
                }
                self.find_last_uses_in_expression(boolean);
            }
            Expression::Assign(assign) => self.find_last_uses_in_assign(assign),
            Expression::Semi(expr) => self.find_last_uses_in_expression(expr),
            Expression::Clone(_) => unreachable!("last_uses is called before clones are inserted"),
            Expression::Drop(_) => unreachable!("last_uses is called before drops are inserted"),
            Expression::Break | Expression::Continue => (),
        }
    }

    fn find_last_uses_in_ident(&mut self, ident: &ast::Ident) {
        if let Definition::Local(local_id) = &ident.definition {
            self.use_variable(*local_id, ident.id);
        }
    }

    fn find_last_uses_in_literal(&mut self, literal: &Literal) {
        match literal {
            Literal::Integer(..) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => (),
            Literal::FmtStr(_, _, captures) => self.find_last_uses_in_expression(captures),
            Literal::Array(array) | Literal::Vector(array) => {
                for element in array.contents.iter().rev() {
                    self.find_last_uses_in_expression(element);
                }
            }
            Literal::Repeated { element, .. } => self.find_last_uses_in_expression(element),
        }
    }

    fn find_last_uses_in_unary(&mut self, unary: &ast::Unary) {
        if matches!(unary.operator, UnaryOp::Reference { .. }) {
            // When a reference is taken to a local variable or one of its fields (e.g. `&mut x`
            // or `&mut x.field`), the variable `x` is now aliased. Mark it so that any future
            // copy of `x` must clone rather than move.
            if let Some(local_id) = base_ident_of_field_access(&unary.rhs) {
                self.referenced_variables.insert(local_id);
            }
        }
        self.find_last_uses_in_expression(&unary.rhs);
    }

    fn find_last_uses_in_for(&mut self, for_expr: &ast::For) {
        // The body is inside the loop scope; start/end ranges are evaluated once before
        // the loop begins and are tracked outside the loop scope.
        self.find_last_uses_in_loop_body(&[&for_expr.block]);
        self.find_last_uses_in_expression(&for_expr.end_range);
        self.find_last_uses_in_expression(&for_expr.start_range);
    }

    /// Process loop body expressions at an increased loop depth.
    /// After processing, remove any pending last uses for variables declared outside the loop
    /// since they cannot be safely moved inside a loop that may execute multiple times.
    fn find_last_uses_in_loop_body(&mut self, body_exprs: &[&Expression]) {
        let pending_lengths: HashMap<LocalId, usize> =
            self.pending_last_uses.iter().map(|(id, uses)| (*id, uses.len())).collect();

        let loop_body_depth = self.loop_depth + 1;
        self.loop_depth = loop_body_depth;
        for expr in body_exprs.iter().rev() {
            self.find_last_uses_in_expression(expr);
        }
        self.loop_depth = loop_body_depth - 1;

        // Variables declared outside this loop cannot be moved inside it.
        // Truncate their pending uses back to the pre-loop lengths.
        for (id, uses) in &mut self.pending_last_uses {
            let decl_depth = self.declaration_depth.get(id).copied().unwrap_or(0);
            if decl_depth < loop_body_depth {
                let before_len = pending_lengths.get(id).copied().unwrap_or(0);
                uses.truncate(before_len);
            }
        }
    }

    fn find_last_uses_in_if(&mut self, if_expr: &ast::If) {
        let saved_seen = self.seen.clone();

        // Process the then-branch
        self.find_last_uses_in_expression(&if_expr.consequence);
        let mut merged = std::mem::replace(&mut self.seen, saved_seen.clone());

        // Process the else-branch, or use saved_seen for the implicit "do nothing" path
        if let Some(alt) = &if_expr.alternative {
            self.find_last_uses_in_expression(alt);
            merged.extend(&self.seen);
        } else {
            merged.extend(&saved_seen);
        }

        self.seen = merged;

        // The condition is evaluated before either branch
        self.find_last_uses_in_expression(&if_expr.condition);
    }

    fn find_last_uses_in_match(&mut self, match_expr: &ast::Match) {
        // Note: We don't track `variable_to_match` as a use here because it's just a LocalId
        // that references a variable defined earlier. The last-use analysis for that variable
        // happens at its actual use sites.
        let saved_seen = self.seen.clone();
        let mut merged = HashSet::default();

        for case in &match_expr.cases {
            self.seen = saved_seen.clone();
            for (argument, _) in &case.arguments {
                self.declare_variable(*argument);
            }
            self.find_last_uses_in_expression(&case.branch);
            merged.extend(&self.seen);
        }

        if let Some(default_case) = &match_expr.default_case {
            self.seen = saved_seen;
            self.find_last_uses_in_expression(default_case);
            merged.extend(&self.seen);
        } else {
            // No default case: conservatively include saved_seen
            merged.extend(&saved_seen);
        }

        self.seen = merged;
    }

    fn find_last_uses_in_call(&mut self, call: &ast::Call) {
        // A reference passed directly as a call argument (e.g. `foo(&mut x)`) is temporary:
        // it only lives for the duration of the call. After the call returns, `x` is no longer
        // aliased, so future copies of `x` don't need to clone.
        //
        // We must fall back to conservative (mark `x` as aliased) if the reference could escape:
        // 1. The call returns a reference type — the passed reference might be returned.
        // 2. Another argument has type `&mut T` where `T` contains a reference — the function
        //    could write the passed reference into `*that_arg`, making it escape without returning.
        let conservative = type_contains_reference(&call.return_type)
            || call.arguments.iter().any(arg_can_store_reference);

        for arg in call.arguments.iter().rev() {
            if !conservative
                && let Expression::Unary(unary) = arg
                && matches!(unary.operator, UnaryOp::Reference { .. })
                && base_ident_of_field_access(&unary.rhs).is_some()
            {
                // Track the use of the variable inside the reference (for last-use analysis)
                // but skip the unary handler, which would mark the variable as aliased.
                self.find_last_uses_in_expression(&unary.rhs);
            } else {
                self.find_last_uses_in_expression(arg);
            }
        }
        self.find_last_uses_in_expression(&call.func);
    }

    fn find_last_uses_in_let(&mut self, let_expr: &ast::Let) {
        self.declare_variable(let_expr.id);
        self.find_last_uses_in_expression(&let_expr.expression);
    }

    fn find_last_uses_in_assign(&mut self, assign: &ast::Assign) {
        // See if we are reassigning a variable, killing the reference to its previous value.
        // Remove it from `seen` so that earlier code can independently identify the last use
        // of the old value.
        if let ast::LValue::Ident(ast::Ident { definition: Definition::Local(local_id), .. }) =
            &assign.lvalue
        {
            self.seen.remove(local_id);
            let pending_before = self.pending_last_uses.get(local_id).map_or(0, |v| v.len());

            self.find_last_uses_in_expression(&assign.expression);

            // Any uses of the variable added while processing the RHS (e.g. `x = f(x)`)
            // are confirmed as last uses of the old value being killed by this assignment.
            // Confirmed moves survive loop-exit truncation.
            if let Some(uses) = self.pending_last_uses.get_mut(local_id)
                && uses.len() > pending_before
            {
                let confirmed: Vec<_> = uses.drain(pending_before..).collect();
                self.confirmed_moves.entry(*local_id).or_default().extend(confirmed);
            }
            return;
        }

        // For compound lvalues (e.g. `a[i] = expr`), process in reverse evaluation order:
        // the lvalue is accessed after the RHS is evaluated.
        self.find_last_uses_in_lvalue(&assign.lvalue, false);
        self.find_last_uses_in_expression(&assign.expression);
    }

    /// A variable in an lvalue position is never moved (otherwise you wouldn't
    /// be able to access the variable you assigned to afterward). However, the
    /// index in an array expression `a[i] = ...` is an arbitrary expression that
    /// is actually in an rvalue position and can thus be moved.
    ///
    /// The `nested` parameter indicates whether this lvalue is nested inside another lvalue.
    /// For top-level identifiers there's nothing to track, but for an identifier happening
    /// as part of an index (`ident[index] = ...`) we do want to consider `ident` as used,
    /// which should preclude any previous last uses that could result in it being moved.
    fn find_last_uses_in_lvalue(&mut self, lvalue: &ast::LValue, nested: bool) {
        match lvalue {
            ast::LValue::Ident(ident) => {
                if nested {
                    self.find_last_uses_in_ident(ident);
                }
            }
            ast::LValue::Index { array, index, .. } => {
                // As in the rvalue Index case, SSA codegen evaluates the index before
                // touching the array in an lvalue position, so visit the array first in
                // the reverse traversal to make it the last use.
                self.find_last_uses_in_lvalue(array, true);
                self.find_last_uses_in_expression(index);
            }
            ast::LValue::MemberAccess { object, .. } => {
                self.find_last_uses_in_lvalue(object, true);
            }
            ast::LValue::Dereference { reference, .. } => {
                self.find_last_uses_in_lvalue(reference, true);
            }
            ast::LValue::Clone(_) => {
                unreachable!("LValue::Clone should only be inserted by the ownership pass")
            }
        }
    }
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
        Type::Reference(inner, true /* mutable */) => type_contains_reference(inner),
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

/// Returns `true` if the type contains a `Reference` anywhere (directly or nested within
/// tuples, arrays, or function types). Used to decide whether a call might return a
/// reference that aliases a variable passed by `&mut` to that call.
fn type_contains_reference(typ: &ast::Type) -> bool {
    use ast::Type;
    match typ {
        Type::Reference(..) => true,
        Type::Tuple(elements) => elements.iter().any(type_contains_reference),
        Type::Array(_, elem) | Type::Vector(elem) | Type::FmtString(_, elem) => {
            type_contains_reference(elem)
        }
        Type::Function(args, ret, env, _) => {
            args.iter().any(type_contains_reference)
                || type_contains_reference(ret)
                || type_contains_reference(env)
        }
        Type::Field | Type::Integer(..) | Type::Bool | Type::String(..) | Type::Unit => false,
    }
}

// --- Field-path disjoint move optimization ---
//
// Post-processing pass that identifies additional moves for variables accessed
// exclusively through disjoint `ExtractTupleField` paths.

/// A field access path through `ExtractTupleField` chains.
/// For example, `x.0.1` has path `[0, 1]`. A bare use of `x` has an empty path `[]`.
type FieldPath = Vec<usize>;

/// Post-processing pass that identifies additional moves for variables accessed
/// exclusively through disjoint `ExtractTupleField` paths.
///
/// This walks the expression tree in **reverse** order (consistent with the main
/// last-use analysis), collecting `(IdentId, field_path)` for each variable use.
/// Because we traverse in reverse, the first encounter of each field path is the
/// last use in forward order. For each non-move field-path use that is disjoint
/// from all earlier-encountered (i.e. later in forward order) paths of the same
/// variable, it adds the use to the move set.
fn add_field_disjoint_moves(body: &Expression, moves: &mut HashMap<LocalId, Vec<IdentId>>) {
    // Collect all uses per variable in reverse order: (IdentId, field_path)
    let mut use_records: HashMap<LocalId, Vec<(IdentId, FieldPath)>> = HashMap::default();
    collect_use_records_rev(body, &mut use_records);

    // Moves identified by the main last-use analysis. Since IdentIds are unique per use-site,
    // a move added for one variable can never appear in another variable's records, so we can
    // safely precompute this set once rather than rebuilding it per variable.
    let prior_moves: HashSet<IdentId> = moves.values().flat_map(|v| v.iter().copied()).collect();

    let mut new_moves: Vec<(LocalId, IdentId)> = Vec::new();

    for (local_id, records) in &use_records {
        // Records are in reverse order. Walk forward through them (which is reverse program
        // order), maintaining the set of unique field paths seen so far (i.e. paths that
        // appear later in program order). Each candidate is checked against at most U unique
        // paths, where U is bounded by the struct/tuple arity — typically very small.
        let mut later_paths: Vec<&FieldPath> = Vec::new();

        for (ident_id, path) in records {
            if path.is_empty() {
                // Bare use — no earlier use (in forward order) can be disjoint from this.
                break;
            }

            // Check if this use can be moved (disjoint from all later field paths).
            // Skip uses already marked as moves — but still track their paths below,
            // since later candidates need to check disjointness against them.
            if !prior_moves.contains(ident_id)
                && later_paths.iter().all(|later| field_paths_are_disjoint(path, later))
            {
                new_moves.push((*local_id, *ident_id));
            }

            if !later_paths.contains(&path) {
                later_paths.push(path);
            }
        }
    }

    for (local_id, ident_id) in new_moves {
        moves.entry(local_id).or_default().push(ident_id);
    }
}

/// Walk the expression tree in **reverse** order (matching the main last-use traversal),
/// recording each variable use with its `ExtractTupleField` access path (empty for bare idents).
fn collect_use_records_rev(
    expr: &Expression,
    records: &mut HashMap<LocalId, Vec<(IdentId, FieldPath)>>,
) {
    match expr {
        Expression::Ident(ident) => {
            if let Definition::Local(local_id) = &ident.definition {
                records.entry(*local_id).or_default().push((ident.id, vec![]));
            }
        }
        Expression::ExtractTupleField(_, _) => {
            // Walk the ExtractTupleField chain to extract the full path
            let mut path = Vec::new();
            let mut current = expr;
            while let Expression::ExtractTupleField(inner, index) = current {
                path.push(*index);
                current = inner;
            }
            path.reverse();

            if let Expression::Ident(ident) = current
                && let Definition::Local(local_id) = &ident.definition
            {
                records.entry(*local_id).or_default().push((ident.id, path));
                return;
            }
            // Base is not a local ident — recurse into it
            collect_use_records_rev(current, records);
        }
        Expression::Literal(literal) => match literal {
            Literal::FmtStr(_, _, captures) => collect_use_records_rev(captures, records),
            Literal::Array(array) | Literal::Vector(array) => {
                for element in array.contents.iter().rev() {
                    collect_use_records_rev(element, records);
                }
            }
            Literal::Repeated { element, .. } => collect_use_records_rev(element, records),
            _ => {}
        },
        Expression::Block(exprs) => {
            for e in exprs.iter().rev() {
                collect_use_records_rev(e, records);
            }
        }
        Expression::Unary(unary) => collect_use_records_rev(&unary.rhs, records),
        Expression::Binary(binary) => {
            collect_use_records_rev(&binary.rhs, records);
            collect_use_records_rev(&binary.lhs, records);
        }
        Expression::Index(index) => {
            collect_use_records_rev(&index.collection, records);
            collect_use_records_rev(&index.index, records);
        }
        Expression::Cast(cast) => collect_use_records_rev(&cast.lhs, records),
        Expression::For(for_expr) => {
            collect_use_records_rev(&for_expr.block, records);
            collect_use_records_rev(&for_expr.end_range, records);
            collect_use_records_rev(&for_expr.start_range, records);
        }
        Expression::Loop(body) => collect_use_records_rev(body, records),
        Expression::While(while_expr) => {
            collect_use_records_rev(&while_expr.body, records);
            collect_use_records_rev(&while_expr.condition, records);
        }
        Expression::If(if_expr) => {
            if let Some(alt) = &if_expr.alternative {
                collect_use_records_rev(alt, records);
            }
            collect_use_records_rev(&if_expr.consequence, records);
            collect_use_records_rev(&if_expr.condition, records);
        }
        Expression::Match(match_expr) => {
            if let Some(default_case) = &match_expr.default_case {
                collect_use_records_rev(default_case, records);
            }
            for case in match_expr.cases.iter().rev() {
                collect_use_records_rev(&case.branch, records);
            }
        }
        Expression::Tuple(elements) => {
            for e in elements.iter().rev() {
                collect_use_records_rev(e, records);
            }
        }
        Expression::Call(call) => {
            for arg in call.arguments.iter().rev() {
                collect_use_records_rev(arg, records);
            }
            collect_use_records_rev(&call.func, records);
        }
        Expression::Let(let_expr) => collect_use_records_rev(&let_expr.expression, records),
        Expression::Constrain(boolean, _, msg) => {
            if let Some(msg) = msg {
                collect_use_records_rev(&msg.0, records);
            }
            collect_use_records_rev(boolean, records);
        }
        Expression::Assign(assign) => {
            collect_use_records_rev(&assign.expression, records);
        }
        Expression::Semi(expr) => collect_use_records_rev(expr, records),
        Expression::Clone(_) | Expression::Drop(_) => {}
        Expression::Break | Expression::Continue => {}
    }
}

/// Two field paths are disjoint if they diverge at some position.
///
/// A path that is a prefix of another is not disjoint.
/// An empty path (bare variable use) is never disjoint from anything.
fn field_paths_are_disjoint(a: &[usize], b: &[usize]) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    for (x, y) in a.iter().zip(b.iter()) {
        if x != y {
            return true;
        }
    }
    false
}
