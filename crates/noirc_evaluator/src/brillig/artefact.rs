use super::acvm_brillig::BrilligOpcode;

const PREFIX_LEN: usize = 3;

#[derive(Default, Debug, Clone)]
pub(crate) struct Artefact {
    pub(crate) byte_code: Vec<BrilligOpcode>,
}

impl Artefact {
    pub(crate) fn link(&mut self, obj: &Artefact) -> Vec<BrilligOpcode> {
        self.link_with(obj);
        self.byte_code.clone()
    }

    fn link_with(&mut self, obj: &Artefact) {
        if obj.byte_code.is_empty() {
            panic!("ICE: unresolved symbol");
        }
        if self.byte_code.is_empty() {
            //prefix
            self.byte_code.push(BrilligOpcode::Jump { location: PREFIX_LEN });
            self.byte_code.push(BrilligOpcode::Trap);
            self.byte_code.push(BrilligOpcode::Stop);
            //assert prefic length is as expected
            assert_eq!(self.byte_code.len(), PREFIX_LEN);
        }
        self.byte_code.extend_from_slice(&obj.byte_code);
    }
}
