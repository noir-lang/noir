pub mod get_order_hints;

use crate::{
    abis::side_effect::Ordered,
    traits::{Empty, is_empty},
    utils::arrays::{
        array_length, assert_exposed_sorted_transformed_value_array::get_order_hints::OrderHint,
    },
};

// original_array must be valid, i.e. validate_array(original_array) == true
// Items in exposed_sorted_transformed_value_array do not have counters. Their corresponding counters are in hints.
pub fn assert_exposed_sorted_transformed_value_array<T, S, let N: u32, Env>(
    original_array: [T; N],
    exposed_sorted_transformed_value_array: [S; N],
    is_transformed: fn[Env](T, S) -> bool,
    hints: [OrderHint; N],
)
where
    T: Ordered + Empty + Eq,
    S: Empty + Eq,
{
    let num_non_empty_items = array_length(original_array);
    let mut should_be_empty = false;
    for i in 0..N {
        should_be_empty |= i == num_non_empty_items;
        if should_be_empty {
            assert(
                is_empty(exposed_sorted_transformed_value_array[i]),
                "array must be padded with empty items",
            );
        } else {
            let original = original_array[i];
            let sorted_index = hints[i].sorted_index;
            let sorted = exposed_sorted_transformed_value_array[sorted_index];
            assert(is_transformed(original, sorted), "incorrect transformed value");
            assert_eq(original.counter(), hints[sorted_index].counter, "incorrect hinted counter");
            if i != 0 {
                assert(
                    hints[i].counter > hints[i - 1].counter,
                    "value array must be sorted by counter in ascending order",
                );
            }
        }
    }
}

mod tests {
    use crate::{
        tests::types::{is_combined_from_two_values, TestCombinedValue, TestTwoValues},
        traits::Empty,
        utils::arrays::assert_exposed_sorted_transformed_value_array::{
            assert_exposed_sorted_transformed_value_array, get_order_hints::OrderHint,
        },
    };

    struct TestDataBuilder<let N: u32> {
        original_array: [TestTwoValues; N],
        exposed_sorted_transformed_value_array: [TestCombinedValue; N],
        hints: [OrderHint; N],
    }

    impl TestDataBuilder<6> {
        pub fn new() -> Self {
            let original_array = [
                TestTwoValues { value_1: 10, value_2: 5, counter: 44 },
                TestTwoValues { value_1: 20, value_2: 6, counter: 22 },
                TestTwoValues { value_1: 30, value_2: 7, counter: 11 },
                TestTwoValues { value_1: 40, value_2: 8, counter: 33 },
                TestTwoValues::empty(),
                TestTwoValues::empty(),
            ];

            let exposed_sorted_transformed_value_array = [
                TestCombinedValue { value: 37 },
                TestCombinedValue { value: 26 },
                TestCombinedValue { value: 48 },
                TestCombinedValue { value: 15 },
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
            ];

            let hints = [
                OrderHint { counter: 11, sorted_index: 3 },
                OrderHint { counter: 22, sorted_index: 1 },
                OrderHint { counter: 33, sorted_index: 0 },
                OrderHint { counter: 44, sorted_index: 2 },
                OrderHint { counter: 0, sorted_index: 4 },
                OrderHint { counter: 0, sorted_index: 5 },
            ];

            TestDataBuilder { original_array, exposed_sorted_transformed_value_array, hints }
        }

        pub fn execute(self) {
            assert_exposed_sorted_transformed_value_array(
                self.original_array,
                self.exposed_sorted_transformed_value_array,
                is_combined_from_two_values,
                self.hints,
            );
        }
    }

    #[test]
    fn assert_exposed_sorted_transformed_value_array_succeeds() {
        let builder = TestDataBuilder::new();
        builder.execute();
    }

    #[test(should_fail_with = "incorrect transformed value")]
    fn assert_exposed_sorted_transformed_value_array_mismatch_value_fails() {
        let mut builder = TestDataBuilder::new();

        // Tweak the value at index 1.
        builder.exposed_sorted_transformed_value_array[1].value += 1;

        builder.execute();
    }

    #[test(should_fail_with = "value array must be sorted by counter in ascending order")]
    fn assert_exposed_sorted_transformed_value_array_unordered_fails() {
        let mut builder = TestDataBuilder::new();

        // Swap the values at index 1 and 2.
        let tmp = builder.exposed_sorted_transformed_value_array[1];
        builder.exposed_sorted_transformed_value_array[1] =
            builder.exposed_sorted_transformed_value_array[2];
        builder.exposed_sorted_transformed_value_array[2] = tmp;

        // Update counters in hints.
        let tmp = builder.hints[1].counter;
        builder.hints[1].counter = builder.hints[2].counter;
        builder.hints[2].counter = tmp;

        // Update sorted indexes.
        // Original: 44, 22, 11, 33
        // New: 11, 33, 22, 44
        builder.hints[0].sorted_index = 3;
        builder.hints[1].sorted_index = 2;
        builder.hints[2].sorted_index = 0;
        builder.hints[3].sorted_index = 1;

        builder.execute();
    }

    #[test(should_fail_with = "incorrect hinted counter")]
    fn assert_exposed_sorted_transformed_value_array_unordered_values_with_ordered_counters_fails() {
        let mut builder = TestDataBuilder::new();

        // Swap the values at index 1 and 2.
        let tmp = builder.exposed_sorted_transformed_value_array[1];
        builder.exposed_sorted_transformed_value_array[1] =
            builder.exposed_sorted_transformed_value_array[2];
        builder.exposed_sorted_transformed_value_array[2] = tmp;

        // Update sorted indexes.
        // Original: 44, 22, 11, 33
        // New: 11, 33, 22, 44
        builder.hints[0].sorted_index = 3;
        builder.hints[1].sorted_index = 2;
        builder.hints[2].sorted_index = 0;
        builder.hints[3].sorted_index = 1;

        builder.execute();
    }

    #[test(should_fail_with = "array must be padded with empty items")]
    fn assert_exposed_sorted_transformed_value_array_extra_non_empty_fails() {
        let mut builder = TestDataBuilder::new();

        // Add a random item.
        builder.exposed_sorted_transformed_value_array[4] = TestCombinedValue { value: 10 };

        builder.execute();
    }

    #[test(should_fail_with = "array must be padded with empty items")]
    fn assert_exposed_sorted_transformed_value_array_hint_to_extra_non_empty_fails() {
        let mut builder = TestDataBuilder::new();

        // Add a random item.
        builder.exposed_sorted_transformed_value_array[4] = TestCombinedValue { value: 10 };
        // Change the hint to point to an empty item.
        builder.hints[4].sorted_index = 5;

        builder.execute();
    }

    #[test(should_fail_with = "incorrect transformed value")]
    fn assert_exposed_sorted_transformed_value_array_missing_item_fails() {
        let mut builder = TestDataBuilder::new();

        // Remove an item.
        builder.exposed_sorted_transformed_value_array[3] = TestCombinedValue::empty();

        builder.execute();
    }
}
