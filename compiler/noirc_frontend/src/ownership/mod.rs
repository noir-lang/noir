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
        Definition, Expression, Function, Ident, LValue, Let, Literal, LocalId, Parameters,
        Program, Type, Unary,
    },
};

use fxhash::FxHashSet as HashSet;
use noirc_errors::Location;

impl Program {
    pub(crate) fn handle_ownership(mut self, mut next_local_id: u32) -> Self {
        for function in self.functions.iter_mut() {
            handle_ownership_in_function(function, &mut next_local_id);
        }

        self
    }
}

fn handle_ownership_in_function(function: &mut Function, local_id: &mut u32) {
    if !function.unconstrained {
        return;
    }

    let new_bindings = collect_parameters_to_clone(&function.parameters);
    handle_expression(&mut function.body);

    // Prepend new_bindings to the function body and insert drops for them at the end.
    if !new_bindings.is_empty() {
        let unit = Expression::Literal(Literal::Unit);
        let old_body = std::mem::replace(&mut function.body, unit);

        // Store anything we want to clone in let bindings first so when we later drop
        // them we know we're dropping the same instance rather than a fresh copy.
        let (mut new_body, new_idents) = create_let_bindings(new_bindings, local_id);

        // Now push the clones for each parameter
        for new_ident in &new_idents {
            new_body.push(Expression::Clone(Box::new(new_ident.clone())));
        }

        // Insert a `let` for the returned value so we can insert drops after it
        let return_id = next_local_id(local_id);
        let return_let = Expression::Let(Let {
            id: return_id,
            mutable: false,
            name: "return".to_string(),
            expression: Box::new(old_body),
        });

        new_body.push(return_let);

        // Now drop each parameter we cloned
        for new_ident in new_idents {
            new_body.push(Expression::Drop(Box::new(new_ident)));
        }

        // Finally, return the original return value we held on to
        new_body.push(Expression::Ident(Ident {
            location: None,
            definition: Definition::Local(return_id),
            mutable: false,
            name: "return".to_string(),
            typ: function.return_type.clone(),
        }));

        function.body = Expression::Block(new_body);
    }
}

fn create_let_bindings(
    bindings_to_create: Vec<(String, Type, Expression)>,
    current_local_id: &mut u32,
) -> (Vec<Expression>, Vec<Expression>) {
    let mut bindings = Vec::with_capacity(bindings_to_create.len());
    let mut idents = Vec::with_capacity(bindings_to_create.len());

    for (name, typ, expression) in bindings_to_create {
        let id = next_local_id(current_local_id);
        let expression = Box::new(expression);
        bindings.push(Expression::Let(Let { id, mutable: false, name: String::new(), expression }));

        idents.push(Expression::Ident(Ident {
            location: None,
            definition: Definition::Local(id),
            mutable: false,
            name,
            typ,
        }));
    }

    (bindings, idents)
}

fn next_local_id(current_local_id: &mut u32) -> LocalId {
    let next = *current_local_id;
    *current_local_id += 1;
    LocalId(next)
}

/// Returns a vector of new parameters to prepend clones to a function - if any.
/// Note that these may be full expressions e.g. `*param.field` so they should
/// be stored in a let binding before being cloned to ensure that a later drop
/// would be to the same value.
fn collect_parameters_to_clone(parameters: &Parameters) -> Vec<(String, Type, Expression)> {
    let mut seen_array_types = HashSet::default();
    let mut new_bindings = Vec::new();

    for (parameter_id, mutable, name, parameter_type) in parameters {
        let parameter = Expression::Ident(Ident {
            location: None,
            definition: Definition::Local(*parameter_id),
            mutable: *mutable,
            name: name.clone(),
            typ: parameter_type.clone(),
        });

        // (by-value) Mutable parameters are always cloned. Otherwise, we have to recur on the type
        // to find a duplicate array types behind mutable references.
        let parameter = if *mutable {
            let name = name.clone();
            new_bindings.push((name, parameter_type.clone(), parameter));
            // disable cloning in recur_on_parameter, we already cloned
            None
        } else {
            Some(parameter)
        };

        recur_on_parameter(
            parameter,
            parameter_type,
            name,
            &mut seen_array_types,
            &mut new_bindings,
            false,
        );
    }

    new_bindings
}

