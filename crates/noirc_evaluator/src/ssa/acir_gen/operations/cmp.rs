use crate::{
    ssa::{
        acir_gen::{acir_mem::AcirMem, constraints, Acir, InternalVar},
        context::SsaContext,
        mem::{MemArray, Memory},
        node::NodeId,
    },
    Evaluator,
};
use acvm::{acir::native_types::Expression, FieldElement};
use iter_extended::vecmap;

// Given two `NodeId`s, generate constraints to check whether
// they are Equal.
//
// This method returns an `Expression` representing `0` or `1`
// If the two `NodeId`s are not equal, then the `Expression`
// returned will represent `1`, otherwise `0` is returned.
//
// A `NodeId` can represent a primitive data type
// like a `Field` or it could represent a composite type like an
// `Array`. Depending on the type, the constraints that will be generated
// will differ.
//
// TODO(check this): Types like structs are decomposed before getting to SSA
// so in reality, the NEQ instruction will be done on the fields
// of the struct
pub(super) fn evaluate_neq(
    acir_gen: &mut Acir,
    lhs: NodeId,
    rhs: NodeId,
    l_c: Option<InternalVar>,
    r_c: Option<InternalVar>,
    ctx: &SsaContext,
    evaluator: &mut Evaluator,
) -> Expression {
    // Check whether the `lhs` and `rhs` are trivially equal
    if lhs == rhs {
        return Expression::zero();
    }

    // Check whether the `lhs` and `rhs` are Arrays
    if let (Some(a), Some(b)) = (Memory::deref(ctx, lhs), Memory::deref(ctx, rhs)) {
        let array_a = &ctx.mem[a];
        let array_b = &ctx.mem[b];

        assert!(l_c.is_none());
        assert!(r_c.is_none());

        // TODO What happens if we call `l_c.expression()` on InternalVar
        // TODO when we know that they should correspond to Arrays
        // TODO(Guillaume): We can add an Option<Expression>  because
        // TODO when the object is composite, it will return One
        if array_a.len != array_b.len {
            unreachable!(
                "ICE: arrays have differing lengths {} and {}. 
                We cannot compare two different types in Noir, 
                so this should have been caught by the type checker",
                array_a.len, array_b.len
            )
        }

        let x = InternalVar::from(array_eq(&mut acir_gen.memory, array_a, array_b, evaluator));
        // TODO we need a witness because of the directive, but we should use an expression
        // TODO if we change the Invert directive to take an `Expression`, then we
        // TODO can get rid of this extra gate.
        let x_witness = acir_gen
            .var_cache
            .get_or_compute_witness(x, evaluator)
            .expect("unexpected constant expression");

        return Expression::from(constraints::evaluate_zero_equality(x_witness, evaluator));
    }

    // Arriving here means that `lhs` and `rhs` are not Arrays
    let l_c = l_c.expect("ICE: unexpected array pointer");
    let r_c = r_c.expect("ICE: unexpected array pointer");
    let x = InternalVar::from(constraints::subtract(
        l_c.expression(),
        FieldElement::one(),
        r_c.expression(),
    ));

    // Check if `x` is constant. If so, we can evaluate whether
    // it is zero at compile time.
    if let Some(x_const) = x.to_const() {
        if x_const.is_zero() {
            Expression::zero()
        } else {
            Expression::one()
        }
    } else {
        //todo we need a witness because of the directive, but we should use an expression
        let x_witness = acir_gen
            .var_cache
            .get_or_compute_witness(x, evaluator)
            .expect("unexpected constant expression");
        Expression::from(constraints::evaluate_zero_equality(x_witness, evaluator))
    }
}

pub(super) fn evaluate_eq(
    acir_gen: &mut Acir,
    lhs: NodeId,
    rhs: NodeId,
    l_c: Option<InternalVar>,
    r_c: Option<InternalVar>,
    ctx: &SsaContext,
    evaluator: &mut Evaluator,
) -> Expression {
    let neq = evaluate_neq(acir_gen, lhs, rhs, l_c, r_c, ctx, evaluator);
    constraints::subtract(&Expression::one(), FieldElement::one(), &neq)
}

// Given two `MemArray`s, generate constraints that check whether
// these two arrays are equal. An `Expression` is returned representing
// `0` if the arrays were equal and `1` otherwise.
//
// N.B. We assumes the lengths of a and b are the same but it is not checked inside the function.
fn array_eq(
    memory_map: &mut AcirMem,
    a: &MemArray,
    b: &MemArray,
    evaluator: &mut Evaluator,
) -> Expression {
    // Fetch the elements in both `MemArrays`s, these are `InternalVar`s
    // We then convert these to `Expressions`
    let internal_var_to_expr = |internal_var: InternalVar| internal_var.expression().clone();
    let a_values = vecmap(memory_map.load_array(a), internal_var_to_expr);
    let b_values = vecmap(memory_map.load_array(b), internal_var_to_expr);

    constraints::arrays_eq_predicate(&a_values, &b_values, evaluator)
}
