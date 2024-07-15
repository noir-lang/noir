use crate::{abis::side_effect::Ordered, traits::{Empty, is_empty}, utils::arrays::sort_get_order_hints::OrderHint};

pub fn assert_sorted_array<T, N, Env>(
    original_array: [T; N],
    sorted_array: [T; N],
    sorted_indexes: [u32; N],
    ordering: fn[Env](T, T) -> bool
) where T: Eq + Empty {
    let mut seen_empty = false;
    for i in 0..N {
        let original_value = original_array[i];
        if is_empty(original_value) {
            seen_empty = true;
            assert(is_empty(sorted_array[i]), "Empty values must not be mixed with sorted values");
        } else {
            assert(!seen_empty, "Empty values must be padded to the right");

            let index = sorted_indexes[i];
            assert(sorted_array[index].eq(original_value), "Invalid index");
            if i != 0 {
                assert(ordering(sorted_array[i - 1], sorted_array[i]), "Values not sorted");
            }
        }
    }
}

// original_array must be valid, i.e. validate_array(original_array) == true
pub fn assert_sorted_array_with_order_hints<T, N>(
    original_array: [T; N],
    sorted_array: [T; N],
    hints: [OrderHint; N]
) where T: Ordered + Empty + Eq {
    for i in 0..N {
        let original_value = original_array[i];
        if is_empty(original_value) {
            assert(is_empty(sorted_array[i]), "Empty values must not be mixed with sorted values");
        } else {
            let hint = hints[i];
            assert_eq(sorted_array[hint.sorted_index], original_value, "Mismatch sorted values");
            if i != 0 {
                assert(
                    sorted_array[i].counter() > sorted_array[i - 1].counter(), "Array must be sorted by counter in ascending order"
                );
            }
        }
    }
}

mod tests {
    use crate::{
        abis::side_effect::Ordered, traits::Empty,
        utils::arrays::{
        assert_sorted_array::{assert_sorted_array, assert_sorted_array_with_order_hints},
        sort_get_order_hints::OrderHint
    }
    };

    #[test]
    fn assert_sorted_array_custom_asc_success() {
        let original = [30, 20, 90, 50, 0, 0];
        let sorted = [20, 30, 50, 90, 0, 0];
        let indexes = [1, 0, 3, 2, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test]
    fn assert_sorted_array_custom_desc_success() {
        let original = [30, 20, 90, 50, 0, 0];
        let sorted = [90, 50, 30, 20, 0, 0];
        let indexes = [2, 3, 0, 1, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| b.lt(a));
    }

    #[test]
    fn assert_sorted_array_custom_all_empty_success() {
        let original = [0, 0, 0, 0, 0, 0];
        let sorted = [0, 0, 0, 0, 0, 0];
        let indexes = [0, 0, 0, 0, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test(should_fail_with="Values not sorted")]
    fn assert_sorted_array_custom_wrong_ordering_fails() {
        let original = [30, 20, 90, 50, 0, 0];
        let sorted = [20, 30, 90, 50, 0, 0];
        let indexes = [1, 0, 2, 3, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test(should_fail_with="Values not sorted")]
    fn assert_sorted_array_custom_misplaced_sorted_fails() {
        let original = [30, 20, 90, 50, 0, 0];
        let sorted = [20, 30, 50, 0, 0, 90];
        let indexes = [1, 0, 5, 2, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test(should_fail_with="Invalid index")]
    fn assert_sorted_array_custom_wrong_index_fails() {
        let original = [30, 20, 90, 50, 0, 0];
        let sorted = [20, 30, 50, 90, 0, 0];
        let indexes = [1, 1, 2, 3, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test(should_fail_with="Empty values must be padded to the right")]
    fn assert_sorted_array_custom_not_padded_fails() {
        let original = [30, 20, 90, 0, 50, 0];
        let sorted = [20, 30, 90, 0, 0, 0];
        let indexes = [1, 0, 2, 0, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test(should_fail_with="Empty values must not be mixed with sorted values")]
    fn assert_sorted_array_custom_mixed_empty_fails() {
        let original = [30, 20, 90, 0, 0, 0];
        let sorted = [20, 30, 90, 0, 0, 10];
        let indexes = [1, 0, 2, 0, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

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

    struct TestDataBuilder<T, N> {
        original_array: [T; N],
        sorted_array: [T; N],
        hints: [OrderHint; N],
    }

    impl TestDataBuilder<TestItem, 6> {
        pub fn new() -> Self {
            let original_array = [
                TestItem { name: 100, price: 10, tax: 5, counter: 44 },
                TestItem { name: 200, price: 20, tax: 6, counter: 22 },
                TestItem { name: 300, price: 30, tax: 7, counter: 11 },
                TestItem { name: 400, price: 40, tax: 8, counter: 33 },
                TestItem::empty(),
                TestItem::empty()
            ];

            let sorted_array = [
                TestItem { name: 300, price: 30, tax: 7, counter: 11 },
                TestItem { name: 200, price: 20, tax: 6, counter: 22 },
                TestItem { name: 400, price: 40, tax: 8, counter: 33 },
                TestItem { name: 100, price: 10, tax: 5, counter: 44 },
                TestItem::empty(),
                TestItem::empty()
            ];

            let hints = [
                OrderHint { counter: 11, sorted_index: 3 },
                OrderHint { counter: 22, sorted_index: 1 },
                OrderHint { counter: 33, sorted_index: 0 },
                OrderHint { counter: 44, sorted_index: 2 },
                OrderHint { counter: 0, sorted_index: 0 },
                OrderHint { counter: 0, sorted_index: 0 }
            ];

            TestDataBuilder { original_array, sorted_array, hints }
        }

        pub fn execute(self) {
            assert_sorted_array_with_order_hints(self.original_array, self.sorted_array, self.hints);
        }
    }

    #[test]
    fn assert_sorted_array_with_order_hints_succeeds() {
        let builder = TestDataBuilder::new();
        builder.execute();
    }

    #[test(should_fail_with="Mismatch sorted values")]
    fn assert_sorted_array_with_order_hints_mismatch_value_fails() {
        let mut builder = TestDataBuilder::new();

        // Tweak the value at index 1.
        builder.sorted_array[1].price += 1;

        builder.execute();
    }

    #[test(should_fail_with="Mismatch sorted values")]
    fn assert_sorted_array_with_order_hints_mismatch_counter_fails() {
        let mut builder = TestDataBuilder::new();

        // Tweak the counter at index 1.
        builder.sorted_array[1].counter += 1;

        builder.execute();
    }

    #[test(should_fail_with="Array must be sorted by counter in ascending order")]
    fn assert_sorted_array_with_order_hints_unordered_fails() {
        let mut builder = TestDataBuilder::new();

        // Swap the values at index 1 and 2.
        let tmp = builder.sorted_array[1];
        builder.sorted_array[1] = builder.sorted_array[2];
        builder.sorted_array[2] = tmp;

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

    #[test(should_fail_with="Empty values must not be mixed with sorted values")]
    fn assert_sorted_array_with_order_hints_extra_non_empty_fails() {
        let mut builder = TestDataBuilder::new();

        // Add a random item.
        builder.sorted_array[4] = TestItem { name: 500, price: 10, tax: 5, counter: 55 };

        builder.execute();
    }

    #[test(should_fail_with="Mismatch sorted values")]
    fn assert_sorted_array_with_order_hints_missing_item_fails() {
        let mut builder = TestDataBuilder::new();

        // Remove an item.
        builder.sorted_array[3] = TestItem::empty();

        builder.execute();
    }
}

