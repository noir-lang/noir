use dep::aztec::context::{PrivateContext, PublicContext, Context};
use dep::aztec::types::type_serialization::TypeSerializationInterface;
use dep::aztec::oracle::storage::{storage_read, storage_write};

use dep::std::hash::pedersen_hash;
use dep::std::merkle::compute_merkle_root;
use dep::std::option::Option;

// The epoch length is just a random number for now.
global EPOCH_LENGTH: u120 = 100;

fn compute_next_change(time: Field) -> Field {
    ((time as u120 / EPOCH_LENGTH + 1) * EPOCH_LENGTH) as Field
}

// A leaf in the tree.
struct Leaf {
  next_change: Field,
  before: Field,
  after: Field,
}

fn serialize_leaf(leaf: Leaf) -> [Field; 3] {
    [leaf.next_change, leaf.before, leaf.after]
}

fn deserialize_leaf(serialized: [Field; 3]) -> Leaf {
    Leaf { next_change: serialized[0], before: serialized[1], after: serialized[2] }
}

impl Leaf {
  fn serialize(self: Self) -> [Field; 3] {
    serialize_leaf(self)
  }
   fn deserialize(serialized: [Field; 3]) -> Self {
    deserialize_leaf(serialized)
  }
}

// Subset of the MembershipProof that is needed for the slow update.
struct SlowUpdateInner<N> {
  value: Field, // Value only really used for the private flow though :thinking:
  sibling_path: [Field; N],
}

// The slow update proof. Containing two merkle paths
// One for the before and one for the after trees.
// M = 2 * N + 4
struct SlowUpdateProof<N, M> {
  index: Field,
  new_value: Field,
  before: SlowUpdateInner<N>,
  after: SlowUpdateInner<N>,
}

pub fn deserialize_slow_update_proof<N, M>(serialized: [Field; M]) -> SlowUpdateProof<N, M> {
    SlowUpdateProof::deserialize(serialized)
}

impl<N, M> SlowUpdateProof<N, M> {
  pub fn serialize(self: Self) -> [Field; M] {
    let mut serialized = [0; M];
    serialized[0] = self.index;
    serialized[1] = self.new_value;
    serialized[2] = self.before.value;
    serialized[3 + N] = self.after.value;

    for i in 0..N {
      serialized[3 + i] = self.before.sibling_path[i];
      serialized[4 + N + i] = self.after.sibling_path[i];
    }
    serialized
  }

  pub fn deserialize(serialized: [Field; M]) -> Self {
    let mut before_sibling_path = [0; N];
    let mut after_sibling_path = [0; N];

    for i in 0..N {
      before_sibling_path[i] = serialized[3 + i];
      after_sibling_path[i] = serialized[4 + N + i];
    }

    Self {
      index: serialized[0],
      new_value: serialized[1],
      before: SlowUpdateInner {
        value: serialized[2],
        sibling_path: before_sibling_path,
      },
      after: SlowUpdateInner {
        value: serialized[3 + N],
        sibling_path: after_sibling_path,
      },
    }
  }
}

// The simple slow map which stores a sparse tree
struct SlowMap<N,M> {
  context: Context,
  storage_slot: Field
}

impl<N,M> SlowMap<N,M> {
  pub fn new(
    context: Context,
    storage_slot: Field
  ) -> Self {
    assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
    Self {
      context,
      storage_slot,
    }
  }

  pub fn read_root(self: Self) -> Leaf {
    storage_read(self.storage_slot, deserialize_leaf)
  }

  // Beware that the initial root could include much state that is not shown by the public storage!
  pub fn initialize(self: Self, initial_root: Field) {
    let mut root_object = self.read_root();
    assert(root_object.next_change == 0, "cannot initialize twice");
    root_object = Leaf {
      next_change: 0xffffffffffffffffffffffffffffff,
      before: initial_root,
      after: initial_root,
    };
    let fields = root_object.serialize();
    storage_write(self.storage_slot, fields);
  }

  // Reads the "CURRENT" value of the root
  pub fn current_root(self: Self) -> Field {
    let time = self.context.public.unwrap().timestamp() as u120;
    let root_object = self.read_root();
    if time <= root_object.next_change as u120 {
      root_object.before
    } else {
      root_object.after
    }
  }

