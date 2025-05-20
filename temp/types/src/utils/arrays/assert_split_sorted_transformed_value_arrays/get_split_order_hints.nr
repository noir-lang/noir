use crate::{
    abis::side_effect::Ordered,
    traits::{Empty, is_empty},
    utils::arrays::{
        get_sorted_tuple::get_sorted_tuple,
        sort_by_counter::{
            compare_by_counter_empty_padded_asc, compare_by_counter_empty_padded_desc,
        },
    },
};

pub struct SplitOrderHints<let N: u32> {
    pub sorted_counters_lt: [u32; N],
    pub sorted_counters_gte: [u32; N],
    pub sorted_indexes: [u32; N],
}

impl<let N: u32> SplitOrderHints<N> {
    pub fn empty() -> SplitOrderHints<N> {
        SplitOrderHints {
            sorted_counters_lt: [0; N],
            sorted_counters_gte: [0; N],
            sorted_indexes: [0; N],
        }
    }
}

impl<let N: u32> Eq for SplitOrderHints<N> {
    fn eq(self, other: Self) -> bool {
        (self.sorted_counters_lt == other.sorted_counters_lt)
            & (self.sorted_counters_gte == other.sorted_counters_gte)
            & (self.sorted_indexes == other.sorted_indexes)
    }
}

unconstrained fn get_split_order_hints<T, let N: u32>(
    array: [T; N],
    split_counter: u32,
    ascending: bool,
) -> SplitOrderHints<N>
where
    T: Ordered + Eq + Empty,
{
    let ordering = if ascending {
        compare_by_counter_empty_padded_asc
    } else {
        compare_by_counter_empty_padded_desc
    };
    let sorted_tuples = get_sorted_tuple(array, ordering);

    let mut sorted_counters_lt = [0; N];
    let mut sorted_counters_gte = [0; N];
    let mut sorted_indexes = [0; N];
    let mut num_lt = 0;
    let mut num_gte = 0;
    let mut found_split = false;
    for i in 0..N {
        let elem = sorted_tuples[i].elem;
        if !is_empty(elem) {
            let is_gte = (elem.counter() >= split_counter) & (elem.counter() != 0);
            found_split |= ascending == is_gte;
            let mut index_offet = 0;
            if found_split != ascending {
                sorted_counters_lt[num_lt] = elem.counter();
                num_lt += 1;
                index_offet = num_gte;
            } else {
                sorted_counters_gte[num_gte] = elem.counter();
                num_gte += 1;
                index_offet = num_lt;
            }
            let original_index = sorted_tuples[i].original_index;
            sorted_indexes[original_index] = if !found_split { i } else { i - index_offet };
        }
    }

    SplitOrderHints { sorted_counters_lt, sorted_counters_gte, sorted_indexes }
}

pub unconstrained fn get_split_order_hints_asc<T, let N: u32>(
    array: [T; N],
    split_counter: u32,
) -> SplitOrderHints<N>
where
    T: Ordered + Eq + Empty,
{
    get_split_order_hints(array, split_counter, true)
}

pub unconstrained fn get_split_order_hints_desc<T, let N: u32>(
    array: [T; N],
    split_counter: u32,
) -> SplitOrderHints<N>
where
    T: Ordered + Eq + Empty,
{
    get_split_order_hints(array, split_counter, false)
}

mod tests {
    use crate::{
        tests::{types::TestValue, utils::pad_end},
        traits::Empty,
        utils::arrays::assert_split_sorted_transformed_value_arrays::get_split_order_hints::{
            get_split_order_hints_asc, get_split_order_hints_desc, SplitOrderHints,
        },
    };

    global full_array: [TestValue; 5] = [
        TestValue { value: 100, counter: 11 },
        TestValue { value: 200, counter: 17 },
        TestValue { value: 300, counter: 7 },
        TestValue { value: 400, counter: 5 },
        TestValue { value: 500, counter: 13 },
    ];

    struct TestBuilder<let N: u32> {
        array: [TestValue; N],
        split_counter: u32,
    }

    impl TestBuilder<5> {
        pub fn new() -> Self {
            TestBuilder { array: full_array, split_counter: 0 }
        }
    }

    impl TestBuilder<7> {
        pub fn new_padded() -> Self {
            TestBuilder { array: pad_end(full_array, TestValue::empty()), split_counter: 0 }
        }
    }

