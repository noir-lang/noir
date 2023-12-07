use crate::ssa::ir::{dfg::DataFlowGraph, value::{ValueId, Value}, instruction::{Intrinsic, Instruction, InstructionId}, types::Type};

use fxhash::FxHashMap as HashMap;

pub(crate) struct SliceCapacityTracker<'a> {
    dfg: &'a DataFlowGraph,
    /// Maps SSA array values representing a slice's contents to its updated array value
    /// after an array set or a slice intrinsic operation.
    /// Maps original value -> result
    mapped_slice_values: HashMap<ValueId, ValueId>,

    /// Maps an updated array value following an array operation to its previous value.
    /// When used in conjunction with `mapped_slice_values` we form a two way map of all array
    /// values being used in array operations.
    /// Maps result -> original value
    slice_parents: HashMap<ValueId, ValueId>,

    // Values containing nested slices to be replaced
    slice_values: Vec<ValueId>,
}

impl<'a> SliceCapacityTracker<'a> {
    pub(crate) fn new(
        dfg: &'a DataFlowGraph,
    ) -> Self {
        SliceCapacityTracker {
            dfg,
            mapped_slice_values: HashMap::default(),
            slice_parents: HashMap::default(),
            slice_values: Vec::new(),
        }
    }

    /// Determine how the slice sizes map needs to be updated according to the provided instruction.
    pub(crate) fn collect_slice_information(
        &mut self,
        instruction: &Instruction,
        slice_sizes: &mut HashMap<ValueId, (usize, Vec<ValueId>)>,
        results: Vec<ValueId>,
    ) {
        // let (instruction, _) = self.inserter.map_instruction(instruction_id);
        // let results = self.inserter.function.dfg.instruction_results(instruction_id);
        // let instruction = &self.dfg[instruction_id];
        match instruction {
            Instruction::ArrayGet { array, .. } => {
                let array_typ = self.dfg.type_of_value(*array);
                let array_value = &self.dfg[*array];
                // If we have an SSA value containing nested slices we should mark it
                // as a slice that potentially requires to be filled with dummy data.
                if matches!(array_value, Value::Array { .. }) && array_typ.contains_slice_element()
                {
                    self.slice_values.push(*array);
                    // Initial insertion into the slice sizes map
                    // Any other insertions should only occur if the value is already
                    // a part of the map.
                    self.compute_slice_sizes(*array, slice_sizes);
                }

                let res_typ = self.dfg.type_of_value(results[0]);
                if res_typ.contains_slice_element() {
                    if let Some(inner_sizes) = slice_sizes.get_mut(array) {
                        // Include the result in the parent array potential children
                        // If the result has internal slices and is called in an array set
                        // we could potentially have a new larger slice which we need to account for
                        inner_sizes.1.push(results[0]);
                        self.slice_parents.insert(results[0], *array);

                        let inner_sizes_iter = inner_sizes.1.clone();
                        for slice_value in inner_sizes_iter {
                            let inner_slice = slice_sizes.get(&slice_value).unwrap_or_else(|| {
                                panic!("ICE: should have inner slice set for {slice_value}")
                            });
                            slice_sizes.insert(results[0], inner_slice.clone());
                            if slice_value != results[0] {
                                self.mapped_slice_values.insert(slice_value, results[0]);
                            }
                        }
                    }
                }
            }
            Instruction::ArraySet { array, value, .. } => {
                let array_typ = self.dfg.type_of_value(*array);
                let array_value = &self.dfg[*array];
                // If we have an SSA value containing nested slices we should mark it
                // as a slice that potentially requires to be filled with dummy data.
                if matches!(array_value, Value::Array { .. }) && array_typ.contains_slice_element()
                {
                    self.slice_values.push(*array);
                    // Initial insertion into the slice sizes map
                    // Any other insertions should only occur if the value is already
                    // a part of the map.
                    self.compute_slice_sizes(*array, slice_sizes);
                }

                let value_typ = self.dfg.type_of_value(*value);
                if value_typ.contains_slice_element() {
                    self.compute_slice_sizes(*value, slice_sizes);

                    let inner_sizes = slice_sizes.get_mut(array).expect("ICE expected slice sizes");
                    inner_sizes.1.push(*value);
                }

                if let Some(inner_sizes) = slice_sizes.get_mut(array) {
                    let inner_sizes = inner_sizes.clone();

                    slice_sizes.insert(results[0], inner_sizes);

                    self.mapped_slice_values.insert(*array, results[0]);
                    self.slice_parents.insert(results[0], *array);
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
                                    self.slice_values.push(*arg);
                                    self.compute_slice_sizes(*arg, slice_sizes);
                                }
                            }
                            if let Some(inner_sizes) = slice_sizes.get_mut(&slice_contents) {
                                inner_sizes.0 += 1;

                                let inner_sizes = inner_sizes.clone();
                                slice_sizes.insert(results[result_index], inner_sizes);

                                self.mapped_slice_values
                                    .insert(slice_contents, results[result_index]);
                                self.slice_parents.insert(results[result_index], slice_contents);
                            }
                        }
                        Intrinsic::SlicePopBack
                        | Intrinsic::SliceRemove
                        | Intrinsic::SlicePopFront => {
                            // We do not decrement the size on intrinsics that could remove values from a slice.
                            // This is because we could potentially go back to the smaller slice and not fill in dummies.
                            // This pass should be tracking the potential max that a slice ***could be***
                            if let Some(inner_sizes) = slice_sizes.get(&slice_contents) {
                                let inner_sizes = inner_sizes.clone();
                                slice_sizes.insert(results[result_index], inner_sizes);

                                self.mapped_slice_values
                                    .insert(slice_contents, results[result_index]);
                                self.slice_parents.insert(results[result_index], slice_contents);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Instruction::Store { address, value } => {
                let value_typ = self.dfg.type_of_value(*value);
                if value_typ.contains_slice_element() {
                    self.compute_slice_sizes(*value, slice_sizes);

                    if let Some(inner_sizes) = slice_sizes.get(value) {
                        let inner_sizes = inner_sizes.clone();
                        slice_sizes.insert(*address, inner_sizes);
                    }
                }
            }
            Instruction::Load { address } => {
                let load_typ = self.dfg.type_of_value(*address);
                if load_typ.contains_slice_element() {
                    if let Some(inner_sizes) = slice_sizes.get(address) {
                        let inner_sizes = inner_sizes.clone();
                        slice_sizes.insert(results[0], inner_sizes);
                    }
                }
            }
            _ => {}
        }
    }

    // This methods computes a map representing a nested slice.
    // The method also automatically computes the given max slice size
    // at each depth of the recursive type.
    // For example if we had a next slice
    fn compute_slice_sizes(
        &self,
        array_id: ValueId,
        slice_sizes: &mut HashMap<ValueId, (usize, Vec<ValueId>)>,
    ) {
        if let Value::Array { array, typ } = &self.dfg[array_id].clone() {
            if let Type::Slice(_) = typ {
                let element_size = typ.element_size();
                let len = array.len() / element_size;
                let mut slice_value = (len, vec![]);
                for value in array {
                    let typ = self.dfg.type_of_value(*value);
                    if let Type::Slice(_) = typ {
                        slice_value.1.push(*value);
                        self.compute_slice_sizes(*value, slice_sizes);
                    }
                }
                // Mark the correct max size based upon an array values internal structure
                let mut max_size = 0;
                for inner_value in slice_value.1.iter() {
                    let inner_slice =
                        slice_sizes.get(inner_value).expect("ICE: should have inner slice set");
                    if inner_slice.0 > max_size {
                        max_size = inner_slice.0;
                    }
                }
                for inner_value in slice_value.1.iter() {
                    let inner_slice =
                        slice_sizes.get_mut(inner_value).expect("ICE: should have inner slice set");
                    if inner_slice.0 < max_size {
                        inner_slice.0 = max_size;
                    }
                }
                slice_sizes.insert(array_id, slice_value);
            }
        }
    }

    pub(crate) fn constant_nested_slices(&mut self) -> Vec<ValueId> {
        std::mem::take(&mut self.slice_values)
    }

    pub(crate) fn slice_parents_map(&mut self) -> HashMap<ValueId, ValueId> {
        std::mem::take(&mut self.slice_parents)
    }

    pub(crate) fn slice_values_map(&mut self) -> HashMap<ValueId, ValueId> {
        std::mem::take(&mut self.mapped_slice_values)
    }

}