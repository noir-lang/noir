use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noirc_evaluator::ssa::ir::types::{NumericType, Type};
use noirc_evaluator::ssa::ir::{map::Id, value::Value};
use serde::{Deserialize, Serialize};

#[derive(Arbitrary, Debug, Clone, PartialEq, Eq, Hash, Copy, Serialize, Deserialize)]
pub enum ValueType {
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

#[derive(Debug, Clone, PartialEq)]
pub struct TypedValue {
    pub value_id: Id<Value>,
    pub type_of_variable: Type,
}

impl TypedValue {
    pub fn new(value_id: Id<Value>, type_of_variable: Type) -> Self {
        Self { value_id, type_of_variable }
    }

    /// Convert from our simple ValueType to the internal SSA Type
    pub fn from_value_type(value_id: u32, value_type: &ValueType) -> Self {
        let type_ = match value_type {
            ValueType::Field => Type::field(),
            ValueType::Boolean => Type::bool(),
            ValueType::U8 => Type::unsigned(8),
            ValueType::U16 => Type::unsigned(16),
            ValueType::U32 => Type::unsigned(32),
            ValueType::U64 => Type::unsigned(64),
            ValueType::U128 => Type::unsigned(128),
            ValueType::I8 => Type::signed(8),
            ValueType::I16 => Type::signed(16),
            ValueType::I32 => Type::signed(32),
            ValueType::I64 => Type::signed(64),
        };

        Self { value_id: Id::new(value_id), type_of_variable: type_ }
    }

    /// Convert to our simple ValueType from the internal SSA Type
    pub fn to_value_type(&self) -> ValueType {
        match &self.type_of_variable {
            Type::Numeric(NumericType::NativeField) => ValueType::Field,
            Type::Numeric(NumericType::Unsigned { bit_size: 1 }) => ValueType::Boolean,
            Type::Numeric(NumericType::Unsigned { bit_size: 8 }) => ValueType::U8,
            Type::Numeric(NumericType::Unsigned { bit_size: 16 }) => ValueType::U16,
            Type::Numeric(NumericType::Unsigned { bit_size: 32 }) => ValueType::U32,
            Type::Numeric(NumericType::Unsigned { bit_size: 64 }) => ValueType::U64,
            Type::Numeric(NumericType::Unsigned { bit_size: 128 }) => ValueType::U128,
            Type::Numeric(NumericType::Signed { bit_size: 8 }) => ValueType::I8,
            Type::Numeric(NumericType::Signed { bit_size: 16 }) => ValueType::I16,
            Type::Numeric(NumericType::Signed { bit_size: 32 }) => ValueType::I32,
            Type::Numeric(NumericType::Signed { bit_size: 64 }) => ValueType::I64,
            _ => unreachable!("Not numeric type {}", self.type_of_variable),
        }
    }

    /// Helper to check if this value has a field type
    pub fn is_field(&self) -> bool {
        matches!(&self.type_of_variable, Type::Numeric(NumericType::NativeField))
    }

    /// Helper to check if this value has a signed integer type
    pub fn is_signed(&self) -> bool {
        matches!(&self.type_of_variable, Type::Numeric(NumericType::Signed { .. }))
    }

    /// Helper to check if this value has an unsigned integer type
    pub fn is_unsigned(&self) -> bool {
        matches!(&self.type_of_variable, Type::Numeric(NumericType::Unsigned { .. }))
    }

    /// Get the numeric type if this value has one
    pub fn numeric_type(&self) -> Option<NumericType> {
        match &self.type_of_variable {
            Type::Numeric(num_type) => Some(*num_type),
            _ => None,
        }
    }

    /// Helper to check if shift operations are supported for this type
    pub fn supports_shift(&self) -> bool {
        !self.is_field()
    }

    /// Helper to check if bitwise operations are supported for this type
    pub fn supports_bitwise(&self) -> bool {
        !self.is_field()
    }

    /// Helper to check if modulo operations are supported for this type
    pub fn supports_mod(&self) -> bool {
        !self.is_field()
    }

    /// Helper to check if not operations are supported for this type
    pub fn supports_not(&self) -> bool {
        !self.is_field()
    }

    /// Helper to check if unchecked operations are supported for this type
    pub fn supports_unchecked(&self) -> bool {
        false
    }
}

impl ValueType {
    /// Convert to the SSA Type
    pub fn to_ssa_type(&self) -> Type {
        match self {
            ValueType::Field => Type::field(),
            ValueType::Boolean => Type::bool(),
            ValueType::U8 => Type::unsigned(8),
            ValueType::U16 => Type::unsigned(16),
            ValueType::U32 => Type::unsigned(32),
            ValueType::U64 => Type::unsigned(64),
            ValueType::U128 => Type::unsigned(128),
            ValueType::I8 => Type::signed(8),
            ValueType::I16 => Type::signed(16),
            ValueType::I32 => Type::signed(32),
            ValueType::I64 => Type::signed(64),
        }
    }

    /// Helper to check if this type could be used for casts into it
    /// Signed types are not supported right now
    pub fn can_be_used_for_casts(&self) -> bool {
        true
    }

    /// Convert to the NumericType
    pub fn to_numeric_type(&self) -> NumericType {
        match self {
            ValueType::Field => NumericType::NativeField,
            ValueType::Boolean => NumericType::Unsigned { bit_size: 1 },
            ValueType::U8 => NumericType::Unsigned { bit_size: 8 },
            ValueType::U16 => NumericType::Unsigned { bit_size: 16 },
            ValueType::U32 => NumericType::Unsigned { bit_size: 32 },
            ValueType::U64 => NumericType::Unsigned { bit_size: 64 },
            ValueType::U128 => NumericType::Unsigned { bit_size: 128 },
            ValueType::I8 => NumericType::Signed { bit_size: 8 },
            ValueType::I16 => NumericType::Signed { bit_size: 16 },
            ValueType::I32 => NumericType::Signed { bit_size: 32 },
            ValueType::I64 => NumericType::Signed { bit_size: 64 },
        }
    }

    pub fn bit_length(&self) -> u32 {
        match self {
            ValueType::Field => 254,
            ValueType::Boolean => 1,
            ValueType::U8 => 8,
            ValueType::U16 => 16,
            ValueType::U32 => 32,
            ValueType::U64 => 64,
            ValueType::U128 => 128,
            ValueType::I8 => 8,
            ValueType::I16 => 16,
            ValueType::I32 => 32,
            ValueType::I64 => 64,
        }
    }
}
