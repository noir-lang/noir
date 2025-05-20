use crate::merkle_tree::leaf_preimage::IndexedTreeLeafPreimage;

pub fn assert_check_valid_low_leaf<LEAF_PREIMAGE, VALUE>(
    key: Field,
    low_leaf_preimage: LEAF_PREIMAGE,
)
where
    LEAF_PREIMAGE: IndexedTreeLeafPreimage<VALUE>,
{
    let low_key = low_leaf_preimage.get_key();
    let next_key = low_leaf_preimage.get_next_key();

    assert(low_key.lt(key), "Key is not greater than the low leaf");
    assert(key.lt(next_key) | (next_key == 0), "Key is not less than the next leaf");
}

mod tests {
    use crate::merkle_tree::{
        indexed_tree::check_valid_low_leaf::assert_check_valid_low_leaf,
        leaf_preimage::IndexedTreeLeafPreimage,
    };
    use crate::merkle_tree::leaf_preimage::LeafPreimage;
    use crate::traits::Empty;

    struct TestLeafPreimage {
        value: Field,
        next_value: Field,
    }

    impl Empty for TestLeafPreimage {
        fn empty() -> Self {
            Self { value: 0, next_value: 0 }
        }
    }

    impl Eq for TestLeafPreimage {
        fn eq(self, other: Self) -> bool {
            (self.value == other.value) & (self.next_value == other.next_value)
        }
    }

    impl LeafPreimage for TestLeafPreimage {
        fn get_key(self) -> Field {
            self.value
        }

        fn as_leaf(self) -> Field {
            self.value
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

    #[test]
    fn test_assert_check_valid_low_leaf() {
        let key = 12;
        let leaf = TestLeafPreimage { value: 11, next_value: 13 };
        assert_check_valid_low_leaf(key, leaf);
    }

    #[test]
    fn test_assert_check_empty_low_leaf() {
        // An all-zero low leaf should be valid. It could be used as the first dummy leaf in a tree.
        // It's not possible to prove against an empty leaf at an uninitialized index.
        // The membership check will fail because the leaf value hash(0, 0) is not 0.
        let key = 12;
        let leaf = TestLeafPreimage { value: 0, next_value: 0 };
        assert_check_valid_low_leaf(key, leaf);
    }

    #[test(should_fail_with = "Key is not greater than the low leaf")]
    fn test_assert_check_valid_low_leaf_failed_wrong_low_leaf() {
        let key = 12;
        let leaf = TestLeafPreimage { value: 13, next_value: 15 };
        assert_check_valid_low_leaf(key, leaf);
    }

    #[test(should_fail_with = "Key is not greater than the low leaf")]
    fn test_assert_check_valid_low_leaf_failed_is_low_leaf() {
        let key = 12;
        let leaf = TestLeafPreimage { value: 12, next_value: 15 };
        assert_check_valid_low_leaf(key, leaf);
    }

    #[test(should_fail_with = "Key is not less than the next leaf")]
    fn test_assert_check_valid_low_leaf_failed_wrong_next_key() {
        let key = 12;
        let leaf = TestLeafPreimage { value: 9, next_value: 11 };
        assert_check_valid_low_leaf(key, leaf);
    }
}
