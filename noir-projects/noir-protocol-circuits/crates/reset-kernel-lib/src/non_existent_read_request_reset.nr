use dep::types::{
    abis::{side_effect::OrderedValue, read_request::ReadRequestContext},
    merkle_tree::{assert_check_non_membership, IndexedTreeLeafPreimage, MembershipWitness},
    traits::{Empty, is_empty}
};

trait NonMembershipHint<TREE_HEIGHT, LEAF_PREIMAGE> where LEAF_PREIMAGE: IndexedTreeLeafPreimage {
    fn low_leaf_preimage(self) -> LEAF_PREIMAGE;
    fn membership_witness(self) -> MembershipWitness<TREE_HEIGHT>;
}

fn check_no_matching_pending_value<T, N>(
    read_request: ReadRequestContext,
    sorted_pending_values: BoundedVec<T, N>,
    next_value_index: u64
) -> bool where T: OrderedValue<Field> {
    if next_value_index == sorted_pending_values.len() {
        let highest_value = sorted_pending_values.get_unchecked(sorted_pending_values.len() - 1).value();
        highest_value.lt(read_request.value)
    } else {
        let next_value = sorted_pending_values.get_unchecked(next_value_index).value();
        let is_less_than_next = read_request.value.lt(next_value);
        let is_greater_than_prev = if next_value_index == 0 {
            true
        } else {
            let prev_value = sorted_pending_values.get_unchecked(next_value_index - 1).value();
            prev_value.lt(read_request.value)
        };
        is_less_than_next & is_greater_than_prev
    }
}

fn check_is_read_before_pending_value<T, N>(
    read_request: ReadRequestContext,
    sorted_pending_values: BoundedVec<T, N>,
    next_value_index: u64
) -> bool where T: OrderedValue<Field> {
    if next_value_index == sorted_pending_values.len() {
        false
    } else {
        let pending = sorted_pending_values.get_unchecked(next_value_index);
        if pending.value() == read_request.value {
            assert(read_request.counter < pending.counter(), "Value exists in pending set");
            true
        } else {
            false
        }
    }
}

// Unlike regular read requests, which can be reset at any time between two function executions.
// Non existent read requests can only be verified at the end, after all pending values are present.
// The values in read_requests and in sorted_pending_values should've been siloed before calling this.
pub fn reset_non_existent_read_requests<T, N, M, NON_MEMBERSHIP_HINT, TREE_HEIGHT, LEAF_PREIMAGE>(
    siloed_read_requests: [ReadRequestContext; N],
    non_membership_hints: [NON_MEMBERSHIP_HINT; N],
    tree_root: Field,
    sorted_pending_values: BoundedVec<T, M>,
    next_pending_value_indices: [u64; N]
) where 
    T: OrderedValue<Field>, 
    NON_MEMBERSHIP_HINT: NonMembershipHint<TREE_HEIGHT, LEAF_PREIMAGE>,
    LEAF_PREIMAGE: IndexedTreeLeafPreimage {
    for i in 0..siloed_read_requests.len() {
        let read_request = siloed_read_requests[i];
        if !is_empty(read_request) {
            // Verify that it's not in the tree.
            let hint = non_membership_hints[i];
            assert_check_non_membership(
                read_request.value,
                hint.low_leaf_preimage(),
                hint.membership_witness(),
                tree_root
            );

            // Verify that its value is either not in the pending set, or is created after the read.
            let next_value_index = next_pending_value_indices[i];
            assert(
                next_value_index <= sorted_pending_values.len(), "Next pending value index out of bounds"
            );
            let no_matching_value = check_no_matching_pending_value(read_request, sorted_pending_values, next_value_index);
            let is_read_before_value = check_is_read_before_pending_value(read_request, sorted_pending_values, next_value_index);
            assert(no_matching_value | is_read_before_value, "Invalid next pending value index");
        }
    }
}

mod tests {
    use crate::non_existent_read_request_reset::{NonMembershipHint, reset_non_existent_read_requests};

    use dep::types::{
        address::AztecAddress, abis::{read_request::ReadRequestContext, side_effect::SideEffect},
        merkle_tree::{leaf_preimage::IndexedTreeLeafPreimage, membership::MembershipWitness},
        hash::silo_nullifier, tests::merkle_tree_utils::NonEmptyMerkleTree
    };

