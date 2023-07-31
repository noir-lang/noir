use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use crate::{
    hir::type_check::TypeCheckError,
    node_interner::{ExprId, NodeInterner},
};
use iter_extended::vecmap;
use noirc_abi::AbiType;
use noirc_errors::Span;

use crate::{node_interner::StructId, Ident, Signedness};

use super::expr::{HirCallExpression, HirExpression, HirIdent};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Type {
    /// A primitive Field type, and whether or not it is known at compile-time.
    FieldElement(CompTime),

    /// Array(N, E) is an array of N elements of type E. It is expected that N
    /// is either a type variable of some kind or a Type::Constant.
    Array(Box<Type>, Box<Type>),

    /// A primitive integer type with the given sign, bit count, and whether it is known at compile-time.
    /// E.g. `u32` would be `Integer(CompTime::No(None), Unsigned, 32)`
    Integer(CompTime, Signedness, u32),

    /// The primitive `bool` type. Like other primitive types, whether booleans are known at CompTime
    /// is also tracked. Unlike the other primitives however, it isn't as useful since it is
    /// primarily only used when converting between a bool and an integer type for indexing arrays.
    Bool(CompTime),

    /// String(N) is an array of characters of length N. It is expected that N
    /// is either a type variable of some kind or a Type::Constant.
    String(Box<Type>),

    /// The unit type `()`.
    Unit,

    /// A user-defined struct type. The `Shared<StructType>` field here refers to
    /// the shared definition for each instance of this struct type. The `Vec<Type>`
    /// represents the generic arguments (if any) to this struct type.
    Struct(Shared<StructType>, Vec<Type>),

    /// A tuple type with the given list of fields in the order they appear in source code.
    Tuple(Vec<Type>),

    /// TypeVariables are stand-in variables for some type which is not yet known.
    /// They are not to be confused with NamedGenerics. While the later mostly works
    /// as with normal types (ie. for two NamedGenerics T and U, T != U), TypeVariables
    /// will be automatically rebound as necessary to satisfy any calls to unify
    /// and make_subtype_of.
    ///
    /// TypeVariables are often created when a generic function is instantiated. This
    /// is a process that replaces each NamedGeneric in a generic function with a TypeVariable.
    /// Doing this at each call site of a generic function is how they can be called with
    /// different argument types each time.
    TypeVariable(TypeVariable, TypeVariableKind),

    /// NamedGenerics are the 'T' or 'U' in a user-defined generic function
    /// like `fn foo<T, U>(...) {}`. Unlike TypeVariables, they cannot be bound over.
    NamedGeneric(TypeVariable, Rc<String>),

    /// A functions with arguments, and a return type.
    Function(Vec<Type>, Box<Type>),

    /// &mut T
    MutableReference(Box<Type>),

    /// A type generic over the given type variables.
    /// Storing both the TypeVariableId and TypeVariable isn't necessary
    /// but it makes handling them both easier. The TypeVariableId should
    /// never be bound over during type checking, but during monomorphization it
    /// will be and thus needs the full TypeVariable link.
    Forall(Generics, Box<Type>),

    /// A type-level integer. Included to let an Array's size type variable
    /// bind to an integer without special checks to bind it to a non-type.
    Constant(u64),

    /// The type of a slice is an array of size NotConstant.
    /// The size of an array literal is resolved to this if it ever uses operations
    /// involving slices.
    NotConstant,

    /// The result of some type error. Remembering type errors as their own type variant lets
    /// us avoid issuing repeat type errors for the same item. For example, a lambda with
    /// an invalid type would otherwise issue a new error each time it is called
    /// if not for this variant.
    Error,
}

/// A list of TypeVariableIds to bind to a type. Storing the
/// TypeVariable in addition to the matching TypeVariableId allows
/// the binding to later be undone if needed.
pub type TypeBindings = HashMap<TypeVariableId, (TypeVariable, Type)>;

/// Represents a struct type in the type system. Each instance of this
/// rust struct will be shared across all Type::Struct variants that represent
/// the same struct type.
#[derive(Debug, Eq)]
pub struct StructType {
    /// A unique id representing this struct type. Used to check if two
    /// struct types are equal.
    pub id: StructId,

    pub name: Ident,

    /// Fields are ordered and private, they should only
    /// be accessed through get_field(), get_fields(), or instantiate()
    /// since these will handle applying generic arguments to fields as well.
    fields: Vec<(Ident, Type)>,

    pub generics: Generics,
    pub span: Span,
}

/// Corresponds to generic lists such as `<T, U>` in the source
/// program. The `TypeVariableId` portion is used to match two
/// type variables to check for equality, while the `TypeVariable` is
/// the actual part that can be mutated to bind it to another type.
pub type Generics = Vec<(TypeVariableId, TypeVariable)>;

impl std::hash::Hash for StructType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
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
        generics: Generics,
    ) -> StructType {
        StructType { id, fields, name, span, generics }
    }

    /// To account for cyclic references between structs, a struct's
    /// fields are resolved strictly after the struct itself is initially
    /// created. Therefore, this method is used to set the fields once they
    /// become known.
    pub fn set_fields(&mut self, fields: Vec<(Ident, Type)>) {
        assert!(self.fields.is_empty());
        self.fields = fields;
    }

    pub fn num_fields(&self) -> usize {
        self.fields.len()
    }

    /// Returns the field matching the given field name, as well as its field index.
    pub fn get_field(&self, field_name: &str, generic_args: &[Type]) -> Option<(Type, usize)> {
        assert_eq!(self.generics.len(), generic_args.len());

        self.fields.iter().enumerate().find(|(_, (name, _))| name.0.contents == field_name).map(
            |(i, (_, typ))| {
                let substitutions = self
                    .generics
                    .iter()
                    .zip(generic_args)
                    .map(|((old_id, old_var), new)| (*old_id, (old_var.clone(), new.clone())))
                    .collect();

                (typ.substitute(&substitutions), i)
            },
        )
    }

    /// Returns all the fields of this type, after being applied to the given generic arguments.
    pub fn get_fields(&self, generic_args: &[Type]) -> Vec<(String, Type)> {
        assert_eq!(self.generics.len(), generic_args.len());

        let substitutions = self
            .generics
            .iter()
            .zip(generic_args)
            .map(|((old_id, old_var), new)| (*old_id, (old_var.clone(), new.clone())))
            .collect();

        vecmap(&self.fields, |(name, typ)| {
            let name = name.0.contents.clone();
            (name, typ.substitute(&substitutions))
        })
    }

    pub fn field_names(&self) -> BTreeSet<Ident> {
        self.fields.iter().map(|(name, _)| name.clone()).collect()
    }

    /// True if the given index is the same index as a generic type of this struct
    /// which is expected to be a numeric generic.
    /// This is needed because we infer type kinds in Noir and don't have extensive kind checking.
    pub fn generic_is_numeric(&self, index_of_generic: usize) -> bool {
        let target_id = self.generics[index_of_generic].0;
        self.fields.iter().any(|(_, field)| field.contains_numeric_typevar(target_id))
    }

    /// Instantiate this struct type, returning a Vec of the new generic args (in
    /// the same order as self.generics)
    pub fn instantiate(&self, interner: &mut NodeInterner) -> Vec<Type> {
        vecmap(&self.generics, |_| interner.next_type_variable())
    }
}

