use std::sync::Arc;

use acvm::{AcirField, FieldElement, acir::brillig::lengths::SemanticLength};
use iter_extended::{try_vecmap, vecmap};
use noirc_frontend::Shared;

use crate::{
    brillig::{assert_u32, assert_usize},
    ssa::ir::{
        function::FunctionId,
        instruction::Intrinsic,
        is_printable_byte,
        types::{CompositeType, NumericType, Type},
        value::ValueId,
    },
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

/// A numeric runtime value in the SSA interpreter.
///
/// Each integer variant stores a [`FieldElement`] holding the value's two's-complement *bit
/// pattern*; the variant itself is the type tag (bit width + signedness). This mirrors how the
/// backends actually represent integers:
///
/// * ACIR represents every integer as a `FieldElement`. Arithmetic is field arithmetic, so a value
///   may temporarily exceed its type's range (e.g. an unchecked `u8` multiplication can produce a
///   field that only fits in a `u16`) until a range-check or truncation brings it back. Signed
///   values are stored as their two's-complement bit pattern (`i32 -2` is the field `4294967294`);
///   signedness lives in the operations, not the value.
/// * Brillig stores native fixed-width unsigned registers (`u8..u128`), always in range, wrapping
///   on overflow. Signed integers are the unsigned bit pattern plus signed operations.
///
/// "In range" is therefore a computed predicate: the stored field is
/// less than `2^bit_size`. An in-range field is the ordinary value; an out-of-range field only
/// arises in ACIR mode, where unchecked arithmetic does not reduce.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NumericValue {
    /// The value's two's-complement bit pattern (for in-range values), stored as a field. It may be
    /// an out-of-range field in ACIR mode, where unchecked arithmetic does not reduce. For a `u1`
    /// this is `0` or `1`; for `NativeField` it is the field itself.
    value: FieldElement,
    /// The value's numeric type (width + signedness, `u1`, or `NativeField`). Signedness lives here
    /// and in the operations, not in a separate representation.
    typ: NumericType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceValue {
    /// This is included mostly for debugging to distinguish different
    /// `ReferenceValues` which store the same element.
    pub original_id: ValueId,

    /// A value of `None` here means this allocation is currently uninitialized
    pub element: Shared<Option<Value>>,

    pub element_type: Arc<Type>,

    /// Whether this reference is mutable (`&mut T`) or immutable (`&T`).
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct ArrayValue {
    pub elements: Shared<Vec<Value>>,

    /// The `Shared` type contains its own reference count but we need to track
    /// the reference count separate to ensure it is only changed by `IncrementRc` and
    /// `DecrementRc` instructions.
    pub rc: Shared<u32>,

    pub element_types: Arc<CompositeType>,
    /// Some length, if this is an array, otherwise None.
    pub length: Option<SemanticLength>,
}

impl ArrayValue {
    pub(crate) fn is_vector(&self) -> bool {
        self.length.is_none()
    }

    pub(crate) fn get_type(&self) -> Type {
        match self.length {
            Some(length) => Type::Array(self.element_types.clone(), length),
            None => Type::Vector(self.element_types.clone()),
        }
    }
}

impl Value {
    #[allow(unused)]
    pub(crate) fn get_type(&self) -> Type {
        match self {
            Value::Numeric(numeric_value) => Type::Numeric(numeric_value.get_type()),
            Value::Reference(reference) => {
                Type::Reference(reference.element_type.clone(), reference.mutable)
            }
            Value::ArrayOrVector(array) => array.get_type(),
            Value::Function(_) | Value::Intrinsic(_) | Value::ForeignFunction(_) => Type::Function,
        }
    }

    /// Create an empty reference value.
    pub(crate) fn reference(original_id: ValueId, element_type: Arc<Type>, mutable: bool) -> Self {
        Value::Reference(ReferenceValue {
            original_id,
            element_type,
            element: Shared::new(None),
            mutable,
        })
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        self.as_numeric()?.as_bool()
    }

    pub(crate) fn as_u8(&self) -> Option<u8> {
        let x = self.as_unsigned(8)?;
        u8::try_from(x).ok()
    }

    pub(crate) fn as_u32(&self) -> Option<u32> {
        let x = self.as_unsigned(32)?;
        u32::try_from(x).ok()
    }

    pub(crate) fn as_u64(&self) -> Option<u64> {
        let x = self.as_unsigned(64)?;
        u64::try_from(x).ok()
    }

    /// If this is an in-range unsigned value of the given bit size, returns its `u128` value.
    fn as_unsigned(&self, bit_size: u32) -> Option<u128> {
        let value = self.as_numeric()?;
        (value.get_type() == NumericType::Unsigned { bit_size } && value.is_in_range())
            .then(|| value.to_field().try_into_u128())
            .flatten()
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
        Self::Numeric(NumericValue::bool(value))
    }

    pub fn field(value: FieldElement) -> Self {
        Self::Numeric(NumericValue::field(value))
    }

    pub fn u8(value: u8) -> Self {
        Self::Numeric(NumericValue::u8(value))
    }

    pub fn u16(value: u16) -> Self {
        Self::Numeric(NumericValue::u16(value))
    }

    pub fn u32(value: u32) -> Self {
        Self::Numeric(NumericValue::u32(value))
    }

    pub fn u128(value: u128) -> Self {
        Self::Numeric(NumericValue::u128(value))
    }

    pub fn u64(value: u64) -> Self {
        Self::Numeric(NumericValue::u64(value))
    }

    pub fn i8(value: i8) -> Self {
        Self::Numeric(NumericValue::i8(value))
    }

    pub fn i16(value: i16) -> Self {
        Self::Numeric(NumericValue::i16(value))
    }

    pub fn i32(value: i32) -> Self {
        Self::Numeric(NumericValue::i32(value))
    }

    pub fn i64(value: i64) -> Self {
        Self::Numeric(NumericValue::i64(value))
    }

    pub fn array(elements: Vec<Value>, element_types: Vec<Type>) -> Self {
        assert!(!element_types.is_empty());

        let length = assert_u32(elements.len() / element_types.len());
        Self::array_with_length(elements, element_types, SemanticLength(length))
    }

    /// Build an array value with an explicit semantic length.
    ///
    /// Unlike [`Value::array`], this supports zero-sized element types (empty `element_types`),
    /// where the length cannot be recovered by dividing the flattened element count by the number
    /// of element types.
    pub(crate) fn array_with_length(
        elements: Vec<Value>,
        element_types: Vec<Type>,
        length: SemanticLength,
    ) -> Self {
        assert_eq!(length.0 as usize * element_types.len(), elements.len());
        Self::ArrayOrVector(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types: Arc::new(element_types),
            length: Some(length),
        })
    }

    pub(crate) fn vector(elements: Vec<Value>, element_types: Arc<Vec<Type>>) -> Self {
        Self::ArrayOrVector(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types,
            length: None,
        })
    }

    /// Return an uninitialized value of the given type. This is usually a zeroed
    /// value but we make no guarantee that it is. This is often used as the default
    /// value to return for side-effectful functions like `call` or `array_get` when
    /// side-effects are disabled.
    pub(crate) fn uninitialized(typ: &Type, id: ValueId) -> Value {
        match typ {
            Type::Numeric(typ) => Value::Numeric(NumericValue::zero(*typ)),
            Type::Reference(element_type, mutable) => {
                // Initialize the reference to a default value, so that if we execute a
                // Load instruction when side effects are disabled, we don't get an error.
                let value = Self::uninitialized(element_type, id);
                Self::Reference(ReferenceValue {
                    original_id: id,
                    element_type: element_type.clone(),
                    element: Shared::new(Some(value)),
                    mutable: *mutable,
                })
            }
            Type::Array(element_types, length) => {
                let first_elements =
                    vecmap(element_types.iter(), |typ| Self::uninitialized(typ, id));
                let elements = std::iter::repeat_n(first_elements, assert_usize(length.0));
                let elements = elements.flatten().collect();
                Self::array_with_length(elements, element_types.to_vec(), *length)
            }
            Type::Vector(element_types) => Self::uninitialized_vector(element_types, 0, id),
            Type::Function => Value::ForeignFunction("uninitialized!".to_string()),
        }
    }

    /// Create an uninitialized (zeroed) vector of the given size.
    /// Each element slot is filled with `Value::uninitialized` of the appropriate element type.
    pub(crate) fn uninitialized_vector(element_types: &[Type], size: usize, id: ValueId) -> Value {
        let element_count = element_types.len();
        let elements = (0..size)
            .map(|i| {
                let element_type = &element_types[i % element_count.max(1)];
                Self::uninitialized(element_type, id)
            })
            .collect();
        Self::vector(elements, Arc::new(element_types.to_vec()))
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
                    mutable: r.mutable,
                })
            }
            Value::ArrayOrVector(a) => {
                let elements = a.elements.borrow().iter().map(|v| v.snapshot()).collect();
                Value::ArrayOrVector(ArrayValue {
                    elements: Shared::new(elements),
                    rc: Shared::new(*a.rc.borrow()),
                    element_types: a.element_types.clone(),
                    length: a.length,
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

    /// Build an integer value directly from a raw bit-pattern field and a numeric type, without
    /// checking that the field is in range. Used to relabel a value's type (as `cast` does) and in
    /// tests to construct expected values that may be out of range (as ACIR's unchecked field
    /// arithmetic can produce).
    pub(crate) fn int_from_field(field: FieldElement, typ: NumericType) -> IResult<Self> {
        NumericValue::int_from_field(field, typ).map(Self::Numeric)
    }
}

impl NumericValue {
    /// Build a value from a raw bit-pattern field and a numeric type, without checking the field is
    /// in range. The field is the value's two's-complement bit pattern for in-range values and may
    /// be an extended/out-of-range field in ACIR mode. Errors only for numeric types the interpreter
    /// does not model. Use [`NumericValue::from_constant`] when the value must be in range.
    pub(crate) fn int_from_field(field: FieldElement, typ: NumericType) -> IResult<NumericValue> {
        use super::InternalError::UnsupportedNumericType;
        use super::InterpreterError::Internal;

        match typ {
            NumericType::NativeField
            | NumericType::Unsigned { bit_size: 1 | 8 | 16 | 32 | 64 | 128 }
            | NumericType::Signed { bit_size: 8 | 16 | 32 | 64 } => Ok(Self { value: field, typ }),
            typ => Err(Internal(UnsupportedNumericType { typ })),
        }
    }

    /// Create a `NumericValue` from a `Field` constant, erroring if the value does not fit the type.
    pub fn from_constant(constant: FieldElement, typ: NumericType) -> IResult<NumericValue> {
        use super::InternalError::ConstantDoesNotFitInType;
        use super::InterpreterError::Internal;

        let value = Self::int_from_field(constant, typ)?;
        let representable = match typ {
            // A `Field` is always representable; a `u1` must be exactly 0 or 1.
            NumericType::NativeField => true,
            NumericType::Unsigned { bit_size: 1 } => constant.is_zero() || constant.is_one(),
            _ => value.is_in_range(),
        };
        if representable {
            Ok(value)
        } else {
            Err(Internal(ConstantDoesNotFitInType { constant, typ }))
        }
    }

    pub(crate) fn get_type(&self) -> NumericType {
        self.typ
    }

    pub(crate) fn zero(typ: NumericType) -> Self {
        Self::from_constant(FieldElement::zero(), typ).expect("zero should fit in every type")
    }

    // Field is 0 or 1.
    pub(crate) fn bool(value: bool) -> Self {
        Self::int(FieldElement::from(u128::from(value)), NumericType::unsigned(1))
    }

    // Any field value.
    pub(crate) fn field(value: FieldElement) -> Self {
        Self::int(value, NumericType::NativeField)
    }

    // Field in [0, 2^8).
    pub(crate) fn u8(value: u8) -> Self {
        Self::int(FieldElement::from(u128::from(value)), NumericType::unsigned(8))
    }

    // Field in [0, 2^16).
    pub(crate) fn u16(value: u16) -> Self {
        Self::int(FieldElement::from(u128::from(value)), NumericType::unsigned(16))
    }

    // Field in [0, 2^32).
    pub(crate) fn u32(value: u32) -> Self {
        Self::int(FieldElement::from(u128::from(value)), NumericType::unsigned(32))
    }

    // Field in [0, 2^128).
    pub(crate) fn u128(value: u128) -> Self {
        Self::int(FieldElement::from(value), NumericType::unsigned(128))
    }

    // Field in [0, 2^64).
    pub(crate) fn u64(value: u64) -> Self {
        Self::int(FieldElement::from(u128::from(value)), NumericType::unsigned(64))
    }

    // Two's-complement bits in [0, 2^8).
    pub(crate) fn i8(value: i8) -> Self {
        Self::int(FieldElement::from(i128::from(value as u8)), NumericType::signed(8))
    }

    // Two's-complement bits in [0, 2^16).
    pub(crate) fn i16(value: i16) -> Self {
        Self::int(FieldElement::from(i128::from(value as u16)), NumericType::signed(16))
    }

    // Two's-complement bits in [0, 2^32).
    pub(crate) fn i32(value: i32) -> Self {
        Self::int(FieldElement::from(i128::from(value as u32)), NumericType::signed(32))
    }

    // Two's-complement bits in [0, 2^64).
    pub(crate) fn i64(value: i64) -> Self {
        Self::int(FieldElement::from(i128::from(value as u64)), NumericType::signed(64))
    }

    /// Construct a value from a bit-pattern field and type, in-module helper for the typed
    /// constructors above (the type is statically known to be supported).
    fn int(value: FieldElement, typ: NumericType) -> Self {
        Self { value, typ }
    }

    pub(crate) fn as_field(&self) -> Option<FieldElement> {
        matches!(self.typ, NumericType::NativeField).then_some(self.value)
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        if self.typ != NumericType::unsigned(1) {
            return None;
        }
        assert!(
            self.value.is_zero() || self.value.is_one(),
            "a u1 value must be 0 or 1, got {}",
            self.value
        );
        Some(self.value.is_one())
    }

    pub fn to_field(&self) -> FieldElement {
        self.value
    }

    /// The bit width of this value's type. `Field` reports the full field bit size.
    pub(crate) fn bit_size(&self) -> u32 {
        self.typ.bit_size::<FieldElement>()
    }

    /// Whether this value's type is a signed integer.
    pub(crate) fn is_signed(&self) -> bool {
        matches!(self.typ, NumericType::Signed { .. })
    }

    /// Whether the stored field is within the value's type range, i.e. `field < 2^bit_size`.
    ///
    /// `Field` is always in range.
    /// Every other type is in range when its bit pattern fits in `bit_size` bits.
    /// An out-of-range field only arises from unreduced ACIR arithmetic or `int_from_field`,
    /// which does not check; a properly constructed value is always in range.
    pub(crate) fn is_in_range(&self) -> bool {
        match self.typ {
            NumericType::NativeField => true,
            _ => self.value.num_bits() <= self.bit_size(),
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

impl std::fmt::Display for NumericValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bit_size = self.bit_size();
        match self.typ {
            NumericType::NativeField => return write!(f, "Field {}", self.value),
            NumericType::Unsigned { bit_size: 1 } => {
                return write!(f, "u1 {}", if self.value.is_one() { "1" } else { "0" });
            }
            _ => {}
        }

        let prefix = if self.is_signed() { format!("i{bit_size}") } else { format!("u{bit_size}") };

        // The stored field is the value's bit pattern. In-range values print as the plain
        // (signed-aware) number; an out-of-range field — which only arises from unreduced ACIR
        // arithmetic — is shown parenthesized to distinguish it.
        if !self.is_in_range() {
            return write!(f, "{prefix} ({})", self.value);
        }
        if self.is_signed() {
            // Reinterpret the in-range bit pattern as a signed value for display.
            let unsigned = self.value.try_into_u128().expect("in-range value fits in u128");
            let half = 1u128 << (bit_size - 1);
            if unsigned >= half {
                let magnitude = (1u128 << bit_size) - unsigned;
                return write!(f, "{prefix} -{magnitude}");
            }
            return write!(f, "{prefix} {unsigned}");
        }
        write!(f, "{prefix} {}", self.value)
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

        let is_vector = if self.is_vector() { "&" } else { "" };
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
            && self.length == other.length
    }
}

impl Eq for ArrayValue {}
