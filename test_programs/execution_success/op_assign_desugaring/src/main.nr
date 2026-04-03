fn main() {
    let mut x = [1, 2, 3];
    x[{
    x[0] += 2;
    0
    }] += 3;
    assert_eq(x[0], 1 + 2 + 3);
}
