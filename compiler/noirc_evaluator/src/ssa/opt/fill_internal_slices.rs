use std::collections::BTreeMap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{Instruction, Intrinsic, InstructionId},
        post_order::PostOrder,
        types::Type,
        value::{Value, ValueId}, dfg::CallStack,
    },
    ssa_gen::Ssa,
};

use acvm::FieldElement;
use fxhash::FxHashMap as HashMap;

impl Ssa {
    /// Fill out slice values represented by SSA array values to contain
    /// dummy data that accounts for dynamic array operations in ACIR gen.
    /// When working with nested slices in ACIR gen it is impossible to discern the size
    /// of internal slices. Thus, we should use the max size of internal nested slices for reading from memory.
    /// However, not increasing the capacity of smaller nested slices can lead to errors where
    /// we end up reading past the specified dynamic index. If we want to also read information that goes
    /// past the nested slice, we will have garbage data in its place if the nested structure is transformed to match.
    pub(crate) fn fill_internal_slices(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            let mut context = Context::new(function);
            context.process_blocks();
        }
        self
    }
}

struct Context<'f> {
    post_order: PostOrder,
    inserter: FunctionInserter<'f>,

    /// Maps SSA array values representing a slice's contents to its updated array value
    /// after an array set or a slice intrinsic operation. 
    /// Maps original value -> result
    mapped_slice_values: BTreeMap<ValueId, ValueId>,

    /// Maps an updated array value following an array operation to its previous value.
    /// When used in conjunction with `mapped_slice_values` we form a two way map of all array 
    /// values being used in array operaitons. 
    /// Maps result -> original value
    slice_parents: BTreeMap<ValueId, ValueId>, 
}

impl<'f> Context<'f> {
    fn new(function: &'f mut Function) -> Self {
        let post_order = PostOrder::with_function(function);
        let inserter = FunctionInserter::new(function);

        Context { post_order, inserter, mapped_slice_values: BTreeMap::new(), slice_parents: BTreeMap::new() }
    }

    fn process_blocks(&mut self) {
        let mut block_order = PostOrder::with_function(self.inserter.function).into_vec();
        block_order.reverse();
        for block in block_order {
            self.process_block(block);
        }
    }

    fn process_block(&mut self, block: BasicBlockId) {
        // Fetch SSA values potentially with internal slices
        let instructions = self.inserter.function.dfg[block].take_instructions();

        // Values containing nested slices to be replaced
        let mut slice_values = Vec::new();
        // Maps SSA array ID representing slice contents to its length and a list of its potential internal slices
        let mut slice_sizes: HashMap<ValueId, (usize, Vec<ValueId>)> = HashMap::default();

        for instruction in instructions.iter() {
            self.collect_slice_information(*instruction, &mut slice_values, &mut slice_sizes);
        }

        // dbg!(self.mapped_slice_values.clone());
        // dbg!(self.slice_parents.clone());
        dbg!(slice_values.clone());

        for instruction in instructions {
            self.push_updated_instruction(instruction, &slice_values, &slice_sizes, block);
        }

        self.inserter.map_terminator_in_place(block);
    }

