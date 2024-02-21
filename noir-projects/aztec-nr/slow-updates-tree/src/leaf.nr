use dep::aztec::protocol_types::traits::{Serialize, Deserialize};

// A leaf in the tree.
struct Leaf {
  next_change: Field,
  before: Field,
  after: Field,
}

impl Serialize<3> for Leaf {
  fn serialize(leaf: Leaf) -> [Field; 3] {
      [leaf.next_change, leaf.before, leaf.after]
  }
}

impl Deserialize<3> for Leaf {
  fn deserialize(serialized: [Field; 3]) -> Leaf {
      Leaf { next_change: serialized[0], before: serialized[1], after: serialized[2] }
  }
}
