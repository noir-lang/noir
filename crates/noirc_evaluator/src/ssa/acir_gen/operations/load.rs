use acvm::{acir::native_types::Expression, FieldElement};
use noirc_errors::Location;

use crate::{
    errors::{RuntimeError, RuntimeErrorKind},
    ssa::{
        acir_gen::{acir_mem::AcirMem, internal_var_cache::InternalVarCache, InternalVar},
        context::SsaContext,
        mem::{self, ArrayId},
        node::NodeId,
    },
    Evaluator,
};

// Returns a variable corresponding to the element at the provided index in the array
// Returns None if index is constant and out-of-bound.
pub(crate) fn evaluate(
    array_id: ArrayId,
    index: NodeId,
    acir_mem: &mut AcirMem,
    var_cache: &mut InternalVarCache,
    location: Option<Location>,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Result<InternalVar, RuntimeError> {
    let mem_array = &ctx.mem[array_id];
    let index = var_cache.get_or_compute_internal_var_unwrap(index, evaluator, ctx);

    if let Some(idx) = index.to_const() {
        let idx = mem::Memory::as_u32(idx);
        // Check to see if the index has gone out of bounds
        let array_length = mem_array.len;
        if idx >= array_length {
            return Err(RuntimeError {
                location,
                kind: RuntimeErrorKind::ArrayOutOfBounds {
                    index: idx as u128,
                    bound: array_length as u128,
                },
            });
        }

        let array_element = acir_mem.load_array_element_constant_index(mem_array, idx);
        if let Some(element) = array_element {
            return Ok(element);
        }
    }

    let w = evaluator.add_witness_to_cs();
    acir_mem.add_to_trace(&array_id, index.to_expression(), w.into(), Expression::zero());
    Ok(InternalVar::from_witness(w))
}

// Returns a variable corresponding to the element at the provided index in the array
// Returns None if index is out-of-bound.
pub(crate) fn evaluate_with_conts_index(
    array_id: ArrayId,
    index: u32,
    acir_mem: &mut AcirMem,
    location: Option<Location>,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Result<InternalVar, RuntimeError> {
    let mem_array = &ctx.mem[array_id];

    let array_length = mem_array.len;
    if index >= array_length {
        return Err(RuntimeError {
            location,
            kind: RuntimeErrorKind::ArrayOutOfBounds {
                index: index as u128,
                bound: array_length as u128,
            },
        });
    }

    let array_element = acir_mem.load_array_element_constant_index(mem_array, index);
    let ivar = if let Some(element) = array_element {
        element
    } else {
        let w = evaluator.add_witness_to_cs();
        acir_mem.add_to_trace(
            &array_id,
            Expression::from_field(FieldElement::from(index as i128)),
            w.into(),
            Expression::zero(),
        );
        InternalVar::from_witness(w)
    };
    Ok(ivar)
}
