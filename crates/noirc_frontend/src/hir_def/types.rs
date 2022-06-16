use std::{cell::RefCell, collections::HashMap, rc::Rc};

use noirc_abi::{AbiFEType, AbiType};
use noirc_errors::Span;

use crate::{
    node_interner::{FuncId, StructId},
    util::vecmap,
    ArraySize, FieldElementType, Ident, Signedness,
};

#[derive(Debug, Eq)]
pub struct StructType {
    pub id: StructId,
    pub name: Ident,
    pub fields: Vec<(Ident, Type)>,
    pub methods: HashMap<String, FuncId>,
    pub span: Span,
}

impl PartialEq for StructType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl StructType {
    pub fn new(id: StructId, name: Ident, span: Span, fields: Vec<(Ident, Type)>) -> StructType {
        StructType { id, fields, name, span, methods: HashMap::new() }
    }

    pub fn get_field(&self, field_name: &str) -> Option<&Type> {
        self.fields.iter().find(|(name, _)| name.0.contents == field_name).map(|(_, typ)| typ)
    }
}

impl std::fmt::Display for StructType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    FieldElement(FieldElementType),
    Array(FieldElementType, ArraySize, Box<Type>), // [4]Witness = Array(4, Witness)
    Integer(FieldElementType, Signedness, u32),    // u32 = Integer(unsigned, 32)
    Bool,
    Unit,
    Struct(FieldElementType, Rc<RefCell<StructType>>),
    Tuple(Vec<Type>),

    Error,
    Unspecified, // This is for when the user declares a variable without specifying it's type
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
        let vis_str = |vis| match vis {
            FieldElementType::Private => "",
            FieldElementType::Public => "pub ",
            FieldElementType::Constant => "const ",
        };

        match self {
            Type::FieldElement(fe_type) => write!(f, "{}Field", vis_str(*fe_type)),
            Type::Array(fe_type, size, typ) => write!(f, "{}{}{}", vis_str(*fe_type), size, typ),
            Type::Integer(fe_type, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{}i{}", vis_str(*fe_type), num_bits),
                Signedness::Unsigned => write!(f, "{}u{}", vis_str(*fe_type), num_bits),
            },
            Type::Struct(vis, s) => write!(f, "{}{}", vis_str(*vis), s.borrow()),
            Type::Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Type::Bool => write!(f, "bool"),
            Type::Unit => write!(f, "()"),
            Type::Error => write!(f, "error"),
            Type::Unspecified => write!(f, "unspecified"),
        }
    }
}

impl Type {
    // Returns true if the Type can be used in a Private statement
    pub fn can_be_used_in_priv(&self) -> bool {
        match self {
            Type::FieldElement(FieldElementType::Private) => true,
            Type::Integer(field_type, _, _) => field_type == &FieldElementType::Private,
            Type::Error => true,
            _ => false,
        }
    }

    /// True if this type 'matches' another. Matching is more strict
    /// than subtyping but less strict than Eq. In particular, a type
    /// matches another iff it is exactly equal to the other type or
    /// either type is the Error type.
    pub fn matches(&self, other: &Type) -> bool {
        self == other || self == &Type::Error || other == &Type::Error
    }

    // A feature of the language is that `Field` is like an
    // `Any` type which allows you to pass in any type which
    // is fundamentally a field element. E.g all integer types
    pub fn is_subtype_of(&self, other: &Type) -> bool {
        use FieldElementType::*;
        use Type::*;

        // Avoid reporting duplicate errors
        if self == &Error || other == &Error {
            return true;
        }

        // Any field element type is a subtype of `priv Field`
        if other == &FieldElement(Private) {
            return self.is_field_element();
        }

        if let (Array(_, arg_size, arg_type), Array(_, param_size, param_type)) = (self, other) {
            // We require array elements to be exactly equal, though an array with known
            // length is a subtype of an array with an unknown length. Originally arrays were
            // covariant (so []i32 <: []Field), but it was changed to be more like rusts and
            // require explicit casts.
            return arg_type == param_type && arg_size.is_subtype_of(param_size);
        }

        // XXX: Should we also allow functions that ask for u16
        // to accept u8? We would need to pad the bit decomposition
        // if so.
        match (self, other) {
            // Const types are subtypes of non-const types
            (FieldElement(Constant), FieldElement(_)) => true,
            (Integer(Constant, self_sign, self_bits), Integer(_, other_sign, other_bits)) => {
                self_sign == other_sign && self_bits == other_bits
            }
            (this, other) => this == other,
        }
    }

