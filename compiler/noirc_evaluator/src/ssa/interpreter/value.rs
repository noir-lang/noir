use std::sync::Arc;

use acvm::{AcirField, FieldElement};
use iter_extended::{try_vecmap, vecmap};
use noirc_frontend::Shared;

use crate::ssa::ir::{
    function::FunctionId,
    instruction::Intrinsic,
    integer::IntegerConstant,
    types::{CompositeType, NumericType, Type},
    value::ValueId,
};

use super::IResult;

/// Be careful when using `Clone`: `ArrayValue` and `ReferenceValue`
/// are backed by a `Shared` data structure, and for example modifying
/// an array element would be reflected in the original and the clone
/// as well. Use `Value::snapshot` to make independent clones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Numeric(NumericValue),
    Reference(ReferenceValue),
    ArrayOrSlice(ArrayValue),
    Function(FunctionId),
    Intrinsic(Intrinsic),
    ForeignFunction(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceValue {
    /// This is included mostly for debugging to distinguish different
    /// ReferenceValues which store the same element.
    pub original_id: ValueId,

    /// A value of `None` here means this allocation is currently uninitialized
    pub element: Shared<Option<Value>>,

    pub element_type: Arc<Type>,
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
    #[allow(unused)]
    pub(crate) fn get_type(&self) -> Type {
        match self {
            Value::Numeric(numeric_value) => Type::Numeric(numeric_value.get_type()),
            Value::Reference(reference) => Type::Reference(reference.element_type.clone()),
            Value::ArrayOrSlice(array) if array.is_slice => {
                Type::Slice(array.element_types.clone())
            }
            Value::ArrayOrSlice(array) => {
                let len = array.elements.borrow().len().checked_div(array.element_types.len());
                let len = len.unwrap_or(0) as u32;
                Type::Array(array.element_types.clone(), len)
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

    pub(crate) fn from_constant(constant: FieldElement, typ: NumericType) -> IResult<Self> {
        NumericValue::from_constant(constant, typ).map(Self::Numeric)
    }

    pub(crate) fn array_from_iter(
        iter: impl IntoIterator<Item = FieldElement>,
        typ: NumericType,
    ) -> IResult<Self> {
        let values = try_vecmap(iter, |v| Value::from_constant(v, typ))?;
        Ok(Self::array(values, vec![Type::Numeric(typ)]))
    }

    // This is used in tests but shouldn't be cfg(test) only
    #[allow(unused)]
    pub(crate) fn bool(value: bool) -> Self {
        Self::Numeric(NumericValue::U1(value))
    }

    pub fn array(elements: Vec<Value>, element_types: Vec<Type>) -> Self {
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

    /// Clone the value in a way that modifications to it won't affect the original.
    pub fn snapshot(&self) -> Self {
        match self {
            Value::Numeric(n) => Value::Numeric(*n),
            Value::Reference(r) => Value::Reference(ReferenceValue {
                original_id: r.original_id,
                element: Shared::new(r.element.borrow().clone()),
                element_type: r.element_type.clone(),
            }),
            Value::ArrayOrSlice(a) => Value::ArrayOrSlice(ArrayValue {
                elements: Shared::new(a.elements.borrow().clone()),
                rc: Shared::new(*a.rc.borrow()),
                element_types: a.element_types.clone(),
                is_slice: a.is_slice,
            }),
            Value::Function(id) => Value::Function(*id),
            Value::Intrinsic(i) => Value::Intrinsic(*i),
            Value::ForeignFunction(s) => Value::ForeignFunction(s.clone()),
        }
    }

    /// Take a snapshot of interpreter arguments.
    ///
    /// Useful when we want to use the same input to run the interpreter
    /// after different SSA passes, without them affecting each other.
    pub fn snapshot_args(args: &[Value]) -> Vec<Value> {
        args.iter().map(|arg| arg.snapshot()).collect()
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
        Self::from_constant(FieldElement::zero(), typ).expect("zero should fit in every type")
    }

    pub(crate) fn neg_one(typ: NumericType) -> Self {
        let neg_one = IntegerConstant::Signed { value: -1, bit_size: typ.bit_size() };
        let (neg_one_constant, typ) = neg_one.into_numeric_constant();
        Self::from_constant(neg_one_constant, typ)
            .unwrap_or_else(|_| panic!("Negative one cannot fit in {typ}"))
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

    pub fn from_constant(constant: FieldElement, typ: NumericType) -> IResult<NumericValue> {
        use super::InternalError::{ConstantDoesNotFitInType, UnsupportedNumericType};
        use super::InterpreterError::Internal;

        let does_not_fit = Internal(ConstantDoesNotFitInType { constant, typ });

        match typ {
            NumericType::NativeField => Ok(Self::Field(constant)),
            NumericType::Unsigned { bit_size: 1 } => {
                if constant.is_zero() || constant.is_one() {
                    Ok(Self::U1(constant.is_one()))
                } else {
                    Err(does_not_fit)
                }
            }
            NumericType::Unsigned { bit_size: 8 } => constant
                .try_into_u128()
                .and_then(|x| x.try_into().ok())
                .map(Self::U8)
                .ok_or(does_not_fit),
            NumericType::Unsigned { bit_size: 16 } => constant
                .try_into_u128()
                .and_then(|x| x.try_into().ok())
                .map(Self::U16)
                .ok_or(does_not_fit),
            NumericType::Unsigned { bit_size: 32 } => constant
                .try_into_u128()
                .and_then(|x| x.try_into().ok())
                .map(Self::U32)
                .ok_or(does_not_fit),
            NumericType::Unsigned { bit_size: 64 } => constant
                .try_into_u128()
                .and_then(|x| x.try_into().ok())
                .map(Self::U64)
                .ok_or(does_not_fit),
            NumericType::Unsigned { bit_size: 128 } => {
                constant.try_into_u128().map(Self::U128).ok_or(does_not_fit)
            }
            // Signed cases are a bit weird. We want to allow all values in the corresponding
            // unsigned range so we have to cast to the unsigned type first to see if it fits.
            // If it does, any values `>= 2^N / 2` for `iN` are interpreted as negative.
            NumericType::Signed { bit_size: 8 } => constant
                .try_into_u128()
                .and_then(|x| u8::try_from(x).ok())
                .map(|x| Self::I8(x as i8))
                .ok_or(does_not_fit),
            NumericType::Signed { bit_size: 16 } => constant
                .try_into_u128()
                .and_then(|x| u16::try_from(x).ok())
                .map(|x| Self::I16(x as i16))
                .ok_or(does_not_fit),
            NumericType::Signed { bit_size: 32 } => constant
                .try_into_u128()
                .and_then(|x| u32::try_from(x).ok())
                .map(|x| Self::I32(x as i32))
                .ok_or(does_not_fit),
            NumericType::Signed { bit_size: 64 } => constant
                .try_into_u128()
                .and_then(|x| u64::try_from(x).ok())
                .map(|x| Self::I64(x as i64))
                .ok_or(does_not_fit),
            typ => Err(Internal(UnsupportedNumericType { typ })),
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
            // Need to cast possibly negative values to the unsigned variants
            // first to ensure they are zero-extended rather than sign-extended
            NumericValue::I8(value) => FieldElement::from(*value as u8 as i128),
            NumericValue::I16(value) => FieldElement::from(*value as u16 as i128),
            NumericValue::I32(value) => FieldElement::from(*value as u32 as i128),
            NumericValue::I64(value) => FieldElement::from(*value as u64 as i128),
        }
    }

    pub fn is_negative(&self) -> bool {
        match self {
            NumericValue::I8(v) => *v < 0,
            NumericValue::I16(v) => *v < 0,
            NumericValue::I32(v) => *v < 0,
            NumericValue::I64(v) => *v < 0,
            _ => false,
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

impl PartialEq for ArrayValue {
    fn eq(&self, other: &Self) -> bool {
        // Don't compare RC
        self.elements == other.elements
            && self.element_types == other.element_types
            && self.is_slice == other.is_slice
    }
}

impl Eq for ArrayValue {}
