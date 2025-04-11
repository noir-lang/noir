use std::sync::Arc;

use acvm::FieldElement;
use noirc_frontend::Shared;

use crate::ssa::ir::{
    function::FunctionId,
    types::{CompositeType, NumericType, Type},
    value::ValueId,
};

#[derive(Debug, Clone)]
pub enum Value {
    Numeric(NumericValue),
    Reference(ReferenceValue),
    ArrayOrSlice(ArrayValue),
    Function(FunctionId),
}

#[derive(Debug, Copy, Clone)]
pub enum NumericValue {
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

#[derive(Debug, Clone)]
pub struct ReferenceValue {
    /// This is included mostly for debugging to distinguish different
    /// ReferenceValues which store the same element.
    pub original_id: ValueId,

    pub element: Shared<Value>,
}

#[derive(Debug, Clone)]
pub struct ArrayValue {
    pub elements: Shared<Vec<Value>>,

    /// The `Shared` type contains its own reference count but we need to track
    /// the reference count separate to ensure it is only changed by IncrementRc and
    /// DecrementRc instructions.
    pub rc: Shared<u32>,

    pub element_types: Arc<CompositeType>,
    pub is_slice: bool,
}

impl Value {
    pub fn get_type(&self) -> Type {
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
            Value::Function(_) => Type::Function,
        }
    }

    pub fn reference(original_id: ValueId, element: Shared<Value>) -> Self {
        Value::Reference(ReferenceValue { original_id, element })
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Numeric(NumericValue::U1(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn as_numeric(&self) -> Option<NumericValue> {
        match self {
            Value::Numeric(value) => Some(*value),
            _ => None,
        }
    }
}

impl NumericValue {
    pub fn get_type(&self) -> NumericType {
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

    pub fn as_field(&self) -> Option<FieldElement> {
        match self {
            NumericValue::Field(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            NumericValue::U1(value) => Some(*value),
            _ => None,
        }
    }
}
