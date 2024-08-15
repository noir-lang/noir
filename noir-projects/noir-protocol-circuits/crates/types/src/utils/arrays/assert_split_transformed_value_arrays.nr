use crate::{abis::side_effect::Ordered, traits::{Empty, is_empty}, utils::arrays::find_index_hint};

pub fn assert_split_transformed_value_arrays_with_hint<T, S, let N: u32, Env>(
    sorted_array: [T; N],
    transformed_value_array_lt: [S; N], // Values whose counters are less than the split counter.
    transformed_value_array_gte: [S; N], // Values whose counters are greater than or equal to the split counter.
    is_transformed: fn[Env](T, S) -> bool,
    split_counter: u32,
    num_non_revertibles: u32 // Hint. Should check in this function.
) where T: Ordered + Empty + Eq, S: Empty + Eq {
    if num_non_revertibles != 0 {
        assert(
            sorted_array[num_non_revertibles - 1].counter() < split_counter, "counter of last non-revertible item is not less than the split counter"
        );
    }

    if num_non_revertibles != N {
        let first_revertible_counter = sorted_array[num_non_revertibles].counter();
        assert(
            (first_revertible_counter == 0) | (first_revertible_counter >= split_counter), "counter of first revertible item is not greater than or equal to the split counter"
        );
    }

    let mut is_lt = true;
    for i in 0..N {
        is_lt &= i != num_non_revertibles;

        let from = sorted_array[i];
        let to = if is_lt {
            transformed_value_array_lt[i]
        } else {
            transformed_value_array_gte[i - num_non_revertibles]
        };
        assert(is_transformed(from, to), "invalid transformed value");

        let padded = if is_lt {
            transformed_value_array_gte[N - i - 1]
        } else {
            transformed_value_array_lt[i]
        };
        assert(is_empty(padded), "array should be padded with empty items");
    }
}

unconstrained fn get_num_non_revertibles<T, let N: u32>(
    sorted_array: [T; N],
    split_counter: u32
) -> u32 where T: Ordered {
    find_index_hint(sorted_array, |n: T| n.counter() >= split_counter)
}

pub fn assert_split_transformed_value_arrays<T, S, let N: u32, Env>(
    sorted_array: [T; N],
    transformed_value_array_lt: [S; N], // Values whose counters are less than the split counter.
    transformed_value_array_gte: [S; N], // Values whose counters are greater than or equal to the split counter.
    is_transformed: fn[Env](T, S) -> bool,
    split_counter: u32
) where T: Ordered + Empty + Eq, S: Empty + Eq {
    let num_non_revertibles = get_num_non_revertibles(sorted_array, split_counter);

    assert_split_transformed_value_arrays_with_hint(
        sorted_array,
        transformed_value_array_lt,
        transformed_value_array_gte,
        is_transformed,
        split_counter,
        num_non_revertibles
    );
}

mod tests {
    use crate::{
        abis::side_effect::Ordered, traits::Empty,
        utils::arrays::assert_split_transformed_value_arrays::{assert_split_transformed_value_arrays, assert_split_transformed_value_arrays_with_hint}
    };

    struct TestItem {
        value: Field,
        counter: u32,
    }

    impl Ordered for TestItem {
        fn counter(self) -> u32 {
            self.counter
        }
    }

    impl Eq for TestItem {
        fn eq(self, other: Self) -> bool {
            (self.value == other.value) & (self.counter == other.counter)
        }
    }

    impl Empty for TestItem {
        fn empty() -> Self {
            TestItem { value: 0, counter: 0 }
        }
    }

    global NUM_TEST_ITEMS = 5;

    fn is_transformed(from: TestItem, to: Field) -> bool {
        from.value == to
    }

    fn pad_end<T, let N: u32>(items: [T; N], emptyItem: T) -> [T; NUM_TEST_ITEMS] {
        let mut output = [emptyItem; NUM_TEST_ITEMS];
        for i in 0..N {
            output[i] = items[i];
        }
        output
    }

    struct TestBuilder {
        sorted_array: [TestItem; NUM_TEST_ITEMS],
        transformed_value_array_lt: [Field; NUM_TEST_ITEMS],
        transformed_value_array_gte: [Field; NUM_TEST_ITEMS],
        split_counter: u32
    }

