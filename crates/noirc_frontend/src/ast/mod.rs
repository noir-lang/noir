/// This module contains two Ident structures, due to the fact that an identifier may or may not return a value
/// statement::Ident does not return a value, while Expression::Ident does.
mod expression;
mod statement;
mod function;
mod structure;

pub use expression::*;
pub use function::*;
pub use structure::*;
use noirc_abi::{AbiFEType, AbiType};
pub use statement::*;

use crate::{node_interner::TypeId, token::IntType};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArraySize {
    Variable,
    Fixed(u128),
}

impl ArraySize {
    fn is_fixed(&self) -> bool {
        matches!(self, ArraySize::Fixed(_))
    }
    fn is_variable(&self) -> bool {
        !self.is_fixed()
    }

    #[allow(clippy::suspicious_operation_groupings)]
    fn is_a_super_type_of(&self, argument: &ArraySize) -> bool {
        (self.is_variable() && argument.is_fixed()) || (self == argument)
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

/// FieldElementType refers to how the Compiler type is interpreted by the proof system
/// Example: FieldElementType::Private means that the Compiler type is seen as a witness/witnesses
#[derive(Debug, Eq, Clone)]
pub enum FieldElementType {
    Private,
    Public,
    Constant,
}

impl PartialEq for FieldElementType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FieldElementType::Private, FieldElementType::Private) => true,
            (FieldElementType::Public, FieldElementType::Public) => true,
            (FieldElementType::Constant, FieldElementType::Constant) => true,
            // The reason we manually implement this, is so that Private and Public
            // are seen as equal
            (FieldElementType::Private, FieldElementType::Public) => true,
            (FieldElementType::Public, FieldElementType::Private) => true,
            (FieldElementType::Private, FieldElementType::Constant) => false,
            (FieldElementType::Public, FieldElementType::Constant) => false,
            (FieldElementType::Constant, FieldElementType::Private) => false,
            (FieldElementType::Constant, FieldElementType::Public) => false,
        }
    }
}

impl FieldElementType {
    // In the majority of places, public and private are
    // interchangeable. The place where the difference does matter is
    // when witnesses are being added to the constraint system.
    // For the compiler, the appropriate place would be in the ABI
    pub fn strict_eq(&self, other: &FieldElementType) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl std::fmt::Display for FieldElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldElementType::Private => write!(f, "priv"),
            FieldElementType::Constant => write!(f, "const"),
            FieldElementType::Public => write!(f, "pub"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    FieldElement(FieldElementType),
    Array(FieldElementType, ArraySize, Box<Type>), // [4]Witness = Array(4, Witness)
    Integer(FieldElementType, Signedness, u32),    // u32 = Integer(unsigned, 32)
    Bool,
    Unit,
    Struct(TypeId),

    Error,       // XXX: Currently have not implemented structs, so this type is a stub
    Unspecified, // This is for when the user declares a variable without specifying it's type
    Unknown, // This is mainly used for array literals, where the parser cannot figure out the type for the literal
}

impl Type {
    // These are here so that the code is more readable.
    // Type::WITNESS vs Type::FieldElement(FieldElementType::Private)
    pub const WITNESS: Type = Type::FieldElement(FieldElementType::Private);
    pub const CONSTANT: Type = Type::FieldElement(FieldElementType::Constant);
    pub const PUBLIC: Type = Type::FieldElement(FieldElementType::Public);
}

impl From<&Type> for AbiType {
    fn from(ty: &Type) -> AbiType {
        ty.as_abi_type()
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::FieldElement(fe_type) => write!(f, "{} Field", fe_type),
            Type::Array(fe_type, size, typ) => write!(f, "{} {}{}", fe_type, size, typ),
            Type::Integer(fe_type, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{} i{}", fe_type, num_bits),
                Signedness::Unsigned => write!(f, "{} u{}", fe_type, num_bits),
            },
            Type::Struct(id) => todo!("Now we need a context parameter for structs"),
            Type::Bool => write!(f, "bool"),
            Type::Unit => write!(f, "()"),
            Type::Error => write!(f, "Error"),
            Type::Unspecified => write!(f, "unspecified"),
            Type::Unknown => write!(f, "unknown"),
        }
    }
}

impl Type {
    // Returns true if the Type can be used in a Private statement
    pub fn can_be_used_in_priv(&self) -> bool {
        match self {
            Type::FieldElement(FieldElementType::Private) => true,
            Type::Integer(field_type, _, _) => field_type == &FieldElementType::Private,
            _ => false,
        }
    }

    // A feature of the language is that `Field` is like an
    // `Any` type which allows you to pass in any type which
    // is fundamentally a field element. E.g all integer types
    pub fn is_super_type_of(&self, argument: &Type) -> bool {
        // if `self` is a `Field` then it is a super type
        // if the argument is a field element
        if let Type::FieldElement(FieldElementType::Private) = self {
            return argument.is_field_element();
        }

        // For composite types, we need to check they are structurally the same
        // and then check that their base types are super types
        if let (Type::Array(_, param_size, param_type), Type::Array(_, arg_size, arg_type)) =
            (self, argument)
        {
            let is_super_type = param_type.is_super_type_of(arg_type);

            let arity_check = param_size.is_a_super_type_of(arg_size);

            return is_super_type && arity_check;
        }

        // XXX: Should we also allow functions that ask for u16
        // to accept u8? We would need to pad the bit decomposition
        // if so.

        self == argument
    }

