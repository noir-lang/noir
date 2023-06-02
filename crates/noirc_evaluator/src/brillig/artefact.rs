use acvm::acir::brillig_vm::Opcode as BrilligOpcode;

#[derive(Default, Debug, Clone)]
/// Artefacts resulting from the compilation of a function into brillig byte code
/// Currently it is just the brillig bytecode of the function
pub(crate) struct BrilligArtefact {
    pub(crate) byte_code: Vec<BrilligOpcode>,
}

impl BrilligArtefact {
    // Link some compiled brillig bytecode with its referenced artefacts
    pub(crate) fn link(&mut self, obj: &BrilligArtefact) -> Vec<BrilligOpcode> {
        self.link_with(obj);
        self.byte_code.clone()
    }

    // Link with a brillig artefact
    fn link_with(&mut self, obj: &BrilligArtefact) {
        if obj.byte_code.is_empty() {
            panic!("ICE: unresolved symbol");
        }

        self.byte_code.extend_from_slice(&obj.byte_code);
    }
}
