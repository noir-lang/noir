use iter_extended::vecmap;
use noirc_errors::{Location, Span};

use super::expr::{HirBlockExpression, HirExpression, HirIdent};
use super::stmt::HirPattern;
use super::traits::TraitConstraint;
use crate::node_interner::{ExprId, NodeInterner, TraitImplId};
use crate::FunctionKind;
use crate::{Distinctness, FunctionReturnType, Type, Visibility};

/// A Hir function is a block expression
/// with a list of statements
#[derive(Debug, Clone)]
pub struct HirFunction(ExprId);

impl HirFunction {
    pub fn empty() -> HirFunction {
        HirFunction(ExprId::empty_block_id())
    }

    pub const fn unchecked_from_expr(expr_id: ExprId) -> HirFunction {
        HirFunction(expr_id)
    }

    pub const fn as_expr(&self) -> &ExprId {
        &self.0
    }

    pub fn block(&self, interner: &NodeInterner) -> HirBlockExpression {
        match interner.expression(&self.0) {
            HirExpression::Block(block_expr) => block_expr,
            _ => unreachable!("ice: functions can only be block expressions"),
        }
    }
}

/// An interned function parameter from a function definition
pub type Param = (HirPattern, Type, Visibility);

#[derive(Debug, Clone)]
pub struct Parameters(pub Vec<Param>);

impl Parameters {
    pub fn span(&self) -> Span {
        assert!(!self.is_empty());
        let mut spans = vecmap(&self.0, |param| param.0.span());

        let merged_span = spans.pop().unwrap();
        for span in spans {
            let _ = merged_span.merge(span);
        }

        merged_span
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Param> {
        self.0.iter()
    }
}

impl IntoIterator for Parameters {
    type Item = Param;
    type IntoIter = <Vec<Param> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<Param>> for Parameters {
    fn from(vec: Vec<Param>) -> Parameters {
        Parameters(vec)
    }
}

pub type FunctionSignature = (Vec<Param>, Option<Type>);

/// A FuncMeta contains the signature of the function and any associated meta data like
/// the function's Location, FunctionKind, and attributes. If the function's body is
/// needed, it can be retrieved separately via `NodeInterner::function(&self, &FuncId)`.
#[derive(Debug, Clone)]
pub struct FuncMeta {
    pub name: HirIdent,

    pub kind: FunctionKind,

    pub parameters: Parameters,

    pub return_type: FunctionReturnType,

    pub return_visibility: Visibility,

    pub return_distinctness: Distinctness,

    /// The type of this function. Either a Type::Function
    /// or a Type::Forall for generic functions.
    pub typ: Type,

    pub location: Location,

    // This flag is needed for the attribute check pass
    pub has_body: bool,

    pub trait_constraints: Vec<TraitConstraint>,

    /// The trait impl this function belongs to, if any
    pub trait_impl: Option<TraitImplId>,
}

impl FuncMeta {
    /// Builtin, LowLevel and Oracle functions usually have the return type
    /// declared, however their function bodies will be empty
    /// So this method tells the type checker to ignore the return
    /// of the empty function, which is unit
    pub fn can_ignore_return_type(&self) -> bool {
        match self.kind {
            FunctionKind::LowLevel | FunctionKind::Builtin | FunctionKind::Oracle => true,
            FunctionKind::Normal | FunctionKind::Recursive => false,
        }
    }

    pub fn function_signature(&self) -> FunctionSignature {
        let return_type = match self.return_type() {
            Type::Unit => None,
            typ => Some(typ.clone()),
        };
        (self.parameters.0.clone(), return_type)
    }

    /// Gives the (uninstantiated) return type of this function.
    pub fn return_type(&self) -> &Type {
        match &self.typ {
            Type::Function(_, ret, _env) => ret,
            Type::Forall(_, typ) => match typ.as_ref() {
                Type::Function(_, ret, _env) => ret,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}
