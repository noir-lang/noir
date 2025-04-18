use iter_extended::vecmap;

use crate::ssa::{
    interpreter::NumericValue,
    ir::{instruction::Intrinsic, value::ValueId},
};

use super::{IResults, Interpreter, Value};

impl Interpreter<'_> {
    pub(super) fn call_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        args: Vec<Value>,
        original_args: &[ValueId],
    ) -> IResults {
        match intrinsic {
            Intrinsic::ArrayLen => {
                assert_eq!(args.len(), 1);
                let length = args[0].as_array_or_slice().unwrap().elements.borrow().len();
                Ok(vec![Value::Numeric(NumericValue::U32(length as u32))])
            }
            Intrinsic::ArrayAsStrUnchecked => {
                assert_eq!(args.len(), 1);
                Ok(args)
            }
            Intrinsic::AsSlice => {
                assert_eq!(args.len(), 1);
                let array = args[0].as_array_or_slice().unwrap();
                let length = args[0].as_array_or_slice().unwrap().elements.borrow().len();
                let length = Value::Numeric(NumericValue::U32(length as u32));

                let elements = array.elements.borrow().to_vec();
                let slice = Value::slice(elements, array.element_types.clone());
                Ok(vec![length, slice])
            }
            Intrinsic::AssertConstant => {
                for arg in original_args {
                    if !self.is_constant(*arg) {
                        panic!("assert_constant: {arg} is not constant");
                    }
                }
                Ok(Vec::new())
            }
            Intrinsic::StaticAssert => {
                assert_eq!(args.len(), 2);

                if !self.is_constant(original_args[0]) {
                    panic!("static_assert: {} is not constant", original_args[0]);
                }

                let condition = args[0].as_bool().unwrap();
                if !condition {
                    let message = args[1].as_string().unwrap();
                    panic!("static_assert failed: {message}");
                }

                Ok(Vec::new())
            }
            Intrinsic::SlicePushBack => self.slice_push_back(args),
            Intrinsic::SlicePushFront => self.slice_push_front(args),
            Intrinsic::SlicePopBack => self.slice_pop_back(args),
            Intrinsic::SlicePopFront => self.slice_pop_front(args),
            Intrinsic::SliceInsert => self.slice_insert(args),
            Intrinsic::SliceRemove => self.slice_remove(args),
            Intrinsic::ApplyRangeConstraint => todo!(),
            Intrinsic::StrAsBytes => todo!(),
            Intrinsic::ToBits(_endian) => todo!(),
            Intrinsic::ToRadix(_endian) => todo!(),
            Intrinsic::BlackBox(_black_box_func) => todo!(),
            Intrinsic::Hint(_hint) => todo!(),
            Intrinsic::AsWitness => todo!(),
            Intrinsic::IsUnconstrained => todo!(),
            Intrinsic::DerivePedersenGenerators => todo!(),
            Intrinsic::FieldLessThan => todo!(),
            Intrinsic::ArrayRefCount => todo!(),
            Intrinsic::SliceRefCount => todo!(),
        }
    }

    fn is_constant(&self, value_id: ValueId) -> bool {
        match self.dfg()[value_id] {
            crate::ssa::ir::value::Value::Instruction { .. }
            | crate::ssa::ir::value::Value::Param { .. } => {
                false
            }
            crate::ssa::ir::value::Value::NumericConstant { .. }
            | crate::ssa::ir::value::Value::Function(_)
            | crate::ssa::ir::value::Value::Intrinsic(_)
            | crate::ssa::ir::value::Value::ForeignFunction(_)
            | crate::ssa::ir::value::Value::Global(_) => true,
        }
    }

    /// (length, slice, elem...) -> (length, slice)
    fn slice_push_back(&self, args: Vec<Value>) -> IResults {
        let length = args[0].as_u32().unwrap();
        let slice = args[1].as_array_or_slice().unwrap();

        // The resulting slice should be cloned - should we check RC here to try mutating it?
        // It'd need to be brillig-only if so since RC is always 1 in acir.
        let mut new_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        new_elements.extend(args.into_iter().skip(2));

        let new_length = Value::Numeric(NumericValue::U32(length + 1));
        let new_slice = Value::slice(new_elements, element_types);
        Ok(vec![new_length, new_slice])
    }

    /// (length, slice, elem...) -> (length, slice)
    fn slice_push_front(&self, args: Vec<Value>) -> IResults {
        let length = args[0].as_u32().unwrap();
        let slice = args[1].as_array_or_slice().unwrap();
        let slice_elements = slice.elements.clone();
        let element_types = slice.element_types.clone();

        let mut new_elements = args.into_iter().skip(2).collect::<Vec<_>>();
        new_elements.extend_from_slice(&slice_elements.borrow());

        let new_length = Value::Numeric(NumericValue::U32(length + 1));
        let new_slice = Value::slice(new_elements, element_types);
        Ok(vec![new_length, new_slice])
    }

    /// (length, slice) -> (length, slice, elem...)
    fn slice_pop_back(&self, args: Vec<Value>) -> IResults {
        let length = args[0].as_u32().unwrap();
        let slice = args[1].as_array_or_slice().unwrap();

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        if slice_elements.is_empty() {
            panic!("slice_pop_back: empty slice");
        }

        assert!(slice_elements.len() >= element_types.len());

        let mut popped_elements = vecmap(0 .. element_types.len(), |_| {
            slice_elements.pop().unwrap()
        });
        popped_elements.reverse();

        let new_length = Value::Numeric(NumericValue::U32(length - 1));
        let new_slice = Value::slice(slice_elements, element_types);
        let mut results = vec![new_length, new_slice];
        results.extend(popped_elements);
        Ok(results)
    }

    /// (length, slice) -> (elem..., length, slice)
    fn slice_pop_front(&self, args: Vec<Value>) -> IResults {
        let length = args[0].as_u32().unwrap();
        let slice = args[1].as_array_or_slice().unwrap();

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        if slice_elements.is_empty() {
            panic!("slice_pop_front: empty slice");
        }

        assert!(slice_elements.len() >= element_types.len());
        let mut results = slice_elements.drain(0 .. element_types.len()).collect::<Vec<_>>();

        let new_length = Value::Numeric(NumericValue::U32(length - 1));
        let new_slice = Value::slice(slice_elements, element_types);
        results.push(new_length);
        results.push(new_slice);
        Ok(results)
    }

    /// (length, slice, index:u32, elem...) -> (length, slice)
    fn slice_insert(&self, args: Vec<Value>) -> IResults {
        let length = args[0].as_u32().unwrap();
        let slice = args[1].as_array_or_slice().unwrap();
        let index = args[2].as_u32().unwrap();

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        let mut index = index as usize * element_types.len();
        for arg in args.into_iter().skip(3) {
            slice_elements.insert(index, arg);
            index += 1;
        }

        let new_length = Value::Numeric(NumericValue::U32(length + 1));
        let new_slice = Value::slice(slice_elements, element_types);
        Ok(vec![new_length, new_slice])
    }

    /// (length, slice, index:u32) -> (length, slice, elem...)
    fn slice_remove(&self, args: Vec<Value>) -> IResults {
        let length = args[0].as_u32().unwrap();
        let slice = args[1].as_array_or_slice().unwrap();
        let index = args[2].as_u32().unwrap();

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        if slice_elements.is_empty() {
            panic!("slice_remove: empty slice");
        }
        assert!(slice_elements.len() >= element_types.len());

        let index = index as usize * element_types.len();
        let removed: Vec<_> = slice_elements.drain(index .. index + element_types.len()).collect();

        let new_length = Value::Numeric(NumericValue::U32(length - 1));
        let new_slice = Value::slice(slice_elements, element_types);
        let mut results = vec![new_length, new_slice];
        results.extend(removed);
        Ok(results)
    }
}
