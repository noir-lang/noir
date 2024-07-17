use crate::{
    abis::side_effect::Ordered, traits::{Empty, is_empty},
    utils::arrays::{
    sort_by_counters::{order_by_counters_empty_padded_asc, order_by_counters_empty_padded_desc},
    sort_get_sorted_tuple::sort_get_sorted_tuple
}
};

struct SplitOrderHints<let N: u32> {
    sorted_counters_lt: [u32; N],
    sorted_counters_gte: [u32; N],
    sorted_indexes: [u32; N],
}

impl<let N: u32> SplitOrderHints<N> {
    pub fn empty() -> SplitOrderHints<N> {
        SplitOrderHints { sorted_counters_lt: [0; N], sorted_counters_gte: [0; N], sorted_indexes: [0; N] }
    }
}

impl<let N: u32> Eq for SplitOrderHints<N> {
    fn eq(self, other: Self) -> bool {
        (self.sorted_counters_lt == other.sorted_counters_lt)
            & (self.sorted_counters_gte == other.sorted_counters_gte)
            & (self.sorted_indexes == other.sorted_indexes)
    }
}

fn sort_get_split_order_hints<T, let N: u32>(
    array: [T; N],
    split_counter: u32,
    ascending: bool
) -> SplitOrderHints<N> where T: Ordered + Eq + Empty {
    let ordering = if ascending {
        order_by_counters_empty_padded_asc
    } else {
        order_by_counters_empty_padded_desc
    };
    let sorted_tuples = sort_get_sorted_tuple(array, ordering);

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

pub fn sort_get_split_order_hints_asc<T, S, let N: u32>(
    array: [T; N],
    split_counter: u32
) -> SplitOrderHints<N> where T: Ordered + Eq + Empty {
    sort_get_split_order_hints(array, split_counter, true)
}

pub fn sort_get_split_order_hints_desc<T, S, let N: u32>(
    array: [T; N],
    split_counter: u32
) -> SplitOrderHints<N> where T: Ordered + Eq + Empty {
    sort_get_split_order_hints(array, split_counter, false)
}

mod tests {
    use crate::{
        abis::side_effect::Ordered, traits::Empty,
        utils::arrays::sort_get_split_order_hints::{sort_get_split_order_hints_asc, sort_get_split_order_hints_desc, SplitOrderHints}
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

    global full_array = [
        TestItem { value: 100, counter: 11 },
        TestItem { value: 200, counter: 17 },
        TestItem { value: 300, counter: 7 },
        TestItem { value: 400, counter: 5 },
        TestItem { value: 500, counter: 13 }
    ];

    global padded_array = [
        TestItem { value: 100, counter: 11 },
        TestItem { value: 200, counter: 17 },
        TestItem { value: 300, counter: 7 },
        TestItem { value: 400, counter: 5 },
        TestItem { value: 500, counter: 13 },
        TestItem::empty(),
        TestItem::empty()
    ];

    // asc

    #[test]
    fn sort_get_split_order_hints_asc_zero_split_counter_full() {
        let split_counter = 0;
        let hints = sort_get_split_order_hints_asc(full_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [0, 0, 0, 0, 0],
            sorted_counters_gte: [5, 7, 11, 13, 17],
            sorted_indexes: [2, 4, 1, 0, 3]
        };
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_split_order_hints_asc_non_zero_split_counter_full() {
        let split_counter = 9;
        let hints = sort_get_split_order_hints_asc(full_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [5, 7, 0, 0, 0],
            sorted_counters_gte: [11, 13, 17, 0, 0],
            sorted_indexes: [0, 2, 1, 0, 1]
        };
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_split_order_hints_asc_non_zero_split_counter_equal_full() {
        let split_counter = 11; // Equal one of the item's counter.
        let hints = sort_get_split_order_hints_asc(full_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [5, 7, 0, 0, 0],
            sorted_counters_gte: [11, 13, 17, 0, 0],
            sorted_indexes: [0, 2, 1, 0, 1]
        };
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_split_order_hints_asc_zero_split_counter_padded_empty() {
        let split_counter = 0;
        let hints = sort_get_split_order_hints_asc(padded_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [0, 0, 0, 0, 0, 0, 0],
            sorted_counters_gte: [5, 7, 11, 13, 17, 0, 0],
            sorted_indexes: [2, 4, 1, 0, 3, 0, 0]
        };
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_split_order_hints_asc_non_zero_split_counter_padded_empty() {
        let split_counter = 9;
        let hints = sort_get_split_order_hints_asc(padded_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [5, 7, 0, 0, 0, 0, 0],
            sorted_counters_gte: [11, 13, 17, 0, 0, 0, 0],
            sorted_indexes: [0, 2, 1, 0, 1, 0, 0]
        };
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_split_order_hints_asc_non_zero_split_counter_equal_padded_empty() {
        let split_counter = 11;
        let hints = sort_get_split_order_hints_asc(padded_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [5, 7, 0, 0, 0, 0, 0],
            sorted_counters_gte: [11, 13, 17, 0, 0, 0, 0],
            sorted_indexes: [0, 2, 1, 0, 1, 0, 0]
        };
        assert_eq(hints, expected_hints);
    }

    // desc

    #[test]
    fn sort_get_split_order_hints_desc_zero_split_counter_empty() {
        let split_counter = 0;
        let hints = sort_get_split_order_hints_desc(full_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [0, 0, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 7, 5],
            sorted_indexes: [2, 0, 3, 4, 1]
        };
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_split_order_hints_desc_non_zero_split_counter_empty() {
        let split_counter = 9;
        let hints = sort_get_split_order_hints_desc(full_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [7, 5, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 0, 0],
            sorted_indexes: [2, 0, 0, 1, 1]
        };
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_split_order_hints_desc_non_zero_split_counter_equal_empty() {
        let split_counter = 11;
        let hints = sort_get_split_order_hints_desc(full_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [7, 5, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 0, 0],
            sorted_indexes: [2, 0, 0, 1, 1]
        };
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_split_order_hints_desc_zero_split_counter_padded_empty() {
        let split_counter = 0;
        let hints = sort_get_split_order_hints_desc(padded_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [0, 0, 0, 0, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 7, 5, 0, 0],
            sorted_indexes: [2, 0, 3, 4, 1, 0, 0]
        };
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_split_order_hints_desc_non_zero_split_counter_padded_empty() {
        let split_counter = 9;
        let hints = sort_get_split_order_hints_desc(padded_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [7, 5, 0, 0, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 0, 0, 0, 0],
            sorted_indexes: [2, 0, 0, 1, 1, 0, 0]
        };
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_split_order_hints_desc_non_zero_split_counter_equal_padded_empty() {
        let split_counter = 11;
        let hints = sort_get_split_order_hints_desc(padded_array, split_counter);
        let expected_hints = SplitOrderHints {
            sorted_counters_lt: [7, 5, 0, 0, 0, 0, 0],
            sorted_counters_gte: [17, 13, 11, 0, 0, 0, 0],
            sorted_indexes: [2, 0, 0, 1, 1, 0, 0]
        };
        assert_eq(hints, expected_hints);
    }
}

