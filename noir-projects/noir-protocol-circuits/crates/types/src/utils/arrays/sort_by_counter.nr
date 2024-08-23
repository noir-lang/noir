use crate::{abis::side_effect::Ordered, traits::{Empty, is_empty}, utils::arrays::sort_by::sort_by};

pub fn compare_by_counter_empty_padded_asc<T>(a: T, b: T) -> bool where T: Ordered + Empty + Eq {
    !is_empty(a) & (is_empty(b) | (a.counter() < b.counter()))
}

pub fn compare_by_counter_empty_padded_desc<T>(a: T, b: T) -> bool where T: Ordered + Empty + Eq {
    !is_empty(a) & (is_empty(b) | (a.counter() >= b.counter()))
}

pub fn sort_by_counter_asc<T, let N: u32>(array: [T; N]) -> [T; N] where T: Ordered + Empty + Eq {
    sort_by(array, compare_by_counter_empty_padded_asc)
}

pub fn sort_by_counter_desc<T, let N: u32>(array: [T; N]) -> [T; N] where T: Ordered + Empty + Eq {
    sort_by(array, compare_by_counter_empty_padded_desc)
}

mod tests {
    use crate::{
        abis::side_effect::Ordered, traits::Empty,
        utils::arrays::sort_by_counter::{
        compare_by_counter_empty_padded_asc, compare_by_counter_empty_padded_desc, sort_by_counter_asc,
        sort_by_counter_desc
    }
    };

    struct TestItem {
        value: u32,
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

    fn sort_by_values_asc<let N: u32>(values: [u32; N]) -> [u32; N] {
        let items = values.map(|value| TestItem { value, counter: value });
        sort_by_counter_asc(items).map(|item: TestItem| item.value)
    }

    fn sort_by_values_desc<let N: u32>(values: [u32; N]) -> [u32; N] {
        let items = values.map(|value| TestItem { value, counter: value });
        sort_by_counter_desc(items).map(|item: TestItem| item.value)
    }

    fn compare_test_items_asc(value_1: u32, value_2: u32) -> bool {
        compare_by_counter_empty_padded_asc(
            TestItem { value: value_1, counter: value_1 },
            TestItem { value: value_2, counter: value_2 }
        )
    }

    fn compare_test_items_desc(value_1: u32, value_2: u32) -> bool {
        compare_by_counter_empty_padded_desc(
            TestItem { value: value_1, counter: value_1 },
            TestItem { value: value_2, counter: value_2 }
        )
    }

    #[test]
    fn compare_by_counter_empty_padded_asc_expected() {
        assert_eq(compare_test_items_asc(1, 2), true);
        assert_eq(compare_test_items_asc(1, 1), false);
        assert_eq(compare_test_items_asc(2, 1), false);
        assert_eq(compare_test_items_asc(0, 0), false);
    }

    #[test]
    fn compare_by_counter_empty_padded_desc_expected() {
        assert_eq(compare_test_items_desc(1, 2), false);
        assert_eq(compare_test_items_desc(1, 1), true);
        assert_eq(compare_test_items_desc(2, 1), true);
        assert_eq(compare_test_items_desc(0, 0), false);
    }

    #[test]
    fn sort_by_counter_asc_full_non_empty() {
        let sorted = sort_by_values_asc([4, 2, 1, 3, 5]);
        assert_eq(sorted, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn sort_by_counter_desc_full_non_empty() {
        let sorted = sort_by_values_desc([4, 2, 1, 3, 5]);
        assert_eq(sorted, [5, 4, 3, 2, 1]);
    }

    #[test]
    fn sort_by_counter_asc_padded_empty() {
        let sorted = sort_by_values_asc([4, 2, 0, 0, 1, 0, 3, 5]);
        assert_eq(sorted, [1, 2, 3, 4, 5, 0, 0, 0]);
    }

    #[test]
    fn sort_by_counter_desc_padded_empty() {
        let sorted = sort_by_values_desc([4, 2, 0, 0, 1, 0, 3, 5]);
        assert_eq(sorted, [5, 4, 3, 2, 1, 0, 0, 0]);
    }

    #[test]
    fn sort_by_counter_asc_with_zero_counters() {
        let original = [
            TestItem { value: 55, counter: 1 },
            TestItem { value: 11, counter: 0 },
            TestItem { value: 33, counter: 2 },
            TestItem { value: 44, counter: 0 },
            TestItem { value: 22, counter: 0 },
            TestItem::empty(),
            TestItem::empty()
        ];
        let expected = [
            TestItem { value: 11, counter: 0 },
            TestItem { value: 44, counter: 0 },
            TestItem { value: 22, counter: 0 },
            TestItem { value: 55, counter: 1 },
            TestItem { value: 33, counter: 2 },
            TestItem::empty(),
            TestItem::empty()
        ];
        assert_eq(sort_by_counter_asc(original), expected);
    }

    #[test]
    fn sort_by_counter_desc_with_zero_counters() {
        let original = [
            TestItem { value: 55, counter: 1 },
            TestItem { value: 11, counter: 0 },
            TestItem { value: 33, counter: 2 },
            TestItem { value: 44, counter: 0 },
            TestItem { value: 22, counter: 0 },
            TestItem::empty(),
            TestItem::empty()
        ];
        let expected = [
            TestItem { value: 33, counter: 2 },
            TestItem { value: 55, counter: 1 },
            TestItem { value: 22, counter: 0 },
            TestItem { value: 44, counter: 0 },
            TestItem { value: 11, counter: 0 },
            TestItem::empty(),
            TestItem::empty()
        ];
        assert_eq(sort_by_counter_desc(original), expected);
    }
}
