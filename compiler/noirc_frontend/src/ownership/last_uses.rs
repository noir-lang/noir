//! This module contains the last use analysis pass which is run on each function before
//! the ownership pass.
//!
//! The purpose of this pass is to find which instance of a variable is the variable's
//! last use. Note that a variable may have multiple last uses. This can happen if the
//! variable's last use is within an `if` expression or similar. It could be last used
//! in one place in the `then` branch and in another place in the `else` branch as long
//! as no code after the `if` expression uses the same variable.
//!
//! This pass works by tracking the last use location for each variable with a
//! "loop index" and a `Branches` enumeration.
//! - The loop index tracks the loop level the variable was declared in. A variable
//!   must be cloned in any loop with an index greater than the variable's own loop index,
//!   except if we are reassigning a variable, which effectively kills the reference to
//!   its previous value, in which case the variable can be moved.
//!   Note that "loop index" refers to how nested we are within loops. A function starts
//!   in index 0, and when we enter the body of a `loop {}` or `for _ in a..b {}`, the index
//!   increments by 1. So `b` in `loop { for _ in 0..1 { b } }` would have index `2`.
//! - The `Branches` enumeration holds each current last use of a variable.
//!   - In the common case this will be `Branches::Direct(ident_id)` indicating that `ident_id`
//!     is the last use of its variable and it was not moved into an `if` or `match`.
//!   - When the variable is used within an `if` or `match` its last use will have a value of
//!     `Branches::IfOrMatch(cases)` with the given nested last uses in each case of the if/match.
//!
//! ## Field-path-aware tracking
//!
//! When a variable is accessed through `ExtractTupleField` (e.g. `x.0.1`), this pass
//! also tracks uses at the **field-path** granularity. Each distinct field path gets its
//! own `Branches` tracking, enabling independent move decisions for disjoint paths.
//! For example, `x.0` and `x.1` can both be moved if neither field is used again after
//! its extraction.
//!
//! When field paths conflict (one is a prefix of another, or a bare use of the variable
//! exists alongside field uses), the per-path tracking is discarded and the pass falls
//! back to the whole-variable `Branches` which tracks the overall last use as before.

use crate::monomorphization::ast::{self, IdentId, LocalId};
use crate::monomorphization::ast::{Expression, Function, Literal};
use iter_extended::vecmap;
use rustc_hash::FxHashMap as HashMap;

use super::Context;

/// A variable's last use may be split into several branches. E.g:
/// ```noir
/// if d.len() == 2 {              // use 1 of d
///     if d.len() == 2 {          // use 2 of d
///         assert_eq(d, [5, 6]);  // use 3 of d
///     }
/// } else {
///     assert_eq(d, [5, 6]);      // use 4 of d
/// }
/// ```
/// d's last uses in the above snippet would be:
/// ```noir
/// Branches::IfOrMatch(vec![
///     Branches::IfOrMatch(vec![
///         Branches::Direct(3),
///         Branches::None,
///     ]),
///     Branches::Direct(4),
/// ])
/// ```
#[derive(Debug, Clone, Default)]
pub(super) enum Branches {
    /// No (last) use in this branch or there is no branch.
    #[default]
    None,
    Direct(IdentId),
    IfOrMatch(IfOrMatchId, HashMap<BranchId, Branches>),
}

impl Branches {
    /// Collect all `IdentId`s from this tree.
    fn flatten_uses(self) -> Vec<IdentId> {
        fn go(branches: Branches, acc: &mut Vec<IdentId>) {
            match branches {
                Branches::None => {}
                Branches::Direct(ident_id) => acc.push(ident_id),
                Branches::IfOrMatch(_, cases) => {
                    for case in cases.into_values() {
                        go(case, acc);
                    }
                }
            }
        }
        let mut acc = Vec::new();
        go(self, &mut acc);
        acc
    }

