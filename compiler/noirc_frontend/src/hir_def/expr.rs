use acvm::FieldElement;
use fm::FileId;
use noirc_errors::Location;

use crate::ast::{BinaryOp, BinaryOpKind, Ident, UnaryOp};
use crate::hir::type_check::generics::TraitGenerics;
use crate::node_interner::{
    DefinitionId, DefinitionKind, ExprId, FuncId, NodeInterner, StmtId, TraitMethodId,
};
use crate::token::{FmtStrFragment, Tokens};
use crate::Shared;

use super::stmt::HirPattern;
use super::traits::{ResolvedTraitBound, TraitConstraint};
use super::types::{DataType, Type};

/// A HirExpression is the result of an Expression in the AST undergoing
/// name resolution. It is almost identical to the Expression AST node, but
/// references other HIR nodes indirectly via IDs rather than directly via
/// boxing. Variables in HirExpressions are tagged with their DefinitionId
/// from the definition that refers to them so there is no ambiguity with names.
#[derive(Debug, Clone)]
pub enum HirExpression {
    // The optional vec here is the optional list of generics
    // provided by the turbofish operator, if it was used
    Ident(HirIdent, Option<Vec<Type>>),
    Literal(HirLiteral),
    Block(HirBlockExpression),
    Prefix(HirPrefixExpression),
    Infix(HirInfixExpression),
    Index(HirIndexExpression),
    Constructor(HirConstructorExpression),
    EnumConstructor(HirEnumConstructorExpression),
    MemberAccess(HirMemberAccess),
    Call(HirCallExpression),
    MethodCall(HirMethodCallExpression),
    Constrain(HirConstrainExpression),
    Cast(HirCastExpression),
    If(HirIfExpression),
    Match(HirMatch),
    Tuple(Vec<ExprId>),
    Lambda(HirLambda),
    Quote(Tokens),
    Unquote(Tokens),
    Comptime(HirBlockExpression),
    Unsafe(HirBlockExpression),
    Error,
}

/// Corresponds to a variable in the source code
#[derive(Debug, Clone)]
pub struct HirIdent {
    pub location: Location,
    pub id: DefinitionId,

    /// If this HirIdent refers to a trait method, this field stores
    /// whether the impl for this method is known or not.
    pub impl_kind: ImplKind,
}

impl HirIdent {
    pub fn non_trait_method(id: DefinitionId, location: Location) -> Self {
        Self { id, location, impl_kind: ImplKind::NotATraitMethod }
    }
}

#[derive(Debug, Clone)]
pub enum ImplKind {
    /// This ident is not a trait method
    NotATraitMethod,

    /// This ident refers to a trait method and its impl needs to be verified,
    /// and eventually linked to this id. The boolean indicates whether the impl
    /// is already assumed to exist - e.g. when resolving a path such as `T::default`
    /// when there is a corresponding `T: Default` constraint in scope.
    TraitMethod(TraitMethod),
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub method_id: TraitMethodId,
    pub constraint: TraitConstraint,
    pub assumed: bool,
}

impl Eq for HirIdent {}
impl PartialEq for HirIdent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::hash::Hash for HirIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HirBinaryOp {
    pub kind: BinaryOpKind,
    pub location: Location,
}

impl HirBinaryOp {
    pub fn new(op: BinaryOp, file: FileId) -> Self {
        let kind = op.contents;
        let location = Location::new(op.span(), file);
        HirBinaryOp { location, kind }
    }
}

#[derive(Debug, Clone)]
pub enum HirLiteral {
    Array(HirArrayLiteral),
    Slice(HirArrayLiteral),
    Bool(bool),
    Integer(FieldElement, bool), //true for negative integer and false for positive
    Str(String),
    FmtStr(Vec<FmtStrFragment>, Vec<ExprId>, u32 /* length */),
    Unit,
}

#[derive(Debug, Clone)]
pub enum HirArrayLiteral {
    Standard(Vec<ExprId>),
    Repeated { repeated_element: ExprId, length: Type },
}

#[derive(Debug, Clone)]
pub struct HirPrefixExpression {
    pub operator: UnaryOp,
    pub rhs: ExprId,

    /// The trait method id for the operator trait method that corresponds to this operator,
    /// if such a trait exists (for example, there's no trait for the dereference operator).
    pub trait_method_id: Option<TraitMethodId>,
}

#[derive(Debug, Clone)]
pub struct HirInfixExpression {
    pub lhs: ExprId,
    pub operator: HirBinaryOp,
    pub rhs: ExprId,

    /// The trait method id for the operator trait method that corresponds to this operator.
    /// For derived operators like `!=`, this will lead to the method `Eq::eq`. For these
    /// cases, it is up to the monomorphization pass to insert the appropriate `not` operation
    /// after the call to `Eq::eq` to get the result of the `!=` operator.
    pub trait_method_id: TraitMethodId,
}

/// This is always a struct field access `my_struct.field`
/// and never a method call. The later is represented by HirMethodCallExpression.
#[derive(Debug, Clone)]
pub struct HirMemberAccess {
    pub lhs: ExprId,
    // This field is not an IdentId since the rhs of a field
    // access has no corresponding definition
    pub rhs: Ident,

