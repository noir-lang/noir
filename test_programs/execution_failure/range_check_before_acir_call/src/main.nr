fn main(w1: Field, w2: Field) {
    // Explicit in-circuit range check on a `Field` parameter. The ABI does not
    // pre-constrain `w1`, so an out-of-range value can be supplied as input and
    // this constraint is what must reject it.
    w1.assert_max_bit_size::<32>();

    // `callee` is `#[fold]`, so it is compiled to a separate ACIR circuit and
    // invoked via `Opcode::Call`. The RangeOptimizer must treat that call as a
    // side-effect boundary and keep the range check above ahead of it.
    callee(w2);

    // A small implied range on `w1`. This must not cause the explicit range
    // check above to be removed across the `callee` ACIR call.
    assert(w1 == 5);
}

#[fold]
fn callee(w2: Field) {
    // Safety: only a print.
    unsafe {
        side_effect(w2);
    }
}

unconstrained fn side_effect(w2: Field) {
    println(w2);
}
