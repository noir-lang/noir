pub mod get_split_order_hints;

use crate::{
    abis::side_effect::Ordered,
    traits::Empty,
    utils::arrays::{
        array_length,
        assert_split_sorted_transformed_value_arrays::get_split_order_hints::SplitOrderHints,
        validate_array,
    },
};

// original_array must be valid, i.e. validate_array(original_array) == true
// transformed_value_array must be verified against original_array before calling this function.
fn assert_split_sorted_transformed_value_arrays<T, S, let N: u32>(
    original_array: [T; N],
    transformed_value_array: [S; N],
    split_counter: u32,
    sorted_transformed_value_array_lt: [S; N], // Values whose counters are less than the split counter.
    sorted_transformed_value_array_gte: [S; N], // Values whose counters are greater than or equal to the split counter.
    sorted_counters_lt: [u32; N], // Counters of the values in sorted_transformed_value_array_lt.
    sorted_counters_gte: [u32; N], // Counters of the values in sorted_transformed_value_array_gte.
    index_hints: [u32; N], // The index of the item in the correspinding sorted_transformed_value_array_(lt/gte) for each item in the original_array.
    ascending: bool, // Whether the items in sorted_transformed_value_array_(lt/gte) is in ascending order.
)
where
    T: Ordered + Empty + Eq,
    S: Empty + Eq,
{
    // Can use array_length instead of validate_array for the original_array because it's taken from the previous kernel and guaranteed to be valid.
    let total_num = array_length(original_array);

    let mut num_lt = 0;
    let mut num_gte = 0;
    let mut should_check = true;
    for i in 0..N {
        should_check &= i != total_num;
        if should_check {
            let original = original_array[i];
            let value = transformed_value_array[i];
            let sorted_index = index_hints[i];
            let is_lt = original.counter() < split_counter;
            let (sorted_value, sorted_counter) = if is_lt {
                (sorted_transformed_value_array_lt[sorted_index], sorted_counters_lt[sorted_index])
            } else {
                (
                    sorted_transformed_value_array_gte[sorted_index],
                    sorted_counters_gte[sorted_index],
                )
            };
            assert_eq(value, sorted_value, "mismatch sorted values");
            assert_eq(original.counter(), sorted_counter, "mismatch counters");
            if is_lt {
                num_lt += 1;
            } else {
                num_gte += 1;
            }
        }
    }

    for i in 1..N {
        if i < num_lt {
            let counter = sorted_counters_lt[i];
            let prev_counter = sorted_counters_lt[i - 1];
            assert(ascending == (counter > prev_counter), "value array must be sorted by counter");
            assert(counter != prev_counter, "counters must not be the same");
        }
        if i < num_gte {
            let counter = sorted_counters_gte[i];
            let prev_counter = sorted_counters_gte[i - 1];
            assert(ascending == (counter > prev_counter), "value array must be sorted by counter");
            assert(counter != prev_counter, "counters must not be the same");
        }
    }

    let num_non_empty_values_lt = validate_array(sorted_transformed_value_array_lt);
    assert_eq(num_non_empty_values_lt, num_lt, "mismatch number of values lt");

    let num_non_empty_values_gte = validate_array(sorted_transformed_value_array_gte);
    assert_eq(num_non_empty_values_gte, num_gte, "mismatch number of values gte");
}

pub fn assert_split_sorted_transformed_value_arrays_asc<T, S, let N: u32>(
    original_array: [T; N],
    transformed_value_array: [S; N],
    split_counter: u32,
    sorted_transformed_value_array_lt: [S; N],
    sorted_transformed_value_array_gte: [S; N],
    hints: SplitOrderHints<N>,
)
where
    T: Ordered + Empty + Eq,
    S: Empty + Eq,
{
    assert_split_sorted_transformed_value_arrays(
        original_array,
        transformed_value_array,
        split_counter,
        sorted_transformed_value_array_lt,
        sorted_transformed_value_array_gte,
        hints.sorted_counters_lt,
        hints.sorted_counters_gte,
        hints.sorted_indexes,
        true,
    );
}

