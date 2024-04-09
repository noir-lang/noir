use std::fmt::{self, Display};
use std::fmt::{Debug, Formatter};

use crate::opcodes::AvmOpcode;

/// Common values of the indirect instruction flag
pub const ALL_DIRECT: u8 = 0b00000000;
pub const ZEROTH_OPERAND_INDIRECT: u8 = 0b00000001;
pub const FIRST_OPERAND_INDIRECT: u8 = 0b00000010;
pub const SECOND_OPERAND_INDIRECT: u8 = 0b00000100;

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

    /// Some instructions have a destination xor input tag
    /// Its usage will depend on the instruction.
    pub tag: Option<AvmTypeTag>,

    /// Different instructions have different numbers of operands
    pub operands: Vec<AvmOperand>,
}

impl Display for AvmInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "opcode {}", self.opcode.name())?;
        if let Some(indirect) = self.indirect {
            write!(f, ", indirect: {}", indirect)?;
        }
        // This will be either inTag or dstTag depending on the operation
        if let Some(dst_tag) = self.tag {
            write!(f, ", tag: {}", dst_tag as u8)?;
        }
        if !self.operands.is_empty() {
            write!(f, ", operands: [")?;
            for operand in &self.operands {
                write!(f, "{operand}, ")?;
            }
            write!(f, "]")?;
        };
        Ok(())
    }
}

impl AvmInstruction {
    /// Bytes representation for generating AVM bytecode
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.opcode as u8);
        if let Some(indirect) = self.indirect {
            bytes.push(indirect);
        }
        // This will be either inTag or dstTag depending on the operation
        if let Some(tag) = self.tag {
            bytes.extend_from_slice(&(tag as u8).to_be_bytes());
        }
        for operand in &self.operands {
            bytes.extend_from_slice(&operand.to_be_bytes());
        }
        bytes
    }
}

impl Debug for AvmInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Default for AvmInstruction {
    fn default() -> Self {
        AvmInstruction {
            opcode: AvmOpcode::ADD,
            // TODO(4266): default to Some(0), since all instructions have indirect flag except jumps
            indirect: None,
            tag: None,
            operands: vec![],
        }
    }
}

/// AVM instructions may include a type tag
#[derive(Copy, Clone, Debug)]
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
    U8 { value: u8 },
    U16 { value: u16 },
    U32 { value: u32 },
    U64 { value: u64 },
    U128 { value: u128 },
}

impl Display for AvmOperand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AvmOperand::U8 { value } => write!(f, " U8:{}", value),
            AvmOperand::U16 { value } => write!(f, " U16:{}", value),
            AvmOperand::U32 { value } => write!(f, " U32:{}", value),
            AvmOperand::U64 { value } => write!(f, " U64:{}", value),
            AvmOperand::U128 { value } => write!(f, " U128:{}", value),
        }
    }
}

impl AvmOperand {
    pub fn to_be_bytes(&self) -> Vec<u8> {
        match self {
            AvmOperand::U8 { value } => value.to_be_bytes().to_vec(),
            AvmOperand::U16 { value } => value.to_be_bytes().to_vec(),
            AvmOperand::U32 { value } => value.to_be_bytes().to_vec(),
            AvmOperand::U64 { value } => value.to_be_bytes().to_vec(),
            AvmOperand::U128 { value } => value.to_be_bytes().to_vec(),
        }
    }
}
