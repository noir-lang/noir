use std::fmt::Display;

use crate::{token::Attribute, FunctionReturnType, Ident, Pattern, Visibility};

use super::{FunctionDefinition, UnresolvedType};

// A NoirFunction can be either a foreign low level function or a function definition
// A closure / function definition will be stored under a name, so we do not differentiate between their variants
// The name for function literal will be the variable it is bound to, and the name for a function definition will
// be the function name itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoirFunction {
    pub kind: FunctionKind,
    pub def: FunctionDefinition,
}

/// Currently, we support three types of functions:
/// - Normal functions
/// - LowLevel/Foreign which link to an OPCODE in ACIR
/// - BuiltIn which are provided by the runtime
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FunctionKind {
    LowLevel,
    Builtin,
    Normal,
    Oracle,
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

    pub fn return_type(&self) -> UnresolvedType {
        match &self.def.return_type {
            FunctionReturnType::Default(_) => UnresolvedType::Unit,
            FunctionReturnType::Ty(ty, _) => ty.clone(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name_ident().0.contents
    }
    pub fn name_ident(&self) -> &Ident {
        &self.def.name
    }
    pub fn parameters(&self) -> &Vec<(Pattern, UnresolvedType, Visibility)> {
        &self.def.parameters
    }
    pub fn attribute(&self) -> Option<&Attribute> {
        self.def.attribute.as_ref()
    }
    pub fn def(&self) -> &FunctionDefinition {
        &self.def
    }
    pub fn def_mut(&mut self) -> &mut FunctionDefinition {
        &mut self.def
    }
    pub fn number_of_statements(&self) -> usize {
        self.def.body.0.len()
    }

    pub fn foreign(&self) -> Option<&FunctionDefinition> {
        match &self.kind {
            FunctionKind::LowLevel => {}
            _ => return None,
        }
        assert!(self.attribute().unwrap().is_foreign());
        Some(&self.def)
    }
}

impl From<FunctionDefinition> for NoirFunction {
    fn from(fd: FunctionDefinition) -> Self {
        let kind = match fd.attribute {
            Some(Attribute::Builtin(_)) => FunctionKind::Builtin,
            Some(Attribute::Foreign(_)) => FunctionKind::LowLevel,
            Some(Attribute::Test) => FunctionKind::Normal,
            Some(Attribute::Oracle(_)) => FunctionKind::Oracle,
            Some(Attribute::Deprecated(_)) | None => FunctionKind::Normal,
        };

        NoirFunction { def: fd, kind }
    }
}

impl Display for NoirFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.def.fmt(f)
    }
}
