use crate::{
    constants::CONTENT_COMMITMENT_LENGTH,
    traits::{Deserialize, Empty, Hash, Serialize}, utils::{arr_copy_slice}
};

struct ContentCommitment {
  tx_tree_height: Field,
  txs_effects_hash: Field,
  in_hash: Field,
  out_hash: Field,
}

impl Serialize<CONTENT_COMMITMENT_LENGTH> for ContentCommitment {
  fn serialize(self) -> [Field; CONTENT_COMMITMENT_LENGTH] {
    let mut fields: BoundedVec<Field, CONTENT_COMMITMENT_LENGTH> = BoundedVec::new();

    fields.push(self.tx_tree_height);
    fields.push(self.txs_effects_hash);
    fields.push(self.in_hash);
    fields.push(self.out_hash);

    fields.storage
  }
}

impl Deserialize<CONTENT_COMMITMENT_LENGTH> for ContentCommitment {
  fn deserialize(serialized: [Field; CONTENT_COMMITMENT_LENGTH]) -> Self {
    let tx_tree_height = serialized[0];

    let txs_effects_hash = serialized[1];

    let in_hash = serialized[2];

    let out_hash = serialized[3];

    Self {
      tx_tree_height,
      txs_effects_hash,
      in_hash,
      out_hash,
    }
  }
}

impl Empty for ContentCommitment {
  fn empty() -> Self {
    Self {
      tx_tree_height: 0,
      txs_effects_hash: 0,
      in_hash: 0,
      out_hash: 0,
    }
  }
}

impl Eq for ContentCommitment {
  fn eq(self, other: Self) -> bool {
    (self.tx_tree_height == other.tx_tree_height)
      & (self.txs_effects_hash == other.txs_effects_hash)
      & (self.in_hash == other.in_hash)
      & (self.out_hash == other.out_hash)
  }
}

#[test]
fn serialization_of_empty() {
    let empty = ContentCommitment::empty();
    let serialized = empty.serialize();
    let deserialized = ContentCommitment::deserialize(serialized);

    assert(empty.eq(deserialized));
}
