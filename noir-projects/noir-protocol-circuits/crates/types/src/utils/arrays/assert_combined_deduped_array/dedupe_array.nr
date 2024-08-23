use crate::{
    abis::side_effect::{Ordered, Positioned}, traits::Empty,
    utils::arrays::assert_combined_deduped_array::sort_by_position_then_counter::sort_by_position_then_counter
};

pub fn dedupe_array<T, let N: u32>(array: [T; N]) -> [T; N] where T: Positioned + Ordered + Empty + Eq {
    let sorted = sort_by_position_then_counter(array);
    let mut deduped = [T::empty(); N];
    let mut num_deduped = 0;
    let mut prev_position = sorted[0].position();
    for item in sorted {
        let position = item.position();
        if position != prev_position {
            num_deduped += 1;
        }
        deduped[num_deduped] = item;
        prev_position = position;
    }
    deduped
}

mod tests {
    use crate::{
        tests::utils::pad_end,
        utils::arrays::assert_combined_deduped_array::{assert_deduped_array::tests::TestContainer, dedupe_array::dedupe_array}
    };

    #[test]
    fn dedupe_array_padded_empty() {
        let original_array = [
            TestContainer { value: 11, position: 3, counter: 2 },
            TestContainer { value: 55, position: 4, counter: 9 },
            TestContainer { value: 99, position: 3, counter: 5 },
            TestContainer { value: 66, position: 1, counter: 1 },
            TestContainer { value: 44, position: 4, counter: 8 },
            TestContainer { value: 77, position: 5, counter: 7 },
            TestContainer { value: 33, position: 1, counter: 4 },
            TestContainer { value: 22, position: 3, counter: 6 },
            TestContainer { value: 88, position: 2, counter: 3 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let deduped_array = [
            TestContainer { value: 33, position: 1, counter: 4 },
            TestContainer { value: 88, position: 2, counter: 3 },
            TestContainer { value: 22, position: 3, counter: 6 },
            TestContainer { value: 55, position: 4, counter: 9 },
            TestContainer { value: 77, position: 5, counter: 7 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        assert_eq(dedupe_array(original_array), deduped_array);
    }

    #[test]
    fn dedupe_array_empty_arrays() {
        let original_array = [TestContainer::empty(); 12];
        let deduped_array = [TestContainer::empty(); 12];
        assert_eq(dedupe_array(original_array), deduped_array);
    }

    #[test]
    fn dedupe_array_no_duplicates() {
        let original_array = [
            TestContainer { value: 88, position: 3, counter: 3 },
            TestContainer { value: 11, position: 4, counter: 4 },
            TestContainer { value: 33, position: 2, counter: 2 },
            TestContainer { value: 99, position: 5, counter: 5 },
            TestContainer { value: 66, position: 1, counter: 1 }
        ];
        let deduped_array = [
            TestContainer { value: 66, position: 1, counter: 1 },
            TestContainer { value: 33, position: 2, counter: 2 },
            TestContainer { value: 88, position: 3, counter: 3 },
            TestContainer { value: 11, position: 4, counter: 4 },
            TestContainer { value: 99, position: 5, counter: 5 }
        ];
        assert_eq(dedupe_array(original_array), deduped_array);
    }

    #[test]
    fn dedupe_array_no_duplicates_padded_empty() {
        let original_array = [
            TestContainer { value: 88, position: 3, counter: 3 },
            TestContainer { value: 11, position: 4, counter: 4 },
            TestContainer { value: 33, position: 2, counter: 2 },
            TestContainer { value: 99, position: 5, counter: 5 },
            TestContainer { value: 66, position: 1, counter: 1 },
            TestContainer::empty(),
            TestContainer::empty(),
            TestContainer::empty()
        ];
        let deduped_array = [
            TestContainer { value: 66, position: 1, counter: 1 },
            TestContainer { value: 33, position: 2, counter: 2 },
            TestContainer { value: 88, position: 3, counter: 3 },
            TestContainer { value: 11, position: 4, counter: 4 },
            TestContainer { value: 99, position: 5, counter: 5 },
            TestContainer::empty(),
            TestContainer::empty(),
            TestContainer::empty()
        ];
        assert_eq(dedupe_array(original_array), deduped_array);
    }

    #[test]
    fn dedupe_array_all_duplicates() {
        let original_array = [
            TestContainer { value: 55, position: 1, counter: 8 },
            TestContainer { value: 33, position: 1, counter: 2 },
            TestContainer { value: 11, position: 1, counter: 4 },
            TestContainer { value: 88, position: 1, counter: 3 },
            TestContainer { value: 99, position: 1, counter: 5 },
            TestContainer { value: 77, position: 1, counter: 9 },
            TestContainer { value: 66, position: 1, counter: 1 },
            TestContainer { value: 44, position: 1, counter: 7 },
            TestContainer { value: 22, position: 1, counter: 6 }
        ];
        let deduped_array = pad_end(
            [TestContainer { value: 77, position: 1, counter: 9 }],
            TestContainer::empty()
        );
        assert_eq(dedupe_array(original_array), deduped_array);
    }
}
