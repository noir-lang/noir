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

    // Another regression: here the RHS expression must be evaluated
    // first, so i ends up being 1, then the entire op-assign expression
    // ends up being `arr[i] += 5`.
    let mut arr = [10_i32, 20];
    let mut i = 0;

    arr[i] += {
        i = 1;
        5
    };

    assert_eq(arr, [10, 25]);
}