    impl TestBuilder {
        pub fn new() -> Self {
            let sorted_array = [
                TestItem { value: 40, counter: 2 },
                TestItem { value: 30, counter: 7 },
                TestItem { value: 80, counter: 11 },
                TestItem { value: 10, counter: 13 },
                TestItem { value: 50, counter: 29 }
            ];

            let transformed_value_array_lt = pad_end([40, 30, 80], 0);

            let transformed_value_array_gte = pad_end([10, 50], 0);

            TestBuilder { sorted_array, transformed_value_array_lt, transformed_value_array_gte, split_counter: 12 }
        }

        pub fn execute(self) {
            assert_split_transformed_value_arrays(
                self.sorted_array,
                self.transformed_value_array_lt,
                self.transformed_value_array_gte,
                is_transformed,
                self.split_counter
            );
        }

        pub fn execute_with_num_non_revertibles(self, num_non_revertibles: u32) {
            assert_split_transformed_value_arrays_with_hint(
                self.sorted_array,
                self.transformed_value_array_lt,
                self.transformed_value_array_gte,
                is_transformed,
                self.split_counter,
                num_non_revertibles
            );
        }
    }

    #[test]
    fn assert_split_transformed_value_arrays_succeeds() {
        let builder = TestBuilder::new();
        builder.execute();
    }

    #[test]
    fn assert_split_transformed_value_arrays_empty_succeeds() {
        let mut builder = TestBuilder::new();

        builder.sorted_array = [TestItem::empty(); NUM_TEST_ITEMS];
        builder.transformed_value_array_lt = [0; NUM_TEST_ITEMS];
        builder.transformed_value_array_gte = [0; NUM_TEST_ITEMS];

        builder.execute();
    }

    #[test]
    fn assert_split_transformed_value_arrays_empty_non_revertible_succeeds() {
        let mut builder = TestBuilder::new();

        builder.transformed_value_array_lt = [0; NUM_TEST_ITEMS];
        builder.transformed_value_array_gte = [40, 30, 80, 10, 50];
        builder.split_counter = 1;

        builder.execute();
    }

    #[test]
    fn assert_split_transformed_value_arrays_empty_revertible_succeeds() {
        let mut builder = TestBuilder::new();

        builder.transformed_value_array_lt = [40, 30, 80, 10, 50];
        builder.transformed_value_array_gte = [0; NUM_TEST_ITEMS];
        builder.split_counter = 99;

        builder.execute();
    }

    #[test(should_fail_with="counter of first revertible item is not greater than or equal to the split counter")]
    fn assert_split_transformed_value_arrays_num_non_revertible_too_small_fails() {
        let builder = TestBuilder::new();
        builder.execute_with_num_non_revertibles(2);
    }

    #[test(should_fail_with="counter of last non-revertible item is not less than the split counter")]
    fn assert_split_transformed_value_arrays_num_non_revertible_too_big_fails() {
        let mut builder = TestBuilder::new();
        builder.execute_with_num_non_revertibles(4);
    }

    #[test(should_fail_with="invalid transformed value")]
    fn assert_split_transformed_value_arrays_wrong_transformed_value_in_lt_fails() {
        let mut builder = TestBuilder::new();

        builder.transformed_value_array_lt[1] += 1;

        builder.execute();
    }

    #[test(should_fail_with="invalid transformed value")]
    fn assert_split_transformed_value_arrays_wrong_transformed_value_in_gte_fails() {
        let mut builder = TestBuilder::new();

        builder.transformed_value_array_gte[1] += 1;

        builder.execute();
    }

    #[test(should_fail_with="array should be padded with empty items")]
    fn assert_split_transformed_value_arrays_extra_non_empty_item_in_lt_fails() {
        let mut builder = TestBuilder::new();

        builder.transformed_value_array_lt[3] = 1;

        builder.execute();
    }

    #[test(should_fail_with="array should be padded with empty items")]
    fn assert_split_transformed_value_arrays_extra_non_empty_item_in_gte_fails() {
        let mut builder = TestBuilder::new();

        builder.transformed_value_array_gte[2] = 1;

        builder.execute();
    }

    #[test(should_fail_with="array should be padded with empty items")]
    fn assert_split_transformed_value_arrays_end_with_non_empty_item_in_lt_fails() {
        let mut builder = TestBuilder::new();

        builder.transformed_value_array_lt[NUM_TEST_ITEMS - 1] = 1;

        builder.execute();
    }

    #[test(should_fail_with="array should be padded with empty items")]
    fn assert_split_transformed_value_arrays_end_with_non_empty_item_in_gte_fails() {
        let mut builder = TestBuilder::new();

        builder.transformed_value_array_gte[NUM_TEST_ITEMS - 1] = 1;

        builder.execute();
    }
}
