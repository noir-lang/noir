use crate::{traits::{Empty, is_empty}, utils::arrays::validate_array};

pub fn assert_array_eq<T, let N: u32, let S: u32>(array: [T; N], expected: [T; S])
where
    T: Empty + Eq,
{
    assert(expected.all(|elem: T| !is_empty(elem)), "cannot expect empty element in the result");
    assert_eq(validate_array(array), S, "mismatch array lengths");
    for i in 0..S {
        assert_eq(array[i], expected[i], "mismatch array elements");
    }
}

// Swap two items in a BoundedVec.
// Useful when we want to shuffle side effects, which by default are ordered by counters when we add mock data to FixtureBuilder.
pub fn swap_items<T, let N: u32>(vec: &mut BoundedVec<T, N>, from_index: u32, to_index: u32) {
    let tmp = vec.get(from_index);
    vec.set(from_index, vec.get(to_index));
    vec.set(to_index, tmp);
}

pub fn pad_end<T, let N: u32, let M: u32>(items: [T; N], emptyItem: T) -> [T; M] {
    let mut output = [emptyItem; M];
    for i in 0..N {
        output[i] = items[i];
    }
    output
}
