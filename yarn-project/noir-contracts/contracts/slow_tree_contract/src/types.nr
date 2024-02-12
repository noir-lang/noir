// docs:start:membership_proof
// A single inclusion proof.
// M = N + 2
struct MembershipProof<N, M> {
    index: Field,
    value: Field,
    sibling_path: [Field; N],
}
// docs:end:membership_proof

fn deserialize_membership_proof<N, M>(serialized: [Field; M]) -> MembershipProof<N, M> {
    let mut sibling_path = [0; N];
    for i in 0..N {
        sibling_path[i] = serialized[2 + i];
    }
    MembershipProof { index: serialized[0], value: serialized[1], sibling_path }
}

impl<N, M> MembershipProof<N, M> {
    fn serialize(self: Self) -> [Field; M] {
        let mut serialized = [0; M];
        serialized[0] = self.index;
        serialized[1] = self.value;
        for i in 0..N {
            serialized[2 + i] = self.sibling_path[i];
        }
        serialized
    }

    fn deserialize(serialized: [Field; M]) -> Self {
        deserialize_membership_proof(serialized)
    }
}