impl std::fmt::Display for StructType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A shared, mutable reference to some T.
/// Wrapper is required for Hash impl of RefCell.
#[derive(Debug, Eq, PartialOrd, Ord)]
pub struct Shared<T>(Rc<RefCell<T>>);

impl<T: std::hash::Hash> std::hash::Hash for Shared<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}

impl<T: PartialEq> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        let ref1 = self.0.borrow();
        let ref2 = other.0.borrow();
        *ref1 == *ref2
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(self.0.clone())
    }
}

impl<T> From<T> for Shared<T> {
    fn from(thing: T) -> Shared<T> {
        Shared::new(thing)
    }
}

impl<T> Shared<T> {
    pub fn new(thing: T) -> Shared<T> {
        Shared(Rc::new(RefCell::new(thing)))
    }

    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
        self.0.borrow_mut()
    }
}

/// A restricted subset of binary operators useable on
/// type level integers for use in the array length positions of types.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum BinaryTypeOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulo,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum TypeVariableKind {
    /// Can bind to any type
    Normal,

    /// A generic integer or field type. This is a more specific kind of TypeVariable
    /// that can only be bound to Type::Field, Type::Integer, or other polymorphic integers.
    /// This is the type of undecorated integer literals like `46`. Typing them in this way
    /// allows them to be polymorphic over the actual integer/field type used without requiring
    /// type annotations on each integer literal.
    IntegerOrField(CompTime),

    /// A potentially constant array size. This will only bind to itself, Type::NotConstant, or
    /// Type::Constant(n) with a matching size. This defaults to Type::Constant(n) if still unbound
    /// during monomorphization.
    Constant(u64),
}

/// A TypeVariable is a mutable reference that is either
/// bound to some type, or unbound with a given TypeVariableId.
pub type TypeVariable = Shared<TypeBinding>;

/// TypeBindings are the mutable insides of a TypeVariable.
/// They are either bound to some type, or are unbound.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeBinding {
    Bound(Type),
    Unbound(TypeVariableId),
}

impl TypeBinding {
    pub fn is_unbound(&self) -> bool {
        matches!(self, TypeBinding::Unbound(_))
    }

    pub fn bind_to(&mut self, binding: Type, span: Span) -> Result<(), TypeCheckError> {
        match self {
            TypeBinding::Bound(_) => panic!("Tried to bind an already bound type variable!"),
            TypeBinding::Unbound(id) => {
                if binding.occurs(*id) {
                    Err(TypeCheckError::TypeAnnotationsNeeded { span })
                } else {
                    *self = TypeBinding::Bound(binding);
                    Ok(())
                }
            }
        }
    }
}

/// A unique ID used to differentiate different type variables
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeVariableId(pub usize);

/// Noir's type system keeps track of whether or not every primitive type's value
/// is known at compile-time. This is exposed through users through the `comptime`
/// keyword in noir which can be prefixed before primitive types. A usage like
/// `t: comptime Field` would correspond to a Field type with a CompTime::Yes(None)
/// variant of this enum
///
/// Note that whether or not a variable is comptime can also be inferred based on its use.
/// A value passed to a function that expects a `comptime Field` must be CompTime::Yes,
/// likewise a parameter of the current function that is just a `Field` can only be CompTime::No.
/// There is also the case of integer literals which are typed as CompTime::Maybe. These are
/// polymorphically comptime because they can be used in both contexts.
#[derive(Debug, Clone, Eq)]
pub enum CompTime {
    // Yes and No variants have optional spans representing the location in the source code
    // which caused them to be compile time.
    Yes(Option<Span>),
    No(Option<Span>),

    /// Maybe has an id and shared inner reference that can be rebound later to
    /// another specific CompTime variant.
    Maybe(TypeVariableId, Shared<Option<CompTime>>),
}

impl std::hash::Hash for CompTime {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);

        if let CompTime::Maybe(id, binding) = self {
            if let Some(is_comp_time) = &*binding.borrow() {
                is_comp_time.hash(state);
            } else {
                id.hash(state);
            }
        }
    }
}

impl PartialEq for CompTime {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CompTime::Maybe(id1, binding1), CompTime::Maybe(id2, binding2)) => {
                if let Some(new_self) = &*binding1.borrow() {
                    return new_self == other;
                }
                if let Some(new_other) = &*binding2.borrow() {
                    return self == new_other;
                }
                id1 == id2
            }
            (CompTime::Yes(_), CompTime::Yes(_)) | (CompTime::No(_), CompTime::No(_)) => true,
            _ => false,
        }
    }
}

/// Internal enum for `unify` to remember the type context of each span
/// to provide better error messages
#[derive(Debug)]
pub enum SpanKind {
    CompTime(Span),
    NotCompTime(Span),
    None,
}

impl CompTime {
    pub fn new(interner: &mut NodeInterner) -> Self {
        let id = interner.next_type_variable_id();
        Self::Maybe(id, Shared::new(None))
    }

    /// Set the Span on this CompTime (if it has one) to keep track of
    /// when it was last changed to give better error messages.
    fn set_span(&mut self, new_span: Span) {
        match self {
            CompTime::Yes(span) | CompTime::No(span) => *span = Some(new_span),
            CompTime::Maybe(_, binding) => {
                if let Some(binding) = &mut *binding.borrow_mut() {
                    binding.set_span(new_span);
                }
            }
        }
    }

