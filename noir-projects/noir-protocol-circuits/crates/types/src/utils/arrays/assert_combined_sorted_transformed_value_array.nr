mod get_combined_order_hints;

use crate::{
    abis::side_effect::Ordered, traits::{Empty, is_empty},
    utils::arrays::{
    array_length,
    assert_combined_sorted_transformed_value_array::get_combined_order_hints::{CombinedOrderHint, count_private_items}
}
};

fn get_num_private_items<T, let N: u32>(array: [T; N]) -> u32 where T: Ordered + Empty + Eq {
    let length = count_private_items(array);
    if length != 0 {
        let last_private_item = array[length - 1];
        assert(!is_empty(last_private_item) & (last_private_item.counter() == 0));
    }
    if length != N {
        let first_non_private_item = array[length];
        assert(is_empty(first_non_private_item) | (first_non_private_item.counter() != 0));
    }
    length
}

// original_array(_lt/_gte) must be valid, i.e. validate_array(original_array) == true
// Both arrays come from the previous kernel, so they are guaranteed to be valid, and meet the following conditions:
//  - Items with a counter of 0 are placed at the begining of each array.
//  - All non-zero counters in original_array_lt must be less than those in original_array_gte.
pub fn assert_combined_sorted_transformed_value_array_asc<T, S, let N: u32, Env>(
    original_array_lt: [T; N],
    original_array_gte: [T; N],
    sorted_transformed_value_array: [S; N],
    is_transformed: fn[Env](T, S) -> bool,
    hints: [CombinedOrderHint; N]
) where T: Ordered + Empty + Eq, S: Empty + Eq {
    let num_private_lt = get_num_private_items(original_array_lt);
    let num_private_gte = get_num_private_items(original_array_gte);
    let total_private = num_private_lt + num_private_gte;
    let num_lt = array_length(original_array_lt);
    let num_gte = array_length(original_array_gte);
    let total_non_public_gte = num_lt + num_private_gte;
    let total_num = num_lt + num_gte;

    let mut prev_counter = 0;
    let mut should_be_private_lt = true;
    let mut should_be_private = true;
    let mut should_be_public_gte = false;
    let mut should_be_empty = false;
    for i in 0..N {
        should_be_private_lt &= i != num_private_lt;
        should_be_private &= i != total_private;
        should_be_public_gte |= i == total_non_public_gte;
        should_be_empty |= i == total_num;
        let hint = hints[i];
        let from_lt = original_array_lt[hint.original_index];
        let from_gte = original_array_gte[hint.original_index];
        let from = if should_be_private_lt {
            assert_eq(hint.original_index, i, "items with a counter of 0 should be prepended");
            from_lt
        } else if should_be_private {
            assert_eq(hint.original_index, i - num_private_lt, "items with a counter of 0 should be prepended");
            from_gte
        } else if should_be_public_gte {
            from_gte
        } else {
            from_lt
        };
        let dest = sorted_transformed_value_array[i];
        let dest_counter = hint.counter;
        assert(is_transformed(from, dest), "incorrect transformed value");
        assert_eq(from.counter(), dest_counter, "mismatch counter");
        if !should_be_empty {
            if !should_be_private {
                assert(
                    dest_counter > prev_counter, "value array must be sorted by counter in ascending order"
                );
            }
        } else {
            assert(is_empty(dest), "array must be padded with empty items");
        }
        prev_counter = dest_counter;
    }
}

mod tests {
    use crate::{
        abis::side_effect::Ordered, tests::utils::pad_end, traits::Empty,
        utils::arrays::{
        array_merge,
        assert_combined_sorted_transformed_value_array::{
        assert_combined_sorted_transformed_value_array_asc,
        get_combined_order_hints::{CombinedOrderHint, get_combined_order_hints_asc}
    },
        sort_by_counter::sort_by_counter_asc
    }
    };

    struct TestItem {
        name: Field,
        price: Field,
        tax: Field,
        counter: u32,
    }

    impl Ordered for TestItem {
        fn counter(self) -> u32 {
            self.counter
        }
    }

    impl Empty for TestItem {
        fn empty() -> Self {
            TestItem { name: 0, price: 0, tax: 0, counter: 0 }
        }
    }

    impl Eq for TestItem {
        fn eq(self, other: Self) -> bool {
            (self.name == other.name) & (self.price == other.price) & (self.tax == other.tax) & (self.counter == other.counter)
        }
    }

    struct TestValue {
        name: Field,
        total: Field,
    }

    impl Empty for TestValue {
        fn empty() -> Self {
            TestValue { name: 0, total: 0 }
        }
    }

    impl Eq for TestValue {
        fn eq(self, other: Self) -> bool {
            (self.name == other.name) & (self.total == other.total)
        }
    }

    fn transform(item: TestItem) -> TestValue {
        TestValue { name: item.name, total: item.price + item.tax }
    }

