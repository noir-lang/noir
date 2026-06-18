// The real program from issue #11889 that motivated this optimization: concatenating
// `BoundedVec`s with `extend_from_bounded_vec`. The original stdlib implementation
// emitted a conditional `storage[len + i] = src[i]` per element, whose per-iteration
// whole-array merge ballooned the circuit. The stdlib was later reworked to assign a
// merged value unconditionally and avoid the merge; collapsing the conditional
// array-set chain makes that rework unnecessary, so this guards the real `BoundedVec`
// path against the balloon returning. Mirrors `bounded_vec_extend_pattern`, which is
// the same loop inlined without the stdlib.
fn main(x: [u8; 64], x_len: u32, y: [u8; 64], y_len: u32, z: [u8; 64], z_len: u32) -> pub [u8; 192] {
    let mut vec: BoundedVec<u8, 192> = BoundedVec::new();
    let vec_x: BoundedVec<u8, 64> = BoundedVec::from_parts(x, x_len);
    let vec_y: BoundedVec<u8, 64> = BoundedVec::from_parts(y, y_len);
    let vec_z: BoundedVec<u8, 64> = BoundedVec::from_parts(z, z_len);
    vec.extend_from_bounded_vec(vec_x);
    vec.extend_from_bounded_vec(vec_y);
    vec.extend_from_bounded_vec(vec_z);
    vec.storage()
}
