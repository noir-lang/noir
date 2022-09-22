use noirc_abi::{Abi, AbiFEType};
use noirc_errors::{Location, Span};

use super::expr::{HirBlockExpression, HirExpression, HirIdent};
use super::stmt::HirPattern;
use crate::node_interner::{ExprId, NodeInterner};
use crate::util::vecmap;
use crate::Type;
use crate::{token::Attribute, FunctionKind};

/// A Hir function is a block expression
/// with a list of statements
#[derive(Debug, Clone)]
pub struct HirFunction(ExprId);

const MAIN_RETURN_NAME: &str = "return";

impl HirFunction {
    pub fn empty() -> HirFunction {
        HirFunction(ExprId::empty_block_id())
    }

    // This function is marked as unsafe because
    // the expression kind is not being checked
    pub const fn unsafe_from_expr(expr_id: ExprId) -> HirFunction {
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
#[derive(Debug, Clone)]
pub struct Param(pub HirPattern, pub Type, pub noirc_abi::AbiFEType);

/// Attempts to retrieve the name of this parameter. Returns None
/// if this parameter is a tuple or struct pattern.
fn get_param_name<'a>(pattern: &HirPattern, interner: &'a NodeInterner) -> Option<&'a str> {
    match pattern {
        HirPattern::Identifier(ident) => Some(interner.definition_name(ident.id)),
        HirPattern::Mutable(pattern, _) => get_param_name(pattern, interner),
        HirPattern::Tuple(_, _) => None,
        HirPattern::Struct(_, _, _) => None,
    }
}

#[derive(Debug, Clone)]
pub struct Parameters(pub Vec<Param>);

impl Parameters {
    fn into_abi(self, interner: &NodeInterner) -> Abi {
        let parameters = vecmap(self.0, |param| {
            let param_name = get_param_name(&param.0, interner)
                .expect("Abi for tuple and struct parameters is unimplemented")
                .to_owned();
            (param_name, param.1.as_abi_type(param.2))
        });
        noirc_abi::Abi { parameters }
    }

    pub fn span(&self) -> Span {
        assert!(!self.is_empty());
        let mut spans = vecmap(&self.0, |param| match &param.0 {
            HirPattern::Identifier(ident) => ident.location.span,
            HirPattern::Mutable(_, span) => *span,
            HirPattern::Tuple(_, span) => *span,
            HirPattern::Struct(_, _, span) => *span,
        });

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

#[derive(Debug, Clone)]
pub struct FuncMeta {
    pub name: HirIdent,

    pub kind: FunctionKind,

    pub attributes: Option<Attribute>,
    pub parameters: Parameters,
    pub return_visibility: AbiFEType,

    /// The type of this function. Either a Type::Function
    /// or a Type::Forall for generic functions.
    pub typ: Type,

    pub location: Location,

    // This flag is needed for the attribute check pass
    pub has_body: bool,
}

impl FuncMeta {
    /// Builtin and LowLevel functions usually have the return type
    /// declared, however their function bodies will be empty
    /// So this method tells the type checker to ignore the return
    /// of the empty function, which is unit
    pub fn can_ignore_return_type(&self) -> bool {
        match self.kind {
            FunctionKind::LowLevel | FunctionKind::Builtin => true,
            FunctionKind::Normal => false,
        }
    }

    pub fn into_abi(self, interner: &NodeInterner) -> Abi {
        let return_type = self.return_type().clone();
        let mut abi = self.parameters.into_abi(interner);

        if return_type != Type::Unit {
            let typ = return_type.as_abi_type(self.return_visibility);
            abi.parameters.push((MAIN_RETURN_NAME.into(), typ));
        }

        abi
    }

    /// Gives the (uninstantiated) return type of this function.
    pub fn return_type(&self) -> &Type {
        match &self.typ {
            Type::Function(_, ret) => ret,
            Type::Forall(_, typ) => match typ.as_ref() {
                Type::Function(_, ret) => ret,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}