    /// Try to unify these two CompTime constraints.
    pub fn unify(&self, other: &Self, span: Span) -> Result<(), SpanKind> {
        match (self, other) {
            (CompTime::Yes(_), CompTime::Yes(_)) | (CompTime::No(_), CompTime::No(_)) => Ok(()),

            (CompTime::Yes(y), CompTime::No(n)) | (CompTime::No(n), CompTime::Yes(y)) => {
                Err(match (y, n) {
                    (_, Some(span)) => SpanKind::NotCompTime(*span),
                    (Some(span), _) => SpanKind::CompTime(*span),
                    _ => SpanKind::None,
                })
            }

            (CompTime::Maybe(_, binding), other) | (other, CompTime::Maybe(_, binding))
                if binding.borrow().is_some() =>
            {
                let binding = &*binding.borrow();
                binding.as_ref().unwrap().unify(other, span)
            }

            (CompTime::Maybe(id1, _), CompTime::Maybe(id2, _)) if id1 == id2 => Ok(()),

            // Both are unbound and do not refer to each other, arbitrarily set one equal to the other
            (CompTime::Maybe(_, binding), other) | (other, CompTime::Maybe(_, binding)) => {
                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                Ok(())
            }
        }
    }

    /// Try to unify these two CompTime constraints.
    pub fn is_subtype_of(&self, other: &Self, span: Span) -> Result<(), SpanKind> {
        match (self, other) {
            (CompTime::Yes(_), CompTime::Yes(_))
            | (CompTime::No(_), CompTime::No(_))

            // This is one of the only 2 differing cases between this and CompTime::unify
            | (CompTime::Yes(_), CompTime::No(_)) => Ok(()),

            (CompTime::No(n), CompTime::Yes(y)) => {
                Err(match (y, n) {
                    (_, Some(span)) => SpanKind::NotCompTime(*span),
                    (Some(span), _) => SpanKind::CompTime(*span),
                    _ => SpanKind::None,
                })
            }

            (CompTime::Maybe(_, binding), other) if binding.borrow().is_some() => {
                let binding = &*binding.borrow();
                binding.as_ref().unwrap().is_subtype_of(other, span)
            }

            (other, CompTime::Maybe(_, binding)) if binding.borrow().is_some() => {
                let binding = &*binding.borrow();
                other.is_subtype_of(binding.as_ref().unwrap(), span)
            }

            (CompTime::Maybe(id1, _), CompTime::Maybe(id2, _)) if id1 == id2 => Ok(()),

            // This is the other differing case between this and CompTime::unify.
            // If this is polymorphically comptime, don't force it to be non-comptime because it is
            // passed as an argument to a function expecting a non-comptime parameter.
            (CompTime::Maybe(_, binding), CompTime::No(_)) if binding.borrow().is_none() => Ok(()),

            (CompTime::Maybe(_, binding), other) => {
                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                Ok(())
            }
            (other, CompTime::Maybe(_, binding)) => {
                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                Ok(())
            }
        }
    }

    /// Combine these two CompTimes together, returning
    /// - CompTime::Yes if both are Yes,
    /// - CompTime::No if either are No,
    /// - or if both are Maybe, unify them both and return the lhs.
    pub fn and(&self, other: &Self, span: Span) -> Self {
        match (self, other) {
            (CompTime::Yes(_), CompTime::Yes(_)) => CompTime::Yes(Some(span)),

            (CompTime::No(_), CompTime::No(_))
            | (CompTime::Yes(_), CompTime::No(_))
            | (CompTime::No(_), CompTime::Yes(_)) => CompTime::No(Some(span)),

            (CompTime::Maybe(_, binding), other) | (other, CompTime::Maybe(_, binding))
                if binding.borrow().is_some() =>
            {
                let binding = &*binding.borrow();
                binding.as_ref().unwrap().and(other, span)
            }

            (CompTime::Maybe(id1, _), CompTime::Maybe(id2, _)) if id1 == id2 => self.clone(),

            (CompTime::Maybe(_, binding), other) | (other, CompTime::Maybe(_, binding)) => {
                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                other.clone()
            }
        }
    }

    pub fn is_comp_time(&self) -> bool {
        match self {
            CompTime::Yes(_) => true,
            CompTime::No(_) => false,
            CompTime::Maybe(_, binding) => {
                if let Some(binding) = &*binding.borrow() {
                    return binding.is_comp_time();
                }
                true
            }
        }
    }
}

impl Type {
    pub fn field(span: Option<Span>) -> Type {
        Type::FieldElement(CompTime::No(span))
    }

    pub fn comp_time(span: Option<Span>) -> Type {
        Type::FieldElement(CompTime::Yes(span))
    }

    pub fn default_int_type(span: Option<Span>) -> Type {
        Type::field(span)
    }

    pub fn type_variable(id: TypeVariableId) -> Type {
        Type::TypeVariable(Shared::new(TypeBinding::Unbound(id)), TypeVariableKind::Normal)
    }

    /// Returns a TypeVariable(_, TypeVariableKind::Constant(length)) to bind to
    /// a constant integer for e.g. an array length.
    pub fn constant_variable(length: u64, interner: &mut NodeInterner) -> Type {
        let id = interner.next_type_variable_id();
        let kind = TypeVariableKind::Constant(length);
        Type::TypeVariable(Shared::new(TypeBinding::Unbound(id)), kind)
    }

    pub fn polymorphic_integer(interner: &mut NodeInterner) -> Type {
        let id = interner.next_type_variable_id();
        let kind = TypeVariableKind::IntegerOrField(CompTime::new(interner));
        Type::TypeVariable(Shared::new(TypeBinding::Unbound(id)), kind)
    }

