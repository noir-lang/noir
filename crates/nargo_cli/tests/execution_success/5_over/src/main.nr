// Test unsafe integer arithmetic
// Test odd bits integer
fn main(mut x: u32, y: u32) {
    x = x * x;
    assert_eq(y, x);

    let c:u3 = 2;
    assert(c > x as u3);
}
