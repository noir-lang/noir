// Test unsafe integer arithmetic
// minimal failing test ...to debug!

fn main(x : u32, y : u32) {
    x = x * x;
    constrain (y) == x;
}