    struct TestLeafPreimage {
        value: Field,
        next_value: Field,
    }

    impl IndexedTreeLeafPreimage for TestLeafPreimage {
        fn get_key(self) -> Field {
            self.value
        }

        fn get_next_key(self) -> Field {
            self.next_value
        }

        fn as_leaf(self) -> Field {
            self.value * 100
        }
    }

    struct TestNonMembershipHint {
        low_leaf_preimage: TestLeafPreimage,
        membership_witness: MembershipWitness<3>,
    }

    impl NonMembershipHint<3, TestLeafPreimage> for TestNonMembershipHint {
        fn low_leaf_preimage(self) -> TestLeafPreimage {
            self.low_leaf_preimage
        }

        fn membership_witness(self) -> MembershipWitness<3> {
            self.membership_witness
        }
    }

    global sorted_pending_values = BoundedVec {
        storage: [
        SideEffect { value: 5, counter: 17 },
        SideEffect { value: 15, counter: 8 },
        SideEffect { value: 25, counter: 11 },
        SideEffect::empty(),
        SideEffect::empty(),
    ],
    len: 3,
    };

    global leaf_preimages = [
        TestLeafPreimage { value: 0, next_value: 10 },
        TestLeafPreimage { value: 20, next_value: 30 },
        TestLeafPreimage { value: 30, next_value: 0 },
        TestLeafPreimage { value: 10, next_value: 20 },
    ];

    fn build_tree<N>() -> NonEmptyMerkleTree<4, 3, 1, 2> {
        NonEmptyMerkleTree::new(
            leaf_preimages.map(|leaf_preimage: TestLeafPreimage| leaf_preimage.as_leaf()),
            [0; 3],
            [0; 1],
            [0; 2]
        )
    }

    fn get_non_membership_hints<N>(leaf_indices: [Field; N]) -> ([TestNonMembershipHint; N], Field) {
        let tree = build_tree();
        let hints = leaf_indices.map(
            |leaf_index| TestNonMembershipHint {
                low_leaf_preimage: leaf_preimages[leaf_index],
                membership_witness: MembershipWitness { leaf_index, sibling_path: tree.get_sibling_path(leaf_index as u64) }
            }
        );
        let tree_root = tree.get_root();
        (hints, tree_root)
    }

