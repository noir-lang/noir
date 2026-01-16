use std::{borrow::Cow, cell::RefCell, collections::BTreeSet, rc::Rc};

use im::HashSet;
use rustc_hash::FxHashMap as HashMap;

#[cfg(test)]
use proptest_derive::Arbitrary;

use acvm::{AcirField, FieldElement};

use crate::{
    ast::{BinaryOpKind, IntegerBitSize, ItemVisibility, UnresolvedTypeExpression},
    hir::{
        def_map::ModuleId,
        type_check::{TypeCheckError, generics::TraitGenerics},
    },
    hir_def::types::{self},
    node_interner::{NodeInterner, TraitAssociatedTypeId, TraitId, TypeAliasId},
    signed_field::{AbsU128, SignedField},
    token::IntegerTypeSuffix,
};
use iter_extended::vecmap;
use noirc_errors::Location;
use noirc_printable_type::PrintableType;

use crate::shared::Signedness;
use crate::{ast::Ident, node_interner::TypeId};

use super::traits::NamedType;

mod arithmetic;
mod unification;
pub(crate) mod validity;

pub use unification::UnificationError;

/// Arbitrary recursion limit when following type variables or recurring on types some other way.
/// Types form trees but are not likely to be more deep than just a few levels in real code.
pub const TYPE_RECURSION_LIMIT: u32 = 100;

#[derive(Eq, Clone, Ord, PartialOrd)]
pub enum Type {
    /// A primitive Field type
    FieldElement,

    /// Array(N, E) is an array of N elements of type E. It is expected that N
    /// is either a type variable of some kind or a Type::Constant.
    Array(Box<Type>, Box<Type>),

    /// Vector(E) is a vector of elements of type E.
    Vector(Box<Type>),

    /// A primitive integer type with the given sign and bit count.
    /// E.g. `u32` would be `Integer(Unsigned, ThirtyTwo)`
    Integer(Signedness, IntegerBitSize),

    /// The primitive `bool` type.
    Bool,

    /// String(N) is an array of characters of length N. It is expected that N
    /// is either a type variable of some kind or a Type::Constant.
    String(Box<Type>),

    /// `FmtString(N, Vec<E>)` is an array of characters of length N that contains
    /// a list of fields specified inside the string by the following regular expression r"\{([\S]+)\}"
    /// and where N is either a type variable of some kind or a Type::Constant
    FmtString(Box<Type>, Box<Type>),

    /// The unit type `()`.
    Unit,

    /// A tuple type with the given list of fields in the order they appear in source code.
    Tuple(Vec<Type>),

    /// A user-defined struct or enum type. The `Shared<DataType>` field here refers to
    /// the shared definition for each instance of this struct or enum type. The `Vec<Type>`
    /// represents the generic arguments (if any) to this struct or enum type.
    DataType(Shared<DataType>, Vec<Type>),

    /// A user-defined alias to another type. Similar to a struct, this carries a shared
    /// reference to the definition of the alias along with any generics that may have
    /// been applied to the alias.
    Alias(Shared<TypeAlias>, Vec<Type>),

    /// TypeVariables are stand-in variables for some type which is not yet known.
    /// They are not to be confused with NamedGenerics. While the latter mostly works
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
    NamedGeneric(NamedGeneric),

    /// A cast (to, from) that's checked at monomorphization.
    ///
    /// Simplifications on arithmetic generics are only allowed on the LHS.
    CheckedCast { from: Box<Type>, to: Box<Type> },

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

    /// &T
    Reference(Box<Type>, /*mutable*/ bool),

    /// A type that's generic over the given type variables.
    /// Storing both the TypeVariableId and TypeVariable isn't necessary
    /// but it makes handling them both easier. The TypeVariableId should
    /// never be bound over during type checking, but during monomorphization it
    /// will be and thus needs the full TypeVariable link.
    Forall(GenericTypeVars, Box<Type>),

    /// A type-level integer. Included to let
    /// 1. an Array's size type variable
    ///    bind to an integer without special checks to bind it to a non-type.
    /// 2. values to be used at the type level
    Constant(SignedField, Kind),

    /// The type of quoted code in macros. This is always a comptime-only type
    Quoted(QuotedType),

    /// An infix expression in the form `lhs * rhs`.
    ///
    /// The `inversion` bool keeps track of whether this expression came from
    /// an expression like `4 = a / b` which was transformed to `a = 4 / b`
    /// so that if at some point a infix expression `b * (4 / b)` is created,
    /// it could be simplified back to `4`.
    InfixExpr(Box<Type>, BinaryTypeOperator, Box<Type>, bool /* inversion */),

    /// The result of some type error. Remembering type errors as their own type variant lets
    /// us avoid issuing repeat type errors for the same item. For example, a lambda with
    /// an invalid type would otherwise issue a new error each time it is called
    /// if not for this variant.
    Error,
}

#[derive(PartialEq, Eq, Clone, Ord, PartialOrd, Debug)]
pub struct NamedGeneric {
    pub type_var: TypeVariable,
    pub name: Rc<String>,
    /// Was this named generic implicitly added?
    pub implicit: bool,
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

