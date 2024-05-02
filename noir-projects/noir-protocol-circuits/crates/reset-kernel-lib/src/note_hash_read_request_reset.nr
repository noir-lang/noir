// This will be moved to a separate Read Request Reset Circuit.
use crate::reset::read_request::{PendingReadHint, ReadRequestStatus, ReadValueHint, SettledReadHint};
use dep::types::{
    abis::{membership_witness::NoteHashMembershipWitness, note_hash_leaf_preimage::NoteHashLeafPreimage},
    constants::{MAX_NOTE_HASH_READ_REQUESTS_PER_TX, NOTE_HASH_TREE_HEIGHT},
    merkle_tree::MembershipWitness
};

struct NoteHashSettledReadHint {
    read_request_index: u64,
    membership_witness: NoteHashMembershipWitness, // Should be MembershipWitness<NOTE_HASH_TREE_HEIGHT> when we can handle generics when converting to ts types.
    leaf_preimage: NoteHashLeafPreimage,
}

impl ReadValueHint for NoteHashSettledReadHint {
    fn read_request_index(self) -> u64 {
        self.read_request_index
    }
}

impl SettledReadHint<NOTE_HASH_TREE_HEIGHT, NoteHashLeafPreimage> for NoteHashSettledReadHint {
    fn membership_witness(self) -> MembershipWitness<NOTE_HASH_TREE_HEIGHT> {
        MembershipWitness { leaf_index: self.membership_witness.leaf_index, sibling_path: self.membership_witness.sibling_path }
    }

    fn leaf_preimage(self) -> NoteHashLeafPreimage {
        self.leaf_preimage
    }

    fn nada(read_request_len: u64) -> Self {
        NoteHashSettledReadHint {
            read_request_index: read_request_len,
            membership_witness: NoteHashMembershipWitness::empty(),
            leaf_preimage: NoteHashLeafPreimage::empty()
        }
    }
}

