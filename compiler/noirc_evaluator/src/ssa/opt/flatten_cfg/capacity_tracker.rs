use crate::ssa::ir::{
    dfg::DataFlowGraph,
    instruction::{Instruction, Intrinsic},
    types::Type,
    value::{Value, ValueId},
};

use acvm::{acir::AcirField, FieldElement};
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
        &self,
        instruction: &Instruction,
        slice_sizes: &mut HashMap<ValueId, u32>,
        results: &[ValueId],
    ) {
        match instruction {
            Instruction::ArrayGet { array, .. } => {
                if let Some((_, array_type)) = self.dfg.get_array_constant(*array) {
                    if array_type.contains_slice_element() {
                        // Initial insertion into the slice sizes map
                        // Any other insertions should only occur if the value is already
                        // a part of the map.
                        self.compute_slice_capacity(*array, slice_sizes);
                    }
                }
            }
            Instruction::ArraySet { array, value, .. } => {
                if let Some((_, array_type)) = self.dfg.get_array_constant(*array) {
                    if array_type.contains_slice_element() {
                        // Initial insertion into the slice sizes map
                        // Any other insertions should only occur if the value is already
                        // a part of the map.
                        self.compute_slice_capacity(*array, slice_sizes);
                    }
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
                        Intrinsic::AsSlice => (0, 1),
                        _ => return,
                    };
                    let result_slice = results[result_index];
                    match intrinsic {
                        Intrinsic::SlicePushBack
                        | Intrinsic::SlicePushFront
                        | Intrinsic::SliceInsert => {
                            let slice_contents = arguments[argument_index];

                            for arg in &arguments[(argument_index + 1)..] {
                                let element_typ = self.dfg.type_of_value(*arg);
                                if element_typ.contains_slice_element() {
                                    self.compute_slice_capacity(*arg, slice_sizes);
                                }
                            }

                            if let Some(contents_capacity) = slice_sizes.get(&slice_contents) {
                                let new_capacity = *contents_capacity + 1;
                                slice_sizes.insert(result_slice, new_capacity);
                            }
                        }
                        Intrinsic::SlicePopBack
                        | Intrinsic::SliceRemove
                        | Intrinsic::SlicePopFront => {
                            let slice_contents = arguments[argument_index];

                            if let Some(contents_capacity) = slice_sizes.get(&slice_contents) {
                                // We use a saturating sub here as calling `pop_front` or `pop_back`
                                // on a zero-length slice would otherwise underflow.
                                let new_capacity = contents_capacity.saturating_sub(1);
                                slice_sizes.insert(result_slice, new_capacity);
                            }
                        }
                        Intrinsic::ToBits(_) => {
                            // Compiler sanity check
                            assert!(matches!(self.dfg.type_of_value(result_slice), Type::Slice(_)));
                            slice_sizes.insert(result_slice, FieldElement::max_num_bits());
                        }
                        Intrinsic::ToRadix(_) => {
                            // Compiler sanity check
                            assert!(matches!(self.dfg.type_of_value(result_slice), Type::Slice(_)));
                            slice_sizes.insert(result_slice, FieldElement::max_num_bytes());
                        }
                        Intrinsic::AsSlice => {
                            let array_size = self
                                .dfg
                                .try_get_array_length(arguments[argument_index])
                                .expect("ICE: Should be have an array length for AsSlice input");
                            slice_sizes.insert(result_slice, array_size);
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
        slice_sizes: &mut HashMap<ValueId, u32>,
    ) {
        if let Some((array, typ)) = self.dfg.get_array_constant(array_id) {
            // Compiler sanity check
            assert!(!typ.is_nested_slice(), "ICE: Nested slices are not allowed and should not have reached the flattening pass of SSA");
            if let Type::Slice(_) = typ {
                let element_size = typ.element_size();
                let len = array.len() / element_size;
                slice_sizes.insert(array_id, len as u32);
            }
        }
    }
}
