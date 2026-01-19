use std::fmt::{self, Display, Formatter};

use acvm::FieldElement;
use noirc_errors::Span;

use crate::ssa::{
    ir::{
        function::RuntimeType,
        instruction::{ArrayOffset, BinaryOp},
        types::Type,
    },
    opt::pure::Purity,
};

#[derive(Debug)]
pub(crate) struct ParsedSsa {
    pub(crate) globals: Vec<ParsedGlobal>,
    pub(crate) functions: Vec<ParsedFunction>,
}

#[derive(Debug)]
pub(crate) struct ParsedGlobal {
    pub(crate) name: Identifier,
    pub(crate) value: ParsedGlobalValue,
}

#[derive(Debug)]
pub(crate) enum ParsedGlobalValue {
    NumericConstant(ParsedNumericConstant),
    MakeArray(ParsedMakeArray),
}

#[derive(Debug)]
pub(crate) struct ParsedMakeArray {
    pub(crate) elements: Vec<ParsedValue>,
    pub(crate) typ: Type,
}

#[derive(Debug)]
pub(crate) struct ParsedFunction {
    pub(crate) runtime_type: RuntimeType,
    pub(crate) purity: Option<Purity>,
    pub(crate) external_name: String,
    pub(crate) internal_name: String,
    pub(crate) data_bus: ParsedDataBus,
    pub(crate) blocks: Vec<ParsedBlock>,
}

#[derive(Debug)]
pub(crate) struct ParsedDataBus {
    pub(crate) call_data: Vec<ParsedCallData>,
    pub(crate) return_data: Option<ParsedValue>,
}

#[derive(Debug)]
pub(crate) struct ParsedCallData {
    pub(crate) call_data_id: u32,
    pub(crate) array: ParsedValue,
    pub(crate) index_map: Vec<(ParsedValue, usize)>,
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
        offset: ArrayOffset,
    },
    ArraySet {
        target: Identifier,
        array: ParsedValue,
        index: ParsedValue,
        value: ParsedValue,
        mutable: bool,
        offset: ArrayOffset,
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
        equals: bool,
        rhs: ParsedValue,
        assert_message: Option<AssertMessage>,
    },
    DecrementRc {
        value: ParsedValue,
    },
    EnableSideEffectsIf {
        condition: ParsedValue,
    },
    IfElse {
        target: Identifier,
        then_condition: ParsedValue,
        then_value: ParsedValue,
        else_condition: ParsedValue,
        else_value: ParsedValue,
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
    Nop,
    Not {
        target: Identifier,
        value: ParsedValue,
    },
    RangeCheck {
        value: ParsedValue,
        max_bit_size: u32,
        assert_message: Option<String>,
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
    Unreachable,
}

#[derive(Debug, Clone)]
pub(crate) enum ParsedValue {
    NumericConstant(ParsedNumericConstant),
    Variable(Identifier),
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedNumericConstant {
    pub(crate) value: FieldElement,
    pub(crate) typ: Type,
}
