use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};

use crate::{hir::type_check::TypeCheckError, node_interner::NodeInterner};
use noirc_abi::{AbiFEType, AbiType};
use noirc_errors::Span;

use crate::{
    node_interner::{FuncId, StructId},
    util::vecmap,
    Ident, Signedness,
};

pub type TypeBindings = HashMap<TypeVariableId, Type>;

#[derive(Debug, Eq)]
pub struct StructType {
    pub id: StructId,
    pub name: Ident,
    /// Fields are ordered
    pub fields: BTreeMap<Ident, Type>,
    pub generics: Vec<TypeVariableId>,
    pub methods: HashMap<String, FuncId>,
    pub span: Span,
}

impl PartialEq for StructType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl StructType {
    pub fn new(
        id: StructId,
        name: Ident,
        span: Span,
        fields: Vec<(Ident, Type)>,
        generics: Vec<TypeVariableId>,
    ) -> StructType {
        let fields = fields.into_iter().collect();
        StructType { id, fields, name, span, generics, methods: HashMap::new() }
    }

    pub fn get_field(&self, field_name: &str, generic_args: &[Type]) -> Option<Type> {
        self.fields.iter().find(|(name, _)| name.0.contents == field_name).map(|(_, typ)| {
            assert_eq!(self.generics.len(), generic_args.len());
            let substitutions = self
                .generics
                .iter()
                .zip(generic_args)
                .map(|(old, new)| (*old, new.clone()))
                .collect();

            typ.substitute(&substitutions)
        })
    }

    /// Instantiate this struct type, returning a Vec of the new generic args (in
    /// the same order as self.generics) and a map of each instantiated field
    pub fn instantiate<'a>(
        &'a self,
        interner: &mut NodeInterner,
    ) -> (Vec<Type>, BTreeMap<&'a str, Type>) {
        let (generics, substitutions) = self
            .generics
            .iter()
            .map(|old| {
                let new = interner.next_type_variable();
                (new.clone(), (*old, new))
            })
            .unzip();

        let fields = self
            .fields
            .iter()
            .map(|(name, typ)| {
                let typ = typ.substitute(&substitutions);
                (name.0.contents.as_str(), typ)
            })
            .collect();

        (generics, fields)
    }
}

impl std::fmt::Display for StructType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    FieldElement(IsConst),
    Array(Box<Type>, Box<Type>),       // Array(4, Field) = [Field; 4]
    Integer(IsConst, Signedness, u32), // u32 = Integer(unsigned, 32)
    PolymorphicInteger(IsConst, TypeVariable),
    Bool(IsConst),
    Unit,
    Struct(Rc<RefCell<StructType>>, Vec<Type>),
    Tuple(Vec<Type>),
    TypeVariable(TypeVariable),

    /// A functions with arguments, a return type, and
    /// a set of possible function ids it may refer to.
    /// The function id set is hidden from users and only
    /// used to monomorphise away most higher order functions.
    Function(Vec<Type>, Box<Type>, BTreeSet<FuncId>),

    /// A type generic over the given type variables
    Forall(Vec<TypeVariableId>, Box<Type>),

    /// A type-level integer. Included to let an Array's size type variable
    /// bind to an integer without special checks to bind it to a non-type.
    ArrayLength(u64),

    Error,
}

pub type TypeVariable = Rc<RefCell<TypeBinding>>;

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

    pub fn type_variable(id: TypeVariableId) -> Type {
        Type::TypeVariable(Rc::new(RefCell::new(TypeBinding::Unbound(id))))
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::FieldElement(is_const) => {
                write!(f, "{}Field", is_const)
            }
            Type::Array(len, typ) => match len.array_length() {
                Some(len) => write!(f, "[{}; {}]", typ, len),
                None => write!(f, "[{}]", typ),
            },
            Type::Integer(is_const, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{}i{}", is_const, num_bits),
                Signedness::Unsigned => write!(f, "{}u{}", is_const, num_bits),
            },
            Type::PolymorphicInteger(_, binding) => write!(f, "{}", binding.borrow()),
            Type::Struct(s, args) => {
                let args = vecmap(args, |arg| arg.to_string());
                if args.is_empty() {
                    write!(f, "{}", s.borrow())
                } else {
                    write!(f, "{}<{}>", s.borrow(), args.join(", "))
                }
            }
            Type::Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Type::Bool(is_const) => write!(f, "{}bool", is_const),
            Type::Unit => write!(f, "()"),
            Type::Error => write!(f, "error"),
            Type::TypeVariable(id) => write!(f, "{}", id.borrow()),
            Type::ArrayLength(n) => n.fmt(f),
            Type::Forall(typevars, typ) => {
                let typevars = vecmap(typevars, ToString::to_string);
                write!(f, "forall {}. {}", typevars.join(" "), typ)
            }
            Type::Function(args, ret, _) => {
                let args = vecmap(args, ToString::to_string);
                write!(f, "fn({}) -> {}", args.join(", "), ret)
            }
        }
    }
}

