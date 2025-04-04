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
/// Branches::If {
///     then_branch: Branches::If {
///         then_branch: Branches::Direct(3),
///         else_branch: Branches::None,
///     },
///     else_branch: Branches::Direct(4),
/// }
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

/// A single path through a `Branches` enum
#[derive(Debug)]
enum BranchesPath {
    /// We've reached our destination
    Here,
    /// We're in a fork in the road, take the branch at the given index
    IfOrMatch { branch_index: usize, rest: Box<BranchesPath> },
}

struct LastUseContext {
    /// This is meant to mirror the structure in `context.last_uses`.
    /// The outer `Vec` is each loop we're currently in, while the `BranchPath` contains
    /// the path to overwrite the last use in any `Branches` enums of the variables we find.
    current_loop_and_branch: Vec<BranchesPath>,

    /// Stores the location of each variable's last use
    ///
    /// Map from each local variable to the last "use instance" of that variable.
    /// E.g. if the last time a variable is used is the third time it is used (in traversal order)
    /// then this map will contain the pair `(LocalId(_), 3)`.
    ///
    /// Note that when a variable is declared it has 0 uses until the first time it is used after
    /// being declared where it now has 1 use, etc.
    ///
    /// This map is used to move the variable on last use instead of cloning it. Code using this
    /// should take care to keep track of any loops the variable is used in. E.g. in:
    /// ```noir
    /// let x = [1, 2];
    /// for i in 0 .. 2 {
    ///     foo(x);
    /// }
    /// ```
    /// The `x` in `foo(x)` should not be counted as a last use since it still must be cloned to
    /// be used on the next iteration of the loop.
    ///
    /// The outer Vec is each loop scope - we always only look at variables declared in the current
    /// loop scope. Variables declared outside of it should always be cloned when used in loops.
    /// The inner hashmap maps from id to its last uses in each branch. For most cases this branch
    /// is just `Branches::Direct { last_use }` but in the case of `if` or `match` expressions, a
    /// variable may have a last use in each branch which the `Branches` enum tracks.
    last_uses: HashMap<LocalId, (/*loop index*/ usize, Branches)>,
}

impl Context {
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
        let loop_index = self.current_loop_and_branch.len() - 1;
        self.last_uses.insert(id, (loop_index, Branches::None));
    }

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
            (branch, BranchesPath::Here) => {
                *branch = Branches::Direct(variable);
            }
            (
                branch @ (Branches::None | Branches::Direct { .. }),
                BranchesPath::IfOrMatch { branch_index, rest: _ },
            ) => {
                // The branch doesn't exist for this variable; create it
                let inner_branches =
                    std::iter::repeat_n(Branches::None, *branch_index + 1).collect();
                *branch = Branches::IfOrMatch(inner_branches);
                Self::remember_use_of_variable_rec(branch, path, variable);
            }
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
                self.track_variables_in_expression(tuple)
            }
            Expression::Call(call) => self.track_variables_in_call(call),
            Expression::Let(let_expr) => self.track_variables_in_let(let_expr),
            Expression::Constrain(boolean, _location, msg) => {
                self.track_variables_in_constrain(boolean, msg)
            }
            Expression::Assign(assign) => self.track_variables_in_assign(assign),
            Expression::Semi(expr) => self.track_variables_in_expression(expr),
            Expression::Clone(_) => unreachable!("last_uses is called before clones are inserted"),
            Expression::Drop(_) => unreachable!("last_uses is called before drops are inserted"),
            Expression::Break => (),
            Expression::Continue => (),
        }
    }

    /// Under the experimental alternate ownership scheme, whenever an ident is used it is
    /// always cloned unless it is the last use of the ident (not in a loop). To simplify this
    /// analysis we always clone here then remove the last clone later if possible.
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

    /// - Extracting an array from another array (`let inner: [_; _] = array[0];`):
    ///   - Extracting a nested array from its outer array will always increment the reference count
    ///     of the nested array.
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

    fn track_variables_in_lvalue(&mut self, lvalue: &ast::LValue) {
        match lvalue {
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
