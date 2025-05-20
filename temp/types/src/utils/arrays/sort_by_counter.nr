use crate::{abis::side_effect::Ordered, traits::{Empty, is_empty}, utils::arrays::sort_by::sort_by};

pub fn compare_by_counter_empty_padded_asc<T>(a: T, b: T) -> bool
where
    T: Ordered + Empty + Eq,
{
    !is_empty(a) & (is_empty(b) | (a.counter() < b.counter()))
}

pub fn compare_by_counter_empty_padded_desc<T>(a: T, b: T) -> bool
where
    T: Ordered + Empty + Eq,
{
    !is_empty(a) & (is_empty(b) | (a.counter() >= b.counter()))
}

pub unconstrained fn sort_by_counter_asc<T, let N: u32>(array: [T; N]) -> [T; N]
where
    T: Ordered + Empty + Eq,
{
    sort_by(array, compare_by_counter_empty_padded_asc)
}

pub unconstrained fn sort_by_counter_desc<T, let N: u32>(array: [T; N]) -> [T; N]
where
    T: Ordered + Empty + Eq,
{
    sort_by(array, compare_by_counter_empty_padded_desc)
}

mod tests {
    use crate::{
        tests::types::TestValue,
        traits::Empty,
        utils::arrays::sort_by_counter::{
            compare_by_counter_empty_padded_asc, compare_by_counter_empty_padded_desc,
            sort_by_counter_asc, sort_by_counter_desc,
        },
    };

    unconstrained fn asc_values_to_equal<let N: u32>(values: [u32; N], expected: [u32; N]) {
        let items = values.map(|value| TestValue { value: value as Field, counter: value });
        let sorted = sort_by_counter_asc(items).map(|item: TestValue| item.counter);
        assert_eq(sorted, expected);
    }

    unconstrained fn desc_values_to_equal<let N: u32>(values: [u32; N], expected: [u32; N]) {
        let items = values.map(|value| TestValue { value: value as Field, counter: value });
        let sorted = sort_by_counter_desc(items).map(|item: TestValue| item.counter);
        assert_eq(sorted, expected);
    }

    unconstrained fn compare_test_items_asc(value_1: u32, value_2: u32) -> bool {
        compare_by_counter_empty_padded_asc(
            TestValue { value: value_1 as Field, counter: value_1 },
            TestValue { value: value_2 as Field, counter: value_2 },
        )
    }

    unconstrained fn compare_test_items_desc(value_1: u32, value_2: u32) -> bool {
        compare_by_counter_empty_padded_desc(
            TestValue { value: value_1 as Field, counter: value_1 },
            TestValue { value: value_2 as Field, counter: value_2 },
        )
    }

    #[test]
    unconstrained fn compare_by_counter_empty_padded_asc_expected() {
        assert_eq(compare_test_items_asc(1, 2), true);
        assert_eq(compare_test_items_asc(1, 1), false);
        assert_eq(compare_test_items_asc(2, 1), false);
        assert_eq(compare_test_items_asc(0, 0), false);
    }

    #[test]
    unconstrained fn compare_by_counter_empty_padded_desc_expected() {
        assert_eq(compare_test_items_desc(1, 2), false);
        assert_eq(compare_test_items_desc(1, 1), true);
        assert_eq(compare_test_items_desc(2, 1), true);
        assert_eq(compare_test_items_desc(0, 0), false);
    }

    #[test]
    unconstrained fn sort_by_counter_asc_full_non_empty() {
        asc_values_to_equal([4, 2, 1, 3, 5], [1, 2, 3, 4, 5]);
    }

    #[test]
    unconstrained fn sort_by_counter_desc_full_non_empty() {
        desc_values_to_equal([4, 2, 1, 3, 5], [5, 4, 3, 2, 1]);
    }

    #[test]
    unconstrained fn sort_by_counter_asc_padded_empty() {
        asc_values_to_equal([4, 2, 0, 0, 1, 0, 3, 5], [1, 2, 3, 4, 5, 0, 0, 0]);
    }

    #[test]
    unconstrained fn sort_by_counter_desc_padded_empty() {
        desc_values_to_equal([4, 2, 0, 0, 1, 0, 3, 5], [5, 4, 3, 2, 1, 0, 0, 0]);
    }

    #[test]
    unconstrained fn sort_by_counter_asc_with_zero_counters() {
        let original = [
            TestValue { value: 55, counter: 1 },
            TestValue { value: 11, counter: 0 },
            TestValue { value: 33, counter: 2 },
            TestValue { value: 44, counter: 0 },
            TestValue { value: 22, counter: 0 },
            TestValue::empty(),
            TestValue::empty(),
        ];
        let expected = [
            TestValue { value: 11, counter: 0 },
            TestValue { value: 44, counter: 0 },
            TestValue { value: 22, counter: 0 },
            TestValue { value: 55, counter: 1 },
            TestValue { value: 33, counter: 2 },
            TestValue::empty(),
            TestValue::empty(),
        ];
        let sorted = sort_by_counter_asc(original);
        assert_eq(sorted, expected);
    }

    #[test]
    unconstrained fn sort_by_counter_desc_with_zero_counters() {
        let original = [
            TestValue { value: 55, counter: 1 },
            TestValue { value: 11, counter: 0 },
            TestValue { value: 33, counter: 2 },
            TestValue { value: 44, counter: 0 },
            TestValue { value: 22, counter: 0 },
            TestValue::empty(),
            TestValue::empty(),
        ];
        let expected = [
            TestValue { value: 33, counter: 2 },
            TestValue { value: 55, counter: 1 },
            TestValue { value: 22, counter: 0 },
            TestValue { value: 44, counter: 0 },
            TestValue { value: 11, counter: 0 },
            TestValue::empty(),
            TestValue::empty(),
        ];
        let sorted = sort_by_counter_desc(original);
        assert_eq(sorted, expected);
    }
}
