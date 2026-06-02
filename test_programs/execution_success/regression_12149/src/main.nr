unconstrained fn main(i: u32, j: u32, x: u32) -> pub u32 {
    let a = [10, 20, 0];
    // `a` should be cloned when it's passed to these calls, so the modification
    // in `bar` does not affect the value returned from `foo`. For this though,
    // the clone has to be where it's passed to `bar`, because SSA codegen generates
    // the index first, and the collection second.
    foo(a)[bar(a, i, j, x)]
}

unconstrained fn foo(mut a: [u32; 3]) -> [u32; 3] {
    a
}

unconstrained fn bar(mut a: [u32; 3], i: u32, j: u32, x: u32) -> u32 {
    a[i] = x;
    a[j]
}
