use crate::{
    merkle_tree::{
        indexed_tree::check_valid_low_leaf::assert_check_valid_low_leaf,
        leaf_preimage::IndexedTreeLeafPreimage, root::root_from_sibling_path,
    },
    traits::Empty,
};

pub struct MembershipWitness<let N: u32> {
    pub leaf_index: Field,
    pub sibling_path: [Field; N],
}

impl<let N: u32> Empty for MembershipWitness<N> {
    fn empty() -> Self {
        MembershipWitness { leaf_index: 0, sibling_path: [0; N] }
    }
}

pub fn check_membership<let N: u32>(
    leaf: Field,
    index: Field,
    sibling_path: [Field; N],
    root: Field,
) -> bool {
    let calculated_root = root_from_sibling_path(leaf, index, sibling_path);
    calculated_root == root
}

pub fn assert_check_membership<let TREE_HEIGHT: u32>(
    leaf: Field,
    index: Field,
    sibling_path: [Field; TREE_HEIGHT],
    root: Field,
) {
    assert(check_membership(leaf, index, sibling_path, root), "membership check failed");
}

pub fn assert_check_non_membership<let TREE_HEIGHT: u32, LEAF_PREIMAGE, VALUE>(
    key: Field,
    low_leaf_preimage: LEAF_PREIMAGE,
    low_leaf_membership_witness: MembershipWitness<TREE_HEIGHT>,
    tree_root: Field,
)
where
    LEAF_PREIMAGE: IndexedTreeLeafPreimage<VALUE>,
{
    assert_check_valid_low_leaf(key, low_leaf_preimage);

    let low_leaf_exists = check_membership(
        low_leaf_preimage.as_leaf(),
        low_leaf_membership_witness.leaf_index,
        low_leaf_membership_witness.sibling_path,
        tree_root,
    );
    assert(low_leaf_exists, "Low leaf does not exist");
}

// Prove either membership or non-membership depending on the value of `exists`.
// If `exists` == false, `key` is not in the tree, `leaf_preimage` and `membership_witness` are for the low leaf.
pub fn conditionally_assert_check_membership<let TREE_HEIGHT: u32, LEAF_PREIMAGE, VALUE>(
    key: Field,
    exists: bool,
    leaf_preimage: LEAF_PREIMAGE,
    membership_witness: MembershipWitness<TREE_HEIGHT>,
    tree_root: Field,
)
where
    LEAF_PREIMAGE: IndexedTreeLeafPreimage<VALUE>,
{
    if exists {
        assert(key == leaf_preimage.get_key(), "Key does not match the key of the leaf preimage");
    } else {
        assert_check_valid_low_leaf(key, leaf_preimage);
    }

    assert_check_membership(
        leaf_preimage.as_leaf(),
        membership_witness.leaf_index,
        membership_witness.sibling_path,
        tree_root,
    );
}

mod tests {
    use crate::{
        merkle_tree::{
            leaf_preimage::{IndexedTreeLeafPreimage, LeafPreimage},
            membership::{
                assert_check_membership, assert_check_non_membership, check_membership,
                conditionally_assert_check_membership, MembershipWitness,
            },
        },
        tests::merkle_tree_utils::NonEmptyMerkleTree,
    };
    use crate::traits::Empty;
    use std::hash::pedersen_hash;

    struct TestLeafPreimage {
        value: Field,
        next_value: Field,
    }

    impl Empty for TestLeafPreimage {
        fn empty() -> Self {
            TestLeafPreimage { value: 0, next_value: 0 }
        }
    }

    impl LeafPreimage for TestLeafPreimage {
        fn get_key(self) -> Field {
            self.value
        }

        fn as_leaf(self) -> Field {
            pedersen_hash([self.value])
        }
    }

    impl Eq for TestLeafPreimage {
        fn eq(self, other: Self) -> bool {
            (self.value == other.value) & (self.next_value == other.next_value)
        }
    }

    impl IndexedTreeLeafPreimage<Field> for TestLeafPreimage {
        fn get_next_key(self) -> Field {
            self.next_value
        }

        fn points_to_infinity(self) -> bool {
            (self.next_value == 0)
        }

        fn update_pointers(self, next_value: Field, _next_index: u32) -> Self {
            Self { value: self.value, next_value }
        }

        fn update_value(self, value: Field) -> Self {
            Self { value, next_value: self.next_value }
        }

        fn build_insertion_leaf(value: Field, low_leaf: Self) -> Self {
            Self { value, next_value: low_leaf.next_value }
        }
    }

    global leaf_preimages: [TestLeafPreimage; 4] = [
        TestLeafPreimage { value: 20, next_value: 30 },
        TestLeafPreimage { value: 40, next_value: 0 },
        TestLeafPreimage { value: 10, next_value: 20 },
        TestLeafPreimage { value: 30, next_value: 40 },
    ];

    fn build_tree() -> NonEmptyMerkleTree<4, 3, 1, 2> {
        NonEmptyMerkleTree::new(
            leaf_preimages.map(|leaf_preimage: TestLeafPreimage| leaf_preimage.as_leaf()),
            [0; 3],
            [0; 1],
            [0; 2],
        )
    }

    fn check_membership_at_index(leaf_index: Field, leaf: Field) -> bool {
        let tree = build_tree();
        let tree_root = tree.get_root();

        check_membership(
            leaf,
            leaf_index,
            tree.get_sibling_path(leaf_index as u32),
            tree_root,
        )
    }

