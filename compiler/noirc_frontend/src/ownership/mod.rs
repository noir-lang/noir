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
        Definition, Expression, Function, Ident, Literal, Parameters, Program, Type, Unary,
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
            Expression::Literal(literal) => todo!(),
            Expression::Block(exprs) => {
                exprs.iter_mut().for_each(|expr| self.handle_expression(expr))
            }
            Expression::Unary(unary) => todo!(),
            Expression::Binary(binary) => todo!(),
            Expression::Index(index) => todo!(),
            Expression::Cast(cast) => todo!(),
            Expression::For(_) => todo!(),
            Expression::Loop(expression) => todo!(),
            Expression::While(_) => todo!(),
            Expression::If(_) => todo!(),
            Expression::Match(_) => todo!(),
            Expression::Tuple(vec) => todo!(),
            Expression::ExtractTupleField(expression, _) => todo!(),
            Expression::Call(call) => todo!(),
            Expression::Let(_) => todo!(),
            Expression::Constrain(expression, location, _) => todo!(),
            Expression::Assign(assign) => todo!(),
            Expression::Semi(expr) => self.handle_expression(expr),
            Expression::Clone(_) => unreachable!("Explicit clones are only inserted by this pass"),
            Expression::Break => (),
            Expression::Continue => (),
        }
    }
}
