// A signed `for` loop whose induction variable crosses zero. The `-1 -> 0` step is an unchecked
// add that must be reduced back into range before constant folding treats it as a small value.
fn main(x: u32) -> pub u32 {
    let mut sum: u32 = x;
    for i in -3..=0_i8 {
        sum = (sum + (i as i16 + 200) as u32) % 1000;
    }
    sum
}