pub fn assert_split_sorted_transformed_value_arrays_desc<T, S, let N: u32>(
    original_array: [T; N],
    transformed_value_array: [S; N],
    split_counter: u32,
    sorted_transformed_value_array_lt: [S; N],
    sorted_transformed_value_array_gte: [S; N],
    hints: SplitOrderHints<N>,
)
where
    T: Ordered + Empty + Eq,
    S: Empty + Eq,
{
    assert_split_sorted_transformed_value_arrays(
        original_array,
        transformed_value_array,
        split_counter,
        sorted_transformed_value_array_lt,
        sorted_transformed_value_array_gte,
        hints.sorted_counters_lt,
        hints.sorted_counters_gte,
        hints.sorted_indexes,
        false,
    );
}

mod tests {
    use crate::{
        tests::types::{combine_two_values, TestCombinedValue, TestTwoValues},
        traits::Empty,
        utils::arrays::assert_split_sorted_transformed_value_arrays::{
            assert_split_sorted_transformed_value_arrays_asc,
            assert_split_sorted_transformed_value_arrays_desc,
            get_split_order_hints::SplitOrderHints,
        },
    };

    global original_array: [TestTwoValues; 8] = [
        TestTwoValues { value_1: 1, value_2: 0, counter: 33 },
        TestTwoValues { value_1: 10, value_2: 6, counter: 44 },
        TestTwoValues { value_1: 20, value_2: 7, counter: 11 },
        TestTwoValues { value_1: 30, value_2: 8, counter: 0 },
        TestTwoValues { value_1: 40, value_2: 9, counter: 22 },
        TestTwoValues::empty(),
        TestTwoValues::empty(),
        TestTwoValues::empty(),
    ];

    struct TestDataBuilder<let N: u32> {
        original_array: [TestTwoValues; N],
        transformed_value_array: [TestCombinedValue; N],
        sorted_transformed_value_array_lt: [TestCombinedValue; N],
        sorted_transformed_value_array_gte: [TestCombinedValue; N],
        split_counter: u32,
        hints: SplitOrderHints<N>,
        ascending: bool,
    }

    impl TestDataBuilder<8> {
        pub fn empty() -> Self {
            TestDataBuilder {
                original_array: [TestTwoValues::empty(); 8],
                transformed_value_array: [TestCombinedValue::empty(); 8],
                sorted_transformed_value_array_lt: [TestCombinedValue::empty(); 8],
                sorted_transformed_value_array_gte: [TestCombinedValue::empty(); 8],
                split_counter: 0,
                hints: SplitOrderHints::empty(),
                ascending: false,
            }
        }

        pub fn new() -> Self {
            let transformed_value_array =
                original_array.map(|item: TestTwoValues| combine_two_values(item));

            let split_counter = 15;
            let sorted_transformed_value_array_lt = [
                TestCombinedValue { value: 38 },
                TestCombinedValue { value: 27 },
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
            ];
            let sorted_transformed_value_array_gte = [
                TestCombinedValue { value: 49 },
                TestCombinedValue { value: 1 },
                TestCombinedValue { value: 16 },
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
            ];
            let hints = SplitOrderHints {
                sorted_counters_lt: [0, 11, 0, 0, 0, 0, 0, 0],
                sorted_counters_gte: [22, 33, 44, 0, 0, 0, 0, 0],
                sorted_indexes: [1, 2, 1, 0, 0, 0, 0, 0],
            };

            TestDataBuilder {
                original_array,
                transformed_value_array,
                sorted_transformed_value_array_lt,
                sorted_transformed_value_array_gte,
                split_counter,
                hints,
                ascending: true,
            }
        }

        pub fn new_desc() -> Self {
            let transformed_value_array =
                original_array.map(|item: TestTwoValues| combine_two_values(item));

            let split_counter = 15;
            let sorted_transformed_value_array_lt = [
                TestCombinedValue { value: 27 },
                TestCombinedValue { value: 38 },
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
            ];
            let sorted_transformed_value_array_gte = [
                TestCombinedValue { value: 16 },
                TestCombinedValue { value: 1 },
                TestCombinedValue { value: 49 },
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
                TestCombinedValue::empty(),
            ];
            let hints = SplitOrderHints {
                sorted_counters_lt: [11, 0, 0, 0, 0, 0, 0, 0],
                sorted_counters_gte: [44, 33, 22, 0, 0, 0, 0, 0],
                sorted_indexes: [1, 0, 0, 1, 2, 0, 0, 0],
            };

            TestDataBuilder {
                original_array,
                transformed_value_array,
                sorted_transformed_value_array_lt,
                sorted_transformed_value_array_gte,
                split_counter,
                hints,
                ascending: false,
            }
        }

