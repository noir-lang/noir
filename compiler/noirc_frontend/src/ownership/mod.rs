//! This module implements the "ownership analysis" compiler pass on the
//! monomorphized AST. It is run after monomorphization and before SSA-gen.
//! At this point the monomorphized AST has no polymorphic types and all functions
//! are specialized into either constrained or unconstrained versions. This pass
//! only operates on unconstrained functions since only Noir's unconstrained runtime Brillig
//! has any notion of cloning a value - specifically arrays with their brillig-only reference counts.
//!
//! Note that this documentation may refer to cloning a value or incrementing an array's reference
//! count. These operations are equivalent on arrays. Cloning may be applied to any value and only
//! increments the reference counts of any arrays contained within (but not behind references or
//! inside nested arrays). This document also focuses on arrays but all reference count operations
//! on arrays are also performed on vectors.
//!
//! Arrays in brillig have copy on write semantics which relies on us incrementing their
//! reference counts when they are shared in multiple places. Note that while Noir has references,
//! arrays can also be shared by value and we want to avoid clones when possible. This pass
//! clones arrays (increments their reference counts) in the following situations which roughly
//! correspond to where a `Copy` variable in Rust would be copied:
//! - Variables are copied on each use, except for the last use where they are moved.
//!   - If a variable's last use is in a loop that it was not defined in, it is copied instead of moved.
//!   - The last use analysis isn't sophisticated on struct fields. It will count `a.b` and `a.c`
//!     both as uses of `a`. Even if both could conceptually be moved, only the last usage will be
//!     moved and the first (say `a.b`) will still be cloned.
//! - Dereferences always clone.
//! - Certain expressions will avoid cloning or delay where clones are performed:
//!   - Reference expressions `&e` will not clone `e` but may still clone variables used within.
//!     - E.g. `&foo.bar` will not clone but `&foo(bar)` may still clone `foo` or `bar`.
//!   - Dereferences will attempt to extract a field first if possible.
//!     - E.g. `(*self).b.c` is transformed to `*(self.b.c)` where the `*` operation also clones.
//!   - Ordinary member access will also delay clones.
//!     - E.g. `self.b.c` is compiled as `self.b.c.clone()` over `self.clone().b.c`
//!   - Array indexing `a[i]` will avoid cloning `a`. The extracted element is always cloned.
//!
//! Most of this logic is contained in this file except for the last use analysis which is in the
//! `last_uses` module. That module contains a separate pass run on each function before this pass
//! to find the last use of each local variable to identify where moves can occur.
use crate::{
    ast::UnaryOp,
    monomorphization::ast::{
        Definition, Expression, Function, Ident, IdentId, LValue, Literal, LocalId, Program, Type,
        Unary,
    },
};

use rustc_hash::FxHashMap as HashMap;

mod last_uses;
mod tests;

impl Program {
    /// Perform "ownership analysis".
    ///
    /// See [ownership](crate::ownership) for details.
    ///
    /// This should only be called once, before converting to SSA.
    pub fn handle_ownership(mut self) -> Self {
        for function in self.functions.iter_mut() {
            function.handle_ownership();
        }
        self
    }
}

impl Function {
    /// Perform "ownership analysis".
    ///
    /// See [ownership](crate::ownership) for details.
    ///
    /// This should only be called on a function once.
    pub fn handle_ownership(&mut self) {
        let mut context = Context { variables_to_move: Default::default() };
        context.handle_ownership_in_function(self);
    }
}

struct Context {
    /// This contains each instance of a variable we should move instead of cloning.
    variables_to_move: HashMap<LocalId, Vec<IdentId>>,
}

impl Context {
    fn should_move(&self, definition: LocalId, variable: IdentId) -> bool {
        self.variables_to_move
            .get(&definition)
            .is_some_and(|instances_to_move| instances_to_move.contains(&variable))
    }

    fn handle_ownership_in_function(&mut self, function: &mut Function) {
        if !function.unconstrained {
            return;
        }

        self.variables_to_move = Self::find_last_uses_of_variables(function);
        self.handle_expression(&mut function.body);
    }

