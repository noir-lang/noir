struct Cell {
    value: Field,
}

// Taking `&mut` into the by-value temporary returned by a vector pop/remove must
// not alias the original vector's elements: the methods take `self` by value and
// leave the original vector unchanged.
fn main() {
    assert_eq(pop_front_does_not_leak(), 1);
    assert_eq(pop_back_does_not_leak(), 2);
    assert_eq(remove_does_not_leak(), 1);
}

fn pop_front_does_not_leak() -> Field {
    let original = [Cell { value: 1 }, Cell { value: 2 }].as_vector();
    let value_ref = &mut original.pop_front().0.value;
    *value_ref = 9;
    original[0].value
}

fn pop_back_does_not_leak() -> Field {
    let original = [Cell { value: 1 }, Cell { value: 2 }].as_vector();
    let value_ref = &mut original.pop_back().1.value;
    *value_ref = 9;
    original[1].value
}

fn remove_does_not_leak() -> Field {
    let original = [Cell { value: 1 }, Cell { value: 2 }].as_vector();
    let value_ref = &mut original.remove(0).1.value;
    *value_ref = 9;
    original[0].value
}
