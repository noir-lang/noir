// Companion test program for the ownership test by_value_arg_aliasing_mut_ref_arg_of_same_call_prevents_move

unconstrained fn f(r: &mut [Field; 3], v: [Field; 3], i: u32) -> ([Field; 3], [Field; 3]) {
    (*r)[i] = 99;
    (v, *r)
}

unconstrained fn main(a: Field, b: Field, c: Field, i: u32) {
    let mut x = [a, b, c];
    let (v, r) = f(&mut x, x, i);
    // `v` must be the pre-write snapshot (the inputs fix `i = 0` and `a = 1`).
    assert(v[i] == a);
    assert(r[i] == 99);
}