    fn get_if_or_match_id(&self) -> Option<IfOrMatchId> {
        match self {
            Branches::IfOrMatch(id, _) => Some(*id),
            _ => None,
        }
    }

    fn get_branches_map(&mut self) -> Option<&mut HashMap<BranchId, Branches>> {
        match self {
            Branches::IfOrMatch(_, map) => Some(map),
            _ => None,
        }
    }
}

/// A single path through a `Branches` enum.
///
/// This is used by the context to keep track of where we currently are within a function.
type BranchesPath = Vec<(IfOrMatchId, BranchId)>;

/// The ID of an `if` or `match`, used to distinguish multiple sequential ifs/matches
/// from each other.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(super) struct IfOrMatchId(u32);

/// The ID for a single branch of an if or match.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(super) struct BranchId(u32);

/// A field access path through `ExtractTupleField` chains.
/// For example, `x.0.1` has path `[0, 1]`. A bare use of `x` has an empty path `[]`.
type FieldPath = Vec<usize>;

struct LastUseContext {
    /// The outer `Vec` is each loop we're currently in, while the `BranchPath` contains
    /// the path to overwrite the last use in any `Branches` enums of the variables we find.
    /// As a whole, this tracks the current control-flow of the function we're in.
    current_loop_and_branch: Vec<BranchesPath>,

    /// Next `if` or `match` ID.
    next_id: u32,

    /// Stores the location of each variable's last use considering all access paths.
    ///
    /// Map from each local variable to the last instance of that variable. Separate uses of
    /// the same variable are differentiated by that identifier's `IdentId` which is always
    /// different on separate identifiers, unlike the `LocalId` which is the same for any
    /// identifier referring to the same underlying definition.
    ///
    /// Each definition is mapped to a `PlaceUses` which contains both the overall last use
    /// (considering all field paths as a single use) and per-field-path last uses for
    /// field-level optimization.
    last_uses: HashMap<LocalId, PlaceUses>,

    /// When a variable is overwritten in an assignment, we can treat the last uses so far
    /// as confirmed, because the reassignment essentially kills the reference to the previous
    /// version and redeclares it as new.
    confirmed_moves: HashMap<LocalId, Vec<IdentId>>,
}

/// Tracks last uses of a variable at both whole-variable and per-field-path granularity.
///
/// The `overall` field tracks the last use considering all accesses as uses of the entire
/// variable — this is the same behavior as before field-path tracking was added and serves
/// as the fallback when field paths conflict.
///
/// The `per_path` field tracks last uses independently for each field access path. When all
/// paths are disjoint (no prefix relationships), each path's last use can be moved
/// independently. When paths conflict, the per-path data is discarded and `overall` is used.
struct PlaceUses {
    /// How many loops deep the variable was declared at.
    loop_index: usize,
    /// Last use of the variable considering all accesses as whole-variable uses.
    /// This is the fallback when field paths conflict.
    overall: Branches,
    /// Per-field-path last uses, tracked with full `Branches` support for if/match branching.
    /// Each distinct field path (e.g. `[0, 1]` for `x.0.1`) gets its own `Branches`.
    per_path: HashMap<FieldPath, Branches>,
}

impl PlaceUses {
    /// Returns true if all recorded field paths are pairwise disjoint.
    ///
    /// Two paths are disjoint if they diverge at some index: `[0, 1]` and `[0, 2]`
    /// diverge at position 1. A path that is a prefix of another (`[0]` and `[0, 1]`)
    /// is NOT disjoint. An empty path (bare variable use) is never disjoint from anything.
    fn all_paths_disjoint(&self) -> bool {
        let keys: Vec<_> = self.per_path.keys().collect();
        for (i, a) in keys.iter().enumerate() {
            for b in &keys[i + 1..] {
                if !field_paths_are_disjoint(a, b) {
                    return false;
                }
            }
        }
        true
    }
}

