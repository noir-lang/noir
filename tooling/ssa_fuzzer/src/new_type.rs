use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noirc_evaluator::ssa::ir::types::{NumericType as SsaNumericType, Type as SsaType};
use noirc_evaluator::ssa::ir::{map::Id, value::Value};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum_macros::EnumCount;

#[derive(Arbitrary, Debug, Clone, PartialEq, Eq, Hash, Copy, Serialize, Deserialize, EnumCount)]
pub enum NumericType {
    Field,
    Boolean,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
}

impl Into<SsaNumericType> for NumericType {
    fn into(self) -> SsaNumericType {
        match self {
            NumericType::Field => SsaNumericType::NativeField,
            NumericType::Boolean => SsaNumericType::Unsigned { bit_size: 1 },
            NumericType::U8 => SsaNumericType::Unsigned { bit_size: 8 },
            NumericType::U16 => SsaNumericType::Unsigned { bit_size: 16 },
            NumericType::U32 => SsaNumericType::Unsigned { bit_size: 32 },
            NumericType::U64 => SsaNumericType::Unsigned { bit_size: 64 },
            NumericType::U128 => SsaNumericType::Unsigned { bit_size: 128 },
            NumericType::I8 => SsaNumericType::Signed { bit_size: 8 },
            NumericType::I16 => SsaNumericType::Signed { bit_size: 16 },
            NumericType::I32 => SsaNumericType::Signed { bit_size: 32 },
            NumericType::I64 => SsaNumericType::Signed { bit_size: 64 },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Numeric(NumericType),
    Reference(Arc<Type>),
    Array(Arc<Vec<Type>>, u32),
    Slice(Arc<Vec<Type>>),
}

impl Type {
    pub fn bit_length(&self) -> u32 {
        match self {
            Type::Numeric(NumericType::Field) => 254,
            Type::Numeric(NumericType::Boolean) => 1,
            Type::Numeric(NumericType::U8) => 8,
            Type::Numeric(NumericType::U16) => 16,
            Type::Numeric(NumericType::U32) => 32,
            Type::Numeric(NumericType::U64) => 64,
            Type::Numeric(NumericType::U128) => 128,
            Type::Numeric(NumericType::I8) => 8,
            Type::Numeric(NumericType::I16) => 16,
            Type::Numeric(NumericType::I32) => 32,
            Type::Numeric(NumericType::I64) => 64,
            Type::Array(_, _) => unreachable!("Array type unexpected"),
            Type::Slice(_) => unreachable!("Slice type unexpected"),
            Type::Reference(value_type) => value_type.bit_length(),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Numeric(_))
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, Type::Reference(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array(_, _))
    }

    pub fn is_slice(&self) -> bool {
        matches!(self, Type::Slice(_))
    }

    pub fn is_field(&self) -> bool {
        matches!(self, Type::Numeric(NumericType::Field))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Type::Numeric(NumericType::Boolean))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedValue {
    pub value_id: Id<Value>,
    pub type_of_variable: Type,
}

impl TypedValue {
    pub fn new(value_id: Id<Value>, type_of_variable: Type) -> Self {
        Self { value_id, type_of_variable }
    }

    pub fn is_numeric(&self) -> bool {
        self.type_of_variable.is_numeric()
    }

    pub fn is_reference(&self) -> bool {
        self.type_of_variable.is_reference()
    }

    pub fn is_array(&self) -> bool {
        self.type_of_variable.is_array()
    }

    pub fn is_field(&self) -> bool {
        self.type_of_variable.is_field()
    }

    pub fn is_boolean(&self) -> bool {
        self.type_of_variable.is_boolean()
    }

    /// Returns the bit length of the type
    ///
    /// For field returns 254, for references returns the bit length of the referenced type
    /// Panics if the type is an array
    pub fn bit_length(&self) -> u32 {
        self.type_of_variable.bit_length()
    }

    pub fn same_types(&self, other: &TypedValue) -> bool {
        self.type_of_variable == other.type_of_variable
    }

    pub fn unwrap_numeric(&self) -> NumericType {
        match self.type_of_variable {
            Type::Numeric(numeric_type) => numeric_type,
            _ => panic!("Expected NumericType, found {:?}", self.type_of_variable),
        }
    }
}

impl From<SsaType> for Type {
    fn from(type_: SsaType) -> Self {
        match type_ {
            SsaType::Numeric(SsaNumericType::NativeField) => Type::Numeric(NumericType::Field),
            SsaType::Numeric(SsaNumericType::Unsigned { bit_size: 1 }) => {
                Type::Numeric(NumericType::Boolean)
            }
            SsaType::Numeric(SsaNumericType::Unsigned { bit_size: 8 }) => {
                Type::Numeric(NumericType::U8)
            }
            SsaType::Numeric(SsaNumericType::Unsigned { bit_size: 16 }) => {
                Type::Numeric(NumericType::U16)
            }
            SsaType::Numeric(SsaNumericType::Unsigned { bit_size: 32 }) => {
                Type::Numeric(NumericType::U32)
            }
            SsaType::Numeric(SsaNumericType::Unsigned { bit_size: 64 }) => {
                Type::Numeric(NumericType::U64)
            }
            SsaType::Numeric(SsaNumericType::Unsigned { bit_size: 128 }) => {
                Type::Numeric(NumericType::U128)
            }
            SsaType::Numeric(SsaNumericType::Signed { bit_size: 8 }) => {
                Type::Numeric(NumericType::I8)
            }
            SsaType::Numeric(SsaNumericType::Signed { bit_size: 16 }) => {
                Type::Numeric(NumericType::I16)
            }
            SsaType::Numeric(SsaNumericType::Signed { bit_size: 32 }) => {
                Type::Numeric(NumericType::I32)
            }
            SsaType::Numeric(SsaNumericType::Signed { bit_size: 64 }) => {
                Type::Numeric(NumericType::I64)
            }
            SsaType::Array(element_types, length) => Type::Array(
                Arc::new(element_types.iter().map(|t| t.clone().into()).collect()),
                length,
            ),
            SsaType::Reference(element_type) => {
                Type::Reference(Arc::new((*element_type).clone().into()))
            }
            _ => unreachable!("Not supported type: {:?}", type_),
        }
    }
}

impl Into<SsaType> for Type {
    fn into(self) -> SsaType {
        match self {
            Type::Numeric(numeric_type) => SsaType::Numeric(numeric_type.into()),
            Type::Array(element_types, length) => SsaType::Array(
                Arc::new(element_types.iter().map(|t| t.clone().into()).collect()),
                length,
            ),
            Type::Reference(element_type) => {
                SsaType::Reference(Arc::new((*element_type).clone().into()))
            }
            Type::Slice(element_types) => {
                SsaType::Slice(Arc::new(element_types.iter().map(|t| t.clone().into()).collect()))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: TypedValue,
    pub y: TypedValue,
    pub is_infinite: TypedValue,
}
impl Point {
    pub fn validate(&self) -> bool {
        self.x.is_field() && self.y.is_field() && self.is_infinite.is_boolean()
    }
    pub fn to_id_vec(&self) -> Vec<Id<Value>> {
        vec![self.x.value_id, self.y.value_id, self.is_infinite.value_id]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scalar {
    pub lo: TypedValue,
    pub hi: TypedValue,
}

impl Scalar {
    pub fn validate(&self) -> bool {
        self.lo.is_field() && self.hi.is_field()
    }
    pub fn to_id_vec(&self) -> Vec<Id<Value>> {
        vec![self.lo.value_id, self.hi.value_id]
    }
}