    /// True if we should return an offset of the field rather than the field itself.
    /// For most cases this is false, corresponding to `foo.bar` in source code.
    /// This is true when calling methods or when we have an lvalue we want to preserve such
    /// that if `foo : &mut Foo` has a field `bar : Bar`, we can return an `&mut Bar`.
    pub is_offset: bool,
}

#[derive(Debug, Clone)]
pub struct HirIfExpression {
    pub condition: ExprId,
    pub consequence: ExprId,
    pub alternative: Option<ExprId>,
}

// `lhs as type` in the source code
#[derive(Debug, Clone)]
pub struct HirCastExpression {
    pub lhs: ExprId,
    pub r#type: Type,
}

#[derive(Debug, Clone)]
pub struct HirCallExpression {
    pub func: ExprId,
    pub arguments: Vec<ExprId>,
    pub location: Location,
    pub is_macro_call: bool,
}

/// These nodes are temporary, they're
/// lowered into HirCallExpression nodes
/// after type checking resolves the object
/// type and the method it calls.
#[derive(Debug, Clone)]
pub struct HirMethodCallExpression {
    pub method: Ident,
    pub object: ExprId,
    /// Method calls have an optional list of generics provided by the turbofish operator
    pub generics: Option<Vec<Type>>,
    pub arguments: Vec<ExprId>,
    pub location: Location,
}

/// Corresponds to `assert` and `assert_eq` in the source code.
/// This node also contains the FileId of the file the constrain
/// originates from. This is used later in the SSA pass to issue
/// an error if a constrain is found to be always false.
#[derive(Debug, Clone)]
pub struct HirConstrainExpression(pub ExprId, pub FileId, pub Option<ExprId>);

#[derive(Debug, Clone)]
pub enum HirMethodReference {
    /// A method can be defined in a regular `impl` block, in which case
    /// it's syntax sugar for a normal function call, and can be
    /// translated to one during type checking
    FuncId(FuncId),

    /// Or a method can come from a Trait impl block, in which case
    /// the actual function called will depend on the instantiated type,
    /// which can be only known during monomorphization.
    TraitMethodId(TraitMethodId, TraitGenerics, bool /* assumed */),
}

impl HirMethodReference {
    pub fn func_id(&self, interner: &NodeInterner) -> Option<FuncId> {
        match self {
            HirMethodReference::FuncId(func_id) => Some(*func_id),
            HirMethodReference::TraitMethodId(method_id, _, _) => {
                let id = interner.trait_method_id(*method_id);
                match &interner.try_definition(id)?.kind {
                    DefinitionKind::Function(func_id) => Some(*func_id),
                    _ => None,
                }
            }
        }
    }

    pub fn into_function_id_and_name(
        self,
        object_type: Type,
        generics: Option<Vec<Type>>,
        location: Location,
        interner: &mut NodeInterner,
    ) -> (ExprId, HirIdent) {
        let (id, impl_kind) = match self {
            HirMethodReference::FuncId(func_id) => {
                (interner.function_definition_id(func_id), ImplKind::NotATraitMethod)
            }
            HirMethodReference::TraitMethodId(method_id, trait_generics, assumed) => {
                let id = interner.trait_method_id(method_id);
                let constraint = TraitConstraint {
                    typ: object_type,
                    trait_bound: ResolvedTraitBound {
                        trait_id: method_id.trait_id,
                        trait_generics,
                        span: location.span,
                    },
                };

                (id, ImplKind::TraitMethod(TraitMethod { method_id, constraint, assumed }))
            }
        };
        let func_var = HirIdent { location, id, impl_kind };
        let func = interner.push_expr(HirExpression::Ident(func_var.clone(), generics));
        interner.push_expr_location(func, location.span, location.file);
        (func, func_var)
    }
}

impl HirMethodCallExpression {
    pub fn into_function_call(
        mut self,
        func: ExprId,
        is_macro_call: bool,
        location: Location,
    ) -> HirCallExpression {
        let mut arguments = vec![self.object];
        arguments.append(&mut self.arguments);
        HirCallExpression { func, arguments, location, is_macro_call }
    }
}

#[derive(Debug, Clone)]
pub struct HirConstructorExpression {
    pub r#type: Shared<DataType>,
    pub struct_generics: Vec<Type>,

    // NOTE: It is tempting to make this a BTreeSet to force ordering of field
    //       names (and thus remove the need to normalize them during type checking)
    //       but doing so would force the order of evaluation of field
    //       arguments to be alphabetical rather than the ordering the user
    //       included in the source code.
    pub fields: Vec<(Ident, ExprId)>,
}

/// An enum constructor is an expression such as `Option::Some(foo)`
/// to construct an enum. These are usually inserted by the compiler itself
/// since `Some` is actually a function with the body implicitly being an
/// enum constructor expression, but in the future these may be directly
/// represented when using enums with named fields.
///
/// During monomorphization, these expressions are translated to tuples of
/// (tag, variant0_fields, variant1_fields, ..) since we cannot actually
/// make a true union in a circuit.
#[derive(Debug, Clone)]
pub struct HirEnumConstructorExpression {
    pub r#type: Shared<DataType>,
    pub variant_index: usize,

