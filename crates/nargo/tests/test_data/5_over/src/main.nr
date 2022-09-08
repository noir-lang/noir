// Test unsafe integer arithmetic
fn main(mut x: u32, y: u32) {
    x = x * x;
    constrain y == x;
}
