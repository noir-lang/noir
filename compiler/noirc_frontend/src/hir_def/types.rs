use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use crate::{
    hir::type_check::TypeCheckError,
    node_interner::{ExprId, NodeInterner, TypeAliasId},
};
use iter_extended::vecmap;
use noirc_errors::Span;
use noirc_printable_type::PrintableType;

use crate::{node_interner::StructId, Ident, Signedness};

use super::expr::{HirCallExpression, HirExpression, HirIdent};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Type {
    /// A primitive Field type
    FieldElement,

    /// Array(N, E) is an array of N elements of type E. It is expected that N
    /// is either a type variable of some kind or a Type::Constant.
    Array(Box<Type>, Box<Type>),

    /// A primitive integer type with the given sign and bit count.
    /// E.g. `u32` would be `Integer(Unsigned, 32)`
    Integer(Signedness, u32),

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

    /// A user-defined struct type. The `Shared<StructType>` field here refers to
    /// the shared definition for each instance of this struct type. The `Vec<Type>`
    /// represents the generic arguments (if any) to this struct type.
    Struct(Shared<StructType>, Vec<Type>),

    /// A tuple type with the given list of fields in the order they appear in source code.
    Tuple(Vec<Type>),

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

    /// NamedGenerics are the 'T' or 'U' in a user-defined generic function
    /// like `fn foo<T, U>(...) {}`. Unlike TypeVariables, they cannot be bound over.
    NamedGeneric(TypeVariable, Rc<String>),

    /// A functions with arguments, a return type and environment.
    /// the environment should be `Unit` by default,
    /// for closures it should contain a `Tuple` type with the captured
    /// variable types.
    Function(Vec<Type>, Box<Type>, Box<Type>),

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

/// Wrap around an unsolved type
#[derive(Debug, Clone, Eq)]
pub struct TypeAliasType {
    pub name: Ident,
    pub id: TypeAliasId,
    pub typ: Type,
    pub generics: Generics,
    pub span: Span,
}

impl std::hash::Hash for TypeAliasType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for TypeAliasType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::fmt::Display for TypeAliasType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;

        if !self.generics.is_empty() {
            let generics = vecmap(&self.generics, |(_, binding)| binding.borrow().to_string());
            write!(f, "{}", generics.join(", "))?;
        }

        Ok(())
    }
}

impl TypeAliasType {
    pub fn new(
        id: TypeAliasId,
        name: Ident,
        span: Span,
        typ: Type,
        generics: Generics,
    ) -> TypeAliasType {
        TypeAliasType { id, typ, name, span, generics }
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
            .map(|((old_id, old_var), new)| (*old_id, (old_var.clone(), new.clone())))
            .collect();

        self.typ.substitute(&substitutions)
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

    pub fn unbind(&mut self, id: TypeVariableId) {
        *self = TypeBinding::Unbound(id);
    }
}

/// A unique ID used to differentiate different type variables
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeVariableId(pub usize);

impl Type {
    pub fn default_int_type() -> Type {
        Type::Integer(Signedness::Unsigned, 64)
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
        let kind = TypeVariableKind::IntegerOrField;
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
        matches!(self.follow_bindings(), Type::FieldElement)
    }

    pub fn is_signed(&self) -> bool {
        matches!(self.follow_bindings(), Type::Integer(Signedness::Signed, _))
    }

