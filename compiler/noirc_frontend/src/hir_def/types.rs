use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use crate::{
    ast::IntegerBitSize,
    hir::type_check::TypeCheckError,
    node_interner::{ExprId, NodeInterner, TraitId, TypeAliasId},
};
use iter_extended::vecmap;
use noirc_errors::{Location, Span};
use noirc_printable_type::PrintableType;

use crate::{
    ast::{Ident, Signedness},
    node_interner::StructId,
};

use super::expr::{HirCallExpression, HirExpression, HirIdent};

#[derive(PartialEq, Eq, Clone, Hash)]
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
    TypeVariable(TypeVariable, TypeVariableKind),

    /// `impl Trait` when used in a type position.
    /// These are only matched based on the TraitId. The trait name parameter is only
    /// used for displaying error messages using the name of the trait.
    TraitAsType(TraitId, /*name:*/ Rc<String>, /*generics:*/ Vec<Type>),

    /// NamedGenerics are the 'T' or 'U' in a user-defined generic function
    /// like `fn foo<T, U>(...) {}`. Unlike TypeVariables, they cannot be bound over.
    NamedGeneric(TypeVariable, Rc<String>, Kind),

    /// A functions with arguments, a return type and environment.
    /// the environment should be `Unit` by default,
    /// for closures it should contain a `Tuple` type with the captured
    /// variable types.
    Function(Vec<Type>, /*return_type:*/ Box<Type>, /*environment:*/ Box<Type>),

    /// &mut T
    MutableReference(Box<Type>),

    /// A type generic over the given type variables.
    /// Storing both the TypeVariableId and TypeVariable isn't necessary
    /// but it makes handling them both easier. The TypeVariableId should
    /// never be bound over during type checking, but during monomorphization it
    /// will be and thus needs the full TypeVariable link.
    Forall(GenericTypeVars, Box<Type>),

    /// A type-level integer. Included to let an Array's size type variable
    /// bind to an integer without special checks to bind it to a non-type.
    Constant(u32),

    /// The type of quoted code in macros. This is always a comptime-only type
    Quoted(QuotedType),

    /// The result of some type error. Remembering type errors as their own type variant lets
    /// us avoid issuing repeat type errors for the same item. For example, a lambda with
    /// an invalid type would otherwise issue a new error each time it is called
    /// if not for this variant.
    Error,
}

impl Type {
    /// Returns the number of field elements required to represent the type once encoded.
    pub fn field_count(&self) -> u32 {
        match self {
            Type::FieldElement | Type::Integer { .. } | Type::Bool => 1,
            Type::Array(size, typ) => {
                let length = size
                    .evaluate_to_u32()
                    .expect("Cannot have variable sized arrays as a parameter to main");
                let typ = typ.as_ref();
                length * typ.field_count()
            }
            Type::Struct(def, args) => {
                let struct_type = def.borrow();
                let fields = struct_type.get_fields(args);
                fields.iter().fold(0, |acc, (_, field_type)| acc + field_type.field_count())
            }
            Type::Alias(def, generics) => def.borrow().get_type(generics).field_count(),
            Type::Tuple(fields) => {
                fields.iter().fold(0, |acc, field_typ| acc + field_typ.field_count())
            }
            Type::String(size) => size
                .evaluate_to_u32()
                .expect("Cannot have variable sized strings as a parameter to main"),
            Type::FmtString(_, _)
            | Type::Unit
            | Type::TypeVariable(_, _)
            | Type::TraitAsType(..)
            | Type::NamedGeneric(_, _, _)
            | Type::Function(_, _, _)
            | Type::MutableReference(_)
            | Type::Forall(_, _)
            | Type::Constant(_)
            | Type::Quoted(_)
            | Type::Slice(_)
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
}

/// A Kind is the type of a Type. These are used since only certain kinds of types are allowed in
/// certain positions.
///
/// For example, the type of a struct field or a function parameter is expected to be
/// a type of kind * (represented here as `Normal`). Types used in positions where a number
/// is expected (such as in an array length position) are expected to be of kind `Kind::Numeric`.
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum Kind {
    Normal,
    Numeric(Box<Type>),
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Normal => write!(f, "normal"),
            Kind::Numeric(typ) => write!(f, "numeric {}", typ),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum QuotedType {
    Expr,
    Quoted,
    TopLevelItem,
    Type,
    StructDefinition,
}

/// A list of TypeVariableIds to bind to a type. Storing the
/// TypeVariable in addition to the matching TypeVariableId allows
/// the binding to later be undone if needed.
pub type TypeBindings = HashMap<TypeVariableId, (TypeVariable, Type)>;

/// Represents a struct type in the type system. Each instance of this
/// rust struct will be shared across all Type::Struct variants that represent
/// the same struct type.
#[derive(Eq)]
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
    pub location: Location,
}

/// Corresponds to generic lists such as `<T, U>` in the source program.
/// Used mainly for resolved types which no longer need information such
/// as names or kinds.
pub type GenericTypeVars = Vec<TypeVariable>;

/// Corresponds to generic lists such as `<T, U>` with additional
/// information gathered during name resolution that is necessary
/// correctly resolving types.
pub type Generics = Vec<ResolvedGeneric>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGeneric {
    pub name: Rc<String>,
    pub type_var: TypeVariable,
    pub kind: Kind,
    pub span: Span,
}

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

        location: Location,
        fields: Vec<(Ident, Type)>,
        generics: Generics,
    ) -> StructType {
        StructType { id, fields, name, location, generics }
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
                    .map(|(old, new)| (old.type_var.id(), (old.type_var.clone(), new.clone())))
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
            .map(|(old, new)| (old.type_var.id(), (old.type_var.clone(), new.clone())))
            .collect();

        vecmap(&self.fields, |(name, typ)| {
            let name = name.0.contents.clone();
            (name, typ.substitute(&substitutions))
        })
    }

