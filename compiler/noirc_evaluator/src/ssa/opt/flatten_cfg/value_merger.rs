use std::hash::Hash;

use acvm::FieldElement;
use fxhash::FxHashMap as HashMap;

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::{CallStack, DataFlowGraph},
    instruction::{BinaryOp, Instruction, Intrinsic},
    types::Type,
    value::{Value, ValueId},
};

use crate::ssa::opt::flatten_cfg::Store;

pub(crate) struct ValueMerger<'a> {
    dfg: &'a mut DataFlowGraph,
    block: BasicBlockId,
    store_values: Option<&'a HashMap<ValueId, Store>>,
    outer_block_stores: Option<&'a HashMap<ValueId, ValueId>>,
    slice_sizes: HashMap<ValueId, usize>,
    // Maps SSA array values to each nested slice size and the array id of its parent array
    inner_slice_sizes: HashMap<ValueId, Vec<(usize, Option<ValueId>)>>,
    new_slice_sizes: &'a mut HashMap<ValueId, (usize, Vec<ValueId>)>,
}

impl<'a> ValueMerger<'a> {
    pub(crate) fn new(
        dfg: &'a mut DataFlowGraph,
        block: BasicBlockId,
        store_values: Option<&'a HashMap<ValueId, Store>>,
        outer_block_stores: Option<&'a HashMap<ValueId, ValueId>>,
        new_slice_sizes: &'a mut HashMap<ValueId, (usize, Vec<ValueId>)>,
    ) -> Self {
        ValueMerger {
            dfg,
            block,
            store_values,
            outer_block_stores,
            slice_sizes: HashMap::default(),
            inner_slice_sizes: HashMap::default(),
            new_slice_sizes,
        }
    }

    /// Merge two values a and b from separate basic blocks to a single value.
    /// If these two values are numeric, the result will be
    /// `then_condition * then_value + else_condition * else_value`.
    /// Otherwise, if the values being merged are arrays, a new array will be made
    /// recursively from combining each element of both input arrays.
    ///
    /// It is currently an error to call this function on reference or function values
    /// as it is less clear how to merge these.
    pub(crate) fn merge_values(
        &mut self,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    ) -> ValueId {
        match self.dfg.type_of_value(then_value) {
            Type::Numeric(_) => {
                self.merge_numeric_values(then_condition, else_condition, then_value, else_value)
            }
            typ @ Type::Array(_, _) => {
                self.merge_array_values(typ, then_condition, else_condition, then_value, else_value)
            }
            typ @ Type::Slice(_) => {
                self.merge_slice_values(typ, then_condition, else_condition, then_value, else_value)
            }
            Type::Reference(_) => panic!("Cannot return references from an if expression"),
            Type::Function => panic!("Cannot return functions from an if expression"),
        }
    }

    /// Merge two numeric values a and b from separate basic blocks to a single value. This
    /// function would return the result of `if c { a } else { b }` as  `c*a + (!c)*b`.
    pub(crate) fn merge_numeric_values(
        &mut self,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    ) -> ValueId {
        let then_type = self.dfg.type_of_value(then_value);
        let else_type = self.dfg.type_of_value(else_value);
        assert_eq!(
            then_type, else_type,
            "Expected values merged to be of the same type but found {then_type} and {else_type}"
        );

        let then_call_stack = self.dfg.get_value_call_stack(then_value);
        let else_call_stack = self.dfg.get_value_call_stack(else_value);

        let call_stack = if then_call_stack.is_empty() { else_call_stack } else { then_call_stack };

        // We must cast the bool conditions to the actual numeric type used by each value.
        let then_condition = self
            .dfg
            .insert_instruction_and_results(
                Instruction::Cast(then_condition, then_type),
                self.block,
                None,
                call_stack.clone(),
            )
            .first();
        let else_condition = self
            .dfg
            .insert_instruction_and_results(
                Instruction::Cast(else_condition, else_type),
                self.block,
                None,
                call_stack.clone(),
            )
            .first();

        let mul = Instruction::binary(BinaryOp::Mul, then_condition, then_value);
        let then_value = self
            .dfg
            .insert_instruction_and_results(mul, self.block, None, call_stack.clone())
            .first();

        let mul = Instruction::binary(BinaryOp::Mul, else_condition, else_value);
        let else_value = self
            .dfg
            .insert_instruction_and_results(mul, self.block, None, call_stack.clone())
            .first();

        let add = Instruction::binary(BinaryOp::Add, then_value, else_value);
        self.dfg.insert_instruction_and_results(add, self.block, None, call_stack).first()
    }