  pub fn read_leaf_at(self: Self, key: Field) -> Leaf {
    let derived_storage_slot = pedersen_hash([self.storage_slot, key]);
    storage_read(derived_storage_slot, deserialize_leaf)
  }

  // Reads the "CURRENT" value of the leaf
  pub fn read_at(self: Self, key: Field) -> Field {
    let time = self.context.public.unwrap().timestamp() as u120;
    let leaf = self.read_leaf_at(key);
    if time <= leaf.next_change as u120 {
      leaf.before
    } else {
      leaf.after
    }
  }

  // Will update values in the "AFTER" tree
  // - updates the leaf and root to follow current values, moving from after to before if 
  //  needed.
  // - checks that the provided merkle paths match state values
  // - update the leaf and compute the net root
  // Should only be used when updates from public are desired, since the hashing will be
  // costly since done by sequencer.
  pub fn update_at(self: Self, p: SlowUpdateProof<N, M>) {
    // The calling function should ensure that the index is within the tree.
    // This must be done separately to ensure we are not constraining too tight here.

    let time = self.context.public.unwrap().timestamp() as u120;
    let next_change = compute_next_change(time as Field);

    let mut root = self.read_root();
    let mut leaf = self.read_leaf_at(p.index);

    // Move leaf if needed
    if time > leaf.next_change as u120 {
      leaf.before = leaf.after; 
    }
    
    // Move root if needed
    if time > root.next_change as u120 {
      root.before = root.after;
    }

    // Ensures that when before is active, it is not altered by this update
    assert(
      root.before == compute_merkle_root(leaf.before, p.index, p.before.sibling_path), 
      "Before root don't match"
    );

    // Ensures that the provided sibling path is valid for the CURRENT "after" tree.
    // Without this check, someone could provide a sibling path for a different tree
    // and update the entire "after" tree at once, causing it to be out of sync with leaf storage.
    assert(
      root.after == compute_merkle_root(leaf.after, p.index, p.after.sibling_path), 
      "After root don't match"
    );

    // Update the leaf 
    leaf.after = p.new_value;
    leaf.next_change = next_change;

    // Update the after root
    root.after = compute_merkle_root(leaf.after, p.index, p.after.sibling_path);
    root.next_change = next_change;

    self.update_unsafe(p.index, leaf, root);
  }

  // A variation of `update_at` that skips the merkle-membership checks.
  // To be used by a contract which has already checked the merkle-membership.
  // This allows us to check the merkle-memberships in private and then update
  // in public, limiting the cost of the update.
  pub fn update_unsafe_at(self: Self, index: Field, leaf_value: Field, new_root: Field) {
    // User must ensure that the checks from update_at is performed for safety
    let time = self.context.public.unwrap().timestamp() as u120;
    let next_change = compute_next_change(time as Field);

    let mut root = self.read_root();
    let mut leaf = self.read_leaf_at(index);

    // Move leaf if needed
    if time > leaf.next_change as u120 {
      leaf.before = leaf.after; 
    }
    
    // Move root if needed
    if time > root.next_change as u120 {
      root.before = root.after;
    }

    // Update the leaf 
    leaf.after = leaf_value;
    leaf.next_change = next_change;
    
    // Update the root
    root.after = new_root;
    root.next_change = next_change;

    self.update_unsafe(index, leaf, root);
  }

  // Updates the value in the in storage with no checks.
  fn update_unsafe(self: Self, index: Field, leaf: Leaf, root: Leaf) {
    let derived_storage_slot = pedersen_hash([self.storage_slot, index]);
    let fields = leaf.serialize();
    storage_write(derived_storage_slot, fields);
    
    let fields = root.serialize();
    storage_write(self.storage_slot, fields);
  }
}

/*pub fn compute_merkle_root<N>(leaf: Field, index: Field, hash_path: [Field; N]) -> Field {
    let n = hash_path.len();
    let index_bits = index.to_le_bits(n as u32);
    let mut current = leaf;
    for i in 0..n {
        let path_bit = index_bits[i] as bool;
        let (hash_left, hash_right) = if path_bit {
            (hash_path[i], current)
        } else {
            (current, hash_path[i])
        };
    current = pedersen_hash([hash_left, hash_right]);
    };
    current
}
*/