    pub fn is_unsigned(&self) -> bool {
        matches!(self.follow_bindings(), Type::Integer(Signedness::Unsigned, _))
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
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
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
            Type::MutableReference(element) => element.contains_numeric_typevar(target_id),
            Type::String(length) => named_generic_id_matches_target(length),
            Type::FmtString(length, elements) => {
                elements.contains_numeric_typevar(target_id)
                    || named_generic_id_matches_target(length)
            }
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::FieldElement => {
                write!(f, "Field")
            }
            Type::Array(len, typ) => {
                if matches!(len.follow_bindings(), Type::NotConstant) {
                    write!(f, "[{typ}]")
                } else {
                    write!(f, "[{typ}; {len}]")
                }
            }
            Type::Integer(sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "i{num_bits}"),
                Signedness::Unsigned => write!(f, "u{num_bits}"),
            },
            Type::TypeVariable(id, TypeVariableKind::Normal) => write!(f, "{}", id.borrow()),
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
                TypeBinding::Unbound(_) if name.is_empty() => write!(f, "_"),
                TypeBinding::Unbound(_) => write!(f, "{name}"),
            },
            Type::Constant(x) => x.fmt(f),
            Type::Forall(typevars, typ) => {
                let typevars = vecmap(typevars, |(var, _)| var.to_string());
                write!(f, "forall {}. {}", typevars.join(" "), typ)
            }
            Type::Function(args, ret, env) => {
                let closure_env_text = match **env {
                    Type::Unit => "".to_string(),
                    _ => format!(" with closure environment {env}"),
                };

                let args = vecmap(args.iter(), ToString::to_string);

                write!(f, "fn({}) -> {ret}{closure_env_text}", args.join(", "))
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

pub struct UnificationError;

impl Type {
    /// Try to bind a MaybeConstant variable to self, succeeding if self is a Constant,
    /// MaybeConstant, or type variable.
    pub fn try_bind_to_maybe_constant(
        &self,
        var: &TypeVariable,
        target_length: u64,
    ) -> Result<(), UnificationError> {
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
                        TypeVariableKind::Constant(_) | TypeVariableKind::IntegerOrField => {
                            Err(UnificationError)
                        }
                    },
                }
            }
            _ => Err(UnificationError),
        }
    }

    /// Try to bind a PolymorphicInt variable to self, succeeding if self is an integer, field,
    /// other PolymorphicInt type, or type variable.
    pub fn try_bind_to_polymorphic_int(&self, var: &TypeVariable) -> Result<(), UnificationError> {
        let target_id = match &*var.borrow() {
            TypeBinding::Bound(_) => unreachable!(),
            TypeBinding::Unbound(id) => *id,
        };

        match self {
            Type::FieldElement | Type::Integer(..) => {
                *var.borrow_mut() = TypeBinding::Bound(self.clone());
                Ok(())
            }
            Type::TypeVariable(self_var, TypeVariableKind::IntegerOrField) => {
                let borrow = self_var.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => typ.try_bind_to_polymorphic_int(var),
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(_) => {
                        drop(borrow);
                        *var.borrow_mut() = TypeBinding::Bound(self.clone());
                        Ok(())
                    }
                }
            }
            Type::TypeVariable(binding, TypeVariableKind::Normal) => {
                let borrow = binding.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => typ.try_bind_to_polymorphic_int(var),
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(_) => {
                        drop(borrow);
                        // PolymorphicInt is more specific than TypeVariable so we bind the type
                        // variable to PolymorphicInt instead.
                        let clone =
                            Type::TypeVariable(var.clone(), TypeVariableKind::IntegerOrField);
                        *binding.borrow_mut() = TypeBinding::Bound(clone);
                        Ok(())
                    }
                }
            }
            _ => Err(UnificationError),
        }
    }

    pub fn try_bind_to(&self, var: &TypeVariable) -> Result<(), UnificationError> {
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
            Err(UnificationError)
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

    /// Try to unify this type with another, setting any type variables found
    /// equal to the other type in the process. Unification is more strict
    /// than sub-typing but less strict than Eq. Returns true if the unification
    /// succeeded. Note that any bindings performed in a failed unification are
    /// not undone. This may cause further type errors later on.
    pub fn unify(
        &self,
        expected: &Type,
        errors: &mut Vec<TypeCheckError>,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        if let Err(UnificationError) = self.try_unify(expected) {
            errors.push(make_error());
        }
    }

    /// `try_unify` is a bit of a misnomer since although errors are not committed,
    /// any unified bindings are on success.
    fn try_unify(&self, other: &Type) -> Result<(), UnificationError> {
        use Type::*;
        use TypeVariableKind as Kind;

        match (self, other) {
            (Error, _) | (_, Error) => Ok(()),

            (TypeVariable(binding, Kind::IntegerOrField), other)
            | (other, TypeVariable(binding, Kind::IntegerOrField)) => {
                // If it is already bound, unify against what it is bound to
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.try_unify(other);
                }

                // Otherwise, check it is unified against an integer and bind it
                other.try_bind_to_polymorphic_int(binding)
            }

            (TypeVariable(binding, Kind::Normal), other)
            | (other, TypeVariable(binding, Kind::Normal)) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.try_unify(other);
                }

                other.try_bind_to(binding)
            }

            (TypeVariable(binding, Kind::Constant(length)), other)
            | (other, TypeVariable(binding, Kind::Constant(length))) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.try_unify(other);
                }

                other.try_bind_to_maybe_constant(binding, *length)
            }

            (Array(len_a, elem_a), Array(len_b, elem_b)) => {
                len_a.try_unify(len_b)?;
                elem_a.try_unify(elem_b)
            }

            (String(len_a), String(len_b)) => len_a.try_unify(len_b),

            (FmtString(len_a, elements_a), FmtString(len_b, elements_b)) => {
                len_a.try_unify(len_b)?;
                elements_a.try_unify(elements_b)
            }

            (Tuple(elements_a), Tuple(elements_b)) => {
                if elements_a.len() != elements_b.len() {
                    Err(UnificationError)
                } else {
                    for (a, b) in elements_a.iter().zip(elements_b) {
                        a.try_unify(b)?;
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
                        a.try_unify(b)?;
                    }
                    Ok(())
                } else {
                    Err(UnificationError)
                }
            }

            (NamedGeneric(binding_a, name_a), NamedGeneric(binding_b, name_b)) => {
                // Ensure NamedGenerics are never bound during type checking
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
                        a.try_unify(b)?;
                    }

                    env_a.try_unify(env_b)?;
                    ret_b.try_unify(ret_a)
                } else {
                    Err(UnificationError)
                }
            }

            (MutableReference(elem_a), MutableReference(elem_b)) => elem_a.try_unify(elem_b),

            (other_a, other_b) => {
                if other_a == other_b {
                    Ok(())
                } else {
                    Err(UnificationError)
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
        if let Err(UnificationError) = self.try_unify(expected) {
            if !self.try_array_to_slice_coercion(expected, expression, interner) {
                errors.push(make_error());
            }
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

        if let (Type::Array(size1, element1), Type::Array(size2, element2)) = (&this, &target) {
            let size1 = size1.follow_bindings();
            let size2 = size2.follow_bindings();

            // If we have an array and our target is a slice
            if matches!(size1, Type::Constant(_)) && matches!(size2, Type::NotConstant) {
                // Still have to ensure the element types match.
                // Don't need to issue an error here if not, it will be done in unify_with_coercions
                if element1.try_unify(element2).is_ok() {
                    convert_array_expression_to_slice(expression, this, target, interner);
                    return true;
                }
            }
        }
        false
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
            Type::FmtString(size, fields) => {
                let size = Box::new(size.substitute(type_bindings));
                let fields = Box::new(fields.substitute(type_bindings));
                Type::FmtString(size, fields)
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
            Type::Function(args, ret, env) => {
                let args = vecmap(args, |arg| arg.substitute(type_bindings));
                let ret = Box::new(ret.substitute(type_bindings));
                let env = Box::new(env.substitute(type_bindings));
                Type::Function(args, ret, env)
            }
            Type::MutableReference(element) => {
                Type::MutableReference(Box::new(element.substitute(type_bindings)))
            }

            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
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
            Type::FmtString(len, fields) => {
                let len_occurs = len.occurs(target_id);
                let field_occurs = fields.occurs(target_id);
                len_occurs || field_occurs
            }
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
            FmtString(size, args) => {
                let size = Box::new(size.follow_bindings());
                let args = Box::new(args.follow_bindings());
                FmtString(size, args)
            }
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

            Function(args, ret, env) => {
                let args = vecmap(args, |arg| arg.follow_bindings());
                let ret = Box::new(ret.follow_bindings());
                let env = Box::new(env.follow_bindings());
                Function(args, ret, env)
            }

            MutableReference(element) => MutableReference(Box::new(element.follow_bindings())),

            // Expect that this function should only be called on instantiated types
            Forall(..) => unreachable!(),

            FieldElement | Integer(_, _) | Bool | Constant(_) | Unit | Error | NotConstant => {
                self.clone()
            }
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
    interner.push_expr_type(
        &func,
        Type::Function(vec![array_type], Box::new(target_type), Box::new(Type::Unit)),
    );
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
            TypeVariableKind::IntegerOrField | TypeVariableKind::Normal => Type::default_int_type(),
            TypeVariableKind::Constant(length) => Type::Constant(*length),
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
                let length = size.evaluate_to_u64().expect("Cannot print variable sized arrays");
                let typ = typ.as_ref();
                PrintableType::Array { length, typ: Box::new(typ.into()) }
            }
            Type::Integer(sign, bit_width) => match sign {
                Signedness::Unsigned => PrintableType::UnsignedInteger { width: *bit_width },
                Signedness::Signed => PrintableType::SignedInteger { width: *bit_width },
            },
            Type::TypeVariable(binding, TypeVariableKind::IntegerOrField) => {
                match &*binding.borrow() {
                    TypeBinding::Bound(typ) => typ.into(),
                    TypeBinding::Unbound(_) => Type::default_int_type().into(),
                }
            }
            Type::Bool => PrintableType::Boolean,
            Type::String(size) => {
                let size = size.evaluate_to_u64().expect("Cannot print variable sized strings");
                PrintableType::String { length: size }
            }
            Type::FmtString(_, _) => unreachable!("format strings cannot be printed"),
            Type::Error => unreachable!(),
            Type::Unit => unreachable!(),
            Type::Constant(_) => unreachable!(),
            Type::Struct(def, ref args) => {
                let struct_type = def.borrow();
                let fields = struct_type.get_fields(args);
                let fields = vecmap(fields, |(name, typ)| (name, typ.into()));
                PrintableType::Struct { fields, name: struct_type.name.to_string() }
            }
            Type::Tuple(_) => todo!("printing tuple types is not yet implemented"),
            Type::TypeVariable(_, _) => unreachable!(),
            Type::NamedGeneric(..) => unreachable!(),
            Type::Forall(..) => unreachable!(),
            Type::Function(_, _, _) => unreachable!(),
            Type::MutableReference(_) => unreachable!("cannot print &mut"),
            Type::NotConstant => unreachable!(),
        }
    }
}
