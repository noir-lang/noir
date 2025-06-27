// Testing integer division: 7/3 = 2
fn main(mut x: u32, y: u32, z: u32) {
    let a = x % y;
    assert(x / y == z);
    assert(a == x - z * y);
    assert(50_u64 % 9 == 5);
}
