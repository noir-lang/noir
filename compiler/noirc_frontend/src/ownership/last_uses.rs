//! This module contains the last use analysis pass which is run on each function before
//! the ownership pass when the experimental ownership scheme is enabled. This pass does
//! not run without this experimental flag - and if it did its results would go unused.
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
//!   must be cloned in any loop with an index greater than the variable's own loop index.
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
use crate::monomorphization::ast::{self, IdentId, LocalId};
use crate::monomorphization::ast::{Expression, Function, Literal};
use fxhash::FxHashMap as HashMap;

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
    /// No use in this branch or there is no branch
    None,
    Direct(IdentId),
    IfOrMatch(Vec<Branches>),
}

impl Branches {
    /// Collect all IdentIds from this tree
    fn flatten_uses(self) -> Vec<IdentId> {
        match self {
            Branches::None => Vec::new(),
            Branches::Direct(ident_id) => vec![ident_id],
            Branches::IfOrMatch(cases) => cases.into_iter().flat_map(Self::flatten_uses).collect(),
        }
    }
}

/// A single path through a `Branches` enum.
///
/// This is used by the context to keep track of where we currently are within a function.
#[derive(Debug)]
enum BranchesPath {
    /// We've reached our destination
    Here,
    /// We're in a fork in the road, take the branch at the given index
    IfOrMatch { branch_index: usize, rest: Box<BranchesPath> },
}

struct LastUseContext {
    /// The outer `Vec` is each loop we're currently in, while the `BranchPath` contains
    /// the path to overwrite the last use in any `Branches` enums of the variables we find.
    /// As a whole, this tracks the current control-flow of the function we're in.
    current_loop_and_branch: Vec<BranchesPath>,

    /// Stores the location of each variable's last use
    ///
    /// Map from each local variable to the last instance of that variable. Separate uses of
    /// the same variable are differentiated by that identifier's `IdentId` which is always
    /// different on separate identifiers, unlike the `LocalId` which is the same for any
    /// identifier referring to the same underlying definition.
    ///
    /// Each definition is mapped to a loop index and a Branches enumeration.
    /// - The loop index tracks how many loops the variable was declared within. It may be moved
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
}

impl Context {
    /// Traverse the given function and return the last use(s) of each local variable.
    /// A variable may have multiple last uses if it was last used within a conditional expression.
    pub(super) fn find_last_uses_of_variables(
        function: &Function,
    ) -> HashMap<LocalId, Vec<IdentId>> {
        let mut context =
            LastUseContext { current_loop_and_branch: Vec::new(), last_uses: HashMap::default() };
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
        self.current_loop_and_branch.push(BranchesPath::Here);
    }

    fn pop_loop_scope(&mut self) {
        self.current_loop_and_branch.pop();
    }

    fn branch(&mut self, branch_index: usize) {
        let path =
            self.current_loop_and_branch.last_mut().expect("We should always have at least 1 path");
        let rest = Box::new(std::mem::replace(path, BranchesPath::Here));
        *path = BranchesPath::IfOrMatch { branch_index, rest };
    }

    fn unbranch(&mut self) {
        let path =
            self.current_loop_and_branch.last_mut().expect("We should always have at least 1 path");
        let rest = std::mem::replace(path, BranchesPath::Here);

        match rest {
            BranchesPath::Here => panic!("unbranch called without any branches"),
            BranchesPath::IfOrMatch { branch_index: _, rest } => *path = *rest,
        }
    }

    fn loop_index(&self) -> usize {
        self.current_loop_and_branch.len() - 1
    }

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
            self.current_loop_and_branch.last().expect("We should always have at least 1 scope");
        let loop_index = self.loop_index();

