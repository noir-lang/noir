use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};

use crate::{hir::type_check::TypeCheckError, node_interner::NodeInterner};
use iter_extended::{btree_map, vecmap};
use noirc_abi::AbiType;
use noirc_errors::Span;

use crate::{
    node_interner::{FuncId, StructId},
    Ident, Signedness,
};

/// A shared, mutable reference to some T.
/// Wrapper is required for Hash impl of RefCell.
#[derive(Debug, Eq, PartialOrd, Ord)]
pub struct Shared<T>(Rc<RefCell<T>>);

impl<T: std::hash::Hash> std::hash::Hash for Shared<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state)
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

/// A list of TypeVariableIds to bind to a type. Storing the
/// TypeVariable in addition to the matching TypeVariableId allows
/// the binding to later be undone if needed.
pub type TypeBindings = HashMap<TypeVariableId, (TypeVariable, Type)>;

#[derive(Debug, Eq)]
pub struct StructType {
    pub id: StructId,
    pub name: Ident,

    /// Fields are ordered and private, they should only
    /// be accessed through get_field(), get_fields(), or instantiate()
    /// since these will handle applying generic arguments to fields as well.
    fields: BTreeMap<Ident, Type>,

    pub generics: Generics,
    pub methods: HashMap<String, FuncId>,
    pub span: Span,
}

pub type Generics = Vec<(TypeVariableId, TypeVariable)>;

impl std::hash::Hash for StructType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
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
        fields: BTreeMap<Ident, Type>,
        generics: Generics,
    ) -> StructType {
        StructType { id, fields, name, span, generics, methods: HashMap::new() }
    }

    pub fn set_fields(&mut self, fields: BTreeMap<Ident, Type>) {
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

    pub fn get_fields(&self, generic_args: &[Type]) -> BTreeMap<String, Type> {
        assert_eq!(self.generics.len(), generic_args.len());

        let substitutions = self
            .generics
            .iter()
            .zip(generic_args)
            .map(|((old_id, old_var), new)| (*old_id, (old_var.clone(), new.clone())))
            .collect();

        self.fields
            .iter()
            .map(|(name, typ)| {
                let name = name.0.contents.clone();
                (name, typ.substitute(&substitutions))
            })
            .collect()
    }

    pub fn field_names(&self) -> BTreeSet<Ident> {
        self.fields.keys().cloned().collect()
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
            .map(|(old_id, old_var)| {
                let new = interner.next_type_variable();
                (new.clone(), (*old_id, (old_var.clone(), new)))
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Type {
    FieldElement(Comptime),
    Array(Box<Type>, Box<Type>),        // Array(4, Field) = [Field; 4]
    Integer(Comptime, Signedness, u32), // u32 = Integer(unsigned, 32)
    PolymorphicInteger(Comptime, TypeVariable),
    Bool(Comptime),
    Unit,
    Struct(Shared<StructType>, Vec<Type>),
    Tuple(Vec<Type>),
    TypeVariable(TypeVariable),

    /// NamedGenerics are the 'T' or 'U' in a user-defined generic function
    /// like `fn foo<T, U>(...) {}`. Unlike TypeVariables, they cannot be bound over.
    NamedGeneric(TypeVariable, Rc<String>),

    /// A functions with arguments, and a return type.
    Function(Vec<Type>, Box<Type>),

    /// A type generic over the given type variables.
    /// Storing both the TypeVariableId and TypeVariable isn't necessary
    /// but it makes handling them both easier. The TypeVariableId should
    /// never be bound over during type checking, but during monomorphisation it
    /// will be and thus needs the full TypeVariable link.
    Forall(Generics, Box<Type>),

    /// A type-level integer. Included to let an Array's size type variable
    /// bind to an integer without special checks to bind it to a non-type.
    Expression(TypeExpression),

    Error,
}

/// An integer expression that is allowed in a type position. This is most often used
/// for array length types. These expressions can only use basic binary operations
/// and can only reference global variables or type parameters.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum TypeExpression {
    /// Some constant value. Restricted to only u64 as TypeExpressions are only used
    /// for array lengths currently. Global variables are translated into Constants as well,
    /// using the value they are initialized to directly and eliminating the need to reference
    /// an external variable.
    Constant(u64),
    TypeVariable(TypeVariable),
    BinaryOperation(Box<TypeExpression>, BinaryTypeOperator, Box<TypeExpression>),
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

pub type TypeVariable = Shared<TypeBinding>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeBinding {
    Bound(Type),
    Unbound(TypeVariableId),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeVariableId(pub usize);

#[derive(Debug, Clone, Eq)]
pub enum Comptime {
    // Yes and No variants have optional spans representing the location in the source code
    // which caused them to be comptime.
    Yes(Option<Span>),
    No(Option<Span>),
    Maybe(TypeVariableId, Rc<RefCell<Option<Comptime>>>),
}

impl std::hash::Hash for Comptime {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);

        if let Comptime::Maybe(id, binding) = self {
            if let Some(is_comptime) = &*binding.borrow() {
                is_comptime.hash(state);
            } else {
                id.hash(state);
            }
        }
    }
}

impl PartialEq for Comptime {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Comptime::Maybe(id1, binding1), Comptime::Maybe(id2, binding2)) => {
                if let Some(new_self) = &*binding1.borrow() {
                    return new_self == other;
                }
                if let Some(new_other) = &*binding2.borrow() {
                    return self == new_other;
                }
                id1 == id2
            }
            (Comptime::Yes(_), Comptime::Yes(_)) | (Comptime::No(_), Comptime::No(_)) => true,
            _ => false,
        }
    }
}

