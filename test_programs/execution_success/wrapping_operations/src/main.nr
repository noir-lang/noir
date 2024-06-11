use dep::std;

fn main(x: u8, y: u8) {
    assert(std::wrapping_sub(x, 1) == y);
    assert(std::wrapping_add(y, 1) == x);
    assert(std::wrapping_mul(y, y) == 1);
}

