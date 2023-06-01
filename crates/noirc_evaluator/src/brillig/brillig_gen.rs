use crate::ssa_refactor::ir::function::Function;

use super::{acvm_brillig::BrilligOpcode, artefact::BrilligArtefact};

#[derive(Default)]
pub(crate) struct BrilligGen {
    obj: BrilligArtefact,
}

impl BrilligGen {
    /// Adds a brillig instruction to the brillig code base
    fn push_code(&mut self, code: BrilligOpcode) {
        self.obj.byte_code.push(code);
    }

    pub(crate) fn compile(func: &Function) -> BrilligArtefact {
        let mut brillig = BrilligGen::default();
        // we only support empty functions for now
        assert_eq!(func.dfg.num_instructions(), 0);
        brillig.push_code(BrilligOpcode::Stop);

        brillig.obj
    }
}
