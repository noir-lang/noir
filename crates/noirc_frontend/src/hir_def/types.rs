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
    PolymorphicInteger(TypeVariable),
    Bool,
    Unit,
    Struct(FieldElementType, Rc<RefCell<StructType>>),
    Tuple(Vec<Type>),

    Error,
    Unspecified, // This is for when the user declares a variable without specifying it's type
}

type TypeVariable = Rc<RefCell<TypeBinding>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeBinding {
    Bound(Type),
    Unbound(TypeVariableId),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeVariableId(pub usize);

impl Type {
    // These are here so that the code is more readable.
    // Type::WITNESS vs Type::FieldElement(FieldElementType::Private)
    pub const WITNESS: Type = Type::FieldElement(FieldElementType::Private);
    pub const CONSTANT: Type = Type::FieldElement(FieldElementType::Constant);
    pub const PUBLIC: Type = Type::FieldElement(FieldElementType::Public);

    pub const DEFAULT_INT_TYPE: Type = Type::WITNESS;
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
            Type::PolymorphicInteger(binding) => write!(f, "{}", binding.borrow()),
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

impl std::fmt::Display for TypeBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeBinding::Bound(typ) => typ.fmt(f),
            TypeBinding::Unbound(_) => write!(f, "Field"),
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

    pub fn try_bind_to_polymorphic_int(&self, var: &TypeVariable) -> Result<(), ()> {
        let target_id = match &*var.borrow() {
            TypeBinding::Bound(_) => unreachable!(),
            TypeBinding::Unbound(id) => *id,
        };

        match self {
            Type::FieldElement(_)
            | Type::Integer(_, _, _) => {
                *var.borrow_mut() = TypeBinding::Bound(self.clone());
                Ok(())
            }
            Type::PolymorphicInteger(self_var) => {
                match &*self_var.borrow() {
                    TypeBinding::Bound(typ) => typ.try_bind_to_polymorphic_int(var),
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(_) => {
                        *var.borrow_mut() = TypeBinding::Bound(self.clone());
                        Ok(())
                    }
                }
            }
            _ => Err(())
        }
    }

    /// Try to unify this type with another, setting any type variables found
    /// equal to the other type in the process. Unification is more strict
    /// than subtyping but less strict than Eq. Returns true if the unification
    /// succeeded. Note that any bindings performed in a failed unification are
    /// not undone. This may cause further type errors later on.
    pub fn unify(&self, other: &Type, error: &mut impl FnMut()) -> bool {
        use Type::*;
        match (self, other) {
            (Error, _)
            | (_, Error) => true,

            (Unspecified, _)
            | (_, Unspecified) => unreachable!(),

            (PolymorphicInteger(binding), other)
            | (other, PolymorphicInteger(binding)) => {
                // If it is already bound, unify against what it is bound to
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.unify(other, error);
                }

                // Otherwise, check it is unified against an integer and bind it
                let success = other.try_bind_to_polymorphic_int(binding).is_ok();
                if !success {
                    error();
                }
                success
            }

            (Array(vis_a, len_a, elem_a), Array(vis_b, len_b, elem_b)) => {
                if vis_a != vis_b || len_a != len_b {
                    error();
                    false
                } else {
                    elem_a.unify(elem_b, error)
                }
            },

            (Tuple(elems_a), Tuple(elems_b)) => {
                if elems_a.len() != elems_b.len() {
                    error();
                    false
                } else {
                    for (a, b) in elems_a.iter().zip(elems_b) {
                        if !a.unify(b, error) {
                            return false;
                        }
                    }
                    true
                }
            },

            // No recursive unify call for struct fields. Don't want
            // to mutate shared type variables within struct definitions.
            // This isn't possible currently but will be once noir gets generic types
            (Struct(vis_a, fields_a), Struct(vis_b, fields_b)) => {
                if vis_a != vis_b || fields_a != fields_b {
                    error();
                    false
                } else {
                    true
                }
            },

            (other_a, other_b) => {
                let success = other_a == other_b;
                if !success {
                    error();
                }
                success
            }
        }
    }

    // A feature of the language is that `Field` is like an
    // `Any` type which allows you to pass in any type which
    // is fundamentally a field element. E.g all integer types
    //
    // This is 'make_subtype_of' rather than 'is_subtype_of' to
    // allude to the side effects it has with setting any integer
    // type variables contained within to other values
    pub fn make_subtype_of(&self, other: &Type) -> bool {
        use FieldElementType::*;
        use Type::*;

        // Avoid reporting duplicate errors
        if self == &Error || other == &Error {
            return true;
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

            (PolymorphicInteger(a), other) => {
                if let TypeBinding::Bound(binding) = &*a.borrow() {
                    return binding.make_subtype_of(other);
                }
                other.try_bind_to_polymorphic_int(a).is_ok()
            }

            (other, PolymorphicInteger(b)) => {
                if let TypeBinding::Bound(binding) = &*b.borrow() {
                    return other.make_subtype_of(binding);
                }
                other.try_bind_to_polymorphic_int(b).is_ok()
            }

            // Any field element type is a subtype of `priv Field`
            (this, FieldElement(Private)) => this.is_field_element(),

            (Integer(Constant, self_sign, self_bits), Integer(_, other_sign, other_bits)) => {
                self_sign == other_sign && self_bits == other_bits
            }
            (this, other) => this == other,
        }
    }

    pub fn is_field_element(&self) -> bool {
        matches!(self, Type::FieldElement(_) | Type::Bool | Type::Integer(_, _, _) | Type::PolymorphicInteger(_))
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
            | Type::PolymorphicInteger(_)
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

    // Returns true if the Type can be used in a Constrain statement
    pub fn can_be_used_in_constrain(&self) -> bool {
        matches!(
            self,
            Type::FieldElement(_)
                | Type::PolymorphicInteger(_)
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
            Type::PolymorphicInteger(binding) => {
                match &*binding.borrow() {
                    TypeBinding::Bound(typ) => typ.as_abi_type(),
                    TypeBinding::Unbound(_) => Type::DEFAULT_INT_TYPE.as_abi_type(),
                }
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