    #[test]
    fn test_reset_non_existent_read_requests_in_range() {
        let read_requests = [
            ReadRequestContext { value: 11, counter: 50, contract_address: AztecAddress::zero() },
            ReadRequestContext { value: 22, counter: 51, contract_address: AztecAddress::zero() },
            ReadRequestContext { value: 6, counter: 52, contract_address: AztecAddress::zero() }
        ];
        let (non_membership_hints, root) = get_non_membership_hints([3, 1, 0]);
        let next_pending_value_indices = [1, 2, 1];
        reset_non_existent_read_requests(
            read_requests,
            non_membership_hints,
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }

    #[test]
    fn test_reset_non_existent_read_requests_less_than_min() {
        let read_requests = [
            ReadRequestContext { value: 3, counter: 50, contract_address: AztecAddress::zero() },
            ReadRequestContext { value: 2, counter: 51, contract_address: AztecAddress::zero() }
        ];
        let (non_membership_hints, root) = get_non_membership_hints([0, 0]);
        let next_pending_value_indices = [0, 0];
        reset_non_existent_read_requests(
            read_requests,
            non_membership_hints,
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }

    #[test]
    fn test_reset_non_existent_read_requests_greater_than_max() {
        let read_requests = [
            ReadRequestContext { value: 35, counter: 50, contract_address: AztecAddress::zero() },
            ReadRequestContext { value: 31, counter: 51, contract_address: AztecAddress::zero() }
        ];
        let (non_membership_hints, root) = get_non_membership_hints([2, 2]);
        let next_pending_value_indices = [3, 3];
        reset_non_existent_read_requests(
            read_requests,
            non_membership_hints,
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }

    #[test]
    fn test_reset_non_existent_read_requests_read_before_pending_emitted() {
        let read_requests = [
            ReadRequestContext { value: 25, counter: 10, contract_address: AztecAddress::zero() },
            ReadRequestContext { value: 5, counter: 11, contract_address: AztecAddress::zero() }
        ];
        let (non_membership_hints, root) = get_non_membership_hints([1, 0]);
        let next_pending_value_indices = [2, 0];
        reset_non_existent_read_requests(
            read_requests,
            non_membership_hints,
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }

    #[test(should_fail_with="Cannot check non membership against empty leaf")]
    fn test_reset_non_existent_read_requests_empty_leaf_failed() {
        let read_requests = [ReadRequestContext { value: 10, counter: 50, contract_address: AztecAddress::zero() }];
        let (non_membership_hints, root) = get_non_membership_hints([0]);
        let mut hint = non_membership_hints[0];
        hint.low_leaf_preimage = TestLeafPreimage { value: 0, next_value: 0 };
        let next_pending_value_indices = [1];
        reset_non_existent_read_requests(
            read_requests,
            [hint],
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }

    #[test(should_fail_with="Low leaf does not exist")]
    fn test_reset_non_existent_read_requests_invalid_preimage_failed() {
        let read_requests = [ReadRequestContext { value: 10, counter: 50, contract_address: AztecAddress::zero() }];
        let (non_membership_hints, root) = get_non_membership_hints([3]);
        let mut hint = non_membership_hints[0];
        hint.low_leaf_preimage = TestLeafPreimage { value: 9, next_value: 20 };
        let next_pending_value_indices = [1];
        reset_non_existent_read_requests(
            read_requests,
            [hint],
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }

    #[test(should_fail_with="Key is not greater than the low leaf")]
    fn test_reset_non_existent_read_requests_read_settled_failed() {
        let read_requests = [ReadRequestContext { value: 10, counter: 50, contract_address: AztecAddress::zero() }];
        let (non_membership_hints, root) = get_non_membership_hints([3]);
        let next_pending_value_indices = [1];
        reset_non_existent_read_requests(
            read_requests,
            non_membership_hints,
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }

    #[test(should_fail_with="Key is not less than the next leaf")]
    fn test_reset_non_existent_read_requests_invalid_non_membership_hint_failed() {
        let read_requests = [ReadRequestContext { value: 10, counter: 50, contract_address: AztecAddress::zero() }];
        let (non_membership_hints, root) = get_non_membership_hints([0]);
        let next_pending_value_indices = [1];
        reset_non_existent_read_requests(
            read_requests,
            non_membership_hints,
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }

    #[test(should_fail_with="Value exists in pending set")]
    fn test_reset_non_existent_read_requests_read_pending_value_failed() {
        let read_requests = [ReadRequestContext { value: 25, counter: 50, contract_address: AztecAddress::zero() }];
        let (non_membership_hints, root) = get_non_membership_hints([1]);
        let next_pending_value_indices = [2];
        reset_non_existent_read_requests(
            read_requests,
            non_membership_hints,
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }

    #[test(should_fail_with="Invalid next pending value index")]
    fn test_reset_non_existent_read_requests_wrong_next_pending_index_failed() {
        let read_requests = [ReadRequestContext { value: 21, counter: 50, contract_address: AztecAddress::zero() }];
        let (non_membership_hints, root) = get_non_membership_hints([1]);
        let next_pending_value_indices = [1];
        reset_non_existent_read_requests(
            read_requests,
            non_membership_hints,
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }

    #[test(should_fail_with="Invalid next pending value index")]
    fn test_reset_non_existent_read_requests_wrong_max_next_pending_index_failed() {
        let read_requests = [ReadRequestContext { value: 21, counter: 50, contract_address: AztecAddress::zero() }];
        let (non_membership_hints, root) = get_non_membership_hints([1]);
        let next_pending_value_indices = [3];
        reset_non_existent_read_requests(
            read_requests,
            non_membership_hints,
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }

    #[test(should_fail_with="Next pending value index out of bounds")]
    fn test_reset_non_existent_read_requests_overflown_index_failed() {
        let read_requests = [ReadRequestContext { value: 21, counter: 50, contract_address: AztecAddress::zero() }];
        let (non_membership_hints, root) = get_non_membership_hints([1]);
        let next_pending_value_indices = [4];
        reset_non_existent_read_requests(
            read_requests,
            non_membership_hints,
            root,
            sorted_pending_values,
            next_pending_value_indices
        );
    }
}
