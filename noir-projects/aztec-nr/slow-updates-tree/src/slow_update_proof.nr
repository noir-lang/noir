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
            before: SlowUpdateInner { value: serialized[2], sibling_path: before_sibling_path },
            after: SlowUpdateInner { value: serialized[3 + N], sibling_path: after_sibling_path }
        }
    }
}
