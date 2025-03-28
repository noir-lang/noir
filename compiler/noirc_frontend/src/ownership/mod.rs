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
//! on arrays are also performed on slices.
//!
//! Arrays in brillig have copy on write semantics which relies on us incrementing their
//! reference counts when they are shared in multiple places. Note that while Noir has references,
//! arrays can also be shared by value and we want to avoid clones when possible. This pass
//! clones arrays (increments their reference counts) in the following situations:
//! - Function parameters:
//!   - Any arrays behind a mutable reference `&mut [T; N]` will have their reference count
//!     incremented iff there was already a prior array of the same type passed into the same
//!     function. E.g. if there are two parameters of type `&mut [Field; 3]` we increment only
//!     the later. If there are 3 we increment the last two.
//!     - This applies within struct & tuple types as well. If a function only takes 1 struct
//!       parameter but that struct contains 2 or more mutable references to the same array
//!       type, we increment the reference count of each instance of the type after the first.
//!     - In the case of references to nested arrays, only the outer array has its reference count incremented.
//!   - Arrays taken by mutable value are always cloned, e.g. in `mut x: [u32; 3]`, `x` will
//!     have its reference count incremented.
//! - Let bindings (`let _ = <expression which returns an array>;`):
//!   - Binding an array to a let binding increments the reference count of the array unless
//!     the expression is an array literal in which case it is considered to be moved.
//! - Assignments (`x = <expression which returns an array>;`):
//!   - Similarly, assigning an array to an existing variable will also increment the reference
//!     count of the array unless it is an array literal.
//! - Array literals:
//!   - Arrays stored inside a nested array literal (e.g. both variables in `[array1, array2]`
//!     have their reference count incremented).
//!   - This does not apply to nested array literals since we know they are not referenced elsewhere.
//! - Extracting an array from another array (`let inner: [_; _] = array[0];`):
//!   - Extracting a nested array from its outer array will always increment the reference count
//!     of the nested array.
//!
//! Additionally we currently only decrement reference counts at the end of the function when
//! a parameter goes out of scope. These means reference counts likely trend upward over time
//! until the array is eventually mutated and it is reset back to 1.
use crate::{
    ast::UnaryOp,
    monomorphization::ast::{
        Definition, Expression, Function, Ident, LValue, Literal, Parameters, Program, Type, Unary,
    },
};

use fxhash::FxHashSet as HashSet;
use noirc_errors::Location;

impl Program {
    pub(crate) fn handle_ownership(mut self) -> Self {
        let mut context = Context {};

        for function in self.functions.iter_mut() {
            context.handle_ownership_in_function(function);
        }

        self
    }
}

struct Context {}

impl Context {
    fn handle_ownership_in_function(&mut self, function: &mut Function) {
        if !function.unconstrained {
            return;
        }

        let mut new_clones = self.increment_parameter_rcs(&function.parameters);
        self.handle_expression(&mut function.body);

        // Prepend new_clones to the function body
        if !new_clones.is_empty() {
            let unit = Expression::Literal(Literal::Unit);
            let old_body = std::mem::replace(&mut function.body, unit);
            new_clones.push(old_body);
            function.body = Expression::Block(new_clones);
        }
    }

    /// Increment any parameter reference counts necessary. Returns a vector of new
    /// clones to prepend to a function - if any.
    fn increment_parameter_rcs(&mut self, parameters: &Parameters) -> Vec<Expression> {
        let mut seen_array_types = HashSet::default();
        let mut new_clones = Vec::new();

        for (parameter_id, mutable, name, parameter_type) in parameters {
            let parameter = Expression::Ident(Ident {
                location: None,
                definition: Definition::Local(*parameter_id),
                mutable: *mutable,
                name: name.clone(),
                typ: parameter_type.clone(),
            });
            self.recur_on_parameter(
                parameter,
                parameter_type,
                &mut seen_array_types,
                &mut new_clones,
            );
        }

        new_clones
    }

