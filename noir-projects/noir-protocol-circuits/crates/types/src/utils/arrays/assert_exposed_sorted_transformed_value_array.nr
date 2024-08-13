use crate::{
    abis::side_effect::Ordered, traits::{Empty, is_empty},
    utils::arrays::{array_length, sort_get_order_hints::OrderHint}
};

// original_array must be valid, i.e. validate_array(original_array) == true
// Items in exposed_sorted_transformed_value_array do not have counters. Their corresponding counters are in hints.
pub fn assert_exposed_sorted_transformed_value_array<T, S, let N: u32, Env>(
    original_array: [T; N],
    exposed_sorted_transformed_value_array: [S; N],
    is_transformed: fn[Env](T, S) -> bool,
    hints: [OrderHint; N]
) where T: Ordered + Empty + Eq, S: Empty + Eq {
    let num_non_empty_items = array_length(original_array);
    let mut should_be_empty = false;
    for i in 0..N {
        should_be_empty |= i == num_non_empty_items;
        if should_be_empty {
            assert(
                is_empty(exposed_sorted_transformed_value_array[i]), "array must be padded with empty items"
            );
        } else {
            let original = original_array[i];
            let sorted_index = hints[i].sorted_index;
            let sorted = exposed_sorted_transformed_value_array[sorted_index];
            assert(is_transformed(original, sorted), "incorrect transformed value");
            assert_eq(original.counter(), hints[sorted_index].counter, "incorrect hinted counter");
            if i != 0 {
                assert(
                    hints[i].counter > hints[i - 1].counter, "value array must be sorted by counter in ascending order"
                );
            }
        }
    }
}

mod tests {
    use crate::{
        abis::side_effect::Ordered, traits::Empty,
        utils::arrays::{
        assert_exposed_sorted_transformed_value_array::assert_exposed_sorted_transformed_value_array,
        sort_get_order_hints::OrderHint
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

    fn is_transformed(item: TestItem, value: TestValue) -> bool {
        (item.name == value.name) & ((item.price + item.tax) == value.total)
    }

    struct TestDataBuilder<T, S, let N: u32> {
        original_array: [T; N],
        exposed_sorted_transformed_value_array: [S; N],
        hints: [OrderHint; N],
    }

    impl TestDataBuilder<TestItem, TestValue, 6> {
        pub fn new() -> Self {
            let original_array = [
                TestItem { name: 100, price: 10, tax: 5, counter: 44 },
                TestItem { name: 200, price: 20, tax: 6, counter: 22 },
                TestItem { name: 300, price: 30, tax: 7, counter: 11 },
                TestItem { name: 400, price: 40, tax: 8, counter: 33 },
                TestItem::empty(),
                TestItem::empty()
            ];

            let exposed_sorted_transformed_value_array = [
                TestValue { name: 300, total: 37 },
                TestValue { name: 200, total: 26 },
                TestValue { name: 400, total: 48 },
                TestValue { name: 100, total: 15 },
                TestValue::empty(),
                TestValue::empty()
            ];

            let hints = [
                OrderHint { counter: 11, sorted_index: 3 },
                OrderHint { counter: 22, sorted_index: 1 },
                OrderHint { counter: 33, sorted_index: 0 },
                OrderHint { counter: 44, sorted_index: 2 },
                OrderHint { counter: 0, sorted_index: 4 },
                OrderHint { counter: 0, sorted_index: 5 }
            ];

            TestDataBuilder { original_array, exposed_sorted_transformed_value_array, hints }
        }

        pub fn execute(self) {
            assert_exposed_sorted_transformed_value_array(
                self.original_array,
                self.exposed_sorted_transformed_value_array,
                is_transformed,
                self.hints
            );
        }
    }

    #[test]
    fn assert_exposed_sorted_transformed_value_array_succeeds() {
        let builder = TestDataBuilder::new();
        builder.execute();
    }

    #[test(should_fail_with="incorrect transformed value")]
    fn assert_exposed_sorted_transformed_value_array_mismatch_value_fails() {
        let mut builder = TestDataBuilder::new();

        // Tweak the value at index 1.
        builder.exposed_sorted_transformed_value_array[1].total += 1;

        builder.execute();
    }

    #[test(should_fail_with="value array must be sorted by counter in ascending order")]
    fn assert_exposed_sorted_transformed_value_array_unordered_fails() {
        let mut builder = TestDataBuilder::new();

        // Swap the values at index 1 and 2.
        let tmp = builder.exposed_sorted_transformed_value_array[1];
        builder.exposed_sorted_transformed_value_array[1] = builder.exposed_sorted_transformed_value_array[2];
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

    #[test(should_fail_with="incorrect hinted counter")]
    fn assert_exposed_sorted_transformed_value_array_unordered_values_with_ordered_counters_fails() {
        let mut builder = TestDataBuilder::new();

        // Swap the values at index 1 and 2.
        let tmp = builder.exposed_sorted_transformed_value_array[1];
        builder.exposed_sorted_transformed_value_array[1] = builder.exposed_sorted_transformed_value_array[2];
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

    #[test(should_fail_with="array must be padded with empty items")]
    fn assert_exposed_sorted_transformed_value_array_extra_non_empty_fails() {
        let mut builder = TestDataBuilder::new();

        // Add a random item.
        builder.exposed_sorted_transformed_value_array[4] = TestValue { name: 500, total: 10 };

        builder.execute();
    }

    #[test(should_fail_with="array must be padded with empty items")]
    fn assert_exposed_sorted_transformed_value_array_hint_to_extra_non_empty_fails() {
        let mut builder = TestDataBuilder::new();

        // Add a random item.
        builder.exposed_sorted_transformed_value_array[4] = TestValue { name: 500, total: 10 };
        // Change the hint to point to an empty item.
        builder.hints[4].sorted_index = 5;

        builder.execute();
    }

    #[test(should_fail_with="incorrect transformed value")]
    fn assert_exposed_sorted_transformed_value_array_missing_item_fails() {
        let mut builder = TestDataBuilder::new();

        // Remove an item.
        builder.exposed_sorted_transformed_value_array[3] = TestValue::empty();

        builder.execute();
    }
}