    pub fn is_field_element(&self) -> bool {
        matches!(self, Type::FieldElement(_) | Type::Bool | Type::Integer(_, _, _))
    }

    /// Computes the number of elements in a Type
    /// Arrays, structs, and tuples will be the only data structures to return more than one
    pub fn num_elements(&self) -> usize {
        match self {
            Type::Array(_, ArraySize::Fixed(fixed_size), _) => *fixed_size as usize,
            Type::Array(_, ArraySize::Variable, _) =>
                unreachable!("ice : this method is only ever called when we want to compare the prover inputs with the ABI in main. The ABI should not have variable input. The program should be compiled before calling this"),
            Type::Struct(_, s) => s.borrow().fields.len(),
            Type::Tuple(fields) => fields.len(),
            Type::FieldElement(_)
            | Type::Integer(_, _, _)
            | Type::Bool
            | Type::Error
            | Type::Unspecified
            | Type::Unit => 1,
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
        self.is_fixed_sized_array()
            || self.is_variable_sized_array()
            || matches!(self, &Type::Struct(..))
            || self == &Type::Error
    }

    // Returns true if the Type can be used in a Constrain statement
    pub fn can_be_used_in_constrain(&self) -> bool {
        matches!(
            self,
            Type::FieldElement(_)
                | Type::Integer(_, _, _)
                | Type::Array(_, _, _)
                | Type::Error
                | Type::Bool
        )
    }

    // Base types are types in the language that are simply alias for a field element
    // Therefore they can be the operands in an infix comparison operator
    pub fn is_base_type(&self) -> bool {
        matches!(self, Type::FieldElement(_) | Type::Integer(_, _, _) | Type::Error)
    }

    pub fn is_constant(&self) -> bool {
        // XXX: Currently no such thing as a const array
        matches!(
            self,
            Type::FieldElement(FieldElementType::Constant)
                | Type::Integer(FieldElementType::Constant, _, _)
                | Type::Error
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
                ArraySize::Variable => {
                    panic!("cannot have variable sized array in entry point")
                }
                ArraySize::Fixed(length) => AbiType::Array {
                    visibility: fet_to_abi(fe_type),
                    length: *length,
                    typ: Box::new(typ.as_abi_type()),
                },
            },
            Type::Integer(fe_type, sign, bit_width) => {
                let sign = match sign {
                    Signedness::Unsigned => noirc_abi::Sign::Unsigned,
                    Signedness::Signed => noirc_abi::Sign::Signed,
                };

                AbiType::Integer { sign, width: *bit_width as u32, visibility: fet_to_abi(fe_type) }
            }
            Type::Bool => panic!("currently, cannot have a bool in the entry point function"),
            Type::Error => unreachable!(),
            Type::Unspecified => unreachable!(),
            Type::Unit => unreachable!(),
            Type::Struct(_, _) => todo!("as_abi_type not yet implemented for struct types"),
            Type::Tuple(_) => todo!("as_abi_type not yet implemented for tuple types"),
        }
    }

    /// Iterate over the fields of this type.
    /// Panics if the type is not a struct or tuple.
    pub fn iter_fields(&self) -> impl Iterator<Item = (String, Type)> {
        let fields: Vec<_> = match self {
            // Unfortunately the .borrow() here forces us to collect into a Vec
            // only to have to call .into_iter again afterward. Trying to ellide
            // collecting to a Vec leads to us dropping the temporary Ref before
            // the iterator is returned
            Type::Struct(_, def) => {
                vecmap(def.borrow().fields.iter(), |(name, typ)| (name.to_string(), typ.clone()))
            }
            Type::Tuple(fields) => {
                let fields = fields.iter().enumerate();
                vecmap(fields, |(i, field)| (i.to_string(), field.clone()))
            }
            other => panic!("Tried to iterate over the fields of '{}', which has none", other),
        };
        fields.into_iter()
    }
}
