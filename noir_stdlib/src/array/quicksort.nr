unconstrained fn partition<T, Env, let N: u32>(
    arr: &mut [T; N],
    low: u32,
    high: u32,
    sortfn: fn[Env](T, T) -> bool,
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

unconstrained fn quicksort_recursive<T, Env, let N: u32>(
    arr: &mut [T; N],
    low: u32,
    high: u32,
    sortfn: fn[Env](T, T) -> bool,
) {
    if low < high {
        let pivot_index = partition(arr, low, high, sortfn);
        if pivot_index > 0 {
            quicksort_recursive(arr, low, pivot_index - 1, sortfn);
        }
        quicksort_recursive(arr, pivot_index + 1, high, sortfn);
    }
}

pub(crate) unconstrained fn quicksort<T, Env, let N: u32>(
    _arr: [T; N],
    sortfn: fn[Env](T, T) -> bool,
) -> [T; N] {
    let mut arr: [T; N] = _arr;
    if arr.len() <= 1 {} else {
        quicksort_recursive(&mut arr, 0, arr.len() - 1, sortfn);
    }
    arr
}
