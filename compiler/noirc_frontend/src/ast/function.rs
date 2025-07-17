use std::fmt::Display;

use noirc_errors::Location;

use crate::{
    ast::{FunctionReturnType, Ident, Param},
    token::{Attributes, FunctionAttributeKind, SecondaryAttribute},
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

    pub fn return_type(&self) -> UnresolvedType {
        match &self.def.return_type {
            FunctionReturnType::Default(location) => {
                UnresolvedTypeData::Unit.with_location(*location)
            }
            FunctionReturnType::Ty(ty) => ty.clone(),
        }
    }
    pub fn name(&self) -> &str {
        self.name_ident().as_str()
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
    pub fn secondary_attributes(&self) -> &[SecondaryAttribute] {
        self.def.attributes.secondary.as_ref()
    }
    pub fn location(&self) -> Location {
        self.def.location
    }
}

impl From<FunctionDefinition> for NoirFunction {
    fn from(fd: FunctionDefinition) -> Self {
        // The function type is determined by the existence of a function attribute
        let kind = match fd.attributes.function().map(|attr| &attr.kind) {
            Some(FunctionAttributeKind::Builtin(_)) => FunctionKind::Builtin,
            Some(FunctionAttributeKind::Foreign(_)) => FunctionKind::LowLevel,
            Some(FunctionAttributeKind::Test { .. }) => FunctionKind::Normal,
            Some(FunctionAttributeKind::FuzzingHarness { .. }) => FunctionKind::Normal,
            Some(FunctionAttributeKind::Oracle(_)) => FunctionKind::Oracle,
            Some(FunctionAttributeKind::Fold) => FunctionKind::Normal,
            Some(FunctionAttributeKind::NoPredicates) => FunctionKind::Normal,
            Some(FunctionAttributeKind::InlineAlways) => FunctionKind::Normal,
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
