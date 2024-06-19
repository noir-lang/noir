use fraction::{Fraction, MAX, floor, toFraction, addFraction};

fn main() {
    let g1 = toFraction(true, 33333333, 5);
    let g2 = toFraction(true, 500000, 33333333);
    let a = addFraction(g1, g2);

    let f1 = floor(a);
    let f2 = MAX;
    assert(f1.sign);
    assert(f2.sign);

    if f1.sign != f2.sign {
        if (f1.sign) { () } else { () }
    } else {
        // Test fails here before the fix to #5202.
        // An optimization which assumes an if condition to be true/false
        // within the then/else branch respectively wasn't being set properly
        // causing f1.sign to be assumed to be false in this else branch.
        assert(f1.sign);
        assert(f2.sign);
    }
}