    fn is_transformed(item: TestItem, value: TestValue) -> bool {
        (item.name == value.name) & ((item.price + item.tax) == value.total)
    }

    struct TestDataBuilder<T, S, let N: u32> {
        original_array_lt: [T; N],
        original_array_gte: [T; N],
        sorted_transformed_value_array: [S; N],
        hints: [CombinedOrderHint; N],
    }

    impl TestDataBuilder<TestItem, TestValue, 12> {
        pub fn new() -> Self {
            let original_array_lt = pad_end(
                [
                TestItem { name: 7, price: 40, tax: 7, counter: 0 },
                TestItem { name: 4, price: 70, tax: 6, counter: 0 },
                TestItem { name: 6, price: 80, tax: 1, counter: 22 },
                TestItem { name: 3, price: 30, tax: 4, counter: 11 }
            ],
                TestItem::empty()
            );

            let original_array_gte = pad_end(
                [
                TestItem { name: 9, price: 20, tax: 2, counter: 0 },
                TestItem { name: 8, price: 90, tax: 3, counter: 0 },
                TestItem { name: 5, price: 50, tax: 9, counter: 55 },
                TestItem { name: 1, price: 60, tax: 8, counter: 33 },
                TestItem { name: 2, price: 10, tax: 5, counter: 44 }
            ],
                TestItem::empty()
            );

            let sorted_transformed_value_array = pad_end(
                [
                TestValue { name: 7, total: 47 },
                TestValue { name: 4, total: 76 },
                TestValue { name: 9, total: 22 },
                TestValue { name: 8, total: 93 },
                TestValue { name: 3, total: 34 },
                TestValue { name: 6, total: 81 },
                TestValue { name: 1, total: 68 },
                TestValue { name: 2, total: 15 },
                TestValue { name: 5, total: 59 }
            ],
                TestValue::empty()
            );

            let hints = [
                CombinedOrderHint { counter: 0, original_index: 0 },
                CombinedOrderHint { counter: 0, original_index: 1 },
                CombinedOrderHint { counter: 0, original_index: 0 },
                CombinedOrderHint { counter: 0, original_index: 1 },
                CombinedOrderHint { counter: 11, original_index: 3 },
                CombinedOrderHint { counter: 22, original_index: 2 },
                CombinedOrderHint { counter: 33, original_index: 3 },
                CombinedOrderHint { counter: 44, original_index: 4 },
                CombinedOrderHint { counter: 55, original_index: 2 },
                CombinedOrderHint { counter: 0, original_index: 5 },
                CombinedOrderHint { counter: 0, original_index: 6 },
                CombinedOrderHint { counter: 0, original_index: 7 }
            ];

            TestDataBuilder { original_array_lt, original_array_gte, sorted_transformed_value_array, hints }
        }
    }

    impl TestDataBuilder<TestItem, TestValue, 8> {
        pub fn new_without_prepended() -> Self {
            let original_array_lt = pad_end(
                [
                TestItem { name: 6, price: 80, tax: 1, counter: 22 },
                TestItem { name: 3, price: 30, tax: 4, counter: 11 }
            ],
                TestItem::empty()
            );

            let original_array_gte = pad_end(
                [
                TestItem { name: 5, price: 50, tax: 9, counter: 55 },
                TestItem { name: 1, price: 60, tax: 8, counter: 33 },
                TestItem { name: 2, price: 10, tax: 5, counter: 44 }
            ],
                TestItem::empty()
            );

            let sorted_transformed_value_array = pad_end(
                [
                TestValue { name: 3, total: 34 },
                TestValue { name: 6, total: 81 },
                TestValue { name: 1, total: 68 },
                TestValue { name: 2, total: 15 },
                TestValue { name: 5, total: 59 }
            ],
                TestValue::empty()
            );

            let hints = [
                CombinedOrderHint { counter: 11, original_index: 1 },
                CombinedOrderHint { counter: 22, original_index: 0 },
                CombinedOrderHint { counter: 33, original_index: 1 },
                CombinedOrderHint { counter: 44, original_index: 2 },
                CombinedOrderHint { counter: 55, original_index: 0 },
                CombinedOrderHint { counter: 0, original_index: 3 },
                CombinedOrderHint { counter: 0, original_index: 4 },
                CombinedOrderHint { counter: 0, original_index: 5 }
            ];

            TestDataBuilder { original_array_lt, original_array_gte, sorted_transformed_value_array, hints }
        }
    }

    impl<let N: u32> TestDataBuilder<TestItem, TestValue, N> {
        pub fn swap_items(&mut self, from: u32, to: u32) {
            let tmp = self.sorted_transformed_value_array[from];
            self.sorted_transformed_value_array[from] = self.sorted_transformed_value_array[to];
            self.sorted_transformed_value_array[to] = tmp;
        }

        pub fn swap_hints(&mut self, from: u32, to: u32) {
            let tmp = self.hints[from];
            self.hints[from] = self.hints[to];
            self.hints[to] = tmp;
        }