impl Context {
    /// Traverse the given function and return the last use(s) of each local variable.
    /// A variable may have multiple last uses if it was last used within a conditional expression.
    pub(super) fn find_last_uses_of_variables(
        function: &Function,
    ) -> HashMap<LocalId, Vec<IdentId>> {
        let mut context = LastUseContext {
            current_loop_and_branch: Vec::new(),
            last_uses: HashMap::default(),
            confirmed_moves: HashMap::default(),
            next_id: 0,
        };

        context.push_loop_scope();
        for (parameter, ..) in &function.parameters {
            context.declare_variable(*parameter);
        }
        context.track_variables_in_expression(&function.body);
        context.get_variables_to_move()
    }
}

impl LastUseContext {
    fn push_loop_scope(&mut self) {
        self.current_loop_and_branch.push(BranchesPath::new());
    }

    fn pop_loop_scope(&mut self) {
        self.current_loop_and_branch.pop().expect("No loop to pop");
    }

    /// Push a branch to the current loop.
    fn branch(&mut self, id: IfOrMatchId, branch_id: u32) {
        let path =
            self.current_loop_and_branch.last_mut().expect("We should always have at least 1 path");

        path.push((id, BranchId(branch_id)));
    }

    fn next_if_or_match_id(&mut self) -> IfOrMatchId {
        let id = self.next_id;
        self.next_id += 1;
        IfOrMatchId(id)
    }

    /// Pop the latest branch of the current loop.
    fn unbranch(&mut self) {
        let path =
            self.current_loop_and_branch.last_mut().expect("We should always have at least 1 path");
        path.pop().expect("No branch to unbranch");
    }

    /// The current loop index.
    ///
    /// Returns 0 for the outermost layer, which is not in a loop.
    fn loop_index(&self) -> usize {
        self.current_loop_and_branch
            .len()
            .checked_sub(1)
            .expect("We should always have at least 1 path")
    }

    /// Insert the last use of a local variable, defined in the current loop with no branching.
    fn declare_variable(&mut self, id: LocalId) {
        let loop_index = self.loop_index();
        self.last_uses.insert(
            id,
            PlaceUses { loop_index, overall: Branches::None, per_path: HashMap::default() },
        );
    }

    /// Remember a new use of the given variable at a specific field access path.
    ///
    /// Updates both the whole-variable `overall` tracking and the per-field-path tracking.
    /// An empty `field_path` indicates a bare/whole-variable use.
    ///
    /// If the loop index is equal to the variable's when it was defined we can
    /// overwrite the last use, but if it is greater we have to set the last use to None.
    /// This is because variables cannot be moved within loops unless they were defined
    /// within the same loop.
    fn remember_use_of_variable(&mut self, id: LocalId, variable: IdentId, field_path: &FieldPath) {
        let branch_path =
            self.current_loop_and_branch.last().expect("We should always have at least 1 path");
        let loop_index = self.loop_index();

        if let Some(place_uses) = self.last_uses.get_mut(&id) {
            if place_uses.loop_index == loop_index {
                // Update overall (whole-variable) tracking
                Self::remember_use_of_variable_rec(&mut place_uses.overall, branch_path, variable);
                // Update per-field-path tracking
                let path_branches = place_uses.per_path.entry(field_path.clone()).or_default();
                Self::remember_use_of_variable_rec(path_branches, branch_path, variable);
            } else {
                place_uses.overall = Branches::None;
                place_uses.per_path.clear();
            }
        }
    }

