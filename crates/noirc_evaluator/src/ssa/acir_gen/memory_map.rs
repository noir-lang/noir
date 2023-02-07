use crate::ssa::{
    acir_gen::InternalVar,
    context::SsaContext,
    mem::{ArrayId, MemArray},
};
use acvm::acir::native_types::Witness;
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
                InternalVar::zero_expr()
            };
            self.inner.insert(address + i, var);
        }
    }

    //Load array values into InternalVars
    //If create_witness is true, we create witnesses for values that do not have witness
    pub(crate) fn load_array(&mut self, array: &MemArray) -> Vec<InternalVar> {
        let mut array_as_internal_var = Vec::with_capacity(array.len as usize);

        for offset in 0..array.len {
            let element = self
                .load_array_element_constant_index(array, offset)
                .expect("infallible: array out of bounds error");
            array_as_internal_var.push(element.clone())
        }

        array_as_internal_var
    }

    // Loads the associated `InternalVar` for the element
    // in the `array` at the given `offset`.
    //
    // First we check if the address of the array element
    // is in the memory_map. If not, then we check the `array`
    //
    // We do not check the `MemArray` initially because the
    // `MemoryMap` holds the most updated InternalVar
    // associated to the array element.
    // TODO: specify what could change between the two?
    //
    // Returns `None` if `offset` is out of bounds.
    pub(crate) fn load_array_element_constant_index(
        &mut self,
        array: &MemArray,
        offset: u32,
    ) -> Option<InternalVar> {
        // Check to see if the index has gone out of bounds
        let array_length = array.len;
        if offset >= array_length {
            return None; // IndexOutOfBoundsError
        }

        let address_of_element = array.absolute_adr(offset);

        // Check the memory_map to see if the element is there
        if let Some(internal_var) = self.inner.get(&address_of_element) {
            return Some(internal_var.clone());
        };

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

    pub(crate) fn insert(&mut self, key: u32, value: InternalVar) -> Option<InternalVar> {
        self.inner.insert(key, value)
    }
}
