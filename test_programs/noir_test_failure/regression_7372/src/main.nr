// TODO(https://github.com/noir-lang/noir/issues/7372): unexpected panic
// The application panicked (crashed).
// Message:  attempt to subtract with overflow
// Location: compiler/noirc_evaluator/src/acir/generated_acir.rs:660

pub fn sort<T>(input: [T; 5])
where
    T: Ord,
{
    sort_via(input, |a, b| a <= b)
}

pub unconstrained fn quicksort<T>(arr: [T; 5]) -> [T; 5] {
    arr
}

pub fn sort_via<T>(input: [T; 5], ordering: fn(T, T) -> bool) {
    // Safety: test
    let sorted = unsafe { quicksort(input) };

    assert(ordering(sorted[0], sorted[0 + 1]), "");
}

mod test {
    use crate::sort;

    global arr_global: [u32; 5] = [42, 123, 87, 93, 48];

    #[test]
    fn test_sort() {
        let arr: [u32; 5] = [3, 6, 8, 10, 1];
        // this panics:
        sort(arr);

        // this works:
        // sort_via(arr, |a, b| a <= b)
    }

    #[test]
    fn test_sort_100_values_comptime() {
        // this panics:
        sort(arr_global);

        // this works:
        // sort_via(arr_global, |a, b| a <= b);
    }

}

fn main() {}
