use acvm::FieldElement;
use noirc_errors::Span;

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
    pub(crate) identifier: Identifier,
    pub(crate) typ: Type,
}

#[derive(Debug)]
pub(crate) struct Identifier {
    pub(crate) name: String,
    pub(crate) span: Span,
}

impl Identifier {
    pub(crate) fn new(name: String, span: Span) -> Self {
        Self { name, span }
    }
}

#[derive(Debug)]
pub(crate) enum ParsedInstruction {}

#[derive(Debug)]
pub(crate) enum ParsedTerminator {
    Jmp { destination: Identifier, arguments: Vec<ParsedValue> },
    Jmpif { condition: ParsedValue, then_block: Identifier, else_block: Identifier },
    Return(Vec<ParsedValue>),
}

#[derive(Debug)]
pub(crate) enum ParsedValue {
    NumericConstant { constant: FieldElement, typ: Type },
    Array { values: Vec<ParsedValue>, typ: Type },
    Variable(Identifier),
}
