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
    FieldElement(IsConst, FieldElementType),
    Array(FieldElementType, ArraySize, Box<Type>), // [4]Witness = Array(4, Witness)
    Integer(IsConst, FieldElementType, Signedness, u32), // u32 = Integer(unsigned, 32)
    PolymorphicInteger(IsConst, TypeVariable),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IsConst {
    // Yes and No variants have optional spans representing the location in the source code
    // which caused them to be const.
    Yes(Option<Span>),
    No(Option<Span>),
    Maybe(TypeVariableId, Rc<RefCell<Option<IsConst>>>),
}

impl IsConst {
    fn set_span(&mut self, new_span: Span) {
        match self {
            IsConst::Yes(span) | IsConst::No(span) => *span = Some(new_span),
            IsConst::Maybe(_, binding) => {
                if let Some(binding) = &mut *binding.borrow_mut() {
                    binding.set_span(new_span);
                }
            }
        }
    }

    /// Try to unify these two IsConst constraints. Returns true on success.
    pub fn unify(&self, other: &Self, span: Span) -> bool {
        match (self, other) {
            (IsConst::Yes(_), IsConst::Yes(_)) | (IsConst::No(_), IsConst::No(_)) => true,

            (IsConst::Yes(_), IsConst::No(_)) | (IsConst::No(_), IsConst::Yes(_)) => false,

            (IsConst::Maybe(id1, _), IsConst::Maybe(id2, _)) if id1 == id2 => true,

            (IsConst::Maybe(_, binding), other) | (other, IsConst::Maybe(_, binding)) => {
                if let Some(binding) = &*binding.borrow() {
                    return binding.unify(other, span);
                }

                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                true
            }
        }
    }

    /// Combine these two IsConsts together, returning
    /// - IsConst::Yes if both are Yes,
    /// - IsConst::No if either are No,
    /// - or if both are Maybe, unify them both and return the lhs.
    pub fn and(&self, other: &Self, span: Span) -> Self {
        match (self, other) {
            (IsConst::Yes(_), IsConst::Yes(_)) => IsConst::Yes(Some(span)),

            (IsConst::No(_), IsConst::No(_))
            | (IsConst::Yes(_), IsConst::No(_))
            | (IsConst::No(_), IsConst::Yes(_)) => IsConst::No(Some(span)),

            (IsConst::Maybe(id1, _), IsConst::Maybe(id2, _)) if id1 == id2 => self.clone(),

            (IsConst::Maybe(_, binding), other) | (other, IsConst::Maybe(_, binding)) => {
                if let Some(binding) = &*binding.borrow() {
                    return binding.and(other, span);
                }

                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                other.clone()
            }
        }
    }
}

impl Type {
    // These are here so that the code is more readable.
    // Type::WITNESS vs Type::FieldElement(FieldElementType::Private)
    pub const WITNESS: Type = Type::FieldElement(IsConst::No(None), FieldElementType::Private);
    pub const PUBLIC: Type = Type::FieldElement(IsConst::No(None), FieldElementType::Public);

    pub const CONSTANT: Type = Type::FieldElement(IsConst::Yes(None), FieldElementType::Private);

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
        };

        match self {
            Type::FieldElement(is_const, fe_type) => {
                write!(f, "{}{}Field", is_const, vis_str(*fe_type))
            }
            Type::Array(fe_type, size, typ) => write!(f, "{}{}{}", vis_str(*fe_type), size, typ),
            Type::Integer(is_const, fe_type, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{}{}i{}", is_const, vis_str(*fe_type), num_bits),
                Signedness::Unsigned => write!(f, "{}{}u{}", is_const, vis_str(*fe_type), num_bits),
            },
            Type::PolymorphicInteger(_, binding) => write!(f, "{}", binding.borrow()),
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

impl std::fmt::Display for IsConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsConst::Yes(_) => write!(f, "const "),
            IsConst::No(_) => Ok(()),
            IsConst::Maybe(_, binding) => match &*binding.borrow() {
                Some(binding) => binding.fmt(f),
                None => write!(f, "const "),
            },
        }
    }
}

impl Type {
    /// Mutate the span for IsConst to track where constness is required for better
    /// error messages that show both the erroring callsite and the callsite before
    /// which required the variable to be const or non-const.
    fn set_const_span(&mut self, new_span: Span) {
        match self {
            Type::FieldElement(is_const, _) | Type::Integer(is_const, _, _, _) => {
                is_const.set_span(new_span)
            }
            Type::PolymorphicInteger(span, binding) => {
                if let TypeBinding::Bound(binding) = &mut *binding.borrow_mut() {
                    return binding.set_const_span(new_span);
                }
                span.set_span(new_span);
            }
            _ => unreachable!(),
        }
    }

    /// Returns true on success
    pub fn try_bind_to_polymorphic_int(
        &self,
        var: &TypeVariable,
        var_const: &IsConst,
        span: Span,
    ) -> bool {
        let target_id = match &*var.borrow() {
            TypeBinding::Bound(_) => unreachable!(),
            TypeBinding::Unbound(id) => *id,
        };

        match self {
            Type::FieldElement(is_const, ..) | Type::Integer(is_const, ..) => {
                let mut clone = self.clone();
                clone.set_const_span(span);
                *var.borrow_mut() = TypeBinding::Bound(clone);
                is_const.unify(var_const, span)
            }
            Type::PolymorphicInteger(is_const, self_var) => {
                match &*self_var.borrow() {
                    TypeBinding::Bound(typ) => {
                        typ.try_bind_to_polymorphic_int(var, var_const, span)
                    }
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => true,
                    TypeBinding::Unbound(_) => {
                        let mut clone = self.clone();
                        clone.set_const_span(span);
                        *var.borrow_mut() = TypeBinding::Bound(clone);
                        is_const.unify(var_const, span)
                    }
                }
            }
            _ => false,
        }
    }

