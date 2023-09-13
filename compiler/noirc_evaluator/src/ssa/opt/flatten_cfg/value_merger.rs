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
    store_values: &'a HashMap<ValueId, Store>,
    outer_block_stores: &'a HashMap<ValueId, Store>,
    slice_sizes: HashMap<ValueId, usize>,
}

impl<'a> ValueMerger<'a> {
    pub(crate) fn new(
        dfg: &'a mut DataFlowGraph,
        block: BasicBlockId,
        store_values: &'a HashMap<ValueId, Store>,
        outer_block_stores: &'a HashMap<ValueId, Store>,
    ) -> Self {
        ValueMerger {
            dfg,
            block,
            store_values,
            outer_block_stores,
            slice_sizes: HashMap::default(),
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
            Type::Reference => panic!("Cannot return references from an if expression"),
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

        let call_stack = if then_call_stack.is_empty() {
            else_call_stack.clone()
        } else {
            then_call_stack.clone()
        };

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
                ))
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

        dbg!(then_value_id);
        let then_len = self.get_slice_length(then_value_id);
        dbg!(then_len);
        dbg!(self.slice_sizes.insert(then_value_id, then_len));

        dbg!(else_value_id);
        let else_len = self.get_slice_length(else_value_id);
        dbg!(else_len);
        dbg!(self.slice_sizes.insert(else_value_id, else_len));

        let len = then_len.max(else_len);

        for i in 0..len {
            for (element_index, element_type) in element_types.iter().enumerate() {
                let index_value = ((i * element_types.len() + element_index) as u128).into();
                let index = self.dfg.make_constant(index_value, Type::field());

                let typevars = Some(vec![element_type.clone()]);

                let mut get_element = |array, typevars, len| {
                    // The smaller slice is filled with placeholder data. Codegen for slice accesses must
                    // include checks against the dynamic slice length so that this placeholder data is not incorrectly accessed.
                    if (len - 1) < index_value.to_u128() as usize {
                        let zero = FieldElement::zero();
                        self.dfg.make_constant(zero, Type::field())
                    } else {
                        let get = Instruction::ArrayGet { array, index };
                        self.dfg
                            .insert_instruction_and_results(
                                get,
                                self.block,
                                typevars,
                                CallStack::new(),
                            )
                            .first()
                    }
                };

                let then_element = get_element(then_value_id, typevars.clone(), then_len);
                let else_element = get_element(else_value_id, typevars, else_len);

                merged.push_back(self.merge_values(
                    then_condition,
                    else_condition,
                    then_element,
                    else_element,
                ));
            }
        }
        dbg!(merged.len());
        let merged = self.dfg.make_array(merged, typ);
        dbg!(merged);
        merged
    }

    fn get_slice_length(&mut self, value_id: ValueId) -> usize {
        let value = &self.dfg[value_id];
        match value {
            Value::Array { array, .. } => array.len(),
            Value::Instruction { instruction: instruction_id, .. } => {
                let instruction = &self.dfg[*instruction_id];
                match instruction {
                    Instruction::ArraySet { array, .. } => {
                        dbg!("got an array set");
                        let array = *array;
                        let len = self.get_slice_length(array);
                        dbg!(self.slice_sizes.insert(array, len));
                        len
                    }
                    Instruction::Load { address } => {
                        let outer_store = self
                            .outer_block_stores
                            .get(address)
                            .expect("ICE: load in merger should have store from outer block");
                        dbg!(self.slice_sizes.get(&outer_store.new_value));
                        if let Some(len) = self.slice_sizes.get(&outer_store.new_value) {
                            dbg!(outer_store.new_value);
                            dbg!(len);
                            return *len;
                        }

                        let context_store = if let Some(store) = self.store_values.get(address) {
                            if let Some(len) = self.slice_sizes.get(&store.new_value) {
                                return *len;
                            }

                            store
                        } else {
                            outer_store
                        };

                        self.get_slice_length(context_store.new_value)
                    }
                    Instruction::Call { func, arguments } => {
                        let func = &self.dfg[*func];
                        let slice_contents = arguments[1];
                        match func {
                            Value::Intrinsic(intrinsic) => match intrinsic {
                                Intrinsic::SlicePushBack
                                | Intrinsic::SlicePushFront
                                | Intrinsic::SliceInsert => {
                                    dbg!("about to call get_slice_length");
                                    let initial_len = self.get_slice_length(slice_contents);
                                    dbg!(self.slice_sizes.insert(slice_contents, initial_len));
                                    initial_len + 1
                                }
                                Intrinsic::SlicePopBack
                                | Intrinsic::SlicePopFront
                                | Intrinsic::SliceRemove => {
                                    // TODO: for some reason this breaks lol
                                    // let initial_len = self.get_slice_length(slice_contents);
                                    // self.slice_sizes.insert(slice_contents, initial_len);
                                    // initial_len + 1
                                    self.get_slice_length(slice_contents) - 1
                                }
                                _ => {
                                    unreachable!("ICE: Intrinsic not supported, got {intrinsic:?}")
                                }
                            },
                            _ => unreachable!("ICE: Expected intrinsic value but got {func:?}"),
                        }
                    }
                    _ => unreachable!("ICE: Got unexpected instruction: {instruction:?}"),
                }
            }
            _ => unreachable!("ICE: Got unexpected value when resolving slice length {value:?}"),
        }
    }
}
