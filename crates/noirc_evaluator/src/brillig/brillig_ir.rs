//! This module is an abstraction layer over `Brillig`
//! To allow for separation of concerns, it knows nothing
//! about SSA types, and can therefore be tested independently.
//! `brillig_gen` is therefore the module which combines both
//! ssa types and types in this module.
//! A similar paradigm can be seen with the `acir_ir` module.
pub(crate) mod artifact;
pub(crate) mod memory;

use acvm::acir::brillig_vm::{Opcode as BrilligOpcode, RegisterIndex};
#[derive(Default)]
pub(crate) struct BrilligContext {
    obj: BrilligArtifact,
    /// A usize indicating the latest un-used register.
    latest_register: usize,
    /// Tracks memory allocations
    memory: memory::BrilligMemory,
}

impl BrilligContext {
    /// Adds a brillig instruction to the brillig byte code
    pub(crate) fn push_opcode(&mut self, opcode: BrilligOpcode) {
        self.obj.byte_code.push(opcode)
    }

    /// Returns the artifact
    pub(crate) fn artifact(self) -> BrilligArtifact {
        self.obj
    }

    pub(crate) fn allocate_memory(&mut self, size: usize) -> usize {
        self.memory.allocate(size)
    }

    #[deprecated(note = " abstraction leak")]
    // TODO: Remove as this needs knowledge of BasicBlockId which is an abstraction leak
    pub(crate) fn add_block_label(&mut self, block: BasicBlockId) {
        self.obj.add_block_label(block)
    }

    #[deprecated(note = " abstraction leak")]
    // TODO: Remove as this needs knowledge of BasicBlockId which is an abstraction leak
    pub(crate) fn add_unresolved_jump(&mut self, destination: UnresolvedJumpLocation) {
        self.obj.add_unresolved_jump(destination)
    }

    #[deprecated(note = " abstraction leak")]
    pub(crate) fn latest_register(&mut self) -> &mut usize {
        &mut self.latest_register
    }

    /// Creates a new register.
    pub(crate) fn create_register(&mut self) -> RegisterIndex {
        let register = RegisterIndex::from(self.latest_register);
        self.latest_register += 1;
        register
    }
}

use acvm::acir::brillig_vm::{BinaryFieldOp, BinaryIntOp};

use crate::ssa_refactor::ir::basic_block::BasicBlockId;

use self::artifact::{BrilligArtifact, UnresolvedJumpLocation};

/// Type to encapsulate the binary operation types in Brillig
pub(crate) enum BrilligBinaryOp {
    Field { op: BinaryFieldOp },
    Integer { op: BinaryIntOp, bit_size: u32 },
}
