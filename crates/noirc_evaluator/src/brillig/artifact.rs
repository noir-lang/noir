use crate::ssa_refactor::ir::basic_block::BasicBlockId;
use acvm::acir::brillig_vm::Opcode as BrilligOpcode;
use std::collections::HashMap;

/// Pointer to a unresolved Jump instruction in
/// the bytecode.
pub(crate) type JumpLabel = usize;

/// Pointer to a position in the bytecode where a
/// particular basic block starts.
pub(crate) type BlockLabel = usize;

#[derive(Default, Debug, Clone)]
/// Artifacts resulting from the compilation of a function into brillig byte code.
/// Currently it is just the brillig bytecode of the function.
pub(crate) struct BrilligArtifact {
    pub(crate) byte_code: Vec<BrilligOpcode>,
    /// The set of jumps that need to have their locations
    /// resolved.
    unresolved_jumps: Vec<(JumpLabel, UnresolvedJumpLocation)>,
    /// A map of the basic blocks to their positions
    /// in the bytecode.
    blocks: HashMap<BasicBlockId, BlockLabel>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum UnresolvedJumpLocation {
    Block(BasicBlockId),
    Relative(i32),
}

impl BrilligArtifact {
    /// Link some compiled brillig bytecode with its referenced artifacts.
    pub(crate) fn link(&mut self, obj: &BrilligArtifact) -> Vec<BrilligOpcode> {
        self.link_with(obj);
        self.resolve_jumps();
        self.byte_code.clone()
    }

    /// Link with a brillig artifact
    fn link_with(&mut self, obj: &BrilligArtifact) {
        let offset = self.code_len();
        for (jump_label, block_id) in &obj.unresolved_jumps {
            self.unresolved_jumps.push((jump_label + offset, *block_id));
        }

        for (block_id, block_label) in &obj.blocks {
            self.blocks.insert(*block_id, block_label + offset);
        }

        self.byte_code.extend_from_slice(&obj.byte_code);
    }

    /// Adds a unresolved jump to be fixed at the end of bytecode processing.
    pub(crate) fn add_unresolved_jump(&mut self, destination: UnresolvedJumpLocation) {
        self.unresolved_jumps.push((self.code_len(), destination));
    }

    /// Adds a label in the bytecode to specify where this block's
    /// opcodes will start.
    pub(crate) fn add_block_label(&mut self, block: BasicBlockId) {
        self.blocks.insert(block, self.code_len());
    }

    /// Number of the opcodes currently in the bytecode
    pub(crate) fn code_len(&self) -> usize {
        self.byte_code.len()
    }

    /// Resolves all of the unresolved jumps in the program.
    ///
    /// Note: This should only be called once all blocks are processed.
    fn resolve_jumps(&mut self) {
        for (jump_label, unresolved_location) in &self.unresolved_jumps {
            let jump_instruction = self.byte_code[*jump_label].clone();

            let actual_block_location = match unresolved_location {
                UnresolvedJumpLocation::Block(b) => self.blocks[b],
                UnresolvedJumpLocation::Relative(location) => {
                    (location + *jump_label as i32) as usize
                }
            };

            match jump_instruction {
                BrilligOpcode::Jump { location } => {
                    assert_eq!(location, 0, "location is not zero, which means that the jump label does not need resolving");

                    self.byte_code[*jump_label] =
                        BrilligOpcode::Jump { location: actual_block_location };
                }
                BrilligOpcode::JumpIfNot { condition, location } => {
                    assert_eq!(location, 0, "location is not zero, which means that the jump label does not need resolving");

                    self.byte_code[*jump_label] =
                        BrilligOpcode::JumpIfNot { condition, location: actual_block_location };
                }
                BrilligOpcode::JumpIf { condition, location } => {
                    assert_eq!(location, 0, "location is not zero, which means that the jump label does not need resolving");

                    self.byte_code[*jump_label] =
                        BrilligOpcode::JumpIf { condition, location: actual_block_location };
                }
                _ => unreachable!(
                    "all jump labels should point to a jump instruction in the bytecode"
                ),
            }
        }
    }
}
