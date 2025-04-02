use crate::monomorphization::ast;
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
/// ```
/// Branches::If {
///     then_branch: Branches::If {
///         then_branch: Branches::Direct(3),
///         else_branch: Branches::None,
///     },
///     else_branch: Branches::Direct(4),
/// }
/// ```
#[derive(Debug)]
pub(super) enum Branches {
    /// No use in this branch or there is no branch
    None,
    Direct {
        use_number: u32,
    },
    If {
        then_branch: Box<Branches>,
        else_branch: Box<Branches>,
    },
    Match(Vec<Branches>),
}

impl Context {
    pub(super) fn find_last_uses_of_variables(&mut self, function: &Function) {
        self.track_variables_in_expression(&function.body);
    }

    fn push_loop_scope(&mut self) {
        self.last_uses.push(HashMap::default());
    }

    fn pop_loop_scope(&mut self) {
        self.last_uses.push(HashMap::default());
    }

    fn remember_use_of_variable(&mut self, id: ast::LocalId) {
        let last_uses = self.last_uses.last_mut().expect("We should always have at least 1 scope");
        if let Some(uses) = last_uses.get_mut(&id) {
            if let Some(use_count) = self.variable_use_counts.get_mut(&id) {
                *use_count += 1;
                uses.push(*use_count);
            }
        }
    }

    fn track_variables_in_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Ident(_) => self.track_variables_in_ident(expr),
            Expression::Literal(literal) => self.track_variables_in_literal(literal),
            Expression::Block(exprs) => {
                exprs.iter().for_each(|expr| self.track_variables_in_expression(expr));
            }
            Expression::Unary(unary) => self.track_variables_in_unary(unary),
            Expression::Binary(binary) => self.track_variables_in_binary(binary),
            Expression::Index(index) => self.track_variables_in_index(index),
            Expression::Cast(cast) => self.track_variables_in_cast(cast),
            Expression::For(for_expr) => self.track_variables_in_for(for_expr),
            Expression::Loop(loop_expr) => self.track_variables_in_expression(loop_expr),
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
            // Clones & Drops are only inserted by this pass so we can assume any code they
            // contain is already handled
            Expression::Clone(_) => (),
            Expression::Drop(_) => (),
            Expression::Break => (),
            Expression::Continue => (),
        }
    }

    /// Handle the rhs of a `&expr` unary expression.
    /// When the experimental ownership flag is enabled variables and field accesses
    /// in these expressions are exempt from clones.
    fn track_variables_in_reference_expression(&mut self, expr: &mut Expression) {
        match expr {
            Expression::Ident(_) => (),
            Expression::Block(exprs) => {
                let len_minus_one = exprs.len().saturating_sub(1);
                for expr in exprs.iter_mut().take(len_minus_one) {
                    // In `&{ a; b; ...; z }` we're only taking the reference of `z`.
                    self.track_variables_in_expression(expr);
                }
                if let Some(expr) = exprs.last_mut() {
                    self.track_variables_in_reference_expression(expr);
                }
            }
            Expression::ExtractTupleField(tuple, _index) => {
                self.track_variables_in_reference_expression(tuple)
            }

            // If we have something like `f(arg)` then we want to treat those variables normally
            // rather than avoid cloning them. So we shouldn't recur in `track_variables_in_reference_expression`.
            other => self.track_variables_in_expression(other),
        }
    }

    /// Under the experimental alternate ownership scheme, whenever an ident is used it is
    /// always cloned unless it is the last use of the ident (not in a loop). To simplify this
    /// analysis we always clone here then remove the last clone later if possible.
    fn track_variables_in_ident(&self, _expr: &Expression) {
        todo!()
    }

    /// - Array literals:
    ///   - Arrays stored inside a nested array literal (e.g. both variables in `[array1, array2]`
    ///     have their reference count incremented).
    ///   - This does not apply to nested array literals since we know they are not referenced elsewhere.
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
        self.track_variables_in_expression(&for_expr.block);
    }

    fn track_variables_in_while(&mut self, while_expr: &ast::While) {
        self.track_variables_in_expression(&while_expr.condition);
        self.track_variables_in_expression(&while_expr.body);
    }

    fn track_variables_in_if(&mut self, if_expr: &ast::If) {
        self.track_variables_in_expression(&if_expr.condition);
        self.track_variables_in_expression(&if_expr.consequence);
        if let Some(alt) = &if_expr.alternative {
            self.track_variables_in_expression(alt);
        }
    }

    fn track_variables_in_match(&mut self, match_expr: &ast::Match) {
        for case in &match_expr.cases {
            self.track_variables_in_expression(&case.branch);
        }

        if let Some(default_case) = &match_expr.default_case {
            self.track_variables_in_expression(default_case);
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

    /// - Assignments (`x = <expression which returns an array>;`):
    ///   - Assigning an array to an existing variable will also increment the reference
    ///     count of the array unless it is an array literal.
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
