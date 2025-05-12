use std::sync::Arc;

use acvm::{AcirField, FieldElement};
use iter_extended::vecmap;
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

    /// A value of `None` here means this allocation is currently uninitialized
    pub element: Shared<Option<Value>>,

    pub element_type: Arc<Type>,
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
    #[allow(unused)]
    pub(crate) fn get_type(&self) -> Type {
        match self {
            Value::Numeric(numeric_value) => Type::Numeric(numeric_value.get_type()),
            Value::Reference(reference) => Type::Reference(reference.element_type.clone()),
            Value::ArrayOrSlice(array) if array.is_slice => {
                Type::Slice(array.element_types.clone())
            }
            Value::ArrayOrSlice(array) => {
                Type::Array(array.element_types.clone(), array.elements.borrow().len() as u32)
            }
            Value::Function(_) | Value::Intrinsic(_) | Value::ForeignFunction(_) => Type::Function,
        }
    }

    pub(crate) fn reference(original_id: ValueId, element_type: Arc<Type>) -> Self {
        Value::Reference(ReferenceValue { original_id, element_type, element: Shared::new(None) })
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Numeric(NumericValue::U1(value)) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_u8(&self) -> Option<u8> {
        match self {
            Value::Numeric(NumericValue::U8(value)) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_u32(&self) -> Option<u32> {
        match self {
            Value::Numeric(NumericValue::U32(value)) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Numeric(NumericValue::U64(value)) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_numeric(&self) -> Option<NumericValue> {
        match self {
            Value::Numeric(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_reference(&self) -> Option<ReferenceValue> {
        match self {
            Value::Reference(value) => Some(value.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_array_or_slice(&self) -> Option<ArrayValue> {
        match self {
            Value::ArrayOrSlice(value) => Some(value.clone()),
            _ => None,
        }
    }

    pub(crate) fn from_constant(constant: FieldElement, typ: NumericType) -> Self {
        Self::Numeric(NumericValue::from_constant(constant, typ))
    }

    pub(crate) fn array_from_slice(slice: &[FieldElement], typ: NumericType) -> Self {
        let values =
            slice.iter().map(|v| Value::Numeric(NumericValue::from_constant(*v, typ))).collect();
        let types = slice.iter().map(|_| Type::Numeric(typ)).collect();
        Self::array(values, types)
    }

    // This is used in tests but shouldn't be cfg(test) only
    #[allow(unused)]
    pub(crate) fn bool(value: bool) -> Self {
        Self::Numeric(NumericValue::U1(value))
    }

    pub(crate) fn array(elements: Vec<Value>, element_types: Vec<Type>) -> Self {
        Self::ArrayOrSlice(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types: Arc::new(element_types),
            is_slice: false,
        })
    }

    pub(crate) fn slice(elements: Vec<Value>, element_types: Arc<Vec<Type>>) -> Self {
        Self::ArrayOrSlice(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types,
            is_slice: true,
        })
    }

    /// Return an uninitialized value of the given type. This is usually a zeroed
    /// value but we make no guarantee that it is. This is often used as the default
    /// value to return for side-effectful functions like `call` or `array_get` when
    /// side-effects are disabled.
    pub(crate) fn uninitialized(typ: &Type, id: ValueId) -> Value {
        match typ {
            Type::Numeric(typ) => Value::Numeric(NumericValue::zero(*typ)),
            Type::Reference(element_type) => Self::reference(id, element_type.clone()),
            Type::Array(element_types, length) => {
                let first_elements =
                    vecmap(element_types.iter(), |typ| Self::uninitialized(typ, id));
                let elements = std::iter::repeat_n(first_elements, *length as usize);
                let elements = elements.flatten().collect();
                Self::array(elements, element_types.to_vec())
            }
            Type::Slice(element_types) => Self::slice(Vec::new(), element_types.clone()),
            Type::Function => Value::ForeignFunction("uninitialized!".to_string()),
        }
    }

    pub(crate) fn as_string(&self) -> Option<String> {
        let array = self.as_array_or_slice()?;
        let elements = array.elements.borrow();
        let bytes = elements.iter().map(|element| element.as_u8()).collect::<Option<Vec<_>>>()?;
        Some(String::from_utf8_lossy(&bytes).into_owned())
    }

    pub(crate) fn as_field(&self) -> Option<FieldElement> {
        self.as_numeric()?.as_field()
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

    pub(crate) fn zero(typ: NumericType) -> Self {
        Self::from_constant(FieldElement::zero(), typ)
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

    pub(crate) fn as_u8(&self) -> Option<u8> {
        match self {
            NumericValue::U8(value) => Some(*value),
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
            NumericValue::U8(value) => FieldElement::from(*value as u32),
            NumericValue::U16(value) => FieldElement::from(*value as u32),
            NumericValue::U32(value) => FieldElement::from(*value),
            NumericValue::U64(value) => FieldElement::from(*value),
            NumericValue::U128(value) => FieldElement::from(*value),
            NumericValue::I8(value) => FieldElement::from(*value as i128),
            NumericValue::I16(value) => FieldElement::from(*value as i128),
            NumericValue::I32(value) => FieldElement::from(*value as i128),
            NumericValue::I64(value) => FieldElement::from(*value as i128),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Numeric(numeric_value) => write!(f, "{numeric_value}"),
            Value::Reference(reference_value) => write!(f, "{reference_value}"),
            Value::ArrayOrSlice(array_value) => write!(f, "{array_value}"),
            Value::Function(id) => write!(f, "{id}"),
            Value::Intrinsic(intrinsic) => write!(f, "{intrinsic}"),
            Value::ForeignFunction(name) => write!(f, "ForeignFunction(\"{name}\")"),
        }
    }
}

impl std::fmt::Display for NumericValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericValue::Field(value) => write!(f, "Field {value}"),
            NumericValue::U1(value) => write!(f, "u1 {value}"),
            NumericValue::U8(value) => write!(f, "u8 {value}"),
            NumericValue::U16(value) => write!(f, "u16 {value}"),
            NumericValue::U32(value) => write!(f, "u32 {value}"),
            NumericValue::U64(value) => write!(f, "u64 {value}"),
            NumericValue::U128(value) => write!(f, "u128 {value}"),
            NumericValue::I8(value) => write!(f, "i8 {value}"),
            NumericValue::I16(value) => write!(f, "i16 {value}"),
            NumericValue::I32(value) => write!(f, "i32 {value}"),
            NumericValue::I64(value) => write!(f, "i64 {value}"),
        }
    }
}

impl std::fmt::Display for ReferenceValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let element = self.element.borrow();
        match &*element {
            Some(element) => write!(f, "*{} = {}", self.original_id, element),
            None => write!(f, "*{} = None", self.original_id),
        }
    }
}

impl std::fmt::Display for ArrayValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let elements = self.elements.borrow();
        let elements = vecmap(elements.iter(), ToString::to_string).join(", ");

        let is_slice = if self.is_slice { "&" } else { "" };
        write!(f, "rc{} {is_slice}[{elements}]", self.rc.borrow())
    }
}
