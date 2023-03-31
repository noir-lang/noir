// Test unsafe integer arithmetic
// Test odd bits integer
fn main(mut x: u32, y: u32) {
    x = x * x;
    constrain y == x;

    let c:u3 = 2;
    constrain c > x as u3;
}