impl std::fmt::Display for TypeVariableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_")
    }
}

impl std::fmt::Display for TypeBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeBinding::Bound(typ) => typ.fmt(f),
            TypeBinding::Unbound(id) => id.fmt(f),
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
            Type::TypeVariable(binding) => {
                let borrow = binding.borrow();
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
                        Ok(())
                    }
                }
            }
            _ => Err(SpanKind::None),
        }
    }

    pub fn try_bind_to(&self, var: &TypeVariable) -> Result<(), SpanKind> {
        let target_id = match &*var.borrow() {
            TypeBinding::Bound(_) => unreachable!(),
            TypeBinding::Unbound(id) => *id,
        };

        if let Type::TypeVariable(binding) = self {
            match &*binding.borrow() {
                TypeBinding::Bound(typ) => return typ.try_bind_to(var),
                // Don't recursively bind the same id to itself
                TypeBinding::Unbound(id) if *id == target_id => return Ok(()),
                _ => (),
            }
        }

        // Check if the target id occurs within self before binding. Otherwise this could
        // cause infinitely recursive types
        if self.occurs(target_id) {
            Err(SpanKind::None)
        } else {
            *var.borrow_mut() = TypeBinding::Bound(self.clone());
            Ok(())
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

            (PolymorphicInteger(is_const, binding), other)
            | (other, PolymorphicInteger(is_const, binding)) => {
                // If it is already bound, unify against what it is bound to
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.try_unify(other, span);
                }

                // Otherwise, check it is unified against an integer and bind it
                other.try_bind_to_polymorphic_int(binding, is_const, span)
            }

            (TypeVariable(binding), other) | (other, TypeVariable(binding)) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.try_unify(other, span);
                }

                Ok(())
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
            (Struct(fields_a, args_a), Struct(fields_b, args_b)) => {
                if fields_a == fields_b {
                    for (a, b) in args_a.iter().zip(args_b) {
                        a.try_unify(b, span)?;
                    }
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

    /// The `subtype` term here is somewhat loose, the only subtyping relations remaining are
    /// between fixed and variable sized arrays, and IsConst tracking.
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

            (TypeVariable(binding), other) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.is_subtype_of(other, span);
                }

                other.try_bind_to(binding)
            }
            (other, TypeVariable(binding)) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return other.is_subtype_of(link, span);
                }

                other.try_bind_to(binding)
            }

            (Array(len_a, elem_a), Array(len_b, elem_b)) => {
                len_a.is_subtype_of(len_b, span)?;
                elem_a.is_subtype_of(elem_b, span)
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
            (Struct(struct_a, args_a), Struct(struct_b, args_b)) => {
                if struct_a == struct_b && args_a.len() == args_b.len() {
                    for (a, b) in args_a.iter().zip(args_b) {
                        a.is_subtype_of(b, span)?;
                    }
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

    pub fn array_length(&self) -> Option<u64> {
        match self {
            Type::PolymorphicInteger(_, binding) | Type::TypeVariable(binding) => {
                match &*binding.borrow() {
                    TypeBinding::Bound(binding) => binding.array_length(),
                    TypeBinding::Unbound(_) => None,
                }
            }
            Type::ArrayLength(size) => Some(*size),
            _ => None,
        }
    }

    // Note; use strict_eq instead of partial_eq when comparing field types
    // in this method, you most likely want to distinguish between public and private
    pub fn as_abi_type(&self, fe_type: AbiFEType) -> AbiType {
        match self {
            Type::FieldElement(_) => AbiType::Field(fe_type),
            Type::Array(size, typ) => {
                let size = size
                    .array_length()
                    .expect("Cannot have variable sized arrays as a parameter to main");
                AbiType::Array {
                    visibility: fe_type,
                    length: size as u128,
                    typ: Box::new(typ.as_abi_type(fe_type)),
                }
            }
            Type::Integer(_, sign, bit_width) => {
                let sign = match sign {
                    Signedness::Unsigned => noirc_abi::Sign::Unsigned,
                    Signedness::Signed => noirc_abi::Sign::Signed,
                };

                AbiType::Integer { sign, width: *bit_width as u32, visibility: fe_type }
            }
            Type::PolymorphicInteger(_, binding) => match &*binding.borrow() {
                TypeBinding::Bound(typ) => typ.as_abi_type(fe_type),
                TypeBinding::Unbound(_) => Type::default_int_type(None).as_abi_type(fe_type),
            },
            Type::Bool(_) => panic!("currently, cannot have a bool in the entry point function"),
            Type::Error => unreachable!(),
            Type::Unit => unreachable!(),
            Type::ArrayLength(_) => unreachable!(),
            Type::Struct(..) => todo!("as_abi_type not yet implemented for struct types"),
            Type::Tuple(_) => todo!("as_abi_type not yet implemented for tuple types"),
            Type::TypeVariable(_) => unreachable!(),
            Type::Forall(..) => unreachable!(),
            Type::Function(_, _, _) => unreachable!(),
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
            Type::Struct(def, args) => vecmap(&def.borrow().fields, |(name, _)| {
                let name = &name.0.contents;
                let typ = def.borrow().get_field(name, args).unwrap();
                (name.clone(), typ)
            }),
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
            Type::Struct(def, args) => def.borrow().get_field(field_name, args).unwrap(),
            Type::Tuple(fields) => {
                let mut fields = fields.iter().enumerate();
                fields.find(|(i, _)| i.to_string() == *field_name).unwrap().1.clone()
            }
            other => panic!("Tried to iterate over the fields of '{}', which has none", other),
        }
    }

    /// Instantiate this type, replacing any type variables it is quantified
    /// over with fresh type variables. If this type is not a Type::Forall,
    /// it is unchanged.
    pub fn instantiate(&self, interner: &mut NodeInterner) -> Type {
        match self {
            Type::Forall(typevars, typ) => {
                let replacements =
                    typevars.iter().map(|old| (*old, interner.next_type_variable())).collect();

                typ.substitute(&replacements)
            }
            other => other.clone(),
        }
    }

    /// Substitute any type variables found within this type with the
    /// given bindings if found. If a type variable is not found within
    /// the given TypeBindings, it is unchanged.
    pub fn substitute(&self, type_bindings: &TypeBindings) -> Type {
        if type_bindings.is_empty() {
            return self.clone();
        }

        let substitute_binding = |binding: &TypeVariable| match &*binding.borrow() {
            TypeBinding::Bound(binding) => binding.substitute(type_bindings),
            TypeBinding::Unbound(id) => {
                type_bindings.get(id).cloned().unwrap_or_else(|| self.clone())
            }
        };

        match self {
            Type::Array(size, element) => {
                let size = Box::new(size.substitute(type_bindings));
                let element = Box::new(element.substitute(type_bindings));
                Type::Array(size, element)
            }
            Type::PolymorphicInteger(_, binding) => substitute_binding(binding),

            // Do not substitute fields, it can lead to infinite recursion
            // and we should not match fields when type checking anyway.
            Type::Struct(fields, args) => {
                let args = vecmap(args, |arg| arg.substitute(type_bindings));
                Type::Struct(fields.clone(), args)
            }
            Type::Tuple(fields) => {
                let fields = vecmap(fields, |field| field.substitute(type_bindings));
                Type::Tuple(fields)
            }
            Type::TypeVariable(binding) => substitute_binding(binding),
            Type::Forall(typevars, typ) => {
                // Trying to substitute a variable defined within a nested Forall
                // is usually impossible and indicative of an error in the type checker somewhere.
                for var in typevars {
                    assert!(!type_bindings.contains_key(var));
                }
                let typ = Box::new(typ.substitute(type_bindings));
                Type::Forall(typevars.clone(), typ)
            }
            Type::Function(args, ret, ids) => {
                let args = vecmap(args, |arg| arg.substitute(type_bindings));
                let ret = Box::new(ret.substitute(type_bindings));
                Type::Function(args, ret, ids.clone())
            }

            Type::FieldElement(_)
            | Type::Integer(_, _, _)
            | Type::Bool(_)
            | Type::ArrayLength(_)
            | Type::Error
            | Type::Unit => self.clone(),
        }
    }

    /// True if the given TypeVariableId is free anywhere
    /// within self
    fn occurs(&self, target_id: TypeVariableId) -> bool {
        match self {
            Type::Array(_, elem) => elem.occurs(target_id),
            Type::PolymorphicInteger(_, binding) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.occurs(target_id),
                TypeBinding::Unbound(id) => *id == target_id,
            },
            Type::Struct(_, generic_args) => generic_args.iter().any(|arg| arg.occurs(target_id)),
            Type::Tuple(fields) => fields.iter().any(|field| field.occurs(target_id)),
            Type::TypeVariable(binding) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.occurs(target_id),
                TypeBinding::Unbound(id) => *id == target_id,
            },
            Type::Forall(typevars, typ) => {
                !typevars.iter().any(|var| *var == target_id) && typ.occurs(target_id)
            }
            Type::Function(args, ret, _) => {
                args.iter().any(|arg| arg.occurs(target_id)) || ret.occurs(target_id)
            }

            Type::FieldElement(_)
            | Type::Integer(_, _, _)
            | Type::Bool(_)
            | Type::ArrayLength(_)
            | Type::Error
            | Type::Unit => false,
        }
    }
}
