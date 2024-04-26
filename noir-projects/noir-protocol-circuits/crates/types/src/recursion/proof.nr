use crate::{
    traits::{Serialize, Deserialize, Empty},
    constants::{RECURSIVE_PROOF_LENGTH, NESTED_RECURSIVE_PROOF_LENGTH}
};

struct RecursiveProof {
    fields: [Field; RECURSIVE_PROOF_LENGTH],
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
    RecursiveProof {
      fields: [0; RECURSIVE_PROOF_LENGTH],
    }
  }
}

struct NestedRecursiveProof {
    fields: [Field; NESTED_RECURSIVE_PROOF_LENGTH],
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
    NestedRecursiveProof {
      fields: [0; NESTED_RECURSIVE_PROOF_LENGTH],
    }
  }
}