    /// Given an if expression that returns an array: `if c { array1 } else { array2 }`,
    /// this function will recursively merge array1 and array2 into a single resulting array
    /// by creating a new array containing the result of self.merge_values for each element.
    pub(crate) fn merge_array_values(
        &mut self,
        typ: Type,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    ) -> ValueId {
        let mut merged = im::Vector::new();

        let (element_types, len) = match &typ {
            Type::Array(elements, len) => (elements, *len),
            _ => panic!("Expected array type"),
        };

        for i in 0..len {
            for (element_index, element_type) in element_types.iter().enumerate() {
                let index = ((i * element_types.len() + element_index) as u128).into();
                let index = self.dfg.make_constant(index, Type::field());

                let typevars = Some(vec![element_type.clone()]);

                let mut get_element = |array, typevars| {
                    let get = Instruction::ArrayGet { array, index };
                    self.dfg
                        .insert_instruction_and_results(get, self.block, typevars, CallStack::new())
                        .first()
                };

                let then_element = get_element(then_value, typevars.clone());
                let else_element = get_element(else_value, typevars);

                merged.push_back(self.merge_values(
                    then_condition,
                    else_condition,
                    then_element,
                    else_element,
                ));
            }
        }

        self.dfg.make_array(merged, typ)
    }

    fn merge_slice_values(
        &mut self,
        typ: Type,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value_id: ValueId,
        else_value_id: ValueId,
    ) -> ValueId {
        let mut merged = im::Vector::new();

        let element_types = match &typ {
            Type::Slice(elements) => elements,
            _ => panic!("Expected slice type"),
        };
        // dbg!(then_value_id);
        // let new_slice_sizes = self.new_slice_sizes("ICE: should have slice sizes");
        // dbg!(new_slice_sizes.get(&then_value_id));
        let then_len = if let Some(val) = self.new_slice_sizes.get(&then_value_id) {
            // dbg!(val.0);
            // dbg!(self.dfg.get_slice_size(then_value_id));
            // dbg!(self.dfg.get_slice_sizes().clone());
            val.0
        } else {
            dbg!(then_value_id);
            dbg!(self.dfg.resolve(then_value_id));
            panic!("got none");
            dbg!(&self.dfg[then_value_id]);
            // dbg!(self.dfg.get_slice_size(then_value_id));
            let then_len = self.get_slice_length(then_value_id);
            dbg!(then_len);
            self.new_slice_sizes.insert(then_value_id, (then_len, vec![]));
            then_len
        };
        // dbg!(else_value_id);
        // dbg!(new_slice_sizes.get(&else_value_id));
        let else_len = if let Some(val) =  self.new_slice_sizes.get(&else_value_id) {
            // dbg!(val.0);
            // dbg!(self.dfg.get_slice_size(else_value_id));
            val.0
        } else {
            dbg!(else_value_id);
            // dbg!(self.dfg.resolve(else_value_id));
            panic!("got none");
            dbg!(&self.dfg[else_value_id]);
            // dbg!(self.dfg.get_slice_size(else_value_id));
            let else_len = self.get_slice_length(else_value_id);
            dbg!(else_len);
            self.new_slice_sizes.insert(else_value_id, (else_len, vec![]));
            else_len
        };

        // let then_len = self.get_slice_length(then_value_id);
        // self.slice_sizes.insert(then_value_id, then_len);

        // let else_len = self.get_slice_length(else_value_id);
        // self.slice_sizes.insert(else_value_id, else_len);

        let len = then_len.max(else_len);
        // dbg!(then_len);
        // dbg!(else_len);
        if len == 0 {
            dbg!(then_value_id);
            dbg!(else_value_id);
        }
        dbg!(len);
        for i in 0..len {
            for (element_index, element_type) in element_types.iter().enumerate() {
                let index_usize = i * element_types.len() + element_index;
                let index_value = (index_usize as u128).into();
                let index = self.dfg.make_constant(index_value, Type::field());

                let typevars = Some(vec![element_type.clone()]);

                let mut get_element = |array, typevars, len| {
                    // The smaller slice is filled with placeholder data. Codegen for slice accesses must
                    // include checks against the dynamic slice length so that this placeholder data is not incorrectly accessed.

                    // NOTE: if we use the max size of the two rather than the actualy one 
                    if len <= index_usize {
                        dbg!("len is less than or equal to index");
                        self.make_slice_dummy_data(element_type)
                    } else {
                        let get = Instruction::ArrayGet { array, index };
                        let res = self.dfg
                            .insert_instruction_and_results(
                                get,
                                self.block,
                                typevars,
                                CallStack::new(),
                            )
                            .first();

                        // self.new_slice_sizes
                        
                        match element_type {
                            Type::Slice(_) => {
                                // dbg!(array);
                                // dbg!(res);
                                let res_sizes = self.new_slice_sizes.get(&res);
                                dbg!(res_sizes.is_none());
                                let inner_sizes = self.new_slice_sizes.get(&array).unwrap_or_else(|| panic!("should have slice sizes"));
                                // self.new_slice_sizes.insert(res, v);
                                let inner_sizes_iter = inner_sizes.1.clone();
                                for slice_value in inner_sizes_iter {
                                    let inner_slice = self.new_slice_sizes.get(&slice_value).unwrap_or_else(|| {
                                        panic!("ICE: should have inner slice set for {slice_value}")
                                    });
                                    // dbg!(inner_slice.0);
                                    // if let Some(previous_res_size) = self.new_slice_sizes.get(&res) {
                                    //     if inner_slice.0 > previous_res_size.0 {
                                    //         // self.new_slice_sizes.insert(res, inner_slice.clone());
                                    //     }
                                    //     // let mut x = inner_slice.1;
                                    //     // previous_res_size.1.append(&mut x);
                                    // }
                                    let previous_res_size = self.new_slice_sizes.get(&res);
                                    if let Some(previous_res_size) = previous_res_size {
                                        if inner_slice.0 > previous_res_size.0 {
                                            self.new_slice_sizes.insert(res, inner_slice.clone());
                                        }
                                    } else {
                                        self.new_slice_sizes.insert(res, inner_slice.clone());
                                    }
                                    // self.new_slice_sizes.insert(res, inner_slice.clone());

                                }
                            }
                            _ => {}
                        }
                        // dbg!(self.new_slice_sizes.get(&res));
                        res
                    }
                };

                let then_element =
                    get_element(then_value_id, typevars.clone(), then_len * element_types.len());
                let else_element =
                    get_element(else_value_id, typevars, else_len * element_types.len());

                merged.push_back(self.merge_values(
                    then_condition,
                    else_condition,
                    then_element,
                    else_element,
                ));
            }
        }

        // dbg!(self.inner_slice_sizes.clone());

        let merged = self.dfg.make_array(merged, typ);
        dbg!(merged);
        self.new_slice_sizes.insert(merged, (len, vec![]));
        merged
    }

