use crate::{merkle_tree::{leaf_preimage::IndexedTreeLeafPreimage, root::root_from_sibling_path}, traits::Empty};

struct MembershipWitness<N> {
    leaf_index: Field,
    sibling_path: [Field; N]
}

impl<N> Empty for MembershipWitness<N> {
    fn empty() -> Self {
        MembershipWitness {
            leaf_index: 0,
            sibling_path: [0; N]
        }
    }
}

pub fn check_membership<N>(leaf: Field, index: Field, sibling_path: [Field; N], root: Field) -> bool {
    let calculated_root = root_from_sibling_path(leaf, index, sibling_path);
    calculated_root == root
}

pub fn assert_check_membership<N>(leaf: Field, index: Field, sibling_path: [Field; N], root: Field) {
    assert(check_membership(leaf, index, sibling_path, root), "membership check failed");
}

struct NonMembershipCheckErrorCodeEnum {
    NADA: u64,
    IS_EMPTY: u64,
    NOT_EXISTS: u64,
    NOT_GREATER_THAN_LOW: u64,
    NOT_LESS_THAN_NEXT: u64,
}

global NonMembershipCheckErrorCode = NonMembershipCheckErrorCodeEnum {
    NADA: 0,
    IS_EMPTY: 1,
    NOT_EXISTS: 2,
    NOT_GREATER_THAN_LOW: 3,
    NOT_LESS_THAN_NEXT: 4,
};

fn check_non_membership_internal<TREE_HEIGHT, LEAF_PREIMAGE>(
    key: Field,
    low_leaf_preimage: LEAF_PREIMAGE,
    low_leaf_membership_witness: MembershipWitness<TREE_HEIGHT>,
    tree_root: Field
) -> u64 where 
    LEAF_PREIMAGE: IndexedTreeLeafPreimage {
    let low_key = low_leaf_preimage.get_key();
    let next_key = low_leaf_preimage.get_next_key();
    let is_empty_leaf = (low_key == 0) & (next_key == 0);

    let low_leaf_exists = check_membership(
        low_leaf_preimage.as_leaf(),
        low_leaf_membership_witness.leaf_index,
        low_leaf_membership_witness.sibling_path,
        tree_root
    );

    if is_empty_leaf {
        NonMembershipCheckErrorCode.IS_EMPTY
    } else if !low_leaf_exists {
        NonMembershipCheckErrorCode.NOT_EXISTS
    } else if !low_key.lt(key) {
        NonMembershipCheckErrorCode.NOT_GREATER_THAN_LOW
    } else if !key.lt(next_key) & (next_key != 0) {
        NonMembershipCheckErrorCode.NOT_LESS_THAN_NEXT
    } else {
        NonMembershipCheckErrorCode.NADA
    }
}

pub fn check_non_membership<TREE_HEIGHT, LEAF_PREIMAGE>(
    key: Field,
    low_leaf_preimage: LEAF_PREIMAGE,
    low_leaf_membership_witness: MembershipWitness<TREE_HEIGHT>,
    tree_root: Field
) -> bool where 
    LEAF_PREIMAGE: IndexedTreeLeafPreimage {
    let error = check_non_membership_internal(key, low_leaf_preimage, low_leaf_membership_witness, tree_root);
    error == NonMembershipCheckErrorCode.NADA
}

pub fn assert_check_non_membership<TREE_HEIGHT, LEAF_PREIMAGE>(
    key: Field,
    low_leaf_preimage: LEAF_PREIMAGE,
    low_leaf_membership_witness: MembershipWitness<TREE_HEIGHT>,
    tree_root: Field
) where 
    LEAF_PREIMAGE: IndexedTreeLeafPreimage {
    let error = check_non_membership_internal(key, low_leaf_preimage, low_leaf_membership_witness, tree_root);
    if error != NonMembershipCheckErrorCode.NADA {
        assert(
            error != NonMembershipCheckErrorCode.IS_EMPTY, "Cannot check non membership against empty leaf"
        );
        assert(error != NonMembershipCheckErrorCode.NOT_EXISTS, "Low leaf does not exist");
        assert(
            error != NonMembershipCheckErrorCode.NOT_GREATER_THAN_LOW, "Key is not greater than the low leaf"
        );
        assert(
            error != NonMembershipCheckErrorCode.NOT_LESS_THAN_NEXT, "Key is not less than the next leaf"
        );
        assert(false, "Unknown error");
    }
}

