// Regression test for issue #4449
fn main(x: u8, result: [u8; 32]) {
    let x = x % 31;
    let mut digest = [0; 32];
    for i in 0..70 {
        let y = x + i;
        let a = [y, x, 32, 0, y + 1, y - 1, y - 2, 5];
        digest = std::sha256::digest(a);
    }

    assert(digest == result);
}