struct NoteHashReadRequestHints {
    read_request_statuses: [ReadRequestStatus; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
    pending_read_hints: [PendingReadHint; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
    settled_read_hints: [NoteHashSettledReadHint; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
}

mod tests {
    use crate::note_hash_read_request_reset::NoteHashSettledReadHint;
    use crate::reset::read_request::{PendingReadHint, ReadRequestState, ReadRequestStatus, reset_read_requests};
    use dep::types::{
        address::AztecAddress,
        abis::{
        membership_witness::NoteHashMembershipWitness, note_hash::NoteHashContext,
        note_hash_leaf_preimage::NoteHashLeafPreimage, read_request::ReadRequestContext
    },
        constants::NOTE_HASH_TREE_HEIGHT, hash::silo_note_hash,
        tests::merkle_tree_utils::NonEmptyMerkleTree
    };

    global contract_address = AztecAddress::from_field(123);

    // Create 4 note hashes. 10 and 11 are settled. 12 and 13 are pending.
    global inner_note_hashes = [10, 11, 12, 13];
    global note_hashes = inner_note_hashes.map(|n| silo_note_hash(contract_address, n));

    // Create 5 read requests. 0 and 3 are reading settled note hashes. 1, 2 and 4 are reading pending note hashes.
    // TODO(#2847): Read request values for settled note hashes shouldn't have been siloed by apps.
    global read_requests = [
        ReadRequestContext { value: note_hashes[1], counter: 11, contract_address }, // settled
        ReadRequestContext { value: inner_note_hashes[3], counter: 13, contract_address }, // pending
        ReadRequestContext { value: inner_note_hashes[2], counter: 39, contract_address }, // pending
        ReadRequestContext { value: note_hashes[0], counter: 46, contract_address }, // settled
        ReadRequestContext { value: inner_note_hashes[3], counter: 78, contract_address }, // pending
    ];

    // TODO(#6122): Pending values shouldn't have been siloed at this point.
    global pending_values = [NoteHashContext { value: note_hashes[2], counter: 2, nullifier_counter: 0 }, NoteHashContext { value: note_hashes[3], counter: 8, nullifier_counter: 0 }];
    global pending_read_hints = [
        PendingReadHint { read_request_index: 1, pending_value_index: 1 },
        PendingReadHint { read_request_index: 2, pending_value_index: 0 },
        PendingReadHint { read_request_index: 4, pending_value_index: 1 },
    ];

    global leaf_preimages = [
        NoteHashLeafPreimage { value: note_hashes[0] },
        NoteHashLeafPreimage { value: note_hashes[1] },
    ];

    fn read_request_statuses() -> [ReadRequestStatus; 5] {
        [
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 0 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 1 },
            ReadRequestStatus { state: ReadRequestState.SETTLED, hint_index: 1 },
            ReadRequestStatus { state: ReadRequestState.PENDING, hint_index: 2 }
        ]
    }

    fn build_tree<N>() -> NonEmptyMerkleTree<2, NOTE_HASH_TREE_HEIGHT, NOTE_HASH_TREE_HEIGHT - 1, 1> {
        NonEmptyMerkleTree::new(
            [leaf_preimages[0].as_leaf(), leaf_preimages[1].as_leaf()],
            [0; NOTE_HASH_TREE_HEIGHT],
            [0; NOTE_HASH_TREE_HEIGHT - 1],
            [0; 1]
        )
    }

    fn get_settled_read_hints() -> ([NoteHashSettledReadHint; 2], Field) {
        let tree = build_tree();
        let hints = [
            NoteHashSettledReadHint {
                read_request_index: 0,
                membership_witness: NoteHashMembershipWitness { leaf_index: 1, sibling_path: tree.get_sibling_path(1) },
                leaf_preimage: leaf_preimages[1]
            },
            NoteHashSettledReadHint {
                read_request_index: 3,
                membership_witness: NoteHashMembershipWitness { leaf_index: 0, sibling_path: tree.get_sibling_path(0) },
                leaf_preimage: leaf_preimages[0]
            }
        ];
        let tree_root = tree.get_root();
        (hints, tree_root)
    }

    #[test]
    fn test_reset_read_requests_all() {
        let (settled_read_hints, tree_root) = get_settled_read_hints();
        let unverified_read_requests = reset_read_requests(
            read_requests,
            pending_values,
            read_request_statuses(),
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
        assert(unverified_read_requests.len() == 0);
    }

    #[test]
    fn test_reset_read_requests_partial() {
        let mut partial_read_request_statuses = read_request_statuses();
        partial_read_request_statuses[2] = ReadRequestStatus::empty();
        partial_read_request_statuses[4] = ReadRequestStatus::empty();

        let (settled_read_hints, tree_root) = get_settled_read_hints();
        let unverified_read_requests = reset_read_requests(
            read_requests,
            pending_values,
            partial_read_request_statuses,
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
        assert(unverified_read_requests.len() == 2);
        assert(unverified_read_requests.get(0) == read_requests[2]);
        assert(unverified_read_requests.get(1) == read_requests[4]);
    }

    #[test(should_fail_with="Value of the note hash does not match read request")]
    fn test_reset_note_hash_read_requests_wrong_hinted_value() {
        let mut tainted_pending_values = pending_values;
        // Tweak the value to be something different.
        tainted_pending_values[0].value += 1;

        let (settled_read_hints, tree_root) = get_settled_read_hints();
        let _ = reset_read_requests(
            read_requests,
            tainted_pending_values,
            read_request_statuses(),
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
    }

    #[test(should_fail_with="Read request counter must be greater than the counter of the note hash")]
    fn test_reset_note_hash_read_requests_invalid_counter() {
        let hint = pending_read_hints[0];
        let pending_read = read_requests[hint.read_request_index];
        let mut tainted_pending_values = pending_values;
        // Tweak the counter of the value to be greater than the read request.
        tainted_pending_values[hint.pending_value_index].counter = pending_read.counter + 1;

        let (settled_read_hints, tree_root) = get_settled_read_hints();
        let _ = reset_read_requests(
            read_requests,
            tainted_pending_values,
            read_request_statuses(),
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
    }

    #[test(should_fail_with="Read request counter must be less than the nullifier counter of the note hash")]
    fn test_reset_note_hash_read_requests_invalid_nullifier_counter() {
        let hint = pending_read_hints[0];
        let pending_read = read_requests[hint.read_request_index];
        let mut tainted_pending_values = pending_values;
        // Tweak the nullifier counter to be less than the read request.
        tainted_pending_values[hint.pending_value_index].nullifier_counter = pending_read.counter - 1;

        let (settled_read_hints, tree_root) = get_settled_read_hints();
        let _ = reset_read_requests(
            read_requests,
            tainted_pending_values,
            read_request_statuses(),
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
    }

    #[test(should_fail_with="Value of the note hash leaf does not match read request")]
    fn test_reset_note_hash_read_requests_invalid_leaf() {
        let (settled_read_hints, tree_root) = get_settled_read_hints();

        let mut tained_read_requests = read_requests;
        let hint = settled_read_hints[0];
        // Tweak the value of the first settled read to be something different.
        tained_read_requests[hint.read_request_index].value += 1;

        let _ = reset_read_requests(
            tained_read_requests,
            pending_values,
            read_request_statuses(),
            pending_read_hints,
            settled_read_hints,
            tree_root
        );
    }
}