    /// Determine how the slice sizes map needs to be updated according to the provided instruction.
    fn collect_slice_information(
        &mut self,
        instruction: InstructionId,
        slice_values: &mut Vec<ValueId>,
        slice_sizes: &mut HashMap<ValueId, (usize, Vec<ValueId>)>,
    ) {
        let results = self.inserter.function.dfg.instruction_results(instruction);
        match &self.inserter.function.dfg[instruction] {
            Instruction::ArrayGet { array, .. } => {
                let array_typ = self.inserter.function.dfg.type_of_value(*array);
                let array_value = &self.inserter.function.dfg[*array];
                // If we have an SSA value containing nested slices we should mark it
                // as a slice that potenitally requires to be filled with dummy data.
                if matches!(array_value, Value::Array { .. })
                    && array_typ.contains_slice_element()
                {
                    slice_values.push(*array);
                    // Initial insertion into the slice sizes map
                    // Any other insertions should only occur if the value is already
                    // a part of the map.
                    self.compute_slice_sizes(*array, slice_sizes);
                }

                let res_typ = self.inserter.function.dfg.type_of_value(results[0]);
                if res_typ.contains_slice_element() {
                    if let Some(inner_sizes) = slice_sizes.get_mut(array) {
                        // Include the result in the parent array potential children
                        // If the result has internal slices and is called in an array set
                        // we could potentially have a new larger slice which we need to account for
                        inner_sizes.1.push(results[0]);
                        self.slice_parents.insert(results[0], *array);

                        let inner_sizes_iter = inner_sizes.1.clone();
                        for slice_value in inner_sizes_iter {
                            let inner_slice =
                                slice_sizes.get(&slice_value).unwrap_or_else(|| {
                                    panic!("ICE: should have inner slice set for {slice_value}")
                                });
                            slice_sizes.insert(results[0], inner_slice.clone());
                        }
                    }
                }
            }
            Instruction::ArraySet { array, value, .. } => {
                let array_typ = self.inserter.function.dfg.type_of_value(*array);
                let array_value = &self.inserter.function.dfg[*array];
                if matches!(array_value, Value::Array { .. })
                    && array_typ.contains_slice_element()
                {
                    slice_values.push(*array);
                    self.compute_slice_sizes(*array, slice_sizes);
                }

                let value_typ = self.inserter.function.dfg.type_of_value(*value);
                if value_typ.contains_slice_element() {
                    self.compute_slice_sizes(*value, slice_sizes);

                    let inner_sizes =
                        slice_sizes.get_mut(array).expect("ICE expected slice sizes");
                    inner_sizes.1.push(*value);

                    let value_parent = self.resolve_slice_parent(*value);
                    if slice_values.contains(&value_parent) {
                        // Map the value parent to the current array in case nested slices
                        // from the current array set to larger values later in the program 
                        self.mapped_slice_values.insert(value_parent, *array);
                    }
                    
                }

                if let Some(inner_sizes) = slice_sizes.get_mut(array) {
                    let inner_sizes = inner_sizes.clone();
                    slice_sizes.insert(results[0], inner_sizes);

                    self.mapped_slice_values.insert(*array, results[0]);
                    self.slice_parents.insert(results[0], *array);
                }
            }
            Instruction::Call { func, arguments } => {
                let func = &self.inserter.function.dfg[*func];
                match func {
                    Value::Intrinsic(intrinsic) => {
                        let (argument_index, result_index) = match intrinsic {
                            Intrinsic::SlicePushBack
                            | Intrinsic::SlicePushFront
                            | Intrinsic::SlicePopBack
                            | Intrinsic::SliceInsert
                            | Intrinsic::SliceRemove => (1, 1),
                            Intrinsic::SlicePopFront => (1, 2),
                            _ => return,
                        };
                        match intrinsic {
                            Intrinsic::SlicePushBack
                            | Intrinsic::SlicePushFront
                            | Intrinsic::SliceInsert => {
                                let slice_contents = arguments[argument_index];
                                if let Some(inner_sizes) = slice_sizes.get_mut(&slice_contents)
                                {
                                    inner_sizes.0 += 1;

                                    let inner_sizes = inner_sizes.clone();
                                    slice_sizes.insert(results[result_index], inner_sizes);

                                    self.mapped_slice_values.insert(slice_contents, results[result_index]);
                                }
                            }
                            Intrinsic::SlicePopBack
                            | Intrinsic::SlicePopFront
                            | Intrinsic::SliceRemove => {
                                let slice_contents = arguments[argument_index];
                                // We do not decrement the size on intrinsics that could remove values from a slice.
                                // This is because we could potentially go back to the smaller slice and not fill in dummies.
                                // This pass should be tracking the potential max that a slice ***could be***
                                if let Some(inner_sizes) = slice_sizes.get(&slice_contents) {
                                    let inner_sizes = inner_sizes.clone();
                                    slice_sizes.insert(results[result_index], inner_sizes);

                                    self.mapped_slice_values.insert(slice_contents, results[result_index]);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }

    fn push_updated_instruction(
        &mut self,
        instruction: InstructionId,
        slice_values: &Vec<ValueId>,
        slice_sizes: &HashMap<ValueId, (usize, Vec<ValueId>)>,
        block: BasicBlockId,
    ) {
        match &self.inserter.function.dfg[instruction] {
            Instruction::ArrayGet { array, .. } => {
                if slice_values.contains(array) {
                    let (new_array_get, call_stack) = self.get_updated_array_op_instr(*array, slice_sizes, instruction);

                    self.inserter.push_instruction_value(
                        new_array_get,
                        instruction,
                        block,
                        call_stack,
                    );
                } else {
                    self.inserter.push_instruction(instruction, block);
                }
            }
            Instruction::ArraySet { array, .. } => {
                if slice_values.contains(array) {
                    let (new_array_set, call_stack) = self.get_updated_array_op_instr(*array, slice_sizes, instruction);
                    
                    self.inserter.push_instruction_value(
                        new_array_set,
                        instruction,
                        block,
                        call_stack,
                    );
                } else {
                    self.inserter.push_instruction(instruction, block);
                }
            }

            _ => {
                self.inserter.push_instruction(instruction, block);
            }
        }
    }
    
    /// Construct an updated ArrayGet or ArraySet instruction where the array value
    /// has been replaced by a newly filled in array according to the max internal 
    /// slice sizes.
    fn get_updated_array_op_instr(
        &mut self,
        array_id: ValueId,
        slice_sizes: &HashMap<ValueId, (usize, Vec<ValueId>)>,
        instruction: InstructionId,
    ) -> (Instruction, CallStack) {
        let results = self.inserter.function.dfg.instruction_results(instruction);
        dbg!(results);
        let mapped_slice_value = self.resolve_slice_value(array_id);

        let (current_size, _) = slice_sizes
        .get(&mapped_slice_value)
        .unwrap_or_else(|| panic!("should have slice sizes: {mapped_slice_value}"));

        let mut max_sizes = Vec::new();
        max_sizes.resize(3, 0);
        max_sizes[0] = *current_size;
        self.compute_slice_max_sizes(array_id, slice_sizes, &mut max_sizes, 1);
        // dbg!(array_id);
        // dbg!(mapped_slice_value);
        dbg!(max_sizes.clone());

        let typ = self.inserter.function.dfg.type_of_value(array_id);
        let new_array =
            self.attach_slice_dummies(&typ, Some(array_id), true, &max_sizes);

        let instruction_id = instruction;
        let (instruction, call_stack) =
            self.inserter.map_instruction(instruction_id);
        let new_array_op_instr = match instruction {
            Instruction::ArrayGet { index, .. } => Instruction::ArrayGet {
                array: new_array,
                index: index,
            },
            Instruction::ArraySet { index, value, .. } => Instruction::ArraySet {
                array: new_array,
                index,
                value,
            },
            _ => panic!("Expected array set"),
        };

        (new_array_op_instr, call_stack)
    }

    fn attach_slice_dummies(
        &mut self,
        typ: &Type,
        value: Option<ValueId>,
        is_parent_slice: bool,
        max_sizes: &[usize],
    ) -> ValueId {
        match typ {
            Type::Numeric(_) => {
                if let Some(value) = value {
                    self.inserter.resolve(value)
                } else {
                    let zero = FieldElement::zero();
                    self.inserter.function.dfg.make_constant(zero, Type::field())
                }
            }
            Type::Array(element_types, len) => {
                if let Some(value) = value {
                    self.inserter.resolve(value)
                } else {
                    let mut array = im::Vector::new();
                    for _ in 0..*len {
                        for typ in element_types.iter() {
                            array.push_back(self.attach_slice_dummies(
                                typ,
                                None,
                                false,
                                max_sizes,
                            ));
                        }
                    }
                    self.inserter.function.dfg.make_array(array, typ.clone())
                }
            }
            Type::Slice(element_types) => {
                let (current_size, max_sizes) = max_sizes.split_first().expect("ICE: Missing internal slice max size");
                let mut max_size = *current_size;
                if let Some(value) = value {
                    let mut slice = im::Vector::new();
                    match &self.inserter.function.dfg[value].clone() {
                        Value::Array { array, .. } => {
                            if is_parent_slice {
                                max_size = array.len() / element_types.len();
                            }
                            for i in 0..max_size {
                                for (element_index, element_type) in
                                    element_types.iter().enumerate()
                                {
                                    let index_usize = i * element_types.len() + element_index;
                                    if index_usize < array.len() {
                                        slice.push_back(self.attach_slice_dummies(
                                            element_type,
                                            Some(array[index_usize]),
                                            false,
                                            max_sizes
                                        ));
                                    } else {
                                        slice.push_back(self.attach_slice_dummies(
                                            element_type,
                                            None,
                                            false,
                                            max_sizes,
                                        ));
                                    }
                                }
                            }
                        }
                        _ => {
                            panic!("Expected an array value");
                        }
                    }
                    self.inserter.function.dfg.make_array(slice, typ.clone())
                } else {
                    let mut slice = im::Vector::new();
                    for _ in 0..max_size {
                        for typ in element_types.iter() {
                            slice.push_back(self.attach_slice_dummies(
                                typ,
                                None,
                                false,
                                max_sizes,
                            ));
                        }
                    }
                    self.inserter.function.dfg.make_array(slice, typ.clone())
                }
            }
            Type::Reference => {
                unreachable!("ICE: Generating dummy data for references is unsupported")
            }
            Type::Function => {
                unreachable!("ICE: Generating dummy data for functions is unsupported")
            }
        }
    }

    fn compute_slice_sizes(
        &self,
        array_id: ValueId,
        slice_sizes: &mut HashMap<ValueId, (usize, Vec<ValueId>)>,
    ) {
        if let Value::Array { array, typ } = &self.inserter.function.dfg[array_id].clone() {
            if let Type::Slice(_) = typ {
                let element_size = typ.element_size();
                let len = array.len() / element_size;
                let mut slice_value = (len, vec![]);
                for value in array {
                    let typ = self.inserter.function.dfg.type_of_value(*value);
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

    /// Determine the maximum possible size of an internal slice at each
    /// layer of a nested slice.
    /// 
    /// If the slice map is incorrectly formed the function will exceed 
    /// the type's nested slice depth and panic.
    fn compute_slice_max_sizes(
        &self,
        array_id: ValueId,
        slice_sizes: &HashMap<ValueId, (usize, Vec<ValueId>)>,
        max_sizes: &mut Vec<usize>,
        depth: usize,
    ) {
        let array_id = self.resolve_slice_value(array_id);
        let (current_size, inner_slices) = slice_sizes
            .get(&array_id)
            .unwrap_or_else(|| panic!("should have slice sizes: {array_id}"));

        if inner_slices.is_empty() {
            return
        }

        let mut max = *current_size;
        for inner_slice in inner_slices.iter() {
            let inner_slice = &self.resolve_slice_value(*inner_slice);

            let (inner_size, _) = slice_sizes[inner_slice];
            if inner_size > max {
                max = inner_size;
            }
            self.compute_slice_max_sizes(*inner_slice, slice_sizes, max_sizes, depth + 1);
        }

        max_sizes[depth] = max;
        if max > max_sizes[depth] {
            max_sizes[depth] = max;
        }
    }

    /// Resolves a ValueId representing a slice's contents to its update value
    /// If there is no resolved value for the supplied value, the value which 
    /// was passed to the method is returned.
    fn resolve_slice_value(
        &self,
        array_id: ValueId,
    ) -> ValueId {
        match self.mapped_slice_values.get(&array_id) {
            Some(value) => self.resolve_slice_value(*value),
            None => array_id,
        }
    }

    fn resolve_slice_parent(
        &self,
        array_id: ValueId
    ) -> ValueId {
        match self.slice_parents.get(&array_id) {
            Some(value) => self.resolve_slice_parent(*value),
            None => array_id,
        }
    }
}
