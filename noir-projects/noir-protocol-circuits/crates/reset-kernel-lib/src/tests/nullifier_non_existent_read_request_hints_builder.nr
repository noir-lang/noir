use crate::nullifier_non_existent_read_request_reset::{NullifierNonMembershipHint, NullifierNonExistentReadRequestHints};
use dep::types::{
    abis::{nullifier::Nullifier, nullifier_leaf_preimage::NullifierLeafPreimage},
    constants::{
    MAX_NULLIFIERS_PER_TX, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX, NULLIFIER_TREE_HEIGHT,
    NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_HEIGHT
},
    merkle_tree::MembershipWitness, tests::{merkle_tree_utils::NonEmptyMerkleTree},
    utils::{arrays::{find_index, sort_get_sorted_hints}, field::full_field_greater_than}
};

struct NullifierNonExistentReadRequestHintsBuilder {
    nullifier_tree: NonEmptyMerkleTree<MAX_NULLIFIERS_PER_TX, NULLIFIER_TREE_HEIGHT, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_HEIGHT>,
    non_membership_hints: BoundedVec<NullifierNonMembershipHint, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX>,
    read_values: BoundedVec<Field, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX>,
    pending_nullifiers: [Nullifier; MAX_NULLIFIERS_PER_TX],
}

impl NullifierNonExistentReadRequestHintsBuilder {
    pub fn new() -> Self {
        NullifierNonExistentReadRequestHintsBuilder {
            nullifier_tree: NonEmptyMerkleTree::empty(),
            non_membership_hints: BoundedVec::new(),
            read_values: BoundedVec::new(),
            pending_nullifiers: [Nullifier::empty(); MAX_NULLIFIERS_PER_TX]
        }
    }

    pub fn set_nullifier_tree(
        &mut self,
        tree: NonEmptyMerkleTree<MAX_NULLIFIERS_PER_TX, NULLIFIER_TREE_HEIGHT, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_HEIGHT>
    ) {
        self.nullifier_tree = tree;
    }

    pub fn set_nullifiers(&mut self, nullifiers: [Nullifier; MAX_NULLIFIERS_PER_TX]) {
        self.pending_nullifiers = nullifiers;
    }

    pub fn add_value_read(&mut self, siloed_value: Field) {
        self.read_values.push(siloed_value);

        // There are only two pre-existing nullifiers in the tree: [0, 100], generated in public_kernel_tail::tests.
        // Assuming the siloed_value is always greater than 100.
        let hint = NullifierNonMembershipHint {
            low_leaf_preimage: NullifierLeafPreimage { nullifier: 100, next_nullifier: 0, next_index: 0 },
            membership_witness: MembershipWitness { leaf_index: 1, sibling_path: self.nullifier_tree.get_sibling_path(1) }
        };
        self.non_membership_hints.push(hint);
    }

    pub fn to_hints(self) -> NullifierNonExistentReadRequestHints {
        let sorted_result = sort_get_sorted_hints(
            self.pending_nullifiers,
            |a: Nullifier, b: Nullifier| a.value.lt(b.value)
        );
        let sorted_pending_values = sorted_result.sorted_array;
        let sorted_pending_value_index_hints = sorted_result.sorted_index_hints;

        let mut next_pending_value_indices = [0; MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX];
        for i in 0..MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX {
            if i < self.read_values.len() {
                let value = self.read_values.get_unchecked(i);
                next_pending_value_indices[i] = find_index(sorted_pending_values, |v: Nullifier| !v.value.lt(value));
            }
        }

        NullifierNonExistentReadRequestHints {
            non_membership_hints: self.non_membership_hints.storage,
            sorted_pending_values,
            sorted_pending_value_index_hints,
            next_pending_value_indices
        }
    }
}
