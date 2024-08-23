mod assert_combined_permuted_array;
mod assert_deduped_array;
mod dedupe_array;
mod get_deduped_hints;
mod sort_by_position_then_counter;

use crate::{
    abis::side_effect::{Positioned, Ordered}, traits::{Empty, is_empty},
    utils::arrays::{
    array_length,
    assert_combined_deduped_array::{
    assert_combined_permuted_array::assert_combined_permuted_array,
    assert_deduped_array::assert_deduped_array, get_deduped_hints::DedupedHints
}
}
};

// original_array_(lt/gte) must be valid, i.e. validate_array(original_array) == true
// All non-empty items in the original arrays must be unique, otherwise duplicate values could be merged and
// assert_combined_permuted_array would still pass undetected.
// This is currently used for deduplicating public data writes, where each public data write is unique due to having a unique counter.
pub fn assert_combined_deduped_array<T, let N: u32>(
    original_array_lt: [T; N],
    original_array_gte: [T; N],
    sorted_array: [T; N],
    deduped_array: [T; N],
    hints: DedupedHints<N>
) where T: Positioned + Ordered + Empty + Eq {
    assert_combined_permuted_array(
        original_array_lt,
        original_array_gte,
        sorted_array,
        hints.combined_indexes
    );
    assert_deduped_array(sorted_array, deduped_array, hints.run_lengths);
}

mod tests {
    use crate::{
        abis::side_effect::{Positioned, Ordered}, tests::utils::pad_end,
        utils::arrays::{
        array_merge,
        assert_combined_deduped_array::{
        assert_deduped_array::{assert_deduped_array, tests::TestContainer},
        assert_combined_deduped_array, dedupe_array::dedupe_array,
        get_deduped_hints::{DedupedHints, get_deduped_hints}
    }
    }
    };

    fn verify_all<let N: u32>(
        original_array_lt: [TestContainer; N],
        original_array_gte: [TestContainer; N],
        sorted_array: [TestContainer; N],
        deduped_array: [TestContainer; N],
        hints: DedupedHints<N>
    ) {
        let merged = array_merge(original_array_lt, original_array_gte);
        assert_eq(deduped_array, dedupe_array(merged));
        assert_eq(hints, get_deduped_hints(original_array_lt, original_array_gte));
        assert_combined_deduped_array(
            original_array_lt,
            original_array_gte,
            sorted_array,
            deduped_array,
            hints
        );
    }

    #[test]
    fn assert_combined_deduped_array_full() {
        let original_array_lt = pad_end(
            [
            TestContainer { value: 4, position: 3, counter: 2 },
            TestContainer { value: 7, position: 4, counter: 8 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 9, position: 5, counter: 7 }
        ],
            TestContainer::empty()
        );
        let original_array_gte = pad_end(
            [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 5, position: 3, counter: 5 },
            TestContainer { value: 6, position: 3, counter: 6 },
            TestContainer { value: 8, position: 4, counter: 9 },
            TestContainer { value: 2, position: 1, counter: 4 }
        ],
            TestContainer::empty()
        );
        let sorted_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 4 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 2 },
            TestContainer { value: 5, position: 3, counter: 5 },
            TestContainer { value: 6, position: 3, counter: 6 },
            TestContainer { value: 7, position: 4, counter: 8 },
            TestContainer { value: 8, position: 4, counter: 9 },
            TestContainer { value: 9, position: 5, counter: 7 }
        ];
        let deduped_array = [
            TestContainer { value: 2, position: 1, counter: 4 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 6, position: 3, counter: 6 },
            TestContainer { value: 8, position: 4, counter: 9 },
            TestContainer { value: 9, position: 5, counter: 7 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let hints = DedupedHints { combined_indexes: [3, 6, 2, 8, 0, 4, 5, 7, 1], run_lengths: [2, 1, 3, 2, 1, 0, 0, 0, 0] };
        verify_all(
            original_array_lt,
            original_array_gte,
            sorted_array,
            deduped_array,
            hints
        );
    }

    #[test]
    fn assert_combined_deduped_array_padded_empty() {
        let original_array_lt = pad_end(
            [
            TestContainer { value: 4, position: 3, counter: 2 },
            TestContainer { value: 7, position: 4, counter: 8 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 9, position: 5, counter: 7 }
        ],
            TestContainer::empty()
        );
        let original_array_gte = pad_end(
            [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 5, position: 3, counter: 5 },
            TestContainer { value: 6, position: 3, counter: 6 },
            TestContainer { value: 8, position: 4, counter: 9 },
            TestContainer { value: 2, position: 1, counter: 4 }
        ],
            TestContainer::empty()
        );
        let sorted_array = pad_end(
            [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 4 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 2 },
            TestContainer { value: 5, position: 3, counter: 5 },
            TestContainer { value: 6, position: 3, counter: 6 },
            TestContainer { value: 7, position: 4, counter: 8 },
            TestContainer { value: 8, position: 4, counter: 9 },
            TestContainer { value: 9, position: 5, counter: 7 }
        ],
            TestContainer::empty()
        );
        let deduped_array = pad_end(
            [
            TestContainer { value: 2, position: 1, counter: 4 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 6, position: 3, counter: 6 },
            TestContainer { value: 8, position: 4, counter: 9 },
            TestContainer { value: 9, position: 5, counter: 7 }
        ],
            TestContainer::empty()
        );
        let hints = DedupedHints {
            combined_indexes: [3, 6, 2, 8, 0, 4, 5, 7, 1, 11, 10, 9],
            run_lengths: [2, 1, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0]
        };
        verify_all(
            original_array_lt,
            original_array_gte,
            sorted_array,
            deduped_array,
            hints
        );
    }

    #[test]
    fn assert_combined_deduped_array_no_duplicates() {
        let original_array_lt = pad_end(
            [
            TestContainer { value: 3, position: 3, counter: 3 },
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 2, counter: 2 }
        ],
            TestContainer::empty()
        );
        let original_array_gte = pad_end(
            [
            TestContainer { value: 4, position: 4, counter: 4 },
            TestContainer { value: 5, position: 5, counter: 5 }
        ],
            TestContainer::empty()
        );
        let sorted_array = pad_end(
            [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 2, counter: 2 },
            TestContainer { value: 3, position: 3, counter: 3 },
            TestContainer { value: 4, position: 4, counter: 4 },
            TestContainer { value: 5, position: 5, counter: 5 }
        ],
            TestContainer::empty()
        );
        let deduped_array = sorted_array;
        let hints = DedupedHints { combined_indexes: [2, 0, 1, 3, 4, 7, 6, 5], run_lengths: [1, 1, 1, 1, 1, 0, 0, 0] };
        verify_all(
            original_array_lt,
            original_array_gte,
            sorted_array,
            deduped_array,
            hints
        );
    }
}
