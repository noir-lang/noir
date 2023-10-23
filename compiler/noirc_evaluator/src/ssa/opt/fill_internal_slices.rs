use std::collections::BTreeMap;

use crate::ssa::{ssa_gen::Ssa, ir::{post_order::PostOrder, function_inserter::FunctionInserter, function::Function, basic_block::BasicBlockId, instruction::Instruction, value::{ValueId, Value}, types::Type, dfg::CallStack}};

use acvm::FieldElement;
use fxhash::FxHashMap as HashMap;

impl Ssa {
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

    /// Maps SSA array values to their slice size and any nested slices internal to the parent slice.
    /// This enables us to maintain the slice structure of a slice when performing an array get.
    slice_sizes: HashMap<ValueId, Vec<(usize, Option<ValueId>)>>,

    /// Maps SSA array value parent to the max internal slice size
    max_slice_sizes: HashMap<ValueId, usize>,

    /// Maps SSA array value to its slice size and children
    /// NOTE: using a BTreeMap here for clean output while debugging which back to a HashMap
    new_slice_sizes: BTreeMap<ValueId, (usize, Vec<ValueId>)>,
}

impl<'f> Context<'f> {
    fn new(function: &'f mut Function) -> Self {
        let post_order = PostOrder::with_function(function);
        let inserter = FunctionInserter::new(function);

        Context {
            post_order,
            inserter,
            slice_sizes: HashMap::default(),
            max_slice_sizes: HashMap::default(),
            new_slice_sizes: BTreeMap::new(),
        }
    }

    fn process_blocks(
        &mut self,
    ) {
        let mut block_order = PostOrder::with_function(self.inserter.function).into_vec();
        block_order.reverse();
        for block in block_order {
            self.process_block(block);
        } 
    }