    /// A bit of an awkward name for this function - this function returns
    /// true for type variables or polymorphic integers which are unbound.
    /// NamedGenerics will always be false as although they are bindable,
    /// they shouldn't be bound over until monomorphization.
    pub fn is_bindable(&self) -> bool {
        match self {
            Type::TypeVariable(binding, _) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.is_bindable(),
                TypeBinding::Unbound(_) => true,
            },
            _ => false,
        }
    }

    pub fn is_field(&self) -> bool {
        matches!(self.follow_bindings(), Type::FieldElement(_))
    }

    pub fn is_signed(&self) -> bool {
        matches!(self.follow_bindings(), Type::Integer(_, Signedness::Signed, _))
    }

    fn contains_numeric_typevar(&self, target_id: TypeVariableId) -> bool {
        // True if the given type is a NamedGeneric with the target_id
        let named_generic_id_matches_target = |typ: &Type| {
            if let Type::NamedGeneric(type_variable, _) = typ {
                match &*type_variable.borrow() {
                    TypeBinding::Bound(_) => {
                        unreachable!("Named generics should not be bound until monomorphization")
                    }
                    TypeBinding::Unbound(id) => target_id == *id,
                }
            } else {
                false
            }
        };

        match self {
            Type::FieldElement(_)
            | Type::Integer(_, _, _)
            | Type::Bool(_)
            | Type::String(_)
            | Type::Unit
            | Type::Error
            | Type::TypeVariable(_, _)
            | Type::Constant(_)
            | Type::NamedGeneric(_, _)
            | Type::NotConstant
            | Type::Forall(_, _) => false,

            Type::Array(length, elem) => {
                elem.contains_numeric_typevar(target_id) || named_generic_id_matches_target(length)
            }

            Type::Tuple(fields) => {
                fields.iter().any(|field| field.contains_numeric_typevar(target_id))
            }
            Type::Function(parameters, return_type) => {
                parameters.iter().any(|parameter| parameter.contains_numeric_typevar(target_id))
                    || return_type.contains_numeric_typevar(target_id)
            }
            Type::Struct(struct_type, generics) => {
                generics.iter().enumerate().any(|(i, generic)| {
                    if named_generic_id_matches_target(generic) {
                        struct_type.borrow().generic_is_numeric(i)
                    } else {
                        generic.contains_numeric_typevar(target_id)
                    }
                })
            }
            Type::MutableReference(element) => element.contains_numeric_typevar(target_id),
        }
    }

    pub(crate) fn try_get_comptime(&self) -> Cow<CompTime> {
        match self {
            Type::FieldElement(comptime)
            | Type::Integer(comptime, _, _)
            | Type::Bool(comptime)
            | Type::TypeVariable(_, TypeVariableKind::IntegerOrField(comptime)) => {
                Cow::Borrowed(comptime)
            }
            _ => Cow::Owned(CompTime::No(None)),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::FieldElement(comp_time) => {
                write!(f, "{comp_time}Field")
            }
            Type::Array(len, typ) => {
                if matches!(len.follow_bindings(), Type::NotConstant) {
                    write!(f, "[{typ}]")
                } else {
                    write!(f, "[{typ}; {len}]")
                }
            }
            Type::Integer(comp_time, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{comp_time}i{num_bits}"),
                Signedness::Unsigned => write!(f, "{comp_time}u{num_bits}"),
            },
            Type::TypeVariable(id, TypeVariableKind::Normal) => write!(f, "{}", id.borrow()),
            Type::TypeVariable(binding, TypeVariableKind::IntegerOrField(_)) => {
                if let TypeBinding::Unbound(_) = &*binding.borrow() {
                    // Show a Field by default if this TypeVariableKind::IntegerOrField is unbound, since that is
                    // what they bind to by default anyway. It is less confusing than displaying it
                    // as a generic.
                    write!(f, "Field")
                } else {
                    write!(f, "{}", binding.borrow())
                }
            }
            Type::TypeVariable(binding, TypeVariableKind::Constant(n)) => {
                if let TypeBinding::Unbound(_) = &*binding.borrow() {
                    // TypeVariableKind::Constant(n) binds to Type::Constant(n) by default, so just show that.
                    write!(f, "{n}")
                } else {
                    write!(f, "{}", binding.borrow())
                }
            }
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
            Type::Bool(comp_time) => write!(f, "{comp_time}bool"),
            Type::String(len) => write!(f, "str<{len}>"),
            Type::Unit => write!(f, "()"),
            Type::Error => write!(f, "error"),
            Type::NamedGeneric(binding, name) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.fmt(f),
                TypeBinding::Unbound(_) if name.is_empty() => write!(f, "_"),
                TypeBinding::Unbound(_) => write!(f, "{name}"),
            },
            Type::Constant(x) => x.fmt(f),
            Type::Forall(typevars, typ) => {
                let typevars = vecmap(typevars, |(var, _)| var.to_string());
                write!(f, "forall {}. {}", typevars.join(" "), typ)
            }
            Type::Function(args, ret) => {
                let args = vecmap(args, ToString::to_string);
                write!(f, "fn({}) -> {}", args.join(", "), ret)
            }
            Type::MutableReference(element) => {
                write!(f, "&mut {element}")
            }
            Type::NotConstant => write!(f, "_"),
        }
    }
}

impl std::fmt::Display for BinaryTypeOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryTypeOperator::Addition => write!(f, "+"),
            BinaryTypeOperator::Subtraction => write!(f, "-"),
            BinaryTypeOperator::Multiplication => write!(f, "*"),
            BinaryTypeOperator::Division => write!(f, "/"),
            BinaryTypeOperator::Modulo => write!(f, "%"),
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

impl std::fmt::Display for CompTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompTime::Yes(_) => write!(f, "comptime "),
            CompTime::No(_) => Ok(()),
            CompTime::Maybe(_, binding) => match &*binding.borrow() {
                Some(binding) => binding.fmt(f),
                None => write!(f, "comptime "),
            },
        }
    }
}

impl Type {
    /// Mutate the span for the `CompTime` enum to track where a type is required to be `comptime`
    /// for error messages that show both the erroring call site and the call site before
    /// which required the variable to be comptime or non-comptime.
    pub fn set_comp_time_span(&mut self, new_span: Span) {
        match self {
            Type::FieldElement(comptime) | Type::Integer(comptime, _, _) => {
                comptime.set_span(new_span);
            }
            Type::TypeVariable(binding, TypeVariableKind::IntegerOrField(span)) => {
                if let TypeBinding::Bound(binding) = &mut *binding.borrow_mut() {
                    return binding.set_comp_time_span(new_span);
                }
                span.set_span(new_span);
            }
            _ => (),
        }
    }

    pub fn set_comp_time(&mut self, new_comptime: CompTime) {
        match self {
            Type::FieldElement(comptime) | Type::Integer(comptime, _, _) => {
                *comptime = new_comptime;
            }
            Type::TypeVariable(binding, TypeVariableKind::IntegerOrField(comptime)) => {
                if let TypeBinding::Bound(binding) = &mut *binding.borrow_mut() {
                    return binding.set_comp_time(new_comptime);
                }
                *comptime = new_comptime;
            }
            _ => (),
        }
    }