/// Internal enum for `unify` to remember the type context of each span
/// to provide better error messages
#[derive(Debug)]
pub enum SpanKind {
    Comptime(Span),
    NotComptime(Span),
    None,
}

impl Comptime {
    pub fn new(interner: &mut NodeInterner) -> Self {
        let id = interner.next_type_variable_id();
        Self::Maybe(id, Rc::new(RefCell::new(None)))
    }

    fn set_span(&mut self, new_span: Span) {
        match self {
            Comptime::Yes(span) | Comptime::No(span) => *span = Some(new_span),
            Comptime::Maybe(_, binding) => {
                if let Some(binding) = &mut *binding.borrow_mut() {
                    binding.set_span(new_span);
                }
            }
        }
    }

    /// Try to unify these two Comptime constraints.
    pub fn unify(&self, other: &Self, span: Span) -> Result<(), SpanKind> {
        match (self, other) {
            (Comptime::Yes(_), Comptime::Yes(_)) | (Comptime::No(_), Comptime::No(_)) => Ok(()),

            (Comptime::Yes(y), Comptime::No(n)) | (Comptime::No(n), Comptime::Yes(y)) => {
                Err(match (y, n) {
                    (_, Some(span)) => SpanKind::NotComptime(*span),
                    (Some(span), _) => SpanKind::Comptime(*span),
                    _ => SpanKind::None,
                })
            }

            (Comptime::Maybe(_, binding), other) | (other, Comptime::Maybe(_, binding))
                if binding.borrow().is_some() =>
            {
                let binding = &*binding.borrow();
                binding.as_ref().unwrap().unify(other, span)
            }

            (Comptime::Maybe(id1, _), Comptime::Maybe(id2, _)) if id1 == id2 => Ok(()),

            // Both are unbound and do not refer to each other, arbitrarily set one equal to the other
            (Comptime::Maybe(_, binding), other) | (other, Comptime::Maybe(_, binding)) => {
                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                Ok(())
            }
        }
    }

    /// Try to unify these two Comptime constraints.
    pub fn is_subtype_of(&self, other: &Self, span: Span) -> Result<(), SpanKind> {
        match (self, other) {
            (Comptime::Yes(_), Comptime::Yes(_))
            | (Comptime::No(_), Comptime::No(_))

            // This is one of the only 2 differing cases between this and Comptime::unify
            | (Comptime::Yes(_), Comptime::No(_)) => Ok(()),

            (Comptime::No(n), Comptime::Yes(y)) => {
                Err(match (y, n) {
                    (_, Some(span)) => SpanKind::NotComptime(*span),
                    (Some(span), _) => SpanKind::Comptime(*span),
                    _ => SpanKind::None,
                })
            }

            (Comptime::Maybe(_, binding), other) if binding.borrow().is_some() => {
                let binding = &*binding.borrow();
                binding.as_ref().unwrap().is_subtype_of(other, span)
            }

            (other, Comptime::Maybe(_, binding)) if binding.borrow().is_some() => {
                let binding = &*binding.borrow();
                other.is_subtype_of(binding.as_ref().unwrap(), span)
            }

            (Comptime::Maybe(id1, _), Comptime::Maybe(id2, _)) if id1 == id2 => Ok(()),

            // This is the other differing case between this and Comptime::unify.
            // If this is polymorphically comptime, dont force it to be non-comptime because it is
            // passed as an argument to a function expecting a non-comptime parameter.
            (Comptime::Maybe(_, binding), Comptime::No(_)) if binding.borrow().is_none() => Ok(()),

            (Comptime::Maybe(_, binding), other) => {
                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                Ok(())
            }
            (other, Comptime::Maybe(_, binding)) => {
                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                Ok(())
            }
        }
    }

