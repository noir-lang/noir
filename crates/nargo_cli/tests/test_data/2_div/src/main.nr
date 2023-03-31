// Testing integer division: 7/3 = 2
fn main(mut x: u32, y: u32, z: u32) {
    let a = x % y;
    constrain x / y == z;
    constrain a == x - z*y;
}
