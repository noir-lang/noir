use std::fmt::{self, Display, Formatter};

use acvm::FieldElement;
use noirc_errors::Span;

use crate::ssa::ir::{function::RuntimeType, instruction::BinaryOp, types::Type};

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

#[derive(Debug, Clone)]
pub(crate) struct Identifier {
    pub(crate) name: String,
    pub(crate) span: Span,
}

impl Identifier {
    pub(crate) fn new(name: String, span: Span) -> Self {
        Self { name, span }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
pub(crate) enum ParsedInstruction {
    Allocate {
        target: Identifier,
        typ: Type,
    },
    ArrayGet {
        target: Identifier,
        element_type: Type,
        array: ParsedValue,
        index: ParsedValue,
    },
    ArraySet {
        target: Identifier,
        array: ParsedValue,
        index: ParsedValue,
        value: ParsedValue,
        mutable: bool,
    },
    BinaryOp {
        target: Identifier,
        lhs: ParsedValue,
        op: BinaryOp,
        rhs: ParsedValue,
    },
    Call {
        targets: Vec<Identifier>,
        function: Identifier,
        arguments: Vec<ParsedValue>,
        types: Vec<Type>,
    },
    Cast {
        target: Identifier,
        lhs: ParsedValue,
        typ: Type,
    },
    Constrain {
        lhs: ParsedValue,
        rhs: ParsedValue,
        assert_message: Option<AssertMessage>,
    },
    DecrementRc {
        value: ParsedValue,
    },
    EnableSideEffectsIf {
        condition: ParsedValue,
    },
    IncrementRc {
        value: ParsedValue,
    },
    Load {
        target: Identifier,
        value: ParsedValue,
        typ: Type,
    },
    MakeArray {
        target: Identifier,
        elements: Vec<ParsedValue>,
        typ: Type,
    },
    Not {
        target: Identifier,
        value: ParsedValue,
    },
    RangeCheck {
        value: ParsedValue,
        max_bit_size: u32,
    },
    Store {
        value: ParsedValue,
        address: ParsedValue,
    },
    Truncate {
        target: Identifier,
        value: ParsedValue,
        bit_size: u32,
        max_bit_size: u32,
    },
}

#[derive(Debug)]
pub(crate) enum AssertMessage {
    Static(String),
    Dynamic(Vec<ParsedValue>),
}

#[derive(Debug)]
pub(crate) enum ParsedTerminator {
    Jmp { destination: Identifier, arguments: Vec<ParsedValue> },
    Jmpif { condition: ParsedValue, then_block: Identifier, else_block: Identifier },
    Return(Vec<ParsedValue>),
}

#[derive(Debug, Clone)]
pub(crate) enum ParsedValue {
    NumericConstant { constant: FieldElement, typ: Type },
    Variable(Identifier),
}
