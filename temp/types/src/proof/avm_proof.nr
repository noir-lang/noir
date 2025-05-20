use crate::{constants::AVM_PROOF_LENGTH_IN_FIELDS, traits::{Deserialize, Empty, Serialize}};

pub struct AvmProof {
    pub fields: [Field; AVM_PROOF_LENGTH_IN_FIELDS],
}

impl Serialize<AVM_PROOF_LENGTH_IN_FIELDS> for AvmProof {
    fn serialize(self) -> [Field; AVM_PROOF_LENGTH_IN_FIELDS] {
        self.fields
    }
}

impl Deserialize<AVM_PROOF_LENGTH_IN_FIELDS> for AvmProof {
    fn deserialize(fields: [Field; AVM_PROOF_LENGTH_IN_FIELDS]) -> Self {
        AvmProof { fields }
    }
}

impl Empty for AvmProof {
    fn empty() -> Self {
        AvmProof { fields: [0; AVM_PROOF_LENGTH_IN_FIELDS] }
    }
}

impl Eq for AvmProof {
    fn eq(self, other: Self) -> bool {
        (self.fields == other.fields)
    }
}

#[test]
fn serialization_of_empty_recursive() {
    let item = AvmProof::empty();
    let serialized = item.serialize();
    let deserialized = AvmProof::deserialize(serialized);
    assert(item.eq(deserialized));
}