    /// Combine these two Comptimes together, returning
    /// - Comptime::Yes if both are Yes,
    /// - Comptime::No if either are No,
    /// - or if both are Maybe, unify them both and return the lhs.
    pub fn and(&self, other: &Self, span: Span) -> Self {
        match (self, other) {
            (Comptime::Yes(_), Comptime::Yes(_)) => Comptime::Yes(Some(span)),

            (Comptime::No(_), Comptime::No(_))
            | (Comptime::Yes(_), Comptime::No(_))
            | (Comptime::No(_), Comptime::Yes(_)) => Comptime::No(Some(span)),

            (Comptime::Maybe(_, binding), other) | (other, Comptime::Maybe(_, binding))
                if binding.borrow().is_some() =>
            {
                let binding = &*binding.borrow();
                binding.as_ref().unwrap().and(other, span)
            }

            (Comptime::Maybe(id1, _), Comptime::Maybe(id2, _)) if id1 == id2 => self.clone(),

            (Comptime::Maybe(_, binding), other) | (other, Comptime::Maybe(_, binding)) => {
                let mut clone = other.clone();
                clone.set_span(span);
                *binding.borrow_mut() = Some(clone);
                other.clone()
            }
        }
    }

    pub fn is_comptime(&self) -> bool {
        match self {
            Comptime::Yes(_) => true,
            Comptime::No(_) => false,
            Comptime::Maybe(_, binding) => {
                if let Some(binding) = &*binding.borrow() {
                    return binding.is_comptime();
                }
                true
            }
        }
    }
}

impl Type {
    pub fn field(span: Option<Span>) -> Type {
        Type::FieldElement(Comptime::No(span))
    }

    pub fn comptime(span: Option<Span>) -> Type {
        Type::FieldElement(Comptime::Yes(span))
    }

    pub fn default_int_type(span: Option<Span>) -> Type {
        Type::field(span)
    }

    pub fn type_variable(id: TypeVariableId) -> Type {
        Type::TypeVariable(Shared::new(TypeBinding::Unbound(id)))
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::FieldElement(comptime) => {
                write!(f, "{}Field", comptime)
            }
            Type::Array(len, typ) => match len.evaluate_to_u64() {
                Some(len) => write!(f, "[{}; {}]", typ, len),
                None => write!(f, "[{}]", typ),
            },
            Type::Integer(comptime, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{}i{}", comptime, num_bits),
                Signedness::Unsigned => write!(f, "{}u{}", comptime, num_bits),
            },
            Type::PolymorphicInteger(_, binding) => {
                if let TypeBinding::Unbound(_) = &*binding.borrow() {
                    // Show a Field by default if this PolymorphicInteger is unbound, since that is
                    // what they bind to by default anyway. It is less confusing than displaying it
                    // as a generic.
                    write!(f, "Field")
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
            Type::Bool(comptime) => write!(f, "{}bool", comptime),
            Type::Unit => write!(f, "()"),
            Type::Error => write!(f, "error"),
            Type::TypeVariable(id) => write!(f, "{}", id.borrow()),
            Type::NamedGeneric(binding, name) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.fmt(f),
                TypeBinding::Unbound(_) if name.is_empty() => write!(f, "_"),
                TypeBinding::Unbound(_) => write!(f, "{}", name),
            },
            Type::Expression(e) => e.fmt(f),
            Type::Forall(typevars, typ) => {
                let typevars = vecmap(typevars, |(var, _)| var.to_string());
                write!(f, "forall {}. {}", typevars.join(" "), typ)
            }
            Type::Function(args, ret) => {
                let args = vecmap(args, ToString::to_string);
                write!(f, "fn({}) -> {}", args.join(", "), ret)
            }
        }
    }
}

