use crate::{traits::Empty, utils::arrays::array_length};

// original_array(_lt/_gte) must be valid, i.e. validate_array(original_array) == true
// This ensures that combined_array is valid when S can only be empty if and only if T is empty.
pub fn assert_combined_transformed_array<T, S, let N: u32, Env>(
    original_array_lt: [T; N],
    original_array_gte: [T; N],
    combined_array: [S; N],
    is_transformed: fn[Env](T, S) -> bool,
)
where
    T: Empty + Eq,
{
    let num_lt = array_length(original_array_lt);
    let mut is_lt = true;
    for i in 0..N {
        is_lt &= i != num_lt;

        let from = if is_lt {
            original_array_lt[i]
        } else {
            original_array_gte[i - num_lt]
        };

        let to = combined_array[i];

        assert(is_transformed(from, to), "hinted item in the combined array does not match");
    }
}

pub unconstrained fn combine_and_transform_arrays<T, S, let N: u32, Env>(
    original_array_lt: [T; N],
    original_array_gte: [T; N],
    transform: fn[Env](T) -> S,
) -> [S; N]
where
    T: Empty + Eq,
{
    let mut combined = original_array_lt.map(transform);

    let num_lt = array_length(original_array_lt);
    for i in 0..N {
        if i >= num_lt {
            let from = original_array_gte[i - num_lt];
            combined[i] = transform(from);
        }
    }

    combined
}

mod tests {
    use crate::{
        tests::{
            types::{is_summed_from_two_values, sum_two_values, TestTwoValues, TestValue},
            utils::pad_end,
        },
        traits::Empty,
        utils::arrays::assert_combined_transformed_array::{
            assert_combined_transformed_array, combine_and_transform_arrays,
        },
    };

    struct TestBuilder<let N: u32> {
        original_array_lt: [TestTwoValues; N],
        original_array_gte: [TestTwoValues; N],
        combined_array: [TestValue; N],
    }

    impl TestBuilder<10> {
        pub fn new_empty() -> Self {
            let original_array_lt = pad_end([], TestTwoValues::empty());
            let original_array_gte = pad_end([], TestTwoValues::empty());
            let combined_array = pad_end([], TestValue::empty());
            TestBuilder { original_array_lt, original_array_gte, combined_array }
        }

        pub fn new() -> Self {
            let original_array_lt = pad_end(
                [
                    TestTwoValues { value_1: 10, value_2: 1, counter: 2 },
                    TestTwoValues { value_1: 20, value_2: 2, counter: 5 },
                    TestTwoValues { value_1: 30, value_2: 3, counter: 3 },
                ],
                TestTwoValues::empty(),
            );

            let original_array_gte = pad_end(
                [
                    TestTwoValues { value_1: 40, value_2: 4, counter: 1 },
                    TestTwoValues { value_1: 50, value_2: 5, counter: 4 },
                ],
                TestTwoValues::empty(),
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
            assert_combined_transformed_array(
                self.original_array_lt,
                self.original_array_gte,
                self.combined_array,
                is_summed_from_two_values,
            );
        }

        pub unconstrained fn check_and_execute(self) {
            let combined = combine_and_transform_arrays(
                self.original_array_lt,
                self.original_array_gte,
                sum_two_values,
            );
            assert_eq(combined, self.combined_array);

            self.execute();
        }
    }

    #[test]
    unconstrained fn assert_combined_transformed_array_empty_succeeds() {
        let builder = TestBuilder::new_empty();
        builder.check_and_execute();
    }

    #[test]
    unconstrained fn assert_combined_transformed_array_succeeds() {
        let builder = TestBuilder::new();
        builder.check_and_execute();
    }

    #[test(should_fail_with = "hinted item in the combined array does not match")]
    unconstrained fn assert_combined_transformed_array_extra_item_fails() {
        let mut builder = TestBuilder::new();

        // Add random value to an empty item.
        builder.combined_array[7].value = 123;

        builder.execute();
    }

    #[test(should_fail_with = "hinted item in the combined array does not match")]
    unconstrained fn assert_combined_transformed_array_missing_item_fails() {
        let mut builder = TestBuilder::new();

        // Clear the last item.
        builder.combined_array[4] = TestValue::empty();

        builder.execute();
    }

    #[test(should_fail_with = "hinted item in the combined array does not match")]
    unconstrained fn assert_combined_transformed_array_unordered_fails() {
        let mut builder = TestBuilder::new();

        // Swap the two items.
        let tmp = builder.combined_array[3];
        builder.combined_array[3] = builder.combined_array[1];
        builder.combined_array[1] = tmp;

        builder.execute();
    }
}
