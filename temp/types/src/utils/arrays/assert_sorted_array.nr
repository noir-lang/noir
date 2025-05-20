use crate::traits::{Empty, is_empty};

pub fn assert_sorted_array<T, let N: u32, Env>(
    original_array: [T; N],
    sorted_array: [T; N],
    sorted_indexes: [u32; N],
    ordering: fn[Env](T, T) -> bool,
)
where
    T: Eq + Empty,
{
    let mut seen_empty = false;
    for i in 0..N {
        let original_value = original_array[i];
        if is_empty(original_value) {
            seen_empty = true;
            assert(is_empty(sorted_array[i]), "Empty values must not be mixed with sorted values");
        } else {
            assert(!seen_empty, "Empty values must be padded to the right");

            let index = sorted_indexes[i];
            assert(sorted_array[index].eq(original_value), "Invalid index");
            if i != 0 {
                assert(ordering(sorted_array[i - 1], sorted_array[i]), "Values not sorted");
            }
        }
    }
}

mod tests {
    use crate::utils::arrays::assert_sorted_array::assert_sorted_array;

    #[test]
    fn assert_sorted_array_custom_asc_success() {
        let original = [30, 20, 90, 50, 0, 0];
        let sorted = [20, 30, 50, 90, 0, 0];
        let indexes = [1, 0, 3, 2, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test]
    fn assert_sorted_array_custom_desc_success() {
        let original = [30, 20, 90, 50, 0, 0];
        let sorted = [90, 50, 30, 20, 0, 0];
        let indexes = [2, 3, 0, 1, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| b.lt(a));
    }

    #[test]
    fn assert_sorted_array_custom_all_empty_success() {
        let original = [0, 0, 0, 0, 0, 0];
        let sorted = [0, 0, 0, 0, 0, 0];
        let indexes = [0, 0, 0, 0, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test(should_fail_with = "Values not sorted")]
    fn assert_sorted_array_custom_wrong_ordering_fails() {
        let original = [30, 20, 90, 50, 0, 0];
        let sorted = [20, 30, 90, 50, 0, 0];
        let indexes = [1, 0, 2, 3, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test(should_fail_with = "Values not sorted")]
    fn assert_sorted_array_custom_misplaced_sorted_fails() {
        let original = [30, 20, 90, 50, 0, 0];
        let sorted = [20, 30, 50, 0, 0, 90];
        let indexes = [1, 0, 5, 2, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test(should_fail_with = "Invalid index")]
    fn assert_sorted_array_custom_wrong_index_fails() {
        let original = [30, 20, 90, 50, 0, 0];
        let sorted = [20, 30, 50, 90, 0, 0];
        let indexes = [1, 1, 2, 3, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test(should_fail_with = "Empty values must be padded to the right")]
    fn assert_sorted_array_custom_not_padded_fails() {
        let original = [30, 20, 90, 0, 50, 0];
        let sorted = [20, 30, 90, 0, 0, 0];
        let indexes = [1, 0, 2, 0, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }

    #[test(should_fail_with = "Empty values must not be mixed with sorted values")]
    fn assert_sorted_array_custom_mixed_empty_fails() {
        let original = [30, 20, 90, 0, 0, 0];
        let sorted = [20, 30, 90, 0, 0, 10];
        let indexes = [1, 0, 2, 0, 0, 0];
        assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
    }
}