    impl<let N: u32> TestBuilder<N> {
        pub unconstrained fn asc_to_equal(self, expected: SplitOrderHints<N>) {
            let hints = get_split_order_hints_asc(self.array, self.split_counter);
            assert_eq(hints, expected);
        }

        pub unconstrained fn desc_to_equal(self, expected: SplitOrderHints<N>) {
            let hints = get_split_order_hints_desc(self.array, self.split_counter);
            assert_eq(hints, expected);
        }
    }

    // asc

    #[test]
    unconstrained fn get_split_order_hints_asc_zero_split_counter_full() {
        let builder = TestBuilder::new();
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [0, 0, 0, 0, 0],
            sorted_counters_gte: [5, 7, 11, 13, 17],
            sorted_indexes: [2, 4, 1, 0, 3],
        };
        builder.asc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_split_order_hints_asc_non_zero_split_counter_full() {
        let mut builder = TestBuilder::new();
        builder.split_counter = 9;

        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [5, 7, 0, 0, 0],
            sorted_counters_gte: [11, 13, 17, 0, 0],
            sorted_indexes: [0, 2, 1, 0, 1],
        };
        builder.asc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_split_order_hints_asc_non_zero_split_counter_equal_full() {
        let mut builder = TestBuilder::new();
        builder.split_counter = 11;

        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [5, 7, 0, 0, 0],
            sorted_counters_gte: [11, 13, 17, 0, 0],
            sorted_indexes: [0, 2, 1, 0, 1],
        };
        builder.asc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_split_order_hints_asc_zero_split_counter_padded_empty() {
        let builder = TestBuilder::new_padded();

        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [0, 0, 0, 0, 0, 0, 0],
            sorted_counters_gte: [5, 7, 11, 13, 17, 0, 0],
            sorted_indexes: [2, 4, 1, 0, 3, 0, 0],
        };
        builder.asc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_split_order_hints_asc_non_zero_split_counter_padded_empty() {
        let mut builder = TestBuilder::new_padded();
        builder.split_counter = 9;

        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [5, 7, 0, 0, 0, 0, 0],
            sorted_counters_gte: [11, 13, 17, 0, 0, 0, 0],
            sorted_indexes: [0, 2, 1, 0, 1, 0, 0],
        };
        builder.asc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_split_order_hints_asc_non_zero_split_counter_equal_padded_empty() {
        let mut builder = TestBuilder::new_padded();
        builder.split_counter = 11;

        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [5, 7, 0, 0, 0, 0, 0],
            sorted_counters_gte: [11, 13, 17, 0, 0, 0, 0],
            sorted_indexes: [0, 2, 1, 0, 1, 0, 0],
        };
        builder.asc_to_equal(expected_hints);
    }

    // desc

    #[test]
    unconstrained fn get_split_order_hints_desc_zero_split_counter_empty() {
        let builder = TestBuilder::new();
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [0, 0, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 7, 5],
            sorted_indexes: [2, 0, 3, 4, 1],
        };
        builder.desc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_split_order_hints_desc_non_zero_split_counter_empty() {
        let mut builder = TestBuilder::new();
        builder.split_counter = 9;

        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [7, 5, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 0, 0],
            sorted_indexes: [2, 0, 0, 1, 1],
        };
        builder.desc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_split_order_hints_desc_non_zero_split_counter_equal_empty() {
        let mut builder = TestBuilder::new();
        builder.split_counter = 11;

        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [7, 5, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 0, 0],
            sorted_indexes: [2, 0, 0, 1, 1],
        };
        builder.desc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_split_order_hints_desc_zero_split_counter_padded_empty() {
        let builder = TestBuilder::new_padded();
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [0, 0, 0, 0, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 7, 5, 0, 0],
            sorted_indexes: [2, 0, 3, 4, 1, 0, 0],
        };
        builder.desc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_split_order_hints_desc_non_zero_split_counter_padded_empty() {
        let mut builder = TestBuilder::new_padded();
        builder.split_counter = 9;

        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [7, 5, 0, 0, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 0, 0, 0, 0],
            sorted_indexes: [2, 0, 0, 1, 1, 0, 0],
        };
        builder.desc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_split_order_hints_desc_non_zero_split_counter_equal_padded_empty() {
        let mut builder = TestBuilder::new_padded();
        builder.split_counter = 11;

        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [7, 5, 0, 0, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 0, 0, 0, 0],
            sorted_indexes: [2, 0, 0, 1, 1, 0, 0],
        };
        builder.desc_to_equal(expected_hints);
    }
}
