use crate::{
    abis::side_effect::{Ordered, Positioned}, traits::Empty,
    utils::arrays::{
    array_merge,
    assert_combined_deduped_array::sort_by_position_then_counter::compare_by_position_then_counter,
    get_sorted_tuple::get_sorted_tuple
}
};

struct DedupedHints<let N: u32> {
    combined_indexes: [u32; N],
    run_lengths: [u32; N],
}

impl<let N: u32> Eq for DedupedHints<N> {
    fn eq(self, other: Self) -> bool {
        (self.combined_indexes == other.combined_indexes) & (self.run_lengths == other.run_lengths)
    }
}

pub fn get_deduped_hints<T, let N: u32>(
    original_array_lt: [T; N],
    original_array_gte: [T; N]
) -> DedupedHints<N> where T: Positioned + Ordered + Empty + Eq {
    let mut combined_indexes = [0; N];
    let mut run_lengths = BoundedVec::new();

    let merged = array_merge(original_array_lt, original_array_gte);
    let sorted = get_sorted_tuple(merged, compare_by_position_then_counter);
    let mut prev_position = sorted[0].elem.position();
    let mut run_length = 0;
    for i in 0..sorted.len() {
        combined_indexes[sorted[i].original_index] = i;

        let position = sorted[i].elem.position();
        if position != 0 {
            if position != prev_position {
                run_lengths.push(run_length);
                run_length = 1;
            } else {
                run_length += 1;
            }
            prev_position = position;
        }
    }
    run_lengths.push(run_length);

    DedupedHints { combined_indexes, run_lengths: run_lengths.storage }
}
