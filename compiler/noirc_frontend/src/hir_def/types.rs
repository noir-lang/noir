use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

#[cfg(test)]
use proptest_derive::Arbitrary;

use acvm::{AcirField, FieldElement};

use crate::{
    ast::{IntegerBitSize, ItemVisibility},
    hir::type_check::{generics::TraitGenerics, TypeCheckError},
    node_interner::{ExprId, NodeInterner, TraitId, TypeAliasId},
};
use iter_extended::vecmap;
use noirc_errors::{Location, Span};
use noirc_printable_type::PrintableType;

use crate::{
    ast::{Ident, Signedness},
    node_interner::StructId,
};

use super::{
    expr::{HirCallExpression, HirExpression, HirIdent},
    traits::NamedType,
};

mod arithmetic;

#[derive(Eq, Clone, Ord, PartialOrd)]
pub enum Type {
    /// A primitive Field type
    FieldElement,

    /// Array(N, E) is an array of N elements of type E. It is expected that N
    /// is either a type variable of some kind or a Type::Constant.
    Array(Box<Type>, Box<Type>),

    /// Slice(E) is a slice of elements of type E.
    Slice(Box<Type>),

    /// A primitive integer type with the given sign and bit count.
    /// E.g. `u32` would be `Integer(Unsigned, ThirtyTwo)`
    Integer(Signedness, IntegerBitSize),

    /// The primitive `bool` type.
    Bool,

    /// String(N) is an array of characters of length N. It is expected that N
    /// is either a type variable of some kind or a Type::Constant.
    String(Box<Type>),

    /// FmtString(N, Vec<E>) is an array of characters of length N that contains
    /// a list of fields specified inside the string by the following regular expression r"\{([\S]+)\}"
    FmtString(Box<Type>, Box<Type>),

    /// The unit type `()`.
    Unit,

    /// A tuple type with the given list of fields in the order they appear in source code.
    Tuple(Vec<Type>),

    /// A user-defined struct type. The `Shared<StructType>` field here refers to
    /// the shared definition for each instance of this struct type. The `Vec<Type>`
    /// represents the generic arguments (if any) to this struct type.
    Struct(Shared<StructType>, Vec<Type>),

    /// A user-defined alias to another type. Similar to a Struct, this carries a shared
    /// reference to the definition of the alias along with any generics that may have
    /// been applied to the alias.
    Alias(Shared<TypeAlias>, Vec<Type>),

    /// TypeVariables are stand-in variables for some type which is not yet known.
    /// They are not to be confused with NamedGenerics. While the later mostly works
    /// as with normal types (ie. for two NamedGenerics T and U, T != U), TypeVariables
    /// will be automatically rebound as necessary to satisfy any calls to unify.
    ///
    /// TypeVariables are often created when a generic function is instantiated. This
    /// is a process that replaces each NamedGeneric in a generic function with a TypeVariable.
    /// Doing this at each call site of a generic function is how they can be called with
    /// different argument types each time.
    TypeVariable(TypeVariable),

    /// `impl Trait` when used in a type position.
    /// These are only matched based on the TraitId. The trait name parameter is only
    /// used for displaying error messages using the name of the trait.
    TraitAsType(TraitId, Rc<String>, TraitGenerics),

    /// NamedGenerics are the 'T' or 'U' in a user-defined generic function
    /// like `fn foo<T, U>(...) {}`. Unlike TypeVariables, they cannot be bound over.
    NamedGeneric(TypeVariable, Rc<String>),

    /// A cast (to, from) that's checked at monomorphization.
    ///
    /// Simplifications on arithmetic generics are only allowed on the LHS.
    CheckedCast {
        from: Box<Type>,
        to: Box<Type>,
    },

    /// A functions with arguments, a return type and environment.
    /// the environment should be `Unit` by default,
    /// for closures it should contain a `Tuple` type with the captured
    /// variable types.
    Function(
        Vec<Type>,
        /*return_type:*/ Box<Type>,
        /*environment:*/ Box<Type>,
        /*unconstrained*/ bool,
    ),

    /// &mut T
    MutableReference(Box<Type>),

    /// A type generic over the given type variables.
    /// Storing both the TypeVariableId and TypeVariable isn't necessary
    /// but it makes handling them both easier. The TypeVariableId should
    /// never be bound over during type checking, but during monomorphization it
    /// will be and thus needs the full TypeVariable link.
    Forall(GenericTypeVars, Box<Type>),

    /// A type-level integer. Included to let
    /// 1. an Array's size type variable
    ///     bind to an integer without special checks to bind it to a non-type.
    /// 2. values to be used at the type level
    Constant(FieldElement, Kind),

    /// The type of quoted code in macros. This is always a comptime-only type
    Quoted(QuotedType),

    InfixExpr(Box<Type>, BinaryTypeOperator, Box<Type>),

    /// The result of some type error. Remembering type errors as their own type variant lets
    /// us avoid issuing repeat type errors for the same item. For example, a lambda with
    /// an invalid type would otherwise issue a new error each time it is called
    /// if not for this variant.
    Error,
}

/// A Kind is the type of a Type. These are used since only certain kinds of types are allowed in
/// certain positions.
///
/// For example, the type of a struct field or a function parameter is expected to be
/// a type of kind * (represented here as `Normal`). Types used in positions where a number
/// is expected (such as in an array length position) are expected to be of kind `Kind::Numeric`.
#[derive(PartialEq, Eq, Clone, Hash, Debug, PartialOrd, Ord)]
pub enum Kind {
    /// Can bind to any type
    // TODO(https://github.com/noir-lang/noir/issues/6194): evaluate need for and usage of
    Any,

    /// Can bind to any type, except Type::Constant and Type::InfixExpr
    Normal,

    /// A generic integer or field type. This is a more specific kind of TypeVariable
    /// that can only be bound to Type::Field, Type::Integer, or other polymorphic integers.
    /// This is the type of undecorated integer literals like `46`. Typing them in this way
    /// allows them to be polymorphic over the actual integer/field type used without requiring
    /// type annotations on each integer literal.
    IntegerOrField,

    /// A generic integer type. This is a more specific kind of TypeVariable
    /// that can only be bound to Type::Integer, or other polymorphic integers.
    Integer,

    /// Can bind to a Type::Constant or Type::InfixExpr of the given kind
    Numeric(Box<Type>),
}

impl Kind {
    // Kind::Numeric constructor helper
    pub fn numeric(typ: Type) -> Kind {
        Kind::Numeric(Box::new(typ))
    }

    pub(crate) fn is_error(&self) -> bool {
        match self.follow_bindings() {
            Self::Numeric(typ) => *typ == Type::Error,
            _ => false,
        }
    }

    pub(crate) fn is_type_level_field_element(&self) -> bool {
        let type_level = false;
        self.is_field_element(type_level)
    }

    /// If value_level, only check for Type::FieldElement,
    /// else only check for a type-level FieldElement
    fn is_field_element(&self, value_level: bool) -> bool {
        match self.follow_bindings() {
            Kind::Numeric(typ) => typ.is_field_element(value_level),
            Kind::IntegerOrField => value_level,
            _ => false,
        }
    }

