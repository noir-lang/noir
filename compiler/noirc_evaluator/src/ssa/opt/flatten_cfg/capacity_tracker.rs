use crate::ssa::ir::{
    dfg::DataFlowGraph,
    instruction::{Instruction, Intrinsic},
    types::Type,
    value::{Value, ValueId},
};

use fxhash::FxHashMap as HashMap;

pub(crate) struct SliceCapacityTracker<'a> {
    dfg: &'a DataFlowGraph,
}

impl<'a> SliceCapacityTracker<'a> {
    pub(crate) fn new(dfg: &'a DataFlowGraph) -> Self {
        SliceCapacityTracker { dfg }
    }

    /// Determine how the slice sizes map needs to be updated according to the provided instruction.
    pub(crate) fn collect_slice_information(
        &mut self,
        instruction: &Instruction,
        slice_sizes: &mut HashMap<ValueId, usize>,
        results: Vec<ValueId>,
    ) {
        match instruction {
            Instruction::ArrayGet { array, .. } => {
                let array_typ = self.dfg.type_of_value(*array);
                let array_value = &self.dfg[*array];
                if matches!(array_value, Value::Array { .. }) && array_typ.contains_slice_element()
                {
                    // Initial insertion into the slice sizes map
                    // Any other insertions should only occur if the value is already
                    // a part of the map.
                    self.compute_slice_capacity(*array, slice_sizes);
                }
            }
            Instruction::ArraySet { array, value, .. } => {
                let array_typ = self.dfg.type_of_value(*array);
                let array_value = &self.dfg[*array];
                if matches!(array_value, Value::Array { .. }) && array_typ.contains_slice_element()
                {
                    // Initial insertion into the slice sizes map
                    // Any other insertions should only occur if the value is already
                    // a part of the map.
                    self.compute_slice_capacity(*array, slice_sizes);
                }

                let value_typ = self.dfg.type_of_value(*value);
                // Compiler sanity check
                assert!(!value_typ.contains_slice_element(), "ICE: Nested slices are not allowed and should not have reached the flattening pass of SSA");

                if let Some(capacity) = slice_sizes.get(array) {
                    slice_sizes.insert(results[0], *capacity);
                }
            }
            Instruction::Call { func, arguments } => {
                let func = &self.dfg[*func];
                if let Value::Intrinsic(intrinsic) = func {
                    let (argument_index, result_index) = match intrinsic {
                        Intrinsic::SlicePushBack
                        | Intrinsic::SlicePushFront
                        | Intrinsic::SlicePopBack
                        | Intrinsic::SliceInsert
                        | Intrinsic::SliceRemove => (1, 1),
                        // `pop_front` returns the popped element, and then the respective slice.
                        // This means in the case of a slice with structs, the result index of the popped slice
                        // will change depending on the number of elements in the struct.
                        // For example, a slice with four elements will look as such in SSA:
                        // v3, v4, v5, v6, v7, v8 = call slice_pop_front(v1, v2)
                        // where v7 is the slice length and v8 is the popped slice itself.
                        Intrinsic::SlicePopFront => (1, results.len() - 1),
                        _ => return,
                    };
                    let slice_contents = arguments[argument_index];
                    match intrinsic {
                        Intrinsic::SlicePushBack
                        | Intrinsic::SlicePushFront
                        | Intrinsic::SliceInsert => {
                            for arg in &arguments[(argument_index + 1)..] {
                                let element_typ = self.dfg.type_of_value(*arg);
                                if element_typ.contains_slice_element() {
                                    self.compute_slice_capacity(*arg, slice_sizes);
                                }
                            }
                            if let Some(contents_capacity) = slice_sizes.get(&slice_contents) {
                                let new_capacity = *contents_capacity + 1;
                                slice_sizes.insert(results[result_index], new_capacity);
                            }
                        }
                        Intrinsic::SlicePopBack
                        | Intrinsic::SliceRemove
                        | Intrinsic::SlicePopFront => {
                            // We do not decrement the size on intrinsics that could remove values from a slice.
                            // This is because we could potentially go back to the smaller slice and not fill in dummies.
                            // This pass should be tracking the potential max that a slice ***could be***
                            if let Some(contents_capacity) = slice_sizes.get(&slice_contents) {
                                let new_capacity = *contents_capacity - 1;
                                slice_sizes.insert(results[result_index], new_capacity);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Instruction::Store { address, value } => {
                let value_typ = self.dfg.type_of_value(*value);
                if value_typ.contains_slice_element() {
                    self.compute_slice_capacity(*value, slice_sizes);

                    let value_capacity = slice_sizes.get(value).unwrap_or_else(|| {
                        panic!("ICE: should have slice capacity set for value {value} being stored at {address}")
                    });

                    slice_sizes.insert(*address, *value_capacity);
                }
            }
            Instruction::Load { address } => {
                let load_typ = self.dfg.type_of_value(*address);
                if load_typ.contains_slice_element() {
                    let result = results[0];

                    let address_capacity = slice_sizes.get(address).unwrap_or_else(|| {
                        panic!("ICE: should have slice capacity set at address {address} being loaded into {result}")
                    });

                    slice_sizes.insert(result, *address_capacity);
                }
            }
            _ => {}
        }
    }

    /// Computes the starting capacity of a slice which is still a `Value::Array`
    pub(crate) fn compute_slice_capacity(
        &self,
        array_id: ValueId,
        slice_sizes: &mut HashMap<ValueId, usize>,
    ) {
        if let Value::Array { array, typ } = &self.dfg[array_id] {
            // Compiler sanity check
            assert!(!typ.is_nested_slice(), "ICE: Nested slices are not allowed and should not have reached the flattening pass of SSA");
            if let Type::Slice(_) = typ {
                let element_size = typ.element_size();
                let len = array.len() / element_size;
                slice_sizes.insert(array_id, len);
            }
        }
    }
}