    fn recur_on_parameter<'typ>(
        &mut self,
        parameter: Expression,
        parameter_type: &'typ Type,
        seen_array_types: &mut HashSet<&'typ Type>,
        new_clones: &mut Vec<Expression>,
    ) {
        match parameter_type {
            // These types never contain arrays
            Type::Field
            | Type::Integer(..)
            | Type::Bool
            | Type::Unit
            | Type::Function(..)
            | Type::Array(..)
            | Type::String(_)
            | Type::FmtString(..)
            | Type::Slice(_) => (),

            Type::Tuple(fields) => {
                for (i, field) in fields.iter().enumerate() {
                    let expr = Expression::ExtractTupleField(Box::new(parameter.clone()), i);
                    self.recur_on_parameter(expr, field, seen_array_types, new_clones);
                }
            }

            Type::Reference(element_type) => {
                if let Some(array) = Self::inner_array_or_slice_type(&element_type) {
                    // Check if the parameter type has been seen before
                    if !seen_array_types.insert(array) {
                        let expr = Expression::Unary(Unary {
                            operator: UnaryOp::Dereference { implicitly_added: true },
                            rhs: Box::new(parameter),
                            result_type: element_type.as_ref().clone(),
                            location: Location::dummy(), // TODO
                        });
                        new_clones.push(Expression::Clone(Box::new(expr)));
                    }
                }
            }
        }
    }

    /// Return the inner array, slice, string, or fmtstring type if it exists.
    fn inner_array_or_slice_type(typ: &Type) -> Option<&Type> {
        match typ {
            Type::Field | Type::Integer(..) | Type::Bool | Type::Function(..) | Type::Unit => None,

            Type::Array(_, _) | Type::Slice(_) | Type::String(_) | Type::FmtString(..) => Some(typ),

            Type::Tuple(elems) => {
                for elem in elems {
                    if let Some(target) = Self::inner_array_or_slice_type(elem) {
                        return Some(target);
                    }
                }
                None
            }

            // The existing SSA code still checked nested references for
            // array types. These probably shouldn't be needed for cloning
            // purposes but are kept for now to avoid differences with the existing code.
            Type::Reference(element) => Self::inner_array_or_slice_type(element),
        }
    }

    fn handle_expression(&mut self, expr: &mut Expression) {
        match expr {
            Expression::Ident(_) => (),
            Expression::Literal(literal) => self.handle_literal(literal),
            Expression::Block(exprs) => {
                exprs.iter_mut().for_each(|expr| self.handle_expression(expr))
            }
            Expression::Unary(unary) => self.handle_unary(unary),
            Expression::Binary(binary) => self.handle_binary(binary),
            Expression::Index(_) => self.handle_index(expr),
            Expression::Cast(cast) => self.handle_cast(cast),
            Expression::For(for_expr) => self.handle_for(for_expr),
            Expression::Loop(loop_expr) => self.handle_expression(loop_expr),
            Expression::While(while_expr) => self.handle_while(while_expr),
            Expression::If(if_expr) => self.handle_if(if_expr),
            Expression::Match(match_expr) => self.handle_match(match_expr),
            Expression::Tuple(elems) => self.handle_tuple(elems),
            Expression::ExtractTupleField(tuple, _index) => self.handle_expression(tuple),
            Expression::Call(call) => self.handle_call(call),
            Expression::Let(let_expr) => self.handle_let(let_expr),
            Expression::Constrain(boolean, _location, msg) => self.handle_constrain(boolean, msg),
            Expression::Assign(assign) => self.handle_assign(assign),
            Expression::Semi(expr) => self.handle_expression(expr),
            // Clones are only inserted by this pass so we can assume any code they contain is
            // already handled
            Expression::Clone(_) => (),
            Expression::Break => (),
            Expression::Continue => (),
        }
    }

    /// - Array literals:
    ///   - Arrays stored inside a nested array literal (e.g. both variables in `[array1, array2]`
    ///     have their reference count incremented).
    ///   - This does not apply to nested array literals since we know they are not referenced elsewhere.
    fn handle_literal(&mut self, literal: &mut Literal) {
        match literal {
            Literal::Integer(..) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => (),

            Literal::FmtStr(_, _, captures) => self.handle_expression(captures),

            Literal::Array(array) | Literal::Slice(array) => {
                let element_type = array
                    .typ
                    .array_element_type()
                    .expect("Array literal should have an array type");
                if contains_array_or_str_type(element_type) {
                    // We have to clone nested arrays unless they are array literals
                    for element in array.contents.iter_mut() {
                        if !is_array_or_str_literal(element) {
                            clone_expr(element);
                        }
                    }
                }
            }
        }
    }

    fn handle_unary(&mut self, unary: &mut Unary) {
        self.handle_expression(&mut unary.rhs);
    }

    fn handle_binary(&mut self, binary: &mut crate::monomorphization::ast::Binary) {
        self.handle_expression(&mut binary.lhs);
        self.handle_expression(&mut binary.rhs);
    }

    /// - Extracting an array from another array (`let inner: [_; _] = array[0];`):
    ///   - Extracting a nested array from its outer array will always increment the reference count
    ///     of the nested array.
    fn handle_index(&mut self, index_expr: &mut Expression) {
        let crate::monomorphization::ast::Expression::Index(index) = index_expr else {
            panic!("handle_index should only be called with Index nodes");
        };

        self.handle_expression(&mut index.collection);
        self.handle_expression(&mut index.index);

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
        for case in &mut match_expr.cases {
            self.handle_expression(&mut case.branch);
        }

        if let Some(default_case) = &mut match_expr.default_case {
            self.handle_expression(default_case);
        }
    }

    fn handle_tuple(&mut self, elems: &mut [Expression]) {
        for elem in elems {
            self.handle_expression(elem);
        }
    }

    fn handle_call(&mut self, call: &mut crate::monomorphization::ast::Call) {
        self.handle_expression(&mut call.func);
        for arg in &mut call.arguments {
            self.handle_expression(arg);
        }
    }

    /// - Let bindings (`let _ = <expression which returns an array>;`):
    ///   - Binding an array to a let binding increments the reference count of the array unless
    ///     the expression is an array literal in which case it is considered to be moved.
    fn handle_let(&mut self, let_expr: &mut crate::monomorphization::ast::Let) {
        self.handle_expression(&mut let_expr.expression);

        if !is_array_or_str_literal(&let_expr.expression) {
            clone_expr(&mut let_expr.expression);
        }
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

    /// - Assignments (`x = <expression which returns an array>;`):
    ///   - Assigning an array to an existing variable will also increment the reference
    ///     count of the array unless it is an array literal.
    fn handle_assign(&mut self, assign: &mut crate::monomorphization::ast::Assign) {
        self.handle_lvalue(&mut assign.lvalue);
        self.handle_expression(&mut assign.expression);

        if !is_array_or_str_literal(&assign.expression) {
            clone_expr(&mut assign.expression);
        }
    }

    fn handle_lvalue(&mut self, lvalue: &mut LValue) {
        match lvalue {
            LValue::Ident(_) => (),
            LValue::Index { array, index, element_type: _, location: _ } => {
                self.handle_expression(index);
                self.handle_lvalue(array);
            }
            LValue::MemberAccess { object, field_index: _ } => {
                self.handle_lvalue(object);
            }
            LValue::Dereference { reference, element_type: _ } => {
                self.handle_lvalue(reference);
            }
        }
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

fn is_array_or_str_literal(expr: &Expression) -> bool {
    match expr {
        Expression::Literal(literal) => match literal {
            Literal::Integer(..) | Literal::Bool(_) | Literal::Unit => false,

            Literal::Array(_) | Literal::Slice(_) | Literal::Str(_) | Literal::FmtStr(..) => true,
        },
        Expression::Block(exprs) => {
            if let Some(expr) = exprs.last() {
                is_array_or_str_literal(expr)
            } else {
                false
            }
        }

        Expression::Unary(_)
        | Expression::Ident(_)
        | Expression::Binary(_)
        | Expression::Index(_)
        | Expression::Cast(_)
        | Expression::For(_)
        | Expression::Loop(_)
        | Expression::While(_)
        | Expression::If(_)
        | Expression::Match(_)
        | Expression::Tuple(_)
        | Expression::ExtractTupleField(_, _)
        | Expression::Call(_)
        | Expression::Let(_)
        | Expression::Constrain(..)
        | Expression::Assign(_)
        | Expression::Semi(_)
        | Expression::Clone(_)
        | Expression::Break
        | Expression::Continue => false,
    }
}

fn contains_array_or_str_type(typ: &Type) -> bool {
    match typ {
        Type::Field | Type::Integer(..) | Type::Bool | Type::Unit | Type::Function(..) => false,

        Type::Array(_, _) | Type::String(_) | Type::FmtString(_, _) | Type::Slice(_) => true,

        Type::Tuple(elems) => elems.iter().any(contains_array_or_str_type),
        Type::Reference(elem) => contains_array_or_str_type(elem),
    }
}