/// Recur on a parameter's type, digging into any struct fields, looking for references to arrays.
/// This will build up an Expression of the current parameter access we're doing, e.g. `*foo.bar`
/// would correspond to a parameter `foo` with struct field `bar` that is a reference to an array.
///
/// This function inserts a .clone() expression to any parameter arrays behind references with
/// repeated types since these may potentially be aliased by other parameters.
///
/// If `parameter` is `None` we'll still traverse the type to find any array types mentioned but we
/// will not issue any clones. This is required e.g. on a mutable by-value parameter like `mut x: ...`
/// since `x` may contain arrays internally that we'll need to remember in case there is another
/// parameter which uses them. E.g. `mut x: [Field; 2], y: &mut [Field; 2]`.
fn recur_on_parameter<'typ>(
    parameter: Option<Expression>,
    parameter_type: &'typ Type,
    parameter_name: &str,
    seen_array_types: &mut HashSet<&'typ Type>,
    new_bindings: &mut Vec<(String, Type, Expression)>,
    passed_reference: bool,
) {
    match parameter_type {
        // These types never contain arrays
        Type::Field | Type::Integer(..) | Type::Bool | Type::Unit | Type::Function(..) => (),

        Type::Array(..) | Type::Slice(_) | Type::String(_) | Type::FmtString(..) => {
            // If we've already seen this type and this is behind a reference
            if !seen_array_types.insert(parameter_type) && passed_reference {
                if let Some(parameter) = parameter {
                    new_bindings.push((
                        parameter_name.to_string(),
                        parameter_type.clone(),
                        parameter,
                    ));
                }
            }

            // Don't recur on the element type here, we rely on the reference count to already be
            // incremented in the nested array case when the nested array is created.
        }

        Type::Tuple(fields) => {
            for (i, field) in fields.iter().enumerate() {
                let expr = parameter.clone().map(|p| Expression::ExtractTupleField(Box::new(p), i));
                recur_on_parameter(
                    expr,
                    field,
                    parameter_name,
                    seen_array_types,
                    new_bindings,
                    passed_reference,
                );
            }
        }

        Type::Reference(element_type, _mutable) => {
            let expr = parameter.map(|parameter| {
                Expression::Unary(Unary {
                    operator: UnaryOp::Dereference { implicitly_added: true },
                    rhs: Box::new(parameter.clone()),
                    result_type: element_type.as_ref().clone(),
                    location: Location::dummy(), // TODO
                })
            });
            recur_on_parameter(
                expr,
                element_type,
                parameter_name,
                seen_array_types,
                new_bindings,
                true,
            );
        }
    }
}

fn handle_expression(expr: &mut Expression) {
    match expr {
        Expression::Ident(_) => (),
        Expression::Literal(literal) => handle_literal(literal),
        Expression::Block(exprs) => {
            exprs.iter_mut().for_each(handle_expression);
        }
        Expression::Unary(unary) => handle_unary(unary),
        Expression::Binary(binary) => handle_binary(binary),
        Expression::Index(_) => handle_index(expr),
        Expression::Cast(cast) => handle_cast(cast),
        Expression::For(for_expr) => handle_for(for_expr),
        Expression::Loop(loop_expr) => handle_expression(loop_expr),
        Expression::While(while_expr) => handle_while(while_expr),
        Expression::If(if_expr) => handle_if(if_expr),
        Expression::Match(match_expr) => handle_match(match_expr),
        Expression::Tuple(elements) => handle_tuple(elements),
        Expression::ExtractTupleField(tuple, _index) => handle_expression(tuple),
        Expression::Call(call) => handle_call(call),
        Expression::Let(let_expr) => handle_let(let_expr),
        Expression::Constrain(boolean, _location, msg) => handle_constrain(boolean, msg),
        Expression::Assign(assign) => handle_assign(assign),
        Expression::Semi(expr) => handle_expression(expr),
        // Clones & Drops are only inserted by this pass so we can assume any code they
        // contain is already handled
        Expression::Clone(_) => (),
        Expression::Drop(_) => (),
        Expression::Break => (),
        Expression::Continue => (),
    }
}

