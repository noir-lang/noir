use std::sync::Arc;

use acvm::{AcirField, FieldElement};
use iter_extended::{try_vecmap, vecmap};
use noirc_frontend::Shared;

use crate::ssa::ir::{
    function::FunctionId,
    instruction::Intrinsic,
    is_printable_byte,
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
    ArrayOrVector(ArrayValue),
    Function(FunctionId),
    Intrinsic(Intrinsic),
    ForeignFunction(String),
}

/// Represents a numeric type that either fits in the expected bit size,
/// or would have to be represented as a `Field` to match the semantics of ACIR.
///
/// The reason this exists is the difference in behavior of unchecked operations in Brillig and ACIR:
/// * In Brillig unchecked operations wrap around, but we have other opcodes surrounding it that
///   either prevent such operations from being carried out, or check for any overflows later.
/// * In ACIR, everything is represented as a `Field`, and overflows are not checked, so e.g. an unchecked
///   multiplication of `u32` values can result in something that only fits in a `u64`.
///
/// When we interpret an operation that would wrap around, if we are in an ACIR context we can use
/// the `Unfit` variant to indicate that the value went beyond what fits into the base type.
///
/// Since we normally require that ACIR and Brillig return the same result, once an operation
/// overflows its type, we have reason to believe that ACIR and Brillig would not return the same
/// value, since Brillig wraps, and ACIR does not.
///
/// However, some operations that we ported back from ACIR to SSA are implemented in such a
/// way that transient values "escape" the boundaries of their type, only to be restored later,
/// so keeping the `Field` serves more than informational purposes. We expect that under normal
/// circumstances this effect is temporary, and by the time we would have to apply operations
/// on the values that aren't implemented for `Field` (e.g. `lt` and bitwise ops), the values will
/// be back on track.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Fitted<T> {
    Fit(T),
    Unfit(FieldElement),
}

impl<A> Fitted<A> {
    pub fn map<B>(
        self,
        f: impl FnOnce(A) -> B,
        g: impl FnOnce(FieldElement) -> FieldElement,
    ) -> Fitted<B> {
        match self {
            Self::Fit(value) => Fitted::Fit(f(value)),
            Self::Unfit(value) => Fitted::Unfit(g(value)),
        }
    }

    pub fn apply<B>(self, f: impl FnOnce(A) -> B, g: impl FnOnce(FieldElement) -> B) -> B {
        match self {
            Self::Fit(value) => f(value),
            Self::Unfit(value) => g(value),
        }
    }
}

macro_rules! impl_fitted {
    ($($t:ty),*) => {
        $(
        impl From<$t> for Fitted<$t> {
            fn from(value: $t) -> Self {
                Self::Fit(value)
            }
        }

        impl From<FieldElement> for Fitted<$t> {
            fn from(value: FieldElement) -> Self {
                Self::Unfit(value)
            }
        }
        )*
    };
}

impl_fitted! { u8, u16, u32, u64, u128, i8, i16, i32, i64 }

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumericValue {
    Field(FieldElement),

    U1(bool),
    U8(Fitted<u8>),
    U16(Fitted<u16>),
    U32(Fitted<u32>),
    U64(Fitted<u64>),
    U128(Fitted<u128>),

    I8(Fitted<i8>),
    I16(Fitted<i16>),
    I32(Fitted<i32>),
    I64(Fitted<i64>),
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
    pub is_vector: bool,
}

impl Value {
    #[allow(unused)]
    pub(crate) fn get_type(&self) -> Type {
        match self {
            Value::Numeric(numeric_value) => Type::Numeric(numeric_value.get_type()),
            Value::Reference(reference) => Type::Reference(reference.element_type.clone()),
            Value::ArrayOrVector(array) if array.is_vector => {
                Type::Vector(array.element_types.clone())
            }
            Value::ArrayOrVector(array) => {
                let len = array.elements.borrow().len().checked_div(array.element_types.len());
                let len = len.unwrap_or(0) as u32;
                Type::Array(array.element_types.clone(), len)
            }
            Value::Function(_) | Value::Intrinsic(_) | Value::ForeignFunction(_) => Type::Function,
        }
    }

