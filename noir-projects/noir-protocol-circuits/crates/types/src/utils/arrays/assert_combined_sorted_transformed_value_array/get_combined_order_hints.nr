use crate::{
    abis::side_effect::Ordered, traits::{Empty, is_empty},
    utils::arrays::{array_length, sort_by_counter::compare_by_counter_empty_padded_asc, get_sorted_tuple::get_sorted_tuple}
};

struct CombinedOrderHint {
    counter: u32,
    original_index: u32,
}

impl CombinedOrderHint {
    pub fn empty() -> Self {
        CombinedOrderHint { counter: 0, original_index: 0 }
    }
}

impl Eq for CombinedOrderHint {
    fn eq(self, other: Self) -> bool {
        (self.counter == other.counter) & (self.original_index == other.original_index)
    }
}

unconstrained fn count_private_items<T, let N: u32>(array: [T; N]) -> u32 where T: Ordered + Empty + Eq {
    let mut length = 0;
    for item in array {
        if !is_empty(item) & (item.counter() == 0) {
            length += 1;
        }
    }
    length
}

pub fn get_combined_order_hints_asc<T, let N: u32>(
    array_lt: [T; N],
    array_gte: [T; N]
) -> [CombinedOrderHint; N] where T: Ordered + Eq + Empty {
    let mut hints = [CombinedOrderHint::empty(); N];
    let sorted_lt = get_sorted_tuple(array_lt, compare_by_counter_empty_padded_asc);
    let sorted_gte = get_sorted_tuple(array_gte, compare_by_counter_empty_padded_asc);
    let num_private_lt = count_private_items(array_lt);
    let num_private_gte = count_private_items(array_gte);
    let num_lt = array_length(array_lt);
    let total_private = num_private_lt + num_private_gte;
    let total_non_public_gte = num_lt + num_private_gte;
    for i in 0..N {
        let sorted = if i < num_private_lt {
            sorted_lt[i]
        } else if i < total_private {
            sorted_gte[i - num_private_lt]
        } else if i < total_non_public_gte {
            sorted_lt[i - num_private_gte]
        } else {
            sorted_gte[i - num_lt]
        };
        hints[i].original_index = sorted.original_index;
        hints[i].counter = sorted.elem.counter();
    }
    hints
}