    pub(crate) fn u32() -> Self {
        Self::numeric(Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo))
    }

    pub(crate) fn follow_bindings(&self) -> Self {
        match self {
            Self::Any => Self::Any,
            Self::Normal => Self::Normal,
            Self::Integer => Self::Integer,
            Self::IntegerOrField => Self::IntegerOrField,
            Self::Numeric(typ) => Self::numeric(typ.follow_bindings()),
        }
    }

    /// Unifies this kind with the other. Returns true on success
    pub(crate) fn unifies(&self, other: &Kind) -> bool {
        match (self, other) {
            // Kind::Any unifies with everything
            (Kind::Any, _) | (_, Kind::Any) => true,

            // Kind::Normal unifies with Kind::Integer and Kind::IntegerOrField
            (Kind::Normal, Kind::Integer | Kind::IntegerOrField)
            | (Kind::Integer | Kind::IntegerOrField, Kind::Normal) => true,

            // Kind::Integer unifies with Kind::IntegerOrField
            (Kind::Integer | Kind::IntegerOrField, Kind::Integer | Kind::IntegerOrField) => true,

            // Kind::Numeric unifies along its Type argument
            (Kind::Numeric(lhs), Kind::Numeric(rhs)) => {
                let mut bindings = TypeBindings::new();
                let unifies = lhs.try_unify(rhs, &mut bindings).is_ok();
                if unifies {
                    Type::apply_type_bindings(bindings);
                }
                unifies
            }

            // everything unifies with itself
            (lhs, rhs) => lhs == rhs,
        }
    }

    pub(crate) fn unify(&self, other: &Kind) -> Result<(), UnificationError> {
        if self.unifies(other) {
            Ok(())
        } else {
            Err(UnificationError)
        }
    }

    /// Returns the default type this type variable should be bound to if it is still unbound
    /// during monomorphization.
    pub(crate) fn default_type(&self) -> Option<Type> {
        match self {
            Kind::IntegerOrField => Some(Type::default_int_or_field_type()),
            Kind::Integer => Some(Type::default_int_type()),
            Kind::Any | Kind::Normal | Kind::Numeric(..) => None,
        }
    }

    fn integral_maximum_size(&self) -> Option<FieldElement> {
        match self.follow_bindings() {
            Kind::Any | Kind::IntegerOrField | Kind::Integer | Kind::Normal => None,
            Self::Numeric(typ) => typ.integral_maximum_size(),
        }
    }

    /// Ensure the given value fits in self.integral_maximum_size()
    fn ensure_value_fits(
        &self,
        value: FieldElement,
        span: Span,
    ) -> Result<FieldElement, TypeCheckError> {
        match self.integral_maximum_size() {
            None => Ok(value),
            Some(maximum_size) => (value <= maximum_size).then_some(value).ok_or_else(|| {
                TypeCheckError::OverflowingConstant {
                    value,
                    kind: self.clone(),
                    maximum_size,
                    span,
                }
            }),
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Any => write!(f, "any"),
            Kind::Normal => write!(f, "normal"),
            Kind::Integer => write!(f, "int"),
            Kind::IntegerOrField => write!(f, "intOrField"),
            Kind::Numeric(typ) => write!(f, "numeric {}", typ),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
#[cfg_attr(test, derive(strum_macros::EnumIter))]
pub enum QuotedType {
    Expr,
    Quoted,
    TopLevelItem,
    Type,
    TypedExpr,
    StructDefinition,
    TraitConstraint,
    TraitDefinition,
    TraitImpl,
    UnresolvedType,
    FunctionDefinition,
    Module,
    CtString,
}

/// A list of (TypeVariableId, Kind)'s to bind to a type. Storing the
/// TypeVariable in addition to the matching TypeVariableId allows
/// the binding to later be undone if needed.
pub type TypeBindings = HashMap<TypeVariableId, (TypeVariable, Kind, Type)>;

/// Represents a struct type in the type system. Each instance of this
/// rust struct will be shared across all Type::Struct variants that represent
/// the same struct type.
pub struct StructType {
    /// A unique id representing this struct type. Used to check if two
    /// struct types are equal.
    pub id: StructId,

    pub name: Ident,

    /// Fields are ordered and private, they should only
    /// be accessed through get_field(), get_fields(), or instantiate()
    /// since these will handle applying generic arguments to fields as well.
    fields: Vec<StructField>,

    pub generics: Generics,
    pub location: Location,
}

pub struct StructField {
    pub visibility: ItemVisibility,
    pub name: Ident,
    pub typ: Type,
}

/// Corresponds to generic lists such as `<T, U>` in the source program.
/// Used mainly for resolved types which no longer need information such
/// as names or kinds
pub type GenericTypeVars = Vec<TypeVariable>;

/// Corresponds to generic lists such as `<T, U>` with additional
/// information gathered during name resolution that is necessary
/// correctly resolving types.
pub type Generics = Vec<ResolvedGeneric>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGeneric {
    pub name: Rc<String>,
    pub type_var: TypeVariable,
    pub span: Span,
}

impl ResolvedGeneric {
    pub fn as_named_generic(self) -> Type {
        Type::NamedGeneric(self.type_var, self.name)
    }

    pub fn kind(&self) -> Kind {
        self.type_var.kind()
    }
}

enum FunctionCoercionResult {
    NoCoercion,
    Coerced(Type),
    UnconstrainedMismatch(Type),
}

impl std::hash::Hash for StructType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for StructType {}

impl PartialEq for StructType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for StructType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StructType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl StructType {
    pub fn new(
        id: StructId,
        name: Ident,

        location: Location,
        fields: Vec<StructField>,
        generics: Generics,
    ) -> StructType {
        StructType { id, fields, name, location, generics }
    }

    /// To account for cyclic references between structs, a struct's
    /// fields are resolved strictly after the struct itself is initially
    /// created. Therefore, this method is used to set the fields once they
    /// become known.
    pub fn set_fields(&mut self, fields: Vec<StructField>) {
        self.fields = fields;
    }

    pub fn num_fields(&self) -> usize {
        self.fields.len()
    }

    /// Returns the field matching the given field name, as well as its visibility and field index.
    pub fn get_field(
        &self,
        field_name: &str,
        generic_args: &[Type],
    ) -> Option<(Type, ItemVisibility, usize)> {
        assert_eq!(self.generics.len(), generic_args.len());

        self.fields.iter().enumerate().find(|(_, field)| field.name.0.contents == field_name).map(
            |(i, field)| {
                let substitutions = self
                    .generics
                    .iter()
                    .zip(generic_args)
                    .map(|(old, new)| {
                        (
                            old.type_var.id(),
                            (old.type_var.clone(), old.type_var.kind(), new.clone()),
                        )
                    })
                    .collect();

                (field.typ.substitute(&substitutions), field.visibility, i)
            },
        )
    }

    /// Returns all the fields of this type, after being applied to the given generic arguments.
    pub fn get_fields_with_visibility(
        &self,
        generic_args: &[Type],
    ) -> Vec<(String, ItemVisibility, Type)> {
        let substitutions = self.get_fields_substitutions(generic_args);

        vecmap(&self.fields, |field| {
            let name = field.name.0.contents.clone();
            (name, field.visibility, field.typ.substitute(&substitutions))
        })
    }

    pub fn get_fields(&self, generic_args: &[Type]) -> Vec<(String, Type)> {
        let substitutions = self.get_fields_substitutions(generic_args);

        vecmap(&self.fields, |field| {
            let name = field.name.0.contents.clone();
            (name, field.typ.substitute(&substitutions))
        })
    }

    fn get_fields_substitutions(
        &self,
        generic_args: &[Type],
    ) -> HashMap<TypeVariableId, (TypeVariable, Kind, Type)> {
        assert_eq!(self.generics.len(), generic_args.len());

        self.generics
            .iter()
            .zip(generic_args)
            .map(|(old, new)| {
                (old.type_var.id(), (old.type_var.clone(), old.type_var.kind(), new.clone()))
            })
            .collect()
    }

    /// Returns the name and raw types of each field of this type.
    /// This will not substitute any generic arguments so a generic field like `x`
    /// in `struct Foo<T> { x: T }` will return a `("x", T)` pair.
    ///
    /// This method is almost never what is wanted for type checking or monomorphization,
    /// prefer to use `get_fields` whenever possible.
    pub fn get_fields_as_written(&self) -> Vec<StructField> {
        vecmap(&self.fields, |field| StructField {
            visibility: field.visibility,
            name: field.name.clone(),
            typ: field.typ.clone(),
        })
    }

    /// Returns the field at the given index. Panics if no field exists at the given index.
    pub fn field_at(&self, index: usize) -> &StructField {
        &self.fields[index]
    }

    pub fn field_names(&self) -> BTreeSet<Ident> {
        self.fields.iter().map(|field| field.name.clone()).collect()
    }

    /// Instantiate this struct type, returning a Vec of the new generic args (in
    /// the same order as self.generics)
    pub fn instantiate(&self, interner: &mut NodeInterner) -> Vec<Type> {
        vecmap(&self.generics, |generic| interner.next_type_variable_with_kind(generic.kind()))
    }
}

impl std::fmt::Display for StructType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Wrap around an unsolved type
#[derive(Debug, Clone, Eq)]
pub struct TypeAlias {
    pub name: Ident,
    pub id: TypeAliasId,
    pub typ: Type,
    pub generics: Generics,
    pub location: Location,
}

impl std::hash::Hash for TypeAlias {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for TypeAlias {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for TypeAlias {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for TypeAlias {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for TypeAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TypeAlias {
    pub fn new(
        id: TypeAliasId,
        name: Ident,
        location: Location,
        typ: Type,
        generics: Generics,
    ) -> TypeAlias {
        TypeAlias { id, typ, name, location, generics }
    }

    pub fn set_type_and_generics(&mut self, new_typ: Type, new_generics: Generics) {
        assert_eq!(self.typ, Type::Error);
        self.typ = new_typ;
        self.generics = new_generics;
    }

    pub fn get_type(&self, generic_args: &[Type]) -> Type {
        assert_eq!(self.generics.len(), generic_args.len());

        let substitutions = self
            .generics
            .iter()
            .zip(generic_args)
            .map(|(old, new)| {
                (old.type_var.id(), (old.type_var.clone(), old.type_var.kind(), new.clone()))
            })
            .collect();

        self.typ.substitute(&substitutions)
    }

    pub fn instantiate(&self, interner: &NodeInterner) -> Type {
        let args = vecmap(&self.generics, |_| interner.next_type_variable());
        self.get_type(&args)
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

    pub fn unwrap_or_clone(self) -> T
    where
        T: Clone,
    {
        match Rc::try_unwrap(self.0) {
            Ok(elem) => elem.into_inner(),
            Err(rc) => rc.as_ref().clone().into_inner(),
        }
    }
}

/// A restricted subset of binary operators useable on
/// type level integers for use in the array length positions of types.
#[cfg_attr(test, derive(Arbitrary))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryTypeOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulo,
}

/// A TypeVariable is a mutable reference that is either
/// bound to some type, or unbound with a given TypeVariableId.
#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct TypeVariable(TypeVariableId, Shared<TypeBinding>);

impl TypeVariable {
    pub fn unbound(id: TypeVariableId, type_var_kind: Kind) -> Self {
        TypeVariable(id, Shared::new(TypeBinding::Unbound(id, type_var_kind)))
    }

    pub fn id(&self) -> TypeVariableId {
        self.0
    }

    /// Bind this type variable to a value.
    ///
    /// Panics if this TypeVariable is already Bound.
    /// Also Panics if the ID of this TypeVariable occurs within the given
    /// binding, as that would cause an infinitely recursive type.
    pub fn bind(&self, typ: Type) {
        let id = match &*self.1.borrow() {
            TypeBinding::Bound(binding) => {
                unreachable!("TypeVariable::bind, cannot bind bound var {} to {}", binding, typ)
            }
            TypeBinding::Unbound(id, _) => *id,
        };

        assert!(!typ.occurs(id), "{self:?} occurs within {typ:?}");
        *self.1.borrow_mut() = TypeBinding::Bound(typ);
    }

    pub fn try_bind(&self, binding: Type, kind: &Kind, span: Span) -> Result<(), TypeCheckError> {
        if !binding.kind().unifies(kind) {
            return Err(TypeCheckError::TypeKindMismatch {
                expected_kind: format!("{}", kind),
                expr_kind: format!("{}", binding.kind()),
                expr_span: span,
            });
        }

        let id = match &*self.1.borrow() {
            TypeBinding::Bound(binding) => {
                unreachable!("Expected unbound, found bound to {binding}")
            }
            TypeBinding::Unbound(id, _) => *id,
        };

        if binding.occurs(id) {
            Err(TypeCheckError::CyclicType { span, typ: binding })
        } else {
            *self.1.borrow_mut() = TypeBinding::Bound(binding);
            Ok(())
        }
    }

    /// Borrows this TypeVariable to (e.g.) manually match on the inner TypeBinding.
    pub fn borrow(&self) -> std::cell::Ref<TypeBinding> {
        self.1.borrow()
    }

    /// Unbind this type variable, setting it to Unbound(id).
    ///
    /// This is generally a logic error to use outside of monomorphization.
    pub fn unbind(&self, id: TypeVariableId, type_var_kind: Kind) {
        *self.1.borrow_mut() = TypeBinding::Unbound(id, type_var_kind);
    }

    /// Forcibly bind a type variable to a new type - even if the type
    /// variable is already bound to a different type. This generally
    /// a logic error to use outside of monomorphization.
    ///
    /// Asserts that the given type is compatible with the given Kind
    pub fn force_bind(&self, typ: Type) {
        if !typ.occurs(self.id()) {
            *self.1.borrow_mut() = TypeBinding::Bound(typ);
        }
    }

    pub fn kind(&self) -> Kind {
        match &*self.borrow() {
            TypeBinding::Bound(binding) => binding.kind(),
            TypeBinding::Unbound(_, type_var_kind) => type_var_kind.clone(),
        }
    }

    /// Check that if bound, it's an integer
    /// and if unbound, that it's a Kind::Integer
    pub fn is_integer(&self) -> bool {
        match &*self.borrow() {
            TypeBinding::Bound(binding) => matches!(binding.follow_bindings(), Type::Integer(..)),
            TypeBinding::Unbound(_, type_var_kind) => {
                matches!(type_var_kind.follow_bindings(), Kind::Integer)
            }
        }
    }

    /// Check that if bound, it's an integer or field
    /// and if unbound, that it's a Kind::IntegerOrField
    pub fn is_integer_or_field(&self) -> bool {
        match &*self.borrow() {
            TypeBinding::Bound(binding) => {
                matches!(binding.follow_bindings(), Type::Integer(..) | Type::FieldElement)
            }
            TypeBinding::Unbound(_, type_var_kind) => {
                matches!(type_var_kind.follow_bindings(), Kind::IntegerOrField)
            }
        }
    }

    /// If value_level, only check for Type::FieldElement,
    /// else only check for a type-level FieldElement
    fn is_field_element(&self, value_level: bool) -> bool {
        match &*self.borrow() {
            TypeBinding::Bound(binding) => binding.is_field_element(value_level),
            TypeBinding::Unbound(_, type_var_kind) => type_var_kind.is_field_element(value_level),
        }
    }
}

/// TypeBindings are the mutable insides of a TypeVariable.
/// They are either bound to some type, or are unbound.
#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum TypeBinding {
    Bound(Type),
    Unbound(TypeVariableId, Kind),
}

impl TypeBinding {
    pub fn is_unbound(&self) -> bool {
        matches!(self, TypeBinding::Unbound(_, _))
    }
}

/// A unique ID used to differentiate different type variables
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeVariableId(pub usize);

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::FieldElement => {
                write!(f, "Field")
            }
            Type::Array(len, typ) => {
                write!(f, "[{typ}; {len}]")
            }
            Type::Slice(typ) => {
                write!(f, "[{typ}]")
            }
            Type::Integer(sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "i{num_bits}"),
                Signedness::Unsigned => write!(f, "u{num_bits}"),
            },
            Type::TypeVariable(var) => {
                let binding = &var.1;
                match &*binding.borrow() {
                    TypeBinding::Unbound(_, type_var_kind) => match type_var_kind {
                        Kind::Any | Kind::Normal => write!(f, "{}", var.borrow()),
                        Kind::Integer => write!(f, "{}", Type::default_int_type()),
                        Kind::IntegerOrField => write!(f, "Field"),
                        Kind::Numeric(_typ) => write!(f, "_"),
                    },
                    TypeBinding::Bound(binding) => {
                        write!(f, "{}", binding)
                    }
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
            Type::Alias(alias, args) => {
                let args = vecmap(args, |arg| arg.to_string());
                if args.is_empty() {
                    write!(f, "{}", alias.borrow())
                } else {
                    write!(f, "{}<{}>", alias.borrow(), args.join(", "))
                }
            }
            Type::TraitAsType(_id, name, generics) => {
                write!(f, "impl {}{}", name, generics)
            }
            Type::Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Type::Bool => write!(f, "bool"),
            Type::String(len) => write!(f, "str<{len}>"),
            Type::FmtString(len, elements) => {
                write!(f, "fmtstr<{len}, {elements}>")
            }
            Type::Unit => write!(f, "()"),
            Type::Error => write!(f, "error"),
            Type::NamedGeneric(binding, name) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.fmt(f),
                TypeBinding::Unbound(_, _) if name.is_empty() => write!(f, "_"),
                TypeBinding::Unbound(_, _) => write!(f, "{name}"),
            },
            Type::CheckedCast { to, .. } => write!(f, "{to}"),
            Type::Constant(x, _kind) => write!(f, "{x}"),
            Type::Forall(typevars, typ) => {
                let typevars = vecmap(typevars, |var| var.id().to_string());
                write!(f, "forall {}. {}", typevars.join(" "), typ)
            }
            Type::Function(args, ret, env, unconstrained) => {
                if *unconstrained {
                    write!(f, "unconstrained ")?;
                }

                let closure_env_text = match **env {
                    Type::Unit => "".to_string(),
                    _ => format!("[{env}]"),
                };

                let args = vecmap(args.iter(), ToString::to_string);

                write!(f, "fn{closure_env_text}({}) -> {ret}", args.join(", "))
            }
            Type::MutableReference(element) => {
                write!(f, "&mut {element}")
            }
            Type::Quoted(quoted) => write!(f, "{}", quoted),
            Type::InfixExpr(lhs, op, rhs) => {
                let this = self.canonicalize_checked();

                // Prevent infinite recursion
                if this != *self {
                    write!(f, "{this}")
                } else {
                    write!(f, "({lhs} {op} {rhs})")
                }
            }
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
            TypeBinding::Unbound(id, _) => id.fmt(f),
        }
    }
}

impl std::fmt::Display for QuotedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotedType::Expr => write!(f, "Expr"),
            QuotedType::Quoted => write!(f, "Quoted"),
            QuotedType::TopLevelItem => write!(f, "TopLevelItem"),
            QuotedType::Type => write!(f, "Type"),
            QuotedType::TypedExpr => write!(f, "TypedExpr"),
            QuotedType::StructDefinition => write!(f, "StructDefinition"),
            QuotedType::TraitDefinition => write!(f, "TraitDefinition"),
            QuotedType::TraitConstraint => write!(f, "TraitConstraint"),
            QuotedType::TraitImpl => write!(f, "TraitImpl"),
            QuotedType::UnresolvedType => write!(f, "UnresolvedType"),
            QuotedType::FunctionDefinition => write!(f, "FunctionDefinition"),
            QuotedType::Module => write!(f, "Module"),
            QuotedType::CtString => write!(f, "CtString"),
        }
    }
}

