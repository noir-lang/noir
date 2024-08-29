use crate::{abis::side_effect::{Positioned, Ordered}, traits::Empty, utils::arrays::{array_length, validate_array}};

/*
    The sorted_array here needs to be sorted based on the `position` field of the container,
    *and* a secondary sort based on the `counter` field of the container.

    For example, the storage slot in the case of public data update requests.
    The run_lengths array should contain the length of each run of the sorted_array.
    The deduped_array should contain the deduplicated array.

    For example, if the original array is writing `(position,value,counter)`s:
    [ (1,1,1), (1,2,4), (2,3,3), (3,4,2), (3,5,5), (3,6,6), (4,7,8), (4,8,9), (5,9,7), (0,0,0), ... padding with zeros ]
    then run_lengths array is:
    [
        2, // run of 1s
        1, // run of 2
        3, // run of 3s
        2, // run of 4s
        1, // run of 5
        0,
        0,
        ... padding with zeros
    ]

    then the deduped_array should be:
    [ (1,2,4), (2,3,3), (3,6,6), (4,8,9), (5,9,7), (0,0,0), ... padding with zeros ]
*/
pub fn assert_deduped_array<T, let N: u32>(
    sorted_array: [T; N],
    deduped_array: [T; N],
    run_lengths: [u32; N]
) where T: Positioned + Ordered + Empty + Eq {
    let num_non_empty_items = array_length(sorted_array);
    let deduped_len = validate_array(deduped_array); // This makes sure that the array is padded with empty items.

    // container at the start of the current run
    let mut start_run_container = sorted_array[0];
    // the index we are collapsing into
    let mut deduped_index = 0;
    // the length of the current run we are collapsing
    let mut run_counter = 0;
    let mut should_check = true;
    for i in 0..N {
        should_check &= i != num_non_empty_items;
        if should_check {
            let current_container = sorted_array[i];

            if run_counter == 0 {
                // Start a new run.
                run_counter = run_lengths[deduped_index];
                if i != 0 {
                    assert(
                        start_run_container.position().lt(current_container.position()), "Containers in a run must be sorted by position"
                    );
                }
                start_run_container = current_container;
            }

            assert(run_counter != 0, "Invalid run length");
            run_counter -= 1;

            assert_eq(
                current_container.position(), start_run_container.position(), "The position of the current container must match the start of the run"
            );

            if run_counter == 0 {
                // End of the run.
                assert_eq(
                    deduped_array[deduped_index], current_container, "The container we are collapsing into must match the current container"
                );
                deduped_index += 1;
            } else {
                // We're in a run, so this container must have a lower counter.
                // Note we don't check for overflow here, as the run_lengths array must be correct.
                assert(
                    current_container.counter() < sorted_array[i + 1].counter(), "Containers in a run must be sorted by counter"
                );
            }
        }
    }

    assert_eq(deduped_index, deduped_len, "Final deduped index does not match deduped array length");
}

mod tests {
    use crate::{
        abis::side_effect::{Positioned, Ordered}, traits::Empty, tests::utils::pad_end,
        utils::arrays::assert_combined_deduped_array::{assert_deduped_array::assert_deduped_array}
    };

    struct TestContainer {
        value: Field,
        position: Field,
        counter: u32,
    }

    impl Positioned for TestContainer {
        fn position(self) -> Field {
            self.position
        }
    }

    impl Ordered for TestContainer {
        fn counter(self) -> u32 {
            self.counter
        }
    }

    impl Empty for TestContainer {
        fn empty() -> Self {
            TestContainer { value: 0, position: 0, counter: 0 }
        }
    }

    impl Eq for TestContainer {
        fn eq(self, other: Self) -> bool {
            self.value.eq(other.value) & self.position.eq(other.position) & self.counter.eq(other.counter)
        }
    }

