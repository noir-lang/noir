// Regression test for https://github.com/noir-lang/noir/issues/3493
fn main(x: u4) {
    if x == 10 {
        x + 15;
    }
}