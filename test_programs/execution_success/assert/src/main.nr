fn main(x: Field) {
    assert(x == 1);
    assert(1 == conditional(x as bool));
}

fn conditional(x: bool) -> Field {
    assert(x, f"Expected x to be true but got {x}");
    assert_eq(x, true, f"Expected x to be true but got {x}");
    1
}
