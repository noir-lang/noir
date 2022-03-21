// Test unsafe integer arithmetic

fn main(x : u32, y : u32) {
    x = x * x;
    constrain (y) == x;
}