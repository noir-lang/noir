use crate::{abis::side_effect::Ordered, traits::{Empty, is_empty}, utils::arrays::sort_get_order_hints::OrderHint};

// original_array must be valid, i.e. validate_array(original_array) == true
// transformed_value_array must be verified against original_array before calling this function.
pub fn assert_sorted_transformed_value_array<T, S, N>(
    original_array: [T; N],
    transformed_value_array: [S; N],
    sorted_transformed_value_array: [S; N],
    hints: [OrderHint; N]
) where T: Ordered, S: Empty + Eq {
    for i in 0..N {
        let original = original_array[i];
        let value = transformed_value_array[i];
        if is_empty(value) {
            assert(is_empty(sorted_transformed_value_array[i]), "unexpected non-empty item");
        } else {
            let sorted_index = hints[i].sorted_index;
            assert_eq(value, sorted_transformed_value_array[sorted_index], "mismatch sorted values");
            assert_eq(original.counter(), hints[sorted_index].counter, "mismatch counters");
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
        assert_sorted_transformed_value_array::assert_sorted_transformed_value_array,
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

    fn transform_value(item: TestItem) -> TestValue {
        TestValue { name: item.name, total: item.price + item.tax }
    }

    struct TestDataBuilder<T, S, N> {
        original_array: [T; N],
        transformed_value_array: [S; N],
        sorted_transformed_value_array: [S; N],
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

            let transformed_value_array = original_array.map(|item: TestItem| transform_value(item));

            let sorted_transformed_value_array = [
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
                OrderHint { counter: 0, sorted_index: 0 },
                OrderHint { counter: 0, sorted_index: 0 }
            ];

            TestDataBuilder { original_array, transformed_value_array, sorted_transformed_value_array, hints }
        }

        pub fn execute(self) {
            assert_sorted_transformed_value_array(
                self.original_array,
                self.transformed_value_array,
                self.sorted_transformed_value_array,
                self.hints
            );
        }
    }

    #[test]
    fn assert_sorted_transformed_value_array_succeeds() {
        let builder = TestDataBuilder::new();
        builder.execute();
    }

    #[test(should_fail_with="mismatch sorted values")]
    fn assert_sorted_transformed_value_array_mismatch_value_fails() {
        let mut builder = TestDataBuilder::new();

        // Tweak the value at index 1.
        builder.sorted_transformed_value_array[1].total += 1;

        builder.execute();
    }

    #[test(should_fail_with="mismatch counters")]
    fn assert_sorted_transformed_value_array_mismatch_counter_fails() {
        let mut builder = TestDataBuilder::new();

        // Tweak the counter at index 1.
        builder.hints[1].counter += 1;

        builder.execute();
    }

    #[test(should_fail_with="value array must be sorted by counter in ascending order")]
    fn assert_sorted_transformed_value_array_unordered_fails() {
        let mut builder = TestDataBuilder::new();

        // Swap the values at index 1 and 2.
        let tmp = builder.sorted_transformed_value_array[1];
        builder.sorted_transformed_value_array[1] = builder.sorted_transformed_value_array[2];
        builder.sorted_transformed_value_array[2] = tmp;

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

    #[test(should_fail_with="unexpected non-empty item")]
    fn assert_sorted_transformed_value_array_extra_non_empty_fails() {
        let mut builder = TestDataBuilder::new();

        // Add a random item.
        builder.sorted_transformed_value_array[4] = TestValue { name: 500, total: 10 };

        builder.execute();
    }

    #[test(should_fail_with="mismatch sorted values")]
    fn assert_sorted_transformed_value_array_missing_item_fails() {
        let mut builder = TestDataBuilder::new();

        // Remove an item.
        builder.sorted_transformed_value_array[3] = TestValue::empty();

        builder.execute();
    }
}