impl std::fmt::Display for TypeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeExpression::Constant(value) => write!(f, "{value}"),
            TypeExpression::TypeVariable(var) => var.borrow().fmt(f),
            TypeExpression::BinaryOperation(lhs, op, rhs) => write!(f, "({} {} {})", lhs, op, rhs),
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

impl std::fmt::Display for Comptime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Comptime::Yes(_) => write!(f, "comptime "),
            Comptime::No(_) => Ok(()),
            Comptime::Maybe(_, binding) => match &*binding.borrow() {
                Some(binding) => binding.fmt(f),
                None => write!(f, "comptime "),
            },
        }
    }
}

impl Type {
    /// Mutate the span for Comptime to track where comptime is required for better
    /// error messages that show both the erroring callsite and the callsite before
    /// which required the variable to be comptime or non-comptime.
    pub fn set_comptime_span(&mut self, new_span: Span) {
        match self {
            Type::FieldElement(comptime) | Type::Integer(comptime, _, _) => {
                comptime.set_span(new_span)
            }
            Type::PolymorphicInteger(span, binding) => {
                if let TypeBinding::Bound(binding) = &mut *binding.borrow_mut() {
                    return binding.set_comptime_span(new_span);
                }
                span.set_span(new_span);
            }
            _ => (),
        }
    }

    pub fn set_comptime(&mut self, new_comptime: Comptime) {
        match self {
            Type::FieldElement(comptime) | Type::Integer(comptime, _, _) => {
                *comptime = new_comptime;
            }
            Type::PolymorphicInteger(comptime, binding) => {
                if let TypeBinding::Bound(binding) = &mut *binding.borrow_mut() {
                    return binding.set_comptime(new_comptime);
                }
                *comptime = new_comptime;
            }
            _ => (),
        }
    }