/// - Array literals:
///   - Arrays stored inside a nested array literal (e.g. both variables in `[array1, array2]`
///     have their reference count incremented).
///   - This does not apply to nested array literals since we know they are not referenced elsewhere.
fn handle_literal(literal: &mut Literal) {
    match literal {
        Literal::Integer(..) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => (),

        Literal::FmtStr(_, _, captures) => handle_expression(captures),

        Literal::Array(array) | Literal::Slice(array) => {
            let element_type =
                array.typ.array_element_type().expect("Array literal should have an array type");
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

fn handle_unary(unary: &mut Unary) {
    handle_expression(&mut unary.rhs);
}

fn handle_binary(binary: &mut crate::monomorphization::ast::Binary) {
    handle_expression(&mut binary.lhs);
    handle_expression(&mut binary.rhs);
}

/// - Extracting an array from another array (`let inner: [_; _] = array[0];`):
///   - Extracting a nested array from its outer array will always increment the reference count
///     of the nested array.
fn handle_index(index_expr: &mut Expression) {
    let crate::monomorphization::ast::Expression::Index(index) = index_expr else {
        panic!("handle_index should only be called with Index nodes");
    };

    handle_expression(&mut index.collection);
    handle_expression(&mut index.index);

    if contains_array_or_str_type(&index.element_type) {
        clone_expr(index_expr);
    }
}

fn handle_cast(cast: &mut crate::monomorphization::ast::Cast) {
    handle_expression(&mut cast.lhs);
}

fn handle_for(for_expr: &mut crate::monomorphization::ast::For) {
    handle_expression(&mut for_expr.start_range);
    handle_expression(&mut for_expr.end_range);
    handle_expression(&mut for_expr.block);
}

fn handle_while(while_expr: &mut crate::monomorphization::ast::While) {
    handle_expression(&mut while_expr.condition);
    handle_expression(&mut while_expr.body);
}

fn handle_if(if_expr: &mut crate::monomorphization::ast::If) {
    handle_expression(&mut if_expr.condition);
    handle_expression(&mut if_expr.consequence);
    if let Some(alt) = &mut if_expr.alternative {
        handle_expression(alt);
    }
}

fn handle_match(match_expr: &mut crate::monomorphization::ast::Match) {
    for case in &mut match_expr.cases {
        handle_expression(&mut case.branch);
    }

    if let Some(default_case) = &mut match_expr.default_case {
        handle_expression(default_case);
    }
}

fn handle_tuple(elems: &mut [Expression]) {
    for elem in elems {
        handle_expression(elem);
    }
}

fn handle_call(call: &mut crate::monomorphization::ast::Call) {
    handle_expression(&mut call.func);
    for arg in &mut call.arguments {
        handle_expression(arg);
    }
}

/// - Let bindings (`let _ = <expression which returns an array>;`):
///   - Binding an array to a let binding increments the reference count of the array unless
///     the expression is an array literal in which case it is considered to be moved.
fn handle_let(let_expr: &mut crate::monomorphization::ast::Let) {
    handle_expression(&mut let_expr.expression);

    if !is_array_or_str_literal(&let_expr.expression) {
        clone_expr(&mut let_expr.expression);
    }
}

fn handle_constrain(
    boolean: &mut Expression,
    msg: &mut Option<Box<(Expression, crate::hir_def::types::Type)>>,
) {
    handle_expression(boolean);

    if let Some(msg) = msg {
        handle_expression(&mut msg.0);
    }
}

/// - Assignments (`x = <expression which returns an array>;`):
///   - Assigning an array to an existing variable will also increment the reference
///     count of the array unless it is an array literal.
fn handle_assign(assign: &mut crate::monomorphization::ast::Assign) {
    handle_lvalue(&mut assign.lvalue);
    handle_expression(&mut assign.expression);

    if !is_array_or_str_literal(&assign.expression) {
        clone_expr(&mut assign.expression);
    }
}

fn handle_lvalue(lvalue: &mut LValue) {
    match lvalue {
        LValue::Ident(_) => (),
        LValue::Index { array, index, element_type: _, location: _ } => {
            handle_expression(index);
            handle_lvalue(array);
        }
        LValue::MemberAccess { object, field_index: _ } => {
            handle_lvalue(object);
        }
        LValue::Dereference { reference, element_type: _ } => {
            handle_lvalue(reference);
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
        | Expression::Drop(_)
        | Expression::Break
        | Expression::Continue => false,
    }
}

fn contains_array_or_str_type(typ: &Type) -> bool {
    match typ {
        Type::Field | Type::Integer(..) | Type::Bool | Type::Unit | Type::Function(..) => false,

        Type::Array(_, _) | Type::String(_) | Type::FmtString(_, _) | Type::Slice(_) => true,

        Type::Tuple(elems) => elems.iter().any(contains_array_or_str_type),
        Type::Reference(elem, _) => contains_array_or_str_type(elem),
    }
}
