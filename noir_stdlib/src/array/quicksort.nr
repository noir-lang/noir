unconstrained fn partition<T, let N: u32, Env>(
    arr: &mut [T; N],
    low: u32,
    high: u32,
    sortfn: unconstrained fn[Env](T, T) -> bool,
) -> u32 {
    let pivot = high;
    let mut i = low;
    for j in low..high {
        if (sortfn(arr[j], arr[pivot])) {
            let temp = arr[i];
            arr[i] = arr[j];
            arr[j] = temp;
            i += 1;
        }
    }

    let temp = arr[i];
    arr[i] = arr[pivot];
    arr[pivot] = temp;
    i
}

unconstrained fn quicksort_loop<T, let N: u32, Env>(
    arr: &mut [T; N],
    low: u32,
    high: u32,
    sortfn: unconstrained fn[Env](T, T) -> bool,
) {
    let mut stack: [(u32, u32)] = [(low, high)].as_vector();
    // TODO(https://github.com/noir-lang/noir_sort/issues/22): use 'loop' once it's stabilized
    for _ in 0..2 * N {
        if stack.len() == 0 {
            break;
        }

        let (new_stack, (new_low, new_high)) = stack.pop_back();
        stack = new_stack;

        if new_high < new_low + 1 {
            continue;
        }

        let pivot_index = partition(arr, new_low, new_high, sortfn);
        stack = stack.push_back((pivot_index + 1, new_high));
        if 0 < pivot_index {
            stack = stack.push_back((new_low, pivot_index - 1));
        }
    }
}

pub unconstrained fn quicksort<T, let N: u32, Env>(
    arr: [T; N],
    sortfn: unconstrained fn[Env](T, T) -> bool,
) -> [T; N] {
    let mut arr: [T; N] = arr;
    if arr.len() > 1 {
        quicksort_loop(&mut arr, 0, arr.len() - 1, sortfn);
    }
    arr
}

mod test {
    use crate::cmp::Ord;
    use super::quicksort;

    // helper functions for testing
    fn ascending<T>(a: T, b: T) -> bool
    where
        T: Ord,
    {
        a < b
    }

    fn descending<T>(a: T, b: T) -> bool
    where
        T: Ord,
    {
        a > b
    }

