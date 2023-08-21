// Testing integer division: 7/3 = 2
fn main(mut x: u32, y: u32, z: u32) {
    let a = x % y;
    assert_eq(x / y, z);
    assert_eq(a, x - z*y);
    assert_eq((50 as u64) % (9 as u64), 5);
}
