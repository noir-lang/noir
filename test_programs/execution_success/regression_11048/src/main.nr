// Regression test for issue #11048
// Because apply_index_side_effects() did not support nested arrays,
// c would be input[0].0 (= -1) which overflows its type
// The cast to u16 would then fail execution.
pub fn main(input: [(Field, [u32; 3]); 2], b: bool, i1: u32, i2: u32) -> pub (u32) {
    if (b) {
        let c = input[i1].1[0] as u16;
        c as u32
    } else {
        input[i2].1[2]
    }
}
