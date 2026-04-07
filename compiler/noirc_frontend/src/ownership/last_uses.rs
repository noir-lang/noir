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
//! - This pass is not sophisticated with regard to struct and tuple fields. It currently
//!   ignores these entirely and counts each use as a use of the entire variable. This is an
//!   area for future optimization. E.g. the program `a.b.c; a.e.f` will result in `a` being
//!   cloned in its entirety in the first statement. Note that this is lessened in the overall
//!   ownership pass such that only `.c` is cloned but it is still an area for improvement.

use crate::ast::UnaryOp;
use crate::monomorphization::ast::{self, Definition, IdentId, LocalId};
use crate::monomorphization::ast::{Expression, Function, Literal};
use iter_extended::vecmap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

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
#[derive(Debug, Clone)]
pub(super) enum Branches {
    /// No (last) use in this branch or there is no branch.
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

struct LastUseContext {
    /// The outer `Vec` is each loop we're currently in, while the `BranchPath` contains
    /// the path to overwrite the last use in any `Branches` enums of the variables we find.
    /// As a whole, this tracks the current control-flow of the function we're in.
    current_loop_and_branch: Vec<BranchesPath>,

    /// Next `if` or `match` ID.
    next_id: u32,

    /// Stores the location of each variable's last use.
    ///
    /// Map from each local variable to the last instance of that variable. Separate uses of
    /// the same variable are differentiated by that identifier's `IdentId` which is always
    /// different on separate identifiers, unlike the `LocalId` which is the same for any
    /// identifier referring to the same underlying definition.
    ///
    /// Each definition is mapped to a loop index and a Branches enumeration.
    /// - The loop index tracks how many loops deep the variable was declared at. It may be moved
    ///   within the same loop but cannot be moved within a nested loop. E.g:
    ///   ```noir
    ///   fn foo() {
    ///       let x = 2;
    ///       for _ in 0 .. 2 {
    ///           let b = true;
    ///           println((x, b));
    ///       }
    ///   }
    ///   ```
    ///   In the snippet above, `x` will have loop index 0 which does not match its last use
    ///   within the for loop (1 loop deep = loop index of 1). However, `b` has loop index 1
    ///   and thus can be moved into its last use in the loop, in this case the `println` call.
    /// - The Branches enumeration holds each last use of the variable. This is usually only
    ///   one use but can be multiple if the last use is spread across several `if` or `match`
    ///   branches. E.g:
    ///   ```noir
    ///   fn bar() {
    ///       let x = 2;
    ///       if true {
    ///           println(x);
    ///       } else {
    ///           assert(x < 5);
    ///       }
    ///   }
    ///   ```
    ///   `x` above has two last uses, one in each if branch.
    last_uses: HashMap<LocalId, (/*loop index*/ usize, Branches)>,

    /// When a variable is overwritten in an assignment, we can treat the last uses so far
    /// as confirmed, because the reassignment essentially kills the reference to the previous
    /// version and redeclares it as new.
    confirmed_moves: HashMap<LocalId, Vec<IdentId>>,

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
            current_loop_and_branch: Vec::new(),
            last_uses: HashMap::default(),
            confirmed_moves: HashMap::default(),
            referenced_variables: HashSet::default(),
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
        self.last_uses.insert(id, (loop_index, Branches::None));
    }