    fn assert_check_membership_at_index(leaf_index: Field, leaf: Field) {
        let tree = build_tree();
        let tree_root = tree.get_root();

        assert_check_membership(
            leaf,
            leaf_index,
            tree.get_sibling_path(leaf_index as u32),
            tree_root,
        );
    }

    fn assert_check_non_membership_at_index(low_leaf_index: u32, key: Field) {
        let tree = build_tree();
        let tree_root = tree.get_root();
        let leaf_preimage = if low_leaf_index as u32 < leaf_preimages.len() {
            leaf_preimages[low_leaf_index]
        } else {
            TestLeafPreimage { value: 0, next_value: 0 }
        };

        assert_check_non_membership(
            key,
            leaf_preimage,
            MembershipWitness {
                leaf_index: low_leaf_index as Field,
                sibling_path: tree.get_sibling_path(low_leaf_index),
            },
            tree_root,
        );
    }

    fn conditionally_assert_check_membership_at_index(
        exists: bool,
        low_leaf_index: u32,
        key: Field,
    ) {
        let tree = build_tree();
        let tree_root = tree.get_root();
        let leaf_preimage = if low_leaf_index as u32 < leaf_preimages.len() {
            leaf_preimages[low_leaf_index]
        } else {
            TestLeafPreimage { value: 0, next_value: 0 }
        };

        conditionally_assert_check_membership(
            key,
            exists,
            leaf_preimage,
            MembershipWitness {
                leaf_index: low_leaf_index as Field,
                sibling_path: tree.get_sibling_path(low_leaf_index),
            },
            tree_root,
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

    #[test(should_fail_with = "membership check failed")]
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
            tree_root,
        );
        assert_eq(res, false);
    }

    #[test(should_fail_with = "membership check failed")]
    fn test_assert_check_membership_false_wrong_root() {
        let tree = build_tree();
        let tree_root = 56;

        assert_check_membership(
            leaf_preimages[0].as_leaf(),
            0,
            tree.get_sibling_path(0),
            tree_root,
        );
    }

    #[test]
    fn test_assert_check_non_membership() {
        assert_check_non_membership_at_index(0, 25);
    }

    #[test]
    fn test_assert_check_non_membership_greater_than_max() {
        assert_check_non_membership_at_index(1, 45);
    }

    #[test(should_fail_with = "Key is not greater than the low leaf")]
    fn test_assert_check_non_membership_failed_wrong_low_leaf() {
        assert_check_non_membership_at_index(3, 25);
    }

    #[test(should_fail_with = "Key is not less than the next leaf")]
    fn test_assert_check_non_membership_failed_wrong_next_key() {
        assert_check_non_membership_at_index(2, 25);
    }

    #[test(should_fail_with = "Low leaf does not exist")]
    fn test_assert_check_non_membership_failed_invalid_leaf() {
        let tree = build_tree();
        let tree_root = tree.get_root();

        let fake_leaf = TestLeafPreimage { value: 50, next_value: 60 };
        assert_check_non_membership(
            55,
            fake_leaf,
            MembershipWitness { leaf_index: 1, sibling_path: tree.get_sibling_path(1) },
            tree_root,
        );
    }

    #[test]
    fn test_conditionally_assert_check_membership_exists() {
        conditionally_assert_check_membership_at_index(true, 1, leaf_preimages[1].get_key());
    }

    #[test]
    fn test_conditionally_assert_check_membership_not_exists() {
        conditionally_assert_check_membership_at_index(false, 1, leaf_preimages[1].get_key() + 1);
    }

    #[test(should_fail_with = "Key does not match the key of the leaf preimage")]
    fn test_conditionally_assert_check_membership_exists_value_mismatch() {
        conditionally_assert_check_membership_at_index(true, 1, leaf_preimages[1].get_key() + 1);
    }

    #[test(should_fail_with = "Key is not greater than the low leaf")]
    fn test_conditionally_assert_check_membership_failed_not_exists_wrong_low_leaf() {
        conditionally_assert_check_membership_at_index(false, 3, 25);
    }

    #[test(should_fail_with = "Key is not less than the next leaf")]
    fn test_conditionally_assert_check_membership_failed_not_exists_wrong_next_key() {
        conditionally_assert_check_membership_at_index(false, 2, 25);
    }

    #[test(should_fail_with = "membership check failed")]
    fn test_conditionally_assert_check_membership_failed_exists_invalid_leaf() {
        let tree = build_tree();
        let tree_root = tree.get_root();
        let fake_leaf = TestLeafPreimage { value: 50, next_value: 60 };
        let exists = true;
        conditionally_assert_check_membership(
            50,
            exists,
            fake_leaf,
            MembershipWitness { leaf_index: 1, sibling_path: tree.get_sibling_path(1) },
            tree_root,
        );
    }

    #[test(should_fail_with = "membership check failed")]
    fn test_conditionally_assert_check_membership_failed_not_exists_invalid_leaf() {
        let tree = build_tree();
        let tree_root = tree.get_root();
        let fake_leaf = TestLeafPreimage { value: 50, next_value: 60 };
        let exists = false;
        conditionally_assert_check_membership(
            55,
            exists,
            fake_leaf,
            MembershipWitness { leaf_index: 1, sibling_path: tree.get_sibling_path(1) },
            tree_root,
        );
    }
}
