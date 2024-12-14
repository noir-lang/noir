use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Location, Span};

use super::expr::{HirBlockExpression, HirExpression, HirIdent};
use super::stmt::HirPattern;
use super::traits::TraitConstraint;
use crate::ast::{BlockExpression, FunctionKind, FunctionReturnType, Visibility};
use crate::graph::CrateId;
use crate::hir::def_map::LocalModuleId;
use crate::node_interner::{ExprId, NodeInterner, StructId, TraitId, TraitImplId};

use crate::{ResolvedGeneric, Type};

/// A Hir function is a block expression with a list of statements.
/// If the function has yet to be resolved, the body starts off empty (None).
#[derive(Debug, Clone)]
pub struct HirFunction(Option<ExprId>);

impl HirFunction {
    pub fn empty() -> HirFunction {
        HirFunction(None)
    }

    pub const fn unchecked_from_expr(expr_id: ExprId) -> HirFunction {
        HirFunction(Some(expr_id))
    }

    pub fn as_expr(&self) -> ExprId {
        self.0.expect("Function has yet to be elaborated, cannot get an ExprId of its body!")
    }

    pub fn try_as_expr(&self) -> Option<ExprId> {
        self.0
    }

    pub fn block(&self, interner: &NodeInterner) -> HirBlockExpression {
        match interner.expression(&self.as_expr()) {
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

    /// The HirIdent of each identifier within the parameter list.
    /// Note that this includes separate entries for each identifier in e.g. tuple patterns.
    pub parameter_idents: Vec<HirIdent>,

    pub return_type: FunctionReturnType,

    pub return_visibility: Visibility,

    /// The type of this function. Either a Type::Function
    /// or a Type::Forall for generic functions.
    pub typ: Type,

    /// The set of generics that are declared directly on this function in the source code.
    /// This does not include generics from an outer scope, like those introduced by
    /// an `impl<T>` block. This also does not include implicit generics added by the compiler
    /// such as a trait's `Self` type variable.
    pub direct_generics: Vec<ResolvedGeneric>,

    /// All the generics used by this function, which includes any implicit generics or generics
    /// from outer scopes, such as those introduced by an impl.
    /// This is stored when the FuncMeta is first created to later be used to set the current
    /// generics when the function's body is later resolved.
    pub all_generics: Vec<ResolvedGeneric>,

    pub location: Location,

    // This flag is needed for the attribute check pass
    pub has_body: bool,

    pub trait_constraints: Vec<TraitConstraint>,

    /// The struct this function belongs to, if any
    pub struct_id: Option<StructId>,

    // The trait this function belongs to, if any
    pub trait_id: Option<TraitId>,

    /// The trait impl this function belongs to, if any
    pub trait_impl: Option<TraitImplId>,

    /// True if this function is an entry point to the program.
    /// For non-contracts, this means the function is `main`.
    pub is_entry_point: bool,

    /// True if this function is marked with an attribute
    /// that indicates it should be inlined differently than the default (inline everything).
    /// For example, such as `fold` (never inlined) or `no_predicates` (inlined after flattening)
    pub has_inline_attribute: bool,

    pub function_body: FunctionBody,

    /// The crate this function was defined in
    pub source_crate: CrateId,

    /// The module this function was defined in
    pub source_module: LocalModuleId,

    /// THe file this function was defined in
    pub source_file: FileId,

    /// If this function is from an impl (trait or regular impl), this
    /// is the object type of the impl. Otherwise this is None.
    pub self_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub enum FunctionBody {
    Unresolved(FunctionKind, BlockExpression, Span),
    Resolving,
    Resolved,
}

impl FuncMeta {
    /// A stub function does not have a body. This includes Builtin, LowLevel,
    /// and Oracle functions in addition to method declarations within a trait.
    ///
    /// We don't check the return type of these functions since it will always have
    /// an empty body, and we don't check for unused parameters.
    pub fn is_stub(&self) -> bool {
        self.kind.can_ignore_return_type() || self.trait_id.is_some()
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
            Type::Function(_, ret, _env, _unconstrained) => ret,
            Type::Forall(_, typ) => match typ.as_ref() {
                Type::Function(_, ret, _env, _unconstrained) => ret,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    /// Take this function body, returning an owned version while avoiding
    /// cloning any large Expressions inside by replacing a Unresolved with a Resolving variant.
    pub fn take_body(&mut self) -> FunctionBody {
        match &mut self.function_body {
            FunctionBody::Unresolved(kind, block, span) => {
                let statements = std::mem::take(&mut block.statements);
                let (kind, span) = (*kind, *span);
                self.function_body = FunctionBody::Resolving;
                FunctionBody::Unresolved(kind, BlockExpression { statements }, span)
            }
            FunctionBody::Resolving => FunctionBody::Resolving,
            FunctionBody::Resolved => FunctionBody::Resolved,
        }
    }
}