    /// Remember a new use of the given variable, possibly overwriting or
    /// adding to the previous last use depending on the current position
    /// in if/match branches or the loop index.
    ///
    /// If the loop index is equal to the variable's when it was defined we can
    /// overwrite the last use, but if it is greater we have to set the last use to None.
    /// This is because variable's cannot be moved within loops unless it was defined
    /// within the same loop.
    fn remember_use_of_variable(&mut self, id: LocalId, variable: IdentId) {
        let path =
            self.current_loop_and_branch.last().expect("We should always have at least 1 path");
        let loop_index = self.loop_index();

        if let Some((variable_loop_index, uses)) = self.last_uses.get_mut(&id) {
            if *variable_loop_index == loop_index {
                Self::remember_use_of_variable_rec(uses, path, variable);
            } else {
                *uses = Branches::None;
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
    fn get_variables_to_move(self) -> HashMap<LocalId, Vec<IdentId>> {
        let mut moves = self.confirmed_moves;
        for (id, (_, branches)) in self.last_uses {
            // Variables aliased via references must always be cloned on copy (never moved).
            // See `referenced_variables` for the reasoning.
            if !self.referenced_variables.contains(&id) {
                moves.entry(id).or_default().extend(branches.flatten_uses());
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
            Expression::ExtractTupleField(tuple, _index) => {
                self.track_variables_in_expression(tuple);
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
        if let Definition::Local(local_id) = &ident.definition {
            self.remember_use_of_variable(*local_id, ident.id);
        }
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
        if matches!(unary.operator, UnaryOp::Reference { .. }) {
            // When a reference is taken to a local variable or one of its fields (e.g. `&mut x`
            // or `&mut x.field`), the variable `x` is now aliased. Mark it so that any future
            // copy of `x` must clone rather than move.
            // See `referenced_variables` for the full explanation.
            if let Some(local_id) = base_ident_of_field_access(&unary.rhs) {
                self.referenced_variables.insert(local_id);
            }
        }
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
        let orig_indices = vecmap(&self.last_uses, |(id, (index, _))| (*id, *index));

        self.push_loop_scope();
        for expr in loop_exprs {
            self.track_variables_in_expression(expr);
        }
        self.pop_loop_scope();

        for (id, orig_index) in orig_indices {
            if let Some((index, branches)) = self.last_uses.get_mut(&id)
                && *index != orig_index
            {
                *index = orig_index;
                // If the value is still accessible outside the loop, don't move it inside the loop after it has been redeclared.
                *branches = Branches::None;
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
        for arg in &call.arguments {
            if !conservative
                && let Expression::Unary(unary) = arg
                && matches!(unary.operator, UnaryOp::Reference { .. })
                && base_ident_of_field_access(&unary.rhs).is_some()
            {
                // Track the use of the variable inside the reference (for last-use analysis)
                // but skip the unary handler, which would mark the variable as aliased.
                self.track_variables_in_expression(&unary.rhs);
            } else {
                self.track_variables_in_expression(arg);
            }
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
            ast::LValue::Ident(ast::Ident { definition: Definition::Local(local_id), .. }) => {
                Some(*local_id)
            }
            _ => None,
        };

        if let Some(local_id) = &variable {
            // Adjust its loop index to be the current loop, so that `remember_use_of_variable`
            // remembers any last use, rather than clear out its current state.
            let current_index = self.loop_index();
            if let Some((index, _)) = self.last_uses.get_mut(local_id) {
                *index = current_index;
            }
        }

        self.track_variables_in_expression(&assign.expression);

        if let Some(local_id) = variable {
            // Confirm any last uses we have on the variable at this point (which may be in `assign.expression`),
            // From here on it acts as a newly declared variable with no history.
            if let Some((_, branches)) = self.last_uses.get_mut(&local_id) {
                let path = self
                    .current_loop_and_branch
                    .last()
                    .expect("We should always have at least 1 path");
                let extracted = Self::extract_branch_at_path(branches, path);
                self.confirmed_moves.entry(local_id).or_default().extend(extracted.flatten_uses());
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
/// Returns `true` if `arg`'s type can be used to store a reference, i.e. the type
/// contains a `&mut T` (at any depth) where `T` itself contains a reference.
///
/// This covers both direct `&mut &mut T` arguments and arguments whose struct/tuple/array
/// type has a field of such a type, as well as reaching through immutable references to
/// find inner mutable ones (e.g. `& SomeStruct` where `SomeStruct` has a `&mut &mut T` field).
///
/// When this is true for any argument, all `&mut x` arguments in the call must conservatively
/// be treated as aliasing `x`, because the callee might write the reference into the location
/// reachable through that argument.
fn arg_can_store_reference(arg: &Expression) -> bool {
    // Expression::return_type() covers all expression variants that carry type information.
    // For the few statement-like variants that return None (For, Loop, While, Let, etc.)
    // being conservative (true) is fine since they cannot be call arguments anyway.
    match arg.return_type() {
        Some(typ) => type_can_store_reference(&typ),
        None => true,
    }
}

/// Returns `true` if `typ` contains — at any depth — a `&mut T` where `T` itself contains
/// a reference. Such a type allows a reference to be written somewhere persistent.
///
/// We recurse through immutable references too: given `& SomeStruct`, its inner mutable
/// reference fields are still accessible and writable (e.g. `*(s.slot) = x`).
fn type_can_store_reference(typ: &ast::Type) -> bool {
    use ast::Type;
    match typ {
        // A mutable reference to something that contains a reference can be written through.
        Type::Reference(inner, true /* mutable */) => type_contains_reference(inner),
        // An immutable reference can't be written to directly, but its inner fields may still
        // expose mutable references we can write through — so recurse.
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
