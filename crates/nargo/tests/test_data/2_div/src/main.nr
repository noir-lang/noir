// Testing integer division: 7/3 = 2
fn main(mut x: u32, y: u32, z: u32) {
    x = x / y;
    constrain x == z;
}