    /// Returns the name and raw types of each field of this type.
    /// This will not substitute any generic arguments so a generic field like `x`
    /// in `struct Foo<T> { x: T }` will return a `("x", T)` pair.
    ///
    /// This method is almost never what is wanted for type checking or monomorphization,
    /// prefer to use `get_fields` whenever possible.
    pub fn get_fields_as_written(&self) -> Vec<(String, Type)> {
        vecmap(&self.fields, |(name, typ)| (name.0.contents.clone(), typ.clone()))
    }

    /// Returns the field at the given index. Panics if no field exists at the given index.
    pub fn field_at(&self, index: usize) -> &(Ident, Type) {
        &self.fields[index]
    }

    pub fn field_names(&self) -> BTreeSet<Ident> {
        self.fields.iter().map(|(name, _)| name.clone()).collect()
    }

    /// Search the fields of a struct for any types with a `TypeKind::Numeric`
    pub fn find_numeric_generics_in_fields(&self, found_names: &mut Vec<String>) {
        for (_, field) in self.fields.iter() {
            field.find_numeric_type_vars(found_names);
        }
    }

    /// True if the given index is the same index as a generic type of this struct
    /// which is expected to be a numeric generic.
    /// This is needed because we infer type kinds in Noir and don't have extensive kind checking.
    /// TODO(https://github.com/noir-lang/noir/issues/5156): This is outdated and we should remove this implicit searching for numeric generics
    pub fn generic_is_numeric(&self, index_of_generic: usize) -> bool {
        let target_id = self.generics[index_of_generic].type_var.id();
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
            .map(|(old, new)| (old.type_var.id(), (old.type_var.clone(), new.clone())))
            .collect();

        self.typ.substitute(&substitutions)
    }

    /// True if the given index is the same index as a generic type of this alias
    /// which is expected to be a numeric generic.
    /// This is needed because we infer type kinds in Noir and don't have extensive kind checking.
    pub fn generic_is_numeric(&self, index_of_generic: usize) -> bool {
        let target_id = self.generics[index_of_generic].type_var.id();
        self.typ.contains_numeric_typevar(target_id)
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
    IntegerOrField,

    /// A generic integer type. This is a more specific kind of TypeVariable
    /// that can only be bound to Type::Integer, or other polymorphic integers.
    Integer,

    /// A potentially constant array size. This will only bind to itself or
    /// Type::Constant(n) with a matching size. This defaults to Type::Constant(n) if still unbound
    /// during monomorphization.
    Constant(u32),
}

/// A TypeVariable is a mutable reference that is either
/// bound to some type, or unbound with a given TypeVariableId.
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct TypeVariable(TypeVariableId, Shared<TypeBinding>);

impl TypeVariable {
    pub fn unbound(id: TypeVariableId) -> Self {
        TypeVariable(id, Shared::new(TypeBinding::Unbound(id)))
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
            TypeBinding::Unbound(id) => *id,
        };

        assert!(!typ.occurs(id), "{self:?} occurs within {typ:?}");
        *self.1.borrow_mut() = TypeBinding::Bound(typ);
    }

    pub fn try_bind(&self, binding: Type, span: Span) -> Result<(), TypeCheckError> {
        let id = match &*self.1.borrow() {
            TypeBinding::Bound(binding) => {
                unreachable!("Expected unbound, found bound to {binding}")
            }
            TypeBinding::Unbound(id) => *id,
        };

        if binding.occurs(id) {
            Err(TypeCheckError::TypeAnnotationsNeeded { span })
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
    pub fn unbind(&self, id: TypeVariableId) {
        *self.1.borrow_mut() = TypeBinding::Unbound(id);
    }

    /// Forcibly bind a type variable to a new type - even if the type
    /// variable is already bound to a different type. This generally
    /// a logic error to use outside of monomorphization.
    pub fn force_bind(&self, typ: Type) {
        *self.1.borrow_mut() = TypeBinding::Bound(typ);
    }
}

/// TypeBindings are the mutable insides of a TypeVariable.
/// They are either bound to some type, or are unbound.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TypeBinding {
    Bound(Type),
    Unbound(TypeVariableId),
}

impl TypeBinding {
    pub fn is_unbound(&self) -> bool {
        matches!(self, TypeBinding::Unbound(_))
    }
}

/// A unique ID used to differentiate different type variables
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeVariableId(pub usize);

impl Type {
    pub fn default_int_or_field_type() -> Type {
        Type::FieldElement
    }

    pub fn default_int_type() -> Type {
        Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo)
    }

    pub fn type_variable(id: TypeVariableId) -> Type {
        let var = TypeVariable::unbound(id);
        Type::TypeVariable(var, TypeVariableKind::Normal)
    }