        pub fn update_sorted_index(&mut self, counter: u32, new_index: u32) {
            let mut original_index = original_array.len();
            for i in 0..original_array.len() {
                if (original_index == original_array.len()) & (original_array[i].counter == counter)
                     {
                        original_index = i;
                    }
                }
                self.hints.sorted_indexes[original_index] = new_index;
            }

        pub fn execute(self) {
            if self.ascending {
                assert_split_sorted_transformed_value_arrays_asc(
                    self.original_array,
                    self.transformed_value_array,
                    self.split_counter,
                    self.sorted_transformed_value_array_lt,
                    self.sorted_transformed_value_array_gte,
                    self.hints,
                );
            } else {
                assert_split_sorted_transformed_value_arrays_desc(
                    self.original_array,
                    self.transformed_value_array,
                    self.split_counter,
                    self.sorted_transformed_value_array_lt,
                    self.sorted_transformed_value_array_gte,
                    self.hints,
                );
            }
        }
    }

    /**
     * asc
     */
    #[test]
    fn assert_split_sorted_transformed_value_array_asc_succeeds() {
        let builder = TestDataBuilder::new();
        builder.execute();
    }

    #[test]
    fn assert_split_sorted_transformed_value_array_asc_empty_succeeds() {
        let builder = TestDataBuilder::empty();
        builder.execute();
    }

    /**
     * desc
     */
    #[test]
    fn assert_split_sorted_transformed_value_array_desc_succeeds() {
        let builder = TestDataBuilder::new_desc();
        builder.execute();
    }

    /**
     * Failed cases.
     */
    #[test(should_fail_with = "mismatch number of values lt")]
    fn assert_split_sorted_transformed_value_array_asc_empty_extra_item_fails() {
        let mut builder = TestDataBuilder::empty();

        builder.sorted_transformed_value_array_lt[0].value = 1;

        builder.execute();
    }

    #[test(should_fail_with = "mismatch sorted values")]
    fn assert_split_sorted_transformed_value_array_asc_lt_wrong_sorted_value_fails() {
        let mut builder = TestDataBuilder::new();

        builder.sorted_transformed_value_array_lt[0].value += 1;

        builder.execute();
    }

    #[test(should_fail_with = "mismatch sorted values")]
    fn assert_split_sorted_transformed_value_array_asc_gte_wrong_sorted_value_fails() {
        let mut builder = TestDataBuilder::new();

        // Swap two values in the sorted array.
        let tmp = builder.sorted_transformed_value_array_gte[0];
        builder.sorted_transformed_value_array_gte[0] =
            builder.sorted_transformed_value_array_gte[1];
        builder.sorted_transformed_value_array_gte[1] = tmp;

        builder.execute();
    }

    #[test(should_fail_with = "value array must be sorted by counter")]
    fn assert_split_sorted_transformed_value_array_asc_gte_wrong_sorted_order_fails() {
        let mut builder = TestDataBuilder::new();

        // Swap two values in the sorted array, also update their hints.
        let tmp = builder.sorted_transformed_value_array_gte[0];
        builder.sorted_transformed_value_array_gte[0] =
            builder.sorted_transformed_value_array_gte[1];
        builder.sorted_transformed_value_array_gte[1] = tmp;
        builder.hints.sorted_counters_gte[0] = 33;
        builder.hints.sorted_counters_gte[1] = 22;
        builder.update_sorted_index(33, 0); // Item of counter 33 is now at index 0 of the sorted array.
        builder.update_sorted_index(22, 1); // Item of counter 22 is now at index 1 of the sorted array.
        builder.execute();
    }

    #[test(should_fail_with = "mismatch counters")]
    fn assert_split_sorted_transformed_value_array_asc_gte_wrong_counter_fails() {
        let mut builder = TestDataBuilder::new();

        // Swap two values in the sorted array, but keep their counters in the correct order.
        let tmp = builder.sorted_transformed_value_array_gte[0];
        builder.sorted_transformed_value_array_gte[0] =
            builder.sorted_transformed_value_array_gte[1];
        builder.sorted_transformed_value_array_gte[1] = tmp;
        builder.update_sorted_index(33, 0); // Item of counter 33 is now at index 0 of the sorted array.
        builder.update_sorted_index(22, 1); // Item of counter 22 is now at index 1 of the sorted array.
        builder.execute();
    }