    /// Given the `branches` in which a local variable is used, update the last use with the latest
    /// `variable` identifier at the current `path`.
    ///
    /// This is only called when the local variable was created in the current loop, so it never
    /// considers a use in a loop body as a last use for something defined outside the loop.
    fn remember_use_of_variable_rec(
        branches: &mut Branches,
        path: &[(IfOrMatchId, BranchId)],
        variable: IdentId,
    ) {
        // Replace the current last use of a variable with an empty IfOrMatch using the current ID,
        // then recursively add the current branch to it.
        let reset_branch_and_recur = |branch: &mut Branches, if_or_match_id, branch_id| {
            let empty_branch = [(branch_id, Branches::None)].into_iter().collect();
            *branch = Branches::IfOrMatch(if_or_match_id, empty_branch);
            Self::remember_use_of_variable_rec(branch, path, variable);
        };

        match (branches, path) {
            // Path is direct; overwrite the last use entirely.
            (branch, []) => {
                *branch = Branches::Direct(variable);
            }
            // Our path says we need to enter an if or match but the variable's last
            // use was direct. So we need to overwrite the last use with an empty IfOrMatch
            // and write to the relevant branch of that.
            (
                branch @ (Branches::None | Branches::Direct { .. }),
                [(if_or_match_id, branch_id), ..],
            ) => {
                // The branch doesn't exist for this variable; create it.
                reset_branch_and_recur(branch, *if_or_match_id, *branch_id);
            }
            // The variable was last use within a branch. If it's a different branch of the same if/match,
            // we extend it with the new last use, otherwise replace it with the new if/match and branch.
            (branches @ Branches::IfOrMatch(..), [(new_if_id, branch_id), rest @ ..]) => {
                if branches.get_if_or_match_id() == Some(*new_if_id) {
                    let nested = branches.get_branches_map().expect("We know this is a IfOrMatch");
                    let entry = nested.entry(*branch_id).or_insert(Branches::None);
                    Self::remember_use_of_variable_rec(entry, rest, variable);
                } else {
                    reset_branch_and_recur(branches, *new_if_id, *branch_id);
                }
            }
        }
    }

    /// Navigate the `Branches` tree along the given path and extract only
    /// the sub-tree at the leaf, replacing it with `Branches::None`.
    /// Sibling branches are left untouched.
    fn extract_branch_at_path(
        branches: &mut Branches,
        path: &[(IfOrMatchId, BranchId)],
    ) -> Branches {
        match path {
            [] => std::mem::replace(branches, Branches::None),
            [(if_id, branch_id), rest @ ..] => {
                if let Branches::IfOrMatch(id, map) = branches
                    && *id == *if_id
                    && let Some(branch) = map.get_mut(branch_id)
                {
                    return Self::extract_branch_at_path(branch, rest);
                }
                Branches::None
            }
        }
    }

    /// Collect the last use(s) of every local variable.
    ///
    /// For each variable, tries field-path-level decomposition first: if all recorded
    /// field paths are pairwise disjoint (no prefix relationships), each path's last
    /// use(s) can be moved independently. Otherwise, falls back to the whole-variable
    /// `overall` tracking.
    fn get_variables_to_move(self) -> HashMap<LocalId, Vec<IdentId>> {
        let mut moves = self.confirmed_moves;

        for (id, place_uses) in self.last_uses {
            if place_uses.all_paths_disjoint() {
                // All field paths are independent — use per-path moves
                for (_, branches) in place_uses.per_path {
                    moves.entry(id).or_default().extend(branches.flatten_uses());
                }
            } else {
                // Paths conflict — fall back to whole-variable last use
                moves.entry(id).or_default().extend(place_uses.overall.flatten_uses());
            }
        }

        moves
    }