    /// Try to bind a MaybeConstant variable to self, succeeding if self is a Constant,
    /// MaybeConstant, or type variable.
    pub fn try_bind_to_maybe_constant(
        &self,
        var: &TypeVariable,
        target_length: u64,
    ) -> Result<(), SpanKind> {
        let target_id = match &*var.borrow() {
            TypeBinding::Bound(_) => unreachable!(),
            TypeBinding::Unbound(id) => *id,
        };

        match self {
            Type::Constant(length) if *length == target_length => {
                *var.borrow_mut() = TypeBinding::Bound(self.clone());
                Ok(())
            }
            Type::NotConstant => {
                *var.borrow_mut() = TypeBinding::Bound(Type::NotConstant);
                Ok(())
            }
            Type::TypeVariable(binding, kind) => {
                let borrow = binding.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => typ.try_bind_to_maybe_constant(var, target_length),
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(_) => match kind {
                        TypeVariableKind::Normal => {
                            drop(borrow);
                            let clone = Type::TypeVariable(
                                var.clone(),
                                TypeVariableKind::Constant(target_length),
                            );
                            *binding.borrow_mut() = TypeBinding::Bound(clone);
                            Ok(())
                        }
                        TypeVariableKind::Constant(length) if *length == target_length => {
                            drop(borrow);
                            let clone = Type::TypeVariable(
                                var.clone(),
                                TypeVariableKind::Constant(target_length),
                            );
                            *binding.borrow_mut() = TypeBinding::Bound(clone);
                            Ok(())
                        }
                        TypeVariableKind::Constant(_) | TypeVariableKind::IntegerOrField(_) => {
                            Err(SpanKind::None)
                        }
                    },
                }
            }
            _ => Err(SpanKind::None),
        }
    }

    /// Try to bind a PolymorphicInt variable to self, succeeding if self is an integer, field,
    /// other PolymorphicInt type, or type variable. If use_subtype is true, the CompTime fields
    /// of each will be checked via sub-typing rather than unification.
    pub fn try_bind_to_polymorphic_int(
        &self,
        var: &TypeVariable,
        var_comp_time: &CompTime,
        use_subtype: bool,
        span: Span,
    ) -> Result<(), SpanKind> {
        let target_id = match &*var.borrow() {
            TypeBinding::Bound(_) => unreachable!(),
            TypeBinding::Unbound(id) => *id,
        };

        let bind = |int_comp_time: &CompTime| {
            let mut clone = self.clone();
            let mut new_comp_time = var_comp_time.clone();
            new_comp_time.set_span(span);
            clone.set_comp_time(new_comp_time);

            *var.borrow_mut() = TypeBinding::Bound(clone);

            if use_subtype {
                var_comp_time.is_subtype_of(int_comp_time, span)
            } else {
                var_comp_time.unify(int_comp_time, span)
            }
        };

        match self {
            Type::FieldElement(int_comp_time, ..) | Type::Integer(int_comp_time, ..) => {
                bind(int_comp_time)
            }
            Type::TypeVariable(self_var, TypeVariableKind::IntegerOrField(int_comp_time)) => {
                let borrow = self_var.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => {
                        typ.try_bind_to_polymorphic_int(var, var_comp_time, use_subtype, span)
                    }
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(_) => {
                        drop(borrow);
                        bind(int_comp_time)
                    }
                }
            }
            Type::TypeVariable(binding, TypeVariableKind::Normal) => {
                let borrow = binding.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => {
                        typ.try_bind_to_polymorphic_int(var, var_comp_time, use_subtype, span)
                    }
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(_) => {
                        drop(borrow);
                        // PolymorphicInt is more specific than TypeVariable so we bind the type
                        // variable to PolymorphicInt instead.
                        let mut clone = Type::TypeVariable(
                            var.clone(),
                            TypeVariableKind::IntegerOrField(var_comp_time.clone()),
                        );
                        clone.set_comp_time_span(span);
                        *binding.borrow_mut() = TypeBinding::Bound(clone);
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

        if let Some(binding) = self.get_inner_type_variable() {
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

    fn get_inner_type_variable(&self) -> Option<Shared<TypeBinding>> {
        match self {
            Type::TypeVariable(var, _) | Type::NamedGeneric(var, _) => Some(var.clone()),
            _ => None,
        }
    }

    fn is_comp_time(&self) -> bool {
        match self {
            Type::FieldElement(comptime) => comptime.is_comp_time(),
            Type::Integer(comptime, ..) => comptime.is_comp_time(),
            Type::TypeVariable(binding, TypeVariableKind::IntegerOrField(comptime)) => {
                if let TypeBinding::Bound(binding) = &*binding.borrow() {
                    return binding.is_comp_time();
                }
                comptime.is_comp_time()
            }
            _ => false,
        }
    }

    /// Try to unify this type with another, setting any type variables found
    /// equal to the other type in the process. Unification is more strict
    /// than sub-typing but less strict than Eq. Returns true if the unification
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
            Self::issue_errors(expected, err_span, errors, make_error);
        }
    }

    fn issue_errors(
        expected: &Type,
        err_span: SpanKind,
        errors: &mut Vec<TypeCheckError>,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        errors.push(make_error());

        match (expected.is_comp_time(), err_span) {
            (true, SpanKind::NotCompTime(span)) => {
                errors.push(TypeCheckError::NotCompTime { span });
            }
            (false, SpanKind::CompTime(span)) => {
                errors.push(TypeCheckError::CompTime { span });
            }
            _ => (),
        }
    }

    /// `try_unify` is a bit of a misnomer since although errors are not committed,
    /// any unified bindings are on success.
    fn try_unify(&self, other: &Type, span: Span) -> Result<(), SpanKind> {
        use Type::*;
        use TypeVariableKind as Kind;

        match (self, other) {
            (Error, _) | (_, Error) => Ok(()),

            (TypeVariable(binding, Kind::IntegerOrField(comptime)), other)
            | (other, TypeVariable(binding, Kind::IntegerOrField(comptime))) => {
                // If it is already bound, unify against what it is bound to
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.try_unify(other, span);
                }

                // Otherwise, check it is unified against an integer and bind it
                other.try_bind_to_polymorphic_int(binding, comptime, false, span)
            }

            (TypeVariable(binding, Kind::Normal), other)
            | (other, TypeVariable(binding, Kind::Normal)) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.try_unify(other, span);
                }

                other.try_bind_to(binding)
            }

            (TypeVariable(binding, Kind::Constant(length)), other)
            | (other, TypeVariable(binding, Kind::Constant(length))) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.try_unify(other, span);
                }

                other.try_bind_to_maybe_constant(binding, *length)
            }

            (Array(len_a, elem_a), Array(len_b, elem_b)) => {
                len_a.try_unify(len_b, span)?;
                elem_a.try_unify(elem_b, span)
            }

            (Tuple(elements_a), Tuple(elements_b)) => {
                if elements_a.len() != elements_b.len() {
                    Err(SpanKind::None)
                } else {
                    for (a, b) in elements_a.iter().zip(elements_b) {
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

            (FieldElement(comptime_a), FieldElement(comptime_b)) => {
                comptime_a.unify(comptime_b, span)
            }

            (Integer(comptime_a, signed_a, bits_a), Integer(comptime_b, signed_b, bits_b)) => {
                if signed_a == signed_b && bits_a == bits_b {
                    comptime_a.unify(comptime_b, span)
                } else {
                    Err(SpanKind::None)
                }
            }

            (Bool(comptime_a), Bool(comptime_b)) => comptime_a.unify(comptime_b, span),

            (NamedGeneric(binding_a, name_a), NamedGeneric(binding_b, name_b)) => {
                // Ensure NamedGenerics are never bound during type checking
                assert!(binding_a.borrow().is_unbound());
                assert!(binding_b.borrow().is_unbound());

                if name_a == name_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }

            (Function(params_a, ret_a), Function(params_b, ret_b)) => {
                if params_a.len() == params_b.len() {
                    for (a, b) in params_a.iter().zip(params_b) {
                        a.try_unify(b, span)?;
                    }

                    ret_b.try_unify(ret_a, span)
                } else {
                    Err(SpanKind::None)
                }
            }

            (MutableReference(elem_a), MutableReference(elem_b)) => elem_a.try_unify(elem_b, span),

            (other_a, other_b) => {
                if other_a == other_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }
        }
    }

    /// The `subtype` term here is somewhat loose, the only sub-typing relations remaining
    /// have to do with CompTime tracking.
    pub fn make_subtype_of(
        &self,
        expected: &Type,
        span: Span,
        errors: &mut Vec<TypeCheckError>,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        if let Err(err_span) = self.is_subtype_of(expected, span) {
            Self::issue_errors(expected, err_span, errors, make_error);
        }
    }

    /// Similar to `make_subtype_of` but if the check fails this will attempt to coerce the
    /// argument to the target type. When this happens, the given expression is wrapped in
    /// a new expression to convert its type. E.g. `array` -> `array.as_slice()`
    ///
    /// Currently the only type coercion in Noir is `[T; N]` into `[T]` via `.as_slice()`.
    pub fn make_subtype_with_coercions(
        &self,
        expected: &Type,
        expression: ExprId,
        interner: &mut NodeInterner,
        errors: &mut Vec<TypeCheckError>,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        let span = interner.expr_span(&expression);
        if let Err(err_span) = self.is_subtype_of(expected, span) {
            if !self.try_array_to_slice_coercion(expected, expression, span, interner) {
                Self::issue_errors(expected, err_span, errors, make_error);
            }
        }
    }

    /// Try to apply the array to slice coercion to this given type pair and expression.
    /// If self can be converted to target this way, do so and return true to indicate success.
    fn try_array_to_slice_coercion(
        &self,
        target: &Type,
        expression: ExprId,
        span: Span,
        interner: &mut NodeInterner,
    ) -> bool {
        let this = self.follow_bindings();
        let target = target.follow_bindings();

        if let (Type::Array(size1, element1), Type::Array(size2, element2)) = (&this, &target) {
            let size1 = size1.follow_bindings();
            let size2 = size2.follow_bindings();

            // If we have an array and our target is a slice
            if matches!(size1, Type::Constant(_)) && matches!(size2, Type::NotConstant) {
                // Still have to ensure the element types match.
                // Don't need to issue an error here if not, it will be done in make_subtype_of_with_coercions
                if element1.is_subtype_of(element2, span).is_ok() {
                    convert_array_expression_to_slice(expression, this, target, interner);
                    return true;
                }
            }
        }
        false
    }

    /// Checks if self is a subtype of `other`. Returns Ok(()) if it is and Err(_) if not.
    /// Note that this function may permanently bind type variables regardless of whether it
    /// returned Ok or Err.
    pub fn is_subtype_of(&self, other: &Type, span: Span) -> Result<(), SpanKind> {
        use Type::*;
        match (self, other) {
            (Error, _) | (_, Error) => Ok(()),

            (TypeVariable(binding, TypeVariableKind::IntegerOrField(comptime)), other) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.is_subtype_of(other, span);
                }

                // Otherwise, check it is unified against an integer and bind it
                other.try_bind_to_polymorphic_int(binding, comptime, true, span)
            }
            // These needs to be a separate case to keep the argument order of is_subtype_of
            (other, TypeVariable(binding, TypeVariableKind::IntegerOrField(comptime))) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return other.is_subtype_of(link, span);
                }

                // use_subtype is false here since we have other <: PolymorphicInt
                // while the flag expects PolymorphicInt <: other
                other.try_bind_to_polymorphic_int(binding, comptime, false, span)
            }

            (TypeVariable(binding, TypeVariableKind::Normal), other) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.is_subtype_of(other, span);
                }

                other.try_bind_to(binding)
            }
            (other, TypeVariable(binding, TypeVariableKind::Normal)) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return other.is_subtype_of(link, span);
                }

                other.try_bind_to(binding)
            }

            (TypeVariable(binding, TypeVariableKind::Constant(length)), other) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.is_subtype_of(other, span);
                }

                other.try_bind_to_maybe_constant(binding, *length)
            }
            (other, TypeVariable(binding, TypeVariableKind::Constant(length))) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return other.is_subtype_of(link, span);
                }

                other.try_bind_to_maybe_constant(binding, *length)
            }

            (Array(len_a, elem_a), Array(len_b, elem_b)) => {
                len_a.is_subtype_of(len_b, span)?;
                elem_a.is_subtype_of(elem_b, span)
            }

            (Tuple(elements_a), Tuple(elements_b)) => {
                if elements_a.len() != elements_b.len() {
                    Err(SpanKind::None)
                } else {
                    for (a, b) in elements_a.iter().zip(elements_b) {
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

            (FieldElement(comptime_a), FieldElement(comptime_b)) => {
                comptime_a.is_subtype_of(comptime_b, span)
            }

            (Integer(comptime_a, signed_a, bits_a), Integer(comptime_b, signed_b, bits_b)) => {
                if signed_a == signed_b && bits_a == bits_b {
                    comptime_a.is_subtype_of(comptime_b, span)
                } else {
                    Err(SpanKind::None)
                }
            }

            (Bool(comptime_a), Bool(comptime_b)) => comptime_a.is_subtype_of(comptime_b, span),

            (NamedGeneric(binding_a, name_a), NamedGeneric(binding_b, name_b)) => {
                // Ensure NamedGenerics are never bound during type checking
                assert!(binding_a.borrow().is_unbound());
                assert!(binding_b.borrow().is_unbound());

                if name_a == name_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }

            (Function(params_a, ret_a), Function(params_b, ret_b)) => {
                if params_a.len() == params_b.len() {
                    for (a, b) in params_a.iter().zip(params_b) {
                        a.is_subtype_of(b, span)?;
                    }

                    // return types are contravariant, so this must be ret_b <: ret_a instead of the reverse
                    ret_b.is_subtype_of(ret_a, span)
                } else {
                    Err(SpanKind::None)
                }
            }

            // `T <: U  =>  &mut T <: &mut U` would be unsound(*), so mutable
            // references are never subtypes of each other.
            //
            // (*) Consider:
            // ```
            // // Assume Dog <: Animal and Cat <: Animal
            // let x: &mut Dog = ...;
            //
            // fn set_to_cat(y: &mut Animal) {
            //     *y = Cat;
            // }
            //
            // set_to_cat(x); // uh-oh: x: Dog, yet it now holds a Cat
            // ```
            (MutableReference(elem_a), MutableReference(elem_b)) => elem_a.try_unify(elem_b, span),

            (other_a, other_b) => {
                if other_a == other_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }
        }
    }

    /// If this type is a Type::Constant (used in array lengths), or is bound
    /// to a Type::Constant, return the constant as a u64.
    pub fn evaluate_to_u64(&self) -> Option<u64> {
        if let Some(binding) = self.get_inner_type_variable() {
            if let TypeBinding::Bound(binding) = &*binding.borrow() {
                return binding.evaluate_to_u64();
            }
        }

        match self {
            Type::TypeVariable(_, TypeVariableKind::Constant(size)) => Some(*size),
            Type::Array(len, _elem) => len.evaluate_to_u64(),
            Type::Constant(x) => Some(*x),
            _ => None,
        }
    }

    // Note; use strict_eq instead of partial_eq when comparing field types
    // in this method, you most likely want to distinguish between public and private
    pub fn as_abi_type(&self) -> AbiType {
        match self {
            Type::FieldElement(_) => AbiType::Field,
            Type::Array(size, typ) => {
                let length = size
                    .evaluate_to_u64()
                    .expect("Cannot have variable sized arrays as a parameter to main");
                AbiType::Array { length, typ: Box::new(typ.as_abi_type()) }
            }
            Type::Integer(_, sign, bit_width) => {
                let sign = match sign {
                    Signedness::Unsigned => noirc_abi::Sign::Unsigned,
                    Signedness::Signed => noirc_abi::Sign::Signed,
                };

                AbiType::Integer { sign, width: *bit_width }
            }
            Type::TypeVariable(binding, TypeVariableKind::IntegerOrField(_)) => {
                match &*binding.borrow() {
                    TypeBinding::Bound(typ) => typ.as_abi_type(),
                    TypeBinding::Unbound(_) => Type::default_int_type(None).as_abi_type(),
                }
            }
            Type::Bool(_) => AbiType::Boolean,
            Type::String(size) => {
                let size = size
                    .evaluate_to_u64()
                    .expect("Cannot have variable sized strings as a parameter to main");
                AbiType::String { length: size }
            }
            Type::Error => unreachable!(),
            Type::Unit => unreachable!(),
            Type::Constant(_) => unreachable!(),
            Type::Struct(def, args) => {
                let struct_type = def.borrow();
                let fields = struct_type.get_fields(args);
                let fields = vecmap(fields, |(name, typ)| (name, typ.as_abi_type()));
                AbiType::Struct { fields }
            }
            Type::Tuple(_) => todo!("as_abi_type not yet implemented for tuple types"),
            Type::TypeVariable(_, _) => unreachable!(),
            Type::NamedGeneric(..) => unreachable!(),
            Type::Forall(..) => unreachable!(),
            Type::Function(_, _) => unreachable!(),
            Type::MutableReference(_) => unreachable!("&mut cannot be used in the abi"),
            Type::NotConstant => unreachable!(),
        }
    }

    /// Iterate over the fields of this type.
    /// Panics if the type is not a struct or tuple.
    pub fn iter_fields(&self) -> impl Iterator<Item = (String, Type)> {
        let fields: Vec<_> = match self {
            // Unfortunately the .borrow() here forces us to collect into a Vec
            // only to have to call .into_iter again afterward. Trying to elide
            // collecting to a Vec leads to us dropping the temporary Ref before
            // the iterator is returned
            Type::Struct(def, args) => vecmap(&def.borrow().fields, |(name, _)| {
                let name = &name.0.contents;
                let typ = def.borrow().get_field(name, args).unwrap().0;
                (name.clone(), typ)
            }),
            Type::Tuple(fields) => {
                let fields = fields.iter().enumerate();
                vecmap(fields, |(i, field)| (i.to_string(), field.clone()))
            }
            other => panic!("Tried to iterate over the fields of '{other}', which has none"),
        };
        fields.into_iter()
    }

    /// Retrieves the type of the given field name
    /// Panics if the type is not a struct or tuple.
    pub fn get_field_type(&self, field_name: &str) -> Type {
        match self {
            Type::Struct(def, args) => def.borrow().get_field(field_name, args).unwrap().0,
            Type::Tuple(fields) => {
                let mut fields = fields.iter().enumerate();
                fields.find(|(i, _)| i.to_string() == *field_name).unwrap().1.clone()
            }
            other => panic!("Tried to iterate over the fields of '{other}', which has none"),
        }
    }

    /// Instantiate this type, replacing any type variables it is quantified
    /// over with fresh type variables. If this type is not a Type::Forall,
    /// it is unchanged.
    pub fn instantiate(&self, interner: &mut NodeInterner) -> (Type, TypeBindings) {
        match self {
            Type::Forall(typevars, typ) => {
                let replacements = typevars
                    .iter()
                    .map(|(id, var)| {
                        let new = interner.next_type_variable();
                        (*id, (var.clone(), new))
                    })
                    .collect();

                let instantiated = typ.substitute(&replacements);
                (instantiated, replacements)
            }
            other => (other.clone(), HashMap::new()),
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
            TypeBinding::Unbound(id) => match type_bindings.get(id) {
                Some((_, binding)) => binding.clone(),
                None => self.clone(),
            },
        };

        match self {
            Type::Array(size, element) => {
                let size = Box::new(size.substitute(type_bindings));
                let element = Box::new(element.substitute(type_bindings));
                Type::Array(size, element)
            }
            Type::String(size) => {
                let size = Box::new(size.substitute(type_bindings));
                Type::String(size)
            }
            Type::NamedGeneric(binding, _) | Type::TypeVariable(binding, _) => {
                substitute_binding(binding)
            }
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
            Type::Forall(typevars, typ) => {
                // Trying to substitute a variable defined within a nested Forall
                // is usually impossible and indicative of an error in the type checker somewhere.
                for (var, _) in typevars {
                    assert!(!type_bindings.contains_key(var));
                }
                let typ = Box::new(typ.substitute(type_bindings));
                Type::Forall(typevars.clone(), typ)
            }
            Type::Function(args, ret) => {
                let args = vecmap(args, |arg| arg.substitute(type_bindings));
                let ret = Box::new(ret.substitute(type_bindings));
                Type::Function(args, ret)
            }
            Type::MutableReference(element) => {
                Type::MutableReference(Box::new(element.substitute(type_bindings)))
            }

            Type::FieldElement(_)
            | Type::Integer(_, _, _)
            | Type::Bool(_)
            | Type::Constant(_)
            | Type::Error
            | Type::NotConstant
            | Type::Unit => self.clone(),
        }
    }

    /// True if the given TypeVariableId is free anywhere within self
    fn occurs(&self, target_id: TypeVariableId) -> bool {
        match self {
            Type::Array(len, elem) => len.occurs(target_id) || elem.occurs(target_id),
            Type::String(len) => len.occurs(target_id),
            Type::Struct(_, generic_args) => generic_args.iter().any(|arg| arg.occurs(target_id)),
            Type::Tuple(fields) => fields.iter().any(|field| field.occurs(target_id)),
            Type::NamedGeneric(binding, _) | Type::TypeVariable(binding, _) => {
                match &*binding.borrow() {
                    TypeBinding::Bound(binding) => binding.occurs(target_id),
                    TypeBinding::Unbound(id) => *id == target_id,
                }
            }
            Type::Forall(typevars, typ) => {
                !typevars.iter().any(|(id, _)| *id == target_id) && typ.occurs(target_id)
            }
            Type::Function(args, ret) => {
                args.iter().any(|arg| arg.occurs(target_id)) || ret.occurs(target_id)
            }
            Type::MutableReference(element) => element.occurs(target_id),

            Type::FieldElement(_)
            | Type::Integer(_, _, _)
            | Type::Bool(_)
            | Type::Constant(_)
            | Type::Error
            | Type::NotConstant
            | Type::Unit => false,
        }
    }

    /// Follow any TypeVariable bindings within this type. Doing so ensures
    /// that if the bindings are rebound or unbound from under the type then the
    /// returned type will not change (because it will no longer contain the
    /// links that may be unbound).
    ///
    /// Expected to be called on an instantiated type (with no Type::Foralls)
    pub fn follow_bindings(&self) -> Type {
        use Type::*;
        match self {
            Array(size, elem) => {
                Array(Box::new(size.follow_bindings()), Box::new(elem.follow_bindings()))
            }
            String(size) => String(Box::new(size.follow_bindings())),
            Struct(def, args) => {
                let args = vecmap(args, |arg| arg.follow_bindings());
                Struct(def.clone(), args)
            }
            Tuple(args) => Tuple(vecmap(args, |arg| arg.follow_bindings())),

            TypeVariable(var, _) | NamedGeneric(var, _) => {
                if let TypeBinding::Bound(typ) = &*var.borrow() {
                    return typ.follow_bindings();
                }
                self.clone()
            }

            Function(args, ret) => {
                let args = vecmap(args, |arg| arg.follow_bindings());
                let ret = Box::new(ret.follow_bindings());
                Function(args, ret)
            }
            MutableReference(element) => MutableReference(Box::new(element.follow_bindings())),

            // Expect that this function should only be called on instantiated types
            Forall(..) => unreachable!(),

            FieldElement(_)
            | Integer(_, _, _)
            | Bool(_)
            | Constant(_)
            | Unit
            | Error
            | NotConstant => self.clone(),
        }
    }
}