    fn process_block(
        &mut self,
        block: BasicBlockId,
    ) {
        // Fetch SSA values potentially with internal slices
        let instructions = self.inserter.function.dfg[block].take_instructions();
        let mut slice_values = Vec::new();
        let mut slice_sizes = HashMap::default();
        let mut mapped_slice_values = BTreeMap::new();
        for instruction in instructions.iter() {
            // TODO: get rid of this clone
            match &self.inserter.function.dfg[*instruction] {
                Instruction::ArrayGet { array, .. } => {
                    // dbg!("got array get");
                    // dbg!(array);
                    // dbg!(results);
                    // let array_typ = self.inserter.function.dfg.type_of_value(*array);
                    match &self.inserter.function.dfg[*array] {
                        Value::Array { typ, .. } => {
                            if typ.contains_slice_element() {
                                slice_values.push(*array);
                                self.compute_slice_sizes_new(*array, &mut slice_sizes);
                            }
                        }
                        _ => {}
                    }

                    let results = self.inserter.function.dfg.instruction_results(*instruction);
                    let res_typ = self.inserter.function.dfg.type_of_value(results[0]);
                    if res_typ.contains_slice_element() {
                        // dbg!(array);
                        let inner_sizes = slice_sizes.get_mut(array).expect("ICE expected slice sizes");
                        // dbg!(inner_sizes.clone());
                        // dbg!(results[0]);
                        (*inner_sizes).1.push(results[0]);
                        // dbg!(array);
                        // dbg!(inner_sizes.clone());
                        let inner_sizes_iter = inner_sizes.1.clone();
                        for slice_value in inner_sizes_iter {
                            let inner_slice = slice_sizes.get(&slice_value).expect("ICE: should have inner slice set");
                            // inner_slices.push(inner_slice.clone());
                            slice_sizes.insert(results[0], inner_slice.clone());

                            mapped_slice_values.insert(results[0], slice_value);

                            // slice_sizes.insert(results[0], inner_slice);
                            // if let Some(inner_value) = slice_sizes.get_mut(&results[0]) {
                            //     (*inner_value).push(inner_slice);
                            // } else {
                            //     slice_sizes.insert(results[0], inner_slice);
                            // }
                        }
                        // slice
                    }

                }
                Instruction::ArraySet { array, value, .. } => {
                    match &self.inserter.function.dfg[*array] {
                        Value::Array { typ, .. } => {
                            if typ.contains_slice_element() {
                                slice_values.push(*array);
                                self.compute_slice_sizes_new(*array, &mut slice_sizes);
                                // self.compute_slice_sizes_new(*array);
                                // let slice_sizes = self.new_slice_sizes.get(array).expect("ICE expected slice sizes");
                                // let (_, inner_sizes) = slice_sizes.1.splt
                            }
                        }
                        _ => {}
                    }

                    match &self.inserter.function.dfg[*value] {
                        Value::Array { typ, .. } => {
                            if typ.contains_slice_element() {
                                self.compute_slice_sizes_new(*value, &mut slice_sizes);
                            }
                        }
                        _ => {}
                    }
                    let results = self.inserter.function.dfg.instruction_results(*instruction);

                    let value_typ = self.inserter.function.dfg.type_of_value(*value);
                    if value_typ.contains_slice_element() {
                        // let slice_size = slice_sizes.get(value).expect("ICE: expected size");
                        // dbg!(slice_size);
                        // dbg!(array);
                        // TODO: probably need to just attach to inner slice
                        let inner_sizes = slice_sizes.get_mut(array).expect("ICE expected slice sizes");
                        // dbg!(inner_sizes.clone());
                        (*inner_sizes).1.push(*value);

                        mapped_slice_values.insert(*value, results[0]);
                    }
                    
                    let array_typ = self.inserter.function.dfg.type_of_value(*array);
                    if array_typ.contains_slice_element() {
                        // dbg!(array);
                        let inner_sizes = slice_sizes.get_mut(array).expect("ICE expected slice sizes");
                        // if let Some(len) = len {
                        //     dbg!(len);
                        //     dbg!(inner_sizes.clone());
                        //     for slice_value in inner_sizes.1.clone() {
                        //         let inner_size = slice_sizes.get(&slice_value).expect("ICE: should have inner slice set");
                        //         dbg!(inner_size);
                        //     }
                        //     let slice_size = slice_sizes.get(value).expect("ICE: expected size");
                        //     dbg!(slice_size);
                        // }
                        // dbg!(results[0]);
                        // let value_typ = self.inserter.function.dfg.type_of_value(*value);
                        // if value_typ.contains_slice_element() {
                        //     // let slice_size = slice_sizes.get(value).expect("ICE: expected size");
                        //     // dbg!(slice_size);
                        //     (*inner_sizes).1.push(*value);
                        // }


                        let inner_sizes = inner_sizes.clone();
                        slice_sizes.insert(results[0], inner_sizes);

                        // TODO: probably should include checks with array get as well
                        mapped_slice_values.insert(*array, results[0]);
                    }
                }
                _ => {}
            }
        }

        // Construct max sizes list following the internal slice structure
        // let mut max_sizes: HashMap<ValueId, Vec<usize>> = HashMap::default();
        // Separate loop as we borrow `self` immutably in the previous loop and borrow it mutably here
        // for slice_value in &slice_values {
        //     // dbg!(slice_value);
        //     self.compute_slice_sizes(*slice_value, None, None);
        //     self.compute_max_slice_sizes(*slice_value);
        //     // dbg!(self.max_slice_sizes.clone());
        //     // Reorganize max size list to follow nested slice structure
        //     let slice_sizes = self.slice_sizes.get(slice_value).expect("should have slice sizes");
        //     for slice_size in slice_sizes {
        //         // dbg!(slice_size.1);
        //         if let Some(parent_array) = slice_size.1 {
        //             let max_size = self.max_slice_sizes.get(&parent_array).expect("ICE: expected max size");
        //             let max_size_list = max_sizes.get_mut(&slice_value).expect("ICE: max size list should have initial list from parent array");
        //             max_size_list.push(*max_size);
        //         } else {
        //             // If we have the initial parent array add to max sizes
        //             max_sizes.insert(*slice_value, vec![slice_size.0]);
        //         }
        //     }
        // }
        // dbg!(max_sizes.clone());

        let mut nested_slice_max = 0;
        for slice_value in &slice_values {
            let mut mapped_slice_value = *slice_value;
            self.follow_mapped_slice_values(*slice_value, &mapped_slice_values, &mut mapped_slice_value);

            let nested_depth = self.find_max_nested_depth(mapped_slice_value, &slice_sizes);
            if nested_depth > nested_slice_max {
                nested_slice_max = nested_depth
            }
        }

        // TODO: Need to be able to account for setting of internal slices to slices of larger size
        // Options:
        // 1. Track the internal types along with the size. If we run into an array set with the same type
        // as one of the internal type I should increase the max size?
        //

        for instruction in instructions {
            match &self.inserter.function.dfg[instruction] {
                Instruction::ArrayGet { array, .. } => {
                    let typ = self.inserter.function.dfg.type_of_value(*array);
                    let mut new_array = None;
                    if matches!(typ, Type::Slice(_)) {
                        if let Value::Array { .. } = self.inserter.function.dfg[*array] {
                            // let max_sizes = max_sizes.get(array).expect("ICE: expected max sizes");
                            // dbg!(max_sizes.clone());
                            // let mut mapped_slice_value = array;
                            // self.follow_mapped_slice_values(*array, &mapped_slice_values, &mut mapped_slice_value);
                            // dbg!(mapped_slice_value);
                            // let max_sizes = vec![2, 5, 6];
                            new_array = Some(self.attach_slice_dummies(&typ, Some(*array), nested_slice_max, &slice_sizes));
                        }
                    };

                    if let Some(new_array) = new_array {
                        let instruction_id = instruction;
                        let (instruction, call_stack) = self.inserter.map_instruction(instruction_id);
                        let new_array_get = match instruction {
                            Instruction::ArrayGet { index, .. } => {
                                Instruction::ArrayGet { array: new_array, index }
                            }
                            _ => panic!("Expected array get"),
                        };
                        self.inserter.push_instruction_value(new_array_get, instruction_id, block, call_stack);
                    } else {
                        self.inserter.push_instruction(instruction, block);
                    }
                }
                Instruction::ArraySet { array, .. } => {
                    // dbg!(slice_values.contains(array));

                    let typ = self.inserter.function.dfg.type_of_value(*array);
                    let mut new_array = None;
                    if matches!(typ, Type::Slice(_)) {
                        if let Value::Array { .. } = self.inserter.function.dfg[*array] {
                            // let max_sizes = max_sizes.get(array).expect("ICE: expected max sizes");
                            // dbg!(max_sizes.clone());
                            // let max_sizes = vec![2 as usize, 5, 6];
                            new_array = Some(self.attach_slice_dummies(&typ, Some(*array), nested_slice_max, &slice_sizes));
                        }
                    };

                    if let Some(new_array) = new_array {
                        let instruction_id = instruction;
                        let (instruction, call_stack) = self.inserter.map_instruction(instruction_id);
                        let new_array_set = match instruction {
                            Instruction::ArraySet { index, value, .. } => {
                                Instruction::ArraySet { array: new_array, index, value }
                            }
                            _ => panic!("Expected array set"),
                        };
                        self.inserter.push_instruction_value(new_array_set, instruction_id, block, call_stack);
                    } else {
                        self.inserter.push_instruction(instruction, block);
                    }
                }
                _ => {
                    self.inserter.push_instruction(instruction, block);
                }
            }
        }
 
    }

