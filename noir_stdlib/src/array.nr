#[builtin(array_len)]
fn len<T>(_input : [T]) -> comptime Field {}

#[builtin(arraysort)]
fn sort<T, N>(_a: [T; N]) -> [T; N] {}

// Sort with a custom sorting function.
fn sort_via<T>(mut a: [T], ordering: fn(T, T) -> bool) -> [T] { 
    for i in 1..len(a) {
        for j in 0..i {
            if ordering(a[i], a[j]) {
                let old_a_j = a[j];
                a[j] = a[i];
                a[i] = old_a_j;
            }
        }
    }
    a
}

// Apply a function to each element of the array and an accumulator value,
// returning the final accumulated value. This function is also sometimes
// called `foldl`, `fold_left`, `reduce`, or `inject`.
fn fold<T, U, N>(array: [T; N], mut accumulator: U, f: fn(U, T) -> U) -> U {
    for i in 0 .. len(array) {
        accumulator = f(accumulator, array[i]);
    }
    accumulator
}

// Returns the index of the first element to match the given predicate.
// If no such element is found, this returns -1.
fn find<T, N>(array: [T; N], predicate: fn(T) -> bool) -> i64 {
    let mut ret: i64 = 0 - 1;
    for i in 0 .. len(array) {
        if predicate(array[i]) & (ret != 0 - 1) {
            ret = i as i64;
        }
    }
    ret
}

// Returns true if all elements in the array satisfy the predicate
fn all<T, N>(array: [T; N], predicate: fn(T) -> bool) -> bool {
    let mut ret = true;
    for i in 0 .. len(array) {
        ret &= predicate(array[i]);
    }
    ret
}

// Returns true if any element in the array satisfies the predicate
fn any<T, N>(array: [T; N], predicate: fn(T) -> bool) -> bool {
    let mut ret = false;
    for i in 0 .. len(array) {
        ret |= predicate(array[i]);
    }
    ret
}