    pub(crate) fn u32() -> Self {
        Self::numeric(Type::u32())
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

    /// Returns the default type this type variable should be bound to if it is still unbound
    /// during monomorphization.
    pub(crate) fn default_type(&self) -> Option<Type> {
        match self {
            Kind::IntegerOrField => Some(Type::default_int_or_field_type()),
            Kind::Integer => Some(Type::default_int_type()),
            Kind::Numeric(_typ) => {
                // Even though we have a type here, that type cannot be used as
                // the default type of a numeric generic.
                // For example, if we have `let N: u32` and we don't know
                // what `N` is, we can't assume it's `u32`.
                None
            }
            Kind::Any | Kind::Normal => None,
        }
    }

    fn integral_maximum_size(&self) -> Option<FieldElement> {
        match self.follow_bindings() {
            Kind::Any | Kind::IntegerOrField | Kind::Integer | Kind::Normal => None,
            Self::Numeric(typ) => typ.integral_maximum_size(),
        }
    }

    fn integral_minimum_size(&self) -> Option<SignedField> {
        match self.follow_bindings() {
            Kind::Any | Kind::IntegerOrField | Kind::Integer | Kind::Normal => None,
            Self::Numeric(typ) => typ.integral_minimum_size(),
        }
    }

    /// Ensure the given value fits in self.integral_maximum_size()
    pub(crate) fn ensure_value_fits(
        &self,
        value: SignedField,
        location: Location,
    ) -> Result<SignedField, TypeCheckError> {
        if let Some(maximum_size) = self.integral_maximum_size() {
            if value > SignedField::positive(maximum_size) {
                return Err(TypeCheckError::OverflowingConstant {
                    value,
                    kind: self.clone(),
                    maximum_size,
                    location,
                });
            }
        }

        if let Some(minimum_size) = self.integral_minimum_size() {
            if value < minimum_size {
                return Err(TypeCheckError::UnderflowingConstant {
                    value,
                    kind: self.clone(),
                    minimum_size,
                    location,
                });
            }
        }

        Ok(value)
    }

    /// Return the corresponding IntegerTypeSuffix if this is a numeric type kind.
    /// Note that `Kind::IntegerOrField` and `Kind::Integer` resolve to types and are thus not numeric.
    pub(crate) fn as_integer_type_suffix(&self) -> Option<IntegerTypeSuffix> {
        match self {
            Kind::Numeric(typ) => typ.as_integer_type_suffix(),
            _ => None,
        }
    }

    pub(crate) fn is_normal_or_any(&self) -> bool {
        matches!(self, Kind::Normal | Kind::Any)
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Any => write!(f, "any"),
            Kind::Normal => write!(f, "normal"),
            Kind::Integer => write!(f, "int"),
            Kind::IntegerOrField => write!(f, "intOrField"),
            Kind::Numeric(typ) => write!(f, "{typ}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord, strum_macros::EnumIter)]
pub enum QuotedType {
    Expr,
    Quoted,
    Type,
    TypedExpr,
    TypeDefinition,
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

/// Pretty print type bindings for debugging
#[allow(unused)]
pub fn type_bindings_to_string(bindings: &TypeBindings) -> String {
    if bindings.is_empty() {
        return "bindings: (none)".to_string();
    }

    let mut ret = if bindings.len() == 1 {
        "1 binding:".to_string()
    } else {
        format!("{} bindings:", bindings.len())
    };

    for (var, _, binding) in bindings.values() {
        ret += &format!("\n    {var:?} := {binding:?}");
    }

    ret
}

/// Represents a struct or enum type in the type system. Each instance of this
/// rust struct will be shared across all Type::DataType variants that represent
/// the same struct or enum type.
pub struct DataType {
    /// A unique id representing this type. Used to check if two types are equal.
    pub id: TypeId,

    pub name: Ident,
    pub visibility: ItemVisibility,

    /// A type's body is private to force struct fields or enum variants to only be
    /// accessed through get_field(), get_fields(), instantiate(), or similar functions
    /// since these will handle applying generic arguments to fields as well.
    body: TypeBody,

    pub generics: ResolvedGenerics,
    pub location: Location,

    pub must_use: MustUse,
}

/// Convenience enum to avoid using `Option<Option<String>>` to indicate
/// whether `#[must_use]` is present (outer option) and the optional message (inner option).
#[derive(Clone)]
pub enum MustUse {
    NoMustUse,
    MustUse(Option<String>),
}

enum TypeBody {
    /// A type with no body is still in the process of being created
    None,
    Struct(Vec<StructField>),

    #[allow(unused)]
    Enum(Vec<EnumVariant>),
}

#[derive(Clone, Debug)]
pub struct StructField {
    pub visibility: ItemVisibility,
    pub name: Ident,
    pub typ: Type,
}

#[derive(Clone)]
pub struct EnumVariant {
    pub name: Ident,
    pub params: Vec<Type>,

    /// True if this variant was declared as a function.
    /// Required to distinguish `Foo::Bar` from `Foo::Bar()`
    /// for zero-parameter variants. Only required for printing.
    pub is_function: bool,
}

impl EnumVariant {
    pub fn new(name: Ident, params: Vec<Type>, is_function: bool) -> EnumVariant {
        Self { name, params, is_function }
    }
}

/// Corresponds to generic lists such as `<T, U>` in the source program.
/// Used mainly for resolved types which no longer need information such
/// as names or kinds
pub type GenericTypeVars = Vec<TypeVariable>;

/// Corresponds to generic lists such as `<T, U>` with additional
/// information gathered during name resolution that is necessary
/// correctly resolving types.
pub type ResolvedGenerics = Vec<ResolvedGeneric>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGeneric {
    pub name: Rc<String>,
    pub type_var: TypeVariable,
    pub location: Location,
}

impl ResolvedGeneric {
    pub fn as_named_generic(self) -> Type {
        Type::NamedGeneric(NamedGeneric {
            type_var: self.type_var,
            name: self.name,
            implicit: false,
        })
    }

    pub fn kind(&self) -> Kind {
        self.type_var.kind()
    }
}

impl std::hash::Hash for DataType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for DataType {}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for DataType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DataType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl DataType {
    pub fn new(
        id: TypeId,
        name: Ident,
        location: Location,
        generics: ResolvedGenerics,
        visibility: ItemVisibility,
    ) -> DataType {
        DataType {
            id,
            name,
            location,
            generics,
            body: TypeBody::None,
            visibility,
            must_use: MustUse::NoMustUse,
        }
    }

    /// To account for cyclic references between structs, a struct's
    /// fields are resolved strictly after the struct itself is initially
    /// created. Therefore, this method is used to set the fields once they
    /// become known.
    pub fn set_fields(&mut self, fields: Vec<StructField>) {
        self.body = TypeBody::Struct(fields);
    }

    pub(crate) fn init_variants(&mut self) {
        match &mut self.body {
            TypeBody::None => {
                self.body = TypeBody::Enum(vec![]);
            }
            _ => panic!("Called init_variants but body was None"),
        }
    }

    pub(crate) fn push_variant(&mut self, variant: EnumVariant) {
        match &mut self.body {
            TypeBody::Enum(variants) => variants.push(variant),
            _ => panic!("Called push_variant on {self} but body wasn't an enum"),
        }
    }

    pub fn is_struct(&self) -> bool {
        matches!(&self.body, TypeBody::Struct(_))
    }

    pub fn is_enum(&self) -> bool {
        matches!(&self.body, TypeBody::Enum(_))
    }

    /// Retrieve the fields of this type with no modifications.
    /// Returns None if this is not a struct type.
    pub fn fields_raw(&self) -> Option<&[StructField]> {
        match &self.body {
            TypeBody::Struct(fields) => Some(fields),
            _ => None,
        }
    }

    /// Retrieve the variants of this type with no modifications.
    /// Panics if this is not an enum type.
    fn variants_raw(&self) -> Option<&[EnumVariant]> {
        match &self.body {
            TypeBody::Enum(variants) => Some(variants),
            _ => None,
        }
    }

    /// Return the generics on this type as a vector of types
    pub fn generic_types(&self) -> Vec<Type> {
        vecmap(&self.generics, |generic| generic.clone().as_named_generic())
    }

    /// Returns the field matching the given field name, as well as its visibility and field index.
    /// Always returns None if this is not a struct type.
    pub fn get_field(
        &self,
        field_name: &str,
        generic_args: &[Type],
    ) -> Option<(Type, ItemVisibility, usize)> {
        assert_eq!(self.generics.len(), generic_args.len());

        let mut fields = self.fields_raw()?.iter().enumerate();
        fields.find(|(_, field)| field.name.as_str() == field_name).map(|(i, field)| {
            let generics = self.generics.iter().zip(generic_args);
            let substitutions = generics
                .map(|(old, new)| {
                    (old.type_var.id(), (old.type_var.clone(), old.type_var.kind(), new.clone()))
                })
                .collect();

            (field.typ.substitute(&substitutions), field.visibility, i)
        })
    }

    /// Returns all the fields of this type, after being applied to the given generic arguments.
    /// Returns None if this is not a struct type.
    pub fn get_fields_with_visibility(
        &self,
        generic_args: &[Type],
    ) -> Option<Vec<(String, ItemVisibility, Type)>> {
        let substitutions = self.get_fields_substitutions(generic_args);

        Some(vecmap(self.fields_raw()?, |field| {
            let name = field.name.to_string();
            (name, field.visibility, field.typ.substitute(&substitutions))
        }))
    }

    /// Retrieve the fields of this type. Returns None if this is not a field type
    pub fn get_fields(&self, generic_args: &[Type]) -> Option<Vec<(String, Type, ItemVisibility)>> {
        let substitutions = self.get_fields_substitutions(generic_args);

        Some(vecmap(self.fields_raw()?, |field| {
            let name = field.name.to_string();
            (name, field.typ.substitute(&substitutions), field.visibility)
        }))
    }

    /// Retrieve the variants of this type. Returns None if this is not an enum type
    pub fn get_variants(&self, generic_args: &[Type]) -> Option<Vec<(String, Vec<Type>)>> {
        let substitutions = self.get_fields_substitutions(generic_args);

        Some(vecmap(self.variants_raw()?, |variant| {
            let name = variant.name.to_string();
            let args = vecmap(&variant.params, |param| param.substitute(&substitutions));
            (name, args)
        }))
    }

    /// Retrieve the given variant at the given variant index of this type.
    /// Returns None if this is not an enum type or `variant_index` is out of bounds.
    pub fn get_variant(
        &self,
        variant_index: usize,
        generic_args: &[Type],
    ) -> Option<(String, Vec<Type>)> {
        let substitutions = self.get_fields_substitutions(generic_args);
        let variant = self.variants_raw()?.get(variant_index)?;

        let name = variant.name.to_string();
        let args = vecmap(&variant.params, |param| param.substitute(&substitutions));
        Some((name, args))
    }

    fn get_fields_substitutions(
        &self,
        generic_args: &[Type],
    ) -> HashMap<TypeVariableId, (TypeVariable, Kind, Type)> {
        assert_eq!(
            self.generics.len(),
            generic_args.len(),
            "get_fields_substitutions: expected the number of generics to equal the number of generic_args"
        );

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
    ///
    /// Returns None if this is not a struct type.
    pub fn get_fields_as_written(&self) -> Option<Vec<StructField>> {
        Some(self.fields_raw()?.to_vec())
    }

    /// Returns the name and raw parameters of each variant of this type.
    /// This will not substitute any generic arguments so a generic variant like `X`
    /// in `enum Foo<T> { X(T) }` will return a `("X", Vec<T>)` pair.
    ///
    /// Returns None if this is not an enum type.
    pub fn get_variants_as_written(&self) -> Option<Vec<EnumVariant>> {
        Some(self.variants_raw()?.to_vec())
    }

    /// Returns the name and raw parameters of the variant at the given variant index.
    /// This will not substitute any generic arguments so a generic variant like `X`
    /// in `enum Foo<T> { X(T) }` will return a `("X", Vec<T>)` pair.
    ///
    /// Returns None if this is not an enum type or the given variant index is out of bounds.
    pub fn get_variant_as_written(&self, variant_index: usize) -> Option<&EnumVariant> {
        self.variants_raw()?.get(variant_index)
    }

    /// Returns the field at the given index. Panics if no field exists at the given index or this
    /// is not a struct type.
    pub fn field_at(&self, index: usize) -> &StructField {
        &self.fields_raw().unwrap()[index]
    }

    /// Returns the enum variant at the given index. Panics if no field exists at the given index
    /// or this is not an enum type.
    pub fn variant_at(&self, index: usize) -> &EnumVariant {
        &self.variants_raw().unwrap()[index]
    }

    /// Returns each of this type's field names. Returns None if this is not a struct type.
    pub fn field_names(&self) -> Option<BTreeSet<Ident>> {
        Some(self.fields_raw()?.iter().map(|field| field.name.clone()).collect())
    }

    /// Instantiate this struct type, returning a Vec of the new generic args (in
    /// the same order as self.generics)
    pub fn instantiate(&self, interner: &mut NodeInterner) -> Vec<Type> {
        vecmap(&self.generics, |generic| interner.next_type_variable_with_kind(generic.kind()))
    }

    /// Returns the function type of the variant at the given index of this enum.
    /// Requires the `Shared<DataType>` handle of self to create the given function type.
    /// Panics if this is not an enum.
    ///
    /// The function type uses the variant "as written" ie. no generic substitutions.
    /// Although the returned function is technically generic, Type::Function is returned
    /// instead of Type::Forall.
    pub fn variant_function_type(&self, variant_index: usize, this: Shared<DataType>) -> Type {
        let variant = self.variant_at(variant_index);
        let args = variant.params.clone();
        assert_eq!(this.borrow().id, self.id);
        let generics = self.generic_types();
        let ret = Box::new(Type::DataType(this, generics));
        Type::Function(args, ret, Box::new(Type::Unit), false)
    }

    /// Returns the function type of the variant at the given index of this enum.
    /// Requires the `Shared<DataType>` handle of self to create the given function type.
    /// Panics if this is not an enum.
    pub fn variant_function_type_with_forall(
        &self,
        variant_index: usize,
        this: Shared<DataType>,
    ) -> Type {
        let function_type = self.variant_function_type(variant_index, this);
        let typevars = vecmap(&self.generics, |generic| generic.type_var.clone());
        Type::Forall(typevars, Box::new(function_type))
    }
}

impl std::fmt::Display for DataType {
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
    pub generics: ResolvedGenerics,
    pub visibility: ItemVisibility,
    pub location: Location,
    /// Optional expression, used by type aliases to numeric generics
    pub numeric_expr: Option<UnresolvedTypeExpression>,
    pub module_id: ModuleId,
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
        generics: ResolvedGenerics,
        visibility: ItemVisibility,
        module_id: ModuleId,
    ) -> TypeAlias {
        TypeAlias { id, typ, name, location, generics, visibility, module_id, numeric_expr: None }
    }

    pub fn set_type_and_generics(
        &mut self,
        new_typ: Type,
        new_generics: ResolvedGenerics,
        num_expr: Option<UnresolvedTypeExpression>,
    ) {
        assert_eq!(self.typ, Type::Error);
        self.typ = new_typ;
        self.generics = new_generics;
        self.numeric_expr = num_expr;
    }

    /// Bind the generics of the aliased [Type] to the given generic arguments.
    ///
    /// Panics if the number of arguments do not meet expectations.
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

#[derive(Debug, Clone)]
pub struct TraitAssociatedType {
    pub id: TraitAssociatedTypeId,
    pub trait_id: TraitId,
    pub name: Ident,
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

impl BinaryTypeOperator {
    pub fn operator_to_binary_op_kind_helper(&self) -> BinaryOpKind {
        match self {
            BinaryTypeOperator::Addition => BinaryOpKind::Add,
            BinaryTypeOperator::Subtraction => BinaryOpKind::Subtract,
            BinaryTypeOperator::Multiplication => BinaryOpKind::Multiply,
            BinaryTypeOperator::Division => BinaryOpKind::Divide,
            BinaryTypeOperator::Modulo => BinaryOpKind::Modulo,
        }
    }
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

    pub fn try_bind(
        &self,
        binding: Type,
        kind: &Kind,
        location: Location,
    ) -> Result<(), TypeCheckError> {
        if !binding.kind().unifies(kind) {
            return Err(TypeCheckError::TypeKindMismatch {
                expected_kind: kind.clone(),
                expr_kind: binding.kind(),
                expr_location: location,
            });
        }

        let id = match &*self.1.borrow() {
            TypeBinding::Bound(binding) => {
                unreachable!("Expected unbound, found bound to {binding}")
            }
            TypeBinding::Unbound(id, _) => *id,
        };

        if binding.occurs(id) {
            Err(TypeCheckError::CyclicType { location, typ: binding })
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
            TypeBinding::Bound(binding) => {
                matches!(binding.follow_bindings(), Type::Integer(..))
            }
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

    /// Check that if bound, it's a signed integer
    pub fn is_signed(&self) -> bool {
        match &*self.borrow() {
            TypeBinding::Bound(binding) => {
                matches!(binding.follow_bindings(), Type::Integer(Signedness::Signed, _))
            }
            _ => false,
        }
    }

    /// Check that if bound, it's an unsigned integer
    pub fn is_unsigned(&self) -> bool {
        match &*self.borrow() {
            TypeBinding::Bound(binding) => {
                matches!(binding.follow_bindings(), Type::Integer(Signedness::Unsigned, _))
            }
            _ => false,
        }
    }

    pub(crate) fn into_named_generic(self, name: Rc<String>) -> Type {
        Type::NamedGeneric(NamedGeneric { type_var: self, name, implicit: false })
    }

    pub(crate) fn into_implicit_named_generic(self, name: Rc<String>) -> Type {
        Type::NamedGeneric(NamedGeneric { type_var: self, name, implicit: true })
    }
}

/// TypeBindings are the mutable insides of a TypeVariable.
/// They are either bound to some type, or are unbound.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
            Type::Vector(typ) => {
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
                        write!(f, "{binding}")
                    }
                }
            }
            Type::DataType(s, args) => {
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
                write!(f, "impl {name}{generics}")
            }
            Type::Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                if elements.len() == 1 {
                    write!(f, "({},)", elements[0])
                } else {
                    write!(f, "({})", elements.join(", "))
                }
            }
            Type::Bool => write!(f, "bool"),
            Type::String(len) => write!(f, "str<{len}>"),
            Type::FmtString(len, elements) => {
                write!(f, "fmtstr<{len}, {elements}>")
            }
            Type::Unit => write!(f, "()"),
            Type::Error => write!(f, "error"),
            Type::NamedGeneric(NamedGeneric { type_var, name, .. }) => match &*type_var.borrow() {
                TypeBinding::Bound(type_var) => type_var.fmt(f),
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
            Type::Reference(element, mutable) if *mutable => {
                write!(f, "&mut {element}")
            }
            Type::Reference(element, _) => {
                write!(f, "&{element}")
            }
            Type::Quoted(quoted) => write!(f, "{quoted}"),
            Type::InfixExpr(lhs, op, rhs, _) => {
                let this = self.canonicalize_checked();

                // Prevent infinite recursion
                if this != *self { write!(f, "{this}") } else { write!(f, "({lhs} {op} {rhs})") }
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

impl std::fmt::Debug for TypeBinding {
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
            QuotedType::Type => write!(f, "Type"),
            QuotedType::TypedExpr => write!(f, "TypedExpr"),
            QuotedType::TypeDefinition => write!(f, "TypeDefinition"),
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

impl Type {
    pub fn default_int_or_field_type() -> Type {
        Type::FieldElement
    }

    pub fn default_int_type() -> Type {
        Self::u32()
    }

    pub fn u32() -> Type {
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
        matches!(self.follow_bindings_shallow().as_ref(), Type::FieldElement)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self.follow_bindings_shallow().as_ref(), Type::Bool)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.follow_bindings_shallow().as_ref(), Type::Integer(_, _))
    }

    pub fn is_signed(&self) -> bool {
        match self.follow_bindings_shallow().as_ref() {
            Type::Integer(Signedness::Signed, _) => true,
            Type::TypeVariable(var) => var.is_signed(),
            _ => false,
        }
    }

    pub fn is_unsigned(&self) -> bool {
        match self.follow_bindings_shallow().as_ref() {
            Type::Integer(Signedness::Unsigned, _) => true,
            Type::TypeVariable(var) => var.is_unsigned(),
            _ => false,
        }
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
            | Type::Vector(_)
            | Type::Integer(..)
            | Type::Bool
            | Type::String(_)
            | Type::FmtString(_, _)
            | Type::Unit
            | Type::Function(..)
            | Type::Tuple(..)
            | Type::Quoted(..) => true,
            Type::Alias(alias_type, generics) => {
                alias_type.borrow().get_type(&generics).is_primitive()
            }
            Type::Reference(typ, _) => typ.is_primitive(),
            Type::DataType(..)
            | Type::TypeVariable(..)
            | Type::TraitAsType(..)
            | Type::NamedGeneric(..)
            | Type::CheckedCast { .. }
            | Type::Forall(..)
            | Type::Constant(..)
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

    pub(crate) fn is_mutable_ref(&self) -> bool {
        matches!(self.follow_bindings_shallow().as_ref(), Type::Reference(_, true))
    }

    pub(crate) fn is_ref(&self) -> bool {
        matches!(self.follow_bindings_shallow().as_ref(), Type::Reference(_, _))
    }

    /// Returns `true` if a type is allowed to appear in an assertion message.
    ///
    /// This should try filter out types which would cause a panic in `abi_gen::abi_type_from_hir_type`,
    /// but it has to be more permissive, as we don't have all information yet; some only become apparent
    /// after monomorphization.
    pub(crate) fn is_message_compatible(&self, is_monomorphized: bool) -> bool {
        match self {
            Type::FieldElement | Type::Integer(_, _) | Type::Bool | Type::String(_) => true,

            Type::Array(_, item) => item.is_message_compatible(is_monomorphized),
            Type::TypeVariable(binding) => match &*binding.borrow() {
                TypeBinding::Bound(typ) => typ.is_message_compatible(is_monomorphized),
                TypeBinding::Unbound(_, kind) => {
                    !is_monomorphized || matches!(kind, Kind::Integer | Kind::IntegerOrField)
                }
            },
            Type::DataType(def, args) => {
                let struct_type = def.borrow();
                let fields = struct_type.get_fields(args).unwrap_or_default();
                fields.iter().all(|(_, typ, _)| typ.is_message_compatible(is_monomorphized))
            }
            Type::Alias(def, args) => {
                let alias_type = def.borrow();
                alias_type.get_type(args).is_message_compatible(is_monomorphized)
            }
            Type::CheckedCast { to, .. } => to.is_message_compatible(is_monomorphized),
            Type::Tuple(fields) => {
                fields.iter().all(|typ| typ.is_message_compatible(is_monomorphized))
            }
            Type::Error
            | Type::Unit
            | Type::Constant(..)
            | Type::InfixExpr(..)
            | Type::TraitAsType(..)
            | Type::Forall(..)
            | Type::Vector(_)
            | Type::Function(_, _, _, _)
            | Type::FmtString(_, _)
            | Type::Quoted(_)
            | Type::Reference(..) => false,

            // A generic would cause a panic in ABI generation, but if we don't allow it
            // here then we reject all functions which are generic over the message type.
            // Since we don't have a marker trait to show what can be turned into into a message,
            // we have to delay this check to monomorphization.
            Type::NamedGeneric(..) => !is_monomorphized,
        }
    }

    /// Returns the number of `Forall`-quantified type variables on this type.
    /// Returns 0 if this is not a Type::Forall
    pub fn generic_count(&self) -> usize {
        match self {
            Type::Forall(generics, _) => generics.len(),
            Type::CheckedCast { to, .. } => to.generic_count(),
            Type::TypeVariable(type_variable)
            | Type::NamedGeneric(NamedGeneric { type_var: type_variable, .. }) => {
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

    /// Return this type as a monomorphic type - without a [Type::Forall] if there is one.
    /// This is only a shallow check since Noir's type system prohibits [Type::Forall] anywhere
    /// inside other types.
    pub fn as_monotype(&self) -> &Type {
        match self {
            Type::Forall(_, typ) => typ.as_ref(),
            other => other,
        }
    }

    /// Return the generics and type within this [Type::Forall].
    ///
    /// Returns an empty list of type variables and the type itself if it's not a [Type::Forall].
    pub fn unwrap_forall(&self) -> (Cow<GenericTypeVars>, &Type) {
        match self {
            Type::Forall(generics, typ) => (Cow::Borrowed(generics), typ.as_ref()),
            other => (Cow::Owned(GenericTypeVars::new()), other),
        }
    }

    pub fn kind(&self) -> Kind {
        match self {
            Type::CheckedCast { to, .. } => to.kind(),
            Type::NamedGeneric(NamedGeneric { type_var, .. }) => type_var.kind(),
            Type::Constant(_, kind) => kind.clone(),
            Type::TypeVariable(var) => match &*var.borrow() {
                TypeBinding::Bound(typ) => typ.kind(),
                TypeBinding::Unbound(_, type_var_kind) => type_var_kind.clone(),
            },
            Type::InfixExpr(lhs, _op, rhs, _) => lhs.infix_kind(rhs),
            Type::Alias(def, generics) => def.borrow().get_type(generics).kind(),
            // This is a concrete FieldElement, not an IntegerOrField
            Type::FieldElement
            | Type::Integer(..)
            | Type::Array(..)
            | Type::Vector(..)
            | Type::Bool
            | Type::String(..)
            | Type::FmtString(..)
            | Type::Unit
            | Type::Tuple(..)
            | Type::DataType(..)
            | Type::TraitAsType(..)
            | Type::Function(..)
            | Type::Reference(..)
            | Type::Forall(..)
            | Type::Quoted(..) => Kind::Normal,
            Type::Error => Kind::Any,
        }
    }

    /// Determines if a type contains a self referring alias by tracking visited TypeAliasId.
    ///
    /// - `aliases` is a mutable set of TypeAliasId to track visited aliases
    /// - it returns `true` if a cyclic alias is detected, `false` otherwise
    pub fn has_cyclic_alias(&self, aliases: &mut HashSet<TypeAliasId>) -> bool {
        match self {
            Type::CheckedCast { to, .. } => to.has_cyclic_alias(aliases),
            Type::NamedGeneric(NamedGeneric { type_var, .. }) => {
                Type::TypeVariable(type_var.clone()).has_cyclic_alias(aliases)
            }
            Type::TypeVariable(var) => match &*var.borrow() {
                TypeBinding::Bound(typ) => typ.has_cyclic_alias(aliases),
                TypeBinding::Unbound(_, _) => false,
            },
            Type::InfixExpr(lhs, _op, rhs, _) => {
                lhs.has_cyclic_alias(aliases) || rhs.has_cyclic_alias(aliases)
            }
            Type::Alias(def, generics) => {
                let alias_id = def.borrow().id;
                if aliases.contains(&alias_id) {
                    true
                } else {
                    aliases.insert(alias_id);
                    def.borrow().get_type(generics).has_cyclic_alias(aliases)
                }
            }
            _ => false,
        }
    }

    /// Unifies self and other kinds or fails with a Kind error
    fn infix_kind(&self, other: &Self) -> Kind {
        let self_kind = self.kind();
        let other_kind = other.kind();
        if self_kind.unifies(&other_kind) { self_kind } else { Kind::numeric(Type::Error) }
    }

    /// Creates an `InfixExpr`.
    pub fn infix_expr(lhs: Box<Type>, op: BinaryTypeOperator, rhs: Box<Type>) -> Type {
        Self::new_infix_expr(lhs, op, rhs, false)
    }

    /// Creates an `InfixExpr` that results from the compiler trying to unify something like
    /// `4 = a * b` into `a = 4 / b` (where `4 / b` is the "inverted" expression).
    pub fn inverted_infix_expr(lhs: Box<Type>, op: BinaryTypeOperator, rhs: Box<Type>) -> Type {
        Self::new_infix_expr(lhs, op, rhs, true)
    }

    pub fn new_infix_expr(
        lhs: Box<Type>,
        op: BinaryTypeOperator,
        rhs: Box<Type>,
        inversion: bool,
    ) -> Type {
        // If this infix expression contains an error then it is eventually an error itself.
        if matches!(*lhs, Type::Error) || matches!(*rhs, Type::Error) {
            return Type::Error;
        }

        // If an InfixExpr like this is tried to be created:
        //
        // a * (b / a)
        //
        // where `b / a` resulted from the compiler creating an inverted InfixExpr from a previous
        // unification (that is, the compiler had `b = a / y` and ended up doing `y = b / a` where
        // `y` is `rhs` here) then we can simplify this to just `b` because there wasn't an actual
        // division in the original expression, so multiplying it back is just going back to the
        // original `y`
        if let Type::InfixExpr(rhs_lhs, rhs_op, rhs_rhs, true) = &*rhs {
            if op.approx_inverse() == Some(*rhs_op) && lhs == *rhs_rhs {
                return *rhs_lhs.clone();
            }
        }

        // Same thing but on the other side.
        if let Type::InfixExpr(lhs_lhs, lhs_op, lhs_rhs, true) = &*lhs {
            if op.approx_inverse() == Some(*lhs_op) && rhs == *lhs_rhs {
                return *lhs_lhs.clone();
            }
        }

        Self::InfixExpr(lhs, op, rhs, inversion)
    }

    /// Check whether this type is an array or vector, and contains a nested vector in its element type.
    pub(crate) fn is_nested_vector(&self) -> bool {
        match self {
            Type::Vector(elem) => elem.as_ref().contains_vector(),
            Type::Array(_, elem) => elem.as_ref().contains_vector(),

            Type::Alias(alias, generics) => alias.borrow().get_type(generics).is_nested_vector(),
            Type::FmtString(_size, elem) => elem.as_ref().is_nested_vector(),
            Type::DataType(typ, generics) => {
                let typ = typ.borrow();
                if let Some(fields) = typ.get_fields(generics) {
                    if fields.iter().any(|(_, field, _)| field.is_nested_vector()) {
                        return true;
                    }
                } else if let Some(variants) = typ.get_variants(generics) {
                    if variants.iter().flat_map(|(_, args)| args).any(|typ| typ.is_nested_vector())
                    {
                        return true;
                    }
                }
                false
            }
            Type::Tuple(types) => {
                for typ in types {
                    if typ.is_nested_vector() {
                        return true;
                    }
                }
                false
            }
            Type::TypeVariable(type_variable)
            | Type::NamedGeneric(NamedGeneric { type_var: type_variable, .. }) => {
                match &*type_variable.borrow() {
                    TypeBinding::Bound(binding) => binding.is_nested_vector(),
                    TypeBinding::Unbound(_, _) => false,
                }
            }
            Type::CheckedCast { from, to } => from.is_nested_vector() || to.is_nested_vector(),
            Type::Reference(element, _) => element.is_nested_vector(),
            Type::Forall(_, typ) => typ.is_nested_vector(),

            Type::FieldElement
            | Type::Integer(..)
            | Type::Bool
            | Type::String(..)
            | Type::Unit
            | Type::TraitAsType(..)
            | Type::Function(..)
            | Type::Constant(..)
            | Type::Quoted(..)
            | Type::InfixExpr(..)
            | Type::Error => false,
        }
    }

    /// Check whether this type is itself a vector, or a struct/enum/tuple/array which contains a vector.
    pub(crate) fn contains_vector(&self) -> bool {
        match self {
            Type::Vector(_) => true,
            Type::Array(_, elem) => elem.as_ref().contains_vector(),
            Type::Alias(alias, generics) => alias.borrow().get_type(generics).contains_vector(),
            Type::DataType(typ, generics) => {
                let typ = typ.borrow();
                if let Some(fields) = typ.get_fields(generics) {
                    if fields.iter().any(|(_, field, _)| field.contains_vector()) {
                        return true;
                    }
                } else if let Some(variants) = typ.get_variants(generics) {
                    if variants.iter().flat_map(|(_, args)| args).any(|typ| typ.contains_vector()) {
                        return true;
                    }
                }
                false
            }
            Type::Tuple(types) => {
                for typ in types.iter() {
                    if typ.contains_vector() {
                        return true;
                    }
                }
                false
            }
            Type::FmtString(_size, elem) => elem.contains_vector(),
            Type::TypeVariable(type_variable)
            | Type::NamedGeneric(NamedGeneric { type_var: type_variable, .. }) => {
                match &*type_variable.borrow() {
                    TypeBinding::Bound(binding) => binding.contains_vector(),
                    TypeBinding::Unbound(_, _) => false,
                }
            }
            Type::CheckedCast { from, to } => from.contains_vector() || to.contains_vector(),
            Type::Reference(element, _) => element.contains_vector(),
            Type::Forall(_, typ) => typ.contains_vector(),

            Type::FieldElement
            | Type::Integer(..)
            | Type::Bool
            | Type::String(..)
            | Type::Unit
            | Type::TraitAsType(..)
            | Type::Function(..)
            | Type::Constant(..)
            | Type::Quoted(..)
            | Type::InfixExpr(..)
            | Type::Error => false,
        }
    }

    pub(crate) fn contains_reference(&self) -> bool {
        match self {
            Type::Unit
            | Type::Bool
            | Type::String(..)
            | Type::Integer(..)
            | Type::FieldElement
            | Type::Quoted(..)
            | Type::Constant(..)
            | Type::Function(..)
            | Type::TraitAsType(..)
            | Type::Forall(..)
            | Type::Error => false,
            Type::Array(length, typ) => length.contains_reference() || typ.contains_reference(),
            Type::Vector(typ) => typ.contains_reference(),
            Type::FmtString(length, typ) => length.contains_reference() || typ.contains_reference(),
            Type::Tuple(types) => types.iter().any(|typ| typ.contains_reference()),
            Type::DataType(typ, generics) => {
                let typ = typ.borrow();
                if let Some(fields) = typ.get_fields(generics) {
                    if fields.iter().any(|(_, field, _)| field.contains_reference()) {
                        return true;
                    }
                } else if let Some(variants) = typ.get_variants(generics) {
                    if variants
                        .iter()
                        .flat_map(|(_, args)| args)
                        .any(|typ| typ.contains_reference())
                    {
                        return true;
                    }
                }
                false
            }
            Type::Alias(alias, generics) => alias.borrow().get_type(generics).contains_reference(),
            Type::TypeVariable(type_variable)
            | Type::NamedGeneric(NamedGeneric { type_var: type_variable, .. }) => {
                match &*type_variable.borrow() {
                    TypeBinding::Bound(binding) => binding.contains_reference(),
                    TypeBinding::Unbound(_, _) => false,
                }
            }
            Type::CheckedCast { from: _, to } => to.contains_reference(),
            Type::InfixExpr(lhs, _op, rhs, _) => {
                lhs.contains_reference() || rhs.contains_reference()
            }
            Type::Reference(..) => true,
        }
    }

    pub(crate) fn contains_function(&self) -> bool {
        match self {
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::String(_)
            | Type::Unit
            | Type::Quoted(_)
            | Type::TraitAsType(..)
            | Type::Forall(..)
            | Type::Constant(..)
            | Type::Error => false,

            Type::Function(..) => true,

            Type::Reference(typ, _) => typ.contains_function(),
            Type::Array(length, typ) => length.contains_function() || typ.contains_function(),
            Type::Vector(typ) => typ.contains_function(),
            Type::FmtString(length, typ) => length.contains_function() || typ.contains_function(),
            Type::Tuple(types) => types.iter().any(|typ| typ.contains_function()),
            Type::DataType(typ, generics) => {
                let typ = typ.borrow();
                if let Some(fields) = typ.get_fields(generics) {
                    if fields.iter().any(|(_, field, _)| field.contains_function()) {
                        return true;
                    }
                } else if let Some(variants) = typ.get_variants(generics) {
                    if variants.iter().flat_map(|(_, args)| args).any(|typ| typ.contains_function())
                    {
                        return true;
                    }
                }
                false
            }
            Type::Alias(alias, generics) => alias.borrow().get_type(generics).contains_function(),
            Type::TypeVariable(type_variable)
            | Type::NamedGeneric(NamedGeneric { type_var: type_variable, .. }) => {
                match &*type_variable.borrow() {
                    TypeBinding::Bound(binding) => binding.contains_function(),
                    TypeBinding::Unbound(_, _) => false,
                }
            }
            Type::CheckedCast { from: _, to } => to.contains_function(),
            Type::InfixExpr(lhs, _op, rhs, _) => lhs.contains_function() || rhs.contains_function(),
        }
    }

    pub(crate) fn contains_type_variable(&self) -> bool {
        match self {
            Type::Integer(..)
            | Type::Bool
            | Type::Unit
            | Type::FieldElement
            | Type::Constant(..)
            | Type::Quoted(..)
            | Type::Error => false,
            Type::Forall(..) => true,
            Type::Array(length, typ) => {
                length.contains_type_variable() || typ.contains_type_variable()
            }
            Type::Vector(typ) => typ.contains_type_variable(),
            Type::String(length) => length.contains_type_variable(),
            Type::FmtString(length, typ) => {
                length.contains_type_variable() || typ.contains_type_variable()
            }
            Type::Tuple(items) | Type::DataType(_, items) | Type::Alias(_, items) => {
                items.iter().any(|typ| typ.contains_type_variable())
            }
            Type::TypeVariable(type_var) | Type::NamedGeneric(NamedGeneric { type_var, .. }) => {
                match &*type_var.borrow() {
                    TypeBinding::Bound(binding) => binding.contains_type_variable(),
                    TypeBinding::Unbound(_, _) => true,
                }
            }
            Type::TraitAsType(_trait_id, _trait_name, trait_generics) => {
                trait_generics.ordered.iter().any(|typ| typ.contains_type_variable())
                    || trait_generics
                        .named
                        .iter()
                        .any(|named_type| named_type.typ.contains_type_variable())
            }
            Type::CheckedCast { from, to } => {
                from.contains_type_variable() || to.contains_type_variable()
            }
            Type::Function(args, ret, env, _) => {
                args.iter().any(|typ| typ.contains_type_variable())
                    || ret.contains_type_variable()
                    || env.contains_type_variable()
            }
            Type::Reference(typ, _) => typ.contains_type_variable(),
            Type::InfixExpr(lhs, _, rhs, _) => {
                lhs.contains_type_variable() || rhs.contains_type_variable()
            }
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
                    TypeBinding::Unbound(id, _) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(new_target_id, Kind::IntegerOrField) => {
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
                    TypeBinding::Unbound(new_target_id, type_var_kind) => {
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
            Type::NamedGeneric(NamedGeneric { type_var, .. }) => {
                Some((type_var.1.clone(), type_var.kind()))
            }
            Type::CheckedCast { to, .. } => to.get_inner_type_variable(),
            _ => None,
        }
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
    pub fn evaluate_to_u32(&self, location: Location) -> Result<u32, TypeCheckError> {
        self.evaluate_to_signed_field(&Kind::u32(), location).map(|signed_field| {
            signed_field
                .try_to_unsigned::<u32>()
                .expect("ICE: size should have already been checked by evaluate_to_field_element")
        })
    }

    // TODO(https://github.com/noir-lang/noir/issues/6260): remove
    // the unifies checks once all kinds checks are implemented?
    pub(crate) fn evaluate_to_signed_field(
        &self,
        kind: &Kind,
        location: Location,
    ) -> Result<SignedField, TypeCheckError> {
        let run_simplifications = true;
        self.evaluate_to_signed_field_helper(kind, location, run_simplifications)
    }

    /// `evaluate_to_field_element` with optional generic arithmetic simplifications
    pub(crate) fn evaluate_to_signed_field_helper(
        &self,
        kind: &Kind,
        location: Location,
        run_simplifications: bool,
    ) -> Result<SignedField, TypeCheckError> {
        if let Some((binding, binding_kind)) = self.get_inner_type_variable() {
            match &*binding.borrow() {
                TypeBinding::Bound(binding) => {
                    if kind.unifies(&binding_kind) {
                        return binding.evaluate_to_signed_field_helper(
                            &binding_kind,
                            location,
                            run_simplifications,
                        );
                    }
                }
                TypeBinding::Unbound(..) => (),
            }
        }

        match self.canonicalize_with_simplifications(run_simplifications) {
            Type::Constant(x, constant_kind) => {
                if kind.unifies(&constant_kind) {
                    kind.ensure_value_fits(x, location)
                } else {
                    Err(TypeCheckError::TypeKindMismatch {
                        expected_kind: constant_kind,
                        expr_kind: kind.clone(),
                        expr_location: location,
                    })
                }
            }
            Type::InfixExpr(lhs, op, rhs, _) => {
                let infix_kind = lhs.infix_kind(&rhs);
                if kind.unifies(&infix_kind) {
                    let lhs_value = lhs.evaluate_to_signed_field_helper(
                        &infix_kind,
                        location,
                        run_simplifications,
                    )?;
                    let rhs_value = rhs.evaluate_to_signed_field_helper(
                        &infix_kind,
                        location,
                        run_simplifications,
                    )?;
                    op.function(lhs_value, rhs_value, &infix_kind, location)
                } else {
                    Err(TypeCheckError::TypeKindMismatch {
                        expected_kind: kind.clone(),
                        expr_kind: infix_kind,
                        expr_location: location,
                    })
                }
            }
            Type::CheckedCast { from, to } => {
                let to_value = to.evaluate_to_signed_field(kind, location)?;

                // if both 'to' and 'from' evaluate to a constant,
                // return None unless they match
                let skip_simplifications = false;
                if let Ok(from_value) =
                    from.evaluate_to_signed_field_helper(kind, location, skip_simplifications)
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
                            location,
                        })
                    }
                } else {
                    Ok(to_value)
                }
            }
            other => Err(TypeCheckError::NonConstantEvaluated { typ: other, location }),
        }
    }

    /// Retrieves the [Type] and [ItemVisibility] of the given field name:
    /// * for structs, it finds a member with a matching name
    /// * for tuples, it find the item by index, treating indexes as names "0", "1", ...
    /// * otherwise returns `None`
    pub fn get_field_type_and_visibility(
        &self,
        field_name: &str,
    ) -> Option<(Type, ItemVisibility)> {
        match self.follow_bindings() {
            Type::DataType(def, args) => def
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
            other => (other.clone(), HashMap::default()),
        }
    }

    /// Instantiates a type with the given types.
    /// This differs from substitute in that only the quantified type variables
    /// are matched against the type list and are eligible for substitution - similar
    /// to normal instantiation. This function is used when the turbofish operator
    /// is used and generic substitutions are provided manually by users.
    ///
    /// Expects the given type vector to be the same length as the Forall type variables.
    pub fn instantiate_with_bindings_and_turbofish(
        &self,
        bindings: TypeBindings,
        turbofish_types: Vec<Type>,
        interner: &NodeInterner,
        implicit_generic_count: usize,
    ) -> (Type, TypeBindings) {
        match self {
            Type::Forall(typevars, typ) => {
                assert_eq!(
                    turbofish_types.len() + implicit_generic_count,
                    typevars.len(),
                    "Turbofish operator used with incorrect generic count which was not caught by name resolution"
                );

                let implicit_and_turbofish_bindings = (0..implicit_generic_count)
                    .map(|_| interner.next_type_variable())
                    .chain(turbofish_types);

                let mut replacements: TypeBindings = typevars
                    .iter()
                    .zip(implicit_and_turbofish_bindings)
                    .map(|(var, binding)| (var.id(), (var.clone(), var.kind(), binding)))
                    .collect();

                for (binding_key, binding_value) in bindings {
                    replacements.insert(binding_key, binding_value);
                }

                let instantiated = typ.substitute(&replacements);
                (instantiated, replacements)
            }
            other => (other.clone(), HashMap::default()),
        }
    }

    fn type_variable_id(&self) -> Option<TypeVariableId> {
        match self {
            Type::TypeVariable(type_var) | Type::NamedGeneric(NamedGeneric { type_var, .. }) => {
                Some(type_var.0)
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
            // Prevent recurring forever if there's a `T := T` binding
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
            Type::Vector(element) => {
                let element = element.substitute_helper(type_bindings, substitute_bound_typevars);
                Type::Vector(Box::new(element))
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
            Type::NamedGeneric(NamedGeneric { type_var, .. }) | Type::TypeVariable(type_var) => {
                substitute_binding(type_var)
            }

            // Do not substitute_helper fields, it can lead to infinite recursion
            // and we should not match fields when type checking anyway.
            Type::DataType(fields, args) => {
                let args = vecmap(args, |arg| {
                    arg.substitute_helper(type_bindings, substitute_bound_typevars)
                });
                Type::DataType(fields.clone(), args)
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
                // Trying to substitute_helper a variable within a nested Forall
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
            Type::Reference(element, mutable) => Type::Reference(
                Box::new(element.substitute_helper(type_bindings, substitute_bound_typevars)),
                *mutable,
            ),

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
            Type::InfixExpr(lhs, op, rhs, inversion) => {
                let lhs = lhs.substitute_helper(type_bindings, substitute_bound_typevars);
                let rhs = rhs.substitute_helper(type_bindings, substitute_bound_typevars);
                Type::InfixExpr(Box::new(lhs), *op, Box::new(rhs), *inversion)
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
            Type::Vector(elem) => elem.occurs(target_id),
            Type::String(len) => len.occurs(target_id),
            Type::FmtString(len, fields) => {
                let len_occurs = len.occurs(target_id);
                let field_occurs = fields.occurs(target_id);
                len_occurs || field_occurs
            }
            Type::DataType(_, generic_args) | Type::Alias(_, generic_args) => {
                generic_args.iter().any(|arg| arg.occurs(target_id))
            }
            Type::TraitAsType(_, _, args) => {
                args.ordered.iter().any(|arg| arg.occurs(target_id))
                    || args.named.iter().any(|arg| arg.typ.occurs(target_id))
            }
            Type::Tuple(fields) => fields.iter().any(|field| field.occurs(target_id)),
            Type::CheckedCast { from, to } => from.occurs(target_id) || to.occurs(target_id),
            Type::NamedGeneric(NamedGeneric { type_var, .. }) | Type::TypeVariable(type_var) => {
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
            Type::Reference(element, _) => element.occurs(target_id),
            Type::InfixExpr(lhs, _op, rhs, _) => lhs.occurs(target_id) || rhs.occurs(target_id),

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
        fn helper(this: &Type, i: u32) -> Type {
            if i >= TYPE_RECURSION_LIMIT {
                panic!("Type recursion limit reached - types are too large")
            }
            let recur = |typ| helper(typ, i);

            use Type::*;
            match this {
                Array(size, elem) => Array(Box::new(recur(size)), Box::new(recur(elem))),
                Vector(elem) => Vector(Box::new(recur(elem))),
                String(size) => String(Box::new(recur(size))),
                FmtString(size, args) => {
                    let size = Box::new(recur(size));
                    let args = Box::new(recur(args));
                    FmtString(size, args)
                }
                DataType(def, args) => {
                    let args = vecmap(args, recur);
                    DataType(def.clone(), args)
                }
                Alias(def, args) => {
                    // We don't need to vecmap(args, recur) since we're recursively
                    // calling recur here already.
                    recur(&def.borrow().get_type(args))
                }
                Tuple(args) => Tuple(vecmap(args, recur)),
                CheckedCast { from, to } => {
                    let from = Box::new(recur(from));
                    let to = Box::new(recur(to));
                    CheckedCast { from, to }
                }
                TypeVariable(var) | NamedGeneric(types::NamedGeneric { type_var: var, .. }) => {
                    if let TypeBinding::Bound(typ) = &*var.borrow() {
                        return recur(typ);
                    }
                    this.clone()
                }
                Function(args, ret, env, unconstrained) => {
                    let args = vecmap(args, recur);
                    let ret = Box::new(recur(ret));
                    let env = Box::new(recur(env));
                    Function(args, ret, env, *unconstrained)
                }

                Reference(element, mutable) => Reference(Box::new(recur(element)), *mutable),

                TraitAsType(s, name, args) => {
                    let ordered = vecmap(&args.ordered, recur);
                    let named = vecmap(&args.named, |arg| NamedType {
                        name: arg.name.clone(),
                        typ: recur(&arg.typ),
                    });
                    TraitAsType(*s, name.clone(), TraitGenerics { ordered, named })
                }
                InfixExpr(lhs, op, rhs, inversion) => {
                    let lhs = recur(lhs);
                    let rhs = recur(rhs);
                    InfixExpr(Box::new(lhs), *op, Box::new(rhs), *inversion)
                }

                // Expect that this function should only be called on instantiated types
                Forall(..) => unreachable!(),
                FieldElement | Integer(_, _) | Bool | Constant(_, _) | Unit | Quoted(_) | Error => {
                    this.clone()
                }
            }
        }
        helper(self, 0)
    }

    /// Follow bindings if this is a type variable or generic to the first non-type-variable
    /// type. Unlike `follow_bindings`, this won't recursively follow any bindings on any
    /// fields or arguments of this type.
    pub fn follow_bindings_shallow(&self) -> Cow<Type> {
        let mut this = Cow::Borrowed(self);
        for _ in 0..TYPE_RECURSION_LIMIT {
            match this.as_ref() {
                Type::TypeVariable(var)
                | Type::NamedGeneric(NamedGeneric { type_var: var, .. }) => {
                    let binding = var.borrow();
                    if let TypeBinding::Bound(typ) = &*binding {
                        let typ = typ.clone();
                        drop(binding);
                        this = Cow::Owned(typ);
                    } else {
                        drop(binding);
                        return this;
                    };
                }
                Type::Alias(alias_def, generics) => {
                    let typ = alias_def.borrow().get_type(generics);
                    this = Cow::Owned(typ);
                }
                _ => return this,
            };
        }
        panic!("Type recursion limit reached - types are too large")
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

            Type::Vector(elem) => elem.replace_named_generics_with_type_variables(),
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
            Type::DataType(_, generics) => {
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
            Type::NamedGeneric(NamedGeneric { type_var, .. }) => {
                let type_binding = type_var.borrow();
                if let TypeBinding::Bound(binding) = &*type_binding {
                    let mut binding = binding.clone();
                    drop(type_binding);
                    binding.replace_named_generics_with_type_variables();
                    *self = binding;
                } else {
                    drop(type_binding);
                    *self = Type::TypeVariable(type_var.clone());
                }
            }
            Type::Function(args, ret, env, _unconstrained) => {
                for arg in args {
                    arg.replace_named_generics_with_type_variables();
                }
                ret.replace_named_generics_with_type_variables();
                env.replace_named_generics_with_type_variables();
            }
            Type::Reference(elem, _) => elem.replace_named_generics_with_type_variables(),
            Type::Forall(_, typ) => typ.replace_named_generics_with_type_variables(),
            Type::InfixExpr(lhs, _op, rhs, _) => {
                lhs.replace_named_generics_with_type_variables();
                rhs.replace_named_generics_with_type_variables();
            }
        }
    }

    pub fn vector_element_type(&self) -> Option<&Type> {
        match self {
            Type::Vector(element) => Some(element),
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
                let max = if max_bit_size == 128 { u128::MAX } else { (1u128 << max_bit_size) - 1 };
                Some(max.into())
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
            Type::NamedGeneric(NamedGeneric { type_var, .. }) => match &*type_var.borrow() {
                TypeBinding::Bound(typ) => typ.integral_maximum_size(),
                TypeBinding::Unbound(_, kind) => kind.integral_maximum_size(),
            },
            Type::Reference(typ, _) => typ.integral_maximum_size(),
            Type::InfixExpr(lhs, _op, rhs, _) => lhs.infix_kind(rhs).integral_maximum_size(),
            Type::Constant(_, kind) => kind.integral_maximum_size(),

            Type::Array(..)
            | Type::Vector(..)
            | Type::String(..)
            | Type::FmtString(..)
            | Type::Unit
            | Type::Tuple(..)
            | Type::DataType(..)
            | Type::TraitAsType(..)
            | Type::Function(..)
            | Type::Forall(..)
            | Type::Quoted(..)
            | Type::Error => None,
        }
    }

    pub(crate) fn integral_minimum_size(&self) -> Option<SignedField> {
        match self.follow_bindings_shallow().as_ref() {
            Type::FieldElement => None,
            Type::Integer(sign, num_bits) => {
                if *sign == Signedness::Unsigned {
                    return Some(SignedField::zero());
                }

                let max_bit_size = num_bits.bit_size() - 1;
                Some(if max_bit_size == 128 {
                    SignedField::negative(i128::MIN.abs_u128())
                } else {
                    SignedField::negative(1u128 << max_bit_size)
                })
            }
            Type::Bool => Some(SignedField::zero()),
            Type::TypeVariable(var) => {
                let binding = &var.1;
                match &*binding.borrow() {
                    TypeBinding::Unbound(_, type_var_kind) => match type_var_kind {
                        Kind::Any | Kind::Normal | Kind::Integer | Kind::IntegerOrField => None,
                        Kind::Numeric(typ) => typ.integral_minimum_size(),
                    },
                    TypeBinding::Bound(typ) => typ.integral_minimum_size(),
                }
            }
            _ => None,
        }
    }

    /// Substitute any [`Kind::Any`] in this type, for types that hold kinds (like [`Type::Constant`])
    /// with the given `kind`.
    pub(crate) fn substitute_kind_any_with_kind(self, kind: &Kind) -> Type {
        match self {
            Type::CheckedCast { from, to } => Type::CheckedCast {
                from: Box::new(from.substitute_kind_any_with_kind(kind)),
                to: Box::new(to.substitute_kind_any_with_kind(kind)),
            },
            Type::Constant(value, constant_kind) => {
                let kind = if let Kind::Any = constant_kind { kind.clone() } else { constant_kind };
                Type::Constant(value, kind)
            }
            Type::InfixExpr(lhs, op, rhs, inverse) => Type::InfixExpr(
                Box::new(lhs.substitute_kind_any_with_kind(kind)),
                op,
                Box::new(rhs.substitute_kind_any_with_kind(kind)),
                inverse,
            ),
            Type::FieldElement
            | Type::Array(..)
            | Type::Vector(..)
            | Type::Integer(..)
            | Type::Bool
            | Type::String(..)
            | Type::FmtString(..)
            | Type::Unit
            | Type::Tuple(..)
            | Type::DataType(..)
            | Type::Alias(..)
            | Type::TypeVariable(..)
            | Type::TraitAsType(..)
            | Type::NamedGeneric(..)
            | Type::Function(..)
            | Type::Reference(..)
            | Type::Quoted(..)
            | Type::Forall(..)
            | Type::Error => self,
        }
    }

    pub(crate) fn as_integer_type_suffix(&self) -> Option<IntegerTypeSuffix> {
        use {IntegerBitSize::*, Signedness::*};
        match self.follow_bindings_shallow().as_ref() {
            Type::FieldElement => Some(IntegerTypeSuffix::Field),
            Type::Integer(Signed, Eight) => Some(IntegerTypeSuffix::I8),
            Type::Integer(Signed, Sixteen) => Some(IntegerTypeSuffix::I16),
            Type::Integer(Signed, ThirtyTwo) => Some(IntegerTypeSuffix::I32),
            Type::Integer(Signed, SixtyFour) => Some(IntegerTypeSuffix::I64),
            Type::Integer(Unsigned, One) => Some(IntegerTypeSuffix::U1),
            Type::Integer(Unsigned, Eight) => Some(IntegerTypeSuffix::U8),
            Type::Integer(Unsigned, Sixteen) => Some(IntegerTypeSuffix::U16),
            Type::Integer(Unsigned, ThirtyTwo) => Some(IntegerTypeSuffix::U32),
            Type::Integer(Unsigned, SixtyFour) => Some(IntegerTypeSuffix::U64),
            Type::Integer(Unsigned, HundredTwentyEight) => Some(IntegerTypeSuffix::U128),
            _ => None,
        }
    }
}

impl From<u8> for Type {
    fn from(value: u8) -> Self {
        Type::Constant(
            value.into(),
            Kind::numeric(Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight)),
        )
    }
}

impl From<u32> for Type {
    fn from(value: u32) -> Self {
        Type::Constant(value.into(), Kind::u32())
    }
}

impl From<SignedField> for Type {
    fn from(value: SignedField) -> Self {
        Type::Constant(value, Kind::numeric(Type::FieldElement))
    }
}

impl BinaryTypeOperator {
    /// Perform the actual rust numeric operation associated with this operator
    pub fn function(
        self,
        a: SignedField,
        b: SignedField,
        kind: &Kind,
        location: Location,
    ) -> Result<SignedField, TypeCheckError> {
        match kind.integral_maximum_size() {
            None => match self {
                BinaryTypeOperator::Addition => Ok(a + b),
                BinaryTypeOperator::Subtraction => Ok(a - b),
                BinaryTypeOperator::Multiplication => Ok(a * b),
                BinaryTypeOperator::Division => (!b.is_zero())
                    .then(|| a / b)
                    .ok_or(TypeCheckError::DivisionByZero { lhs: a, rhs: b, location }),
                BinaryTypeOperator::Modulo => {
                    Err(TypeCheckError::ModuloOnFields { lhs: a, rhs: b, location })
                }
            },
            Some(maximum_size) => {
                if maximum_size.to_u128() == u128::MAX {
                    // For u128 operations we need to use u128
                    let a = a.to_u128();
                    let b = b.to_u128();

                    let err = TypeCheckError::FailingBinaryOp {
                        op: self,
                        lhs: a.to_string(),
                        rhs: b.to_string(),
                        location,
                    };
                    let result = match self {
                        BinaryTypeOperator::Addition => a.checked_add(b).ok_or(err)?,
                        BinaryTypeOperator::Subtraction => a.checked_sub(b).ok_or(err)?,
                        BinaryTypeOperator::Multiplication => a.checked_mul(b).ok_or(err)?,
                        BinaryTypeOperator::Division => a.checked_div(b).ok_or(err)?,
                        BinaryTypeOperator::Modulo => a.checked_rem(b).ok_or(err)?,
                    };

                    Ok(result.into())
                } else {
                    // Every other type first in i128, allowing both positive and negative values
                    let a = a.to_i128();
                    let b = b.to_i128();

                    let err = TypeCheckError::FailingBinaryOp {
                        op: self,
                        lhs: a.to_string(),
                        rhs: b.to_string(),
                        location,
                    };
                    let result = match self {
                        BinaryTypeOperator::Addition => a.checked_add(b).ok_or(err)?,
                        BinaryTypeOperator::Subtraction => a.checked_sub(b).ok_or(err)?,
                        BinaryTypeOperator::Multiplication => a.checked_mul(b).ok_or(err)?,
                        BinaryTypeOperator::Division => a.checked_div(b).ok_or(err)?,
                        BinaryTypeOperator::Modulo => a.checked_rem(b).ok_or(err)?,
                    };

                    kind.ensure_value_fits(result.into(), location)
                }
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
                let dummy_location = Location::dummy();
                let length = size
                    .evaluate_to_u32(dummy_location)
                    .expect("Cannot print variable sized arrays");
                let typ = typ.as_ref();
                PrintableType::Array { length, typ: Box::new(typ.into()) }
            }
            Type::Vector(typ) => {
                let typ = typ.as_ref();
                PrintableType::Vector { typ: Box::new(typ.into()) }
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
                let dummy_location = Location::dummy();
                let size = size
                    .evaluate_to_u32(dummy_location)
                    .expect("Cannot print variable sized strings");
                PrintableType::String { length: size }
            }
            Type::FmtString(size, typ) => {
                let dummy_location = Location::dummy();
                let size = size
                    .evaluate_to_u32(dummy_location)
                    .expect("Cannot print variable sized strings");
                PrintableType::FmtString { length: size, typ: Box::new(typ.as_ref().into()) }
            }
            Type::Error => unreachable!(),
            Type::Unit => PrintableType::Unit,
            Type::Constant(_, _) => unreachable!(),
            Type::DataType(def, args) => {
                let data_type = def.borrow();
                let name = data_type.name.to_string();

                if let Some(fields) = data_type.get_fields(args) {
                    let fields = vecmap(fields, |(name, typ, _)| (name, typ.into()));
                    PrintableType::Struct { fields, name }
                } else if let Some(variants) = data_type.get_variants(args) {
                    let variants =
                        vecmap(variants, |(name, args)| (name, vecmap(args, Into::into)));
                    PrintableType::Enum { name, variants }
                } else {
                    unreachable!()
                }
            }
            Type::Alias(alias, args) => alias.borrow().get_type(args).into(),
            Type::TraitAsType(..) => unreachable!(),
            Type::Tuple(types) => PrintableType::Tuple { types: vecmap(types, |typ| typ.into()) },
            Type::CheckedCast { to, .. } => to.as_ref().into(),
            Type::NamedGeneric(..) => unreachable!(),
            Type::Forall(..) => unreachable!(),
            Type::Function(arguments, return_type, env, _unconstrained) => {
                // Mimicking `Monomorphizer::convert_type_helper`: functions are represented as a tuple of constrained and unconstrained version.
                let make_function = |unconstrained| PrintableType::Function {
                    arguments: arguments.iter().map(|arg| arg.into()).collect(),
                    return_type: Box::new(return_type.as_ref().into()),
                    env: Box::new(env.as_ref().into()),
                    unconstrained,
                };
                PrintableType::Tuple { types: vecmap([false, true], make_function) }
            }
            Type::Reference(typ, mutable) => {
                PrintableType::Reference { typ: Box::new(typ.as_ref().into()), mutable: *mutable }
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
            Type::Vector(typ) => {
                write!(f, "[{typ:?}]")
            }
            Type::Integer(sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "i{num_bits}"),
                Signedness::Unsigned => write!(f, "u{num_bits}"),
            },
            Type::TypeVariable(var) => {
                let binding = &var.1;
                let binding = &*binding.borrow();
                if let TypeBinding::Unbound(_, type_var_kind) = binding {
                    match type_var_kind {
                        Kind::Any | Kind::Normal => write!(f, "{var:?}"),
                        Kind::IntegerOrField => write!(f, "IntOrField{binding:?}"),
                        Kind::Integer => write!(f, "Int{binding:?}"),
                        Kind::Numeric(typ) => write!(f, "Numeric({binding:?}: {typ:?})"),
                    }
                } else {
                    write!(f, "{binding:?}")
                }
            }
            Type::DataType(s, args) => {
                let args = vecmap(args, |arg| format!("{arg:?}"));
                if args.is_empty() {
                    write!(f, "{}", s.borrow())
                } else {
                    write!(f, "{}<{}>", s.borrow(), args.join(", "))
                }
            }
            Type::Alias(alias, args) => {
                let args = vecmap(args, |arg| format!("{arg:?}"));
                if args.is_empty() {
                    write!(f, "{}", alias.borrow())
                } else {
                    write!(f, "{}<{}>", alias.borrow(), args.join(", "))
                }
            }
            Type::TraitAsType(_id, name, generics) => write!(f, "impl {name}{generics:?}"),
            Type::Tuple(elements) => {
                let elements = vecmap(elements, |arg| format!("{arg:?}"));
                if elements.len() == 1 {
                    write!(f, "({},)", elements[0])
                } else {
                    write!(f, "({})", elements.join(", "))
                }
            }
            Type::Bool => write!(f, "bool"),
            Type::String(len) => write!(f, "str<{len:?}>"),
            Type::FmtString(len, elements) => {
                write!(f, "fmtstr<{len:?}, {elements:?}>")
            }
            Type::Unit => write!(f, "()"),
            Type::Error => write!(f, "error"),
            Type::CheckedCast { to, .. } => write!(f, "{to:?}"),
            Type::NamedGeneric(NamedGeneric { type_var, name, .. }) => match type_var.kind() {
                Kind::Any | Kind::Normal | Kind::Integer | Kind::IntegerOrField => {
                    write!(f, "{name}{type_var:?}")
                }
                Kind::Numeric(typ) => {
                    write!(f, "({name} : {typ}){type_var:?}")
                }
            },
            Type::Constant(x, kind) => write!(f, "({x}: {kind})"),
            Type::Forall(typevars, typ) => {
                let typevars = vecmap(typevars, |var| format!("{var:?}"));
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

                let args = vecmap(args.iter(), |arg| format!("{arg:?}"));

                write!(f, "fn({}) -> {ret:?}{closure_env_text}", args.join(", "))
            }
            Type::Reference(element, false) => {
                write!(f, "&{element:?}")
            }
            Type::Reference(element, true) => {
                write!(f, "&mut {element:?}")
            }
            Type::Quoted(quoted) => write!(f, "{quoted}"),
            Type::InfixExpr(lhs, op, rhs, _) => write!(f, "({lhs:?} {op} {rhs:?})"),
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

impl std::fmt::Debug for DataType {
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
            Type::Vector(elem) => elem.hash(state),
            Type::Integer(sign, bits) => {
                sign.hash(state);
                bits.hash(state);
            }
            Type::String(len) => len.hash(state),
            Type::FmtString(len, env) => {
                len.hash(state);
                env.hash(state);
            }
            Type::Tuple(elements) => elements.hash(state),
            Type::DataType(def, args) => {
                def.hash(state);
                args.hash(state);
            }
            Type::Alias(alias, args) => {
                alias.hash(state);
                args.hash(state);
            }
            Type::NamedGeneric(NamedGeneric { type_var, implicit: true, .. }) => {
                // An implicitly added unbound named generic's hash must be the same as any other
                // implicitly added unbound named generic's hash.
                if !type_var.borrow().is_unbound() {
                    type_var.hash(state);
                }
            }
            Type::TypeVariable(var) | Type::NamedGeneric(NamedGeneric { type_var: var, .. }) => {
                var.hash(state);
            }
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
            Type::Reference(elem, mutable) => {
                elem.hash(state);
                mutable.hash(state);
            }
            Type::Forall(vars, typ) => {
                vars.hash(state);
                typ.hash(state);
            }
            Type::CheckedCast { to, .. } => to.hash(state),
            Type::Constant(value, _) => value.hash(state),
            Type::Quoted(typ) => typ.hash(state),
            Type::InfixExpr(lhs, op, rhs, _) => {
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
            (Vector(lhs_elem), Vector(rhs_elem)) => lhs_elem == rhs_elem,
            (Integer(lhs_sign, lhs_bits), Integer(rhs_sign, rhs_bits)) => {
                lhs_sign == rhs_sign && lhs_bits == rhs_bits
            }
            (String(lhs_len), String(rhs_len)) => lhs_len == rhs_len,
            (FmtString(lhs_len, lhs_env), FmtString(rhs_len, rhs_env)) => {
                lhs_len == rhs_len && lhs_env == rhs_env
            }
            (Tuple(lhs_types), Tuple(rhs_types)) => lhs_types == rhs_types,
            (DataType(lhs_struct, lhs_generics), DataType(rhs_struct, rhs_generics)) => {
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
            (Reference(lhs_elem, lhs_mut), Reference(rhs_elem, rhs_mut)) => {
                lhs_elem == rhs_elem && lhs_mut == rhs_mut
            }
            (Forall(lhs_vars, lhs_type), Forall(rhs_vars, rhs_type)) => {
                lhs_vars == rhs_vars && lhs_type == rhs_type
            }
            (CheckedCast { to, .. }, other) | (other, CheckedCast { to, .. }) => **to == *other,
            (Constant(lhs, lhs_kind), Constant(rhs, rhs_kind)) => {
                lhs == rhs && lhs_kind == rhs_kind
            }
            (Quoted(lhs), Quoted(rhs)) => lhs == rhs,
            (InfixExpr(l_lhs, l_op, l_rhs, _), InfixExpr(r_lhs, r_op, r_rhs, _)) => {
                l_lhs == r_lhs && l_op == r_op && l_rhs == r_rhs
            }
            // Two implicitly added unbound named generics are equal
            (
                NamedGeneric(types::NamedGeneric { type_var: lhs_var, implicit: true, .. }),
                NamedGeneric(types::NamedGeneric { type_var: rhs_var, implicit: true, .. }),
            ) => {
                lhs_var.borrow().is_unbound() && rhs_var.borrow().is_unbound()
                    || lhs_var.id() == rhs_var.id()
            }
            // Special case: we consider unbound named generics and type variables to be equal to each
            // other if their type variable ids match. This is important for some corner cases in
            // monomorphization where we call `replace_named_generics_with_type_variables` but
            // still want them to be equal for canonicalization checks in arithmetic generics.
            // Without this we'd fail the `serialize` test.
            (
                NamedGeneric(types::NamedGeneric { type_var: lhs_var, .. }) | TypeVariable(lhs_var),
                NamedGeneric(types::NamedGeneric { type_var: rhs_var, .. }) | TypeVariable(rhs_var),
            ) => lhs_var.id() == rhs_var.id(),
            _ => false,
        }
    }
}
