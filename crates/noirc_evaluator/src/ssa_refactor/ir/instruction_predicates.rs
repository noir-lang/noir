use super::{
    super::basic_block::BasicBlockId, function::Function, instruction::TerminatorInstruction,
};

/// Visit all successors of a block with a given visitor closure. The closure
/// arguments are the branch instruction that is used to reach the successor,
/// and the id of the successor block itself.
pub(crate) fn visit_block_succs<F: FnMut(BasicBlockId)>(
    func: &Function,
    basic_block_id: BasicBlockId,
    mut visit: F,
) {
    let terminator = &func[basic_block_id].terminator();
    match terminator {
        TerminatorInstruction::Jmp { destination, .. } => visit(*destination),
        TerminatorInstruction::JmpIf { then_destination, else_destination, .. } => {
            visit(*then_destination);
            visit(*else_destination);
        }
    }
}