    fn handle_expression(&mut self, expr: &mut Expression) {
        match expr {
            Expression::Ident(_) => self.handle_ident(expr),
            Expression::Literal(literal) => self.handle_literal(literal),
            Expression::Block(exprs) => {
                exprs.iter_mut().for_each(|expr| self.handle_expression(expr));
            }
            Expression::Unary(_) => self.handle_unary(expr),
            Expression::Binary(binary) => self.handle_binary(binary),
            Expression::Index(_) => self.handle_index(expr),
            Expression::Cast(cast) => self.handle_cast(cast),
            Expression::For(for_expr) => self.handle_for(for_expr),
            Expression::Loop(loop_expr) => self.handle_expression(loop_expr),
            Expression::While(while_expr) => self.handle_while(while_expr),
            Expression::If(if_expr) => self.handle_if(if_expr),
            Expression::Match(match_expr) => self.handle_match(match_expr),
            Expression::Tuple(elements) => self.handle_tuple(elements),
            Expression::ExtractTupleField(..) => self.handle_extract_expression(expr),
            Expression::Call(call) => self.handle_call(call),
            Expression::Let(let_expr) => self.handle_let(let_expr),
            Expression::Constrain(boolean, _location, msg) => self.handle_constrain(boolean, msg),
            Expression::Assign(assign) => self.handle_assign(assign),
            Expression::Semi(expr) => self.handle_expression(expr),
            // Clones & Drops are only inserted by this pass so we can assume any code they
            // contain is already handled
            Expression::Clone(_) => (),
            Expression::Drop(_) => (),
            Expression::Break => (),
            Expression::Continue => (),
        }
    }

    /// Handle the RHS of a `&expr` unary expression.
    /// Variables and field accesses in these expressions are exempt from clones.
    ///
    /// Note that this also matches on dereference operations to exempt their LHS from clones,
    /// but their LHS is always exempt from clones so this is unchanged.
    fn handle_reference_expression(&mut self, expr: &mut Expression) {
        match expr {
            Expression::Ident(_) => (),
            Expression::Block(exprs) => {
                let len_minus_one = exprs.len().saturating_sub(1);
                for expr in exprs.iter_mut().take(len_minus_one) {
                    // In `&{ a; b; ...; z }` we're only taking the reference of `z`.
                    self.handle_expression(expr);
                }
                if let Some(expr) = exprs.last_mut() {
                    self.handle_reference_expression(expr);
                }
            }
            Expression::Unary(Unary { rhs, operator: UnaryOp::Dereference { .. }, .. }) => {
                self.handle_reference_expression(rhs);
            }
            Expression::ExtractTupleField(tuple, _index) => self.handle_reference_expression(tuple),

            Expression::Index(index) => {
                self.handle_reference_expression(&mut index.collection);
                self.handle_expression(&mut index.index);
            }

            // If we have something like `f(arg)` then we want to treat those variables normally
            // rather than avoid cloning them. So we shouldn't recur in `handle_reference_expression`.
            other => self.handle_expression(other),
        }
    }

    /// Handle an [Expression::ExtractTupleField] by moving the cloning to limit its scope to the
    /// innermost item it needs to be applied to.
    ///
    /// Panics if called on a different kind of expression.
    fn handle_extract_expression(&mut self, expr: &mut Expression) {
        let Expression::ExtractTupleField(tuple, index) = expr else {
            panic!("handle_extract_expression given non-extract expression {expr}");
        };

        // We want to avoid cloning the entire object if we're only accessing one field of it
        // so we check here to move the clone to the outermost extract expression instead.
        // E.g. we want to change `a.clone().b.c` to `a.b.c.clone()`.
        if let Some((should_clone, tuple_type)) = self.handle_extract_expression_rec(tuple) {
            if let Some(elements) = unwrap_tuple_type(tuple_type) {
                if should_clone && contains_array_or_str_type(&elements[*index]) {
                    clone_expr(expr);
                }
            }
        } else {
            self.handle_expression(tuple);
        }
    }

    /// Traverse an expression comprised of only identifiers, tuple field extractions, and
    /// dereferences returning whether we should clone the result and the type of that result.
    ///
    /// Returns None if a different expression variant was found.
    fn handle_extract_expression_rec(&mut self, expr: &mut Expression) -> Option<(bool, Type)> {
        match expr {
            Expression::Ident(ident) => {
                let should_clone = self.should_clone_ident(ident);
                Some((should_clone, ident.typ.clone()))
            }
            // Delay dereferences as well so we change `(*self).foo.bar` to `*(self.foo.bar)`
            Expression::Unary(Unary {
                rhs,
                operator: UnaryOp::Dereference { .. },
                result_type,
                ..
            }) => {
                self.handle_reference_expression(rhs);
                Some((true, result_type.clone()))
            }
            Expression::ExtractTupleField(tuple, index) => {
                let (should_clone, typ) = self.handle_extract_expression_rec(tuple)?;
                let mut elements = unwrap_tuple_type(typ)?;
                Some((should_clone, elements.swap_remove(*index)))
            }
            _ => None,
        }
    }

