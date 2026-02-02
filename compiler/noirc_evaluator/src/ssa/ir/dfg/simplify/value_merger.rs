use acvm::acir::brillig::lengths::SemanticLength;
use noirc_errors::{Location, call_stack::CallStackId};
use rustc_hash::FxHashMap as HashMap;

use crate::{
    brillig::assert_u32,
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
    vector_sizes: &'a HashMap<ValueId, SemanticLength>,

    call_stack: CallStackId,
}

impl<'a> ValueMerger<'a> {
    pub(crate) fn new(
        dfg: &'a mut DataFlowGraph,
        block: BasicBlockId,
        vector_sizes: &'a HashMap<ValueId, SemanticLength>,
        call_stack: CallStackId,
    ) -> Self {
        ValueMerger { dfg, block, vector_sizes, call_stack }
    }

    /// Choose a call stack to return with the [RuntimeError].
    ///
    /// If the call stack of the value is empty, it returns the call stack of the if-then-else itself.
    fn get_call_stack(&self, value: ValueId) -> Vec<Location> {
        // The value points at one of the problematic references, while the instruction would
        // point at where we got the if-then-else; it's not clear which one is more useful.
        let call_stack = self.dfg.get_value_call_stack(value);
        if call_stack.is_empty() { self.dfg.get_call_stack(self.call_stack) } else { call_stack }
    }

    /// Merge two values a and b to a single value.
    /// If these two values are numeric, the result will be
    /// `then_condition * (then_value - else_value) + else_value`.
    /// Otherwise, if the values being merged are arrays, a new array will be made
    /// recursively from combining each element of both input arrays.
    ///
    /// Returns an error if called with a function value or a reference or function values
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
            typ @ Type::Array(_, _) => self.merge_array_values_flat_nested(
                typ,
                then_condition,
                else_condition,
                then_value,
                else_value,
            ),
            typ @ Type::Vector(_) => self.merge_vector_values(
                typ,
                then_condition,
                else_condition,
                then_value,
                else_value,
            ),
            Type::Reference(_) => {
                let call_stack = self.get_call_stack(then_value);
                Err(RuntimeError::ReturnedReferenceFromDynamicIf { call_stack })
            }
            Type::Function => {
                let call_stack = self.get_call_stack(then_value);
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
    /// by creating a new array containing the result of self.merge_values for each element.
    pub(crate) fn merge_array_values_flat_nested(
        &mut self,
        typ: Type,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    ) -> Result<ValueId, RuntimeError> {
        let mut merged = im::Vector::new();

        assert!(matches!(&typ, Type::Array(..)));

        // TODO: Try to bring this back
        // let actual_length = len * element_types.len() as u32;
        // if let Some(result) = self.try_merge_only_changed_indices(
        //     then_condition,
        //     else_condition,
        //     then_value,
        //     else_value,
        //     actual_length,
        // ) {
        //     return result;
        // }

        let flat_typ = typ.clone().flatten();
        for (my_index, typ) in flat_typ.into_iter().enumerate() {
            let index = self.dfg.make_constant(my_index.into(), NumericType::length_type());
            assert!(matches!(typ, Type::Numeric(_)));
            let typevars = Some(vec![typ]);
            let mut get_element = |array, typevars: Option<Vec<Type>>| {
                let get = Instruction::ArrayGet { array, index };
                let res = self
                    .dfg
                    .insert_instruction_and_results(get, self.block, typevars, self.call_stack)
                    .first();

                let res_typ = self.dfg.type_of_value(res);
                assert!(
                    matches!(res_typ, Type::Numeric(_)),
                    "ICE: Array get is returning a non-numeric type. All arrays in ACIR work upon flat memory. Got {res_typ}"
                );
                res
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

        let flat_element_types_size =
            element_types.iter().fold(0u32, |acc, typ| acc + typ.flattened_size().0);

        let then_len = self.vector_sizes.get(&then_value_id).copied().unwrap_or_else(|| {
            let (vector, _) = self.dfg.get_array_constant(then_value_id).unwrap_or_else(|| {
                panic!("ICE: Merging values during flattening encountered vector {then_value_id} without a preset size");
            });
            SemanticLength(vector.len() as u32)
        });

        let else_len = self.vector_sizes.get(&else_value_id).copied().unwrap_or_else(|| {
            let (vector, _) = self.dfg.get_array_constant(else_value_id).unwrap_or_else(|| {
                panic!("ICE: Merging values during flattening encountered vector {else_value_id} without a preset size");
            });
            SemanticLength(vector.len() as u32)
        });
        let len = then_len.max(else_len);
        let composite_len = if flat_element_types_size == 0 {
            0
        } else {
            len.0 / flat_element_types_size
        };

        let flat_types: Vec<Type> = (0..composite_len)
            .flat_map(|_| element_types.iter().cloned().flat_map(Type::flatten))
            .collect();

        for (my_index, typ) in flat_types.into_iter().enumerate() {
            let index_u32 = my_index as u32;
            let index = self.dfg.make_constant(my_index.into(), NumericType::length_type());
            assert!(matches!(typ, Type::Numeric(_)) || matches!(typ, Type::Reference(_)));
            let typevars = Some(vec![typ]);

            let mut get_element = |array, typevars: Option<Vec<Type>>, len| {
                assert!(index_u32 < len, "get_element invoked with an out of bounds index");
                let get = Instruction::ArrayGet { array, index };
                let res = self
                    .dfg
                    .insert_instruction_and_results(
                        get,
                        self.block,
                        typevars.clone(),
                        self.call_stack,
                    )
                    .first();

                let res_typ = self.dfg.type_of_value(res);
                assert!(
                    matches!(res_typ, Type::Numeric(_) | Type::Reference(_)),
                    "ICE: Array get is returning a non-numeric type. All arrays in ACIR work upon flat memory. Got {res_typ}"
                );
                res
            };

            // If it's out of bounds for the "then" vector, a value in the "else" *must* exist.
            // We can use that value directly as accessing it is always checked against the actual
            // vector length.
            if index_u32 >= then_len.0 {
                let else_element = get_element(else_value_id, typevars, else_len.0);
                merged.push_back(else_element);
                continue;
            }

            // Same for if it's out of bounds for the "else" vector.
            if index_u32 >= else_len.0 {
                let then_element = get_element(then_value_id, typevars, then_len.0);
                merged.push_back(then_element);
                continue;
            }

            let then_element = get_element(then_value_id, typevars.clone(), then_len.0);
            let else_element = get_element(else_value_id, typevars, else_len.0);

            merged.push_back(self.merge_values(
                then_condition,
                else_condition,
                then_element,
                else_element,
            )?);
        }

        let instruction = Instruction::MakeArray { elements: merged, typ };
        let call_stack = self.call_stack;
        let result =
            self.dfg.insert_instruction_and_results(instruction, self.block, None, call_stack);
        Ok(result.first())
    }
}