    #[test]
    fn assert_deduped_array_basic_test() {
        let sorted_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 4 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 2 },
            TestContainer { value: 5, position: 3, counter: 5 },
            TestContainer { value: 6, position: 3, counter: 6 },
            TestContainer { value: 7, position: 4, counter: 8 },
            TestContainer { value: 8, position: 4, counter: 9 },
            TestContainer { value: 9, position: 5, counter: 7 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
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
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let run_lengths = [2, 1, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }

    #[test]
    fn assert_deduped_array_empty_arrays() {
        let sorted_array = [TestContainer::empty(); 12];
        let deduped_array = [TestContainer::empty(); 12];
        let run_lengths = [0; 12];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }

    #[test]
    fn assert_deduped_array_no_duplicates() {
        let sorted_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 2, counter: 2 },
            TestContainer { value: 3, position: 3, counter: 3 },
            TestContainer { value: 4, position: 4, counter: 4 },
            TestContainer { value: 5, position: 5, counter: 5 }
        ];
        let deduped_array = sorted_array;
        let run_lengths = [1; 5];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }

    #[test]
    fn assert_deduped_array_no_duplicates_padded_empty() {
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
        let run_lengths = [1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }

    #[test]
    fn assert_deduped_array_single_run_at_end() {
        let sorted_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 2, counter: 2 },
            TestContainer { value: 3, position: 3, counter: 3 },
            TestContainer { value: 4, position: 4, counter: 4 },
            TestContainer { value: 5, position: 5, counter: 5 },
            TestContainer { value: 6, position: 6, counter: 7 },
            TestContainer { value: 7, position: 6, counter: 8 },
            TestContainer { value: 8, position: 6, counter: 9 }
        ];
        let deduped_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 2, counter: 2 },
            TestContainer { value: 3, position: 3, counter: 3 },
            TestContainer { value: 4, position: 4, counter: 4 },
            TestContainer { value: 5, position: 5, counter: 5 },
            TestContainer { value: 8, position: 6, counter: 9 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let run_lengths = [1, 1, 1, 1, 1, 3, 0, 0];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }

    #[test]
    fn assert_deduped_array_all_duplicates() {
        let sorted_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 1, counter: 3 },
            TestContainer { value: 4, position: 1, counter: 4 },
            TestContainer { value: 5, position: 1, counter: 5 },
            TestContainer { value: 6, position: 1, counter: 6 },
            TestContainer { value: 7, position: 1, counter: 7 },
            TestContainer { value: 8, position: 1, counter: 8 },
            TestContainer { value: 9, position: 1, counter: 9 }
        ];
        let deduped_array = pad_end(
            [TestContainer { value: 9, position: 1, counter: 9 }],
            TestContainer::empty()
        );
        let run_lengths = [9, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }

    #[test(should_fail_with="The position of the current container must match the start of the run")]
    fn assert_deduped_array_mismatched_position_in_run() {
        let sorted_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 }
        ];
        let deduped_array = [
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let run_lengths = [3, 1, 1, 0];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }

    #[test(should_fail_with="The container we are collapsing into must match the current container")]
    fn assert_deduped_array_mismatched_deduped_value() {
        let sorted_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 }
        ];
        let deduped_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let run_lengths = [2, 1, 1, 0];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }

    #[test(should_fail_with="Containers in a run must be sorted by position")]
    fn assert_deduped_array_missed_deduped_value() {
        let sorted_array = [
            TestContainer { value: 1, position: 1, counter: 1 },// This should've been deduped.
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 }
        ];
        let deduped_array = sorted_array;
        let run_lengths = [1, 1, 1, 1];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }

    #[test(should_fail_with="Containers in a run must be sorted by position")]
    fn assert_deduped_array_unsorted_array() {
        let sorted_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 2, position: 1, counter: 2 },// Not sorted by position.
            TestContainer { value: 4, position: 3, counter: 4 }
        ];
        let deduped_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 4, position: 3, counter: 4 }
        ];
        let run_lengths = [1, 1, 1, 1];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }

    #[test(should_fail_with="Final deduped index does not match deduped array length")]
    fn assert_deduped_array_extra_value() {
        let sorted_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 }
        ];
        let deduped_array = [
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 },
            TestContainer { value: 1, position: 1, counter: 1 }// This should be empty.
        ];
        let run_lengths = [2, 1, 1, 0];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }

    #[test(should_fail_with="invalid array")]
    fn assert_deduped_array_padded_non_empty_value() {
        let sorted_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer::empty()
        ];
        let deduped_array = [
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer::empty(),
            TestContainer { value: 1, position: 1, counter: 1 }// This should be empty.
        ];
        let run_lengths = [2, 1, 0, 0];
        assert_deduped_array(sorted_array, deduped_array, run_lengths);
    }
}