    /// Try to bind a PolymorphicInt variable to self, succeeding if self is an integer, field,
    /// other PolymorphicInt type, or type variable. If use_subtype is true, the Comptime fields
    /// of each will be checked via subtyping rather than unification.
    pub fn try_bind_to_polymorphic_int(
        &self,
        var: &TypeVariable,
        var_comptime: &Comptime,
        use_subtype: bool,
        span: Span,
    ) -> Result<(), SpanKind> {
        let target_id = match &*var.borrow() {
            TypeBinding::Bound(_) => unreachable!(),
            TypeBinding::Unbound(id) => *id,
        };

        let bind = |int_comptime: &Comptime| {
            let mut clone = self.clone();
            let mut new_comptime = var_comptime.clone();
            new_comptime.set_span(span);
            clone.set_comptime(new_comptime);

            *var.borrow_mut() = TypeBinding::Bound(clone);

            if use_subtype {
                var_comptime.is_subtype_of(int_comptime, span)
            } else {
                var_comptime.unify(int_comptime, span)
            }
        };

        match self {
            Type::FieldElement(int_comptime, ..) | Type::Integer(int_comptime, ..) => {
                bind(int_comptime)
            }
            Type::PolymorphicInteger(int_comptime, self_var) => {
                let borrow = self_var.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => {
                        typ.try_bind_to_polymorphic_int(var, var_comptime, use_subtype, span)
                    }
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(_) => {
                        drop(borrow);
                        bind(int_comptime)
                    }
                }
            }
            Type::TypeVariable(binding) => {
                let borrow = binding.borrow();
                match &*borrow {
                    TypeBinding::Bound(typ) => {
                        typ.try_bind_to_polymorphic_int(var, var_comptime, use_subtype, span)
                    }
                    // Avoid infinitely recursive bindings
                    TypeBinding::Unbound(id) if *id == target_id => Ok(()),
                    TypeBinding::Unbound(_) => {
                        drop(borrow);
                        // PolymorphicInt is more specific than TypeVariable so we bind the type
                        // variable to PolymorphicInt instead.
                        let mut clone = Type::PolymorphicInteger(var_comptime.clone(), var.clone());
                        clone.set_comptime_span(span);
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

    fn is_comptime(&self) -> bool {
        match self {
            Type::FieldElement(comptime) => comptime.is_comptime(),
            Type::Integer(comptime, ..) => comptime.is_comptime(),
            Type::PolymorphicInteger(comptime, binding) => {
                if let TypeBinding::Bound(binding) = &*binding.borrow() {
                    return binding.is_comptime();
                }
                comptime.is_comptime()
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

        match (expected.is_comptime(), err_span) {
            (true, SpanKind::NotComptime(span)) => {
                let msg = "The value is non-comptime because of this expression, which uses another non-comptime value".into();
                errors.push(TypeCheckError::Unstructured { msg, span });
            }
            (false, SpanKind::Comptime(span)) => {
                let msg = "The value is comptime because of this expression, which forces the value to be comptime".into();
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

            (PolymorphicInteger(comptime, binding), other)
            | (other, PolymorphicInteger(comptime, binding)) => {
                // If it is already bound, unify against what it is bound to
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.try_unify(other, span);
                }

                // Otherwise, check it is unified against an integer and bind it
                other.try_bind_to_polymorphic_int(binding, comptime, false, span)
            }

            (TypeVariable(binding), other) | (other, TypeVariable(binding)) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.try_unify(other, span);
                }

                Ok(())
            }

            (Array(len_a, elem_a), Array(len_b, elem_b)) => {
                len_a.try_unify(len_b, span)?;
                elem_a.try_unify(elem_b, span)
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
                let is_unbound = |binding: &Shared<TypeBinding>| {
                    matches!(&*binding.borrow(), TypeBinding::Unbound(_))
                };

                // Ensure NamedGenerics are never bound during type checking
                assert!(is_unbound(binding_a) && is_unbound(binding_b));
                if name_a == name_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }

            (Expression(expr_a), Expression(expr_b)) => expr_a.try_unify(expr_b, span),

            (other_a, other_b) => {
                if other_a == other_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }
        }
    }

    /// The `subtype` term here is somewhat loose, the only subtyping relations remaining
    /// have to do with Comptime tracking.
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

            (PolymorphicInteger(comptime, binding), other) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return link.is_subtype_of(other, span);
                }

                // Otherwise, check it is unified against an integer and bind it
                other.try_bind_to_polymorphic_int(binding, comptime, true, span)
            }
            // These needs to be a separate case to keep the argument order of is_subtype_of
            (other, PolymorphicInteger(comptime, binding)) => {
                if let TypeBinding::Bound(link) = &*binding.borrow() {
                    return other.is_subtype_of(link, span);
                }

                // use_subtype is false here since we have other <: PolymorphicInt
                // while the flag expects PolymorphicInt <: other
                other.try_bind_to_polymorphic_int(binding, comptime, false, span)
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
                let is_unbound = |binding: &Shared<TypeBinding>| {
                    matches!(&*binding.borrow(), TypeBinding::Unbound(_))
                };

                // Ensure NamedGenerics are never bound during type checking
                assert!(is_unbound(binding_a) && is_unbound(binding_b));
                if name_a == name_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }

            // Re-using try_unify here should be fine as TypeExpressions only have exact equality
            // and do not contain any Comptime variables which is the only part that would differ
            // between unification and subtyping.
            (Expression(expr_a), Expression(expr_b)) => expr_a.try_unify(expr_b, span),

            (other_a, other_b) => {
                if other_a == other_b {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            }
        }
    }

    pub fn evaluate_to_u64(&self) -> Option<u64> {
        match self {
            Type::PolymorphicInteger(_, binding)
            | Type::NamedGeneric(binding, _)
            | Type::TypeVariable(binding) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.evaluate_to_u64(),
                TypeBinding::Unbound(_) => None,
            },
            Type::Array(len, _elem) => len.evaluate_to_u64(),
            Type::Expression(expr) => expr.evaluate_to_u64(),
            _ => None,
        }
    }

