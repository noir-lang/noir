use dep::std;

// Test integer addition: 3 + 4 = 7
fn main(mut x: u32, y: u32, z: u32) {
    x += y;
    std::assert_eq(x, z);

    x *= 8;
    assert(x>9);
}
