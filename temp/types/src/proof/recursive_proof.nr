use crate::{
    constants::{NESTED_RECURSIVE_PROOF_LENGTH, RECURSIVE_PROOF_LENGTH},
    traits::{Deserialize, Empty, Serialize},
};

pub struct RecursiveProof {
    pub fields: [Field; RECURSIVE_PROOF_LENGTH],
}

impl Serialize<RECURSIVE_PROOF_LENGTH> for RecursiveProof {
    fn serialize(self) -> [Field; RECURSIVE_PROOF_LENGTH] {
        self.fields
    }
}

impl Deserialize<RECURSIVE_PROOF_LENGTH> for RecursiveProof {
    fn deserialize(fields: [Field; RECURSIVE_PROOF_LENGTH]) -> Self {
        RecursiveProof { fields }
    }
}

impl Empty for RecursiveProof {
    fn empty() -> Self {
        RecursiveProof { fields: [0; RECURSIVE_PROOF_LENGTH] }
    }
}

pub struct NestedRecursiveProof {
    pub fields: [Field; NESTED_RECURSIVE_PROOF_LENGTH],
}

impl Serialize<NESTED_RECURSIVE_PROOF_LENGTH> for NestedRecursiveProof {
    fn serialize(self) -> [Field; NESTED_RECURSIVE_PROOF_LENGTH] {
        self.fields
    }
}

impl Deserialize<NESTED_RECURSIVE_PROOF_LENGTH> for NestedRecursiveProof {
    fn deserialize(fields: [Field; NESTED_RECURSIVE_PROOF_LENGTH]) -> Self {
        NestedRecursiveProof { fields }
    }
}

impl Empty for NestedRecursiveProof {
    fn empty() -> Self {
        NestedRecursiveProof { fields: [0; NESTED_RECURSIVE_PROOF_LENGTH] }
    }
}

impl Eq for RecursiveProof {
    fn eq(self, other: Self) -> bool {
        (self.fields == other.fields)
    }
}

impl Eq for NestedRecursiveProof {
    fn eq(self, other: Self) -> bool {
        (self.fields == other.fields)
    }
}

#[test]
fn serialization_of_empty_recursive() {
    let item = RecursiveProof::empty();
    let serialized = item.serialize();
    let deserialized = RecursiveProof::deserialize(serialized);
    assert(item.eq(deserialized));
}

#[test]
fn serialization_of_empty_nested() {
    let item = NestedRecursiveProof::empty();
    let serialized = item.serialize();
    let deserialized = NestedRecursiveProof::deserialize(serialized);
    assert(item.eq(deserialized));
}
