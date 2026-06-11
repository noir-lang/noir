unconstrained fn one(_r: &Field) -> Field {
    1
}

unconstrained fn two(_r: &Field) -> Field {
    2
}

fn main(condition: bool, x: Field) {
    let f = if condition { one } else { two };
    // Safety: testing function-pointer dispatch across the ACIR/Brillig boundary.
    let y = unsafe { f(&x) };
    let expected: Field = if condition { 1 } else { 2 };
    assert(y == expected);
}