    // Note; use strict_eq instead of partial_eq when comparing field types
    // in this method, you most likely want to distinguish between public and private
    pub fn as_abi_type(&self) -> AbiType {
        match self {
            Type::FieldElement(_) => AbiType::Field,
            Type::Array(size, typ) => {
                let size = size
                    .evaluate_to_u64()
                    .expect("Cannot have variable sized arrays as a parameter to main");
                AbiType::Array { length: size as u128, typ: Box::new(typ.as_abi_type()) }
            }
            Type::Integer(_, sign, bit_width) => {
                let sign = match sign {
                    Signedness::Unsigned => noirc_abi::Sign::Unsigned,
                    Signedness::Signed => noirc_abi::Sign::Signed,
                };

                AbiType::Integer { sign, width: *bit_width }
            }
            Type::PolymorphicInteger(_, binding) => match &*binding.borrow() {
                TypeBinding::Bound(typ) => typ.as_abi_type(),
                TypeBinding::Unbound(_) => Type::default_int_type(None).as_abi_type(),
            },
            Type::Bool(_) => AbiType::Integer { sign: noirc_abi::Sign::Unsigned, width: 1 },
            Type::Error => unreachable!(),
            Type::Unit => unreachable!(),
            Type::Expression(_) => unreachable!(),
            Type::Struct(def, args) => {
                let struct_type = def.borrow();
                let fields = struct_type.get_fields(args);
                let abi_map = btree_map(fields, |(name, typ)| (name, typ.as_abi_type()));
                AbiType::Struct { fields: abi_map }
            }
            Type::Tuple(_) => todo!("as_abi_type not yet implemented for tuple types"),
            Type::TypeVariable(_) => unreachable!(),
            Type::NamedGeneric(..) => unreachable!(),
            Type::Forall(..) => unreachable!(),
            Type::Function(_, _) => unreachable!(),
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
                let typ = def.borrow().get_field(name, args).unwrap().0;
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
            Type::Struct(def, args) => def.borrow().get_field(field_name, args).unwrap().0,
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
            Type::PolymorphicInteger(_, binding)
            | Type::NamedGeneric(binding, _)
            | Type::TypeVariable(binding) => substitute_binding(binding),

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
            Type::Expression(expr) => Type::Expression(expr.substitute(type_bindings)),

            Type::FieldElement(_)
            | Type::Integer(_, _, _)
            | Type::Bool(_)
            | Type::Error
            | Type::Unit => self.clone(),
        }
    }

    /// True if the given TypeVariableId is free anywhere
    /// within self
    fn occurs(&self, target_id: TypeVariableId) -> bool {
        match self {
            Type::Array(len, elem) => len.occurs(target_id) || elem.occurs(target_id),
            Type::Struct(_, generic_args) => generic_args.iter().any(|arg| arg.occurs(target_id)),
            Type::Tuple(fields) => fields.iter().any(|field| field.occurs(target_id)),
            Type::PolymorphicInteger(_, binding)
            | Type::NamedGeneric(binding, _)
            | Type::TypeVariable(binding) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.occurs(target_id),
                TypeBinding::Unbound(id) => *id == target_id,
            },
            Type::Forall(typevars, typ) => {
                !typevars.iter().any(|(id, _)| *id == target_id) && typ.occurs(target_id)
            }
            Type::Function(args, ret) => {
                args.iter().any(|arg| arg.occurs(target_id)) || ret.occurs(target_id)
            }
            Type::Expression(expr) => expr.occurs(target_id),

            Type::FieldElement(_)
            | Type::Integer(_, _, _)
            | Type::Bool(_)
            | Type::Error
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
            Struct(def, args) => {
                let args = vecmap(args, |arg| arg.follow_bindings());
                Struct(def.clone(), args)
            }
            Tuple(args) => Tuple(vecmap(args, |arg| arg.follow_bindings())),

            TypeVariable(var) | PolymorphicInteger(_, var) | NamedGeneric(var, _) => {
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

            Expression(expr) => Expression(expr.follow_bindings()),

            // Expect that this function should only be called on instantiated types
            Forall(..) => unreachable!(),

            FieldElement(_) | Integer(_, _, _) | Bool(_) | Unit | Error => self.clone(),
        }
    }
}

impl TypeExpression {
    fn evaluate_to_u64(&self) -> Option<u64> {
        match self {
            TypeExpression::Constant(value) => Some(*value),
            TypeExpression::TypeVariable(variable) => match &*variable.borrow() {
                TypeBinding::Bound(binding) => binding.evaluate_to_u64(),
                TypeBinding::Unbound(_) => None,
            },
            TypeExpression::BinaryOperation(lhs, op, rhs) => {
                let lhs = lhs.evaluate_to_u64()?;
                let rhs = rhs.evaluate_to_u64()?;
                Some(op.function()(lhs, rhs))
            }
        }
    }

    fn substitute(
        &self,
        type_bindings: &HashMap<TypeVariableId, (Shared<TypeBinding>, Type)>,
    ) -> TypeExpression {
        match self {
            TypeExpression::Constant(x) => TypeExpression::Constant(*x),
            TypeExpression::TypeVariable(var) => match &*var.borrow() {
                TypeBinding::Bound(binding) => Self::from_type(binding.substitute(type_bindings)),
                TypeBinding::Unbound(id) => match type_bindings.get(id) {
                    Some((_, binding)) => Self::from_type(binding.clone()),
                    None => self.clone(),
                },
            },
            TypeExpression::BinaryOperation(lhs, op, rhs) => {
                let lhs = lhs.substitute(type_bindings);
                let rhs = rhs.substitute(type_bindings);
                TypeExpression::BinaryOperation(Box::new(lhs), *op, Box::new(rhs))
            }
        }
    }

