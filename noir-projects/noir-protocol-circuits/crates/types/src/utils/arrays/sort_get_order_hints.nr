use crate::{
    abis::side_effect::Ordered, traits::{Empty, is_empty},
    utils::arrays::{
    sort_by_counters::{order_by_counters_empty_padded_asc, order_by_counters_empty_padded_desc},
    sort_get_sorted_tuple::sort_get_sorted_tuple
}
};

struct OrderHint {
    counter: u32,
    sorted_index: u32,
}

impl OrderHint {
    pub fn empty() -> Self {
        OrderHint { counter: 0, sorted_index: 0 }
    }
}

impl Eq for OrderHint {
    fn eq(self, other: Self) -> bool {
        (self.counter == other.counter) & (self.sorted_index == other.sorted_index)
    }
}

pub fn sort_get_order_hints<T, N>(
    array: [T; N],
    ordering: fn(T, T) -> bool
) -> [OrderHint; N] where T: Ordered + Eq + Empty {
    let sorted_tuples = sort_get_sorted_tuple(array, ordering);

    let mut hints = [OrderHint::empty(); N];
    for i in 0..N {
        let elem = sorted_tuples[i].elem;
        hints[i].counter = elem.counter();
        if !is_empty(elem) {
            let original_index = sorted_tuples[i].original_index;
            hints[original_index].sorted_index = i;
        }
    }

    hints
}

pub fn sort_get_order_hints_asc<T, S, N>(array: [T; N]) -> [OrderHint; N] where T: Ordered + Eq + Empty {
    sort_get_order_hints(array, order_by_counters_empty_padded_asc)
}

pub fn sort_get_order_hints_desc<T, S, N>(array: [T; N]) -> [OrderHint; N] where T: Ordered + Eq + Empty {
    sort_get_order_hints(array, order_by_counters_empty_padded_desc)
}

mod tests {
    use crate::{
        abis::side_effect::Ordered, traits::Empty,
        utils::arrays::sort_get_order_hints::{OrderHint, sort_get_order_hints_asc, sort_get_order_hints_desc}
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
    fn sort_get_order_hints_asc_full_non_empty() {
        let array = [
            TestItem { value: 100, counter: 9 },
            TestItem { value: 200, counter: 3 },
            TestItem { value: 300, counter: 6 }
        ];
        let hints = sort_get_order_hints_asc(array);
        let expected_hints = [
            OrderHint { counter: 3, sorted_index: 2 },
            OrderHint { counter: 6, sorted_index: 0 },
            OrderHint { counter: 9, sorted_index: 1 }
        ];
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_order_hints_asc_padded_empty() {
        let array = [
            TestItem { value: 100, counter: 9 },
            TestItem { value: 200, counter: 3 },
            TestItem { value: 300, counter: 6 },
            TestItem::empty(),
            TestItem::empty()
        ];
        let hints = sort_get_order_hints_asc(array);
        let expected_hints = [
            OrderHint { counter: 3, sorted_index: 2 },
            OrderHint { counter: 6, sorted_index: 0 },
            OrderHint { counter: 9, sorted_index: 1 },
            OrderHint { counter: 0, sorted_index: 0 },
            OrderHint { counter: 0, sorted_index: 0 }
        ];
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_order_hints_desc_full_non_empty() {
        let array = [
            TestItem { value: 100, counter: 9 },
            TestItem { value: 200, counter: 3 },
            TestItem { value: 300, counter: 6 }
        ];
        let hints = sort_get_order_hints_desc(array);
        let expected_hints = [
            OrderHint { counter: 9, sorted_index: 0 },
            OrderHint { counter: 6, sorted_index: 2 },
            OrderHint { counter: 3, sorted_index: 1 }
        ];
        assert_eq(hints, expected_hints);
    }

    #[test]
    fn sort_get_order_hints_desc_padded_empty() {
        let array = [
            TestItem { value: 100, counter: 9 },
            TestItem { value: 200, counter: 3 },
            TestItem { value: 300, counter: 6 },
            TestItem::empty(),
            TestItem::empty()
        ];
        let hints = sort_get_order_hints_desc(array);
        let expected_hints = [
            OrderHint { counter: 9, sorted_index: 0 },
            OrderHint { counter: 6, sorted_index: 2 },
            OrderHint { counter: 3, sorted_index: 1 },
            OrderHint { counter: 0, sorted_index: 0 },
            OrderHint { counter: 0, sorted_index: 0 }
        ];
        assert_eq(hints, expected_hints);
    }
}