        pub fn execute(self) {
            assert_combined_sorted_transformed_value_array_asc(
                self.original_array_lt,
                self.original_array_gte,
                self.sorted_transformed_value_array,
                is_transformed,
                self.hints
            );
        }
    }

    #[test]
    fn assert_combined_sorted_transformed_value_array_asc_succeeds() {
        let builder = TestDataBuilder::new();

        let hints = get_combined_order_hints_asc(builder.original_array_lt, builder.original_array_gte);
        assert_eq(hints, builder.hints);

        let sorted = sort_by_counter_asc(array_merge(builder.original_array_lt, builder.original_array_gte)).map(transform);
        assert_eq(sorted, builder.sorted_transformed_value_array);

        builder.execute();
    }

    #[test]
    fn assert_combined_sorted_transformed_value_array_asc_without_prepended_counter_zeros_succeeds() {
        let builder = TestDataBuilder::new_without_prepended();

        let hints = get_combined_order_hints_asc(builder.original_array_lt, builder.original_array_gte);
        assert_eq(hints, builder.hints);

        let sorted = sort_by_counter_asc(array_merge(builder.original_array_lt, builder.original_array_gte)).map(transform);
        assert_eq(sorted, builder.sorted_transformed_value_array);

        builder.execute();
    }

    #[test(should_fail_with="incorrect transformed value")]
    fn assert_combined_sorted_transformed_value_array_asc_mismatch_value_fails() {
        let mut builder = TestDataBuilder::new();

        // Tweak the value at index 1.
        builder.sorted_transformed_value_array[1].total += 1;

        builder.execute();
    }

    #[test(should_fail_with="incorrect transformed value")]
    fn assert_combined_sorted_transformed_value_array_asc_cross_boundary_fails() {
        let mut builder = TestDataBuilder::new();

        builder.swap_items(4, 6);

        builder.execute();
    }

    #[test(should_fail_with="items with a counter of 0 should be prepended")]
    fn assert_combined_sorted_transformed_value_array_asc_unordered_prepended_fails() {
        let mut builder = TestDataBuilder::new();

        builder.swap_items(0, 1);
        builder.swap_hints(0, 1);

        builder.execute();
    }

    #[test(should_fail_with="items with a counter of 0 should be prepended")]
    fn assert_combined_sorted_transformed_value_array_asc_mixed_prepended_fails() {
        let mut builder = TestDataBuilder::new();

        builder.swap_items(1, 4);
        builder.swap_hints(1, 4);

        builder.execute();
    }

    #[test(should_fail_with="value array must be sorted by counter in ascending order")]
    fn assert_combined_sorted_transformed_value_array_asc_unordered_fails() {
        let mut builder = TestDataBuilder::new();

        builder.swap_items(4, 5);
        builder.swap_hints(4, 5);

        builder.execute();
    }

    #[test(should_fail_with="mismatch counter")]
    fn assert_combined_sorted_transformed_value_array_asc_unordered_values_with_ordered_counters_fails() {
        let mut builder = TestDataBuilder::new();

        builder.swap_items(4, 5);

        // Update indexes in hints.
        let tmp = builder.hints[4].original_index;
        builder.hints[4].original_index = builder.hints[5].original_index;
        builder.hints[5].original_index = tmp;

        builder.execute();
    }

    #[test(should_fail_with="array must be padded with empty items")]
    fn assert_combined_sorted_transformed_value_array_asc_extra_non_empty_fails() {
        let mut builder = TestDataBuilder::new();

        // Add an item.
        builder.sorted_transformed_value_array[9] = builder.sorted_transformed_value_array[8];
        builder.hints[9] = builder.hints[8];

        builder.execute();
    }

    #[test(should_fail_with="value array must be sorted by counter in ascending order")]
    fn assert_combined_sorted_transformed_value_array_asc_extra_empty_fails() {
        let mut builder = TestDataBuilder::new();

        // Clear an item.
        builder.sorted_transformed_value_array[8] = TestValue::empty();
        builder.hints[8] = CombinedOrderHint { counter: 0, original_index: 6 };

        builder.execute();
    }

    #[test(should_fail_with="value array must be sorted by counter in ascending order")]
    fn assert_combined_sorted_transformed_value_array_asc_duplicate_fails() {
        let mut builder = TestDataBuilder::new();

        // Duplicate an item.
        builder.sorted_transformed_value_array[5] = builder.sorted_transformed_value_array[4];
        builder.hints[5] = builder.hints[4];

        builder.execute();
    }

    #[test(should_fail_with="items with a counter of 0 should be prepended")]
    fn assert_combined_sorted_transformed_value_array_asc_duplicate_zero_counter_fails() {
        let mut builder = TestDataBuilder::new();

        // Duplicate an item with 0 counter.
        builder.sorted_transformed_value_array[1] = builder.sorted_transformed_value_array[0];
        builder.hints[1] = builder.hints[0];

        builder.execute();
    }
}