    fn track_variables_in_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Ident(ident) => self.track_variables_in_ident(ident),
            Expression::Literal(literal) => self.track_variables_in_literal(literal),
            Expression::Block(exprs) => {
                exprs.iter().for_each(|expr| self.track_variables_in_expression(expr));
            }
            Expression::Unary(unary) => self.track_variables_in_unary(unary),
            Expression::Binary(binary) => self.track_variables_in_binary(binary),
            Expression::Index(index) => self.track_variables_in_index(index),
            Expression::Cast(cast) => self.track_variables_in_cast(cast),
            Expression::For(for_expr) => self.track_variables_in_for(for_expr),
            Expression::Loop(loop_expr) => self.track_variables_in_loop(loop_expr),
            Expression::While(while_expr) => self.track_variables_in_while(while_expr),
            Expression::If(if_expr) => self.track_variables_in_if(if_expr),
            Expression::Match(match_expr) => self.track_variables_in_match(match_expr),
            Expression::Tuple(elements) => self.track_variables_in_tuple(elements),
            Expression::ExtractTupleField(_, _) => {
                self.track_variables_in_extract(expr);
            }
            Expression::Call(call) => self.track_variables_in_call(call),
            Expression::Let(let_expr) => self.track_variables_in_let(let_expr),
            Expression::Constrain(boolean, _location, msg) => {
                self.track_variables_in_constrain(boolean, msg);
            }
            Expression::Assign(assign) => self.track_variables_in_assign(assign),
            Expression::Semi(expr) => self.track_variables_in_expression(expr),
            Expression::Clone(_) => unreachable!("last_uses is called before clones are inserted"),
            Expression::Drop(_) => unreachable!("last_uses is called before drops are inserted"),
            Expression::Break => (),
            Expression::Continue => (),
        }
    }

    fn track_variables_in_ident(&mut self, ident: &ast::Ident) {
        // We only track last uses for local variables, globals are always cloned
        if let ast::Definition::Local(local_id) = &ident.definition {
            self.remember_use_of_variable(*local_id, ident.id, &vec![]);
        }
    }

    /// Track a variable accessed through one or more `ExtractTupleField` operations.
    ///
    /// Walks the chain to extract the full field path (e.g. `x.0.1` → `[0, 1]`)
    /// and records the use at that specific path for field-level optimization.
    fn track_variables_in_extract(&mut self, expr: &Expression) {
        let mut path = Vec::new();
        let mut current = expr;

        while let Expression::ExtractTupleField(inner, index) = current {
            path.push(*index);
            current = inner;
        }
        path.reverse();

        if let Expression::Ident(ident) = current
            && let ast::Definition::Local(local_id) = &ident.definition
        {
            self.remember_use_of_variable(*local_id, ident.id, &path);
            return;
        }

        // Base is not a local ident (e.g. function call result) — track normally
        self.track_variables_in_expression(current);
    }

    fn track_variables_in_literal(&mut self, literal: &Literal) {
        match literal {
            Literal::Integer(..) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => (),

            Literal::FmtStr(_, _, captures) => self.track_variables_in_expression(captures),

            Literal::Array(array) | Literal::Vector(array) => {
                for element in &array.contents {
                    self.track_variables_in_expression(element);
                }
            }

            Literal::Repeated { element, .. } => self.track_variables_in_expression(element),
        }
    }

    fn track_variables_in_unary(&mut self, unary: &ast::Unary) {
        self.track_variables_in_expression(&unary.rhs);
    }

    fn track_variables_in_binary(&mut self, binary: &ast::Binary) {
        self.track_variables_in_expression(&binary.lhs);
        self.track_variables_in_expression(&binary.rhs);
    }

    fn track_variables_in_index(&mut self, index: &ast::Index) {
        self.track_variables_in_expression(&index.collection);
        self.track_variables_in_expression(&index.index);
    }

    fn track_variables_in_cast(&mut self, cast: &ast::Cast) {
        self.track_variables_in_expression(&cast.lhs);
    }

    fn track_variables_in_for(&mut self, for_expr: &ast::For) {
        // The start and end range are evaluated once before the loop begins,
        // so they are tracked outside the loop scope.
        self.track_variables_in_expression(&for_expr.start_range);
        self.track_variables_in_expression(&for_expr.end_range);
        self.track_variables_in_loop_exprs(&[&for_expr.block]);
    }

    fn track_variables_in_while(&mut self, while_expr: &ast::While) {
        // The condition is evaluated on every iteration of the loop and thus must be included in the loop scope.
        self.track_variables_in_loop_exprs(&[&while_expr.condition, &while_expr.body]);
    }

    fn track_variables_in_loop(&mut self, loop_body: &Expression) {
        self.track_variables_in_loop_exprs(&[loop_body]);
    }

    /// Track variables in a loop body. Each expression in `loop_exprs` must be
    /// an expression that is evaluated on each iteration of the loop.
    fn track_variables_in_loop_exprs(&mut self, loop_exprs: &[&Expression]) {
        // Save the current loop index of the variables we are tracking.
        // They *might* be reassigned inside the loop, which would change their index, but we need to restore them after.
        let orig_indices = vecmap(&self.last_uses, |(id, place_uses)| (*id, place_uses.loop_index));

        self.push_loop_scope();
        for expr in loop_exprs {
            self.track_variables_in_expression(expr);
        }
        self.pop_loop_scope();

        for (id, orig_index) in orig_indices {
            if let Some(place_uses) = self.last_uses.get_mut(&id)
                && place_uses.loop_index != orig_index
            {
                place_uses.loop_index = orig_index;
                // If the value is still accessible outside the loop, don't move it inside the loop after it has been redeclared.
                place_uses.overall = Branches::None;
                place_uses.per_path.clear();
            }
        }
    }

    fn track_variables_in_if(&mut self, if_expr: &ast::If) {
        self.track_variables_in_expression(&if_expr.condition);

        let if_id = self.next_if_or_match_id();
        self.branch(if_id, 0);
        self.track_variables_in_expression(&if_expr.consequence);
        self.unbranch();

        if let Some(alt) = &if_expr.alternative {
            self.branch(if_id, 1);
            self.track_variables_in_expression(alt);
            self.unbranch();
        }
    }

    fn track_variables_in_match(&mut self, match_expr: &ast::Match) {
        // Note: We don't track `variable_to_match` as a use here because it's just a LocalId
        // that references a variable defined earlier. The last-use analysis for that variable
        // happens at its actual use sites (when it's assigned and when it's used after the match).
        // The match expression itself only destructures the value and binds new variables
        // (the case arguments), which we do track below.
        let match_id = self.next_if_or_match_id();

        for (i, case) in match_expr.cases.iter().enumerate() {
            for (argument, _) in &case.arguments {
                self.declare_variable(*argument);
            }

            self.branch(match_id, i as u32);
            self.track_variables_in_expression(&case.branch);
            self.unbranch();
        }

        if let Some(default_case) = &match_expr.default_case {
            self.branch(match_id, match_expr.cases.len() as u32);
            self.track_variables_in_expression(default_case);
            self.unbranch();
        }
    }

    fn track_variables_in_tuple(&mut self, elements: &[Expression]) {
        for elem in elements {
            self.track_variables_in_expression(elem);
        }
    }

    fn track_variables_in_call(&mut self, call: &ast::Call) {
        self.track_variables_in_expression(&call.func);
        for arg in &call.arguments {
            self.track_variables_in_expression(arg);
        }
    }

    fn track_variables_in_let(&mut self, let_expr: &ast::Let) {
        self.track_variables_in_expression(&let_expr.expression);
        self.declare_variable(let_expr.id);
    }

    fn track_variables_in_constrain(
        &mut self,
        boolean: &Expression,
        msg: &Option<Box<(Expression, crate::hir_def::types::Type)>>,
    ) {
        self.track_variables_in_expression(boolean);

        if let Some(msg) = msg {
            self.track_variables_in_expression(&msg.0);
        }
    }

    fn track_variables_in_assign(&mut self, assign: &ast::Assign) {
        // See if we are reassigning a variable, killing the reference to its previous value.
        // (Since we have a separate `Let` to declare variables, any `Assign` to an `Ident` is a reassignment).
        // Only considering simple variables here, not member access or indexing; those would require a more
        // careful analysis and potentially a different algorithm.
        let variable = match &assign.lvalue {
            ast::LValue::Ident(ast::Ident {
                definition: ast::Definition::Local(local_id), ..
            }) => Some(*local_id),
            _ => None,
        };

        if let Some(local_id) = &variable {
            // Adjust its loop index to be the current loop, so that `remember_use_of_variable`
            // remembers any last use, rather than clear out its current state.
            let current_index = self.loop_index();
            if let Some(place_uses) = self.last_uses.get_mut(local_id) {
                place_uses.loop_index = current_index;
            }
        }

        self.track_variables_in_expression(&assign.expression);

        if let Some(local_id) = variable {
            // Confirm any last uses we have on the variable at this point (which may be in `assign.expression`),
            // From here on it acts as a newly declared variable with no history.
            if let Some(place_uses) = self.last_uses.get_mut(&local_id) {
                let branch_path = self
                    .current_loop_and_branch
                    .last()
                    .expect("We should always have at least 1 path");
                let extracted = Self::extract_branch_at_path(&mut place_uses.overall, branch_path);
                self.confirmed_moves.entry(local_id).or_default().extend(extracted.flatten_uses());
                // Also extract and confirm per-path uses at the current branch only
                for branches in place_uses.per_path.values_mut() {
                    let extracted = Self::extract_branch_at_path(branches, branch_path);
                    self.confirmed_moves
                        .entry(local_id)
                        .or_default()
                        .extend(extracted.flatten_uses());
                }
                return;
            }
        }
        self.track_variables_in_lvalue(&assign.lvalue, false /* nested */);
    }

    /// A variable in an lvalue position is never moved (otherwise you wouldn't
    /// be able to access the variable you assigned to afterward). However, the
    /// index in an array expression `a[i] = ...` is an arbitrary expression that
    /// is actually in an rvalue position and can thus be moved.
    ///
    /// Subtle point: if the last use of an identifier was just before it is assigned,
    /// it can actually be moved before it is assigned. This should be fine for top level
    /// identifiers, because we move out of the binding, and the binding isn't used until
    /// it is set to a new value. However, for identifiers used in indexing or member
    /// access, we don't want the identifier to be moved before assignment, as we still
    /// need access to the "rest" of it without potential modification, which brings us
    /// to the `nested` parameter.
    ///
    /// The `nested` parameter indicates whether this lvalue is nested inside another lvalue.
    /// For top-level identifiers there's nothing to track, but for an identifier happening
    /// as part of an index (`ident[index] = ...`) we do want to consider `ident` as used,
    /// which should preclude any previous last uses that could result in it being moved.
    fn track_variables_in_lvalue(&mut self, lvalue: &ast::LValue, nested: bool) {
        match lvalue {
            // All identifiers in lvalues are implicitly `&mut ident` and thus aren't moved
            ast::LValue::Ident(ident) => {
                if nested {
                    self.track_variables_in_ident(ident);
                }
            }
            ast::LValue::Index { array, index, element_type: _, location: _ } => {
                self.track_variables_in_expression(index);
                self.track_variables_in_lvalue(array, true);
            }
            ast::LValue::MemberAccess { object, field_index: _ } => {
                self.track_variables_in_lvalue(object, true);
            }
            ast::LValue::Dereference { reference, element_type: _ } => {
                self.track_variables_in_lvalue(reference, true);
            }
            // LValue::Clone is only inserted by the ownership pass, which runs after last-use analysis
            ast::LValue::Clone(_) => {
                unreachable!("LValue::Clone should only be inserted by the ownership pass")
            }
        }
    }
}

/// Two field paths are disjoint if they diverge at some position.
///
/// An empty path (whole-variable use) is never disjoint from anything.
/// A path that is a prefix of another is not disjoint.
fn field_paths_are_disjoint(a: &[usize], b: &[usize]) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    for (x, y) in a.iter().zip(b.iter()) {
        if x != y {
            return true;
        }
    }
    // One is a prefix of the other (or they're equal)
    false
}
