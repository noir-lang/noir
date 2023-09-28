use dep::std;

// Test unsafe integer arithmetic
// Test odd bits integer
fn main(mut x: u32, y: u32) {
    x = std::wrapping_mul(x,x);
    assert(y == x);

    let c:u3 = 2;
    assert(c > x as u3);
}
