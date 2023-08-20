use dep::std;

fn main(x: Field, y: Field) {
    let pair = (x, y);
    std::assert_eq(pair.0, 1);
    std::assert_eq(pair.1, 0);

    let (a, b) = if true { (0, 1) } else { (2, 3) };
    std::assert_eq(a, 0);
    std::assert_eq(b, 1);

    let (u,v) = if x as u32 < 1 {
        (x, x + 1)
    } else {
        (x + 1, x)
    };
    std::assert_eq(u, x+1);
    std::assert_eq(v, x);

    // Test mutating tuples
    let mut mutable = ((0, 0), 1, 2, 3);
    mutable.0 = (x, y);
    mutable.2 = 7;
    std::assert_eq(mutable.0.0, 1);
    std::assert_eq(mutable.0.1, 0);
    std::assert_eq(mutable.1, 1);
    std::assert_eq(mutable.2, 7);
    std::assert_eq(mutable.3, 3);
}