    pub fn is_field_element(&self) -> bool {
        matches!(
            self,
            Type::FieldElement(_) | Type::Bool | Type::Integer(_, _, _)
        )
    }

    /// Computes the number of elements in a Type
    /// Arrays and Structs will be the only data structures to return more than one

    pub fn num_elements(&self) -> usize {
        let arr_size = match self {
            Type::Array(_, size, _) => size,
            Type::Struct(_) => todo!("Getting the elements of a struct requires context"),
            Type::FieldElement(_)
            | Type::Integer(_, _, _)
            | Type::Bool
            | Type::Error
            | Type::Unspecified
            | Type::Unknown
            | Type::Unit => return 1,
        };

        match arr_size {
            ArraySize::Variable => unreachable!("ice : this method is only ever called when we want to compare the prover inputs with the ABI in main. The ABI should not have variable input. The program should be compiled before calling this"),
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
            Type::Array(_, sized, typ) => Some((sized, typ)),
            _ => None,
        }
    }

    // Returns true if the Type can be used in a Let statement
    pub fn can_be_used_in_let(&self) -> bool {
        self.is_fixed_sized_array() || self.is_variable_sized_array()
    }
    // Returns true if the Type can be used in a Constrain statement
    pub fn can_be_used_in_constrain(&self) -> bool {
        matches!(
            self,
            Type::FieldElement(_) | Type::Integer(_, _, _) | Type::Array(_, _, _)
        )
    }

    // Base types are types in the language that are simply alias for a field element
    // Therefore they can be the operands in an infix comparison operator
    pub fn is_base_type(&self) -> bool {
        matches!(self, Type::FieldElement(_) | Type::Integer(_, _, _))
    }

    pub fn is_constant(&self) -> bool {
        // XXX: Currently no such thing as a const array
        matches!(
            self,
            Type::FieldElement(FieldElementType::Constant)
                | Type::Integer(FieldElementType::Constant, _, _)
        )
    }

    pub fn is_public(&self) -> bool {
        matches!(
            self,
            Type::FieldElement(FieldElementType::Public)
                | Type::Integer(FieldElementType::Public, _, _)
                | Type::Array(FieldElementType::Public, _, _)
        )
    }

    // Returns true, if both type can be used in an infix expression
    pub fn can_be_used_for_infix(&self, other: &Type) -> bool {
        self.is_base_type() && other.is_base_type()
    }

    // Note; use strict_eq instead of partial_eq when comparing field types
    // in this method, you most likely want to distinguish between public and private
    pub fn as_abi_type(&self) -> AbiType {
        // converts a field element type
        fn fet_to_abi(fe: &FieldElementType) -> AbiFEType {
            match fe {
                FieldElementType::Private => noirc_abi::AbiFEType::Private,
                FieldElementType::Public => noirc_abi::AbiFEType::Public,
                FieldElementType::Constant => {
                    panic!("constant field in the ABI, this is not allowed!")
                }
            }
        }

        match self {
            Type::FieldElement(fe_type) => AbiType::Field(fet_to_abi(fe_type)),
            Type::Array(fe_type, size, typ) => match size {
                crate::ArraySize::Variable => {
                    panic!("cannot have variable sized array in entry point")
                }
                crate::ArraySize::Fixed(length) => AbiType::Array {
                    visibility: fet_to_abi(fe_type),
                    length: *length,
                    typ: Box::new(typ.as_abi_type()),
                },
            },
            Type::Integer(fe_type, sign, bit_width) => {
                let sign = match sign {
                    crate::Signedness::Unsigned => noirc_abi::Sign::Unsigned,
                    crate::Signedness::Signed => noirc_abi::Sign::Signed,
                };

                AbiType::Integer {
                    sign,
                    width: *bit_width as u32,
                    visibility: fet_to_abi(fe_type),
                }
            }
            Type::Bool => panic!("currently, cannot have a bool in the entry point function"),
            Type::Error => unreachable!(),
            Type::Unspecified => unreachable!(),
            Type::Unknown => unreachable!(),
            Type::Unit => unreachable!(),
            Type::Struct(_) => todo!(),
        }
    }
}

impl Type {
    pub fn from_int_tok(field_type: FieldElementType, int_tok: &IntType) -> Type {
        match int_tok {
            IntType::Signed(num_bits) => Type::Integer(field_type, Signedness::Signed, *num_bits),
            IntType::Unsigned(num_bits) => {
                Type::Integer(field_type, Signedness::Unsigned, *num_bits)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Signedness {
    Unsigned,
    Signed,
}
