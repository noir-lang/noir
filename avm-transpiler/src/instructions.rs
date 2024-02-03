use std::fmt;
use std::fmt::{Debug, Formatter};

use crate::opcodes::AvmOpcode;

/// Common values of the indirect instruction flag
pub const ZEROTH_OPERAND_INDIRECT: u8 = 0b00000001;
pub const FIRST_OPERAND_INDIRECT: u8 = 0b00000010;
pub const ZEROTH_FIRST_OPERANDS_INDIRECT: u8 = 0b00000011;

/// A simple representation of an AVM instruction for the purpose
/// of generating an AVM bytecode from Brillig.
/// Note: this does structure not impose rules like "ADD instruction must have 3 operands"
/// That job is left to the instruction decoder, not this thin transpiler.
pub struct AvmInstruction {
    pub opcode: AvmOpcode,

    /// Any instructions with memory offset operands have the indirect flag
    /// Each bit is a boolean: 0:direct, 1:indirect
    /// The 0th bit corresponds to an instruction's 0th offset arg, 1st to 1st, etc...
    pub indirect: Option<u8>,

    /// Some instructions have a destination or input tag
    // TODO(4271): add in_tag alongside its support in TS
    //pub in_tag: Option<AvmTypeTag>,
    pub dst_tag: Option<AvmTypeTag>,

    /// Different instructions have different numbers of operands
    pub operands: Vec<AvmOperand>,
}
impl AvmInstruction {
    /// String representation for printing AVM programs
    pub fn to_string(&self) -> String {
        let mut out_str = format!("opcode {}", self.opcode.name());
        if let Some(indirect) = self.indirect {
            out_str += format!(", indirect: {}", indirect).as_str();
        }
        // TODO(4271): add in_tag alongside its support in TS
        if let Some(dst_tag) = self.dst_tag {
            out_str += format!(", dst_tag: {}", dst_tag as u8).as_str();
        }
        if !self.operands.is_empty() {
            out_str += ", operands: [";
            for operand in &self.operands {
                out_str += format!("{}, ", operand.to_string()).as_str();
            }
            out_str += "]";
        }
        out_str
    }
    /// Bytes representation for generating AVM bytecode
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.opcode as u8);
        if let Some(indirect) = self.indirect {
            bytes.push(indirect);
        }
        // TODO(4271): add in_tag alongside its support in TS
        if let Some(dst_tag) = self.dst_tag {
            // TODO(4271): make 8 bits when TS supports deserialization of 8 bit flags
            bytes.extend_from_slice(&(dst_tag as u8).to_be_bytes());
        }
        for operand in &self.operands {
            bytes.extend_from_slice(&operand.to_be_bytes());
        }
        bytes
    }
}

impl Debug for AvmInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for AvmInstruction {
    fn default() -> Self {
        AvmInstruction {
            opcode: AvmOpcode::ADD,
            // TODO(4266): default to Some(0), since all instructions have indirect flag except jumps
            indirect: None,
            dst_tag: None,
            operands: vec![],
        }
    }
}

/// AVM instructions may include a type tag
#[derive(Copy, Clone)]
pub enum AvmTypeTag {
    UNINITIALIZED,
    UINT8,
    UINT16,
    UINT32,
    UINT64,
    UINT128,
    FIELD,
    INVALID,
}

/// Operands are usually 32 bits (offsets or jump destinations)
/// Constants (as used by the SET instruction) can have size
/// different from 32 bits
pub enum AvmOperand {
    U32 { value: u32 },
    // TODO(4267): Support operands of size other than 32 bits (for SET)
    U128 { value: u128 },
}
impl AvmOperand {
    pub fn to_string(&self) -> String {
        match self {
            AvmOperand::U32 { value } => format!(" U32:{}", value),
            // TODO(4267): Support operands of size other than 32 bits (for SET)
            AvmOperand::U128 { value } => format!(" U128:{}", value),
        }
    }
    pub fn to_be_bytes(&self) -> Vec<u8> {
        match self {
            AvmOperand::U32 { value } => value.to_be_bytes().to_vec(),
            // TODO(4267): Support operands of size other than 32 bits (for SET)
            AvmOperand::U128 { value } => value.to_be_bytes().to_vec(),
        }
    }
}
