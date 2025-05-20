use crate::{
    abis::side_effect::Ordered,
    traits::Empty,
    utils::arrays::{
        get_sorted_tuple::get_sorted_tuple,
        sort_by_counter::{
            compare_by_counter_empty_padded_asc, compare_by_counter_empty_padded_desc,
        },
    },
};

pub struct OrderHint {
    pub counter: u32,
    pub sorted_index: u32,
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

pub unconstrained fn get_order_hints<T, let N: u32>(
    array: [T; N],
    ordering: fn(T, T) -> bool,
) -> [OrderHint; N]
where
    T: Ordered,
{
    let sorted_tuples = get_sorted_tuple(array, ordering);

    let mut hints = [OrderHint::empty(); N];
    for i in 0..N {
        let elem = sorted_tuples[i].elem;
        hints[i].counter = elem.counter();
        let original_index = sorted_tuples[i].original_index;
        hints[original_index].sorted_index = i;
    }

    hints
}

pub unconstrained fn get_order_hints_asc<T, let N: u32>(array: [T; N]) -> [OrderHint; N]
where
    T: Ordered + Eq + Empty,
{
    get_order_hints(array, compare_by_counter_empty_padded_asc)
}

pub unconstrained fn get_order_hints_desc<T, let N: u32>(array: [T; N]) -> [OrderHint; N]
where
    T: Ordered + Eq + Empty,
{
    get_order_hints(array, compare_by_counter_empty_padded_desc)
}

mod tests {
    use crate::{
        tests::{types::TestValue, utils::pad_end},
        traits::Empty,
        utils::arrays::assert_exposed_sorted_transformed_value_array::get_order_hints::{
            get_order_hints_asc, get_order_hints_desc, OrderHint,
        },
    };

    struct TestBuilder<let N: u32> {
        array: [TestValue; N],
    }

    impl TestBuilder<3> {
        pub fn new() -> Self {
            let array = [
                TestValue { value: 100, counter: 9 },
                TestValue { value: 200, counter: 3 },
                TestValue { value: 300, counter: 6 },
            ];
            TestBuilder { array }
        }
    }

    impl TestBuilder<5> {
        pub fn new_padded() -> Self {
            let array = pad_end(
                [
                    TestValue { value: 100, counter: 9 },
                    TestValue { value: 200, counter: 3 },
                    TestValue { value: 300, counter: 6 },
                ],
                TestValue::empty(),
            );
            TestBuilder { array }
        }
    }

    impl<let N: u32> TestBuilder<N> {
        pub unconstrained fn asc_to_equal(self, expected: [OrderHint; N]) {
            let hints = get_order_hints_asc(self.array);
            assert_eq(hints, expected);
        }

        pub unconstrained fn desc_to_equal(self, expected: [OrderHint; N]) {
            let hints = get_order_hints_desc(self.array);
            assert_eq(hints, expected);
        }
    }

    #[test]
    unconstrained fn get_order_hints_asc_full_non_empty() {
        let builder = TestBuilder::new();
        let expected_hints = [
            OrderHint { counter: 3, sorted_index: 2 },
            OrderHint { counter: 6, sorted_index: 0 },
            OrderHint { counter: 9, sorted_index: 1 },
        ];
        builder.asc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_order_hints_asc_padded_empty() {
        let builder = TestBuilder::new_padded();
        let expected_hints = [
            OrderHint { counter: 3, sorted_index: 2 },
            OrderHint { counter: 6, sorted_index: 0 },
            OrderHint { counter: 9, sorted_index: 1 },
            OrderHint { counter: 0, sorted_index: 3 },
            OrderHint { counter: 0, sorted_index: 4 },
        ];
        builder.asc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_order_hints_desc_full_non_empty() {
        let builder = TestBuilder::new();
        let expected_hints = [
            OrderHint { counter: 9, sorted_index: 0 },
            OrderHint { counter: 6, sorted_index: 2 },
            OrderHint { counter: 3, sorted_index: 1 },
        ];
        builder.desc_to_equal(expected_hints);
    }

    #[test]
    unconstrained fn get_order_hints_desc_padded_empty() {
        let builder = TestBuilder::new_padded();
        let expected_hints = [
            OrderHint { counter: 9, sorted_index: 0 },
            OrderHint { counter: 6, sorted_index: 2 },
            OrderHint { counter: 3, sorted_index: 1 },
            OrderHint { counter: 0, sorted_index: 3 },
            OrderHint { counter: 0, sorted_index: 4 },
        ];
        builder.desc_to_equal(expected_hints);
    }
}
