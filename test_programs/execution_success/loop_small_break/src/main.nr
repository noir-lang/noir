// Regression for issue #7359 (https://github.com/noir-lang/noir/issues/7359)
// We want the loop to be small enough that the compiler may attempt to unroll it.
unconstrained fn main(x: Field) {
    let mut count = 0;

    for i in 0..1 {
        if x == 5 {
            count = i;
            break;
        }
    }
    assert(count == 0);
}
