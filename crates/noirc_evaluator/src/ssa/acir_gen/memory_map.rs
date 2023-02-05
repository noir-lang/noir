use crate::{
    ssa::{
        acir_gen::InternalVar,
        context::SsaContext,
        mem::{ArrayId, MemArray},
    },
    Evaluator,
};
use acvm::acir::native_types::{Expression, Witness};
use std::collections::HashMap;

// maps memory address to expression
#[derive(Default)]
pub struct MemoryMap {
    inner: HashMap<u32, InternalVar>,
}

impl MemoryMap {
    //Map the outputs into the array
    pub(crate) fn map_array(&mut self, a: ArrayId, outputs: &[Witness], ctx: &SsaContext) {
        let array = &ctx.mem[a];
        let address = array.adr;
        for i in 0..array.len {
            let var = if i < outputs.len() as u32 {
                InternalVar::from(outputs[i as usize])
            } else {
                InternalVar::from(Expression::zero())
            };
            self.inner.insert(address + i, var);
        }
    }

    //Load array values into InternalVars
    //If create_witness is true, we create witnesses for values that do not have witness
    pub(crate) fn load_array(
        &mut self,
        array: &MemArray,
        create_witness: bool, // TODO: Should we remove this, since it's always false
        evaluator: &mut Evaluator,
    ) -> Vec<InternalVar> {
        let mut array_as_internal_var = Vec::with_capacity(array.len as usize);

        // Get the pointer to the start of the array
        let array_pointer = array.adr;

        for offset in 0..array.len {
            // Get the address of the `i'th` element.
            // Elements in Array are assumed to be contiguous
            let address_of_element = array_pointer + offset;

            let element = match self.inner.get_mut(&address_of_element) {
                Some(memory) => {
                    // TODO this is creating a `Witness` for a constant expression
                    // TODO Since that function only returns `None` when
                    // TODO the expression is a constant.
                    //
                    // TODO If this is not a bug, then we should pass a flag to
                    // TODO `cached_witness` to tell it when we want to create a witness
                    // TODO for a constant expression
                    if create_witness && memory.cached_witness().is_none() {
                        let witness =
                            evaluator.create_intermediate_variable(memory.expression().clone());
                        *self.inner.get_mut(&address_of_element).unwrap().cached_witness_mut() =
                            Some(witness);
                    }
                    &self.inner[&address_of_element]
                }
                // If one cannot find the associated `InternalVar` for
                // the array in the `memory_map`. Then one returns the
                // `InternalVar` in `array.values`
                // TODO this has been done in `intrinsics::resolve_array`
                // TODO also.
                // TODO - Why does one not use the `array.values` initially?
                // TODO - Can there be a discrepancy/difference between array.values and `memory_map`
                // TODO - Should we have a convenience function that does this?
                //
                None => &array.values[offset as usize],
            };

            array_as_internal_var.push(element.clone())
        }

        array_as_internal_var
    }

    pub(crate) fn internal_var(&self, key: &u32) -> Option<&InternalVar> {
        self.inner.get(key)
    }
    pub(crate) fn insert(&mut self, key: u32, value: InternalVar) -> Option<InternalVar> {
        self.inner.insert(key, value)
    }
}