/// Wraps a given `expression` in `expression.as_slice()`
fn convert_array_expression_to_slice(
    expression: ExprId,
    array_type: Type,
    target_type: Type,
    interner: &mut NodeInterner,
) {
    let as_slice_method = interner
        .lookup_primitive_method(&array_type, "as_slice")
        .expect("Expected 'as_slice' method to be present in Noir's stdlib");

    let as_slice_id = interner.function_definition_id(as_slice_method);
    let location = interner.expr_location(&expression);
    let as_slice = HirExpression::Ident(HirIdent { location, id: as_slice_id });
    let func = interner.push_expr(as_slice);

    let arguments = vec![expression];
    let call = HirExpression::Call(HirCallExpression { func, arguments, location });
    let call = interner.push_expr(call);

    interner.push_expr_location(call, location.span, location.file);
    interner.push_expr_location(func, location.span, location.file);

    interner.push_expr_type(&call, target_type.clone());
    interner.push_expr_type(&func, Type::Function(vec![array_type], Box::new(target_type)));
}

impl BinaryTypeOperator {
    /// Return the actual rust numeric function associated with this operator
    pub fn function(self) -> fn(u64, u64) -> u64 {
        match self {
            BinaryTypeOperator::Addition => |a, b| a.wrapping_add(b),
            BinaryTypeOperator::Subtraction => |a, b| a.wrapping_sub(b),
            BinaryTypeOperator::Multiplication => |a, b| a.wrapping_mul(b),
            BinaryTypeOperator::Division => |a, b| a.wrapping_div(b),
            BinaryTypeOperator::Modulo => |a, b| a.wrapping_rem(b), // % b,
        }
    }
}

impl TypeVariableKind {
    /// Returns the default type this type variable should be bound to if it is still unbound
    /// during monomorphization.
    pub(crate) fn default_type(&self) -> Type {
        match self {
            TypeVariableKind::IntegerOrField(_) | TypeVariableKind::Normal => Type::field(None),
            TypeVariableKind::Constant(length) => Type::Constant(*length),
        }
    }
}