    /// Whenever an ident is used it is always cloned unless it is the last use of the ident (not in a loop).
    fn should_clone_ident(&self, ident: &Ident) -> bool {
        match &ident.definition {
            Definition::Local(local_id) => {
                contains_array_or_str_type(&ident.typ) && !self.should_move(*local_id, ident.id)
            }
            // Globals are always cloned if they contain arrays
            Definition::Global(_) => contains_array_or_str_type(&ident.typ),
            _ => false,
        }
    }

    fn handle_ident(&self, expr: &mut Expression) {
        let ident = match expr {
            Expression::Ident(ident) => ident,
            other => panic!("handle_ident given non-ident expr: {other}"),
        };

        if self.should_clone_ident(ident) {
            clone_expr(expr);
        }
    }

    fn handle_literal(&mut self, literal: &mut Literal) {
        match literal {
            Literal::Integer(..) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => (),

            Literal::FmtStr(_, _, captures) => self.handle_expression(captures),

            Literal::Array(array) | Literal::Vector(array) => {
                for element in array.contents.iter_mut() {
                    self.handle_expression(element);
                }
            }
        }
    }

    fn handle_unary(&mut self, expr: &mut Expression) {
        let unary = match expr {
            Expression::Unary(unary) => unary,
            other => panic!("handle_unary given non-unary expression: {other}"),
        };

        // Don't clone `rhs` if this is a reference or dereference expression.
        // - If this is a reference expression `&rhs`, `rhs` by definition shouldn't be cloned
        // - If this is `*rhs` we're going to clone the extracted element instead
        if matches!(unary.operator, UnaryOp::Reference { .. } | UnaryOp::Dereference { .. }) {
            self.handle_reference_expression(&mut unary.rhs);
        } else {
            self.handle_expression(&mut unary.rhs);
        }

        if matches!(unary.operator, UnaryOp::Dereference { .. })
            && contains_array_or_str_type(&unary.result_type)
        {
            clone_expr(expr);
        }
    }

    fn handle_binary(&mut self, binary: &mut crate::monomorphization::ast::Binary) {
        self.handle_expression(&mut binary.lhs);
        self.handle_expression(&mut binary.rhs);
    }

    fn handle_index(&mut self, index_expr: &mut Expression) {
        let Expression::Index(index) = index_expr else {
            panic!("handle_index given non-index expression: {index_expr}");
        };

        // Don't clone the collection, cloning only the resulting element is cheaper.
        self.handle_reference_expression(&mut index.collection);
        self.handle_expression(&mut index.index);

        // If the index collection is being borrowed we need to clone the result.
        if contains_array_or_str_type(&index.element_type) {
            clone_expr(index_expr);
        }
    }

    fn handle_cast(&mut self, cast: &mut crate::monomorphization::ast::Cast) {
        self.handle_expression(&mut cast.lhs);
    }

    fn handle_for(&mut self, for_expr: &mut crate::monomorphization::ast::For) {
        self.handle_expression(&mut for_expr.start_range);
        self.handle_expression(&mut for_expr.end_range);
        self.handle_expression(&mut for_expr.block);
    }

    fn handle_while(&mut self, while_expr: &mut crate::monomorphization::ast::While) {
        self.handle_expression(&mut while_expr.condition);
        self.handle_expression(&mut while_expr.body);
    }

    fn handle_if(&mut self, if_expr: &mut crate::monomorphization::ast::If) {
        self.handle_expression(&mut if_expr.condition);
        self.handle_expression(&mut if_expr.consequence);
        if let Some(alt) = &mut if_expr.alternative {
            self.handle_expression(alt);
        }
    }

    fn handle_match(&mut self, match_expr: &mut crate::monomorphization::ast::Match) {
        // Note: We don't need to explicitly handle `Match::variable_to_match` here.
        // The matched variable is just a LocalId reference to a variable that was assigned earlier.
        // Cloning for that variable happens at its use sites (e.g., when passed to the enum
        // constructor or used after the match), not at the match expression itself.
        // The match will only destructure the value; it doesn't "use" the variable in a way that
        // requires additional cloning beyond what the last-use analysis already handles.
        for case in &mut match_expr.cases {
            self.handle_expression(&mut case.branch);
        }

        if let Some(default_case) = &mut match_expr.default_case {
            self.handle_expression(default_case);
        }
    }

