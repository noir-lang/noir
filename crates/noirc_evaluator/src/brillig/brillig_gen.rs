use super::{acvm_brillig::BrilligOpcode, artefact::Artefact};

#[derive(Default)]
pub(crate) struct BrilligGen {
    obj: Artefact,
}

impl BrilligGen {
    /// Adds a brillig instruction to the brillig code base
    fn push_code(&mut self, code: BrilligOpcode) {
        self.obj.byte_code.push(code);
    }

    pub(crate) fn compile() -> Artefact {
        let mut brillig = BrilligGen::default();
        brillig.push_code(BrilligOpcode::Stop);
        brillig.obj
    }
}
