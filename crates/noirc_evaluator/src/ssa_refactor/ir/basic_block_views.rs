use super::{
    basic_block::{BasicBlock, BasicBlockId},
    instruction::TerminatorInstruction,
};

/// Get an iterator over all successor ids of the provided basic block.
pub(crate) fn successors_iter(basic_block: &BasicBlock) -> impl Iterator<Item = BasicBlockId> {
    let ids = match basic_block.terminator() {
        TerminatorInstruction::Jmp { destination, .. } => {
            vec![*destination]
        }
        TerminatorInstruction::JmpIf { then_destination, else_destination, .. } => {
            vec![*then_destination, *else_destination]
        }
        TerminatorInstruction::Return { .. } => {
            // The last block of the control flow - no successors
            vec![]
        }
    };
    ids.into_iter()
}