    /// Try to unify this type with another, setting any type variables found
    /// equal to the other type in the process. Unification is more strict
    /// than subtyping but less strict than Eq. Returns true if the unification
    /// succeeded. Note that any bindings performed in a failed unification are
    /// not undone. This may cause further type errors later on.
    pub fn unify(&self, other: &Type, span: Span, error: &mut impl FnMut()) -> bool {
        let success = self.try_unify(other, span);
        if !success {
            error();
        }
        success
    }

    /// `try_unify` is a bit of a misnomer since although errors are not committed,
    /// any unified bindings are on success.
    fn try_unify(&self, other: &Type, span: Span) -> bool {
        use Type::*;
        match (self, other) {
            (Error, _) | (_, Error) => true,

            (Unspecified, _) | (_, Unspecified) => unreachable!(),

            (PolymorphicInteger(is_const, binding), other)
            | (other, PolymorphicInteger(is_const, binding)) => {
                // If it is already bound, unify against what it is bound to
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.try_unify(other, span);
                }

                // Otherwise, check it is unified against an integer and bind it
                other.try_bind_to_polymorphic_int(binding, is_const, span)
            }

            (Array(_, len_a, elem_a), Array(_, len_b, elem_b)) => {
                len_a == len_b && elem_a.try_unify(elem_b, span)
            }

            (Tuple(elems_a), Tuple(elems_b)) => {
                if elems_a.len() != elems_b.len() {
                    false
                } else {
                    elems_a.iter().zip(elems_b)
                        .all(|(a, b)| a.try_unify(b, span))
                }
            }

            // No recursive try_unify call for struct fields. Don't want
            // to mutate shared type variables within struct definitions.
            // This isn't possible currently but will be once noir gets generic types
            (Struct(_, fields_a), Struct(_, fields_b)) => {
                fields_a == fields_b
            }

            (FieldElement(const_a, _), FieldElement(const_b, _)) => {
                const_a.unify(const_b, span)
            }

            (Integer(const_a, _, signed_a, bits_a), Integer(const_b, _, signed_b, bits_b)) => {
                signed_a == signed_b && bits_a == bits_b && const_a.unify(const_b, span)
            }

            (other_a, other_b) => other_a == other_b,
        }
    }

    // A feature of the language is that `Field` is like an
    // `Any` type which allows you to pass in any type which
    // is fundamentally a field element. E.g all integer types
    //
    // This is 'make_subtype_of' rather than 'is_subtype_of' to
    // allude to the side effects it has with setting any integer
    // type variables contained within to other values
    pub fn make_subtype_of(&self, other: &Type, span: Span) -> bool {
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
            (PolymorphicInteger(is_const, a), other) => {
                if let TypeBinding::Bound(binding) = &*a.borrow() {
                    return binding.make_subtype_of(other, span);
                }
                other.try_bind_to_polymorphic_int(a, is_const, span)
            }

            (other, PolymorphicInteger(is_const, b)) => {
                if let TypeBinding::Bound(binding) = &*b.borrow() {
                    return other.make_subtype_of(binding, span);
                }
                other.try_bind_to_polymorphic_int(b, is_const, span)
            }

            (FieldElement(const_a, _), FieldElement(const_b, _)) => {
                const_a.unify(const_b, span)
            }

            (Integer(const_a, _, signed_a, bits_a), Integer(const_b, _, signed_b, bits_b)) => {
                signed_a == signed_b && bits_a == bits_b && const_a.unify(const_b, span)
            }

            (this, other) => this == other,
        }
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
            Type::FieldElement(..)
            | Type::Integer(..)
            | Type::PolymorphicInteger(..)
            | Type::Bool
            | Type::Error
            | Type::Unspecified
            | Type::Unit => 1,
        }
    }

    pub fn can_be_used_in_constrain(&self) -> bool {
        matches!(
            self,
            Type::FieldElement(..)
                | Type::Integer(..)
                | Type::PolymorphicInteger(..)
                | Type::Array(_, _, _)
                | Type::Error
                | Type::Bool
        )
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

    pub fn is_public(&self) -> bool {
        matches!(
            self,
            Type::FieldElement(_, FieldElementType::Public)
                | Type::Integer(_, FieldElementType::Public, _, _)
                | Type::Array(FieldElementType::Public, _, _)
        )
    }

    // Note; use strict_eq instead of partial_eq when comparing field types
    // in this method, you most likely want to distinguish between public and private
    pub fn as_abi_type(&self) -> AbiType {
        // converts a field element type
        fn fet_to_abi(fe: &FieldElementType) -> AbiFEType {
            match fe {
                FieldElementType::Private => noirc_abi::AbiFEType::Private,
                FieldElementType::Public => noirc_abi::AbiFEType::Public,
            }
        }

        match self {
            Type::FieldElement(_, fe_type) => AbiType::Field(fet_to_abi(fe_type)),
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
            Type::Integer(_, fe_type, sign, bit_width) => {
                let sign = match sign {
                    Signedness::Unsigned => noirc_abi::Sign::Unsigned,
                    Signedness::Signed => noirc_abi::Sign::Signed,
                };

                AbiType::Integer { sign, width: *bit_width as u32, visibility: fet_to_abi(fe_type) }
            }
            Type::PolymorphicInteger(_, binding) => match &*binding.borrow() {
                TypeBinding::Bound(typ) => typ.as_abi_type(),
                TypeBinding::Unbound(_) => Type::DEFAULT_INT_TYPE.as_abi_type(),
            },
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
