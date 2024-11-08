use acvm::FieldElement;

use crate::ssa::ir::{function::RuntimeType, types::Type};

#[derive(Debug)]
pub(crate) struct ParsedSsa {
    pub(crate) functions: Vec<ParsedFunction>,
}

#[derive(Debug)]
pub(crate) struct ParsedFunction {
    pub(crate) runtime_type: RuntimeType,
    pub(crate) external_name: String,
    pub(crate) internal_name: String,
    pub(crate) blocks: Vec<ParsedBlock>,
}

#[derive(Debug)]
pub(crate) struct ParsedBlock {
    pub(crate) name: String,
    pub(crate) parameters: Vec<ParsedParameter>,
    pub(crate) instructions: Vec<ParsedInstruction>,
    pub(crate) terminator: ParsedTerminator,
}

#[derive(Debug)]
pub(crate) struct ParsedParameter {
    pub(crate) name: String,
    pub(crate) typ: Type,
}

#[derive(Debug)]
pub(crate) enum ParsedInstruction {}

#[derive(Debug)]
pub(crate) enum ParsedTerminator {
    Return(Vec<ParsedValue>),
}

#[derive(Debug)]
pub(crate) enum ParsedValue {
    NumericConstant { constant: FieldElement, typ: Type },
    Array { values: Vec<ParsedValue>, typ: Type },
    Variable(String),
}
