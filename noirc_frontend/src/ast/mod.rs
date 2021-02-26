/// This module contains two Ident structures, due to the fact that an identifier may or may not return a value
/// statement::Ident does not return a value, while Expression::Ident does.
mod expression;
mod function;
mod statement;

pub use expression::*;
pub use function::*;
use noirc_abi::AbiType;
pub use statement::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArraySize {
    Variable,
    Fixed(u128),
}

impl ArraySize {
    pub fn is_fixed(&self) -> bool {
        match self {
            ArraySize::Fixed(_) => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for ArraySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArraySize::Variable => write!(f, "[]"),
            ArraySize::Fixed(size) => write!(f, "[{}]", size),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    FieldElement, // This type was introduced for directives.
    Constant,
    Public,
    Witness,
    Array(ArraySize, Box<Type>), // [4]Witness = Array(4, Witness)
    Integer(Signedness, u32),    // u32 = Integer(unsigned, 32)
    Bool,
    Error,       // XXX: Currently have not implemented structs, so this type is a stub
    Unspecified, // This is for when the user declares a variable without specifying it's type
    Unknown, // This is mainly used for array literals, where the parser cannot figure out the type for the literal
    Unit,
}

impl Into<AbiType> for &Type {
    fn into(self) -> AbiType {
        self.as_abi_type()
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::FieldElement => write!(f, "Field"),
            Type::Constant => write!(f, "Constant"),
            Type::Public => write!(f, "Public"),
            Type::Witness => write!(f, "Witness"),
            Type::Array(size, typ) => write!(f, "{}{}", size, typ),
            Type::Integer(sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "i{}", num_bits),
                Signedness::Unsigned => write!(f, "u{}", num_bits),
            },
            Type::Bool => write!(f, "bool"),
            Type::Error => write!(f, "Error"),
            Type::Unspecified => write!(f, "unspecified"),
            Type::Unknown => write!(f, "unknown"),
            Type::Unit => write!(f, "()"),
        }
    }
}

impl Type {
    // Returns true if the Type can be used in a Private statement
    pub fn can_be_used_in_priv(&self) -> bool {
        match self {
            Type::Witness => true,
            Type::Integer(_, _) => true,
            _ => false,
        }
    }

    /// Computes the number of elements in a Type
    /// Arrays and Structs will be the only data structures to return more than one

    pub fn num_elements(&self) -> usize {
        let arr_size = match self {
            Type::Array(size, _) => size,
            Type::FieldElement
            | Type::Constant
            | Type::Public
            | Type::Witness
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Error
            | Type::Unspecified
            | Type::Unknown
            | Type::Unit => return 1,
        };

        match arr_size {
            ArraySize::Variable => unreachable!("ice : this method is only ever called when we want to compare the prover inputs with the abi in main. The ABI should not have variable input. The program should be compiled before calling this"),
            ArraySize::Fixed(fixed_size) => *fixed_size as usize
        }
    }

    pub fn is_fixed_sized_array(&self) -> bool {
        let (sized, _) = match self.array() {
            None => return false,
            Some(arr) => arr,
        };
        sized.is_fixed()
    }
    pub fn is_variable_sized_array(&self) -> bool {
        let (sized, _) = match self.array() {
            None => return false,
            Some(arr) => arr,
        };
        !sized.is_fixed()
    }

    fn array(&self) -> Option<(&ArraySize, &Type)> {
        match self {
            Type::Array(sized, typ) => Some((sized, typ)),
            _ => None,
        }
    }

    // Returns true if the Type can be used in a Let statement
    pub fn can_be_used_in_let(&self) -> bool {
        self.is_fixed_sized_array() || self.is_variable_sized_array()
    }
    // Returns true if the Type can be used in a Constrain statement
    pub fn can_be_used_in_constrain(&self) -> bool {
        match self {
            Type::Witness => true,
            Type::Public => true,
            Type::Integer(_, _) => true,
            Type::Constant => true,
            _ => false,
        }
    }

    // Base types are types in the language that are simply alias for a field element
    // Therefore they can be the operands in an infix comparison operator
    pub fn is_base_type(&self) -> bool {
        match self {
            Type::Constant => true,
            Type::Public => true,
            Type::Witness => true,
            Type::Integer(_, _) => true,
            _ => false,
        }
    }

    // Returns true, if both type can be used in an infix expression
    pub fn can_be_used_for_infix(&self, other: &Type) -> bool {
        self.is_base_type() && other.is_base_type()
    }

    pub fn as_abi_type(&self) -> AbiType {
        match self {
            Type::FieldElement => {
                panic!("currently, cannot have a field in the entry point function")
            }
            Type::Constant => panic!("cannot have a constant in the entry point function"),
            Type::Public => AbiType::Public,
            Type::Witness => AbiType::Private,
            Type::Array(size, typ) => match size {
                crate::ArraySize::Variable => {
                    panic!("cannot have variable sized array in entry point")
                }
                crate::ArraySize::Fixed(length) => AbiType::Array {
                    length: *length,
                    typ: Box::new(typ.as_abi_type()),
                },
            },
            Type::Integer(sign, bit_width) => {
                let sign = match sign {
                    crate::Signedness::Unsigned => noirc_abi::Sign::Unsigned,
                    crate::Signedness::Signed => noirc_abi::Sign::Signed,
                };

                AbiType::Integer {
                    sign,
                    width: *bit_width as u32,
                }
            }
            Type::Bool => panic!("currently, cannot have a bool in the entry point function"),
            Type::Error => unreachable!(),
            Type::Unspecified => unreachable!(),
            Type::Unknown => unreachable!(),
            Type::Unit => unreachable!(),
        }
    }
}

use crate::token::IntType;

impl From<&IntType> for Type {
    fn from(it: &IntType) -> Type {
        match it {
            IntType::Signed(num_bits) => Type::Integer(Signedness::Signed, *num_bits),
            IntType::Unsigned(num_bits) => Type::Integer(Signedness::Unsigned, *num_bits),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Signedness {
    Unsigned,
    Signed,
}
