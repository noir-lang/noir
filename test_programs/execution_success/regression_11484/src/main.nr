fn main() {
    let mut y = 0;
    let mut x = 0;
    let n = 17;
    for i in 0..n {
        x = 10u32;
        y = i;
    }

    println(y);
    assert(y == n - 1);
    assert(x == 10);
}