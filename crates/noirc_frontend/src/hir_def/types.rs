use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{hir::type_check::TypeCheckError, node_interner::NodeInterner};
use noirc_abi::{AbiFEType, AbiType};
use noirc_errors::Span;

use crate::{
    node_interner::{FuncId, StructId},
    util::vecmap,
    Ident, Signedness,
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
pub enum ArraySize {
    Variable,
    Fixed(u128),
}

impl ArraySize {
    pub fn is_fixed(&self) -> bool {
        matches!(self, ArraySize::Fixed(_))
    }

    pub fn is_variable(&self) -> bool {
        !self.is_fixed()
    }

    pub fn is_subtype_of(&self, argument: &ArraySize) -> bool {
        (self.is_fixed() && argument.is_variable()) || (self == argument)
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
    FieldElement(IsConst),
    Array(ArraySize, Box<Type>),       // [4]Witness = Array(4, Witness)
    Integer(IsConst, Signedness, u32), // u32 = Integer(unsigned, 32)
    PolymorphicInteger(IsConst, TypeVariable),
    Bool(IsConst),
    Unit,
    Struct(Rc<RefCell<StructType>>),
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

/// Internal enum for `unify` to remember the type context of each span
/// to provide better error messages
#[derive(Debug)]
pub enum SpanKind {
    Const(Span),
    NonConst(Span),
    None,
}

impl IsConst {
    pub fn new(interner: &mut NodeInterner) -> Self {
        let id = interner.next_type_variable_id();
        Self::Maybe(id, Rc::new(RefCell::new(None)))
    }

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

    /// Try to unify these two IsConst constraints.
    pub fn unify(&self, other: &Self, span: Span) -> Result<(), SpanKind> {
        match (self, other) {
            (IsConst::Yes(_), IsConst::Yes(_)) | (IsConst::No(_), IsConst::No(_)) => Ok(()),

            (IsConst::Yes(y), IsConst::No(n)) | (IsConst::No(n), IsConst::Yes(y)) => {
                Err(match (y, n) {
                    (_, Some(span)) => SpanKind::NonConst(*span),
                    (Some(span), _) => SpanKind::Const(*span),
                    _ => SpanKind::None,
                })
            }

            (IsConst::Maybe(id1, _), IsConst::Maybe(id2, _)) if id1 == id2 => Ok(()),

            (IsConst::Maybe(_, binding), other) | (other, IsConst::Maybe(_, binding)) => {
                if let Some(binding) = &*binding.borrow() {
                    return binding.unify(other, span);
                }

                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                Ok(())
            }
        }
    }

    /// Try to unify these two IsConst constraints.
    pub fn is_subtype_of(&self, other: &Self, span: Span) -> Result<(), SpanKind> {
        match (self, other) {
            (IsConst::Yes(_), IsConst::Yes(_))
            | (IsConst::No(_), IsConst::No(_))

            // This is the only differing case between this and IsConst::unify
            | (IsConst::Yes(_), IsConst::No(_)) => Ok(()),

            (IsConst::No(n), IsConst::Yes(y)) => {
                Err(match (y, n) {
                    (_, Some(span)) => SpanKind::NonConst(*span),
                    (Some(span), _) => SpanKind::Const(*span),
                    _ => SpanKind::None,
                })
            }

            (IsConst::Maybe(id1, _), IsConst::Maybe(id2, _)) if id1 == id2 => Ok(()),

            (IsConst::Maybe(_, binding), other) => {
                if let Some(binding) = &*binding.borrow() {
                    return binding.is_subtype_of(other, span);
                }

                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                Ok(())
            }
            (other, IsConst::Maybe(_, binding)) => {
                if let Some(binding) = &*binding.borrow() {
                    return other.is_subtype_of(binding, span);
                }

                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                Ok(())
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

    pub fn is_const(&self) -> bool {
        match self {
            IsConst::Yes(_) => true,
            IsConst::No(_) => false,
            IsConst::Maybe(_, binding) => {
                if let Some(binding) = &*binding.borrow() {
                    return binding.is_const();
                }
                true
            }
        }
    }
}

impl Type {
    pub fn field(span: Option<Span>) -> Type {
        Type::FieldElement(IsConst::No(span))
    }

    pub fn constant(span: Option<Span>) -> Type {
        Type::FieldElement(IsConst::Yes(span))
    }

    pub fn default_int_type(span: Option<Span>) -> Type {
        Type::field(span)
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::FieldElement(is_const) => {
                write!(f, "{}Field", is_const)
            }
            Type::Array(size, typ) => write!(f, "{}{}", size, typ),
            Type::Integer(is_const, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{}i{}", is_const, num_bits),
                Signedness::Unsigned => write!(f, "{}u{}", is_const, num_bits),
            },
            Type::PolymorphicInteger(_, binding) => write!(f, "{}", binding.borrow()),
            Type::Struct(s) => write!(f, "{}", s.borrow()),
            Type::Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Type::Bool(is_const) => write!(f, "{}bool", is_const),
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
    pub fn set_const_span(&mut self, new_span: Span) {
        match self {
            Type::FieldElement(is_const) | Type::Integer(is_const, _, _) => {
                is_const.set_span(new_span)
            }
            Type::PolymorphicInteger(span, binding) => {
                if let TypeBinding::Bound(binding) = &mut *binding.borrow_mut() {
                    return binding.set_const_span(new_span);
                }
                span.set_span(new_span);
            }
            _ => (),
        }
    }

    /// Returns true on success
    pub fn try_bind_to_polymorphic_int(
        &self,
        var: &TypeVariable,
        var_const: &IsConst,
        span: Span,
    ) -> Result<(), SpanKind> {
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
                let borrow = self_var.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => {
                        typ.try_bind_to_polymorphic_int(var, var_const, span)
                    }
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(_) => {
                        drop(borrow);
                        let mut clone = self.clone();
                        clone.set_const_span(span);
                        *var.borrow_mut() = TypeBinding::Bound(clone);
                        is_const.unify(var_const, span)
                    }
                }
            }
            _ => Err(SpanKind::None),
        }
    }

    fn is_const(&self) -> bool {
        match self {
            Type::FieldElement(is_const) => is_const.is_const(),
            Type::Integer(is_const, ..) => is_const.is_const(),
            Type::PolymorphicInteger(is_const, binding) => {
                if let TypeBinding::Bound(binding) = &*binding.borrow() {
                    return binding.is_const();
                }
                is_const.is_const()
            }
            _ => false,
        }
    }

    /// Try to unify this type with another, setting any type variables found
    /// equal to the other type in the process. Unification is more strict
    /// than subtyping but less strict than Eq. Returns true if the unification
    /// succeeded. Note that any bindings performed in a failed unification are
    /// not undone. This may cause further type errors later on.
    pub fn unify(
        &self,
        expected: &Type,
        span: Span,
        errors: &mut Vec<TypeCheckError>,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        if let Err(err_span) = self.try_unify(expected, span) {
            Self::issue_errors(expected, err_span, errors, make_error)
        }
    }

    fn issue_errors(
        expected: &Type,
        err_span: SpanKind,
        errors: &mut Vec<TypeCheckError>,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        errors.push(make_error());

        match (expected.is_const(), err_span) {
            (true, SpanKind::NonConst(span)) => {
                let msg = "The value is non-const because of this expression, which uses another non-const value".into();
                errors.push(TypeCheckError::Unstructured { msg, span });
            }
            (false, SpanKind::Const(span)) => {
                let msg = "The value is const because of this expression, which forces the value to be const".into();
                errors.push(TypeCheckError::Unstructured { msg, span });
            }
            _ => (),
        }
    }

    /// `try_unify` is a bit of a misnomer since although errors are not committed,
    /// any unified bindings are on success.
    fn try_unify(&self, other: &Type, span: Span) -> Result<(), SpanKind> {
        use Type::*;
        match (self, other) {
            (Error, _) | (_, Error) => Ok(()),

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

            (Array(len_a, elem_a), Array(len_b, elem_b)) => {
                if len_a == len_b {
                    elem_a.try_unify(elem_b, span)
                } else {
                    Err(SpanKind::None)
                }
            }

            (Tuple(elems_a), Tuple(elems_b)) => {
                if elems_a.len() != elems_b.len() {
                    Err(SpanKind::None)
                } else {
                    for (a, b) in elems_a.iter().zip(elems_b) {
                        a.try_unify(b, span)?;
                    }
                    Ok(())
                }
            }

            // No recursive try_unify call for struct fields. Don't want
            // to mutate shared type variables within struct definitions.
            // This isn't possible currently but will be once noir gets generic types
            (Struct(fields_a), Struct(fields_b)) => {
                if fields_a == fields_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }

            (FieldElement(const_a), FieldElement(const_b)) => const_a.unify(const_b, span),

            (Integer(const_a, signed_a, bits_a), Integer(const_b, signed_b, bits_b)) => {
                if signed_a == signed_b && bits_a == bits_b {
                    const_a.unify(const_b, span)
                } else {
                    Err(SpanKind::None)
                }
            }

            (Bool(const_a), Bool(const_b)) => const_a.unify(const_b, span),

            (other_a, other_b) => {
                if other_a == other_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }
        }
    }

    /// A feature of the language is that `Field` is like an
    /// `Any` type which allows you to pass in any type which
    /// is fundamentally a field element. E.g all integer types
    pub fn make_subtype_of(
        &self,
        expected: &Type,
        span: Span,
        errors: &mut Vec<TypeCheckError>,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        if let Err(err_span) = self.is_subtype_of(expected, span) {
            Self::issue_errors(expected, err_span, errors, make_error)
        }
    }

    fn is_subtype_of(&self, other: &Type, span: Span) -> Result<(), SpanKind> {
        use Type::*;
        match (self, other) {
            (Error, _) | (_, Error) => Ok(()),

            (Unspecified, _) | (_, Unspecified) => unreachable!(),

            (PolymorphicInteger(is_const, binding), other) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.is_subtype_of(other, span);
                }

                // Otherwise, check it is unified against an integer and bind it
                other.try_bind_to_polymorphic_int(binding, is_const, span)
            }
            // These needs to be a separate case to keep the argument order of is_subtype_of
            (other, PolymorphicInteger(is_const, binding)) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return other.is_subtype_of(link, span);
                }

                other.try_bind_to_polymorphic_int(binding, is_const, span)
            }

            (Array(len_a, elem_a), Array(len_b, elem_b)) => {
                if len_a.is_subtype_of(len_b) {
                    elem_a.is_subtype_of(elem_b, span)
                } else {
                    Err(SpanKind::None)
                }
            }

            (Tuple(elems_a), Tuple(elems_b)) => {
                if elems_a.len() != elems_b.len() {
                    Err(SpanKind::None)
                } else {
                    for (a, b) in elems_a.iter().zip(elems_b) {
                        a.is_subtype_of(b, span)?;
                    }
                    Ok(())
                }
            }

            // No recursive try_unify call needed for struct fields, we just
            // check the struct ids match.
            (Struct(struct_a), Struct(struct_b)) => {
                if struct_a == struct_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }

            (FieldElement(const_a), FieldElement(const_b)) => const_a.is_subtype_of(const_b, span),

            (Integer(const_a, signed_a, bits_a), Integer(const_b, signed_b, bits_b)) => {
                if signed_a == signed_b && bits_a == bits_b {
                    const_a.is_subtype_of(const_b, span)
                } else {
                    Err(SpanKind::None)
                }
            }

            (Bool(const_a), Bool(const_b)) => const_a.is_subtype_of(const_b, span),

            (other_a, other_b) => {
                if other_a == other_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }
        }
    }

    /// Computes the number of elements in a Type
    /// Arrays, structs, and tuples will be the only data structures to return more than one
    pub fn num_elements(&self) -> usize {
        match self {
            Type::Array(ArraySize::Fixed(fixed_size), _) => *fixed_size as usize,
            Type::Array(ArraySize::Variable, _) =>
                unreachable!("ice : this method is only ever called when we want to compare the prover inputs with the ABI in main. The ABI should not have variable input. The program should be compiled before calling this"),
            Type::Struct(s) => s.borrow().fields.len(),
            Type::Tuple(fields) => fields.len(),
            Type::FieldElement(..)
            | Type::Integer(..)
            | Type::PolymorphicInteger(..)
            | Type::Bool(_)
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
            Type::Array(sized, typ) => Some((sized, typ)),
            _ => None,
        }
    }

    // Note; use strict_eq instead of partial_eq when comparing field types
    // in this method, you most likely want to distinguish between public and private
    pub fn as_abi_type(&self, fe_type: AbiFEType, interner: &NodeInterner) -> AbiType {
        match self {
            Type::FieldElement(_) => AbiType::Field(fe_type),
            Type::Array(size, typ) => match size {
                ArraySize::Variable => {
                    panic!("cannot have variable sized array in entry point")
                }
                ArraySize::Fixed(length) => AbiType::Array {
                    visibility: fe_type,
                    length: *length,
                    typ: Box::new(typ.as_abi_type(fe_type, interner)),
                },
            },
            Type::Integer(_, sign, bit_width) => {
                let sign = match sign {
                    Signedness::Unsigned => noirc_abi::Sign::Unsigned,
                    Signedness::Signed => noirc_abi::Sign::Signed,
                };

                AbiType::Integer { sign, width: *bit_width as u32, visibility: fe_type }
            }
            Type::PolymorphicInteger(_, binding) => match &*binding.borrow() {
                TypeBinding::Bound(typ) => typ.as_abi_type(fe_type, interner),
                TypeBinding::Unbound(_) => {
                    Type::default_int_type(None).as_abi_type(fe_type, interner)
                }
            },
            Type::Bool(_) => panic!("currently, cannot have a bool in the entry point function"),
            Type::Error => unreachable!(),
            Type::Unspecified => unreachable!(),
            Type::Unit => unreachable!(),
            Type::Struct(_) => todo!("as_abi_type not yet implemented for struct types"),
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
            Type::Struct(def) => {
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

    /// Retrieves the type of the given field name
    /// Panics if the type is not a struct or tuple.
    pub fn get_field_type(&self, field_name: &str) -> Type {
        match self {
            Type::Struct(def) => def.borrow().get_field(field_name).unwrap().clone(),
            Type::Tuple(fields) => {
                let mut fields = fields.iter().enumerate();
                fields.find(|(i, _)| i.to_string() == *field_name).unwrap().1.clone()
            }
            other => panic!("Tried to iterate over the fields of '{}', which has none", other),
        }
    }
}
