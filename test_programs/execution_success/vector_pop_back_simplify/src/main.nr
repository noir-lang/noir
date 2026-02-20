use std::collections::unbounded_vec::UnboundedVec;

// TODO: Original test uses (u32, u32) tuples but resize_array growth doesn't support
// non-numeric element types yet. Using two separate vectors of u32 instead.
pub fn main(x: u32, y: u32, i: u32) -> pub ((u32, u32), u32) {
    let mut vec_a: UnboundedVec<u32> = UnboundedVec::from_array([x]);
    let mut vec_b: UnboundedVec<u32> = UnboundedVec::from_array([2]);
    if i < 10 {
        let _ = vec_a.pop_back();
        vec_a.push_back(y);
        let _ = vec_b.pop_back();
        vec_b.push_back(4);
    }
    ((vec_a.get(i), vec_b.get(i)), vec_a.len())
}
