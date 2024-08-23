use crate::{abis::side_effect::{Positioned, Ordered}};

pub fn compare_by_position_then_counter<T>(a: T, b: T) -> bool where T: Positioned + Ordered {
    if a.position() == b.position() {
        (a.counter() == 0) | (b.counter() > a.counter())
    } else {
        (b.position() == 0) | ((a.position() != 0) & a.position().lt(b.position()))
    }
}

pub fn sort_by_position_then_counter<T, let N: u32>(array: [T; N]) -> [T; N] where T: Positioned + Ordered {
    array.sort_via(|a, b| compare_by_position_then_counter(a, b))
}

mod tests {
    use crate::utils::arrays::assert_combined_deduped_array::{
        assert_deduped_array::tests::TestContainer,
        sort_by_position_then_counter::sort_by_position_then_counter
    };

    #[test]
    fn sort_by_position_then_counter_empty_padded() {
        let original_array = [
            TestContainer { value: 55, position: 4, counter: 8 },
            TestContainer { value: 11, position: 3, counter: 5 },
            TestContainer { value: 88, position: 1, counter: 4 },
            TestContainer { value: 44, position: 3, counter: 2 },
            TestContainer { value: 33, position: 1, counter: 1 },
            TestContainer { value: 66, position: 5, counter: 7 },
            TestContainer { value: 99, position: 4, counter: 9 },
            TestContainer { value: 77, position: 2, counter: 3 },
            TestContainer { value: 22, position: 3, counter: 6 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let expected = [
            TestContainer { value: 33, position: 1, counter: 1 },
            TestContainer { value: 88, position: 1, counter: 4 },
            TestContainer { value: 77, position: 2, counter: 3 },
            TestContainer { value: 44, position: 3, counter: 2 },
            TestContainer { value: 11, position: 3, counter: 5 },
            TestContainer { value: 22, position: 3, counter: 6 },
            TestContainer { value: 55, position: 4, counter: 8 },
            TestContainer { value: 99, position: 4, counter: 9 },
            TestContainer { value: 66, position: 5, counter: 7 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        assert_eq(sort_by_position_then_counter(original_array), expected);
    }
}
