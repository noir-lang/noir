fn main(x: field, y: pub field) {
    assert(x != y, f"Expected x != y, but got both equal {x}");
    assert(x != y);
    let z = x + y;
    assert(z != y, f"Expected z != y, but got both equal {z}");
    assert_eq(x, y, f"Expected x == y, but x is {x} and y is {y}");
}
