fn main(mut a: bool) {
    // MakeArray composite slot type `&bool` vs element value `&mut bool`.
    let _: [&bool; 1] = [(&a); 1];

    // Call return: callee returns `&mut bool`, declared `&bool`.
    let _ = foo(a);

    // Allocate/Store/Load where the slot's element type contains a reference
    // with different mutability than the stored value carries.
    let _ref: &bool = &a;

    // `let`-binding of a reference value — Allocate + Store + Load round-trip
    // of a reference-typed value, exercising the Load/Store paths against an
    // Allocate whose element type itself contains a reference.
    let r: &bool = &a;
    let _r2 = r;

    // Call argument: nested mutability mismatch passed through an array.
    let arr: [&bool; 1] = [(&a); 1];
    bar(arr);

    // Call argument: top-level `&mut → &` (already handled by
    // is_mut_ref_to_immutable_ref).
    baz(&a);

    // Store-into-Allocate path: a mutable unconstrained tuple whose declared
    // type contains a `&bool` slot is allocated as `&mut &bool`, but `&tup.0`
    // inside `qux` produces `&mut bool` because `tup.0` lives in the mutable
    // binding. The SSA validator must accept storing the latter into the
    // former — same mutability-equivalence rule as the MakeArray / Call /
    // Block-argument positions above.
    // safety: no actual unsafety, only exercising unconstrained SSA-gen
    unsafe {
        qux(a);
    }
}

unconstrained fn qux(a: bool) {
    let mut tup: (bool, &bool) = (a, &a);
    let _x = tup.1;
    tup.1 = &tup.0;
}

fn foo(mut a: bool) -> &bool {
    &a
}

fn bar(_arr: [&bool; 1]) {}

fn baz(_r: &bool) {}