    /// Create an empty reference value.
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
            Value::Numeric(NumericValue::U8(Fitted::Fit(value))) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_u32(&self) -> Option<u32> {
        match self {
            Value::Numeric(NumericValue::U32(Fitted::Fit(value))) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Numeric(NumericValue::U64(Fitted::Fit(value))) => Some(*value),
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

    pub(crate) fn as_array_or_vector(&self) -> Option<ArrayValue> {
        match self {
            Value::ArrayOrVector(value) => Some(value.clone()),
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

    pub fn bool(value: bool) -> Self {
        Self::Numeric(NumericValue::U1(value))
    }

    pub fn field(value: FieldElement) -> Self {
        Self::Numeric(NumericValue::Field(value))
    }

    pub fn u8(value: u8) -> Self {
        Self::Numeric(NumericValue::U8(value.into()))
    }

    pub fn u16(value: u16) -> Self {
        Self::Numeric(NumericValue::U16(value.into()))
    }

    pub fn u32(value: u32) -> Self {
        Self::Numeric(NumericValue::U32(value.into()))
    }

    pub fn u128(value: u128) -> Self {
        Self::Numeric(NumericValue::U128(value.into()))
    }

    pub fn u64(value: u64) -> Self {
        Self::Numeric(NumericValue::U64(value.into()))
    }

    pub fn i8(value: i8) -> Self {
        Self::Numeric(NumericValue::I8(value.into()))
    }

    pub fn i16(value: i16) -> Self {
        Self::Numeric(NumericValue::I16(value.into()))
    }

    pub fn i32(value: i32) -> Self {
        Self::Numeric(NumericValue::I32(value.into()))
    }

    pub fn i64(value: i64) -> Self {
        Self::Numeric(NumericValue::I64(value.into()))
    }

    pub fn array(elements: Vec<Value>, element_types: Vec<Type>) -> Self {
        Self::ArrayOrVector(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types: Arc::new(element_types),
            is_vector: false,
        })
    }

    pub(crate) fn vector(elements: Vec<Value>, element_types: Arc<Vec<Type>>) -> Self {
        Self::ArrayOrVector(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types,
            is_vector: true,
        })
    }

    /// Return an uninitialized value of the given type. This is usually a zeroed
    /// value but we make no guarantee that it is. This is often used as the default
    /// value to return for side-effectful functions like `call` or `array_get` when
    /// side-effects are disabled.
    pub(crate) fn uninitialized(typ: &Type, id: ValueId) -> Value {
        match typ {
            Type::Numeric(typ) => Value::Numeric(NumericValue::zero(*typ)),
            Type::Reference(element_type) => {
                // Initialize the reference to a default value, so that if we execute a
                // Load instruction when side effects are disabled, we don't get an error.
                let value = Self::uninitialized(element_type, id);
                Self::Reference(ReferenceValue {
                    original_id: id,
                    element_type: element_type.clone(),
                    element: Shared::new(Some(value)),
                })
            }
            Type::Array(element_types, length) => {
                let first_elements =
                    vecmap(element_types.iter(), |typ| Self::uninitialized(typ, id));
                let elements = std::iter::repeat_n(first_elements, *length as usize);
                let elements = elements.flatten().collect();
                Self::array(elements, element_types.to_vec())
            }
            Type::Vector(element_types) => Self::vector(Vec::new(), element_types.clone()),
            Type::Function => Value::ForeignFunction("uninitialized!".to_string()),
        }
    }

    pub(crate) fn as_string(&self) -> Option<String> {
        let array = self.as_array_or_vector()?;
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
            Value::Reference(r) => {
                let element = r.element.borrow().as_ref().map(|v| v.snapshot());
                Value::Reference(ReferenceValue {
                    original_id: r.original_id,
                    element: Shared::new(element),
                    element_type: r.element_type.clone(),
                })
            }
            Value::ArrayOrVector(a) => {
                let elements = a.elements.borrow().iter().map(|v| v.snapshot()).collect();
                Value::ArrayOrVector(ArrayValue {
                    elements: Shared::new(elements),
                    rc: Shared::new(*a.rc.borrow()),
                    element_types: a.element_types.clone(),
                    is_vector: a.is_vector,
                })
            }
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

    /// Wrap a `Field` into an `Unfit`, with a type that we were _supposed_ to get,
    /// had some operation not overflown.
    ///
    /// This is used only in tests to construct expected values.
    #[cfg(test)]
    pub(crate) fn unfit(field: FieldElement, typ: NumericType) -> IResult<Self> {
        NumericValue::unfit(field, typ).map(Self::Numeric)
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

    /// Create a `NumericValue` from a `Field` constant.
    ///
    /// Returns an error if the value does not fit into the number of bits indicated by the `NumericType`.
    ///
    /// Never creates `Fitted::Unfit` values.
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
                .map(Fitted::Fit)
                .map(Self::U8)
                .ok_or(does_not_fit),
            NumericType::Unsigned { bit_size: 16 } => constant
                .try_into_u128()
                .and_then(|x| x.try_into().ok())
                .map(Fitted::Fit)
                .map(Self::U16)
                .ok_or(does_not_fit),
            NumericType::Unsigned { bit_size: 32 } => constant
                .try_into_u128()
                .and_then(|x| x.try_into().ok())
                .map(Fitted::Fit)
                .map(Self::U32)
                .ok_or(does_not_fit),
            NumericType::Unsigned { bit_size: 64 } => constant
                .try_into_u128()
                .and_then(|x| x.try_into().ok())
                .map(Fitted::Fit)
                .map(Self::U64)
                .ok_or(does_not_fit),
            NumericType::Unsigned { bit_size: 128 } => {
                constant.try_into_u128().map(Fitted::Fit).map(Self::U128).ok_or(does_not_fit)
            }
            // Signed cases are a bit weird. We want to allow all values in the corresponding
            // unsigned range so we have to cast to the unsigned type first to see if it fits.
            // If it does, any values `>= 2^N / 2` for `iN` are interpreted as negative.
            NumericType::Signed { bit_size: 8 } => constant
                .try_into_u128()
                .and_then(|x| u8::try_from(x).ok())
                .map(|x| Fitted::Fit(x as i8))
                .map(Self::I8)
                .ok_or(does_not_fit),
            NumericType::Signed { bit_size: 16 } => constant
                .try_into_u128()
                .and_then(|x| u16::try_from(x).ok())
                .map(|x| Fitted::Fit(x as i16))
                .map(Self::I16)
                .ok_or(does_not_fit),
            NumericType::Signed { bit_size: 32 } => constant
                .try_into_u128()
                .and_then(|x| u32::try_from(x).ok())
                .map(|x| Fitted::Fit(x as i32))
                .map(Self::I32)
                .ok_or(does_not_fit),
            NumericType::Signed { bit_size: 64 } => constant
                .try_into_u128()
                .and_then(|x| u64::try_from(x).ok())
                .map(|x| Fitted::Fit(x as i64))
                .map(Self::I64)
                .ok_or(does_not_fit),
            typ => Err(Internal(UnsupportedNumericType { typ })),
        }
    }

    pub fn convert_to_field(&self) -> FieldElement {
        match self {
            NumericValue::Field(field) => *field,
            NumericValue::U1(boolean) if *boolean => FieldElement::one(),
            NumericValue::U1(_) => FieldElement::zero(),
            NumericValue::U8(Fitted::Fit(value)) => FieldElement::from(u32::from(*value)),
            NumericValue::U16(Fitted::Fit(value)) => FieldElement::from(u32::from(*value)),
            NumericValue::U32(Fitted::Fit(value)) => FieldElement::from(*value),
            NumericValue::U64(Fitted::Fit(value)) => FieldElement::from(*value),
            NumericValue::U128(Fitted::Fit(value)) => FieldElement::from(*value),
            // Need to cast possibly negative values to the unsigned variants
            // first to ensure they are zero-extended rather than sign-extended
            NumericValue::I8(Fitted::Fit(value)) => FieldElement::from(i128::from(*value as u8)),
            NumericValue::I16(Fitted::Fit(value)) => FieldElement::from(i128::from(*value as u16)),
            NumericValue::I32(Fitted::Fit(value)) => FieldElement::from(i128::from(*value as u32)),
            NumericValue::I64(Fitted::Fit(value)) => FieldElement::from(i128::from(*value as u64)),

            NumericValue::U8(Fitted::Unfit(value))
            | NumericValue::U16(Fitted::Unfit(value))
            | NumericValue::U32(Fitted::Unfit(value))
            | NumericValue::U64(Fitted::Unfit(value))
            | NumericValue::U128(Fitted::Unfit(value))
            | NumericValue::I8(Fitted::Unfit(value))
            | NumericValue::I16(Fitted::Unfit(value))
            | NumericValue::I32(Fitted::Unfit(value))
            | NumericValue::I64(Fitted::Unfit(value)) => *value,
        }
    }

    /// Creates a `NumericValue` of a specific bit size with a `Fitted::Unfit` value.
    #[cfg(test)]
    pub fn unfit(field: FieldElement, typ: NumericType) -> IResult<NumericValue> {
        use super::InternalError::UnsupportedNumericType;
        use super::InterpreterError::Internal;

        match typ {
            NumericType::NativeField | NumericType::Unsigned { bit_size: 1 } => {
                unreachable!("{typ} cannot be unfit")
            }
            NumericType::Unsigned { bit_size: 8 } => Ok(Self::U8(Fitted::Unfit(field))),
            NumericType::Unsigned { bit_size: 16 } => Ok(Self::U16(Fitted::Unfit(field))),
            NumericType::Unsigned { bit_size: 32 } => Ok(Self::U32(Fitted::Unfit(field))),
            NumericType::Unsigned { bit_size: 64 } => Ok(Self::U64(Fitted::Unfit(field))),
            NumericType::Unsigned { bit_size: 128 } => Ok(Self::U128(Fitted::Unfit(field))),
            NumericType::Signed { bit_size: 8 } => Ok(Self::I8(Fitted::Unfit(field))),
            NumericType::Signed { bit_size: 16 } => Ok(Self::I16(Fitted::Unfit(field))),
            NumericType::Signed { bit_size: 32 } => Ok(Self::I32(Fitted::Unfit(field))),
            NumericType::Signed { bit_size: 64 } => Ok(Self::I64(Fitted::Unfit(field))),
            typ => Err(Internal(UnsupportedNumericType { typ })),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Numeric(numeric_value) => write!(f, "{numeric_value}"),
            Value::Reference(reference_value) => write!(f, "{reference_value}"),
            Value::ArrayOrVector(array_value) => write!(f, "{array_value}"),
            Value::Function(id) => write!(f, "{id}"),
            Value::Intrinsic(intrinsic) => write!(f, "{intrinsic}"),
            Value::ForeignFunction(name) => write!(f, "ForeignFunction(\"{name}\")"),
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Fitted<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fitted::Fit(v) => v.fmt(f),
            // Distinguish an overflowed value from the type it's supposed to be.
            Fitted::Unfit(v) => write!(f, "({v})"),
        }
    }
}

impl std::fmt::Display for NumericValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericValue::Field(value) => write!(f, "Field {value}"),
            NumericValue::U1(value) => write!(f, "u1 {}", if *value { "1" } else { "0" }),
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
        let rc = self.rc.borrow();

        let is_vector = if self.is_vector { "&" } else { "" };
        write!(f, "rc{rc} {is_vector}")?;

        // Check if the array could be shown as a string literal
        if self.element_types.len() == 1
            && matches!(self.element_types[0], Type::Numeric(NumericType::Unsigned { bit_size: 8 }))
        {
            let printable = self
                .elements
                .borrow()
                .iter()
                .all(|value| value.as_u8().is_some_and(is_printable_byte));

            if printable {
                let bytes = self
                    .elements
                    .borrow()
                    .iter()
                    .map(|value| {
                        let Some(byte) = value.as_u8() else {
                            panic!("Expected U8 value in array, found {value}");
                        };
                        byte
                    })
                    .collect::<Vec<_>>();
                let string = String::from_utf8(bytes).unwrap();
                write!(f, "b{string:?}")?;
                return Ok(());
            }
        }

        write!(f, "[")?;

        let length = self.elements.borrow().len() / self.element_types.len();
        if length == 0 {
            // We show an array length zero like `[T; 0]` or `[(T1, T2, ...); 0]`
            let element_types = if self.element_types.len() == 1 {
                self.element_types[0].to_string()
            } else {
                let element_types =
                    vecmap(self.element_types.iter(), ToString::to_string).join(", ");
                format!("({element_types})")
            };
            write!(f, "{element_types}; {length}")?;
        } else {
            // Otherwise we show the elements, but try to group them if the element type is a composite type
            // (that way the element types can be inferred from the elements)
            let element_types_len = self.element_types.len();
            for (index, element) in self.elements.borrow().iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                if element_types_len > 1 && index % element_types_len == 0 {
                    write!(f, "(")?;
                }
                write!(f, "{element}")?;
                if element_types_len > 1 && index % element_types_len == element_types_len - 1 {
                    write!(f, ")")?;
                }
            }
        }

        write!(f, "]")
    }
}

impl PartialEq for ArrayValue {
    fn eq(&self, other: &Self) -> bool {
        // Don't compare RC
        self.elements == other.elements
            && self.element_types == other.element_types
            && self.is_vector == other.is_vector
    }
}

impl Eq for ArrayValue {}