    /// This refers to just the arguments that are passed. E.g. just
    /// `foo` in `Foo::Bar(foo)`, even if other variants have their
    /// "fields" defaulted to `std::mem::zeroed`, these aren't specified
    /// at this step.
    pub arguments: Vec<ExprId>,
}

/// Indexing, as in `array[index]`
#[derive(Debug, Clone)]
pub struct HirIndexExpression {
    pub collection: ExprId,
    pub index: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirBlockExpression {
    pub statements: Vec<StmtId>,
}

impl HirBlockExpression {
    pub fn statements(&self) -> &[StmtId] {
        &self.statements
    }
}

/// A variable captured inside a closure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirCapturedVar {
    pub ident: HirIdent,

    /// This will be None when the capture refers to a local variable declared
    /// in the same scope as the closure. In a closure-inside-another-closure
    /// scenarios, we might have a transitive captures of variables that must
    /// be propagated during the construction of each closure. In this case,
    /// we store the index of the captured variable in the environment of our
    /// direct parent closure. We do this in order to simplify the HIR to AST
    /// transformation in the monomorphization pass.
    pub transitive_capture_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirLambda {
    pub parameters: Vec<(HirPattern, Type)>,
    pub return_type: Type,
    pub body: ExprId,
    pub captures: Vec<HirCapturedVar>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirMatch {
    /// Jump directly to ExprId
    Success(ExprId),

    Failure,

    /// Run `body` if the given expression is true.
    /// Otherwise continue with the given decision tree.
    Guard {
        cond: ExprId,
        body: ExprId,
        otherwise: Box<HirMatch>,
    },

    /// Switch on the given variable with the given cases to test.
    /// The final argument is an optional match-all case to take if
    /// none of the cases matched.
    Switch(DefinitionId, Vec<Case>, Option<Box<HirMatch>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Case {
    pub constructor: Constructor,
    pub arguments: Vec<DefinitionId>,
    pub body: HirMatch,
}

impl Case {
    pub fn new(constructor: Constructor, arguments: Vec<DefinitionId>, body: HirMatch) -> Self {
        Self { constructor, arguments, body }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SignedField {
    pub field: FieldElement,
    pub is_negative: bool,
}

impl SignedField {
    pub fn new(field: FieldElement, is_negative: bool) -> Self {
        Self { field, is_negative }
    }
}

impl std::ops::Neg for SignedField {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.is_negative = !self.is_negative;
        self
    }
}

impl std::cmp::PartialOrd for SignedField {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.is_negative != other.is_negative {
            if self.is_negative {
                return Some(std::cmp::Ordering::Less);
            } else {
                return Some(std::cmp::Ordering::Greater);
            }
        }
        self.field.partial_cmp(&other.field)
    }
}

impl std::fmt::Display for SignedField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_negative {
            write!(f, "-")?;
        }
        write!(f, "{}", self.field)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Constructor {
    True,
    False,
    Unit,
    Int(SignedField),
    Tuple(Vec<Type>),
    Variant(Type, usize),
    Range(SignedField, SignedField),
}

impl Constructor {
    pub fn variant_index(&self) -> usize {
        match self {
            Constructor::False
            | Constructor::Int(_)
            | Constructor::Unit
            | Constructor::Tuple(_)
            | Constructor::Range(_, _) => 0,
            Constructor::True => 1,
            Constructor::Variant(_, index) => *index,
        }
    }

    /// True if this constructor constructs an enum value.
    /// Enums contain a tag value and often have values to
    /// unpack for each different variant index.
    pub fn is_enum(&self) -> bool {
        match self {
            Constructor::Variant(typ, _) => match typ.follow_bindings_shallow().as_ref() {
                Type::DataType(def, _) => def.borrow().is_enum(),
                _ => false,
            },
            _ => false,
        }
    }

    /// True if this constructor constructs a tuple or struct value.
    /// Tuples or structs will still have values to unpack but do not
    /// store a tag value internally.
    pub fn is_tuple_or_struct(&self) -> bool {
        match self {
            Constructor::Tuple(_) => true,
            Constructor::Variant(typ, _) => match typ.follow_bindings_shallow().as_ref() {
                Type::DataType(def, _) => def.borrow().is_struct(),
                _ => false,
            },
            _ => false,
        }
    }
}

impl std::fmt::Display for Constructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constructor::True => write!(f, "true"),
            Constructor::False => write!(f, "false"),
            Constructor::Unit => write!(f, "()"),
            Constructor::Int(x) => write!(f, "{x}"),
            Constructor::Tuple(_) => Ok(()), // args are printed after already
            Constructor::Variant(typ, variant_index) => {
                if let Type::DataType(def, _) = typ {
                    let def = def.borrow();
                    if let Some(variant) = def.get_variant_as_written(*variant_index) {
                        write!(f, "{}", variant.name)?;
                    } else if def.is_struct() {
                        write!(f, "{}", def.name)?;
                    }
                }
                Ok(())
            }
            Constructor::Range(start, end) => write!(f, "{start} .. {end}"),
        }
    }
}
