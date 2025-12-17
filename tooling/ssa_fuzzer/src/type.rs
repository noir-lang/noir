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

impl NumericType {
    pub fn bit_length(&self) -> u32 {
        match self {
            NumericType::Field => 254,
            NumericType::Boolean => 1,
            NumericType::U8 => 8,
            NumericType::U16 => 16,
            NumericType::U32 => 32,
            NumericType::U64 => 64,
            NumericType::U128 => 128,
            NumericType::I8 => 8,
            NumericType::I16 => 16,
            NumericType::I32 => 32,
            NumericType::I64 => 64,
        }
    }
}

impl From<NumericType> for SsaNumericType {
    fn from(numeric_type: NumericType) -> Self {
        let bit_size = numeric_type.bit_length();
        match numeric_type {
            NumericType::Field => SsaNumericType::NativeField,
            NumericType::Boolean => SsaNumericType::Unsigned { bit_size },
            NumericType::U8 => SsaNumericType::Unsigned { bit_size },
            NumericType::U16 => SsaNumericType::Unsigned { bit_size },
            NumericType::U32 => SsaNumericType::Unsigned { bit_size },
            NumericType::U64 => SsaNumericType::Unsigned { bit_size },
            NumericType::U128 => SsaNumericType::Unsigned { bit_size },
            NumericType::I8 => SsaNumericType::Signed { bit_size },
            NumericType::I16 => SsaNumericType::Signed { bit_size },
            NumericType::I32 => SsaNumericType::Signed { bit_size },
            NumericType::I64 => SsaNumericType::Signed { bit_size },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumCount)]
pub enum Type {
    Numeric(NumericType),
    Reference(Arc<Type>),
    Array(Arc<Vec<Type>>, u32),
    List(Arc<Vec<Type>>),
}

/// Used as default value for mutations
impl Default for Type {
    fn default() -> Self {
        Type::Numeric(NumericType::Field)
    }
}

impl Type {
    pub fn bit_length(&self) -> u32 {
        match self {
            Type::Numeric(numeric_type) => numeric_type.bit_length(),
            Type::Array(_, _) => unreachable!("Array type unexpected"),
            Type::List(_) => unreachable!("List type unexpected"),
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

    pub fn is_list(&self) -> bool {
        matches!(self, Type::List(_))
    }

    pub fn is_field(&self) -> bool {
        matches!(self, Type::Numeric(NumericType::Field))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Type::Numeric(NumericType::Boolean))
    }

    // TODO(sn): legacy
    pub fn is_array_of_references(&self) -> bool {
        match self {
            Type::Array(element_types, _) => element_types.iter().all(|t| t.is_reference()),
            _ => false,
        }
    }

    pub fn unwrap_reference(&self) -> Type {
        match self {
            Type::Reference(value_type) => value_type.as_ref().clone(),
            _ => panic!("Expected Reference, found {self:?}"),
        }
    }

    pub fn unwrap_numeric(&self) -> NumericType {
        match self {
            Type::Numeric(numeric_type) => *numeric_type,
            _ => panic!("Expected NumericType, found {self:?}"),
        }
    }

    pub fn unwrap_array_element_type(&self) -> Type {
        match self {
            Type::Array(element_types, _) => element_types[0].clone(),
            _ => panic!("Expected Array, found {self:?}"),
        }
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
            SsaType::List(element_types) => {
                Type::List(Arc::new(element_types.iter().map(|t| t.clone().into()).collect()))
            }
            _ => unreachable!("Not supported type: {:?}", type_),
        }
    }
}

impl From<Type> for SsaType {
    fn from(typ: Type) -> Self {
        match typ {
            Type::Numeric(numeric_type) => SsaType::Numeric(numeric_type.into()),
            Type::Array(element_types, length) => SsaType::Array(
                Arc::new(element_types.iter().map(|t| t.clone().into()).collect()),
                length,
            ),
            Type::Reference(element_type) => {
                SsaType::Reference(Arc::new((*element_type).clone().into()))
            }
            Type::List(element_types) => {
                SsaType::List(Arc::new(element_types.iter().map(|t| t.clone().into()).collect()))
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
