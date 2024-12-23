use std::fmt::Display;

use noirc_errors::Span;

use crate::{
    ast::{FunctionReturnType, Ident, Param, Visibility},
    token::{Attributes, FunctionAttribute, SecondaryAttribute},
};

use super::{FunctionDefinition, UnresolvedType, UnresolvedTypeData};

// A NoirFunction can be either a foreign low level function or a function definition
// A closure / function definition will be stored under a name, so we do not differentiate between their variants
// The name for function literal will be the variable it is bound to, and the name for a function definition will
// be the function name itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoirFunction {
    pub kind: FunctionKind,
    pub def: FunctionDefinition,
}

/// Currently, we support four types of functions:
/// - Normal functions
/// - LowLevel/Foreign which link to an OPCODE in ACIR
/// - BuiltIn which are provided by the runtime
/// - TraitFunctionWithoutBody for which we don't type-check their body
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FunctionKind {
    LowLevel,
    Builtin,
    Normal,
    Oracle,
    TraitFunctionWithoutBody,
}

impl FunctionKind {
    pub fn can_ignore_return_type(self) -> bool {
        match self {
            FunctionKind::LowLevel
            | FunctionKind::Builtin
            | FunctionKind::Oracle
            | FunctionKind::TraitFunctionWithoutBody => true,
            FunctionKind::Normal => false,
        }
    }
}

impl NoirFunction {
    pub fn normal(def: FunctionDefinition) -> NoirFunction {
        NoirFunction { kind: FunctionKind::Normal, def }
    }
    pub fn builtin(def: FunctionDefinition) -> NoirFunction {
        NoirFunction { kind: FunctionKind::Builtin, def }
    }
    pub fn low_level(def: FunctionDefinition) -> NoirFunction {
        NoirFunction { kind: FunctionKind::LowLevel, def }
    }
    pub fn oracle(def: FunctionDefinition) -> NoirFunction {
        NoirFunction { kind: FunctionKind::Oracle, def }
    }

    pub fn return_visibility(&self) -> Visibility {
        self.def.return_visibility
    }

    pub fn return_type(&self) -> UnresolvedType {
        match &self.def.return_type {
            FunctionReturnType::Default(span) => UnresolvedTypeData::Unit.with_span(*span),
            FunctionReturnType::Ty(ty) => ty.clone(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name_ident().0.contents
    }
    pub fn name_ident(&self) -> &Ident {
        &self.def.name
    }
    pub fn parameters(&self) -> &[Param] {
        &self.def.parameters
    }
    pub fn attributes(&self) -> &Attributes {
        &self.def.attributes
    }
    pub fn function_attribute(&self) -> Option<&FunctionAttribute> {
        self.def.attributes.function()
    }
    pub fn secondary_attributes(&self) -> &[SecondaryAttribute] {
        self.def.attributes.secondary.as_ref()
    }
    pub fn def(&self) -> &FunctionDefinition {
        &self.def
    }
    pub fn def_mut(&mut self) -> &mut FunctionDefinition {
        &mut self.def
    }
    pub fn number_of_statements(&self) -> usize {
        self.def.body.statements.len()
    }
    pub fn span(&self) -> Span {
        self.def.span
    }

    pub fn foreign(&self) -> Option<&FunctionDefinition> {
        match &self.kind {
            FunctionKind::LowLevel => {}
            _ => return None,
        }
        assert!(self.function_attribute().unwrap().is_foreign());
        Some(&self.def)
    }
}

impl From<FunctionDefinition> for NoirFunction {
    fn from(fd: FunctionDefinition) -> Self {
        // The function type is determined by the existence of a function attribute
        let kind = match fd.attributes.function() {
            Some(FunctionAttribute::Builtin(_)) => FunctionKind::Builtin,
            Some(FunctionAttribute::Foreign(_)) => FunctionKind::LowLevel,
            Some(FunctionAttribute::Test { .. }) => FunctionKind::Normal,
            Some(FunctionAttribute::Oracle(_)) => FunctionKind::Oracle,
            Some(FunctionAttribute::Fold) => FunctionKind::Normal,
            Some(FunctionAttribute::NoPredicates) => FunctionKind::Normal,
            Some(FunctionAttribute::InlineAlways) => FunctionKind::Normal,
            None => FunctionKind::Normal,
        };

        NoirFunction { def: fd, kind }
    }
}

impl Display for NoirFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.def.fmt(f)
    }
}