    fn get_slice_length(
        &mut self, 
        value_id: ValueId,
        // slice_sizes: &HashMap<ValueId, (usize, Vec<ValueId>)>,
    ) -> usize {
        // dbg!(value_id);
        let value = &self.dfg[value_id];
        match value.clone() {
            Value::Array { array, typ } => {
                // self.compute_inner_slice_sizes(value_id, None, None);
                let element_size = typ.element_size();
                array.len() / element_size
            }
            Value::Instruction { instruction: instruction_id, .. } => {
                let instruction = &self.dfg[instruction_id];
                // dbg!(instruction.clone());
                match instruction {
                    // TODO(#3188): A slice can be the result of an ArrayGet when it is the
                    // fetched from a slice of slices or as a struct field.
                    // However, we need to incorporate nested slice support in flattening
                    // in order for this to be valid

                    // Instruction::ArrayGet { array, .. } => {
                    //     dbg!("GOT ARRAY GET");
                    //     // If the index is dynamic I do not know which value I am fetching
                    //     // and thus its size. Thus I need to find the max of the internal slices and use that
                    //     // This is not correct we want the size of the children
                    //     let x = self.new_slice_sizes.get(array).unwrap_or_else(|| panic!("should have slice sizes for {array}")).clone();
                    //     let mut max = 0;
                    //     // let mut new_inner_size = (max, vec![]);
                    //     for value in x.1.iter() {
                    //         let inner_size = self.new_slice_sizes.get(value).expect("ah");
                    //         // dbg!(inner_size);
                    //         if inner_size.0 > max {
                    //             max = inner_size.0;
                    //             // self.new_slice_sizes.insert(value_id, inner_size.clone());
                    //             // new_inner_size = inner_size.clone();
                    //         }
                    //     }
                    //     dbg!(x.0);
                    //     dbg!(x.1.len());
                    //     if x.1.len() == 0 {
                    //         dbg!(value_id);
                    //         dbg!(array);
                    //         // dbg!(self.dfg.type_of_value(*array));
                    //         // dbg!()
                    //     }
                    //     return max;
                    // }
                    
                    // Instruction::ArraySet { .. } => { dbg!("got array set"); return 0; }

                    // Instruction::ArraySet { array, .. } => {
                    //     let array = *array;
                    //     let len = self.get_slice_length(array, slice_sizes);
                    //     let array_typ = self.dfg.type_of_value(array);
                    //     let results = self.dfg.instruction_results(instruction_id);
                    //     if array_typ.contains_slice_element() {
                    //         dbg!(array);
                    //         let slice_sizes = self
                    //             .inner_slice_sizes
                    //             .get(&array)
                    //             .expect("ICE: expeted slice sizes")
                    //             .clone();
                    //         self.inner_slice_sizes.insert(results[0], slice_sizes);
                    //     }
                    //     self.slice_sizes.insert(array, len);
                    //     len
                    // }

                    // Instruction::Load { address } => {
                    //     let outer_block_stores = self.outer_block_stores.expect("ICE: A map of previous stores is required in order to resolve a slice load");
                    //     let store_values = self.store_values.expect("ICE: A map of previous stores is required in order to resolve a slice load");
                    //     let store_value = outer_block_stores
                    //         .get(address)
                    //         .expect("ICE: load in merger should have store from outer block");
                    //     if let Some(len) = self.slice_sizes.get(store_value) {
                    //         return *len;
                    //     }
                    //     let store_value = if let Some(store) = store_values.get(address) {
                    //         if let Some(len) = self.slice_sizes.get(&store.new_value) {
                    //             return *len;
                    //         }
                    //         store.new_value
                    //     } else {
                    //         *store_value
                    //     };
                    //     self.get_slice_length(store_value, slice_sizes)
                    // }

                    // Instruction::Call { func, arguments } => {
                    //     let slice_contents = arguments[1];
                    //     let func = &self.dfg[*func];
                    //     match func {
                    //         Value::Intrinsic(intrinsic) => match intrinsic {
                    //             Intrinsic::SlicePushBack
                    //             | Intrinsic::SlicePushFront
                    //             | Intrinsic::SliceInsert => {
                    //                 // `get_slice_length` needs to be called here as it is borrows self as mutable
                    //                 let initial_len = self.get_slice_length(slice_contents, slice_sizes);
                    //                 self.slice_sizes.insert(slice_contents, initial_len);
                    //                 initial_len + 1
                    //             }
                    //             Intrinsic::SlicePopBack
                    //             | Intrinsic::SlicePopFront
                    //             | Intrinsic::SliceRemove => {
                    //                 // `get_slice_length` needs to be called here as it is borrows self as mutable
                    //                 let initial_len = self.get_slice_length(slice_contents, slice_sizes);
                    //                 self.slice_sizes.insert(slice_contents, initial_len);
                    //                 initial_len - 1
                    //             }
                    //             _ => {
                    //                 unreachable!("ICE: Intrinsic not supported, got {intrinsic:?}")
                    //             }
                    //         },
                    //         _ => unreachable!("ICE: Expected intrinsic value but got {func:?}"),
                    //     }
                    // }
                    
                    _ => unreachable!("ICE: Got unexpected instruction: {instruction:?}"),
                }
            }
            _ => unreachable!("ICE: Got unexpected value when resolving slice length {value:?}"),
        }
    }