    fn attach_slice_dummies(
        &mut self,
        typ: &Type,
        value: Option<ValueId>,
        nested_slice_max: usize,
        slice_sizes: &HashMap<ValueId, (usize, Vec<ValueId>)>,
    ) -> ValueId {
        match typ {
            Type::Numeric(_) => {
                if let Some(value) = value {
                    return value;
                } else {
                    let zero = FieldElement::zero();
                    return self.inserter.function.dfg.make_constant(zero, Type::field());   
                }
            }
            Type::Array(element_types, len) => {
                if let Some(value) = value {
                    return value;
                } else {
                    let mut array = im::Vector::new();
                    for _ in 0..*len {
                        for typ in element_types.iter() {
                            array.push_back(self.attach_slice_dummies(typ, None, nested_slice_max, slice_sizes));
                        }
                    }
                    return self.inserter.function.dfg.make_array(array, typ.clone())
                }
            }
            Type::Slice(element_types) => {
                // let (max_size, max_sizes) = max_sizes.split_first().expect("should be able to split max sizes");
                // Everything works if I just set everything to the nested max
                let max_size = 6;
                // let max_size = *max_size;
                if let Some(value) = value {
                    let mut slice = im::Vector::new();
                    // let inner_slice_sizes = slice_sizes.get(&value);
                    // dbg!(inner_slice_sizes.clone());
                    // if value.to_usize() == 1533 {
                    //     dbg!("GOT 1533");
                    // }
                    // let mut mapped_slice_value = value;
                    // if mapped_slice_value.to_usize() == 1533 {
                    //     dbg!("GOT mapped_slice_value 1533");
                    // }
                    // self.follow_mapped_slice_values(value, &mapped_slice_values, &mut mapped_slice_value);
                    // let x = self.compute_inner_max_size(mapped_slice_value, slice_sizes);
                    // dbg!(x);
                    match &self.inserter.function.dfg[value].clone() {
                        Value::Array { array, .. } => {
                            for i in 0..max_size {
                                for (element_index, element_type) in element_types.iter().enumerate() {
                                    let index_usize = i * element_types.len() + element_index;
                                    if index_usize < array.len() {
                                        slice.push_back(self.attach_slice_dummies(element_type, Some(array[index_usize]), nested_slice_max, slice_sizes));
                                    } else {
                                        slice.push_back(self.attach_slice_dummies(element_type, None, nested_slice_max, slice_sizes));
                                    }
                                }
                            }
                        }
                        _ => {
                            panic!("Expected an array value");
                        }
                    }
                    return self.inserter.function.dfg.make_array(slice, typ.clone())
                } else {
                    let mut slice = im::Vector::new();
                    // dbg!(max_size);
                    // dbg!(element_types.clone());
                    for _ in 0..max_size {
                        for typ in element_types.iter() {
                            slice.push_back(self.attach_slice_dummies(typ, None, nested_slice_max, slice_sizes));
                        }
                    }
                    return self.inserter.function.dfg.make_array(slice, typ.clone())
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
        &mut self,
        current_array_id: ValueId,
        parent_array: Option<ValueId>,
        inner_parent_array: Option<ValueId>,
    ) {
        if let Value::Array { array, typ } = &self.inserter.function.dfg[current_array_id].clone() {
            if let Type::Slice(_) = typ {
                let element_size = typ.element_size();
                let true_len = array.len() / element_size;
                if let Some(parent_array) = parent_array {
                    let sizes_list =
                        self.slice_sizes.get_mut(&parent_array).expect("ICE: expected size list");
                    let inner_parent_array = inner_parent_array.expect("ICE: expected inner_parent_array");
                    sizes_list.push((true_len, Some(inner_parent_array)));
                } else {
                    // This means the current_array_id is the parent array
                    self.slice_sizes.insert(current_array_id, vec![(true_len, None)]);
                }
                for value in array {
                    let typ = self.inserter.function.dfg.type_of_value(*value);
                    if let Type::Slice(_) = typ {
                        if parent_array.is_some() {
                            self.compute_slice_sizes(*value, parent_array, Some(current_array_id));
                        } else {
                            self.compute_slice_sizes(*value, Some(current_array_id), Some(current_array_id));
                        }
                    }
                }
            }
        }
    }

    fn compute_max_slice_sizes(
        &mut self,
        array_id: ValueId,
    ) {
        let slice_sizes = self.slice_sizes.get(&array_id).expect("ICE: expected slice sizes");
        for slice_size in slice_sizes {
            if let Some(parent_array) = slice_size.1 {
                if let Some(max_size) = self.max_slice_sizes.get_mut(&parent_array) {
                    if slice_size.0 > *max_size {
                        *max_size = slice_size.0;
                    }
                } else {
                    self.max_slice_sizes.insert(parent_array, slice_size.0);
                }
            }
        }
    }

    fn compute_slice_sizes_new(
        &self,
        array_id: ValueId,
        // TODO: could probably change this to a single ValueId as we are getting the max inner size here
        slice_sizes: &mut HashMap<ValueId, (usize, Vec<ValueId>)>,
    ) {
        if let Value::Array { array, typ } = &self.inserter.function.dfg[array_id].clone() {
            if let Type::Slice(_) = typ {
                let element_size = typ.element_size();
                let len = array.len() / element_size;
                // self.new_slice_sizes.insert(array_id, (len,))
                let mut slice_value = (len, vec![]);
                for value in array {
                    let typ = self.inserter.function.dfg.type_of_value(*value);
                    if let Type::Slice(_) = typ {
                        slice_value.1.push(*value);
                        self.compute_slice_sizes_new(*value, slice_sizes);
                    }
                }
                let mut max_size = 0;
                for inner_value in slice_value.1.iter() {
                    let inner_slice = slice_sizes.get(inner_value).expect("ICE: should have inner slice set");
                    if inner_slice.0 > max_size {
                        max_size = inner_slice.0;
                    }
                }
                for inner_value in slice_value.1.iter() {
                    let inner_slice = slice_sizes.get_mut(inner_value).expect("ICE: should have inner slice set");
                    if inner_slice.0 < max_size {
                        (*inner_slice).0 = max_size;
                    }
                }
                slice_sizes.insert(array_id, slice_value);
                // let mut max_size = 0;
                // for inner_value in slice_value.1.iter() {
                //     let inner_slice = self.new_slice_sizes.get(inner_value).expect("ICE: should have inner slice set");
                //     if inner_slice.0 > max_size {
                //         max_size = inner_slice.0;
                //     }
                // }
                // for inner_value in slice_value.1.iter() {
                //     let inner_slice = self.new_slice_sizes.get_mut(inner_value).expect("ICE: should have inner slice set");
                //     if inner_slice.0 < max_size {
                //         (*inner_slice).0 = max_size;
                //     }
                // }
                // self.new_slice_sizes.insert(array_id, slice_value);
            }
        }
    }

    fn find_max_nested_depth(
        &self,
        array_id: ValueId,
        slice_sizes: &HashMap<ValueId, (usize, Vec<ValueId>)>,
    ) -> usize {
        let (current_size, inner_slices) = slice_sizes.get(&array_id).expect("should have slice sizes");
        let mut max = *current_size;
        for inner_slice in inner_slices.iter() {
            if let Some(inner_max) = self.compute_inner_max_size(*inner_slice, slice_sizes) {
                if inner_max > max {
                    max = inner_max;
                }
            }
            let inner_nested_max = self.find_max_nested_depth(*inner_slice, slice_sizes);
            if inner_nested_max > max {
                max = inner_nested_max;
            }
        }
        max
    }

    fn compute_outer_max_size(
        &self,
        parent_array: ValueId,
        current_array_id: ValueId,
        slice_sizes: &HashMap<ValueId, (usize, Vec<ValueId>)>,
        max_sizes: &mut HashMap<ValueId, Vec<usize>>,
    ) {
        // let mut new_max_sizes = Vec::new();
        let (current_size, inner_slices) = slice_sizes.get(&current_array_id).expect("should have slice sizes");
        if let None = max_sizes.get_mut(&parent_array) {
            max_sizes.insert(parent_array, vec![*current_size]);
        } 
        let inner_max = self.compute_inner_max_size(current_array_id, slice_sizes);
        // dbg!(inner_max);
        if let Some(max) = inner_max {
            let max_size_list = max_sizes.get_mut(&parent_array).expect("ICE: should have max list");
            max_size_list.push(max);
        }
        let mut current_max = None;
        for inner_slice in inner_slices {
            let inner_max = self.compute_inner_max_size(*inner_slice, slice_sizes);
            if let Some(inner_size) = inner_max {
                if let Some(inner_max) = current_max {
                    if inner_size > inner_max {
                        current_max = Some(inner_size);
                    }
                } else {
                    current_max = Some(inner_size);
                }
            }
        }
        if let Some(max) = current_max {
            let max_size_list = max_sizes.get_mut(&parent_array).expect("ICE: should have max list");
            max_size_list.push(max);
        }
        // let mut possible_max_sizes = Vec::new();
        for inner_slice in inner_slices {
            self.compute_outer_max_size(parent_array, *inner_slice, slice_sizes, max_sizes);
        }

    }

    fn compute_inner_max_size(
        &self,
        current_array_id: ValueId,
        slice_sizes: &HashMap<ValueId, (usize, Vec<ValueId>)>,
    ) -> Option<usize> {
        let (_, inner_slices) = slice_sizes.get(&current_array_id).expect("should have slice sizes");
        if inner_slices.is_empty() {
            return None
        }
        let mut max_size = None;
        for inner_slice in inner_slices.iter() {
            let (inner_size, _) = slice_sizes.get(inner_slice).expect("should have slice sizes");
            if let Some(inner_max) = max_size {
                if *inner_size > inner_max {
                    max_size = Some(*inner_size);
                }
            } else {
                max_size = Some(*inner_size);
            }
        }
        max_size
    }

    fn follow_mapped_slice_values(
        &self,
        array_id: ValueId,
        mapped_slice_values: &BTreeMap<ValueId, ValueId>,
        new_array_id: &mut ValueId,
    ) {
        if let Some(value) = mapped_slice_values.get(&array_id) {
            *new_array_id = *value;
            self.follow_mapped_slice_values(*value, mapped_slice_values, new_array_id);
        } 
    }
    
}