use acvm::acir::brillig_vm::Opcode as BrilligOpcode;

#[derive(Default, Debug, Clone)]
/// Artifacts resulting from the compilation of a function into brillig byte code
/// Currently it is just the brillig bytecode of the function
pub(crate) struct BrilligArtifact {
    pub(crate) byte_code: Vec<BrilligOpcode>,
}

impl BrilligArtifact {
    // Link some compiled brillig bytecode with its referenced artifacts
    pub(crate) fn link(&mut self, obj: &BrilligArtifact) -> Vec<BrilligOpcode> {
        self.link_with(obj);
        self.byte_code.clone()
    }

    // Link with a brillig artifact
    fn link_with(&mut self, obj: &BrilligArtifact) {
        if obj.byte_code.is_empty() {
            panic!("ICE: unresolved symbol");
        }

        self.byte_code.extend_from_slice(&obj.byte_code);
    }
}