    #[test(should_fail_with = "mismatch sorted values")]
    fn assert_split_sorted_transformed_value_array_asc_misplace_lt_to_gte_fails() {
        let mut builder = TestDataBuilder::new();

        // Move the item with counter 44 from _gte to _lt.
        builder.sorted_transformed_value_array_lt[2] =
            builder.sorted_transformed_value_array_gte[2];
        builder.hints.sorted_counters_lt[2] = 44;
        builder.sorted_transformed_value_array_gte[2] = TestCombinedValue::empty();
        builder.hints.sorted_counters_lt[2] = 0;
        builder.update_sorted_index(44, 2); // Item of counter 44 is now at index 2 of the sorted array.
        builder.execute();
    }

    #[test(should_fail_with = "mismatch number of values lt")]
    fn assert_split_sorted_transformed_value_array_asc_duplicate_lt_to_gte_fails() {
        let mut builder = TestDataBuilder::new();

        // Copy the item with counter 44 to _lt.
        builder.sorted_transformed_value_array_lt[2] =
            builder.sorted_transformed_value_array_gte[2];
        builder.hints.sorted_counters_lt[2] = 44;

        builder.execute();
    }

    #[test(should_fail_with = "mismatch number of values gte")]
    fn assert_split_sorted_transformed_value_array_asc_gte_duplicate_items_fails() {
        let mut builder = TestDataBuilder::new();

        // Duplicate the item with counter 44.
        builder.sorted_transformed_value_array_gte[3] =
            builder.sorted_transformed_value_array_gte[2];
        builder.hints.sorted_counters_gte[3] = 44;

        builder.execute();
    }

    #[test(should_fail_with = "value array must be sorted by counter")]
    fn assert_split_sorted_transformed_value_array_asc_multiple_zero_counter_items_fails() {
        let mut builder = TestDataBuilder::new();

        // Change the counter of the item from 11 to 0.
        builder.original_array[2].counter = 0;
        builder.hints.sorted_counters_lt[1] = 0;

        builder.execute();
    }

    #[test(should_fail_with = "counters must not be the same")]
    fn assert_split_sorted_transformed_value_array_desc_multiple_zero_counter_items_fails() {
        let mut builder = TestDataBuilder::new_desc();

        // Change the counter of the item from 11 to 0.
        builder.original_array[2].counter = 0;
        builder.hints.sorted_counters_lt[0] = 0;

        builder.execute();
    }

    #[test(should_fail_with = "value array must be sorted by counter")]
    fn assert_split_sorted_transformed_value_array_asc_lt_empty_item_at_index_0_fails() {
        let mut builder = TestDataBuilder::new();

        // Move the item at index i to index i + 1.
        let num_items = 2;
        for i in 0..num_items {
            let from_index = num_items - 1 - i;
            let to_index = num_items - i;
            builder.sorted_transformed_value_array_lt[to_index] =
                builder.sorted_transformed_value_array_lt[from_index];
            let counter = builder.hints.sorted_counters_lt[from_index];
            builder.hints.sorted_counters_lt[to_index] = counter;
            builder.update_sorted_index(counter, to_index);
        }
        // Empty the values at index 0.
        builder.sorted_transformed_value_array_lt[0] = TestCombinedValue::empty();
        builder.hints.sorted_counters_lt[0] = 0;

        builder.execute();
    }

    #[test(should_fail_with = "invalid array")]
    fn assert_split_sorted_transformed_value_array_asc_gte_empty_item_at_index_0_fails() {
        let mut builder = TestDataBuilder::new();

        let num_items = 3;
        for i in 0..num_items {
            let from_index = num_items - 1 - i;
            let to_index = num_items - i;
            builder.sorted_transformed_value_array_gte[to_index] =
                builder.sorted_transformed_value_array_gte[from_index];
            let counter = builder.hints.sorted_counters_gte[from_index];
            builder.hints.sorted_counters_gte[to_index] = counter;
            builder.update_sorted_index(counter, to_index);
        }
        // Empty the values at index 0.
        builder.sorted_transformed_value_array_gte[0] = TestCombinedValue::empty();
        builder.hints.sorted_counters_gte[0] = 0;

        builder.execute();
    }
}
