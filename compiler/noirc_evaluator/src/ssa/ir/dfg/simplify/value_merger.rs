use noirc_errors::call_stack::CallStackId;
use rustc_hash::FxHashMap as HashMap;

use crate::{
    errors::{RtResult, RuntimeError},
    ssa::ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        instruction::{BinaryOp, Instruction},
        types::{NumericType, Type},
        value::ValueId,
    },
};

pub(crate) struct ValueMerger<'a> {
    dfg: &'a mut DataFlowGraph,
    block: BasicBlockId,

    /// Maps SSA array values with a vector type to their size.
    /// This must be computed before merging values.
    vector_sizes: &'a HashMap<ValueId, u32>,

    call_stack: CallStackId,
}

impl<'a> ValueMerger<'a> {
    pub(crate) fn new(
        dfg: &'a mut DataFlowGraph,
        block: BasicBlockId,
        vector_sizes: &'a HashMap<ValueId, u32>,
        call_stack: CallStackId,
    ) -> Self {
        ValueMerger { dfg, block, vector_sizes, call_stack }
    }

    /// Merge two values a and b to a single value.
    /// If these two values are numeric, the result will be
    /// `then_condition * (then_value - else_value) + else_value`.
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
    ) -> RtResult<ValueId> {
        if then_value == else_value {
            return Ok(then_value);
        }

        match self.dfg.type_of_value(then_value) {
            Type::Numeric(_) => Ok(Self::merge_numeric_values(
                self.dfg,
                self.block,
                then_condition,
                else_condition,
                then_value,
                else_value,
            )),
            typ @ Type::Array(_, _) => {
                self.merge_array_values(typ, then_condition, else_condition, then_value, else_value)
            }
            typ @ Type::Vector(_) => self.merge_vector_values(
                typ,
                then_condition,
                else_condition,
                then_value,
                else_value,
            ),
            Type::Reference(_) => {
                // FIXME: none of then_value, else_value, then_condition, or else_condition have
                // non-empty call stacks
                let call_stack = self.dfg.get_value_call_stack(then_value);
                Err(RuntimeError::ReturnedReferenceFromDynamicIf { call_stack })
            }
            Type::Function => {
                let call_stack = self.dfg.get_value_call_stack(then_value);
                Err(RuntimeError::ReturnedFunctionFromDynamicIf { call_stack })
            }
        }
    }

    /// Merge two numeric values a and b from separate basic blocks to a single value. This
    /// function would return the result of `if c { a } else { b }` as  `c*a + (!c)*b`.
    pub(crate) fn merge_numeric_values(
        dfg: &mut DataFlowGraph,
        block: BasicBlockId,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    ) -> ValueId {
        let then_type = dfg.type_of_value(then_value).unwrap_numeric();
        let else_type = dfg.type_of_value(else_value).unwrap_numeric();
        assert_eq!(
            then_type, else_type,
            "Expected values merged to be of the same type but found {then_type} and {else_type}"
        );

        if then_value == else_value {
            return then_value;
        }

        let then_call_stack = dfg.get_value_call_stack_id(then_value);
        let else_call_stack = dfg.get_value_call_stack_id(else_value);

        let call_stack = if then_call_stack.is_root() { else_call_stack } else { then_call_stack };

        // We must cast the bool conditions to the actual numeric type used by each value.
        let cast = Instruction::Cast(then_condition, then_type);
        let then_condition =
            dfg.insert_instruction_and_results(cast, block, None, call_stack).first();

        let cast = Instruction::Cast(else_condition, else_type);
        let else_condition =
            dfg.insert_instruction_and_results(cast, block, None, call_stack).first();

        // Unchecked mul because `then_condition` will be 1 or 0
        let mul =
            Instruction::binary(BinaryOp::Mul { unchecked: true }, then_condition, then_value);
        let then_value = dfg.insert_instruction_and_results(mul, block, None, call_stack).first();

        // Unchecked mul because `else_condition` will be 1 or 0
        let mul =
            Instruction::binary(BinaryOp::Mul { unchecked: true }, else_condition, else_value);
        let else_value = dfg.insert_instruction_and_results(mul, block, None, call_stack).first();

        // Unchecked add because one of the values will always be 0
        let add = Instruction::binary(BinaryOp::Add { unchecked: true }, then_value, else_value);
        dfg.insert_instruction_and_results(add, block, None, call_stack).first()
    }

    /// Given an if expression that returns an array: `if c { array1 } else { array2 }`,
    /// this function will recursively merge array1 and array2 into a single resulting array
    /// by creating a new array containing the result of `self.merge_values` for each element.
    pub(crate) fn merge_array_values(
        &mut self,
        typ: Type,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    ) -> Result<ValueId, RuntimeError> {
        let mut merged = im::Vector::new();

        let (element_types, len) = match &typ {
            Type::Array(elements, len) => (elements.as_slice(), *len),
            _ => panic!("Expected array type"),
        };

        let element_count = element_types.len() as u32;

        for i in 0..len {
            for (element_index, element_type) in element_types.iter().enumerate() {
                let index = u128::from(i * element_count + element_index as u32).into();
                let index = self.dfg.make_constant(index, NumericType::length_type());

                let typevars = Some(vec![element_type.clone()]);

                let mut get_element = |array, typevars| {
                    let get = Instruction::ArrayGet { array, index };
                    self.dfg
                        .insert_instruction_and_results(get, self.block, typevars, self.call_stack)
                        .first()
                };

                let then_element = get_element(then_value, typevars.clone());
                let else_element = get_element(else_value, typevars);

                merged.push_back(self.merge_values(
                    then_condition,
                    else_condition,
                    then_element,
                    else_element,
                )?);
            }
        }

        let instruction = Instruction::MakeArray { elements: merged, typ };
        let result =
            self.dfg.insert_instruction_and_results(instruction, self.block, None, self.call_stack);
        Ok(result.first())
    }

    fn merge_vector_values(
        &mut self,
        typ: Type,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value_id: ValueId,
        else_value_id: ValueId,
    ) -> Result<ValueId, RuntimeError> {
        let mut merged = im::Vector::new();

        let element_types = match &typ {
            Type::Vector(elements) => elements.as_slice(),
            _ => panic!("Expected vector type"),
        };

        let then_len = self.vector_sizes.get(&then_value_id).copied().unwrap_or_else(|| {
            panic!("ICE: Merging values during flattening encountered vector {then_value_id} without a preset size");
        });

        let else_len = self.vector_sizes.get(&else_value_id).copied().unwrap_or_else(|| {
            panic!("ICE: Merging values during flattening encountered vector {else_value_id} without a preset size");
        });

        let len = then_len.max(else_len);
        let element_count = element_types.len() as u32;

        let flat_then_length = then_len * element_types.len() as u32;
        let flat_else_length = else_len * element_types.len() as u32;

        for i in 0..len {
            for (element_index, element_type) in element_types.iter().enumerate() {
                let index_u32 = i * element_count + element_index as u32;
                let index_value = u128::from(index_u32).into();
                let index = self.dfg.make_constant(index_value, NumericType::length_type());

                let typevars = Some(vec![element_type.clone()]);

                let mut get_element = |array, typevars, len| {
                    assert!(index_u32 < len, "get_element invoked with an out of bounds index");
                    let get = Instruction::ArrayGet { array, index };
                    let results = self.dfg.insert_instruction_and_results(
                        get,
                        self.block,
                        typevars,
                        self.call_stack,
                    );
                    results.first()
                };

                // If it's out of bounds for the "then" vector, a value in the "else" *must* exist.
                // We can use that value directly as accessing it is always checked against the actual
                // vector length.
                if index_u32 >= flat_then_length {
                    let else_element = get_element(else_value_id, typevars, flat_else_length);
                    merged.push_back(else_element);
                    continue;
                }

                // Same for if it's out of bounds for the "else" vector.
                if index_u32 >= flat_else_length {
                    let then_element = get_element(then_value_id, typevars, flat_then_length);
                    merged.push_back(then_element);
                    continue;
                }

                let then_element = get_element(then_value_id, typevars.clone(), flat_then_length);
                let else_element = get_element(else_value_id, typevars, flat_else_length);

                merged.push_back(self.merge_values(
                    then_condition,
                    else_condition,
                    then_element,
                    else_element,
                )?);
            }
        }

        let instruction = Instruction::MakeArray { elements: merged, typ };
        let call_stack = self.call_stack;
        let result =
            self.dfg.insert_instruction_and_results(instruction, self.block, None, call_stack);
        Ok(result.first())
    }
}