    #[test]
    fn test_empty_array() {
        let arr: [u32; 0] = [];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, arr);
    }

    #[test]
    fn test_single_element() {
        let arr: [u32; 1] = [42];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, arr);
    }

    #[test]
    fn test_two_elements_ascending() {
        let arr: [u32; 2] = [2, 1];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [1, 2]);
    }

    #[test]
    fn test_two_elements_descending() {
        let arr: [u32; 2] = [1, 2];
        // Safety: test
        let result = unsafe { quicksort(arr, descending) };
        assert_eq(result, [2, 1]);
    }

    #[test]
    fn test_already_sorted_ascending() {
        let arr: [u32; 5] = [1, 2, 3, 4, 5];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_already_sorted_ascending_with_one_out_of_order() {
        let mut arr: [u32; 1000] = [0; 1000];
        for i in 0..1000 {
            arr[i] = i as u32;
        }
        arr[0] = 2;
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        arr[0] = 1;
        arr[1] = 2;
        assert_eq(result, arr);
    }

    #[test]
    fn test_already_sorted_descending() {
        let arr: [u32; 5] = [5, 4, 3, 2, 1];
        // Safety: test
        let result = unsafe { quicksort(arr, descending) };
        assert_eq(result, [5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_reverse_sorted_ascending() {
        let arr: [u32; 5] = [5, 4, 3, 2, 1];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_reverse_sorted_descending() {
        let arr: [u32; 5] = [1, 2, 3, 4, 5];
        // Safety: test
        let result = unsafe { quicksort(arr, descending) };
        assert_eq(result, [5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_random_array_ascending() {
        let arr: [u32; 8] = [3, 1, 4, 1, 5, 9, 2, 6];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_random_array_descending() {
        let arr: [u32; 8] = [3, 1, 4, 1, 5, 9, 2, 6];
        // Safety: test
        let result = unsafe { quicksort(arr, descending) };
        assert_eq(result, [9, 6, 5, 4, 3, 2, 1, 1]);
    }

    #[test]
    fn test_duplicate_elements() {
        let arr: [u32; 11] = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [1, 1, 2, 3, 3, 4, 5, 5, 5, 6, 9]);
    }

    #[test]
    fn test_all_same_elements() {
        let arr: [u32; 5] = [42, 42, 42, 42, 42];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [42, 42, 42, 42, 42]);
    }

    #[test]
    fn test_large_array() {
        let arr: [u32; 10] = [9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_negative_numbers() {
        let arr: [i32; 8] = [-3, 1, -4, 1, -5, 9, -2, 6];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [-5, -4, -3, -2, 1, 1, 6, 9]);
    }

    #[test]
    fn test_zero_values() {
        let arr: [u32; 6] = [0, 1, 0, 2, 0, 3];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [0, 0, 0, 1, 2, 3]);
    }

    #[test]
    fn test_edge_case_near_sorted() {
        let arr: [u32; 8] = [1, 2, 3, 5, 4, 6, 7, 8];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_edge_case_near_reverse_sorted() {
        let arr: [u32; 8] = [8, 7, 6, 4, 5, 3, 2, 1];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    fn abs_compare(a: i32, b: i32) -> bool {
        let abs_a = if a < 0 { -a } else { a };
        let abs_b = if b < 0 { -b } else { b };
        abs_a < abs_b
    }

    #[test]
    fn test_custom_comparison() {
        // sort by absolute value

        let arr: [i32; 8] = [-3, 1, -4, 1, -5, 9, -2, 6];
        // Safety: test
        let result = unsafe { quicksort(arr, abs_compare) };
        assert_eq(result, [1, 1, -2, -3, -4, -5, 6, 9]);
    }

    #[test]
    fn test_equivalent_same_relative_order() {
        // sort by absolute value
        let arr: [i32; 11] = [-3, 1, -4, 1, -5, 5, -5, 5, 9, -2, 6];
        // Safety: test
        let result = unsafe { quicksort(arr, abs_compare) };
        assert_eq(result, [1, 1, -2, -3, -4, -5, 5, -5, 5, 6, 9]);
    }

    #[test]
    fn test_maximum_values() {
        let arr: [u32; 4] = [2 ^ 32 - 1, 1, 0, 2 ^ 32 - 2];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [0, 1, 2 ^ 32 - 2, 2 ^ 32 - 1]);
    }

    #[test]
    fn test_alternating_pattern() {
        let arr: [u32; 9] = [1, 9, 2, 8, 3, 7, 4, 6, 5];
        // Safety: test
        let result = unsafe { quicksort(arr, ascending) };
        assert_eq(result, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_multiple_sorts_same_array() {
        let arr: [u32; 8] = [3, 1, 4, 1, 5, 9, 2, 6];
        // Safety: test
        let result1 = unsafe { quicksort(arr, ascending) };
        // Safety: test
        let result2 = unsafe { quicksort(arr, ascending) };
        // Safety: test
        let result3 = unsafe { quicksort(arr, ascending) };

        assert_eq(result1, result2);
        assert_eq(result2, result3);
        assert_eq(result1, [1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_descending_then_ascending() {
        let arr: [u32; 8] = [3, 1, 4, 1, 5, 9, 2, 6];
        // Safety: test
        let desc_result = unsafe { quicksort(arr, descending) };
        assert_eq(desc_result, [9, 6, 5, 4, 3, 2, 1, 1]);

        // Safety: test
        let asc_result: [u32; 8] = unsafe { quicksort(desc_result, ascending) };
        assert_eq(asc_result, [1, 1, 2, 3, 4, 5, 6, 9]);
    }
}
