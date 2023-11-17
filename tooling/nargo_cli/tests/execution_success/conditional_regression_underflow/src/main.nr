// Regression test for https://github.com/noir-lang/noir/issues/3493
fn main(x: u4) {
    if x == 10 {
        x + 15;
    }
    if x == 9 {
        x << 3;
    }
    if x == 8 {
        x * 3;
    }
    if x == 7 {
        x - 8;
    }
}