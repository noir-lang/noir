use crate::ssa_refactor::ir::function::Function;

use super::artifact::BrilligArtifact;

use acvm::acir::brillig_vm::Opcode as BrilligOpcode;
#[derive(Default)]
/// Generate the compilation artifacts for compiling a function into brillig bytecode.
pub(crate) struct BrilligGen {
    obj: BrilligArtifact,
}

impl BrilligGen {
    /// Adds a brillig instruction to the brillig code base
    fn push_code(&mut self, code: BrilligOpcode) {
        self.obj.byte_code.push(code);
    }

    pub(crate) fn compile(func: &Function) -> BrilligArtifact {
        let mut brillig = BrilligGen::default();
        // we only support empty functions for now
        assert_eq!(func.dfg.num_instructions(), 0);
        brillig.push_code(BrilligOpcode::Stop);

        brillig.obj
    }
}
