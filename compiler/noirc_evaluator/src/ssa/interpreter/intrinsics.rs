use acvm::FieldElement;
use iter_extended::vecmap;

use crate::ssa::{
    interpreter::NumericValue,
    ir::{
        dfg,
        instruction::{Endian, Intrinsic},
        types::{NumericType, Type},
        value::ValueId,
    },
};

use super::{ArrayValue, IResult, IResults, InternalError, Interpreter, InterpreterError, Value};

impl Interpreter<'_> {
    pub(super) fn call_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        args: &[ValueId],
        results: &[ValueId],
    ) -> IResults {
        match intrinsic {
            Intrinsic::ArrayLen => {
                check_argument_count(args, 1, intrinsic)?;
                let array = self.lookup_array_or_slice(args[0], "call to array_len")?;
                let length = array.elements.borrow().len();
                Ok(vec![Value::Numeric(NumericValue::U32(length as u32))])
            }
            Intrinsic::ArrayAsStrUnchecked => {
                check_argument_count(args, 1, intrinsic)?;
                Ok(vec![self.lookup(args[0])])
            }
            Intrinsic::AsSlice => {
                check_argument_count(args, 1, intrinsic)?;
                let array = self.lookup_array_or_slice(args[0], "call to as_slice")?;
                let length = array.elements.borrow().len();
                let length = Value::Numeric(NumericValue::U32(length as u32));

                let elements = array.elements.borrow().to_vec();
                let slice = Value::slice(elements, array.element_types.clone());
                Ok(vec![length, slice])
            }
            Intrinsic::AssertConstant => {
                // Nothing we can do here unfortunately if we want to allow code with
                // assert_constant to still pass pre-inlining and other optimizations.
                Ok(Vec::new())
            }
            Intrinsic::StaticAssert => {
                check_argument_count(args, 2, intrinsic)?;

                let condition = self.lookup_bool(args[0], "static_assert")?;
                if condition {
                    Ok(Vec::new())
                } else {
                    let message = self.lookup_string(args[1], "static_assert")?;
                    Err(InterpreterError::StaticAssertFailed { condition: args[0], message })
                }
            }
            Intrinsic::SlicePushBack => self.slice_push_back(args),
            Intrinsic::SlicePushFront => self.slice_push_front(args),
            Intrinsic::SlicePopBack => self.slice_pop_back(args),
            Intrinsic::SlicePopFront => self.slice_pop_front(args),
            Intrinsic::SliceInsert => self.slice_insert(args),
            Intrinsic::SliceRemove => self.slice_remove(args),
            Intrinsic::ApplyRangeConstraint => {
                todo!("Intrinsic::ApplyRangeConstraint is currently unimplemented")
            }
            // Both of these are no-ops
            Intrinsic::StrAsBytes | Intrinsic::AsWitness => {
                check_argument_count(args, 1, intrinsic)?;
                Ok(vec![self.lookup(args[0])])
            }
            Intrinsic::ToBits(endian) => {
                check_argument_count(args, 1, intrinsic)?;
                let field = self.lookup_field(args[0], "call to to_bits")?;
                self.to_radix(endian, args[0], field, 2, results[0])
            }
            Intrinsic::ToRadix(endian) => {
                check_argument_count(args, 2, intrinsic)?;
                let field = self.lookup_field(args[0], "call to to_bits")?;
                let radix = self.lookup_u32(args[1], "call to to_bits")?;
                self.to_radix(endian, args[0], field, radix, results[0])
            }
            Intrinsic::BlackBox(black_box_func) => {
                todo!("Intrinsic::BlackBox({black_box_func}) is currently unimplemented")
            }
            Intrinsic::Hint(_) => todo!("Intrinsic::Hint is currently unimplemented"),
            Intrinsic::IsUnconstrained => {
                check_argument_count(args, 0, intrinsic)?;
                Ok(vec![Value::bool(self.in_unconstrained_context())])
            }
            Intrinsic::DerivePedersenGenerators => {
                todo!("Intrinsic::DerivePedersenGenerators is currently unimplemented")
            }
            Intrinsic::FieldLessThan => {
                if !self.in_unconstrained_context() {
                    return Err(InterpreterError::Internal(
                        InternalError::FieldLessThanCalledInConstrainedContext,
                    ));
                }
                check_argument_count(args, 2, intrinsic)?;
                let lhs = self.lookup_field(args[0], "lhs of call to field less than")?;
                let rhs = self.lookup_field(args[1], "rhs of call to field less than")?;
                Ok(vec![Value::bool(lhs < rhs)])
            }
            Intrinsic::ArrayRefCount | Intrinsic::SliceRefCount => {
                let array = self.lookup_array_or_slice(args[0], "array/slice ref count")?;
                let rc = *array.rc.borrow();
                Ok(vec![Value::from_constant(rc.into(), NumericType::unsigned(32))])
            }
        }
    }

    fn to_radix(
        &self,
        endian: Endian,
        field_id: ValueId,
        field: FieldElement,
        radix: u32,
        result: ValueId,
    ) -> IResults {
        let result_type = self.dfg().type_of_value(result);
        let Type::Array(_, limb_count) = result_type else {
            return Err(InterpreterError::Internal(InternalError::TypeError {
                value_id: result,
                value: result_type.to_string(),
                expected_type: "array",
                instruction: "call to to_radix",
            }));
        };

        let Some(limbs) = dfg::simplify::constant_to_radix(endian, field, radix, limb_count) else {
            return Err(InterpreterError::ToRadixFailed { field_id, field, radix });
        };

        let elements = vecmap(limbs, |limb| Value::from_constant(limb, NumericType::unsigned(8)));
        Ok(vec![Value::array(elements, vec![Type::unsigned(8)])])
    }

    /// (length, slice, elem...) -> (length, slice)
    fn slice_push_back(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_push_back")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_push_back")?;

        // The resulting slice should be cloned - should we check RC here to try mutating it?
        // It'd need to be brillig-only if so since RC is always 1 in acir.
        let mut new_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        new_elements.extend(args.iter().skip(2).map(|arg| self.lookup(*arg)));

        let new_length = Value::Numeric(NumericValue::U32(length + 1));
        let new_slice = Value::slice(new_elements, element_types);
        Ok(vec![new_length, new_slice])
    }

    /// (length, slice, elem...) -> (length, slice)
    fn slice_push_front(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_push_front")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_push_front")?;
        let slice_elements = slice.elements.clone();
        let element_types = slice.element_types.clone();

        let mut new_elements = vecmap(args.iter().skip(2), |arg| self.lookup(*arg));
        new_elements.extend_from_slice(&slice_elements.borrow());

        let new_length = Value::Numeric(NumericValue::U32(length + 1));
        let new_slice = Value::slice(new_elements, element_types);
        Ok(vec![new_length, new_slice])
    }

    /// (length, slice) -> (length, slice, elem...)
    fn slice_pop_back(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_pop_back")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_pop_back")?;

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        if slice_elements.is_empty() {
            let instruction = "slice_pop_back";
            return Err(InterpreterError::PoppedFromEmptySlice { slice: args[1], instruction });
        }
        check_slice_can_pop_all_element_types(args[1], &slice)?;

        let mut popped_elements = vecmap(0..element_types.len(), |_| slice_elements.pop().unwrap());
        popped_elements.reverse();

        let new_length = Value::Numeric(NumericValue::U32(length - 1));
        let new_slice = Value::slice(slice_elements, element_types);
        let mut results = vec![new_length, new_slice];
        results.extend(popped_elements);
        Ok(results)
    }

    /// (length, slice) -> (elem..., length, slice)
    fn slice_pop_front(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_pop_front")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_pop_front")?;

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        if slice_elements.is_empty() {
            let instruction = "slice_pop_front";
            return Err(InterpreterError::PoppedFromEmptySlice { slice: args[1], instruction });
        }
        check_slice_can_pop_all_element_types(args[1], &slice)?;

        let mut results = slice_elements.drain(0..element_types.len()).collect::<Vec<_>>();

        let new_length = Value::Numeric(NumericValue::U32(length - 1));
        let new_slice = Value::slice(slice_elements, element_types);
        results.push(new_length);
        results.push(new_slice);
        Ok(results)
    }

    /// (length, slice, index:u32, elem...) -> (length, slice)
    fn slice_insert(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_insert")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_insert")?;
        let index = self.lookup_u32(args[2], "call to slice_insert")?;

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        let mut index = index as usize * element_types.len();
        for arg in args.iter().skip(3) {
            slice_elements.insert(index, self.lookup(*arg));
            index += 1;
        }

        let new_length = Value::Numeric(NumericValue::U32(length + 1));
        let new_slice = Value::slice(slice_elements, element_types);
        Ok(vec![new_length, new_slice])
    }

    /// (length, slice, index:u32) -> (length, slice, elem...)
    fn slice_remove(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_remove")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_remove")?;
        let index = self.lookup_u32(args[2], "call to slice_remove")?;

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        if slice_elements.is_empty() {
            let instruction = "slice_remove";
            return Err(InterpreterError::PoppedFromEmptySlice { slice: args[1], instruction });
        }
        check_slice_can_pop_all_element_types(args[1], &slice)?;

        let index = index as usize * element_types.len();
        let removed: Vec<_> = slice_elements.drain(index..index + element_types.len()).collect();

        let new_length = Value::Numeric(NumericValue::U32(length - 1));
        let new_slice = Value::slice(slice_elements, element_types);
        let mut results = vec![new_length, new_slice];
        results.extend(removed);
        Ok(results)
    }

    /// Print is not an intrinsic but it is treated like one.
    pub(super) fn call_print(&mut self, _args: Vec<Value>) -> IResults {
        // Stub the call for now
        Ok(Vec::new())
    }
}

fn check_argument_count(
    args: &[ValueId],
    expected_count: usize,
    intrinsic: Intrinsic,
) -> IResult<()> {
    if args.len() != expected_count {
        Err(InterpreterError::Internal(InternalError::IntrinsicArgumentCountMismatch {
            intrinsic,
            arguments: args.len(),
            parameters: expected_count,
        }))
    } else {
        Ok(())
    }
}

fn check_slice_can_pop_all_element_types(slice_id: ValueId, slice: &ArrayValue) -> IResult<()> {
    let actual_length = slice.elements.borrow().len();
    if actual_length >= slice.element_types.len() {
        Ok(())
    } else {
        Err(InterpreterError::Internal(InternalError::NotEnoughElementsToPopSliceOfStructs {
            slice_id,
            slice: slice.to_string(),
            actual_length,
            element_types: vecmap(slice.element_types.iter(), ToString::to_string),
        }))
    }
}