    /// Returns a TypeVariable(_, TypeVariableKind::Constant(length)) to bind to
    /// a constant integer for e.g. an array length.
    pub fn constant_variable(length: u32, interner: &mut NodeInterner) -> Type {
        let id = interner.next_type_variable_id();
        let kind = TypeVariableKind::Constant(length);
        let var = TypeVariable::unbound(id);
        Type::TypeVariable(var, kind)
    }

    pub fn polymorphic_integer_or_field(interner: &mut NodeInterner) -> Type {
        let id = interner.next_type_variable_id();
        let kind = TypeVariableKind::IntegerOrField;
        let var = TypeVariable::unbound(id);
        Type::TypeVariable(var, kind)
    }

    pub fn polymorphic_integer(interner: &mut NodeInterner) -> Type {
        let id = interner.next_type_variable_id();
        let kind = TypeVariableKind::Integer;
        let var = TypeVariable::unbound(id);
        Type::TypeVariable(var, kind)
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

    pub fn is_signed(&self) -> bool {
        matches!(self.follow_bindings(), Type::Integer(Signedness::Signed, _))
    }

    pub fn is_unsigned(&self) -> bool {
        matches!(self.follow_bindings(), Type::Integer(Signedness::Unsigned, _))
    }

    pub fn is_numeric(&self) -> bool {
        use Type::*;
        use TypeVariableKind as K;
        matches!(
            self.follow_bindings(),
            FieldElement | Integer(..) | Bool | TypeVariable(_, K::Integer | K::IntegerOrField)
        )
    }

    fn contains_numeric_typevar(&self, target_id: TypeVariableId) -> bool {
        // True if the given type is a NamedGeneric with the target_id
        let named_generic_id_matches_target = |typ: &Type| {
            if let Type::NamedGeneric(type_variable, _, _) = typ {
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
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Unit
            | Type::Error
            | Type::TypeVariable(_, _)
            | Type::Constant(_)
            | Type::NamedGeneric(_, _, _)
            | Type::Forall(_, _)
            | Type::Quoted(_) => false,

            Type::TraitAsType(_, _, args) => {
                args.iter().any(|generic| generic.contains_numeric_typevar(target_id))
            }
            Type::Array(length, elem) => {
                elem.contains_numeric_typevar(target_id) || named_generic_id_matches_target(length)
            }
            Type::Slice(elem) => elem.contains_numeric_typevar(target_id),
            Type::Tuple(fields) => {
                fields.iter().any(|field| field.contains_numeric_typevar(target_id))
            }
            Type::Function(parameters, return_type, env) => {
                parameters.iter().any(|parameter| parameter.contains_numeric_typevar(target_id))
                    || return_type.contains_numeric_typevar(target_id)
                    || env.contains_numeric_typevar(target_id)
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
            Type::Alias(alias, generics) => generics.iter().enumerate().any(|(i, generic)| {
                if named_generic_id_matches_target(generic) {
                    alias.borrow().generic_is_numeric(i)
                } else {
                    generic.contains_numeric_typevar(target_id)
                }
            }),
            Type::MutableReference(element) => element.contains_numeric_typevar(target_id),
            Type::String(length) => named_generic_id_matches_target(length),
            Type::FmtString(length, elements) => {
                elements.contains_numeric_typevar(target_id)
                    || named_generic_id_matches_target(length)
            }
        }
    }

    /// TODO(https://github.com/noir-lang/noir/issues/5156): Remove with explicit numeric generics
    pub fn find_numeric_type_vars(&self, found_names: &mut Vec<String>) {
        // Return whether the named generic has a TypeKind::Numeric and save its name
        let named_generic_is_numeric = |typ: &Type, found_names: &mut Vec<String>| {
            if let Type::NamedGeneric(_, name, Kind::Numeric { .. }) = typ {
                found_names.push(name.to_string());
                true
            } else {
                false
            }
        };

        match self {
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Unit
            | Type::Error
            | Type::Constant(_)
            | Type::Forall(_, _)
            | Type::Quoted(_) => {}

            Type::TypeVariable(type_var, _) => {
                if let TypeBinding::Bound(typ) = &*type_var.borrow() {
                    named_generic_is_numeric(typ, found_names);
                }
            }

            Type::NamedGeneric(_, _, _) => {
                named_generic_is_numeric(self, found_names);
            }

            Type::TraitAsType(_, _, args) => {
                for arg in args.iter() {
                    arg.find_numeric_type_vars(found_names);
                }
            }
            Type::Array(length, elem) => {
                elem.find_numeric_type_vars(found_names);
                named_generic_is_numeric(length, found_names);
            }
            Type::Slice(elem) => elem.find_numeric_type_vars(found_names),
            Type::Tuple(fields) => {
                for field in fields.iter() {
                    field.find_numeric_type_vars(found_names);
                }
            }
            Type::Function(parameters, return_type, env) => {
                for parameter in parameters.iter() {
                    parameter.find_numeric_type_vars(found_names);
                }
                return_type.find_numeric_type_vars(found_names);
                env.find_numeric_type_vars(found_names);
            }
            Type::Struct(_, generics) => {
                for generic in generics.iter() {
                    if !named_generic_is_numeric(generic, found_names) {
                        generic.find_numeric_type_vars(found_names);
                    }
                }
            }
            Type::Alias(_, generics) => {
                for generic in generics.iter() {
                    if !named_generic_is_numeric(generic, found_names) {
                        generic.find_numeric_type_vars(found_names);
                    }
                }
            }
            Type::MutableReference(element) => element.find_numeric_type_vars(found_names),
            Type::String(length) => {
                named_generic_is_numeric(length, found_names);
            }
            Type::FmtString(length, elements) => {
                elements.find_numeric_type_vars(found_names);
                named_generic_is_numeric(length, found_names);
            }
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
            | Type::Constant(_)
            | Type::Error => true,

            Type::FmtString(_, _)
            | Type::TypeVariable(_, _)
            | Type::NamedGeneric(_, _, _)
            | Type::Function(_, _, _)
            | Type::MutableReference(_)
            | Type::Forall(_, _)
            | Type::Quoted(_)
            | Type::Slice(_)
            | Type::TraitAsType(..) => false,

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
            | Type::Constant(_)
            | Type::TypeVariable(_, _)
            | Type::NamedGeneric(_, _, _)
            | Type::Error => true,

            Type::FmtString(_, _)
            // To enable this we would need to determine the size of the closure outputs at compile-time.
            // This is possible as long as the output size is not dependent upon a witness condition.
            | Type::Function(_, _, _)
            | Type::Slice(_)
            | Type::MutableReference(_)
            | Type::Forall(_, _)
            // TODO: probably can allow code as it is all compile time
            | Type::Quoted(_)
            | Type::TraitAsType(..) => false,

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
            | Type::Constant(_)
            | Type::Slice(_)
            | Type::TypeVariable(_, _)
            | Type::NamedGeneric(_, _, _)
            | Type::Function(_, _, _)
            | Type::FmtString(_, _)
            | Type::Error => true,

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
            Type::TypeVariable(type_variable, _) | Type::NamedGeneric(type_variable, _, _) => {
                match &*type_variable.borrow() {
                    TypeBinding::Bound(binding) => binding.generic_count(),
                    TypeBinding::Unbound(_) => 0,
                }
            }
            _ => 0,
        }
    }

    /// Takes a monomorphic type and generalizes it over each of the type variables in the
    /// given type bindings, ignoring what each type variable is bound to in the TypeBindings.
    pub(crate) fn generalize_from_substitutions(self, type_bindings: TypeBindings) -> Type {
        let polymorphic_type_vars = vecmap(type_bindings, |(_, (type_var, _))| type_var);
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

    // TODO(https://github.com/noir-lang/noir/issues/5156): Bring back this method when we remove implicit numeric generics
    // It has been commented out as to not trigger clippy for an unused method
    // pub(crate) fn kind(&self) -> Kind {
    //     match self {
    //         Type::NamedGeneric(_, _, kind) => kind.clone(),
    //         Type::Constant(_) => Kind::Numeric(Box::new(Type::Integer(
    //             Signedness::Unsigned,
    //             IntegerBitSize::ThirtyTwo,
    //         ))),
    //         Type::FieldElement
    //         | Type::Array(_, _)
    //         | Type::Slice(_)
    //         | Type::Integer(_, _)
    //         | Type::Bool
    //         | Type::String(_)
    //         | Type::FmtString(_, _)
    //         | Type::Unit
    //         | Type::Tuple(_)
    //         | Type::Struct(_, _)
    //         | Type::Alias(_, _)
    //         | Type::TypeVariable(_, _)
    //         | Type::TraitAsType(_, _, _)
    //         | Type::Function(_, _, _)
    //         | Type::MutableReference(_)
    //         | Type::Forall(_, _)
    //         | Type::Quoted(_)
    //         | Type::Error => Kind::Normal,
    //     }
    // }
}

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
            Type::TypeVariable(var, TypeVariableKind::Normal) => write!(f, "{}", var.borrow()),
            Type::TypeVariable(binding, TypeVariableKind::Integer) => {
                if let TypeBinding::Unbound(_) = &*binding.borrow() {
                    write!(f, "{}", Type::default_int_type())
                } else {
                    write!(f, "{}", binding.borrow())
                }
            }
            Type::TypeVariable(binding, TypeVariableKind::IntegerOrField) => {
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
            Type::Alias(alias, args) => {
                let args = vecmap(args, |arg| arg.to_string());
                if args.is_empty() {
                    write!(f, "{}", alias.borrow())
                } else {
                    write!(f, "{}<{}>", alias.borrow(), args.join(", "))
                }
            }
            Type::TraitAsType(_id, name, generics) => {
                write!(f, "impl {}", name)?;
                if !generics.is_empty() {
                    let generics = vecmap(generics, ToString::to_string).join(", ");
                    write!(f, "<{generics}>")?;
                }
                Ok(())
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
            Type::NamedGeneric(binding, name, _) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.fmt(f),
                TypeBinding::Unbound(_) if name.is_empty() => write!(f, "_"),
                TypeBinding::Unbound(_) => write!(f, "{name}"),
            },
            Type::Constant(x) => x.fmt(f),
            Type::Forall(typevars, typ) => {
                let typevars = vecmap(typevars, |var| var.id().to_string());
                write!(f, "forall {}. {}", typevars.join(" "), typ)
            }
            Type::Function(args, ret, env) => {
                let closure_env_text = match **env {
                    Type::Unit => "".to_string(),
                    _ => format!(" with env {env}"),
                };

                let args = vecmap(args.iter(), ToString::to_string);

                write!(f, "fn({}) -> {ret}{closure_env_text}", args.join(", "))
            }
            Type::MutableReference(element) => {
                write!(f, "&mut {element}")
            }
            Type::Quoted(quoted) => write!(f, "{}", quoted),
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

impl std::fmt::Display for QuotedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotedType::Expr => write!(f, "Expr"),
            QuotedType::Quoted => write!(f, "Quoted"),
            QuotedType::TopLevelItem => write!(f, "TopLevelItem"),
            QuotedType::Type => write!(f, "Type"),
            QuotedType::StructDefinition => write!(f, "StructDefinition"),
        }
    }
}

pub struct UnificationError;

impl Type {
    /// Try to bind a MaybeConstant variable to self, succeeding if self is a Constant,
    /// MaybeConstant, or type variable. If successful, the binding is placed in the
    /// given TypeBindings map rather than linked immediately.
    fn try_bind_to_maybe_constant(
        &self,
        var: &TypeVariable,
        target_length: u32,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        let target_id = match &*var.borrow() {
            TypeBinding::Bound(_) => unreachable!(),
            TypeBinding::Unbound(id) => *id,
        };

        let this = self.substitute(bindings).follow_bindings();

        match &this {
            Type::Constant(length) if *length == target_length => {
                bindings.insert(target_id, (var.clone(), this));
                Ok(())
            }
            // A TypeVariable is less specific than a MaybeConstant, so we bind
            // to the other type variable instead.
            Type::TypeVariable(new_var, kind) => {
                let borrow = new_var.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => {
                        typ.try_bind_to_maybe_constant(var, target_length, bindings)
                    }
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(new_target_id) => match kind {
                        TypeVariableKind::Normal => {
                            let clone = Type::TypeVariable(
                                var.clone(),
                                TypeVariableKind::Constant(target_length),
                            );
                            bindings.insert(*new_target_id, (new_var.clone(), clone));
                            Ok(())
                        }
                        TypeVariableKind::Constant(length) if *length == target_length => {
                            let clone = Type::TypeVariable(
                                var.clone(),
                                TypeVariableKind::Constant(target_length),
                            );
                            bindings.insert(*new_target_id, (new_var.clone(), clone));
                            Ok(())
                        }
                        // *length != target_length
                        TypeVariableKind::Constant(_) => Err(UnificationError),
                        TypeVariableKind::IntegerOrField => Err(UnificationError),
                        TypeVariableKind::Integer => Err(UnificationError),
                    },
                }
            }
            _ => Err(UnificationError),
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
            TypeBinding::Unbound(id) => *id,
        };

        let this = self.substitute(bindings).follow_bindings();
        match &this {
            Type::Integer(..) => {
                bindings.insert(target_id, (var.clone(), this));
                Ok(())
            }
            Type::FieldElement if !only_integer => {
                bindings.insert(target_id, (var.clone(), this));
                Ok(())
            }
            Type::TypeVariable(self_var, TypeVariableKind::IntegerOrField) => {
                let borrow = self_var.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => {
                        typ.try_bind_to_polymorphic_int(var, bindings, only_integer)
                    }
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(new_target_id) => {
                        if only_integer {
                            // Integer is more specific than IntegerOrField so we bind the type
                            // variable to Integer instead.
                            let clone = Type::TypeVariable(var.clone(), TypeVariableKind::Integer);
                            bindings.insert(*new_target_id, (self_var.clone(), clone));
                        } else {
                            bindings.insert(target_id, (var.clone(), this.clone()));
                        }
                        Ok(())
                    }
                }
            }
            Type::TypeVariable(self_var, TypeVariableKind::Integer) => {
                let borrow = self_var.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => {
                        typ.try_bind_to_polymorphic_int(var, bindings, only_integer)
                    }
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(_) => {
                        bindings.insert(target_id, (var.clone(), this.clone()));
                        Ok(())
                    }
                }
            }
            Type::TypeVariable(self_var, TypeVariableKind::Normal) => {
                let borrow = self_var.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => {
                        typ.try_bind_to_polymorphic_int(var, bindings, only_integer)
                    }
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(new_target_id) => {
                        // Bind to the most specific type variable kind
                        let clone_kind = if only_integer {
                            TypeVariableKind::Integer
                        } else {
                            TypeVariableKind::IntegerOrField
                        };
                        let clone = Type::TypeVariable(var.clone(), clone_kind);
                        bindings.insert(*new_target_id, (self_var.clone(), clone));
                        Ok(())
                    }
                }
            }
            _ => Err(UnificationError),
        }
    }

    /// Try to bind the given type variable to self. Although the given type variable
    /// is expected to be of TypeVariableKind::Normal, this binding can still fail
    /// if the given type variable occurs within `self` as that would create a recursive type.
    ///
    /// If successful, the binding is placed in the
    /// given TypeBindings map rather than linked immediately.
    fn try_bind_to(
        &self,
        var: &TypeVariable,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        let target_id = match &*var.borrow() {
            TypeBinding::Bound(_) => unreachable!(),
            TypeBinding::Unbound(id) => *id,
        };

        let this = self.substitute(bindings).follow_bindings();
        if let Some(binding) = this.get_inner_type_variable() {
            match &*binding.borrow() {
                TypeBinding::Bound(typ) => return typ.try_bind_to(var, bindings),
                // Don't recursively bind the same id to itself
                TypeBinding::Unbound(id) if *id == target_id => return Ok(()),
                _ => (),
            }
        }

        // Check if the target id occurs within `this` before binding. Otherwise this could
        // cause infinitely recursive types
        if this.occurs(target_id) {
            Err(UnificationError)
        } else {
            bindings.insert(target_id, (var.clone(), this.clone()));
            Ok(())
        }
    }

    fn get_inner_type_variable(&self) -> Option<Shared<TypeBinding>> {
        match self {
            Type::TypeVariable(var, _) | Type::NamedGeneric(var, _, _) => Some(var.1.clone()),
            _ => None,
        }
    }

    /// Try to unify this type with another, setting any type variables found
    /// equal to the other type in the process. When comparing types, unification
    /// (including try_unify) are almost always preferred over Type::eq as unification
    /// will correctly handle generic types.
    pub fn unify(
        &self,
        expected: &Type,
        errors: &mut Vec<TypeCheckError>,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        let mut bindings = TypeBindings::new();

        match self.try_unify(expected, &mut bindings) {
            Ok(()) => {
                // Commit any type bindings on success
                Self::apply_type_bindings(bindings);
            }
            Err(UnificationError) => errors.push(make_error()),
        }
    }

    /// `try_unify` is a bit of a misnomer since although errors are not committed,
    /// any unified bindings are on success.
    pub fn try_unify(
        &self,
        other: &Type,
        bindings: &mut TypeBindings,
    ) -> Result<(), UnificationError> {
        use Type::*;
        use TypeVariableKind as Kind;

        match (self, other) {
            (Error, _) | (_, Error) => Ok(()),

            (Alias(alias, args), other) | (other, Alias(alias, args)) => {
                let alias = alias.borrow().get_type(args);
                alias.try_unify(other, bindings)
            }

            (TypeVariable(var, Kind::IntegerOrField), other)
            | (other, TypeVariable(var, Kind::IntegerOrField)) => {
                other.try_unify_to_type_variable(var, bindings, |bindings| {
                    let only_integer = false;
                    other.try_bind_to_polymorphic_int(var, bindings, only_integer)
                })
            }

            (TypeVariable(var, Kind::Integer), other)
            | (other, TypeVariable(var, Kind::Integer)) => {
                other.try_unify_to_type_variable(var, bindings, |bindings| {
                    let only_integer = true;
                    other.try_bind_to_polymorphic_int(var, bindings, only_integer)
                })
            }

            (TypeVariable(var, Kind::Normal), other) | (other, TypeVariable(var, Kind::Normal)) => {
                other.try_unify_to_type_variable(var, bindings, |bindings| {
                    other.try_bind_to(var, bindings)
                })
            }

            (TypeVariable(var, Kind::Constant(length)), other)
            | (other, TypeVariable(var, Kind::Constant(length))) => other
                .try_unify_to_type_variable(var, bindings, |bindings| {
                    other.try_bind_to_maybe_constant(var, *length, bindings)
                }),

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

            (NamedGeneric(binding, _, _), other) | (other, NamedGeneric(binding, _, _))
                if !binding.borrow().is_unbound() =>
            {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    link.try_unify(other, bindings)
                } else {
                    unreachable!("If guard ensures binding is bound")
                }
            }

            (NamedGeneric(binding_a, name_a, _), NamedGeneric(binding_b, name_b, _)) => {
                // Bound NamedGenerics are caught by the check above
                assert!(binding_a.borrow().is_unbound());
                assert!(binding_b.borrow().is_unbound());

                if name_a == name_b {
                    Ok(())
                } else {
                    Err(UnificationError)
                }
            }

            (Function(params_a, ret_a, env_a), Function(params_b, ret_b, env_b)) => {
                if params_a.len() == params_b.len() {
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
        // TypeVariableKind, there are different methods to check whether the variable can
        // bind to the given type or not.
        bind_variable: impl FnOnce(&mut TypeBindings) -> Result<(), UnificationError>,
    ) -> Result<(), UnificationError> {
        match &*type_variable.borrow() {
            // If it is already bound, unify against what it is bound to
            TypeBinding::Bound(link) => link.try_unify(self, bindings),
            TypeBinding::Unbound(id) => {
                // We may have already "bound" this type variable in this call to
                // try_unify, so check those bindings as well.
                match bindings.get(id) {
                    Some((_, binding)) => binding.clone().try_unify(self, bindings),

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
        interner: &mut NodeInterner,
        errors: &mut Vec<TypeCheckError>,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        let mut bindings = TypeBindings::new();

        if let Err(UnificationError) = self.try_unify(expected, &mut bindings) {
            if !self.try_array_to_slice_coercion(expected, expression, interner) {
                errors.push(make_error());
            }
        } else {
            Type::apply_type_bindings(bindings);
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
            // Still have to ensure the element types match.
            // Don't need to issue an error here if not, it will be done in unify_with_coercions
            let mut bindings = TypeBindings::new();
            if element1.try_unify(element2, &mut bindings).is_ok() {
                convert_array_expression_to_slice(expression, this, target, interner);
                Self::apply_type_bindings(bindings);
                return true;
            }
        }
        false
    }

    /// Apply the given type bindings, making them permanently visible for each
    /// clone of each type variable bound.
    pub fn apply_type_bindings(bindings: TypeBindings) {
        for (type_variable, binding) in bindings.values() {
            type_variable.bind(binding.clone());
        }
    }

    /// If this type is a Type::Constant (used in array lengths), or is bound
    /// to a Type::Constant, return the constant as a u32.
    pub fn evaluate_to_u32(&self) -> Option<u32> {
        if let Some(binding) = self.get_inner_type_variable() {
            if let TypeBinding::Bound(binding) = &*binding.borrow() {
                return binding.evaluate_to_u32();
            }
        }

        match self {
            Type::TypeVariable(_, TypeVariableKind::Constant(size)) => Some(*size),
            Type::Array(len, _elem) => len.evaluate_to_u32(),
            Type::Constant(x) => Some(*x),
            _ => None,
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
    pub fn get_field_type(&self, field_name: &str) -> Option<Type> {
        match self {
            Type::Struct(def, args) => def.borrow().get_field(field_name, args).map(|(typ, _)| typ),
            Type::Tuple(fields) => {
                let mut fields = fields.iter().enumerate();
                fields.find(|(i, _)| i.to_string() == *field_name).map(|(_, typ)| typ).cloned()
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
                    bindings
                        .entry(var.id())
                        .or_insert_with(|| (var.clone(), interner.next_type_variable()));
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
                        let new = interner.next_type_variable();
                        (var.id(), (var.clone(), new))
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

                let replacements = typevars
                    .iter()
                    .enumerate()
                    .map(|(i, var)| {
                        let binding = if i < implicit_generic_count {
                            interner.next_type_variable()
                        } else {
                            types[i - implicit_generic_count].clone()
                        };
                        (var.id(), (var.clone(), binding))
                    })
                    .collect();

                let instantiated = typ.substitute(&replacements);
                (instantiated, replacements)
            }
            other => (other.clone(), HashMap::new()),
        }
    }

    fn type_variable_id(&self) -> Option<TypeVariableId> {
        match self {
            Type::TypeVariable(variable, _) | Type::NamedGeneric(variable, _, _) => {
                Some(variable.0)
            }
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
                Some((_, replacement)) if substitute_bound_typevars => {
                    recur_on_binding(binding.0, replacement)
                }
                _ => match &*binding.borrow() {
                    TypeBinding::Bound(binding) => {
                        binding.substitute_helper(type_bindings, substitute_bound_typevars)
                    }
                    TypeBinding::Unbound(id) => match type_bindings.get(id) {
                        Some((_, replacement)) => recur_on_binding(binding.0, replacement),
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
            Type::NamedGeneric(binding, _, _) | Type::TypeVariable(binding, _) => {
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
            Type::Function(args, ret, env) => {
                let args = vecmap(args, |arg| {
                    arg.substitute_helper(type_bindings, substitute_bound_typevars)
                });
                let ret = Box::new(ret.substitute_helper(type_bindings, substitute_bound_typevars));
                let env = Box::new(env.substitute_helper(type_bindings, substitute_bound_typevars));
                Type::Function(args, ret, env)
            }
            Type::MutableReference(element) => Type::MutableReference(Box::new(
                element.substitute_helper(type_bindings, substitute_bound_typevars),
            )),

            Type::TraitAsType(s, name, args) => {
                let args = vecmap(args, |arg| {
                    arg.substitute_helper(type_bindings, substitute_bound_typevars)
                });
                Type::TraitAsType(*s, name.clone(), args)
            }

            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Constant(_)
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
            Type::Struct(_, generic_args)
            | Type::Alias(_, generic_args)
            | Type::TraitAsType(_, _, generic_args) => {
                generic_args.iter().any(|arg| arg.occurs(target_id))
            }
            Type::Tuple(fields) => fields.iter().any(|field| field.occurs(target_id)),
            Type::NamedGeneric(binding, _, _) | Type::TypeVariable(binding, _) => {
                match &*binding.borrow() {
                    TypeBinding::Bound(binding) => binding.occurs(target_id),
                    TypeBinding::Unbound(id) => *id == target_id,
                }
            }
            Type::Forall(typevars, typ) => {
                !typevars.iter().any(|var| var.id() == target_id) && typ.occurs(target_id)
            }
            Type::Function(args, ret, env) => {
                args.iter().any(|arg| arg.occurs(target_id))
                    || ret.occurs(target_id)
                    || env.occurs(target_id)
            }
            Type::MutableReference(element) => element.occurs(target_id),

            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Constant(_)
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
            TypeVariable(var, _) | NamedGeneric(var, _, _) => {
                if let TypeBinding::Bound(typ) = &*var.borrow() {
                    return typ.follow_bindings();
                }
                self.clone()
            }

            Function(args, ret, env) => {
                let args = vecmap(args, |arg| arg.follow_bindings());
                let ret = Box::new(ret.follow_bindings());
                let env = Box::new(env.follow_bindings());
                Function(args, ret, env)
            }

            MutableReference(element) => MutableReference(Box::new(element.follow_bindings())),

            TraitAsType(s, name, args) => {
                let args = vecmap(args, |arg| arg.follow_bindings());
                TraitAsType(*s, name.clone(), args)
            }

            // Expect that this function should only be called on instantiated types
            Forall(..) => unreachable!(),
            FieldElement | Integer(_, _) | Bool | Constant(_) | Unit | Quoted(_) | Error => {
                self.clone()
            }
        }
    }

    pub fn from_generics(generics: &GenericTypeVars) -> Vec<Type> {
        vecmap(generics, |var| Type::TypeVariable(var.clone(), TypeVariableKind::Normal))
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
    let as_slice = HirExpression::Ident(HirIdent::non_trait_method(as_slice_id, location), None);
    let func = interner.push_expr(as_slice);

    // Copy the expression and give it a new ExprId. The old one
    // will be mutated in place into a Call expression.
    let argument = interner.expression(&expression);
    let argument = interner.push_expr(argument);
    interner.push_expr_type(argument, array_type.clone());
    interner.push_expr_location(argument, location.span, location.file);

    let arguments = vec![argument];
    let call = HirExpression::Call(HirCallExpression { func, arguments, location });
    interner.replace_expr(&expression, call);

    interner.push_expr_location(func, location.span, location.file);
    interner.push_expr_type(expression, target_type.clone());

    let func_type = Type::Function(vec![array_type], Box::new(target_type), Box::new(Type::Unit));
    interner.push_expr_type(func, func_type);
}

impl BinaryTypeOperator {
    /// Return the actual rust numeric function associated with this operator
    pub fn function(self) -> fn(u32, u32) -> u32 {
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
    pub(crate) fn default_type(&self) -> Option<Type> {
        match self {
            TypeVariableKind::IntegerOrField => Some(Type::default_int_or_field_type()),
            TypeVariableKind::Integer => Some(Type::default_int_type()),
            TypeVariableKind::Constant(length) => Some(Type::Constant(*length)),
            TypeVariableKind::Normal => None,
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
                let length = size.evaluate_to_u32().expect("Cannot print variable sized arrays");
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
            Type::TypeVariable(binding, TypeVariableKind::Integer) => match &*binding.borrow() {
                TypeBinding::Bound(typ) => typ.into(),
                TypeBinding::Unbound(_) => Type::default_int_type().into(),
            },
            Type::TypeVariable(binding, TypeVariableKind::IntegerOrField) => {
                match &*binding.borrow() {
                    TypeBinding::Bound(typ) => typ.into(),
                    TypeBinding::Unbound(_) => Type::default_int_or_field_type().into(),
                }
            }
            Type::Bool => PrintableType::Boolean,
            Type::String(size) => {
                let size = size.evaluate_to_u32().expect("Cannot print variable sized strings");
                PrintableType::String { length: size }
            }
            Type::FmtString(_, _) => unreachable!("format strings cannot be printed"),
            Type::Error => unreachable!(),
            Type::Unit => PrintableType::Unit,
            Type::Constant(_) => unreachable!(),
            Type::Struct(def, ref args) => {
                let struct_type = def.borrow();
                let fields = struct_type.get_fields(args);
                let fields = vecmap(fields, |(name, typ)| (name, typ.into()));
                PrintableType::Struct { fields, name: struct_type.name.to_string() }
            }
            Type::Alias(alias, args) => alias.borrow().get_type(args).into(),
            Type::TraitAsType(_, _, _) => unreachable!(),
            Type::Tuple(types) => PrintableType::Tuple { types: vecmap(types, |typ| typ.into()) },
            Type::TypeVariable(_, _) => unreachable!(),
            Type::NamedGeneric(..) => unreachable!(),
            Type::Forall(..) => unreachable!(),
            Type::Function(arguments, return_type, env) => PrintableType::Function {
                arguments: arguments.iter().map(|arg| arg.into()).collect(),
                return_type: Box::new(return_type.as_ref().into()),
                env: Box::new(env.as_ref().into()),
            },
            Type::MutableReference(typ) => {
                PrintableType::MutableReference { typ: Box::new(typ.as_ref().into()) }
            }
            Type::Quoted(_) => unreachable!(),
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
            Type::TypeVariable(var, TypeVariableKind::Normal) => write!(f, "{:?}", var),
            Type::TypeVariable(binding, TypeVariableKind::IntegerOrField) => {
                write!(f, "IntOrField{:?}", binding)
            }
            Type::TypeVariable(binding, TypeVariableKind::Integer) => {
                write!(f, "Int{:?}", binding)
            }
            Type::TypeVariable(binding, TypeVariableKind::Constant(n)) => {
                write!(f, "{}{:?}", n, binding)
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
            Type::TraitAsType(_id, name, generics) => {
                write!(f, "impl {}", name)?;
                if !generics.is_empty() {
                    let generics = vecmap(generics, |arg| format!("{:?}", arg)).join(", ");
                    write!(f, "<{generics}>")?;
                }
                Ok(())
            }
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
            Type::NamedGeneric(binding, name, kind) => match kind {
                Kind::Normal => {
                    write!(f, "{} -> {:?}", name, binding)
                }
                Kind::Numeric(typ) => {
                    write!(f, "({} : {}) -> {:?}", name, typ, binding)
                }
            },
            Type::Constant(x) => x.fmt(f),
            Type::Forall(typevars, typ) => {
                let typevars = vecmap(typevars, |var| format!("{:?}", var));
                write!(f, "forall {}. {:?}", typevars.join(" "), typ)
            }
            Type::Function(args, ret, env) => {
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
