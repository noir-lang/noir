use acvm::FieldElement;
use fm::FileId;
use noirc_errors::Location;

use crate::ast::{BinaryOp, BinaryOpKind, Ident, UnaryOp};
use crate::node_interner::{DefinitionId, ExprId, FuncId, NodeInterner, StmtId, TraitMethodId};
use crate::Shared;

use super::stmt::HirPattern;
use super::traits::TraitConstraint;
use super::types::{StructType, Type};

/// A HirExpression is the result of an Expression in the AST undergoing
/// name resolution. It is almost identical to the Expression AST node, but
/// references other HIR nodes indirectly via IDs rather than directly via
/// boxing. Variables in HirExpressions are tagged with their DefinitionId
/// from the definition that refers to them so there is no ambiguity with names.
#[derive(Debug, Clone)]
pub enum HirExpression {
    Ident(HirIdent),
    Literal(HirLiteral),
    Block(HirBlockExpression),
    Prefix(HirPrefixExpression),
    Infix(HirInfixExpression),
    Index(HirIndexExpression),
    Constructor(HirConstructorExpression),
    MemberAccess(HirMemberAccess),
    Call(HirCallExpression),
    MethodCall(HirMethodCallExpression),
    Cast(HirCastExpression),
    If(HirIfExpression),
    Tuple(Vec<ExprId>),
    Lambda(HirLambda),
    Quote(crate::ast::BlockExpression),
    Unquote(crate::ast::BlockExpression),
    Comptime(HirBlockExpression),
    Error,
}

impl HirExpression {
    /// Returns an empty block expression
    pub const fn empty_block() -> HirExpression {
        HirExpression::Block(HirBlockExpression { statements: vec![] })
    }
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
    TraitMethod(TraitMethodId, TraitConstraint, bool),
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
    FmtStr(String, Vec<ExprId>),
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
}

/// These nodes are temporary, they're
/// lowered into HirCallExpression nodes
/// after type checking resolves the object
/// type and the method it calls.
#[derive(Debug, Clone)]
pub struct HirMethodCallExpression {
    pub method: Ident,
    pub object: ExprId,
    pub arguments: Vec<ExprId>,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub enum HirMethodReference {
    /// A method can be defined in a regular `impl` block, in which case
    /// it's syntax sugar for a normal function call, and can be
    /// translated to one during type checking
    FuncId(FuncId),

    /// Or a method can come from a Trait impl block, in which case
    /// the actual function called will depend on the instantiated type,
    /// which can be only known during monomorphization.
    TraitMethodId(TraitMethodId, /*trait generics:*/ Vec<Type>),
}

impl HirMethodCallExpression {
    /// Converts a method call into a function call
    ///
    /// Returns ((func_var_id, func_var), call_expr)
    pub fn into_function_call(
        mut self,
        method: &HirMethodReference,
        object_type: Type,
        location: Location,
        interner: &mut NodeInterner,
    ) -> ((ExprId, HirIdent), HirCallExpression) {
        let mut arguments = vec![self.object];
        arguments.append(&mut self.arguments);

        let (id, impl_kind) = match method {
            HirMethodReference::FuncId(func_id) => {
                (interner.function_definition_id(*func_id), ImplKind::NotATraitMethod)
            }
            HirMethodReference::TraitMethodId(method_id, generics) => {
                let id = interner.trait_method_id(*method_id);
                let constraint = TraitConstraint {
                    typ: object_type,
                    trait_id: method_id.trait_id,
                    trait_generics: generics.clone(),
                };
                (id, ImplKind::TraitMethod(*method_id, constraint, false))
            }
        };
        let func_var = HirIdent { location, id, impl_kind };
        let func = interner.push_expr(HirExpression::Ident(func_var.clone()));
        interner.push_expr_location(func, location.span, location.file);
        let expr = HirCallExpression { func, arguments, location };
        ((func, func_var), expr)
    }
}

#[derive(Debug, Clone)]
pub struct HirConstructorExpression {
    pub r#type: Shared<StructType>,
    pub struct_generics: Vec<Type>,

    // NOTE: It is tempting to make this a BTreeSet to force ordering of field
    //       names (and thus remove the need to normalize them during type checking)
    //       but doing so would force the order of evaluation of field
    //       arguments to be alphabetical rather than the ordering the user
    //       included in the source code.
    pub fields: Vec<(Ident, ExprId)>,
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
