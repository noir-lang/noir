fn main() {
    let mut x = 3;
    let f = || x;
    let _x2 = f();
    assert(x == 3);
}
