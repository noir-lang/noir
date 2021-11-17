mod foo;
// Test integer addition: 3 + 4 = 7

fn main(x : u32, y : u32, z : u32) {
    x = x + y;
    constrain (x) == z;
}