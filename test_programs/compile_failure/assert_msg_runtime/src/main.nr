fn main(x: Field, y: pub Field) {
    assert(x != y, f"Expected x != y, but got both equal {x}");
    assert(x != y, f"Expected x != y, but got both equal {x}");
    assert(x != y, f"Expected x != y, but got both equal {x}");
    assert(x != y, f"Expected x != y, but got both equal {x}");
    assert(x != y, f"Expected x != y, but got both equal {x}");
    let z = x + y;
    assert(z != y, f"Expected x != y, but got both equal {x}");
    // assert_eq(x, y);
    assert_eq(x, y, f"Expected x == y, but x is {x} and y is {y}");
}