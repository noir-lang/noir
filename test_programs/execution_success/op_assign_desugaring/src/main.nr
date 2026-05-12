struct Pair {
    x: Field,
    y: Field,
}

fn main() {
    // Index sub-expression must be evaluated only once.
    let mut x = [1, 2, 3];
    x[{
        x[0] += 2;
        0
    }] += 3;
    assert_eq(x[0], 1 + 2 + 3);

    // Member-access op-assign must resolve the correct field index.
    let mut p = Pair { x: 10, y: 20 };
    p.x += 5;
    assert_eq(p.x, 15);
    assert_eq(p.y, 20);
}
