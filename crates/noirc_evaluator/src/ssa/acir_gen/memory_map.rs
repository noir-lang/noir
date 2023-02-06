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

        for offset in 0..array.len {
            // Get the address of the `i'th` element.
            // Elements in Array are assumed to be contiguous
            let address_of_element = array.absolute_adr(offset);

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

    // Loads the associated `InternalVar` for the element
    // in the `array` at the given `offset`.
    // Returns `None` if `offset` is out of bounds.
    pub(crate) fn load_array_element_constant_index(
        &mut self,
        array: &MemArray,
        offset: u32,
    ) -> Option<InternalVar> {
        let address_of_element = array.absolute_adr(offset);

        // First check the memory_map to see if the element is there
        if let Some(internal_var) = self.inner.get(&address_of_element) {
            return Some(internal_var.clone());
        };

        // Now check to see if the index has gone out of bounds
        // TODO: should we check this first?
        let array_length = array.len;
        if offset >= array_length {
            return None; // IndexOutOfBoundsError
        }

        // TODO (Guillaume) we could put the values into the memory_map when
        // TODO we process the ABI, then we only need to check the memory_map
        // TODO since this is the only case where the values will be in the
        // TODO array, but not in the `memory_map`
        let array_element = array.values[offset as usize].clone();

        // Compiler sanity check
        //
        // Since the only time we take the array values
        // from the array is when it has been defined in the
        // ABI. We know that it must have been initialized with a `Witness`
        array_element.cached_witness().expect("ICE: since the value is not in the memory_map it must have came from the ABI, so it is a Witness");

        Some(array_element)
    }

    // TODO check if we can replace usage of this method with
    // TODO `load_array_element_constant_index`.
    // TODO this is blocked by intrinsics::resolve_array
    // TODO because that method distinguishes on the case
    // TODO where the witness came from `array.values` or
    // TODO the `memory_map`
    pub(crate) fn internal_var(&self, key: &u32) -> Option<&InternalVar> {
        self.inner.get(key)
    }
    pub(crate) fn insert(&mut self, key: u32, value: InternalVar) -> Option<InternalVar> {
        self.inner.insert(key, value)
    }
}
