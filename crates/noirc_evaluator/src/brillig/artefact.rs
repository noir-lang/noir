use super::acvm_brillig::BrilligOpcode;

#[derive(Default, Debug, Clone)]
pub(crate) struct BrilligArtefact {
    pub(crate) byte_code: Vec<BrilligOpcode>,
}

impl BrilligArtefact {
    pub(crate) fn link(&mut self, obj: &BrilligArtefact) -> Vec<BrilligOpcode> {
        self.link_with(obj);
        self.byte_code.clone()
    }

    fn link_with(&mut self, obj: &BrilligArtefact) {
        if obj.byte_code.is_empty() {
            panic!("ICE: unresolved symbol");
        }

        self.byte_code.extend_from_slice(&obj.byte_code);
    }
}