mod tests {
    use crate::{
        abis::side_effect::Ordered, tests::utils::pad_end, traits::Empty,
        utils::arrays::assert_combined_sorted_transformed_value_array::get_combined_order_hints::{CombinedOrderHint, get_combined_order_hints_asc}
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

    #[test]
    fn get_combined_order_hints_asc_full_non_empty() {
        let array_lt = pad_end(
            [
            TestItem { value: 600, counter: 9 },
            TestItem { value: 400, counter: 3 },
            TestItem { value: 500, counter: 6 }
        ],
            TestItem::empty()
        );
        let array_gte = pad_end(
            [
            TestItem { value: 200, counter: 13 },
            TestItem { value: 100, counter: 19 },
            TestItem { value: 300, counter: 16 }
        ],
            TestItem::empty()
        );
        let expected_hints = [
            CombinedOrderHint { counter: 3, original_index: 1 },
            CombinedOrderHint { counter: 6, original_index: 2 },
            CombinedOrderHint { counter: 9, original_index: 0 },
            CombinedOrderHint { counter: 13, original_index: 0 },
            CombinedOrderHint { counter: 16, original_index: 2 },
            CombinedOrderHint { counter: 19, original_index: 1 }
        ];
        assert_eq(get_combined_order_hints_asc(array_lt, array_gte), expected_hints);
    }

    #[test]
    fn get_combined_order_hints_asc_padded_empty() {
        let array_lt = pad_end(
            [
            TestItem { value: 500, counter: 6 },
            TestItem { value: 400, counter: 3 }
        ],
            TestItem::empty()
        );
        let array_gte = pad_end([TestItem { value: 100, counter: 19 }], TestItem::empty());
        let expected_hints = [
            CombinedOrderHint { counter: 3, original_index: 1 },
            CombinedOrderHint { counter: 6, original_index: 0 },
            CombinedOrderHint { counter: 19, original_index: 0 },
            CombinedOrderHint { counter: 0, original_index: 1 },
            CombinedOrderHint { counter: 0, original_index: 2 }
        ];
        assert_eq(get_combined_order_hints_asc(array_lt, array_gte), expected_hints);
    }

    #[test]
    fn get_combined_order_hints_asc_lt_empty() {
        let array_lt = [TestItem::empty(); 4];
        let array_gte = pad_end(
            [
            TestItem { value: 200, counter: 13 },
            TestItem { value: 100, counter: 19 }
        ],
            TestItem::empty()
        );
        let expected_hints = [
            CombinedOrderHint { counter: 13, original_index: 0 },
            CombinedOrderHint { counter: 19, original_index: 1 },
            CombinedOrderHint { counter: 0, original_index: 2 },
            CombinedOrderHint { counter: 0, original_index: 3 }
        ];
        assert_eq(get_combined_order_hints_asc(array_lt, array_gte), expected_hints);
    }

    #[test]
    fn get_combined_order_hints_asc_gte_empty() {
        let array_lt = pad_end(
            [
            TestItem { value: 400, counter: 3 },
            TestItem { value: 500, counter: 6 }
        ],
            TestItem::empty()
        );
        let array_gte = [TestItem::empty(); 4];
        let expected_hints = [
            CombinedOrderHint { counter: 3, original_index: 0 },
            CombinedOrderHint { counter: 6, original_index: 1 },
            CombinedOrderHint { counter: 0, original_index: 0 },
            CombinedOrderHint { counter: 0, original_index: 1 }
        ];
        assert_eq(get_combined_order_hints_asc(array_lt, array_gte), expected_hints);
    }

    #[test]
    fn get_combined_order_hints_asc_all_empty() {
        let array_lt = [TestItem::empty(); 3];
        let array_gte = [TestItem::empty(); 3];
        let expected_hints = [
            CombinedOrderHint { counter: 0, original_index: 0 },
            CombinedOrderHint { counter: 0, original_index: 1 },
            CombinedOrderHint { counter: 0, original_index: 2 }
        ];
        assert_eq(get_combined_order_hints_asc(array_lt, array_gte), expected_hints);
    }

    #[test]
    fn get_combined_order_hints_asc_prepended_zero_counters() {
        let array_lt = pad_end(
            [
            TestItem { value: 700, counter: 0 },
            TestItem { value: 500, counter: 0 },
            TestItem { value: 100, counter: 16 },
            TestItem { value: 400, counter: 13 }
        ],
            TestItem::empty()
        );
        let array_gte = pad_end(
            [
            TestItem { value: 300, counter: 0 },
            TestItem { value: 600, counter: 0 },
            TestItem { value: 200, counter: 19 }
        ],
            TestItem::empty()
        );
        let expected_hints = [
            CombinedOrderHint { counter: 0, original_index: 0 },
            CombinedOrderHint { counter: 0, original_index: 1 },
            CombinedOrderHint { counter: 0, original_index: 0 },
            CombinedOrderHint { counter: 0, original_index: 1 },
            CombinedOrderHint { counter: 13, original_index: 3 },
            CombinedOrderHint { counter: 16, original_index: 2 },
            CombinedOrderHint { counter: 19, original_index: 2 },
            CombinedOrderHint { counter: 0, original_index: 3 },
            CombinedOrderHint { counter: 0, original_index: 4 },
            CombinedOrderHint { counter: 0, original_index: 5 }
        ];
        assert_eq(get_combined_order_hints_asc(array_lt, array_gte), expected_hints);
    }

    #[test]
    fn get_combined_order_hints_asc_prepended_zero_counters_lt_empty() {
        let array_lt = [TestItem::empty(); 6];
        let array_gte = pad_end(
            [
            TestItem { value: 300, counter: 0 },
            TestItem { value: 600, counter: 0 },
            TestItem { value: 200, counter: 19 },
            TestItem { value: 400, counter: 13 }
        ],
            TestItem::empty()
        );
        let expected_hints = [
            CombinedOrderHint { counter: 0, original_index: 0 },
            CombinedOrderHint { counter: 0, original_index: 1 },
            CombinedOrderHint { counter: 13, original_index: 3 },
            CombinedOrderHint { counter: 19, original_index: 2 },
            CombinedOrderHint { counter: 0, original_index: 4 },
            CombinedOrderHint { counter: 0, original_index: 5 }
        ];
        assert_eq(get_combined_order_hints_asc(array_lt, array_gte), expected_hints);
    }

    #[test]
    fn get_combined_order_hints_asc_prepended_zero_counters_gte_empty() {
        let array_lt = pad_end(
            [
            TestItem { value: 300, counter: 0 },
            TestItem { value: 600, counter: 0 },
            TestItem { value: 200, counter: 19 },
            TestItem { value: 400, counter: 13 }
        ],
            TestItem::empty()
        );
        let array_gte = [TestItem::empty(); 6];
        let expected_hints = [
            CombinedOrderHint { counter: 0, original_index: 0 },
            CombinedOrderHint { counter: 0, original_index: 1 },
            CombinedOrderHint { counter: 13, original_index: 3 },
            CombinedOrderHint { counter: 19, original_index: 2 },
            CombinedOrderHint { counter: 0, original_index: 0 },
            CombinedOrderHint { counter: 0, original_index: 1 }
        ];
        assert_eq(get_combined_order_hints_asc(array_lt, array_gte), expected_hints);
    }
}

