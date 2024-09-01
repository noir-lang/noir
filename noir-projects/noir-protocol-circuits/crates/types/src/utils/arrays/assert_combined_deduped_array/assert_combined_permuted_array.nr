use crate::{traits::{Empty, is_empty}, utils::arrays::array_length};

pub fn assert_combined_permuted_array<T, let N: u32>(
    original_array_lt: [T; N],
    original_array_gte: [T; N],
    permuted_array: [T; N],
    combined_indexes: [u32; N]
) where T: Empty + Eq {
    let num_lt = array_length(original_array_lt);
    let num_gte = array_length(original_array_gte);
    let total_num = num_lt + num_gte;

    let mut is_lt = true;
    let mut should_be_empty = false;
    for i in 0..N {
        is_lt &= i != num_lt;
        should_be_empty |= i == total_num;

        let from = if is_lt {
            original_array_lt[i]
        } else {
            original_array_gte[i - num_lt]
        };

        let combined_index = combined_indexes[i];
        let to = permuted_array[combined_index];
        assert_eq(from, to, "hinted item in the permuted array does not match");

        if should_be_empty {
            assert(is_empty(permuted_array[i]), "permuted array must be padded with empty items");
        }
    }
}

mod tests {
    use crate::utils::arrays::assert_combined_deduped_array::assert_combined_permuted_array::assert_combined_permuted_array;

    struct TestBuilder<let N: u32> {
        original_array_lt: [Field; N],
        original_array_gte: [Field; N],
        permuted_array: [Field; N],
        combined_indexes: [u32; N]
    }

    impl TestBuilder<7> {
        pub fn new() -> Self {
            let original_array_lt = [6, 3, 8, 1, 0, 0, 0];
            let original_array_gte = [4, 9, 5, 0, 0, 0, 0];
            let permuted_array = [5, 8, 9, 3, 1, 6, 4];
            let combined_indexes = [5, 3, 1, 4, 6, 2, 0];
            TestBuilder { original_array_lt, original_array_gte, permuted_array, combined_indexes }
        }
    }

    impl TestBuilder<10> {
        pub fn new_with_padded_zeros() -> Self {
            let original_array_lt = [6, 3, 8, 1, 0, 0, 0, 0, 0, 0];
            let original_array_gte = [4, 9, 5, 0, 0, 0, 0, 0, 0, 0];
            let permuted_array = [5, 8, 9, 3, 1, 6, 4, 0, 0, 0];
            let combined_indexes = [5, 3, 1, 4, 6, 2, 0, 7, 8, 9];
            TestBuilder { original_array_lt, original_array_gte, permuted_array, combined_indexes }
        }
    }

    impl<let N: u32> TestBuilder<N> {
        pub fn verify(self) {
            assert_combined_permuted_array(
                self.original_array_lt,
                self.original_array_gte,
                self.permuted_array,
                self.combined_indexes
            );
        }
    }

    #[test]
    fn assert_combined_permuted_array_full() {
        let builder = TestBuilder::new();
        builder.verify()
    }

    #[test]
    fn assert_combined_permuted_array_padded_empty() {
        let builder = TestBuilder::new_with_padded_zeros();
        builder.verify()
    }

    #[test(should_fail_with="hinted item in the permuted array does not match")]
    fn assert_combined_permuted_array_missing_value_fails() {
        let mut builder = TestBuilder::new_with_padded_zeros();

        // Clear a value.
        builder.permuted_array[6] = 0;

        builder.verify()
    }

    #[test(should_fail_with="hinted item in the permuted array does not match")]
    fn assert_combined_permuted_array_duplicated_item_fails() {
        let mut builder = TestBuilder::new_with_padded_zeros();

        // Duplicate a value.
        builder.permuted_array[3] = builder.permuted_array[2];

        builder.verify()
    }

    #[test(should_fail_with="permuted array must be padded with empty items")]
    fn assert_combined_permuted_array_extra_item_fails() {
        let mut builder = TestBuilder::new_with_padded_zeros();

        // Overwrite the first empty value with a non-empty value.
        builder.permuted_array[8] = builder.permuted_array[1];
        // Update the index to point to an empty value.
        builder.combined_indexes[8] = 9;

        builder.verify()
    }

    #[test(should_fail_with="permuted array must be padded with empty items")]
    fn assert_combined_permuted_array_mixed_empty_item_fails() {
        let mut builder = TestBuilder::new_with_padded_zeros();

        // Swap the positions of a non-empty item and an empty item.
        builder.permuted_array[8] = builder.permuted_array[4];
        builder.permuted_array[4] = 0;
        // Update the index to point to the value.
        let original_index = 3;
        assert_eq(builder.original_array_lt[original_index], builder.permuted_array[8]);
        builder.combined_indexes[original_index] = 8;
        builder.combined_indexes[8] = 4;

        builder.verify()
    }
}
