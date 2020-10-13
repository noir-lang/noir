/// This module contains two Ident structures, due to the fact that an identifier may or may not return a value
/// statement::Ident does not return a value, while Expression::Ident does.
mod expression;
mod statement;

pub use expression::*;
pub use statement::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArraySize {
    Variable, 
    Fixed(u128)
}

impl ArraySize {
    pub fn is_fixed(&self) -> bool {
        match self {
            ArraySize::Fixed(_) => true, 
            _ => false
        }
    }
}

impl std::fmt::Display for ArraySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArraySize::Variable => write!(f, "[]"),
            ArraySize::Fixed(size) => write!(f, "[{}]", size)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    FieldElement, // This type was introduced for directives.  
    Constant,
    Public,
    Witness,
    Array(ArraySize, Box<Type>),   // [4]Witness = Array(4, Witness)
    Integer(Signedness, u32), // u32 = Integer(unsigned, 32)
    Bool,
    Error, // XXX: Currently have not implemented structs, so this type is a stub
    Unspecified, // This is for when the user declares a variable without specifying it's type
    Unknown, // This is mainly used for literals, where the parser cannot figure out the type for the literal
    Unit,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::FieldElement => write!(f, "Field"),
            Type::Constant => write!(f, "Constant"),
            Type::Public => write!(f, "Public"),
            Type::Witness => write!(f, "Witness"),
            Type::Array(size, typ) => write!(f, "{}{}", size, typ),
            Type::Integer(sign, num_bits) => {
                match sign{
                    Signedness::Signed => write!(f, "i{}", num_bits),                     
                    Signedness::Unsigned => write!(f, "u{}", num_bits),                     
                }
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
            Type::Integer(_,_) => true,
            _=> false
        }
    }


    pub fn is_fixed_sized_array(&self) -> bool {
        let (sized, _ ) = match self.array() {
            None => return false,
            Some(arr) => arr
        };
        sized.is_fixed()
    }
    pub fn is_variable_sized_array(&self) -> bool {
        let (sized, _ ) = match self.array() {
            None => return false,
            Some(arr) => arr
        };
        !sized.is_fixed()
    }

    fn array(&self) -> Option<(&ArraySize, &Type)> {
        match self {
            Type::Array(sized, typ) => Some((sized, typ)),
            _=> None
        }
    }

    // Returns true if the Type can be used in a Let statement
    pub fn can_be_used_in_let(&self) -> bool {    
        self.is_fixed_sized_array()
    }
    // Returns true if the Type can be used in a Constrain statement
    pub fn can_be_used_in_constrain(&self) -> bool {    
        match self {
            Type::Witness => true,
            Type::Integer(_,_) => true,
            Type::Constant => true,
            _=> false
        }
    }

    // Base types are types in the language that are simply alias for a field element 
    // Therefore they can be the operands in an infix comparison operator
    pub fn is_base_type(&self) -> bool {
        match self {
            Type::Constant => true,
            Type::Public => true,
            Type::Witness =>  true,
            Type::Integer(_, _) => true,
            _=> false,
        }
    }

    // Returns true, if both type can be used in an infix expression
    pub fn can_be_used_for_infix(&self, other : &Type) -> bool {
        self.is_base_type() && other.is_base_type()
    }

    // Given a binary operator and another type. This method will produce the 
    // output type
    pub fn infix_operand_type_rules(&self, op : &BinaryOp, other: &Type) -> Type {
        if op.is_comparator() {
            return Type::Bool
        }
        
        match (self, other)  {

            (Type::Integer(sign_x, bit_width_x), Type::Integer(sign_y, bit_width_y)) => {
                assert_eq!(sign_x, sign_y, "Integers must have the same Signedness lhs is {:?}, rhs is {:?} ", sign_x, sign_y);
                assert_eq!(bit_width_x, bit_width_y);
                return Type::Integer(*sign_x, *bit_width_x)
            }
            (Type::Integer(_, _), Type::Witness) | ( Type::Witness, Type::Integer(_, _) ) => { 
                panic!("Cannot use an integer and a witness in a binary operation, try converting the witness into an integer")
            }
            (Type::Integer(sign_x, bit_width_x), Type::Constant)| (Type::Constant,Type::Integer(sign_x, bit_width_x)) => {
                return Type::Integer(*sign_x, *bit_width_x)
            }
            (Type::Integer(_, _), typ) | (typ,Type::Integer(_, _)) => {
                panic!("Integer cannot be used with type {:?}", typ)
            }

            // If no side contains an integer. Then we check if either side contains a witness
            // If either side contains a witness, then the final result will be a witness
            (Type::Witness, _) | (_,Type::Witness) => return Type::Witness,
            // Public types are added as witnesses under the hood
            (Type::Public, _) | (_,Type::Public) => return Type::Witness,
            (Type::Bool, _) | (_,Type::Bool) => return Type::Bool,

            // An error type on either side will always return an error
            (Type::Error, _) | (_,Type::Error) => return Type::Error,
            (Type::Unspecified, _) | (_,Type::Unspecified) => return Type::Unspecified,
            (Type::Unknown, _) | (_,Type::Unknown) => return Type::Unknown,
            (Type::Unit, _) | (_,Type::Unit) => return Type::Unit,

            (Type::FieldElement, _) | (_,Type::FieldElement) => return Type::FieldElement,
            
            // Currently, arrays are not supported in binary operations
            (Type::Array(_,_), _) | (_,Type::Array(_, _)) => return Type::Error,
            
            (Type::Constant, Type::Constant)  => return Type::Constant,

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
