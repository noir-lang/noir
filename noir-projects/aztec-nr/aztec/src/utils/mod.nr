use dep::protocol_types::traits::Eq;

mod test;
mod point;

// Collapses an array of Options with sparse Some values into a BoundedVec, essentially unwrapping the Options and
// removing the None values. For example, given:
//   input: [some(3), none(), some(1)]
// this returns 
//   collapsed: [3, 1]
pub fn collapse<T, let N: u32>(input: [Option<T>; N]) -> BoundedVec<T, N> where T: Eq {
    let (collapsed, collapsed_to_input_index_mapping) = get_collapse_hints(input);
    verify_collapse_hints(input, collapsed, collapsed_to_input_index_mapping);
    collapsed
}

fn verify_collapse_hints<T, let N: u32>(
    input: [Option<T>; N],
    collapsed: BoundedVec<T, N>,
    collapsed_to_input_index_mapping: BoundedVec<u32, N>
) where T: Eq {
    // collapsed should be a BoundedVec with all the non-none elements in input, in the same order. We need to lay down
    // multiple constraints to guarantee this.

    // First we check that the number of elements is correct
    let mut count = 0;
    for i in 0..N {
        if input[i].is_some() {
            count += 1;
        }
    }
    assert_eq(count, collapsed.len(), "Wrong collapsed vec length");

    // Then we check that all elements exist in the original array, and are in the same order. To do this we use the
    // auxiliary collapsed_to_input_index_mapping array, which at index n contains the index in the input array that 
    // corresponds to the collapsed entry at index n.
    // Example:
    //  - input: [some(3), none(), some(1)]
    //  - collapsed: [3, 1]
    //  - collapsed_to_input_index_mapping: [0, 2]
    // These two arrays should therefore have the same length.
    assert_eq(collapsed.len(), collapsed_to_input_index_mapping.len(), "Collapse hint vec length mismatch");

    // We now look at each collapsed entry and check that there is a valid equal entry in the input array.
    let mut last_index = Option::none();
    for i in 0..N {
        if i < collapsed.len() {
            let input_index = collapsed_to_input_index_mapping.get_unchecked(i);
            assert(input_index < N, "Out of bounds index hint");

            assert_eq(collapsed.get_unchecked(i), input[input_index].unwrap(), "Wrong collapsed vec content");

            // By requiring increasing input indices, we both guarantee that we're not looking at the same input
            // element more than once, and that we're going over them in the original order.
            if last_index.is_some() {
                assert(input_index > last_index.unwrap_unchecked(), "Wrong collapsed vec order");
            }
            last_index = Option::some(input_index);
        } else {
            // BoundedVec assumes that the unused parts of the storage are zeroed out (e.g. in the Eq impl), so we make
            // sure that this property holds.
            assert_eq(collapsed.get_unchecked(i), std::unsafe::zeroed(), "Dirty collapsed vec storage");
        }
    }
    // We now know that:
    //  - all values in the collapsed array exist in the input array
    //  - the order of the collapsed values is the same as in the input array
    //  - no input value is present more than once in the collapsed array
    //  - the number of elements in the collapsed array is the same as in the input array.
    // Therefore, the collapsed array is correct.
}

unconstrained fn get_collapse_hints<T, let N: u32>(input: [Option<T>; N]) -> (BoundedVec<T, N>, BoundedVec<u32, N>) {
    let mut collapsed: BoundedVec<T, N> = BoundedVec::new();
    let mut collapsed_to_input_index_mapping: BoundedVec<u32, N> = BoundedVec::new();

    for i in 0..N {
        if input[i].is_some() {
            collapsed.push(input[i].unwrap_unchecked());
            collapsed_to_input_index_mapping.push(i);
        }
    }

    (collapsed, collapsed_to_input_index_mapping)
}
