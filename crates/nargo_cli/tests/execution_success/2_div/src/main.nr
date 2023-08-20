use dep::std;

// Testing integer division: 7/3 = 2
fn main(mut x: u32, y: u32, z: u32) {
    let a = x % y;
    std::assert_eq(x / y, z);
    std::assert_eq(a, x - z*y);
    std::assert_eq((50 as u64) % (9 as u64), 5);
}