    fn from_type(typ: Type) -> TypeExpression {
        match typ {
            Type::TypeVariable(var) | Type::NamedGeneric(var, _) => {
                TypeExpression::TypeVariable(var)
            }
            Type::Expression(expr) => expr,
            other => panic!(
                "Cannot convert type {} into an array length expression - it is not an integer",
                other
            ),
        }
    }

    fn occurs(&self, target_id: TypeVariableId) -> bool {
        match self {
            TypeExpression::Constant(_) => false,
            TypeExpression::TypeVariable(binding) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.occurs(target_id),
                TypeBinding::Unbound(id) => *id == target_id,
            },
            TypeExpression::BinaryOperation(lhs, _, rhs) => {
                lhs.occurs(target_id) || rhs.occurs(target_id)
            }
        }
    }

    fn follow_bindings(&self) -> TypeExpression {
        match self {
            TypeExpression::Constant(x) => TypeExpression::Constant(*x),
            TypeExpression::TypeVariable(var) => match &*var.borrow() {
                TypeBinding::Bound(binding) => Self::from_type(binding.follow_bindings()),
                TypeBinding::Unbound(_) => self.clone(),
            },
            TypeExpression::BinaryOperation(lhs, op, rhs) => {
                let lhs = lhs.follow_bindings();
                let rhs = rhs.follow_bindings();
                TypeExpression::BinaryOperation(Box::new(lhs), *op, Box::new(rhs))
            }
        }
    }

    fn try_unify(&self, other: &TypeExpression, span: Span) -> Result<(), SpanKind> {
        match (self, other) {
            (TypeExpression::TypeVariable(var), other) | (other, TypeExpression::TypeVariable(var)) => {
                if let TypeBinding::Bound(link) = &*var.borrow() {
                    return link.try_unify(&Type::Expression(other.clone()), span);
                }
                Ok(())
            },

            (TypeExpression::Constant(x), TypeExpression::Constant(y)) => {
                if x == y {
                    Ok(())
                } else {
                    Err(SpanKind::None)
                }
            },

            (TypeExpression::BinaryOperation(self_lhs, self_op, self_rhs), 
             TypeExpression::BinaryOperation(other_lhs, other_op, other_rhs)) => {
                if self_op != other_op {
                    return Err(SpanKind::None);
                }

                let mut result = self_lhs.try_unify(other_lhs, span);
                result = result.and_then(|()| self_rhs.try_unify(other_rhs, span));

                // If we got an error and our operator is commutative, try swapping the ordering of
                // the two arguments to see if we succeed. Note that this may still fail in cases
                // like (N + 1 + M) = (1 + M + N) as this would require multiple nested swaps
                // instead of just one at a time.
                if result.is_err() && self_op.commutative() {
                    // No `and_then` here, we want to discard the previous failure.
                    result = self_lhs.try_unify(other_rhs, span);
                    result = result.and_then(|()| self_rhs.try_unify(other_lhs, span));
                }

                result
            },

            (TypeExpression::Constant(_), TypeExpression::BinaryOperation(_, _, _))
            | (TypeExpression::BinaryOperation(_, _, _), TypeExpression::Constant(_)) => {
                // In the future we could try to evaluate both expressions to see if they
                // simplify down to the same expression. For now we just fail.
                Err(SpanKind::None)
            },
        }
    }
}

impl BinaryTypeOperator {
    /// Return the actual rust numeric function associated with this operator
    fn function(self) -> fn(u64, u64) -> u64 {
        match self {
            BinaryTypeOperator::Addition => |a, b| a + b,
            BinaryTypeOperator::Subtraction => |a, b| a - b,
            BinaryTypeOperator::Multiplication => |a, b| a * b,
            BinaryTypeOperator::Division => |a, b| a / b,
            BinaryTypeOperator::Modulo => |a, b| a % b,
        }
    }

    /// True if this operator is commutative
    fn commutative(self) -> bool {
        matches!(self, BinaryTypeOperator::Addition
            | BinaryTypeOperator::Multiplication
        )
    }
}
