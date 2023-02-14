use crate::ssa::{
    acir_gen::InternalVar,
    context::SsaContext,
    mem::{ArrayId, MemArray},
};
use acvm::acir::native_types::Witness;
use iter_extended::vecmap;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct ArrayHeap {
    // maps memory address to InternalVar
    memory_map: BTreeMap<u32, InternalVar>,
}

/// Handle virtual memory access
#[derive(Default)]
pub struct AcirMem {
    virtual_memory: BTreeMap<ArrayId, ArrayHeap>,
}

impl AcirMem {
    // Returns the memory_map for the array
    fn array_map_mut(&mut self, array_id: ArrayId) -> &mut BTreeMap<u32, InternalVar> {
        &mut self.virtual_memory.entry(array_id).or_default().memory_map
    }

    // Write the value to the array's VM at the specified index
    pub fn insert(&mut self, array_id: ArrayId, index: u32, value: InternalVar) {
        self.array_map_mut(array_id).insert(index, value);
    }

    //Map the outputs into the array
    pub(crate) fn map_array(&mut self, a: ArrayId, outputs: &[Witness], ctx: &SsaContext) {
        let array = &ctx.mem[a];
        for i in 0..array.len {
            let var = if i < outputs.len() as u32 {
                InternalVar::from(outputs[i as usize])
            } else {
                InternalVar::zero_expr()
            };
            self.array_map_mut(array.id).insert(i, var);
        }
    }

    // Load array values into InternalVars
    // If create_witness is true, we create witnesses for values that do not have witness
    pub(crate) fn load_array(&mut self, array: &MemArray) -> Vec<InternalVar> {
        vecmap(0..array.len, |offset| {
            self.load_array_element_constant_index(array, offset)
                .expect("infallible: array out of bounds error")
        })
    }

    // Loads the associated `InternalVar` for the element
    // in the `array` at the given `offset`.
    //
    // We check if the address of the array element
    // is in the memory_map.
    //
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

        // Check the memory_map to see if the element is there
        let array_element = self
            .array_map_mut(array.id)
            .get(&offset)
            .expect("ICE: Could not find value at index {offset}");
        Some(array_element.clone())
    }
}
