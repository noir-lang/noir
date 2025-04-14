use std::sync::Arc;

use acvm::{AcirField, FieldElement};
use noirc_frontend::Shared;

use crate::ssa::ir::{
    function::FunctionId,
    instruction::Intrinsic,
    types::{CompositeType, NumericType, Type},
    value::ValueId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
    Numeric(NumericValue),
    Reference(ReferenceValue),
    ArrayOrSlice(ArrayValue),
    Function(FunctionId),
    Intrinsic(Intrinsic),
    ForeignFunction(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum NumericValue {
    Field(FieldElement),

    U1(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceValue {
    /// This is included mostly for debugging to distinguish different
    /// ReferenceValues which store the same element.
    pub original_id: ValueId,

    pub element: Shared<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArrayValue {
    pub elements: Shared<Vec<Value>>,

    /// The `Shared` type contains its own reference count but we need to track
    /// the reference count separate to ensure it is only changed by IncrementRc and
    /// DecrementRc instructions.
    pub rc: Shared<u32>,

    pub element_types: Arc<CompositeType>,
    pub is_slice: bool,
}

impl Value {
    pub(crate) fn get_type(&self) -> Type {
        match self {
            Value::Numeric(numeric_value) => Type::Numeric(numeric_value.get_type()),
            Value::Reference(reference) => {
                Type::Reference(Arc::new(reference.element.borrow().get_type()))
            }
            Value::ArrayOrSlice(array) if array.is_slice => {
                Type::Slice(array.element_types.clone())
            }
            Value::ArrayOrSlice(array) => {
                Type::Array(array.element_types.clone(), array.elements.borrow().len() as u32)
            }
            Value::Function(_) | Value::Intrinsic(_) | Value::ForeignFunction(_) => Type::Function,
        }
    }

    pub(crate) fn reference(original_id: ValueId, element: Shared<Value>) -> Self {
        Value::Reference(ReferenceValue { original_id, element })
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Numeric(NumericValue::U1(value)) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_numeric(&self) -> Option<NumericValue> {
        match self {
            Value::Numeric(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn from_constant(constant: FieldElement, typ: NumericType) -> Self {
        Self::Numeric(NumericValue::from_constant(constant, typ))
    }
}

impl NumericValue {
    pub(crate) fn get_type(&self) -> NumericType {
        match self {
            NumericValue::Field(_) => NumericType::NativeField,
            NumericValue::U1(_) => NumericType::unsigned(1),
            NumericValue::U8(_) => NumericType::unsigned(8),
            NumericValue::U16(_) => NumericType::unsigned(16),
            NumericValue::U32(_) => NumericType::unsigned(32),
            NumericValue::U64(_) => NumericType::unsigned(64),
            NumericValue::U128(_) => NumericType::unsigned(128),
            NumericValue::I8(_) => NumericType::signed(8),
            NumericValue::I16(_) => NumericType::signed(16),
            NumericValue::I32(_) => NumericType::signed(32),
            NumericValue::I64(_) => NumericType::signed(64),
        }
    }

    pub(crate) fn as_field(&self) -> Option<FieldElement> {
        match self {
            NumericValue::Field(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            NumericValue::U1(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn from_constant(constant: FieldElement, typ: NumericType) -> NumericValue {
        match typ {
            NumericType::NativeField => Self::Field(constant),
            NumericType::Unsigned { bit_size: 1 } => Self::U1(constant.is_one()),
            NumericType::Unsigned { bit_size: 8 } => {
                Self::U8(constant.try_into_u128().unwrap().try_into().unwrap())
            }
            NumericType::Unsigned { bit_size: 16 } => {
                Self::U16(constant.try_into_u128().unwrap().try_into().unwrap())
            }
            NumericType::Unsigned { bit_size: 32 } => {
                Self::U32(constant.try_into_u128().unwrap().try_into().unwrap())
            }
            NumericType::Unsigned { bit_size: 64 } => {
                Self::U64(constant.try_into_u128().unwrap().try_into().unwrap())
            }
            NumericType::Unsigned { bit_size: 128 } => {
                Self::U128(constant.try_into_u128().unwrap())
            }
            NumericType::Signed { bit_size: 8 } => {
                Self::I8(constant.try_into_i128().unwrap().try_into().unwrap())
            }
            NumericType::Signed { bit_size: 16 } => {
                Self::I16(constant.try_into_i128().unwrap().try_into().unwrap())
            }
            NumericType::Signed { bit_size: 32 } => {
                Self::I32(constant.try_into_i128().unwrap().try_into().unwrap())
            }
            NumericType::Signed { bit_size: 64 } => {
                Self::I64(constant.try_into_i128().unwrap().try_into().unwrap())
            }
            other => panic!("Unsupported numeric type: {other}"),
        }
    }

    pub(crate) fn convert_to_field(&self) -> FieldElement {
        match self {
            NumericValue::Field(field) => *field,
            NumericValue::U1(boolean) if *boolean => FieldElement::one(),
            NumericValue::U1(_) => FieldElement::zero(),
            NumericValue::U8(value) => FieldElement::from(*value),
            NumericValue::U16(value) => FieldElement::from(*value),
            NumericValue::U32(value) => FieldElement::from(*value),
            NumericValue::U64(value) => FieldElement::from(*value),
            NumericValue::U128(value) => FieldElement::from(*value),
            NumericValue::I8(value) => FieldElement::from(*value),
            NumericValue::I16(value) => FieldElement::from(*value),
            NumericValue::I32(value) => FieldElement::from(*value),
            NumericValue::I64(value) => FieldElement::from(*value),
        }
    }
}
