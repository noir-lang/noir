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

use super::{IResults, Interpreter, Value};

impl Interpreter<'_> {
    pub(super) fn call_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        mut args: Vec<Value>,
        results: &[ValueId],
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
                assert_eq!(args.len(), 2);

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
            Intrinsic::ApplyRangeConstraint => {
                todo!("Intrinsic::ApplyRangeConstraint is currently unimplemented")
            }
            // Both of these are no-ops
            Intrinsic::StrAsBytes | Intrinsic::AsWitness => {
                assert_eq!(args.len(), 1);
                let arg = args.pop().unwrap();
                Ok(vec![arg])
            }
            Intrinsic::ToBits(endian) => {
                assert_eq!(args.len(), 1);
                assert_eq!(results.len(), 1);
                let field = args[0].as_field().unwrap();
                self.to_radix(endian, field, 2, results[0])
            }
            Intrinsic::ToRadix(endian) => {
                assert_eq!(args.len(), 2);
                assert_eq!(results.len(), 1);
                let field = args[0].as_field().unwrap();
                let radix = args[1].as_u32().unwrap();
                self.to_radix(endian, field, radix, results[0])
            }
            Intrinsic::BlackBox(black_box_func) => {
                todo!("Intrinsic::BlackBox({black_box_func}) is currently unimplemented")
            }
            Intrinsic::Hint(_) => todo!("Intrinsic::Hint is currently unimplemented"),
            Intrinsic::IsUnconstrained => {
                assert_eq!(args.len(), 0);
                Ok(vec![Value::bool(self.in_unconstrained_context())])
            }
            Intrinsic::DerivePedersenGenerators => {
                todo!("Intrinsic::DerivePedersenGenerators is currently unimplemented")
            }
            Intrinsic::FieldLessThan => {
                assert!(
                    self.in_unconstrained_context(),
                    "FieldLessThan can only be called in unconstrained"
                );
                assert_eq!(args.len(), 2);
                let lhs = args[0].as_field().unwrap();
                let rhs = args[1].as_field().unwrap();
                Ok(vec![Value::bool(lhs < rhs)])
            }
            Intrinsic::ArrayRefCount | Intrinsic::SliceRefCount => {
                let array = args[0].as_array_or_slice().unwrap();
                let rc = *array.rc.borrow();
                Ok(vec![Value::from_constant(rc.into(), NumericType::unsigned(32))])
            }
        }
    }

    fn to_radix(
        &self,
        endian: Endian,
        field: FieldElement,
        radix: u32,
        result: ValueId,
    ) -> IResults {
        let Type::Array(_, limb_count) = self.dfg().type_of_value(result) else {
            unreachable!("Expected result of to_radix/to_bytes to be an array")
        };

        let Some(limbs) = dfg::simplify::constant_to_radix(endian, field, radix, limb_count) else {
            panic!("Unable to convert `{field}` to radix `{radix}`")
        };

        let elements = vecmap(limbs, |limb| Value::from_constant(limb, NumericType::unsigned(8)));
        Ok(vec![Value::array(elements, vec![Type::unsigned(8)])])
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

        let mut popped_elements = vecmap(0..element_types.len(), |_| slice_elements.pop().unwrap());
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
        let mut results = slice_elements.drain(0..element_types.len()).collect::<Vec<_>>();

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
