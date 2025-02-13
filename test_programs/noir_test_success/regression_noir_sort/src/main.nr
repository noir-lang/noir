mod quicksort;

pub fn sort<T, let N: u32>(input: [T; N]) -> [T; N] where T: Ord {
    sort_via(input, |a, b| a <= b)
}

pub fn sort_via<T, let N: u32>(input: [T; N], sortfn: fn(T, T) -> bool) -> [T; N]
{
    // Safety: test
    let sorted = unsafe { quicksort::quicksort(input, sortfn) };

    for i in 0..N - 1 {
        assert(
            sortfn(sorted[i], sorted[i + 1]),
            "Array has not been sorted correctly according to `ordering`.",
        );
    }

    sorted
}

mod test {
    use crate::sort;

    global arr_comptime: [u32; 7] = [3, 6, 8, 10, 1, 2, 1];
    global expected_comptime: [u32; 7] = [1, 1, 2, 3, 6, 8, 10];

    #[test]
    fn test_sort() {
        let mut arr: [u32; 7] = [3, 6, 8, 10, 1, 2, 1];

        let sorted = sort(arr);

        let expected: [u32; 7] = [1, 1, 2, 3, 6, 8, 10];
        assert(sorted == expected);
    }

    #[test]
    fn test_sort_comptime() {

        let sorted = sort(arr_comptime);

        assert(sorted == expected_comptime);
    }
}