        if let Some((variable_loop_index, uses)) = self.last_uses.get_mut(&id) {
            if *variable_loop_index == loop_index {
                Self::remember_use_of_variable_rec(uses, path, variable);
            } else {
                *uses = Branches::None;
            }
        }
    }

    fn remember_use_of_variable_rec(
        branches: &mut Branches,
        path: &BranchesPath,
        variable: IdentId,
    ) {
        match (branches, path) {
            // Path is direct, overwrite the last use entirely
            (branch, BranchesPath::Here) => {
                *branch = Branches::Direct(variable);
            }
            // Our path says we need to enter an if or match but the variable's last
            // use was direct. So we need to overwrite the last use with an empty IfOrMatch
            // and write to the relevant branch of that
            (
                branch @ (Branches::None | Branches::Direct { .. }),
                BranchesPath::IfOrMatch { branch_index, rest: _ },
            ) => {
                // The branch doesn't exist for this variable; create it
                let empty_branches =
                    std::iter::repeat_n(Branches::None, *branch_index + 1).collect();
                *branch = Branches::IfOrMatch(empty_branches);
                Self::remember_use_of_variable_rec(branch, path, variable);
            }
            // The variable's last use was in an if or match, and our current path is in one
            // as well so just follow to the relevant branch in both. We just need to possibly
            // resize the variable's branches count in case it was created with fewer than we
            // know we currently have.
            (Branches::IfOrMatch(branches), BranchesPath::IfOrMatch { branch_index, rest }) => {
                let required_len = *branch_index + 1;
                if branches.len() < required_len {
                    branches.resize(required_len, Branches::None);
                }
                Self::remember_use_of_variable_rec(&mut branches[*branch_index], rest, variable);
            }
        }
    }

    fn get_variables_to_move(self) -> HashMap<LocalId, Vec<IdentId>> {
        self.last_uses
            .into_iter()
            .map(|(definition, (_, last_uses))| (definition, last_uses.flatten_uses()))
            .collect()
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
            Expression::Tuple(elems) => self.track_variables_in_tuple(elems),
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
        if let ast::Definition::Local(local_id) = &ident.definition {
            self.remember_use_of_variable(*local_id, ident.id);
        }
    }

    fn track_variables_in_literal(&mut self, literal: &Literal) {
        match literal {
            Literal::Integer(..) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => (),

            Literal::FmtStr(_, _, captures) => self.track_variables_in_expression(captures),

            Literal::Array(array) | Literal::Slice(array) => {
                for element in array.contents.iter() {
                    self.track_variables_in_expression(element);
                }
            }
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
        self.track_variables_in_expression(&for_expr.start_range);
        self.track_variables_in_expression(&for_expr.end_range);
        self.track_variables_in_loop(&for_expr.block);
    }

    fn track_variables_in_while(&mut self, while_expr: &ast::While) {
        self.track_variables_in_expression(&while_expr.condition);
        self.track_variables_in_loop(&while_expr.body);
    }

    fn track_variables_in_loop(&mut self, loop_body: &Expression) {
        self.push_loop_scope();
        self.track_variables_in_expression(loop_body);
        self.pop_loop_scope();
    }

    fn track_variables_in_if(&mut self, if_expr: &ast::If) {
        self.track_variables_in_expression(&if_expr.condition);

        self.branch(0);
        self.track_variables_in_expression(&if_expr.consequence);
        self.unbranch();

        if let Some(alt) = &if_expr.alternative {
            self.branch(1);
            self.track_variables_in_expression(alt);
            self.unbranch();
        }
    }

    fn track_variables_in_match(&mut self, match_expr: &ast::Match) {
        for (i, case) in match_expr.cases.iter().enumerate() {
            for argument in &case.arguments {
                self.declare_variable(*argument);
            }

            self.branch(i);
            self.track_variables_in_expression(&case.branch);
            self.unbranch();
        }

        if let Some(default_case) = &match_expr.default_case {
            self.branch(match_expr.cases.len());
            self.track_variables_in_expression(default_case);
            self.unbranch();
        }
    }

    fn track_variables_in_tuple(&mut self, elems: &[Expression]) {
        for elem in elems {
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
        self.track_variables_in_lvalue(&assign.lvalue);
        self.track_variables_in_expression(&assign.expression);
    }

    /// A variable in an lvalue position is never moved (otherwise you wouldn't
    /// be able to access the variable you assigned to afterward). However, the
    /// index in an array expression `a[i] = ...` is an arbitrary expression that
    /// is actually in an rvalue position and can thus be moved.
    ///
    /// Subtle point: since we don't track identifier uses here at all this means
    /// if the last use of one was just before it is assigned, it can actually be
    /// moved before it is assigned. This should be fine because we move out of the
    /// binding, and the binding isn't used until it is set to a new value.
    fn track_variables_in_lvalue(&mut self, lvalue: &ast::LValue) {
        match lvalue {
            // All identifiers in lvalues are implicitly `&mut ident` and thus aren't moved
            ast::LValue::Ident(_) => (),
            ast::LValue::Index { array, index, element_type: _, location: _ } => {
                self.track_variables_in_expression(index);
                self.track_variables_in_lvalue(array);
            }
            ast::LValue::MemberAccess { object, field_index: _ } => {
                self.track_variables_in_lvalue(object);
            }
            ast::LValue::Dereference { reference, element_type: _ } => {
                self.track_variables_in_lvalue(reference);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        monomorphization::ast::{IdentId, LocalId},
        test_utils::get_monomorphized,
    };

    #[test]
    fn smoke_test() {
        let src = "
        fn main(d: [Field; 2]) {
            if len(d) == 2 {              // use 1 of d
                if len(d) == 2 {          // use 2 of d
                    assert(eq(d, [5, 6]));  // use 3 of d
                }
            } else {
                assert(eq(d, [5, 6]));      // use 4 of d
            }
        }

        fn eq(lhs: [Field; 2], rhs: [Field; 2]) -> bool {
            (lhs[0] == rhs[0]) & (lhs[1] == rhs[1])
        }
        
        fn len(arr: [Field; 2]) -> u32 {
            2
        }
        ";

        let program = get_monomorphized(src, None, crate::test_utils::Expect::Success).unwrap();

        let function = program.main();
        let last_uses = super::Context::find_last_uses_of_variables(function);

        let d_local_id = LocalId(0);
        let d_last_uses = &last_uses[&d_local_id];

        // We should improve testing of this pass so that it is more clear where these ids
        // correspond to.
        assert_eq!(*d_last_uses, &[IdentId(7), IdentId(10)]);
    }
}
