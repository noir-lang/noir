use super::{acvm_brillig::BrilligOpcode, artefact::Brillig};

#[derive(Default)]
pub(crate) struct BrilligGen {
    obj: Brillig,
}

impl BrilligGen {
    /// Adds a brillig instruction to the brillig code base
    fn push_code(&mut self, code: BrilligOpcode) {
        self.obj.byte_code.push(code);
    }

    pub(crate) fn compile() -> Brillig {
        let mut brillig = BrilligGen::default();
        brillig.push_code(BrilligOpcode::Stop);
        brillig.obj
    }
}
