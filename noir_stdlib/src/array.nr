#[builtin(array_len)]
fn len<T>(_input : [T]) -> comptime Field {}

// insertion sort - n.b. it is a quadratic sort
fn sort<T>(mut a: [T]) -> [T] { 
    for i in 1..len(a) {
        for j in 0..i {
            if a[i] < a[j] {
                let c = a[j];
                a[j] = a[i];
                a[i]= c;
            }
        };
    };
    a
}

// Sort with a custom sorting function.
fn sort_via<T>(mut a: [T], ordering: fn(T, T) -> bool) -> [T] { 
    for i in 1..len(a) {
        for j in 0..i {
            if ordering(a[i], a[j]) {
                let c = a[j];
                a[j] = a[i];
                a[i]= c;
            }
        };
    };
    a
}

// Apply the given function to each element of the array
fn map<T, U, N>(array: [T; N], f: fn(T) -> U) -> [U; N] {
    let mut ret = [f(array[0]); N];
    for i in 1 .. len(array) {
        ret[i] = f(array[i]);
    }
    ret
}

// Apply a function to each element of the array and an accumulator value,
// returning the final accumulated value. This function is also sometimes
// called `foldl`, `fold_left`, `reduce`, or `inject`.
fn fold<T, U, N>(array: [T; N], mut accumulator: U, f: fn(U, T) -> U) -> U {
    for elem in array {
        accumulator = f(accumulator, elem);
    }
    accumulator
}

// Returns the index of the first element to match the given predicate.
// If no such element is found, this returns -1.
fn find<T, N>(array: [T; N], predicate: fn(T) -> bool) -> i64 {
    let mut ret = -1;
    for i in 0 .. len(array) {
        if predicate(array[i]) & ret != -1 {
            ret = i as i64;
        }
    }
    ret
}

// Returns true if all elements in the array satisfy the predicate
fn all<T, N>(array: [T; N], predicate: fn(T) -> bool) -> bool {
    let mut ret = true;
    for elem in array {
        ret &= predicate(elem);
    }
    ret
}

// Returns true if any element in the array satisfies the predicate
fn any<T, N>(array: [T; N], predicate: fn(T) -> bool) -> bool {
    let mut ret = false;
    for elem in array {
        ret |= predicate(elem);
    }
    ret
}
