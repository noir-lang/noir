use crate::{
    constants::{NUM_FIELDS_PER_SHA256, CONTENT_COMMITMENT_LENGTH},
    traits::{Deserialize, Empty, Hash, Serialize}, utils::{arr_copy_slice}
};

struct ContentCommitment {
  tx_tree_height: Field,
  txs_hash: [Field; NUM_FIELDS_PER_SHA256],
  in_hash: [Field; NUM_FIELDS_PER_SHA256],
  out_hash: [Field; NUM_FIELDS_PER_SHA256],
}

impl Serialize<CONTENT_COMMITMENT_LENGTH> for ContentCommitment {
  fn serialize(self) -> [Field; CONTENT_COMMITMENT_LENGTH] {
    let mut fields: BoundedVec<Field, CONTENT_COMMITMENT_LENGTH> = BoundedVec::new();

    fields.extend_from_array([self.tx_tree_height]);
    fields.extend_from_array(self.txs_hash);
    fields.extend_from_array(self.in_hash);
    fields.extend_from_array(self.out_hash);

    fields.storage
  }
}

impl Deserialize<CONTENT_COMMITMENT_LENGTH> for ContentCommitment {
  fn deserialize(serialized: [Field; CONTENT_COMMITMENT_LENGTH]) -> Self {
    let tx_tree_height = serialized[0];
    let mut offset = 1;

    let txs_hash = arr_copy_slice(serialized, [0; NUM_FIELDS_PER_SHA256], offset);
    offset = offset + NUM_FIELDS_PER_SHA256;

    let in_hash = arr_copy_slice(serialized, [0; NUM_FIELDS_PER_SHA256], offset);
    offset = offset + NUM_FIELDS_PER_SHA256;

    let out_hash = arr_copy_slice(serialized, [0; NUM_FIELDS_PER_SHA256], offset);

    Self {
      tx_tree_height,
      txs_hash,
      in_hash,
      out_hash,
    }
  }
}

impl Empty for ContentCommitment {
  fn empty() -> Self {
    Self {
      tx_tree_height: 0,
      txs_hash: [0; NUM_FIELDS_PER_SHA256],
      in_hash: [0; NUM_FIELDS_PER_SHA256],
      out_hash: [0; NUM_FIELDS_PER_SHA256],
    }
  }
}

impl Eq for ContentCommitment {
  fn eq(self, other: Self) -> bool {
    (self.tx_tree_height == other.tx_tree_height)
      & (self.txs_hash == other.txs_hash)
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
