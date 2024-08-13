use crate::{abis::side_effect::Ordered, traits::{Empty, is_empty}, utils::arrays::array_length};

pub fn assert_sorted_transformed_value_array<T, S, let N: u32, Env>(
    original_array: [T; N],
    sorted_transformed_value_array: [S; N],
    is_transformed: fn[Env](T, S) -> bool,
    sorted_indexes: [u32; N]
) where T: Eq + Empty, S: Ordered + Eq + Empty {
    assert_sorted_transformed_value_array_capped_size(
        original_array,
        sorted_transformed_value_array,
        is_transformed,
        sorted_indexes,
        N
    );
}

// original_array must be valid, i.e. validate_array(original_array) == true
// capped_size should be a known constant at compile time.
pub fn assert_sorted_transformed_value_array_capped_size<T, S, let N: u32, Env>(
    original_array: [T; N],
    sorted_transformed_value_array: [S; N],
    is_transformed: fn[Env](T, S) -> bool,
    sorted_indexes: [u32; N],
    capped_size: u32
) where T: Eq + Empty, S: Ordered + Eq + Empty {
    let num_non_empty_items = array_length(original_array);
    let mut should_be_empty = false;
    for i in 0..N {
        let original = original_array[i];
        let mapped_item = sorted_transformed_value_array[i];
        let is_mapped_item_empty = is_empty(mapped_item);
        if i == capped_size {
            assert(is_empty(original), "capped_size does not cover all non-empty items");
        }
        if i < capped_size {
            should_be_empty |= i == num_non_empty_items;
            let sorted_index = sorted_indexes[i];
            let sorted = sorted_transformed_value_array[sorted_index];
            assert(is_transformed(original, sorted), "incorrect transformed value");
            if should_be_empty {
                assert(is_mapped_item_empty, "unexpected non-empty item");
            } else if i != 0 {
                assert(
                    mapped_item.counter() > sorted_transformed_value_array[i - 1].counter(), "value array must be sorted by counter in ascending order"
                );
            }
        } else {
            assert(is_mapped_item_empty, "array must be padded with empty items");
        }
    }
}

mod tests {
    use crate::{
        abis::side_effect::Ordered, traits::Empty,
        utils::arrays::{assert_sorted_transformed_value_array::{assert_sorted_transformed_value_array, assert_sorted_transformed_value_array_capped_size}}
    };

    struct TestItem {
        name: Field,
        price: Field,
        tax: Field,
        counter: u32,
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
        counter: u32,
    }

    impl Ordered for TestValue {
        fn counter(self) -> u32 {
            self.counter
        }
    }

    impl Empty for TestValue {
        fn empty() -> Self {
            TestValue { name: 0, total: 0, counter: 0 }
        }
    }

    impl Eq for TestValue {
        fn eq(self, other: Self) -> bool {
            (self.name == other.name) & (self.total == other.total) & (self.counter == other.counter)
        }
    }

    fn is_transformed(item: TestItem, value: TestValue) -> bool {
        (item.name == value.name)
            & ((item.price + item.tax) == value.total)
            & (item.counter == value.counter)
    }

    struct TestDataBuilder<T, S, let N: u32> {
        original_array: [T; N],
        sorted_transformed_value_array: [S; N],
        sorted_indexes: [u32; N],
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

            let sorted_transformed_value_array = [
                TestValue { name: 300, total: 37, counter: 11 },
                TestValue { name: 200, total: 26, counter: 22 },
                TestValue { name: 400, total: 48, counter: 33 },
                TestValue { name: 100, total: 15, counter: 44 },
                TestValue::empty(),
                TestValue::empty()
            ];

            let sorted_indexes = [3, 1, 0, 2, 4, 5];

            TestDataBuilder { original_array, sorted_transformed_value_array, sorted_indexes }
        }

        pub fn execute(self) {
            assert_sorted_transformed_value_array(
                self.original_array,
                self.sorted_transformed_value_array,
                is_transformed,
                self.sorted_indexes
            );
        }

        pub fn execute_capped(self, capped_size: u32) {
            assert_sorted_transformed_value_array_capped_size(
                self.original_array,
                self.sorted_transformed_value_array,
                is_transformed,
                self.sorted_indexes,
                capped_size
            );
        }
    }

    #[test]
    fn assert_sorted_transformed_value_array_succeeds() {
        let builder = TestDataBuilder::new();
        builder.execute();
    }

    #[test(should_fail_with="incorrect transformed value")]
    fn assert_sorted_transformed_value_array_mismatch_value_fails() {
        let mut builder = TestDataBuilder::new();

        // Tweak the value at index 1.
        builder.sorted_transformed_value_array[1].total += 1;

        builder.execute();
    }

    #[test(should_fail_with="value array must be sorted by counter in ascending order")]
    fn assert_sorted_transformed_value_array_unordered_fails() {
        let mut builder = TestDataBuilder::new();

        // Swap the values at index 1 and 2.
        let tmp = builder.sorted_transformed_value_array[1];
        builder.sorted_transformed_value_array[1] = builder.sorted_transformed_value_array[2];
        builder.sorted_transformed_value_array[2] = tmp;

        // Update sorted indexes.
        // Original: 44, 22, 11, 33
        // New: 11, 33, 22, 44
        builder.sorted_indexes[0] = 3;
        builder.sorted_indexes[1] = 2;
        builder.sorted_indexes[2] = 0;
        builder.sorted_indexes[3] = 1;

        builder.execute();
    }

    #[test(should_fail_with="incorrect transformed value")]
    fn assert_sorted_transformed_value_array_extra_non_empty_fails() {
        let mut builder = TestDataBuilder::new();

        // Add a random item.
        builder.sorted_transformed_value_array[4] = TestValue { name: 500, total: 10, counter: 55 };

        builder.execute();
    }

    #[test(should_fail_with="unexpected non-empty item")]
    fn assert_sorted_transformed_value_array_hint_to_extra_non_empty_fails() {
        let mut builder = TestDataBuilder::new();

        // Add a random item.
        builder.sorted_transformed_value_array[4] = TestValue { name: 500, total: 10, counter: 55 };
        // Change the hint to point to an empty item.
        builder.sorted_indexes[4] = 5;

        builder.execute();
    }

    #[test(should_fail_with="incorrect transformed value")]
    fn assert_sorted_transformed_value_array_missing_item_fails() {
        let mut builder = TestDataBuilder::new();

        // Remove an item.
        builder.sorted_transformed_value_array[3] = TestValue::empty();

        builder.execute();
    }

    #[test]
    fn assert_sorted_transformed_value_array_capped_size_succeeds() {
        let builder = TestDataBuilder::new();
        builder.execute_capped(5);
    }

    #[test(should_fail_with="capped_size does not cover all non-empty items")]
    fn assert_sorted_transformed_value_array_capped_size_not_fully_covered_fails() {
        let builder = TestDataBuilder::new();
        builder.execute_capped(3);
    }

    #[test(should_fail_with="array must be padded with empty items")]
    fn assert_sorted_transformed_value_array_capped_size_extra_non_empty_fails() {
        let mut builder = TestDataBuilder::new();

        // Add a random item outside of capped_size.
        builder.sorted_transformed_value_array[4] = TestValue { name: 500, total: 10, counter: 55 };

        builder.execute_capped(4);
    }
}