pub struct UnificationError;

impl Type {
    pub fn default_int_or_field_type() -> Type {
        Type::FieldElement
    }

    pub fn default_int_type() -> Type {
        Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo)
    }

    pub fn type_variable_with_kind(interner: &NodeInterner, type_var_kind: Kind) -> Type {
        let id = interner.next_type_variable_id();
        let var = TypeVariable::unbound(id, type_var_kind);
        Type::TypeVariable(var)
    }

    pub fn type_variable(id: TypeVariableId) -> Type {
        let var = TypeVariable::unbound(id, Kind::Any);
        Type::TypeVariable(var)
    }

    pub fn polymorphic_integer_or_field(interner: &NodeInterner) -> Type {
        let type_var_kind = Kind::IntegerOrField;
        Self::type_variable_with_kind(interner, type_var_kind)
    }

    pub fn polymorphic_integer(interner: &NodeInterner) -> Type {
        let type_var_kind = Kind::Integer;
        Self::type_variable_with_kind(interner, type_var_kind)
    }

    /// A bit of an awkward name for this function - this function returns
    /// true for type variables or polymorphic integers which are unbound.
    /// NamedGenerics will always be false as although they are bindable,
    /// they shouldn't be bound over until monomorphization.
    pub fn is_bindable(&self) -> bool {
        match self {
            Type::TypeVariable(binding) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.is_bindable(),
                TypeBinding::Unbound(_, _) => true,
            },
            Type::Alias(alias, args) => alias.borrow().get_type(args).is_bindable(),
            _ => false,
        }
    }

    pub fn is_field(&self) -> bool {
        matches!(self.follow_bindings(), Type::FieldElement)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self.follow_bindings(), Type::Bool)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.follow_bindings(), Type::Integer(_, _))
    }

    /// If value_level, only check for Type::FieldElement,
    /// else only check for a type-level FieldElement
    fn is_field_element(&self, value_level: bool) -> bool {
        match self.follow_bindings() {
            Type::FieldElement => value_level,
            Type::TypeVariable(var) => var.is_field_element(value_level),
            Type::Constant(_, kind) => !value_level && kind.is_field_element(true),
            _ => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        matches!(self.follow_bindings(), Type::Integer(Signedness::Signed, _))
    }

    pub fn is_unsigned(&self) -> bool {
        matches!(self.follow_bindings(), Type::Integer(Signedness::Unsigned, _))
    }

    /// While Kind::is_numeric refers to numeric _types_,
    /// this method checks for numeric _values_
    pub fn is_numeric_value(&self) -> bool {
        use Kind as K;
        use Type::*;
        match self.follow_bindings() {
            FieldElement => true,
            Integer(..) => true,
            Bool => true,
            TypeVariable(var) => match &*var.borrow() {
                TypeBinding::Bound(typ) => typ.is_numeric_value(),
                TypeBinding::Unbound(_, type_var_kind) => {
                    matches!(type_var_kind, K::Integer | K::IntegerOrField)
                }
            },
            _ => false,
        }
    }

    pub fn is_primitive(&self) -> bool {
        match self.follow_bindings() {
            Type::FieldElement
            | Type::Array(_, _)
            | Type::Slice(_)
            | Type::Integer(..)
            | Type::Bool
            | Type::String(_)
            | Type::FmtString(_, _)
            | Type::Unit
            | Type::Function(..)
            | Type::Tuple(..) => true,
            Type::Alias(alias_type, generics) => {
                alias_type.borrow().get_type(&generics).is_primitive()
            }
            Type::MutableReference(typ) => typ.is_primitive(),
            Type::Struct(..)
            | Type::TypeVariable(..)
            | Type::TraitAsType(..)
            | Type::NamedGeneric(..)
            | Type::CheckedCast { .. }
            | Type::Forall(..)
            | Type::Constant(..)
            | Type::Quoted(..)
            | Type::InfixExpr(..)
            | Type::Error => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match self.follow_bindings_shallow().as_ref() {
            Type::Function(..) => true,
            Type::Alias(alias_type, _) => alias_type.borrow().typ.is_function(),
            _ => false,
        }
    }

    /// True if this type can be used as a parameter to `main` or a contract function.
    /// This is only false for unsized types like slices or slices that do not make sense
    /// as a program input such as named generics or mutable references.
    ///
    /// This function should match the same check done in `create_value_from_type` in acir_gen.
    /// If this function does not catch a case where a type should be valid, it will later lead to a
    /// panic in that function instead of a user-facing compiler error message.
    pub(crate) fn is_valid_for_program_input(&self) -> bool {
        match self {
            // Type::Error is allowed as usual since it indicates an error was already issued and
            // we don't need to issue further errors about this likely unresolved type
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Unit
            | Type::Constant(_, _)
            | Type::Error => true,

            Type::FmtString(_, _)
            | Type::TypeVariable(_)
            | Type::NamedGeneric(_, _)
            | Type::Function(_, _, _, _)
            | Type::MutableReference(_)
            | Type::Forall(_, _)
            | Type::Quoted(_)
            | Type::Slice(_)
            | Type::TraitAsType(..) => false,

            Type::CheckedCast { to, .. } => to.is_valid_for_program_input(),

            Type::Alias(alias, generics) => {
                let alias = alias.borrow();
                alias.get_type(generics).is_valid_for_program_input()
            }

            Type::Array(length, element) => {
                length.is_valid_for_program_input() && element.is_valid_for_program_input()
            }
            Type::String(length) => length.is_valid_for_program_input(),
            Type::Tuple(elements) => elements.iter().all(|elem| elem.is_valid_for_program_input()),
            Type::Struct(definition, generics) => definition
                .borrow()
                .get_fields(generics)
                .into_iter()
                .all(|(_, field)| field.is_valid_for_program_input()),

            Type::InfixExpr(lhs, _, rhs) => {
                lhs.is_valid_for_program_input() && rhs.is_valid_for_program_input()
            }
        }
    }

    /// True if this type can be used as a parameter to an ACIR function that is not `main` or a contract function.
    /// This encapsulates functions for which we may not want to inline during compilation.
    ///
    /// The inputs allowed for a function entry point differ from those allowed as input to a program as there are
    /// certain types which through compilation we know what their size should be.
    /// This includes types such as numeric generics.
    pub(crate) fn is_valid_non_inlined_function_input(&self) -> bool {
        match self {
            // Type::Error is allowed as usual since it indicates an error was already issued and
            // we don't need to issue further errors about this likely unresolved type
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Unit
            | Type::Constant(_, _)
            | Type::TypeVariable(_)
            | Type::NamedGeneric(_, _)
            | Type::InfixExpr(..)
            | Type::Error => true,

            Type::FmtString(_, _)
            // To enable this we would need to determine the size of the closure outputs at compile-time.
            // This is possible as long as the output size is not dependent upon a witness condition.
            | Type::Function(_, _, _, _)
            | Type::Slice(_)
            | Type::MutableReference(_)
            | Type::Forall(_, _)
            // TODO: probably can allow code as it is all compile time
            | Type::Quoted(_)
            | Type::TraitAsType(..) => false,

            Type::CheckedCast { to, .. } => to.is_valid_non_inlined_function_input(),

            Type::Alias(alias, generics) => {
                let alias = alias.borrow();
                alias.get_type(generics).is_valid_non_inlined_function_input()
            }

            Type::Array(length, element) => {
                length.is_valid_non_inlined_function_input() && element.is_valid_non_inlined_function_input()
            }
            Type::String(length) => length.is_valid_non_inlined_function_input(),
            Type::Tuple(elements) => elements.iter().all(|elem| elem.is_valid_non_inlined_function_input()),
            Type::Struct(definition, generics) => definition
                .borrow()
                .get_fields(generics)
                .into_iter()
                .all(|(_, field)| field.is_valid_non_inlined_function_input()),
        }
    }

    /// Returns true if a value of this type can safely pass between constrained and
    /// unconstrained functions (and vice-versa).
    pub(crate) fn is_valid_for_unconstrained_boundary(&self) -> bool {
        match self {
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Unit
            | Type::Constant(_, _)
            | Type::Slice(_)
            | Type::Function(_, _, _, _)
            | Type::FmtString(_, _)
            | Type::InfixExpr(..)
            | Type::Error => true,

            Type::TypeVariable(type_var) | Type::NamedGeneric(type_var, _) => {
                if let TypeBinding::Bound(typ) = &*type_var.borrow() {
                    typ.is_valid_for_unconstrained_boundary()
                } else {
                    true
                }
            }

            Type::CheckedCast { to, .. } => to.is_valid_for_unconstrained_boundary(),

            // Quoted objects only exist at compile-time where the only execution
            // environment is the interpreter. In this environment, they are valid.
            Type::Quoted(_) => true,

            Type::MutableReference(_) | Type::Forall(_, _) | Type::TraitAsType(..) => false,

            Type::Alias(alias, generics) => {
                let alias = alias.borrow();
                alias.get_type(generics).is_valid_for_unconstrained_boundary()
            }

            Type::Array(length, element) => {
                length.is_valid_for_unconstrained_boundary()
                    && element.is_valid_for_unconstrained_boundary()
            }
            Type::String(length) => length.is_valid_for_unconstrained_boundary(),
            Type::Tuple(elements) => {
                elements.iter().all(|elem| elem.is_valid_for_unconstrained_boundary())
            }
            Type::Struct(definition, generics) => definition
                .borrow()
                .get_fields(generics)
                .into_iter()
                .all(|(_, field)| field.is_valid_for_unconstrained_boundary()),
        }
    }

    /// Returns the number of `Forall`-quantified type variables on this type.
    /// Returns 0 if this is not a Type::Forall
    pub fn generic_count(&self) -> usize {
        match self {
            Type::Forall(generics, _) => generics.len(),
            Type::CheckedCast { to, .. } => to.generic_count(),
            Type::TypeVariable(type_variable) | Type::NamedGeneric(type_variable, _) => {
                match &*type_variable.borrow() {
                    TypeBinding::Bound(binding) => binding.generic_count(),
                    TypeBinding::Unbound(_, _) => 0,
                }
            }
            _ => 0,
        }
    }

    /// Takes a monomorphic type and generalizes it over each of the type variables in the
    /// given type bindings, ignoring what each type variable is bound to in the TypeBindings
    /// and their Kind's
    pub(crate) fn generalize_from_substitutions(self, type_bindings: TypeBindings) -> Type {
        let polymorphic_type_vars = vecmap(type_bindings, |(_, (type_var, _kind, _))| type_var);
        Type::Forall(polymorphic_type_vars, Box::new(self))
    }

    /// Return this type as a monomorphic type - without a `Type::Forall` if there is one.
    /// This is only a shallow check since Noir's type system prohibits `Type::Forall` anywhere
    /// inside other types.
    pub fn as_monotype(&self) -> &Type {
        match self {
            Type::Forall(_, typ) => typ.as_ref(),
            other => other,
        }
    }

    /// Return the generics and type within this `Type::Forall`.
    /// Panics if `self` is not `Type::Forall`
    pub fn unwrap_forall(&self) -> (Cow<GenericTypeVars>, &Type) {
        match self {
            Type::Forall(generics, typ) => (Cow::Borrowed(generics), typ.as_ref()),
            other => (Cow::Owned(GenericTypeVars::new()), other),
        }
    }

    pub(crate) fn kind(&self) -> Kind {
        match self {
            Type::CheckedCast { to, .. } => to.kind(),
            Type::NamedGeneric(var, _) => var.kind(),
            Type::Constant(_, kind) => kind.clone(),
            Type::TypeVariable(var) => match &*var.borrow() {
                TypeBinding::Bound(ref typ) => typ.kind(),
                TypeBinding::Unbound(_, ref type_var_kind) => type_var_kind.clone(),
            },
            Type::InfixExpr(lhs, _op, rhs) => lhs.infix_kind(rhs),
            Type::Alias(def, generics) => def.borrow().get_type(generics).kind(),
            // This is a concrete FieldElement, not an IntegerOrField
            Type::FieldElement
            | Type::Integer(..)
            | Type::Array(..)
            | Type::Slice(..)
            | Type::Bool
            | Type::String(..)
            | Type::FmtString(..)
            | Type::Unit
            | Type::Tuple(..)
            | Type::Struct(..)
            | Type::TraitAsType(..)
            | Type::Function(..)
            | Type::MutableReference(..)
            | Type::Forall(..)
            | Type::Quoted(..) => Kind::Normal,
            Type::Error => Kind::Any,
        }
    }

    /// Unifies self and other kinds or fails with a Kind error
    fn infix_kind(&self, other: &Self) -> Kind {
        let self_kind = self.kind();
        let other_kind = other.kind();
        if self_kind.unifies(&other_kind) {
            self_kind
        } else {
            Kind::numeric(Type::Error)
        }
    }

    /// Returns the number of field elements required to represent the type once encoded.
    pub fn field_count(&self, location: &Location) -> u32 {
        match self {
            Type::FieldElement | Type::Integer { .. } | Type::Bool => 1,
            Type::Array(size, typ) => {
                let length = size
                    .evaluate_to_u32(location.span)
                    .expect("Cannot have variable sized arrays as a parameter to main");
                let typ = typ.as_ref();
                length * typ.field_count(location)
            }
            Type::Struct(def, args) => {
                let struct_type = def.borrow();
                let fields = struct_type.get_fields(args);
                fields.iter().fold(0, |acc, (_, field_type)| acc + field_type.field_count(location))
            }
            Type::CheckedCast { to, .. } => to.field_count(location),
            Type::Alias(def, generics) => def.borrow().get_type(generics).field_count(location),
            Type::Tuple(fields) => {
                fields.iter().fold(0, |acc, field_typ| acc + field_typ.field_count(location))
            }
            Type::String(size) => size
                .evaluate_to_u32(location.span)
                .expect("Cannot have variable sized strings as a parameter to main"),
            Type::FmtString(_, _)
            | Type::Unit
            | Type::TypeVariable(_)
            | Type::TraitAsType(..)
            | Type::NamedGeneric(_, _)
            | Type::Function(_, _, _, _)
            | Type::MutableReference(_)
            | Type::Forall(_, _)
            | Type::Constant(_, _)
            | Type::Quoted(_)
            | Type::Slice(_)
            | Type::InfixExpr(..)
            | Type::Error => unreachable!("This type cannot exist as a parameter to main"),
        }
    }

    pub(crate) fn is_nested_slice(&self) -> bool {
        match self {
            Type::Slice(elem) => elem.as_ref().contains_slice(),
            Type::Array(_, elem) => elem.as_ref().contains_slice(),
            Type::Alias(alias, generics) => alias.borrow().get_type(generics).is_nested_slice(),
            _ => false,
        }
    }

    pub(crate) fn contains_slice(&self) -> bool {
        match self {
            Type::Slice(_) => true,
            Type::Struct(struct_typ, generics) => {
                let fields = struct_typ.borrow().get_fields(generics);
                for field in fields.iter() {
                    if field.1.contains_slice() {
                        return true;
                    }
                }
                false
            }
            Type::Tuple(types) => {
                for typ in types.iter() {
                    if typ.contains_slice() {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Try to bind a PolymorphicInt variable to self, succeeding if self is an integer, field,
    /// other PolymorphicInt type, or type variable. If successful, the binding is placed in the
    /// given TypeBindings map rather than linked immediately.
    fn try_bind_to_polymorphic_int(
        &self,
        var: &TypeVariable,
        bindings: &mut TypeBindings,
        only_integer: bool,
    ) -> Result<(), UnificationError> {
        let target_id = match &*var.borrow() {
            TypeBinding::Bound(_) => unreachable!(),
            TypeBinding::Unbound(id, _) => *id,
        };

        if !self.kind().unifies(&Kind::IntegerOrField) {
            return Err(UnificationError);
        }

        let this = self.substitute(bindings).follow_bindings();
        match &this {
            Type::Integer(..) => {
                bindings.insert(target_id, (var.clone(), Kind::Integer, this));
                Ok(())
            }
            Type::FieldElement if !only_integer => {
                bindings.insert(target_id, (var.clone(), Kind::IntegerOrField, this));
                Ok(())
            }
            Type::TypeVariable(self_var) => {
                let borrow = self_var.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => {
                        typ.try_bind_to_polymorphic_int(var, bindings, only_integer)
                    }
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(ref id, _) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(ref new_target_id, Kind::IntegerOrField) => {
                        let type_var_kind = Kind::IntegerOrField;
                        if only_integer {
                            let var_clone = var.clone();
                            Kind::Integer.unify(&type_var_kind)?;
                            // Integer is more specific than IntegerOrField so we bind the type
                            // variable to Integer instead.
                            let clone = Type::TypeVariable(var_clone);
                            bindings
                                .insert(*new_target_id, (self_var.clone(), type_var_kind, clone));
                        } else {
                            bindings.insert(
                                target_id,
                                (var.clone(), Kind::IntegerOrField, this.clone()),
                            );
                        }
                        Ok(())
                    }
                    TypeBinding::Unbound(_new_target_id, Kind::Integer) => {
                        Kind::Integer.unify(&Kind::Integer)?;
                        bindings.insert(target_id, (var.clone(), Kind::Integer, this.clone()));
                        Ok(())
                    }
                    TypeBinding::Unbound(new_target_id, ref type_var_kind) => {
                        let var_clone = var.clone();
                        // Bind to the most specific type variable kind
                        let clone_kind =
                            if only_integer { Kind::Integer } else { Kind::IntegerOrField };
                        clone_kind.unify(type_var_kind)?;
                        let clone = Type::TypeVariable(var_clone);
                        bindings.insert(*new_target_id, (self_var.clone(), clone_kind, clone));
                        Ok(())
                    }
                }
            }
            _ => Err(UnificationError),
        }
    }

    /// Try to bind the given type variable to self. Although the given type variable
    /// is expected to be of Kind::Normal, this binding can still fail
    /// if the given type variable occurs within `self` as that would create a recursive type.
    ///
    /// If successful, the binding is placed in the
    /// given TypeBindings map rather than linked immediately.
    fn try_bind_to(
        &self,
        var: &TypeVariable,
        bindings: &mut TypeBindings,
        kind: Kind,
    ) -> Result<(), UnificationError> {
        let target_id = match &*var.borrow() {
            TypeBinding::Bound(_) => unreachable!(),
            TypeBinding::Unbound(id, _) => *id,
        };

        if !self.kind().unifies(&kind) {
            return Err(UnificationError);
        }

        let this = self.substitute(bindings).follow_bindings();
        if let Some((binding, kind)) = this.get_inner_type_variable() {
            match &*binding.borrow() {
                TypeBinding::Bound(typ) => return typ.try_bind_to(var, bindings, kind),
                // Don't recursively bind the same id to itself
                TypeBinding::Unbound(id, _) if *id == target_id => return Ok(()),
                _ => (),
            }
        }

        // Check if the target id occurs within `this` before binding. Otherwise this could
        // cause infinitely recursive types
        if this.occurs(target_id) {
            Err(UnificationError)
        } else {
            bindings.insert(target_id, (var.clone(), this.kind(), this.clone()));
            Ok(())
        }
    }

    fn get_inner_type_variable(&self) -> Option<(Shared<TypeBinding>, Kind)> {
        match self {
            Type::TypeVariable(var) => Some((var.1.clone(), var.kind())),
            Type::NamedGeneric(var, _) => Some((var.1.clone(), var.kind())),
            Type::CheckedCast { to, .. } => to.get_inner_type_variable(),
            _ => None,
        }
    }

    /// Try to unify this type with another, setting any type variables found
    /// equal to the other type in the process. When comparing types, unification
    /// (including try_unify) are almost always preferred over Type::eq as unification
    /// will correctly handle generic types.
    pub fn unify(&self, expected: &Type) -> Result<(), UnificationError> {
        let mut bindings = TypeBindings::new();

        self.try_unify(expected, &mut bindings).map(|()| {
            // Commit any type bindings on success
            Self::apply_type_bindings(bindings);
        })
    }

    /// `try_unify` is a bit of a misnomer since although errors are not committed,
    /// any unified bindings are on success.
    pub fn try_unify(
        &self,
        other: &Type,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        use Type::*;

        let lhs = self.follow_bindings_shallow();
        let rhs = other.follow_bindings_shallow();

        let lhs = match lhs.as_ref() {
            Type::InfixExpr(..) => Cow::Owned(self.canonicalize()),
            other => Cow::Borrowed(other),
        };

        let rhs = match rhs.as_ref() {
            Type::InfixExpr(..) => Cow::Owned(other.canonicalize()),
            other => Cow::Borrowed(other),
        };

        match (lhs.as_ref(), rhs.as_ref()) {
            (Error, _) | (_, Error) => Ok(()),

            (Alias(alias, args), other) | (other, Alias(alias, args)) => {
                let alias = alias.borrow().get_type(args);
                alias.try_unify(other, bindings)
            }

            (TypeVariable(var), other) | (other, TypeVariable(var)) => match &*var.borrow() {
                TypeBinding::Bound(typ) => {
                    if typ.is_numeric_value() {
                        other.try_unify_to_type_variable(var, bindings, |bindings| {
                            let only_integer = matches!(typ, Type::Integer(..));
                            other.try_bind_to_polymorphic_int(var, bindings, only_integer)
                        })
                    } else {
                        other.try_unify_to_type_variable(var, bindings, |bindings| {
                            other.try_bind_to(var, bindings, typ.kind())
                        })
                    }
                }
                TypeBinding::Unbound(_id, Kind::IntegerOrField) => other
                    .try_unify_to_type_variable(var, bindings, |bindings| {
                        let only_integer = false;
                        other.try_bind_to_polymorphic_int(var, bindings, only_integer)
                    }),
                TypeBinding::Unbound(_id, Kind::Integer) => {
                    other.try_unify_to_type_variable(var, bindings, |bindings| {
                        let only_integer = true;
                        other.try_bind_to_polymorphic_int(var, bindings, only_integer)
                    })
                }
                TypeBinding::Unbound(_id, type_var_kind) => {
                    other.try_unify_to_type_variable(var, bindings, |bindings| {
                        other.try_bind_to(var, bindings, type_var_kind.clone())
                    })
                }
            },

            (Array(len_a, elem_a), Array(len_b, elem_b)) => {
                len_a.try_unify(len_b, bindings)?;
                elem_a.try_unify(elem_b, bindings)
            }

            (Slice(elem_a), Slice(elem_b)) => elem_a.try_unify(elem_b, bindings),

            (String(len_a), String(len_b)) => len_a.try_unify(len_b, bindings),

            (FmtString(len_a, elements_a), FmtString(len_b, elements_b)) => {
                len_a.try_unify(len_b, bindings)?;
                elements_a.try_unify(elements_b, bindings)
            }

            (Tuple(elements_a), Tuple(elements_b)) => {
                if elements_a.len() != elements_b.len() {
                    Err(UnificationError)
                } else {
                    for (a, b) in elements_a.iter().zip(elements_b) {
                        a.try_unify(b, bindings)?;
                    }
                    Ok(())
                }
            }

            // No recursive try_unify call for struct fields. Don't want
            // to mutate shared type variables within struct definitions.
            // This isn't possible currently but will be once noir gets generic types
            (Struct(id_a, args_a), Struct(id_b, args_b)) => {
                if id_a == id_b && args_a.len() == args_b.len() {
                    for (a, b) in args_a.iter().zip(args_b) {
                        a.try_unify(b, bindings)?;
                    }
                    Ok(())
                } else {
                    Err(UnificationError)
                }
            }

            (CheckedCast { to, .. }, other) | (other, CheckedCast { to, .. }) => {
                to.try_unify(other, bindings)
            }

            (NamedGeneric(binding, _), other) | (other, NamedGeneric(binding, _))
                if !binding.borrow().is_unbound() =>
            {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    link.try_unify(other, bindings)
                } else {
                    unreachable!("If guard ensures binding is bound")
                }
            }

            (NamedGeneric(binding_a, name_a), NamedGeneric(binding_b, name_b)) => {
                // Bound NamedGenerics are caught by the check above
                assert!(binding_a.borrow().is_unbound());
                assert!(binding_b.borrow().is_unbound());

                if name_a == name_b {
                    binding_a.kind().unify(&binding_b.kind())
                } else {
                    Err(UnificationError)
                }
            }

            (
                Function(params_a, ret_a, env_a, unconstrained_a),
                Function(params_b, ret_b, env_b, unconstrained_b),
            ) => {
                if unconstrained_a == unconstrained_b && params_a.len() == params_b.len() {
                    for (a, b) in params_a.iter().zip(params_b.iter()) {
                        a.try_unify(b, bindings)?;
                    }

                    env_a.try_unify(env_b, bindings)?;
                    ret_b.try_unify(ret_a, bindings)
                } else {
                    Err(UnificationError)
                }
            }

            (MutableReference(elem_a), MutableReference(elem_b)) => {
                elem_a.try_unify(elem_b, bindings)
            }

            (InfixExpr(lhs_a, op_a, rhs_a), InfixExpr(lhs_b, op_b, rhs_b)) => {
                if op_a == op_b {
                    // We need to preserve the original bindings since if syntactic equality
                    // fails we fall back to other equality strategies.
                    let mut new_bindings = bindings.clone();
                    let lhs_result = lhs_a.try_unify(lhs_b, &mut new_bindings);
                    let rhs_result = rhs_a.try_unify(rhs_b, &mut new_bindings);

                    if lhs_result.is_ok() && rhs_result.is_ok() {
                        *bindings = new_bindings;
                        Ok(())
                    } else {
                        lhs.try_unify_by_moving_constant_terms(&rhs, bindings)
                    }
                } else {
                    Err(UnificationError)
                }
            }

            (Constant(value, kind), other) | (other, Constant(value, kind)) => {
                let dummy_span = Span::default();
                if let Ok(other_value) = other.evaluate_to_field_element(kind, dummy_span) {
                    if *value == other_value && kind.unifies(&other.kind()) {
                        Ok(())
                    } else {
                        Err(UnificationError)
                    }
                } else if let InfixExpr(lhs, op, rhs) = other {
                    if let Some(inverse) = op.approx_inverse() {
                        // Handle cases like `4 = a + b` by trying to solve to `a = 4 - b`
                        let new_type = InfixExpr(
                            Box::new(Constant(*value, kind.clone())),
                            inverse,
                            rhs.clone(),
                        );
                        new_type.try_unify(lhs, bindings)?;
                        Ok(())
                    } else {
                        Err(UnificationError)
                    }
                } else {
                    Err(UnificationError)
                }
            }

            (other_a, other_b) => {
                if other_a == other_b {
                    Ok(())
                } else {
                    Err(UnificationError)
                }
            }
        }
    }

    /// Try to unify a type variable to `self`.
    /// This is a helper function factored out from try_unify.
    fn try_unify_to_type_variable(
        &self,
        type_variable: &TypeVariable,
        bindings: &mut TypeBindings,

        // Bind the type variable to a type. This is factored out since depending on the
        // Kind, there are different methods to check whether the variable can
        // bind to the given type or not.
        bind_variable: impl FnOnce(&mut TypeBindings) -> Result<(), UnificationError>,
    ) -> Result<(), UnificationError> {
        match &*type_variable.borrow() {
            // If it is already bound, unify against what it is bound to
            TypeBinding::Bound(link) => link.try_unify(self, bindings),
            TypeBinding::Unbound(id, _) => {
                // We may have already "bound" this type variable in this call to
                // try_unify, so check those bindings as well.
                match bindings.get(id) {
                    Some((_, kind, binding)) => {
                        if !kind.unifies(&binding.kind()) {
                            return Err(UnificationError);
                        }
                        binding.clone().try_unify(self, bindings)
                    }

                    // Otherwise, bind it
                    None => bind_variable(bindings),
                }
            }
        }
    }

    /// Similar to `unify` but if the check fails this will attempt to coerce the
    /// argument to the target type. When this happens, the given expression is wrapped in
    /// a new expression to convert its type. E.g. `array` -> `array.as_slice()`
    ///
    /// Currently the only type coercion in Noir is `[T; N]` into `[T]` via `.as_slice()`.
    pub fn unify_with_coercions(
        &self,
        expected: &Type,
        expression: ExprId,
        span: Span,
        interner: &mut NodeInterner,
        errors: &mut Vec<TypeCheckError>,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        let mut bindings = TypeBindings::new();

        if let Ok(()) = self.try_unify(expected, &mut bindings) {
            Type::apply_type_bindings(bindings);
            return;
        }

        if self.try_array_to_slice_coercion(expected, expression, interner) {
            return;
        }

        // Try to coerce `fn (..) -> T` to `unconstrained fn (..) -> T`
        match self.try_fn_to_unconstrained_fn_coercion(expected) {
            FunctionCoercionResult::NoCoercion => errors.push(make_error()),
            FunctionCoercionResult::Coerced(coerced_self) => {
                coerced_self
                    .unify_with_coercions(expected, expression, span, interner, errors, make_error);
            }
            FunctionCoercionResult::UnconstrainedMismatch(coerced_self) => {
                errors.push(TypeCheckError::UnsafeFn { span });

                coerced_self
                    .unify_with_coercions(expected, expression, span, interner, errors, make_error);
            }
        }
    }

    // If `self` and `expected` are function types, tries to coerce `self` to `expected`.
    // Returns None if no coercion can be applied, otherwise returns `self` coerced to `expected`.
    fn try_fn_to_unconstrained_fn_coercion(&self, expected: &Type) -> FunctionCoercionResult {
        // If `self` and `expected` are function types, `self` can be coerced to `expected`
        // if `self` is unconstrained and `expected` is not. The other way around is an error, though.
        if let (
            Type::Function(params, ret, env, unconstrained_self),
            Type::Function(_, _, _, unconstrained_expected),
        ) = (self.follow_bindings(), expected.follow_bindings())
        {
            let coerced_type = Type::Function(params, ret, env, unconstrained_expected);

            match (unconstrained_self, unconstrained_expected) {
                (true, true) | (false, false) => FunctionCoercionResult::NoCoercion,
                (false, true) => FunctionCoercionResult::Coerced(coerced_type),
                (true, false) => FunctionCoercionResult::UnconstrainedMismatch(coerced_type),
            }
        } else {
            FunctionCoercionResult::NoCoercion
        }
    }

    /// Try to apply the array to slice coercion to this given type pair and expression.
    /// If self can be converted to target this way, do so and return true to indicate success.
    fn try_array_to_slice_coercion(
        &self,
        target: &Type,
        expression: ExprId,
        interner: &mut NodeInterner,
    ) -> bool {
        let this = self.follow_bindings();
        let target = target.follow_bindings();

        if let (Type::Array(_size, element1), Type::Slice(element2)) = (&this, &target) {
            // We can only do the coercion if the `as_slice` method exists.
            // This is usually true, but some tests don't have access to the standard library.
            if let Some(as_slice) = interner.lookup_primitive_method(&this, "as_slice", true) {
                // Still have to ensure the element types match.
                // Don't need to issue an error here if not, it will be done in unify_with_coercions
                let mut bindings = TypeBindings::new();
                if element1.try_unify(element2, &mut bindings).is_ok() {
                    convert_array_expression_to_slice(expression, this, target, as_slice, interner);
                    Self::apply_type_bindings(bindings);
                    return true;
                }
            }
        }
        false
    }

    /// Apply the given type bindings, making them permanently visible for each
    /// clone of each type variable bound.
    pub fn apply_type_bindings(bindings: TypeBindings) {
        for (type_variable, _kind, binding) in bindings.values() {
            type_variable.bind(binding.clone());
        }
    }

    /// If this type is a Type::Constant (used in array lengths), or is bound
    /// to a Type::Constant, return the constant as a u32.
    pub fn evaluate_to_u32(&self, span: Span) -> Result<u32, TypeCheckError> {
        self.evaluate_to_field_element(&Kind::u32(), span).map(|field_element| {
            field_element
                .try_to_u32()
                .expect("ICE: size should have already been checked by evaluate_to_field_element")
        })
    }

    // TODO(https://github.com/noir-lang/noir/issues/6260): remove
    // the unifies checks once all kinds checks are implemented?
    pub(crate) fn evaluate_to_field_element(
        &self,
        kind: &Kind,
        span: Span,
    ) -> Result<acvm::FieldElement, TypeCheckError> {
        let run_simplifications = true;
        self.evaluate_to_field_element_helper(kind, span, run_simplifications)
    }

    /// evaluate_to_field_element with optional generic arithmetic simplifications
    pub(crate) fn evaluate_to_field_element_helper(
        &self,
        kind: &Kind,
        span: Span,
        run_simplifications: bool,
    ) -> Result<acvm::FieldElement, TypeCheckError> {
        if let Some((binding, binding_kind)) = self.get_inner_type_variable() {
            if let TypeBinding::Bound(binding) = &*binding.borrow() {
                if kind.unifies(&binding_kind) {
                    return binding.evaluate_to_field_element_helper(
                        &binding_kind,
                        span,
                        run_simplifications,
                    );
                }
            }
        }

        let could_be_checked_cast = false;
        match self.canonicalize_helper(could_be_checked_cast, run_simplifications) {
            Type::Constant(x, constant_kind) => {
                if kind.unifies(&constant_kind) {
                    kind.ensure_value_fits(x, span)
                } else {
                    Err(TypeCheckError::TypeKindMismatch {
                        expected_kind: format!("{}", constant_kind),
                        expr_kind: format!("{}", kind),
                        expr_span: span,
                    })
                }
            }
            Type::InfixExpr(lhs, op, rhs) => {
                let infix_kind = lhs.infix_kind(&rhs);
                if kind.unifies(&infix_kind) {
                    let lhs_value = lhs.evaluate_to_field_element_helper(
                        &infix_kind,
                        span,
                        run_simplifications,
                    )?;
                    let rhs_value = rhs.evaluate_to_field_element_helper(
                        &infix_kind,
                        span,
                        run_simplifications,
                    )?;
                    op.function(lhs_value, rhs_value, &infix_kind, span)
                } else {
                    Err(TypeCheckError::TypeKindMismatch {
                        expected_kind: format!("{}", kind),
                        expr_kind: format!("{}", infix_kind),
                        expr_span: span,
                    })
                }
            }
            Type::CheckedCast { from, to } => {
                let to_value = to.evaluate_to_field_element(kind, span)?;

                // if both 'to' and 'from' evaluate to a constant,
                // return None unless they match
                let skip_simplifications = false;
                if let Ok(from_value) =
                    from.evaluate_to_field_element_helper(kind, span, skip_simplifications)
                {
                    if to_value == from_value {
                        Ok(to_value)
                    } else {
                        let to = *to.clone();
                        let from = *from.clone();
                        Err(TypeCheckError::TypeCanonicalizationMismatch {
                            to,
                            from,
                            to_value,
                            from_value,
                            span,
                        })
                    }
                } else {
                    Ok(to_value)
                }
            }
            other => Err(TypeCheckError::NonConstantEvaluated { typ: other, span }),
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
            Type::Struct(def, args) => vecmap(&def.borrow().fields, |field| {
                let name = &field.name.0.contents;
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
    pub fn get_field_type_and_visibility(
        &self,
        field_name: &str,
    ) -> Option<(Type, ItemVisibility)> {
        match self.follow_bindings() {
            Type::Struct(def, args) => def
                .borrow()
                .get_field(field_name, &args)
                .map(|(typ, visibility, _)| (typ, visibility)),
            Type::Tuple(fields) => {
                let mut fields = fields.into_iter().enumerate();
                fields
                    .find(|(i, _)| i.to_string() == *field_name)
                    .map(|(_, typ)| (typ, ItemVisibility::Public))
            }
            _ => None,
        }
    }

    /// Instantiate this type with the given type bindings.
    /// If any type variables which would be instantiated are contained in the
    /// given type bindings instead, the value from the type bindings is used.
    pub fn instantiate_with_bindings(
        &self,
        mut bindings: TypeBindings,
        interner: &NodeInterner,
    ) -> (Type, TypeBindings) {
        match self {
            Type::Forall(typevars, typ) => {
                for var in typevars {
                    bindings.entry(var.id()).or_insert_with(|| {
                        (var.clone(), var.kind(), interner.next_type_variable_with_kind(var.kind()))
                    });
                }

                let instantiated = typ.force_substitute(&bindings);
                (instantiated, bindings)
            }
            other => (other.clone(), bindings),
        }
    }

    /// Instantiate this type, replacing any type variables it is quantified
    /// over with fresh type variables. If this type is not a Type::Forall,
    /// it is unchanged.
    pub fn instantiate(&self, interner: &NodeInterner) -> (Type, TypeBindings) {
        match self {
            Type::Forall(typevars, typ) => {
                let replacements = typevars
                    .iter()
                    .map(|var| {
                        let new = interner.next_type_variable_with_kind(var.kind());
                        (var.id(), (var.clone(), var.kind(), new))
                    })
                    .collect();

                let instantiated = typ.force_substitute(&replacements);
                (instantiated, replacements)
            }
            other => (other.clone(), HashMap::new()),
        }
    }

    /// Instantiates a type with the given types.
    /// This differs from substitute in that only the quantified type variables
    /// are matched against the type list and are eligible for substitution - similar
    /// to normal instantiation. This function is used when the turbofish operator
    /// is used and generic substitutions are provided manually by users.
    ///
    /// Expects the given type vector to be the same length as the Forall type variables.
    pub fn instantiate_with(
        &self,
        types: Vec<Type>,
        interner: &NodeInterner,
        implicit_generic_count: usize,
    ) -> (Type, TypeBindings) {
        match self {
            Type::Forall(typevars, typ) => {
                assert_eq!(types.len() + implicit_generic_count, typevars.len(), "Turbofish operator used with incorrect generic count which was not caught by name resolution");

                let bindings =
                    (0..implicit_generic_count).map(|_| interner.next_type_variable()).chain(types);

                let replacements = typevars
                    .iter()
                    .zip(bindings)
                    .map(|(var, binding)| (var.id(), (var.clone(), var.kind(), binding)))
                    .collect();

                let instantiated = typ.substitute(&replacements);
                (instantiated, replacements)
            }
            other => (other.clone(), HashMap::new()),
        }
    }

    fn type_variable_id(&self) -> Option<TypeVariableId> {
        match self {
            Type::TypeVariable(variable) | Type::NamedGeneric(variable, _) => Some(variable.0),
            _ => None,
        }
    }

    /// Substitute any type variables found within this type with the
    /// given bindings if found. If a type variable is not found within
    /// the given TypeBindings, it is unchanged.
    pub fn substitute(&self, type_bindings: &TypeBindings) -> Type {
        self.substitute_helper(type_bindings, false)
    }

    /// Forcibly substitute any type variables found within this type with the
    /// given bindings if found. If a type variable is not found within
    /// the given TypeBindings, it is unchanged.
    ///
    /// Compared to `substitute`, this function will also substitute any type variables
    /// from type_bindings, even if they are bound in `self`. Since this can undo previous
    /// bindings, this function should be avoided unless necessary. Currently, it is only
    /// needed when handling bindings between trait methods and their corresponding impl
    /// method during monomorphization.
    pub fn force_substitute(&self, type_bindings: &TypeBindings) -> Type {
        self.substitute_helper(type_bindings, true)
    }

    /// This helper function only differs in the additional parameter which, if set,
    /// allows substitutions on already-bound type variables. This should be `false`
    /// for most uses, but is currently needed during monomorphization when instantiating
    /// trait functions to shed any previous bindings from recursive parent calls to the
    /// same trait.
    fn substitute_helper(
        &self,
        type_bindings: &TypeBindings,
        substitute_bound_typevars: bool,
    ) -> Type {
        if type_bindings.is_empty() {
            return self.clone();
        }

        let recur_on_binding = |id, replacement: &Type| {
            // Prevent recuring forever if there's a `T := T` binding
            if replacement.type_variable_id() == Some(id) {
                replacement.clone()
            } else {
                replacement.substitute_helper(type_bindings, substitute_bound_typevars)
            }
        };

        let substitute_binding = |binding: &TypeVariable| {
            // Check the id first to allow substituting to
            // type variables that have already been bound over.
            // This is needed for monomorphizing trait impl methods.
            match type_bindings.get(&binding.0) {
                Some((_, _kind, replacement)) if substitute_bound_typevars => {
                    recur_on_binding(binding.0, replacement)
                }
                _ => match &*binding.borrow() {
                    TypeBinding::Bound(binding) => {
                        binding.substitute_helper(type_bindings, substitute_bound_typevars)
                    }
                    TypeBinding::Unbound(id, _) => match type_bindings.get(id) {
                        Some((_, kind, replacement)) => {
                            assert!(
                                kind.unifies(&replacement.kind()),
                                "while substituting (unbound): expected kind of unbound TypeVariable ({:?}) to match the kind of its binding ({:?})",
                                kind,
                                replacement.kind()
                            );

                            recur_on_binding(binding.0, replacement)
                        }
                        None => self.clone(),
                    },
                },
            }
        };

        match self {
            Type::Array(size, element) => {
                let size = size.substitute_helper(type_bindings, substitute_bound_typevars);
                let element = element.substitute_helper(type_bindings, substitute_bound_typevars);
                Type::Array(Box::new(size), Box::new(element))
            }
            Type::Slice(element) => {
                let element = element.substitute_helper(type_bindings, substitute_bound_typevars);
                Type::Slice(Box::new(element))
            }
            Type::String(size) => {
                let size = size.substitute_helper(type_bindings, substitute_bound_typevars);
                Type::String(Box::new(size))
            }
            Type::FmtString(size, fields) => {
                let size = size.substitute_helper(type_bindings, substitute_bound_typevars);
                let fields = fields.substitute_helper(type_bindings, substitute_bound_typevars);
                Type::FmtString(Box::new(size), Box::new(fields))
            }
            Type::CheckedCast { from, to } => {
                let from = from.substitute_helper(type_bindings, substitute_bound_typevars);
                let to = to.substitute_helper(type_bindings, substitute_bound_typevars);
                Type::CheckedCast { from: Box::new(from), to: Box::new(to) }
            }
            Type::NamedGeneric(binding, _) | Type::TypeVariable(binding) => {
                substitute_binding(binding)
            }
            // Do not substitute_helper fields, it can lead to infinite recursion
            // and we should not match fields when type checking anyway.
            Type::Struct(fields, args) => {
                let args = vecmap(args, |arg| {
                    arg.substitute_helper(type_bindings, substitute_bound_typevars)
                });
                Type::Struct(fields.clone(), args)
            }
            Type::Alias(alias, args) => {
                let args = vecmap(args, |arg| {
                    arg.substitute_helper(type_bindings, substitute_bound_typevars)
                });
                Type::Alias(alias.clone(), args)
            }
            Type::Tuple(fields) => {
                let fields = vecmap(fields, |field| {
                    field.substitute_helper(type_bindings, substitute_bound_typevars)
                });
                Type::Tuple(fields)
            }
            Type::Forall(typevars, typ) => {
                // Trying to substitute_helper a variable de, substitute_bound_typevarsfined within a nested Forall
                // is usually impossible and indicative of an error in the type checker somewhere.
                for var in typevars {
                    assert!(!type_bindings.contains_key(&var.id()));
                }
                let typ = Box::new(typ.substitute_helper(type_bindings, substitute_bound_typevars));
                Type::Forall(typevars.clone(), typ)
            }
            Type::Function(args, ret, env, unconstrained) => {
                let args = vecmap(args, |arg| {
                    arg.substitute_helper(type_bindings, substitute_bound_typevars)
                });
                let ret = Box::new(ret.substitute_helper(type_bindings, substitute_bound_typevars));
                let env = Box::new(env.substitute_helper(type_bindings, substitute_bound_typevars));
                Type::Function(args, ret, env, *unconstrained)
            }
            Type::MutableReference(element) => Type::MutableReference(Box::new(
                element.substitute_helper(type_bindings, substitute_bound_typevars),
            )),

            Type::TraitAsType(s, name, generics) => {
                let ordered = vecmap(&generics.ordered, |arg| {
                    arg.substitute_helper(type_bindings, substitute_bound_typevars)
                });
                let named = vecmap(&generics.named, |arg| {
                    let typ = arg.typ.substitute_helper(type_bindings, substitute_bound_typevars);
                    NamedType { name: arg.name.clone(), typ }
                });
                Type::TraitAsType(*s, name.clone(), TraitGenerics { ordered, named })
            }
            Type::InfixExpr(lhs, op, rhs) => {
                let lhs = lhs.substitute_helper(type_bindings, substitute_bound_typevars);
                let rhs = rhs.substitute_helper(type_bindings, substitute_bound_typevars);
                Type::InfixExpr(Box::new(lhs), *op, Box::new(rhs))
            }

            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Constant(_, _)
            | Type::Error
            | Type::Quoted(_)
            | Type::Unit => self.clone(),
        }
    }

    /// True if the given TypeVariableId is free anywhere within self
    pub fn occurs(&self, target_id: TypeVariableId) -> bool {
        match self {
            Type::Array(len, elem) => len.occurs(target_id) || elem.occurs(target_id),
            Type::Slice(elem) => elem.occurs(target_id),
            Type::String(len) => len.occurs(target_id),
            Type::FmtString(len, fields) => {
                let len_occurs = len.occurs(target_id);
                let field_occurs = fields.occurs(target_id);
                len_occurs || field_occurs
            }
            Type::Struct(_, generic_args) | Type::Alias(_, generic_args) => {
                generic_args.iter().any(|arg| arg.occurs(target_id))
            }
            Type::TraitAsType(_, _, args) => {
                args.ordered.iter().any(|arg| arg.occurs(target_id))
                    || args.named.iter().any(|arg| arg.typ.occurs(target_id))
            }
            Type::Tuple(fields) => fields.iter().any(|field| field.occurs(target_id)),
            Type::CheckedCast { from, to } => from.occurs(target_id) || to.occurs(target_id),
            Type::NamedGeneric(type_var, _) | Type::TypeVariable(type_var) => {
                match &*type_var.borrow() {
                    TypeBinding::Bound(binding) => {
                        type_var.id() == target_id || binding.occurs(target_id)
                    }
                    TypeBinding::Unbound(id, _) => *id == target_id,
                }
            }
            Type::Forall(typevars, typ) => {
                !typevars.iter().any(|var| var.id() == target_id) && typ.occurs(target_id)
            }
            Type::Function(args, ret, env, _unconstrained) => {
                args.iter().any(|arg| arg.occurs(target_id))
                    || ret.occurs(target_id)
                    || env.occurs(target_id)
            }
            Type::MutableReference(element) => element.occurs(target_id),
            Type::InfixExpr(lhs, _op, rhs) => lhs.occurs(target_id) || rhs.occurs(target_id),

            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Constant(_, _)
            | Type::Error
            | Type::Quoted(_)
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
            Slice(elem) => Slice(Box::new(elem.follow_bindings())),
            String(size) => String(Box::new(size.follow_bindings())),
            FmtString(size, args) => {
                let size = Box::new(size.follow_bindings());
                let args = Box::new(args.follow_bindings());
                FmtString(size, args)
            }
            Struct(def, args) => {
                let args = vecmap(args, |arg| arg.follow_bindings());
                Struct(def.clone(), args)
            }
            Alias(def, args) => {
                // We don't need to vecmap(args, follow_bindings) since we're recursively
                // calling follow_bindings here already.
                def.borrow().get_type(args).follow_bindings()
            }
            Tuple(args) => Tuple(vecmap(args, |arg| arg.follow_bindings())),
            CheckedCast { from, to } => {
                let from = Box::new(from.follow_bindings());
                let to = Box::new(to.follow_bindings());
                CheckedCast { from, to }
            }
            TypeVariable(var) | NamedGeneric(var, _) => {
                if let TypeBinding::Bound(typ) = &*var.borrow() {
                    return typ.follow_bindings();
                }
                self.clone()
            }
            Function(args, ret, env, unconstrained) => {
                let args = vecmap(args, |arg| arg.follow_bindings());
                let ret = Box::new(ret.follow_bindings());
                let env = Box::new(env.follow_bindings());
                Function(args, ret, env, *unconstrained)
            }

            MutableReference(element) => MutableReference(Box::new(element.follow_bindings())),

            TraitAsType(s, name, args) => {
                let ordered = vecmap(&args.ordered, |arg| arg.follow_bindings());
                let named = vecmap(&args.named, |arg| NamedType {
                    name: arg.name.clone(),
                    typ: arg.typ.follow_bindings(),
                });
                TraitAsType(*s, name.clone(), TraitGenerics { ordered, named })
            }
            InfixExpr(lhs, op, rhs) => {
                let lhs = lhs.follow_bindings();
                let rhs = rhs.follow_bindings();
                InfixExpr(Box::new(lhs), *op, Box::new(rhs))
            }

            // Expect that this function should only be called on instantiated types
            Forall(..) => unreachable!(),
            FieldElement | Integer(_, _) | Bool | Constant(_, _) | Unit | Quoted(_) | Error => {
                self.clone()
            }
        }
    }

    /// Follow bindings if this is a type variable or generic to the first non-typevariable
    /// type. Unlike `follow_bindings`, this won't recursively follow any bindings on any
    /// fields or arguments of this type.
    pub fn follow_bindings_shallow(&self) -> Cow<Type> {
        match self {
            Type::TypeVariable(var) | Type::NamedGeneric(var, _) => {
                if let TypeBinding::Bound(typ) = &*var.borrow() {
                    return Cow::Owned(typ.follow_bindings_shallow().into_owned());
                }
                Cow::Borrowed(self)
            }
            other => Cow::Borrowed(other),
        }
    }

    pub fn from_generics(generics: &GenericTypeVars) -> Vec<Type> {
        vecmap(generics, |var| Type::TypeVariable(var.clone()))
    }

    /// Replace any `Type::NamedGeneric` in this type with a `Type::TypeVariable`
    /// using to the same inner `TypeVariable`. This is used during monomorphization
    /// to bind to named generics since they are unbindable during type checking.
    pub fn replace_named_generics_with_type_variables(&mut self) {
        match self {
            Type::FieldElement
            | Type::Constant(_, _)
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Unit
            | Type::Error
            | Type::Quoted(_) => (),

            Type::Array(len, elem) => {
                len.replace_named_generics_with_type_variables();
                elem.replace_named_generics_with_type_variables();
            }

            Type::Slice(elem) => elem.replace_named_generics_with_type_variables(),
            Type::String(len) => len.replace_named_generics_with_type_variables(),
            Type::FmtString(len, captures) => {
                len.replace_named_generics_with_type_variables();
                captures.replace_named_generics_with_type_variables();
            }
            Type::Tuple(fields) => {
                for field in fields {
                    field.replace_named_generics_with_type_variables();
                }
            }
            Type::Struct(_, generics) => {
                for generic in generics {
                    generic.replace_named_generics_with_type_variables();
                }
            }
            Type::Alias(alias, generics) => {
                let mut typ = alias.borrow().get_type(generics);
                typ.replace_named_generics_with_type_variables();
                *self = typ;
            }
            Type::TypeVariable(var) => {
                let var = var.borrow();
                if let TypeBinding::Bound(binding) = &*var {
                    let binding = binding.clone();
                    drop(var);
                    *self = binding;
                }
            }
            Type::TraitAsType(_, _, generics) => {
                for generic in &mut generics.ordered {
                    generic.replace_named_generics_with_type_variables();
                }
                for generic in &mut generics.named {
                    generic.typ.replace_named_generics_with_type_variables();
                }
            }
            Type::CheckedCast { from, to } => {
                from.replace_named_generics_with_type_variables();
                to.replace_named_generics_with_type_variables();
            }
            Type::NamedGeneric(var, _) => {
                let type_binding = var.borrow();
                if let TypeBinding::Bound(binding) = &*type_binding {
                    let mut binding = binding.clone();
                    drop(type_binding);
                    binding.replace_named_generics_with_type_variables();
                    *self = binding;
                } else {
                    drop(type_binding);
                    *self = Type::TypeVariable(var.clone());
                }
            }
            Type::Function(args, ret, env, _unconstrained) => {
                for arg in args {
                    arg.replace_named_generics_with_type_variables();
                }
                ret.replace_named_generics_with_type_variables();
                env.replace_named_generics_with_type_variables();
            }
            Type::MutableReference(elem) => elem.replace_named_generics_with_type_variables(),
            Type::Forall(_, typ) => typ.replace_named_generics_with_type_variables(),
            Type::InfixExpr(lhs, _op, rhs) => {
                lhs.replace_named_generics_with_type_variables();
                rhs.replace_named_generics_with_type_variables();
            }
        }
    }

    pub fn slice_element_type(&self) -> Option<&Type> {
        match self {
            Type::Slice(element) => Some(element),
            _ => None,
        }
    }

    pub(crate) fn integral_maximum_size(&self) -> Option<FieldElement> {
        match self {
            Type::FieldElement => None,
            Type::Integer(sign, num_bits) => {
                let mut max_bit_size = num_bits.bit_size();
                if sign == &Signedness::Signed {
                    max_bit_size -= 1;
                }
                Some(((1u128 << max_bit_size) - 1).into())
            }
            Type::Bool => Some(FieldElement::one()),
            Type::TypeVariable(var) => {
                let binding = &var.1;
                match &*binding.borrow() {
                    TypeBinding::Unbound(_, type_var_kind) => match type_var_kind {
                        Kind::Any | Kind::Normal | Kind::Integer | Kind::IntegerOrField => None,
                        Kind::Numeric(typ) => typ.integral_maximum_size(),
                    },
                    TypeBinding::Bound(typ) => typ.integral_maximum_size(),
                }
            }
            Type::Alias(alias, args) => alias.borrow().get_type(args).integral_maximum_size(),
            Type::CheckedCast { to, .. } => to.integral_maximum_size(),
            Type::NamedGeneric(binding, _name) => match &*binding.borrow() {
                TypeBinding::Bound(typ) => typ.integral_maximum_size(),
                TypeBinding::Unbound(_, kind) => kind.integral_maximum_size(),
            },
            Type::MutableReference(typ) => typ.integral_maximum_size(),
            Type::InfixExpr(lhs, _op, rhs) => lhs.infix_kind(rhs).integral_maximum_size(),
            Type::Constant(_, kind) => kind.integral_maximum_size(),

            Type::Array(..)
            | Type::Slice(..)
            | Type::String(..)
            | Type::FmtString(..)
            | Type::Unit
            | Type::Tuple(..)
            | Type::Struct(..)
            | Type::TraitAsType(..)
            | Type::Function(..)
            | Type::Forall(..)
            | Type::Quoted(..)
            | Type::Error => None,
        }
    }
}

/// Wraps a given `expression` in `expression.as_slice()`
fn convert_array_expression_to_slice(
    expression: ExprId,
    array_type: Type,
    target_type: Type,
    as_slice_method: crate::node_interner::FuncId,
    interner: &mut NodeInterner,
) {
    let as_slice_id = interner.function_definition_id(as_slice_method);
    let location = interner.expr_location(&expression);
    let as_slice = HirExpression::Ident(HirIdent::non_trait_method(as_slice_id, location), None);
    let func = interner.push_expr(as_slice);

    // Copy the expression and give it a new ExprId. The old one
    // will be mutated in place into a Call expression.
    let argument = interner.expression(&expression);
    let argument = interner.push_expr(argument);
    interner.push_expr_type(argument, array_type.clone());
    interner.push_expr_location(argument, location.span, location.file);

    let arguments = vec![argument];
    let is_macro_call = false;
    let call = HirExpression::Call(HirCallExpression { func, arguments, location, is_macro_call });
    interner.replace_expr(&expression, call);

    interner.push_expr_location(func, location.span, location.file);
    interner.push_expr_type(expression, target_type.clone());

    let func_type =
        Type::Function(vec![array_type], Box::new(target_type), Box::new(Type::Unit), false);
    interner.push_expr_type(func, func_type);
}

impl BinaryTypeOperator {
    /// Perform the actual rust numeric operation associated with this operator
    pub fn function(
        self,
        a: FieldElement,
        b: FieldElement,
        kind: &Kind,
        span: Span,
    ) -> Result<FieldElement, TypeCheckError> {
        match kind.follow_bindings().integral_maximum_size() {
            None => match self {
                BinaryTypeOperator::Addition => Ok(a + b),
                BinaryTypeOperator::Subtraction => Ok(a - b),
                BinaryTypeOperator::Multiplication => Ok(a * b),
                BinaryTypeOperator::Division => (b != FieldElement::zero())
                    .then(|| a / b)
                    .ok_or(TypeCheckError::DivisionByZero { lhs: a, rhs: b, span }),
                BinaryTypeOperator::Modulo => {
                    Err(TypeCheckError::ModuloOnFields { lhs: a, rhs: b, span })
                }
            },
            Some(_maximum_size) => {
                let a = a.to_i128();
                let b = b.to_i128();

                let err = TypeCheckError::FailingBinaryOp { op: self, lhs: a, rhs: b, span };
                let result = match self {
                    BinaryTypeOperator::Addition => a.checked_add(b).ok_or(err)?,
                    BinaryTypeOperator::Subtraction => a.checked_sub(b).ok_or(err)?,
                    BinaryTypeOperator::Multiplication => a.checked_mul(b).ok_or(err)?,
                    BinaryTypeOperator::Division => a.checked_div(b).ok_or(err)?,
                    BinaryTypeOperator::Modulo => a.checked_rem(b).ok_or(err)?,
                };

                Ok(result.into())
            }
        }
    }

    fn is_commutative(self) -> bool {
        matches!(self, BinaryTypeOperator::Addition | BinaryTypeOperator::Multiplication)
    }

    /// Return the operator that will "undo" this operation if applied to the rhs
    fn inverse(self) -> Option<BinaryTypeOperator> {
        match self {
            BinaryTypeOperator::Addition => Some(BinaryTypeOperator::Subtraction),
            BinaryTypeOperator::Subtraction => Some(BinaryTypeOperator::Addition),
            BinaryTypeOperator::Multiplication => None,
            BinaryTypeOperator::Division => None,
            BinaryTypeOperator::Modulo => None,
        }
    }

    /// Return the operator that will "undo" this operation if applied to the rhs
    fn approx_inverse(self) -> Option<BinaryTypeOperator> {
        match self {
            BinaryTypeOperator::Addition => Some(BinaryTypeOperator::Subtraction),
            BinaryTypeOperator::Subtraction => Some(BinaryTypeOperator::Addition),
            BinaryTypeOperator::Multiplication => Some(BinaryTypeOperator::Division),
            BinaryTypeOperator::Division => Some(BinaryTypeOperator::Multiplication),
            BinaryTypeOperator::Modulo => None,
        }
    }
}

impl From<Type> for PrintableType {
    fn from(value: Type) -> Self {
        Self::from(&value)
    }
}

impl From<&Type> for PrintableType {
    fn from(value: &Type) -> Self {
        // Note; use strict_eq instead of partial_eq when comparing field types
        // in this method, you most likely want to distinguish between public and private
        match value {
            Type::FieldElement => PrintableType::Field,
            Type::Array(size, typ) => {
                let dummy_span = Span::default();
                let length =
                    size.evaluate_to_u32(dummy_span).expect("Cannot print variable sized arrays");
                let typ = typ.as_ref();
                PrintableType::Array { length, typ: Box::new(typ.into()) }
            }
            Type::Slice(typ) => {
                let typ = typ.as_ref();
                PrintableType::Slice { typ: Box::new(typ.into()) }
            }
            Type::Integer(sign, bit_width) => match sign {
                Signedness::Unsigned => {
                    PrintableType::UnsignedInteger { width: (*bit_width).into() }
                }
                Signedness::Signed => PrintableType::SignedInteger { width: (*bit_width).into() },
            },
            Type::TypeVariable(binding) => match &*binding.borrow() {
                TypeBinding::Bound(typ) => typ.into(),
                TypeBinding::Unbound(_, Kind::Integer) => Type::default_int_type().into(),
                TypeBinding::Unbound(_, Kind::IntegerOrField) => {
                    Type::default_int_or_field_type().into()
                }
                TypeBinding::Unbound(_, Kind::Numeric(typ)) => (*typ.clone()).into(),
                TypeBinding::Unbound(_, Kind::Any | Kind::Normal) => unreachable!(),
            },
            Type::Bool => PrintableType::Boolean,
            Type::String(size) => {
                let dummy_span = Span::default();
                let size =
                    size.evaluate_to_u32(dummy_span).expect("Cannot print variable sized strings");
                PrintableType::String { length: size }
            }
            Type::FmtString(_, _) => unreachable!("format strings cannot be printed"),
            Type::Error => unreachable!(),
            Type::Unit => PrintableType::Unit,
            Type::Constant(_, _) => unreachable!(),
            Type::Struct(def, ref args) => {
                let struct_type = def.borrow();
                let fields = struct_type.get_fields(args);
                let fields = vecmap(fields, |(name, typ)| (name, typ.into()));
                PrintableType::Struct { fields, name: struct_type.name.to_string() }
            }
            Type::Alias(alias, args) => alias.borrow().get_type(args).into(),
            Type::TraitAsType(..) => unreachable!(),
            Type::Tuple(types) => PrintableType::Tuple { types: vecmap(types, |typ| typ.into()) },
            Type::CheckedCast { to, .. } => to.as_ref().into(),
            Type::NamedGeneric(..) => unreachable!(),
            Type::Forall(..) => unreachable!(),
            Type::Function(arguments, return_type, env, unconstrained) => PrintableType::Function {
                arguments: arguments.iter().map(|arg| arg.into()).collect(),
                return_type: Box::new(return_type.as_ref().into()),
                env: Box::new(env.as_ref().into()),
                unconstrained: *unconstrained,
            },
            Type::MutableReference(typ) => {
                PrintableType::MutableReference { typ: Box::new(typ.as_ref().into()) }
            }
            Type::Quoted(_) => unreachable!(),
            Type::InfixExpr(..) => unreachable!(),
        }
    }
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::FieldElement => {
                write!(f, "Field")
            }
            Type::Array(len, typ) => {
                write!(f, "[{typ:?}; {len:?}]")
            }
            Type::Slice(typ) => {
                write!(f, "[{typ:?}]")
            }
            Type::Integer(sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "i{num_bits}"),
                Signedness::Unsigned => write!(f, "u{num_bits}"),
            },
            Type::TypeVariable(var) => {
                let binding = &var.1;
                if let TypeBinding::Unbound(_, type_var_kind) = &*binding.borrow() {
                    match type_var_kind {
                        Kind::Any | Kind::Normal => write!(f, "{:?}", var),
                        Kind::IntegerOrField => write!(f, "IntOrField{:?}", binding),
                        Kind::Integer => write!(f, "Int{:?}", binding),
                        Kind::Numeric(typ) => write!(f, "Numeric({:?}: {:?})", binding, typ),
                    }
                } else {
                    write!(f, "{}", binding.borrow())
                }
            }
            Type::Struct(s, args) => {
                let args = vecmap(args, |arg| format!("{:?}", arg));
                if args.is_empty() {
                    write!(f, "{}", s.borrow())
                } else {
                    write!(f, "{}<{}>", s.borrow(), args.join(", "))
                }
            }
            Type::Alias(alias, args) => {
                let args = vecmap(args, |arg| format!("{:?}", arg));
                if args.is_empty() {
                    write!(f, "{}", alias.borrow())
                } else {
                    write!(f, "{}<{}>", alias.borrow(), args.join(", "))
                }
            }
            Type::TraitAsType(_id, name, generics) => write!(f, "impl {}{:?}", name, generics),
            Type::Tuple(elements) => {
                let elements = vecmap(elements, |arg| format!("{:?}", arg));
                write!(f, "({})", elements.join(", "))
            }
            Type::Bool => write!(f, "bool"),
            Type::String(len) => write!(f, "str<{len:?}>"),
            Type::FmtString(len, elements) => {
                write!(f, "fmtstr<{len:?}, {elements:?}>")
            }
            Type::Unit => write!(f, "()"),
            Type::Error => write!(f, "error"),
            Type::CheckedCast { to, .. } => write!(f, "{:?}", to),
            Type::NamedGeneric(binding, name) => match binding.kind() {
                Kind::Any | Kind::Normal | Kind::Integer | Kind::IntegerOrField => {
                    write!(f, "{}{:?}", name, binding)
                }
                Kind::Numeric(typ) => {
                    write!(f, "({} : {}){:?}", name, typ, binding)
                }
            },
            Type::Constant(x, kind) => write!(f, "({}: {})", x, kind),
            Type::Forall(typevars, typ) => {
                let typevars = vecmap(typevars, |var| format!("{:?}", var));
                write!(f, "forall {}. {:?}", typevars.join(" "), typ)
            }
            Type::Function(args, ret, env, unconstrained) => {
                if *unconstrained {
                    write!(f, "unconstrained ")?;
                }

                let closure_env_text = match **env {
                    Type::Unit => "".to_string(),
                    _ => format!(" with env {env:?}"),
                };

                let args = vecmap(args.iter(), |arg| format!("{:?}", arg));

                write!(f, "fn({}) -> {ret:?}{closure_env_text}", args.join(", "))
            }
            Type::MutableReference(element) => {
                write!(f, "&mut {element:?}")
            }
            Type::Quoted(quoted) => write!(f, "{}", quoted),
            Type::InfixExpr(lhs, op, rhs) => write!(f, "({lhs:?} {op} {rhs:?})"),
        }
    }
}

impl std::fmt::Debug for TypeVariableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}", self.0)
    }
}

impl std::fmt::Debug for TypeVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id())?;

        if let TypeBinding::Bound(typ) = &*self.borrow() {
            write!(f, " -> {typ:?}")?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for StructType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::hash::Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Some((variable, kind)) = self.get_inner_type_variable() {
            kind.hash(state);
            if let TypeBinding::Bound(typ) = &*variable.borrow() {
                typ.hash(state);
                return;
            }
        }

        if !matches!(self, Type::TypeVariable(..) | Type::NamedGeneric(..)) {
            std::mem::discriminant(self).hash(state);
        }

        match self {
            Type::FieldElement | Type::Bool | Type::Unit | Type::Error => (),
            Type::Array(len, elem) => {
                len.hash(state);
                elem.hash(state);
            }
            Type::Slice(elem) => elem.hash(state),
            Type::Integer(sign, bits) => {
                sign.hash(state);
                bits.hash(state);
            }
            Type::String(len) => len.hash(state),
            Type::FmtString(len, env) => {
                len.hash(state);
                env.hash(state);
            }
            Type::Tuple(elems) => elems.hash(state),
            Type::Struct(def, args) => {
                def.hash(state);
                args.hash(state);
            }
            Type::Alias(alias, args) => {
                alias.hash(state);
                args.hash(state);
            }
            Type::TypeVariable(var) | Type::NamedGeneric(var, ..) => var.hash(state),
            Type::TraitAsType(trait_id, _, args) => {
                trait_id.hash(state);
                args.hash(state);
            }
            Type::Function(args, ret, env, is_unconstrained) => {
                args.hash(state);
                ret.hash(state);
                env.hash(state);
                is_unconstrained.hash(state);
            }
            Type::MutableReference(elem) => elem.hash(state),
            Type::Forall(vars, typ) => {
                vars.hash(state);
                typ.hash(state);
            }
            Type::CheckedCast { to, .. } => to.hash(state),
            Type::Constant(value, _) => value.hash(state),
            Type::Quoted(typ) => typ.hash(state),
            Type::InfixExpr(lhs, op, rhs) => {
                lhs.hash(state);
                op.hash(state);
                rhs.hash(state);
            }
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        if let Some((variable, kind)) = self.get_inner_type_variable() {
            if kind != other.kind() {
                return false;
            }
            if let TypeBinding::Bound(typ) = &*variable.borrow() {
                return typ == other;
            }
        }

        if let Some((variable, other_kind)) = other.get_inner_type_variable() {
            if self.kind() != other_kind {
                return false;
            }
            if let TypeBinding::Bound(typ) = &*variable.borrow() {
                return self == typ;
            }
        }

        use Type::*;
        match (self, other) {
            (FieldElement, FieldElement) | (Bool, Bool) | (Unit, Unit) | (Error, Error) => true,
            (Array(lhs_len, lhs_elem), Array(rhs_len, rhs_elem)) => {
                lhs_len == rhs_len && lhs_elem == rhs_elem
            }
            (Slice(lhs_elem), Slice(rhs_elem)) => lhs_elem == rhs_elem,
            (Integer(lhs_sign, lhs_bits), Integer(rhs_sign, rhs_bits)) => {
                lhs_sign == rhs_sign && lhs_bits == rhs_bits
            }
            (String(lhs_len), String(rhs_len)) => lhs_len == rhs_len,
            (FmtString(lhs_len, lhs_env), FmtString(rhs_len, rhs_env)) => {
                lhs_len == rhs_len && lhs_env == rhs_env
            }
            (Tuple(lhs_types), Tuple(rhs_types)) => lhs_types == rhs_types,
            (Struct(lhs_struct, lhs_generics), Struct(rhs_struct, rhs_generics)) => {
                lhs_struct == rhs_struct && lhs_generics == rhs_generics
            }
            (Alias(lhs_alias, lhs_generics), Alias(rhs_alias, rhs_generics)) => {
                lhs_alias == rhs_alias && lhs_generics == rhs_generics
            }
            (TraitAsType(lhs_trait, _, lhs_generics), TraitAsType(rhs_trait, _, rhs_generics)) => {
                lhs_trait == rhs_trait && lhs_generics == rhs_generics
            }
            (
                Function(lhs_args, lhs_ret, lhs_env, lhs_unconstrained),
                Function(rhs_args, rhs_ret, rhs_env, rhs_unconstrained),
            ) => {
                let args_and_ret_eq = lhs_args == rhs_args && lhs_ret == rhs_ret;
                args_and_ret_eq && lhs_env == rhs_env && lhs_unconstrained == rhs_unconstrained
            }
            (MutableReference(lhs_elem), MutableReference(rhs_elem)) => lhs_elem == rhs_elem,
            (Forall(lhs_vars, lhs_type), Forall(rhs_vars, rhs_type)) => {
                lhs_vars == rhs_vars && lhs_type == rhs_type
            }
            (CheckedCast { to, .. }, other) | (other, CheckedCast { to, .. }) => **to == *other,
            (Constant(lhs, lhs_kind), Constant(rhs, rhs_kind)) => {
                lhs == rhs && lhs_kind == rhs_kind
            }
            (Quoted(lhs), Quoted(rhs)) => lhs == rhs,
            (InfixExpr(l_lhs, l_op, l_rhs), InfixExpr(r_lhs, r_op, r_rhs)) => {
                l_lhs == r_lhs && l_op == r_op && l_rhs == r_rhs
            }
            // Special case: we consider unbound named generics and type variables to be equal to each
            // other if their type variable ids match. This is important for some corner cases in
            // monomorphization where we call `replace_named_generics_with_type_variables` but
            // still want them to be equal for canonicalization checks in arithmetic generics.
            // Without this we'd fail the `serialize` test.
            (
                NamedGeneric(lhs_var, _) | TypeVariable(lhs_var),
                NamedGeneric(rhs_var, _) | TypeVariable(rhs_var),
            ) => lhs_var.id() == rhs_var.id(),
            _ => false,
        }
    }
}