mod tests {
    use crate::{
        merkle_tree::{
        leaf_preimage::{IndexedTreeLeafPreimage, LeafPreimage},
        membership::{
        assert_check_membership, assert_check_non_membership, check_membership, check_non_membership,
        MembershipWitness
    }
    },
        tests::merkle_tree_utils::NonEmptyMerkleTree
    };
    use dep::std::hash::pedersen_hash;

    struct TestLeafPreimage {
        value: Field,
        next_value: Field,
    }

    impl LeafPreimage for TestLeafPreimage {
        fn get_key(self) -> Field {
            self.value
        }

        fn as_leaf(self) -> Field {
            pedersen_hash([self.value])
        }
    }

    impl IndexedTreeLeafPreimage for TestLeafPreimage {
        fn get_key(self) -> Field {
            self.value
        }

        fn get_next_key(self) -> Field {
            self.next_value
        }

        fn as_leaf(self) -> Field {
            pedersen_hash([self.value])
        }
    }

    global leaf_preimages = [
        TestLeafPreimage { value: 20, next_value: 30 },
        TestLeafPreimage { value: 40, next_value: 0 },
        TestLeafPreimage { value: 10, next_value: 20 },
        TestLeafPreimage { value: 30, next_value: 40 },
    ];

    fn build_tree<N>() -> NonEmptyMerkleTree<4, 3, 1, 2> {
        NonEmptyMerkleTree::new(
            leaf_preimages.map(|leaf_preimage: TestLeafPreimage| leaf_preimage.as_leaf()),
            [0; 3],
            [0; 1],
            [0; 2]
        )
    }

    fn check_membership_at_index(leaf_index: Field, leaf: Field) -> bool {
        let tree = build_tree();
        let tree_root = tree.get_root();

        check_membership(
            leaf,
            leaf_index,
            tree.get_sibling_path(leaf_index as u64),
            tree_root
        )
    }

    fn assert_check_membership_at_index(leaf_index: Field, leaf: Field) {
        let tree = build_tree();
        let tree_root = tree.get_root();

        assert_check_membership(
            leaf,
            leaf_index,
            tree.get_sibling_path(leaf_index as u64),
            tree_root
        );
    }

    fn check_non_membership_at_index(low_leaf_index: u64, leaf: Field) -> bool {
        let tree = build_tree();
        let tree_root = tree.get_root();
        let leaf_preimage = if low_leaf_index < leaf_preimages.len() {
            leaf_preimages[low_leaf_index]
        } else {
            TestLeafPreimage { value: 0, next_value: 0 }
        };

        check_non_membership(
            leaf,
            leaf_preimage,
            MembershipWitness { leaf_index: low_leaf_index as Field, sibling_path: tree.get_sibling_path(low_leaf_index) } ,
            tree_root
        )
    }

    fn assert_check_non_membership_at_index(low_leaf_index: u64, leaf: Field) {
        let tree = build_tree();
        let tree_root = tree.get_root();
        let leaf_preimage = if low_leaf_index < leaf_preimages.len() {
            leaf_preimages[low_leaf_index]
        } else {
            TestLeafPreimage { value: 0, next_value: 0 }
        };

        assert_check_non_membership(
            leaf,
            leaf_preimage,
            MembershipWitness { leaf_index: low_leaf_index as Field, sibling_path: tree.get_sibling_path(low_leaf_index) } ,
            tree_root
        );
    }

