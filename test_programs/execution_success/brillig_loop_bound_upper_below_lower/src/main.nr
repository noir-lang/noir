// Example of a loop with `lower > upper`: It starts at 10 and ends at 8.
// The loop is valid due to u8 wrapping and will terminate once `i` reaches 8.
// Loop unrolling cannot prove termination so it will probably not handle it.
// However, it must not error.
use std::ops::WrappingAdd;

unconstrained fn main() {
    let mut i: u8 = 10;
    loop {
        if i == 8 {
            break;
        }
        i = i.wrapping_add(3);
    }
    assert(i == 8);
}