    fn compute_inner_slice_sizes(
        &mut self,
        current_array_id: ValueId,
        parent_array: Option<ValueId>,
        inner_parent_array: Option<ValueId>,
    ) {
        // annoying try and get rid of this clone
        match &self.dfg[current_array_id].clone() {
            Value::Array { array, typ } => {
                match typ {
                    Type::Slice(_) => {
                        // dbg!(array.len());
                        let element_size = typ.element_size();
                        let true_len = array.len() / element_size;
                        // dbg!(true_len);
                        if let Some(parent_array) = parent_array {
                            let sizes_list = self
                                .inner_slice_sizes
                                .get_mut(&parent_array)
                                .expect("ICE: expected size list");
                            let inner_parent_array =
                                inner_parent_array.expect("ICE: expected inner_parent_array");
                            sizes_list.push((true_len, Some(inner_parent_array)));
                            // self.new_slice_sizes.entry(parent_array).or_default().push(true_len)
                        } else {
                            // This means the current_array_id is the parent as well as the inner parent id
                            self.inner_slice_sizes.insert(current_array_id, vec![(true_len, None)]);
                        }
                        for (i, value) in array.iter().enumerate() {
                            let typ = self.dfg.type_of_value(*value);
                            match typ {
                                Type::Slice(_) => {
                                    if parent_array.is_some() {
                                        self.compute_inner_slice_sizes(
                                            *value,
                                            parent_array,
                                            Some(current_array_id),
                                        );
                                    } else {
                                        self.compute_inner_slice_sizes(
                                            *value,
                                            Some(current_array_id),
                                            Some(current_array_id),
                                        );
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Construct a dummy value to be attached to the smaller of two slices being merged.
    /// We need to make sure we follow the internal element type structure of the slice type
    /// even for dummy data to ensure that we do not have errors later in the compiler,
    /// such as with dynamic indexing of non-homogenous slices.
    fn make_slice_dummy_data(&mut self, typ: &Type) -> ValueId {
        match typ {
            Type::Numeric(_) => {
                let zero = FieldElement::zero();
                self.dfg.make_constant(zero, Type::field())
            }
            Type::Array(element_types, len) => {
                let mut array = im::Vector::new();
                for _ in 0..*len {
                    for typ in element_types.iter() {
                        array.push_back(self.make_slice_dummy_data(typ));
                    }
                }
                self.dfg.make_array(array, typ.clone())
            }
            Type::Slice(_) => {
                // TODO(#3188): Need to update flattening to use true user facing length of slices
                // to accurately construct dummy data
                unreachable!("ICE: Cannot return a slice of slices from an if expression")
            }
            Type::Reference(_) => {
                unreachable!("ICE: Merging references is unsupported")
            }
            Type::Function => {
                unreachable!("ICE: Merging functions is unsupported")
            }
        }
    }
}