    #[test]
    fn test_check_membership() {
        assert_eq(check_membership_at_index(0, leaf_preimages[0].as_leaf()), true);
        assert_eq(check_membership_at_index(2, leaf_preimages[2].as_leaf()), true);
    }

    #[test]
    fn test_assert_check_membership() {
        assert_check_membership_at_index(0, leaf_preimages[0].as_leaf());
        assert_check_membership_at_index(2, leaf_preimages[2].as_leaf());
    }

    #[test]
    fn test_check_membership_false_wrong_leaf() {
        assert_eq(check_membership_at_index(0, leaf_preimages[1].as_leaf()), false);
        assert_eq(check_membership_at_index(2, leaf_preimages[0].as_leaf()), false);
    }

    #[test(should_fail_with="membership check failed")]
    fn test_assert_check_membership_failed_wrong_leaf() {
        assert_check_membership_at_index(0, leaf_preimages[1].as_leaf());
    }

    #[test]
    fn test_check_membership_false_wrong_root() {
        let tree = build_tree();
        let tree_root = 56;

        let res = check_membership(
            leaf_preimages[0].as_leaf(),
            0,
            tree.get_sibling_path(0),
            tree_root
        );
        assert_eq(res, false);
    }

    #[test(should_fail_with="membership check failed")]
    fn test_assert_check_membership_false_wrong_root() {
        let tree = build_tree();
        let tree_root = 56;

        assert_check_membership(
            leaf_preimages[0].as_leaf(),
            0,
            tree.get_sibling_path(0),
            tree_root
        );
    }

    #[test]
    fn test_check_non_membership() {
        assert_eq(check_non_membership_at_index(0, 25), true);
    }

    #[test]
    fn test_assert_check_non_membership() {
        assert_check_non_membership_at_index(0, 25);
    }

    #[test]
    fn test_check_non_membership_greater_than_max() {
        assert_eq(check_non_membership_at_index(1, 45), true);
    }

    #[test]
    fn test_assert_check_non_membership_greater_than_max() {
        assert_check_non_membership_at_index(1, 45);
    }

    #[test]
    fn test_check_non_membership_false_empty_leaf() {
        assert_eq(check_non_membership_at_index(4, 25), false);
    }

    #[test(should_fail_with="Cannot check non membership against empty leaf")]
    fn test_assert_check_non_membership_failed_empty_leaf() {
        assert_check_non_membership_at_index(4, 25);
    }

    #[test]
    fn test_check_non_membership_false_wrong_low_leaf() {
        assert_eq(check_non_membership_at_index(3, 25), false);
    }

    #[test(should_fail_with="Key is not greater than the low leaf")]
    fn test_assert_check_non_membership_failed_wrong_low_leaf() {
        assert_check_non_membership_at_index(3, 25);
    }

    #[test]
    fn test_check_non_membership_false_wrong_next_key() {
        assert_eq(check_non_membership_at_index(2, 25), false);
    }

    #[test(should_fail_with="Key is not less than the next leaf")]
    fn test_assert_check_non_membership_failed_wrong_next_key() {
        assert_check_non_membership_at_index(2, 25);
    }

    #[test]
    fn test_check_non_membership_false_invalid_leaf() {
        let tree = build_tree();
        let tree_root = tree.get_root();

        let fake_leaf = TestLeafPreimage { value: 50, next_value: 60 };
        assert_eq(
            check_non_membership(
                55,
                fake_leaf,
                MembershipWitness { leaf_index: 1, sibling_path: tree.get_sibling_path(1) } ,
                tree_root
            ), false
        );
    }

    #[test(should_fail_with="Low leaf does not exist")]
    fn test_assert_check_non_membership_failed_invalid_leaf() {
        let tree = build_tree();
        let tree_root = tree.get_root();

        let fake_leaf = TestLeafPreimage { value: 50, next_value: 60 };
        assert_check_non_membership(
            55,
            fake_leaf,
            MembershipWitness { leaf_index: 1, sibling_path: tree.get_sibling_path(1) } ,
            tree_root
        );
    }
}
