use super::{instruction::InstructionId, types::Typ};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
/// Value is the most basic type allowed in the IR.
/// Transition Note: This is similar to `NodeId` in our previous IR.
pub(crate) struct ValueId(pub(crate) u32);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum Value {
    /// This value was created due to an instruction
    ///
    /// instruction -- This is the instruction which defined it
    /// typ -- This is the `Type` of the instruction
    /// position -- Returns the position in the results
    /// vector that this `Value` is located.
    /// Example, if you add two numbers together, then the resulting
    /// value would have position `0`, the typ would be the type
    /// of the operands, and the instruction would map to an add instruction.
    Instruction { typ: Typ, position: u16, instruction: InstructionId },
}