    fn handle_tuple(&mut self, elements: &mut [Expression]) {
        for elem in elements {
            self.handle_expression(elem);
        }
    }

    fn handle_call(&mut self, call: &mut crate::monomorphization::ast::Call) {
        self.handle_expression(&mut call.func);
        for arg in &mut call.arguments {
            self.handle_expression(arg);
        }

        // Hack to avoid clones when calling `array.len()`.
        // That function takes arrays by value but we know it never mutates them.
        if let Expression::Ident(ident) = call.func.as_ref() {
            if let Definition::Builtin(name) = &ident.definition {
                if name == "array_len" {
                    if let Some(Expression::Clone(array)) = call.arguments.get_mut(0) {
                        let array =
                            std::mem::replace(array.as_mut(), Expression::Literal(Literal::Unit));
                        call.arguments[0] = array;
                    }
                }
            }
        }
    }

    fn handle_let(&mut self, let_expr: &mut crate::monomorphization::ast::Let) {
        self.handle_expression(&mut let_expr.expression);
    }

    fn handle_constrain(
        &mut self,
        boolean: &mut Expression,
        msg: &mut Option<Box<(Expression, crate::hir_def::types::Type)>>,
    ) {
        self.handle_expression(boolean);

        if let Some(msg) = msg {
            self.handle_expression(&mut msg.0);
        }
    }

    fn handle_assign(&mut self, assign: &mut crate::monomorphization::ast::Assign) {
        self.handle_expression(&mut assign.expression);
        self.handle_lvalue(&mut assign.lvalue);
    }

    fn handle_lvalue(&mut self, lvalue: &mut LValue) {
        match lvalue {
            // A variable can never be moved into an LValue position so it doesn't
            // need to be cloned or checked here.
            LValue::Ident(_) => (),
            LValue::Index { array, index, element_type: _, location: _ } => {
                self.handle_expression(index);
                self.handle_lvalue(array);

                if contains_index(array) {
                    **array = LValue::Clone(array.clone());
                }
            }
            LValue::MemberAccess { object, field_index: _ } => {
                self.handle_lvalue(object);
            }
            LValue::Dereference { reference, element_type: _ } => {
                self.handle_lvalue(reference);
            }
            // LValue::Clone isn't present before this pass and is only inserted after we already
            // handle the corresponding lvalue
            LValue::Clone(_) => unreachable!("LValue::Clone should only be inserted by this pass"),
        }
    }
}

fn contains_index(lvalue: &LValue) -> bool {
    use LValue::*;
    match lvalue {
        Ident(_) => false,
        Index { .. } => true,
        Dereference { reference: lvalue, .. }
        | MemberAccess { object: lvalue, .. }
        | Clone(lvalue) => contains_index(lvalue),
    }
}

/// Adds a `.clone()` to the given expression.
/// Note that this method should be careful not to actually duplicate the given expression
/// so that we do not duplicate any side-effects.
fn clone_expr(expr: &mut Expression) {
    let unit = Expression::Literal(Literal::Unit);
    let old_expr = std::mem::replace(expr, unit);
    *expr = Expression::Clone(Box::new(old_expr));
}

/// Returns `true` if the type contains an `Array`, `Vector`, `String` or `FmtString`,
/// directly or as part of a `Tuple`, but _not_ through a reference.
fn contains_array_or_str_type(typ: &Type) -> bool {
    match typ {
        Type::Field
        | Type::Integer(..)
        | Type::Bool
        | Type::Unit
        | Type::Function(..)
        | Type::Reference(..) => false,

        Type::Array(_, _) | Type::String(_) | Type::FmtString(_, _) | Type::Vector(_) => true,

        Type::Tuple(elements) => elements.iter().any(contains_array_or_str_type),
    }
}

/// Returns the element types of a [Type::Tuple], or a reference to a tuple.
///
/// Returns `None` for any other type.
fn unwrap_tuple_type(typ: Type) -> Option<Vec<Type>> {
    match typ {
        Type::Tuple(elements) => Some(elements),
        // array accesses will automatically dereference so we do too
        Type::Reference(element, _) => unwrap_tuple_type(*element),
        _ => None,
    }
}
