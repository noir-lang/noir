use super::{
    basic_block::{BasicBlock, BasicBlockId},
    instruction::TerminatorInstruction,
};

/// Visit all successors of a block with a given visitor closure. The closure
/// arguments are the branch instruction that is used to reach the successor,
/// and the id of the successor block itself.
pub(crate) fn visit_block_succs<F: FnMut(BasicBlockId)>(basic_block: &BasicBlock, mut visit: F) {
    match basic_block
        .terminator()
        .expect("ICE: No terminator indicates block is still under construction.")
    {
        TerminatorInstruction::Jmp { destination, .. } => visit(*destination),
        TerminatorInstruction::JmpIf { then_destination, else_destination, .. } => {
            visit(*then_destination);
            visit(*else_destination);
        }
        TerminatorInstruction::Return { .. } => {
            // The last block of the control flow - no successors
        }
    }
}
