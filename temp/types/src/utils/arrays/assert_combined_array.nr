use crate::{
    traits::Empty,
    utils::arrays::assert_combined_transformed_array::{
        assert_combined_transformed_array, combine_and_transform_arrays,
    },
};

// original_array(_lt/_gte) must be valid, i.e. validate_array(original_array) == true
// This ensures that combined_array is valid when S can only be empty if and only if T is empty.
pub fn assert_combined_array<T, let N: u32>(
    original_array_lt: [T; N],
    original_array_gte: [T; N],
    combined_array: [T; N],
)
where
    T: Empty + Eq,
{
    assert_combined_transformed_array(
        original_array_lt,
        original_array_gte,
        combined_array,
        |from: T, to: T| from == to,
    )
}

pub unconstrained fn combine_arrays<T, let N: u32>(
    original_array_lt: [T; N],
    original_array_gte: [T; N],
) -> [T; N]
where
    T: Empty + Eq,
{
    combine_and_transform_arrays(original_array_lt, original_array_gte, |from: T| from)
}

mod tests {
    use crate::{
        tests::{types::TestValue, utils::pad_end},
        traits::Empty,
        utils::arrays::assert_combined_array::{assert_combined_array, combine_arrays},
    };

    struct TestBuilder<let N: u32> {
        original_array_lt: [TestValue; N],
        original_array_gte: [TestValue; N],
        combined_array: [TestValue; N],
    }

    impl TestBuilder<10> {
        pub fn new_empty() -> Self {
            let original_array_lt = pad_end([], TestValue::empty());
            let original_array_gte = pad_end([], TestValue::empty());
            let combined_array = pad_end([], TestValue::empty());
            TestBuilder { original_array_lt, original_array_gte, combined_array }
        }

        pub fn new() -> Self {
            let original_array_lt = pad_end(
                [
                    TestValue { value: 11, counter: 2 },
                    TestValue { value: 22, counter: 5 },
                    TestValue { value: 33, counter: 3 },
                ],
                TestValue::empty(),
            );

            let original_array_gte = pad_end(
                [TestValue { value: 44, counter: 1 }, TestValue { value: 55, counter: 4 }],
                TestValue::empty(),
            );

            let combined_array = pad_end(
                [
                    TestValue { value: 11, counter: 2 },
                    TestValue { value: 22, counter: 5 },
                    TestValue { value: 33, counter: 3 },
                    TestValue { value: 44, counter: 1 },
                    TestValue { value: 55, counter: 4 },
                ],
                TestValue::empty(),
            );

            TestBuilder { original_array_lt, original_array_gte, combined_array }
        }

        pub fn execute(self) {
            assert_combined_array(
                self.original_array_lt,
                self.original_array_gte,
                self.combined_array,
            );
        }

        pub unconstrained fn check_and_execute(self) {
            let combined = combine_arrays(self.original_array_lt, self.original_array_gte);
            assert_eq(combined, self.combined_array);

            self.execute();
        }
    }

    #[test]
    unconstrained fn assert_combined_array_empty_succeeds() {
        let builder = TestBuilder::new_empty();
        builder.check_and_execute();
    }

    #[test]
    unconstrained fn assert_combined_array_succeeds() {
        let builder = TestBuilder::new();
        builder.check_and_execute();
    }

    #[test(should_fail_with = "hinted item in the combined array does not match")]
    fn assert_combined_array_extra_item_fails() {
        let mut builder = TestBuilder::new();

        // Add random value to an empty item.
        builder.combined_array[7].value = 123;

        builder.execute();
    }

    #[test(should_fail_with = "hinted item in the combined array does not match")]
    fn assert_combined_array_missing_item_fails() {
        let mut builder = TestBuilder::new();

        // Clear the last item.
        builder.combined_array[4] = TestValue::empty();

        builder.execute();
    }

    #[test(should_fail_with = "hinted item in the combined array does not match")]
    fn assert_combined_array_unordered_fails() {
        let mut builder = TestBuilder::new();

        // Swap the two items.
        let tmp = builder.combined_array[3];
        builder.combined_array[3] = builder.combined_array[1];
        builder.combined_array[1] = tmp;

        builder.execute();
    }
}
