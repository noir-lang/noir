use crate::{constants::TUBE_PROOF_LENGTH, traits::{Deserialize, Empty, Serialize}};

pub struct TubeProof {
    pub fields: [Field; TUBE_PROOF_LENGTH],
}

impl Serialize<TUBE_PROOF_LENGTH> for TubeProof {
    fn serialize(self) -> [Field; TUBE_PROOF_LENGTH] {
        self.fields
    }
}

impl Deserialize<TUBE_PROOF_LENGTH> for TubeProof {
    fn deserialize(fields: [Field; TUBE_PROOF_LENGTH]) -> Self {
        TubeProof { fields }
    }
}

impl Empty for TubeProof {
    fn empty() -> Self {
        TubeProof { fields: [0; TUBE_PROOF_LENGTH] }
    }
}

impl Eq for TubeProof {
    fn eq(self, other: Self) -> bool {
        (self.fields == other.fields)
    }
}

#[test]
fn serialization_of_empty_recursive() {
    let item = TubeProof::empty();
    let serialized = item.serialize();
    let deserialized = TubeProof::deserialize(serialized);
    assert(item.eq(deserialized));
}
